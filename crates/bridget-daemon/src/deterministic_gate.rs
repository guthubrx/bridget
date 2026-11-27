//! Exécution isolée et bornée des portes déterministes (`convergence.gate`, Spec 314).
//!
//! Invariants sécurité et déterminisme :
//! - Aucun shell : `Command::new(&argv[0]).args(&argv[1..])` uniquement.
//! - Environnement nettoyé (`env_clear()`) : seuls quelques chemins système essentiels
//!   (`PATH`, `HOME`, `TMPDIR`, `LANG`, `LC_ALL`, `RUSTUP_HOME`, `CARGO_HOME`) sont
//!   recopiés, jamais de jeton, socket ou variable `BRIDGET_*`.
//! - `stdin` fermé (`Stdio::null()`).
//! - Vérification Git avant/après : `HEAD == expected_commit` et arbre propre avant,
//!   `HEAD` inchangé après — preuve que la porte teste le commit exact que verra `commit_view`.
//! - Timeout dur sur tout le groupe de processus Unix (`setpgid(0, 0)` + `kill(-pgid, SIGKILL)`).
//! - Lecture continue bornée (stdout + stderr) sur threads dédiés pour éviter tout deadlock
//!   sur pipe plein, sortie tronquée proprement en UTF-8 à `MAX_GATE_OUTPUT_BYTES` (8 Kio).

use bridget_transport::protocol::{
    GraphDeterministicGate, MAX_GATE_ARG_BYTES, MAX_GATE_ARGV, MAX_GATE_OUTPUT_BYTES,
    MAX_GATE_TIMEOUT_SECS, MIN_GATE_TIMEOUT_SECS,
};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Résultat fermé d'une exécution de porte déterministe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateExecOutput {
    pub passed: bool,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub duration_ms: u64,
    pub output_excerpt: String,
    /// Renseigné uniquement sur panne d'infrastructure / précondition Git (fail-closed).
    pub unavailable_reason: Option<String>,
}

/// Valide la forme d'une porte déterministe avant admission ou exécution.
pub fn validate_gate_spec(gate: &GraphDeterministicGate) -> Result<(), String> {
    if gate.argv.is_empty() || gate.argv.len() > MAX_GATE_ARGV {
        return Err(format!(
            "convergence.gate.argv doit contenir entre 1 et {MAX_GATE_ARGV} arguments."
        ));
    }
    if gate.argv[0].trim().is_empty() {
        return Err("convergence.gate.argv[0] (exécutable) ne peut pas être vide.".into());
    }
    let exe_base = Path::new(gate.argv[0].trim())
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if matches!(
        exe_base.as_str(),
        "sh" | "bash"
            | "zsh"
            | "dash"
            | "ksh"
            | "fish"
            | "csh"
            | "tcsh"
            | "cmd"
            | "cmd.exe"
            | "powershell"
            | "pwsh"
    ) {
        return Err(format!(
            "convergence.gate.argv[0] ne peut pas être un shell ('{}') : déclarez directement l'exécutable et ses arguments.",
            gate.argv[0]
        ));
    }
    for (idx, arg) in gate.argv.iter().enumerate() {
        if arg.len() > MAX_GATE_ARG_BYTES {
            return Err(format!(
                "convergence.gate.argv[{idx}] dépasse la borne de {MAX_GATE_ARG_BYTES} octets."
            ));
        }
        if arg.chars().any(bridget_core::is_disallowed_control) || arg.contains('\0') {
            return Err(format!(
                "convergence.gate.argv[{idx}] contient un caractère de contrôle interdit."
            ));
        }
    }
    if !(MIN_GATE_TIMEOUT_SECS..=MAX_GATE_TIMEOUT_SECS).contains(&gate.timeout_secs) {
        return Err(format!(
            "convergence.gate.timeout_secs doit être compris entre {MIN_GATE_TIMEOUT_SECS} et {MAX_GATE_TIMEOUT_SECS} secondes."
        ));
    }
    Ok(())
}

/// Empreinte normalisée pour la détection de stagnation/oscillation sur échec de porte.
/// Préfixée par `gate:{exit_code}:` pour distinguer les échecs déterministes des verdicts LLM,
/// en neutralisant les durées/nombres volatils (ex. `0.14s`, `128ms`, horodatages).
pub fn gate_failure_fingerprint(exit_code: Option<i32>, timed_out: bool, output: &str) -> String {
    let code_tag = if timed_out {
        "timeout".to_string()
    } else {
        exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "none".to_string())
    };
    let normalized = output
        .split_whitespace()
        .map(|tok| {
            let lower = tok.to_ascii_lowercase();
            let core =
                lower.trim_matches(|c: char| matches!(c, '(' | ')' | '[' | ']' | ',' | ';' | ':'));
            let is_duration = (core.ends_with("ms")
                && core.len() > 2
                && core[..core.len() - 2]
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.'))
                || (core.ends_with('s')
                    && core.len() > 1
                    && core[..core.len() - 1]
                        .chars()
                        .all(|c| c.is_ascii_digit() || c == '.'));
            if is_duration {
                "<dur>".to_string()
            } else {
                lower
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let payload = format!("gate:{code_tag}:{normalized}");
    format!("{:x}", Sha256::digest(payload.as_bytes()))
}

fn sanitize_env_for_gate(cmd: &mut Command) {
    cmd.env_clear();
    const ALLOWED_KEYS: &[&str] = &[
        "PATH",
        "HOME",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "RUSTUP_HOME",
        "CARGO_HOME",
        "CARGO_TARGET_DIR",
        "TERM",
    ];
    for key in ALLOWED_KEYS {
        if let Ok(val) = std::env::var(key)
            && !val.is_empty()
        {
            cmd.env(key, val);
        }
    }
    if std::env::var_os("PATH").is_none() {
        cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
    }
}

fn git_head_sha(dir: &Path) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .stdin(Stdio::null());
    sanitize_env_for_gate(&mut cmd);
    let output = cmd
        .output()
        .map_err(|e| format!("git_rev_parse_failed:{e}"))?;
    if !output.status.success() {
        return Err("git_head_unreadable".into());
    }
    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if sha.len() != 40 || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("git_head_invalid".into());
    }
    Ok(sha)
}

fn git_worktree_is_clean(dir: &Path) -> Result<bool, String> {
    let mut cmd = Command::new("git");
    cmd.args(["status", "--porcelain", "--untracked-files=normal"])
        .current_dir(dir)
        .stdin(Stdio::null());
    sanitize_env_for_gate(&mut cmd);
    let output = cmd.output().map_err(|e| format!("git_status_failed:{e}"))?;
    if !output.status.success() {
        return Err("git_status_unreadable".into());
    }
    Ok(output.stdout.iter().all(|b| b.is_ascii_whitespace()))
}

/// Lit un flux en continu jusqu'à EOF en conservant au plus `cap` octets,
/// afin qu'une commande bavarde ne bloque jamais sur un tube plein.
fn drain_bounded<R: Read>(mut reader: R, cap: usize) -> (Vec<u8>, bool) {
    let mut kept = Vec::with_capacity(cap.min(4096));
    let mut truncated = false;
    let mut buf = [0u8; 4096];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if kept.len() < cap {
                    let take = (cap - kept.len()).min(n);
                    kept.extend_from_slice(&buf[..take]);
                    if take < n {
                        truncated = true;
                    }
                } else {
                    truncated = true;
                }
            }
            Err(_) => break,
        }
    }
    (kept, truncated)
}

fn format_bounded_output(stdout: &[u8], stderr: &[u8], truncated: bool) -> String {
    let out_s = String::from_utf8_lossy(stdout);
    let err_s = String::from_utf8_lossy(stderr);
    let mut combined = String::new();
    if !out_s.trim().is_empty() {
        combined.push_str(out_s.trim_end());
    }
    if !err_s.trim().is_empty() {
        if !combined.is_empty() {
            combined.push_str("\n--- stderr ---\n");
        }
        combined.push_str(err_s.trim_end());
    }
    let suffix = if truncated {
        "\n[sortie tronquée]"
    } else {
        ""
    };
    let budget = MAX_GATE_OUTPUT_BYTES.saturating_sub(suffix.len());
    if combined.len() > budget {
        let mut end = budget;
        while end > 0 && !combined.is_char_boundary(end) {
            end -= 1;
        }
        combined.truncate(end);
        combined.push_str("\n[sortie tronquée]");
    } else if truncated {
        combined.push_str(suffix);
    }
    combined
}

/// Exécute une porte déterministe `argv` dans `dir` après vérification de `expected_commit`
/// et d'un arbre Git propre, avec environnement nettoyé, groupe de processus Unix isolé
/// et timeout dur.
pub fn execute_gate_in_dir(
    dir: &Path,
    expected_commit: &str,
    argv: &[String],
    timeout_secs: u64,
) -> GateExecOutput {
    let gate = GraphDeterministicGate {
        argv: argv.to_vec(),
        timeout_secs,
    };
    if let Err(reason) = validate_gate_spec(&gate) {
        return GateExecOutput {
            passed: false,
            exit_code: None,
            timed_out: false,
            duration_ms: 0,
            output_excerpt: String::new(),
            unavailable_reason: Some(format!("gate_spec_invalid:{reason}")),
        };
    }
    if !dir.is_dir() {
        return GateExecOutput {
            passed: false,
            exit_code: None,
            timed_out: false,
            duration_ms: 0,
            output_excerpt: String::new(),
            unavailable_reason: Some("gate_worktree_missing".into()),
        };
    }
    match git_head_sha(dir) {
        Ok(head) if head.eq_ignore_ascii_case(expected_commit) => {}
        Ok(head) => {
            return GateExecOutput {
                passed: false,
                exit_code: None,
                timed_out: false,
                duration_ms: 0,
                output_excerpt: String::new(),
                unavailable_reason: Some(format!(
                    "gate_head_mismatch: attendu {expected_commit}, observé {head}"
                )),
            };
        }
        Err(reason) => {
            return GateExecOutput {
                passed: false,
                exit_code: None,
                timed_out: false,
                duration_ms: 0,
                output_excerpt: String::new(),
                unavailable_reason: Some(reason),
            };
        }
    }
    match git_worktree_is_clean(dir) {
        Ok(true) => {}
        Ok(false) => {
            return GateExecOutput {
                passed: false,
                exit_code: None,
                timed_out: false,
                duration_ms: 0,
                output_excerpt: String::new(),
                unavailable_reason: Some(
                    "gate_worktree_dirty: modifications non commitées avant la porte déterministe"
                        .into(),
                ),
            };
        }
        Err(reason) => {
            return GateExecOutput {
                passed: false,
                exit_code: None,
                timed_out: false,
                duration_ms: 0,
                output_excerpt: String::new(),
                unavailable_reason: Some(reason),
            };
        }
    }

    let started = Instant::now();
    let mut cmd = Command::new(&argv[0]);
    cmd.args(&argv[1..])
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    sanitize_env_for_gate(&mut cmd);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: `setpgid(0, 0)` est async-signal-safe (POSIX) et isole la commande et
        // ses petits-enfants dans un groupe dédié afin de garantir un arrêt propre au timeout.
        unsafe {
            cmd.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(error) => {
            return GateExecOutput {
                passed: false,
                exit_code: None,
                timed_out: false,
                duration_ms: started.elapsed().as_millis() as u64,
                output_excerpt: String::new(),
                unavailable_reason: Some(format!("gate_spawn_failed:{error}")),
            };
        }
    };

    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let (out_tx, out_rx) = std::sync::mpsc::channel();
    let _out_handle = std::thread::spawn(move || {
        let res = stdout_pipe
            .map(|p| drain_bounded(p, MAX_GATE_OUTPUT_BYTES))
            .unwrap_or_else(|| (Vec::new(), false));
        let _ = out_tx.send(res);
    });
    let (err_tx, err_rx) = std::sync::mpsc::channel();
    let _err_handle = std::thread::spawn(move || {
        let res = stderr_pipe
            .map(|p| drain_bounded(p, MAX_GATE_OUTPUT_BYTES))
            .unwrap_or_else(|| (Vec::new(), false));
        let _ = err_tx.send(res);
    });

    let child_pid = child.id();
    let kill_group = || {
        #[cfg(unix)]
        {
            let pgid = child_pid as libc::pid_t;
            // SAFETY: envoi de SIGKILL au groupe isolé créé par pre_exec.
            unsafe {
                libc::kill(-pgid, libc::SIGKILL);
            }
        }
    };

    let deadline = started + Duration::from_secs(timeout_secs);
    let mut timed_out = false;
    let mut wait_error: Option<String> = None;
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => {
                // Nettoyer aussi d'éventuels petits-enfants ayant hérité de stdout/stderr.
                kill_group();
                break Some(st);
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    timed_out = true;
                    kill_group();
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => {
                wait_error = Some(format!("gate_wait_failed:{error}"));
                kill_group();
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };

    let pipe_grace = Duration::from_secs(2);
    let (stdout_bytes, out_trunc) = out_rx.recv_timeout(pipe_grace).unwrap_or_default();
    let (stderr_bytes, err_trunc) = err_rx.recv_timeout(pipe_grace).unwrap_or_default();
    let duration_ms = started.elapsed().as_millis() as u64;
    let output_excerpt =
        format_bounded_output(&stdout_bytes, &stderr_bytes, out_trunc || err_trunc);

    if let Some(reason) = wait_error {
        return GateExecOutput {
            passed: false,
            exit_code: None,
            timed_out,
            duration_ms,
            output_excerpt,
            unavailable_reason: Some(reason),
        };
    }

    // Vérifier que la porte n'a altéré ni HEAD ni l'arbre de travail pendant son exécution.
    match git_head_sha(dir) {
        Ok(head_after) if head_after.eq_ignore_ascii_case(expected_commit) => {}
        Ok(head_after) => {
            return GateExecOutput {
                passed: false,
                exit_code: status.and_then(|s| s.code()),
                timed_out,
                duration_ms,
                output_excerpt,
                unavailable_reason: Some(format!(
                    "gate_head_mutated: attendu {expected_commit}, observé {head_after}"
                )),
            };
        }
        Err(reason) => {
            return GateExecOutput {
                passed: false,
                exit_code: status.and_then(|s| s.code()),
                timed_out,
                duration_ms,
                output_excerpt,
                unavailable_reason: Some(reason),
            };
        }
    }
    match git_worktree_is_clean(dir) {
        Ok(true) => {}
        Ok(false) => {
            return GateExecOutput {
                passed: false,
                exit_code: status.and_then(|s| s.code()),
                timed_out,
                duration_ms,
                output_excerpt,
                unavailable_reason: Some(
                    "gate_worktree_mutated: la porte déterministe a modifié le worktree pendant son exécution"
                        .into(),
                ),
            };
        }
        Err(reason) => {
            return GateExecOutput {
                passed: false,
                exit_code: status.and_then(|s| s.code()),
                timed_out,
                duration_ms,
                output_excerpt,
                unavailable_reason: Some(reason),
            };
        }
    }

    let exit_code = status.and_then(|s| s.code());
    let passed = !timed_out && exit_code == Some(0);
    GateExecOutput {
        passed,
        exit_code,
        timed_out,
        duration_ms,
        output_excerpt,
        unavailable_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDirGuard(std::path::PathBuf);
    impl TempDirGuard {
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn init_temp_git_repo() -> (TempDirGuard, String) {
        let dir = TempDirGuard(std::env::temp_dir().join(format!(
            "bridget-gate-test-{}",
            uuid::Uuid::new_v4().simple()
        )));
        std::fs::create_dir_all(dir.path()).unwrap();
        let run = |args: &[&str]| {
            let st = Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .status()
                .unwrap();
            assert!(st.success(), "git {args:?} failed");
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "test@bridget.local"]);
        run(&["config", "user.name", "Bridget Test"]);
        std::fs::write(dir.path().join("README.md"), "hello\n").unwrap();
        run(&["add", "README.md"]);
        run(&["commit", "-q", "-m", "init"]);
        let sha = git_head_sha(dir.path()).unwrap();
        (dir, sha)
    }

    #[test]
    fn spec314_gate_pass_and_fail_on_clean_commit() {
        let (dir, sha) = init_temp_git_repo();
        let ok = execute_gate_in_dir(
            dir.path(),
            &sha,
            &["/usr/bin/env".into(), "true".into()],
            10,
        );
        assert!(ok.passed, "{ok:?}");
        assert_eq!(ok.exit_code, Some(0));
        assert!(ok.unavailable_reason.is_none());

        let fail = execute_gate_in_dir(
            dir.path(),
            &sha,
            &[
                "python3".into(),
                "-c".into(),
                "import sys; sys.stderr.write('fail_msg\\n'); sys.exit(2)".into(),
            ],
            10,
        );
        assert!(!fail.passed);
        assert_eq!(fail.exit_code, Some(2));
        assert!(fail.output_excerpt.contains("fail_msg"));
        assert!(fail.unavailable_reason.is_none());

        // Un shell explicite dans argv[0] est refusé fail-closed.
        let shell_refused = execute_gate_in_dir(
            dir.path(),
            &sha,
            &["/bin/sh".into(), "-c".into(), "true".into()],
            10,
        );
        assert!(!shell_refused.passed);
        assert!(
            shell_refused
                .unavailable_reason
                .as_deref()
                .is_some_and(|r| r.contains("shell"))
        );
    }

    #[test]
    fn spec314_gate_refuses_dirty_worktree_and_mismatched_head() {
        let (dir, sha) = init_temp_git_repo();
        let bad_sha = "0".repeat(40);
        let mismatch = execute_gate_in_dir(dir.path(), &bad_sha, &["/usr/bin/true".into()], 10);
        assert!(!mismatch.passed);
        assert!(
            mismatch
                .unavailable_reason
                .as_deref()
                .is_some_and(|r| r.starts_with("gate_head_mismatch"))
        );

        std::fs::write(dir.path().join("dirty.txt"), "uncommitted").unwrap();
        let dirty = execute_gate_in_dir(dir.path(), &sha, &["/usr/bin/true".into()], 10);
        assert!(!dirty.passed);
        assert!(
            dirty
                .unavailable_reason
                .as_deref()
                .is_some_and(|r| r.starts_with("gate_worktree_dirty"))
        );
    }

    #[test]
    fn spec314_gate_strips_bridget_env_and_kills_process_group_on_timeout() {
        let (dir, sha) = init_temp_git_repo();
        let env_check = execute_gate_in_dir(
            dir.path(),
            &sha,
            &[
                "python3".into(),
                "-c".into(),
                "import os, sys; bad=[k for k in os.environ if k.startswith('BRIDGET_')]; sys.exit(0 if not bad else 3)".into(),
            ],
            10,
        );
        assert!(env_check.passed, "{env_check:?}");

        let res = execute_gate_in_dir(
            dir.path(),
            &sha,
            &[
                "python3".into(),
                "-c".into(),
                "import subprocess, time; subprocess.Popen(['sleep', '30']); time.sleep(30)".into(),
            ],
            5,
        );
        assert!(!res.passed);
        assert!(res.timed_out, "{res:?}");
        assert!(
            res.duration_ms < 8000,
            "timeout must kill child group promptly: {}ms",
            res.duration_ms
        );

        // P1-2 : petit-enfant conservant stdout ouvert après sortie 0 du parent -> aucun blocage.
        let orphan_pipe = execute_gate_in_dir(
            dir.path(),
            &sha,
            &[
                "python3".into(),
                "-c".into(),
                "import subprocess; subprocess.Popen(['sleep', '30'])".into(),
            ],
            10,
        );
        assert!(orphan_pipe.passed, "{orphan_pipe:?}");
        assert!(
            orphan_pipe.duration_ms < 4000,
            "grandchild holding stdout must be killed promptly: {}ms",
            orphan_pipe.duration_ms
        );

        // P1-1 : une porte qui modifie un fichier suivi échoue en fail-closed (gate_worktree_mutated).
        let mutated = execute_gate_in_dir(
            dir.path(),
            &sha,
            &[
                "python3".into(),
                "-c".into(),
                "open('README.md', 'a').write('mutated\\n')".into(),
            ],
            10,
        );
        assert!(!mutated.passed);
        assert!(
            mutated
                .unavailable_reason
                .as_deref()
                .is_some_and(|r| r.starts_with("gate_worktree_mutated")),
            "{mutated:?}"
        );
    }

    #[test]
    fn spec314_gate_fingerprint_normalizes_durations_and_prefixes_gate() {
        let a = gate_failure_fingerprint(Some(1), false, "FAIL test_x in 0.14s (12ms)");
        let b = gate_failure_fingerprint(Some(1), false, "fail   test_x in 2.91s (450ms)");
        let c = gate_failure_fingerprint(Some(2), false, "fail test_x in 2.91s (450ms)");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
