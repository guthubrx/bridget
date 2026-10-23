use super::*;
use std::os::unix::fs::PermissionsExt;

// Même recette que wrapper::prompt_tests : PATH privé dans un fils de test,
// aucune mutation de l'environnement commun de la suite.
fn isolated(case: &str, name: &str, check: impl FnOnce(&Path)) {
    const CHILD: &str = "BRIDGET_GIT154_TEST_ROOT";
    if let Some(root) = std::env::var_os(CHILD) {
        check(Path::new(&root));
        return;
    }
    let root = std::env::temp_dir().join(format!("git154-{}", uuid::Uuid::new_v4().simple()));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    for directory in ["bin", "home", "tmp", "repo"] {
        fs::create_dir(root.join(directory)).unwrap();
    }
    let real_git = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|directory| directory.join("git"))
        .find(|path| path.is_file())
        .unwrap();
    let version = if case == "old" { "2.43.0" } else { "2.55.0" };
    let action = match case {
        "old" => format!("exec '{}' \"$@\"", real_git.display()),
        "env" => "printf 'global=%s\\nsystem=%s\\nlazy=%s\\nallow=%s\\nDIR=%s\\nTREE=%s\\nCOMMON=%s\\n' \"$GIT_CONFIG_GLOBAL\" \"$GIT_CONFIG_NOSYSTEM\" \"$GIT_NO_LAZY_FETCH\" \"$GIT_ALLOW_PROTOCOL\" \"${GIT_DIR-unset}\" \"${GIT_WORK_TREE-unset}\" \"${GIT_COMMON_DIR-unset}\"; printf 'arg=%s\\n' \"$@\"".into(),
        "large" => "/usr/bin/awk 'BEGIN { for (i=0;i<262144;i++) { print \"abcdefgh\"; print \"ijklmnop\" > \"/dev/stderr\" } }'".into(),
        "slow" => format!("printf '%s' \"$$\" > '{}/git.pid'; exec /bin/sleep 2", root.display()),
        "disk-full" => format!(r#"previous=
repository=
branch=
checkout=no
for argument in "$@"; do
 [ "$previous" = -C ] && repository=$argument
 [ "$previous" = -b ] && branch=$argument
 [ "$previous" = worktree ] && [ "$argument" = add ] && checkout=yes
 previous=$argument
done
if [ "$checkout" = yes ]; then
 '{}' -c core.hooksPath=/dev/null -C "$repository" branch "$branch" HEAD || exit 1
 printf '%s' "$branch" > "$BRIDGET_GIT154_TEST_ROOT/branch-created"
 printf '%s\n' 'fatal: unable to write a file: No space left on device' >&2
 exit 128
fi
exec '{}' "$@""#, real_git.display(), real_git.display()),
        "fetch-full" => format!(r#"for argument in "$@"; do
 if [ "$argument" = fetch ]; then
  printf '%s\n' 'fatal: unable to create temporary file: No space left on device' >&2
  exit 128
 fi
done
exec '{}' "$@""#, real_git.display()),
        "large-out-no-space" => "/usr/bin/awk 'BEGIN { for (i=0;i<262144;i++) print \"abcdefgh\" }'; printf '%s\n' 'fatal: No space left on device' >&2; exit 128".into(),
        "forbidden" => format!("printf launched > '{}/launched'", root.display()),
        _ => panic!("fixture inconnue"),
    };
    let script = root.join("bin/git");
    fs::write(&script, format!("#!/bin/sh\nfor argument in \"$@\"; do\n if [ \"$argument\" = --version ]; then printf 'git version {version}\\n'; exit 0; fi\ndone\n{action}\n")).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    let log_path = root.join("child.log");
    let log = fs::File::create(&log_path).unwrap();
    let executable = std::env::current_exe().unwrap();
    let mut child = Command::new(&executable)
        .args(["--exact", name, "--nocapture"])
        .env_clear()
        .env("HOME", root.join("home"))
        .env("TMPDIR", root.join("tmp"))
        .env("PATH", root.join("bin"))
        .env(CHILD, &root)
        .env("GIT_DIR", "/not-the-repository")
        .env("GIT_WORK_TREE", "/not-the-worktree")
        .env("GIT_COMMON_DIR", "/not-the-common-dir")
        .env("GIT_EXTERNAL_DIFF", "/not-a-program")
        .stdin(Stdio::null())
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    let result = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if Instant::now() >= deadline {
            let observed = Command::new("/bin/ps")
                .args(["-p", &child.id().to_string(), "-o", "command="])
                .output()
                .unwrap();
            assert!(
                String::from_utf8_lossy(&observed.stdout)
                    .trim()
                    .starts_with(&executable.display().to_string())
            );
            assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }, 0);
            let stop = Instant::now() + Duration::from_secs(3);
            while matches!(child.try_wait(), Ok(None)) && Instant::now() < stop {
                std::thread::sleep(Duration::from_millis(5));
            }
            break None;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let diagnostic = fs::read_to_string(&log_path).unwrap();
    fs::remove_dir_all(&root).unwrap();
    assert!(
        result.is_some_and(|status| status.success()),
        "Git {case}, {result:?} : {diagnostic}"
    );
}

#[test]
fn native154_git_version_ancienne_refusee_a_l_avant_vol() {
    isolated(
        "old",
        "workspace_git::tests::native154_git_version_ancienne_refusee_a_l_avant_vol",
        |root| {
            let repo = root.join("repo");
            // Le faux Git ne change que --version ; le dépôt est réel.
            for args in [
                &["init", "-q"][..],
                &["add", "."][..],
                &[
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "-c",
                    "commit.gpgSign=false",
                    "commit",
                    "-qm",
                    "initial",
                ][..],
            ] {
                if args.first() == Some(&"add") {
                    fs::write(repo.join(".gitignore"), ".worktrees/\n").unwrap();
                    fs::write(repo.join("tracked"), "initial\n").unwrap();
                }
                let mut git = Command::new("git");
                for variable in [
                    "GIT_DIR",
                    "GIT_WORK_TREE",
                    "GIT_COMMON_DIR",
                    "GIT_EXTERNAL_DIFF",
                ] {
                    git.env_remove(variable);
                }
                assert!(
                    git.env("GIT_CONFIG_GLOBAL", "/dev/null")
                        .env("GIT_CONFIG_NOSYSTEM", "1")
                        .arg("-C")
                        .arg(&repo)
                        .args(args)
                        .status()
                        .unwrap()
                        .success()
                );
            }
            let result = preflight(&repo, None, None);
            assert!(
                matches!(result, Err(ref error) if error.starts_with("workspace_git_unsupported:")),
                "Git 2.43 doit être refusé : {result:?}"
            );
        },
    );
}

#[test]
fn native154_git_environnement_et_options_effectives() {
    isolated(
        "env",
        "workspace_git::tests::native154_git_environnement_et_options_effectives",
        |root| {
            let commits = format!("{}..{}", "1".repeat(40), "2".repeat(40));
            let output = run_git(
                &root.join("repo"),
                &[
                    "log",
                    "--max-count=50",
                    "--format=%H%x1f%s",
                    "--end-of-options",
                    &commits,
                ],
                READ_TIMEOUT,
            )
            .unwrap();
            assert!(output.success());
            for expected in [
                "global=/dev/null",
                "system=1",
                "lazy=1",
                "allow=\n",
                "DIR=unset",
                "TREE=unset",
                "COMMON=unset",
                "arg=--no-pager",
                "arg=core.hooksPath=/dev/null",
                "arg=core.fsmonitor=false",
                "arg=protocol.allow=never",
                "arg=log.showSignature=false",
                "arg=--no-show-signature",
                "arg=--no-textconv",
                "arg=--no-ext-diff",
            ] {
                assert!(
                    output.stdout.contains(expected),
                    "protection absente {expected}: {}",
                    output.stdout
                );
            }
        },
    );
}

#[test]
fn native154_git_sorties_cumulees_bornees() {
    isolated(
        "large",
        "workspace_git::tests::native154_git_sorties_cumulees_bornees",
        |root| {
            let output = run_git(
                &root.join("repo"),
                &["status", "--porcelain=v1", "--untracked-files=normal"],
                READ_TIMEOUT,
            )
            .unwrap();
            assert!(output.success());
            assert!(output.truncated);
            assert_eq!(output.stdout.len(), MAX_OUTPUT);
            assert_eq!(output.stderr.len(), MAX_OUTPUT);
            assert!(matches!(
                preflight_classified(&root.join("repo"), None, None),
                Err(WorkspacePreflightError::Technical(error))
                    if error.contains("sortie Git tronquée")
            ));
        },
    );
}

#[test]
fn native154_git_delai_termine_son_enfant() {
    isolated(
        "slow",
        "workspace_git::tests::native154_git_delai_termine_son_enfant",
        |root| {
            let started = Instant::now();
            let error = run_git(
                &root.join("repo"),
                &["status", "--porcelain=v1", "--untracked-files=normal"],
                Duration::from_millis(100),
            )
            .unwrap_err();
            assert!(error.contains("délai Git dépassé"));
            assert!(started.elapsed() < Duration::from_secs(1));
            let pid = fs::read_to_string(root.join("git.pid"))
                .unwrap()
                .parse::<i32>()
                .unwrap();
            assert_eq!(
                unsafe { libc::kill(pid, 0) },
                -1,
                "le seul enfant Git lancé doit être terminé/récolté"
            );
            assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
        },
    );
}

#[test]
fn native154_git_formes_interdites_ne_lancent_rien() {
    isolated(
        "forbidden",
        "workspace_git::tests::native154_git_formes_interdites_ne_lancent_rien",
        |root| {
            for args in [
                &["worktree", "remove", "x"][..],
                &["branch", "-D", "x"][..],
                &["update-ref", "-d", "x"][..],
                &["clean", "-fd"][..],
                &["fetch"][..],
                &["log", "--output=/tmp/foreign"][..],
                &["config", "user.name", "foreign"][..],
                &["-C", "/tmp", "status"][..],
                &["rev-parse", "--git-dir"][..],
                &["worktree", "add", "--force", "x"][..],
            ] {
                assert!(
                    run_git(&root.join("repo"), args, READ_TIMEOUT).is_err(),
                    "forme acceptée : {args:?}"
                );
            }
            assert!(!root.join("launched").exists());
        },
    );
}

#[test]
fn native154_git_noms_canoniques_sans_option() {
    for invalid in [
        "../x",
        "-x",
        "a@{x",
        "a.lock",
        "a b",
        "a//b",
        "a/",
        "a/.hidden",
        "a/hidden.lock",
        "a.",
    ] {
        assert!(validate_branch(invalid).is_err(), "branche {invalid}");
    }
    assert!(validate_branch(&"a".repeat(101)).is_err());
    assert!(validate_branch("bridget/task-1234").is_ok());
    for invalid in [
        "--upload-pack=x",
        "",
        "has space",
        "control\n",
        "bidi\u{202e}",
    ] {
        assert!(validate_base_ref(invalid).is_err());
    }
    assert!(validate_base_ref(&"a".repeat(201)).is_err());
    assert!(validate_base_ref("HEAD~1").is_ok());
}

// Fixture réelle sous le TMPDIR privé du principal. Les commandes d'écriture
// appartiennent seulement aux tests, jamais à l'exécuteur de production.
fn fixture_git(repo: &Path, args: &[&str]) {
    let mut git = Command::new("git");
    for (name, _) in
        std::env::vars_os().filter(|(name, _)| name.to_string_lossy().starts_with("GIT_"))
    {
        git.env_remove(name);
    }
    assert!(
        git.env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgSign=false"
            ])
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .unwrap()
            .success(),
        "Git fixture : {args:?}"
    );
}

fn with_repo(check: impl FnOnce(&Path)) {
    let root = std::env::temp_dir().join(format!("git154-real-{}", uuid::Uuid::new_v4().simple()));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let root = root.canonicalize().unwrap();
    let repo = root.join("repo");
    fs::create_dir(&repo).unwrap();
    fixture_git(&repo, &["init", "-q"]);
    fs::create_dir(repo.join("sub")).unwrap();
    fs::write(repo.join(".gitignore"), ".worktrees/\n").unwrap();
    fs::write(repo.join("sub/tracked"), "initial\n").unwrap();
    fixture_git(&repo, &["add", "."]);
    fixture_git(&repo, &["commit", "-qm", "initial"]);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check(&repo)));
    fs::remove_dir_all(root).unwrap();
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}

#[test]
fn native154_git_hook_post_checkout_et_gpg_ne_sont_jamais_lances() {
    with_repo(|repo| {
        let hook_marker = repo.join("hook-executed");
        let hook = repo.join(".git/hooks/post-checkout");
        fs::write(
            &hook,
            format!("#!/bin/sh\nprintf ran > '{}'\n", hook_marker.display()),
        )
        .unwrap();
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
        let control = repo.parent().unwrap().join("hook-control");
        fixture_git(
            repo,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "hook-control",
                control.to_str().unwrap(),
            ],
        );
        assert!(
            hook_marker.exists(),
            "le contrôle positif doit prouver le hook actif"
        );
        fs::remove_file(&hook_marker).unwrap();
        let gpg_marker = repo.join("gpg-executed");
        let gpg = repo.join("gpg.sh");
        fs::write(
            &gpg,
            format!(
                "#!/bin/sh\nprintf ran > '{}'\nexit 1\n",
                gpg_marker.display()
            ),
        )
        .unwrap();
        fs::set_permissions(&gpg, fs::Permissions::from_mode(0o700)).unwrap();
        fixture_git(repo, &["config", "gpg.program", gpg.to_str().unwrap()]);
        fixture_git(repo, &["config", "log.showSignature", "true"]);
        let plan = preflight(repo, None, None).unwrap();
        let record = create(&plan, &uuid::Uuid::new_v4().to_string()).unwrap();
        // En-tête gpgsig synthétique réel : sans --no-show-signature, git log
        // demande une vérification au programme choisi par le dépôt.
        let head = run_git(
            repo,
            &["rev-parse", "--verify", "--end-of-options", "HEAD"],
            READ_TIMEOUT,
        )
        .unwrap()
        .complete()
        .unwrap();
        let raw = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["cat-file", "commit", &head])
            .output()
            .unwrap();
        assert!(raw.status.success());
        let text = String::from_utf8(raw.stdout).unwrap();
        let (headers, body) = text.split_once("\n\n").unwrap();
        let signed = format!(
            "{headers}\ngpgsig -----BEGIN PGP SIGNATURE-----\n fake\n -----END PGP SIGNATURE-----\n\n{body}"
        );
        let mut writer = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["hash-object", "-t", "commit", "-w", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        writer
            .stdin
            .take()
            .unwrap()
            .write_all(signed.as_bytes())
            .unwrap();
        let signed = writer.wait_with_output().unwrap();
        assert!(signed.status.success());
        let signed = String::from_utf8(signed.stdout).unwrap().trim().to_string();
        fixture_git(Path::new(&record.path), &["update-ref", "HEAD", &signed]);
        // Contrôle positif : le faux programme GPG écrit bien son marqueur
        // quand Git reçoit explicitement --show-signature sur cet objet.
        Command::new("git")
            .arg("--no-pager")
            .arg("-C")
            .arg(&record.path)
            .args(["log", "--show-signature", "--max-count=1", &signed])
            .status()
            .unwrap();
        assert!(
            gpg_marker.exists(),
            "le contrôle positif doit prouver gpg.program actif"
        );
        fs::remove_file(&gpg_marker).unwrap();
        let range = format!("{}..{}", plan.base_commit, signed);
        let output = run_git(
            Path::new(&record.path),
            &[
                "log",
                "--max-count=50",
                "--format=%H%x1f%s",
                "--end-of-options",
                &range,
            ],
            READ_TIMEOUT,
        )
        .unwrap();
        assert!(output.success());
        assert!(
            output.stdout.starts_with(&signed),
            "le log doit lire le commit signé réel"
        );
        assert!(!hook_marker.exists(), "post-checkout a été exécuté");
        assert!(!gpg_marker.exists(), "gpg.program a été exécuté");
    });
}

#[test]
fn native154_git_creation_depuis_sous_dossier_et_worktree_emboite() {
    with_repo(|repo| {
        assert!(matches!(
            preflight_classified(repo, Some("base-r8-inconnue"), None),
            Err(WorkspacePreflightError::Refused(error))
                if error == "workspace_base_unavailable: référence de base inconnue"
        ));
        let before = run_git(
            repo,
            &["rev-parse", "--verify", "--end-of-options", "HEAD"],
            READ_TIMEOUT,
        )
        .unwrap()
        .complete()
        .unwrap();
        let plan = preflight(&repo.join("sub"), None, None).unwrap();
        assert_eq!(plan.offset, PathBuf::from("sub"));
        let first = create(&plan, &uuid::Uuid::new_v4().to_string()).unwrap();
        assert!(Path::new(&first.path).starts_with(repo.join("sub/.worktrees")));
        assert!(
            Path::new(&first.path)
                .join(&plan.offset)
                .join("tracked")
                .is_file()
        );
        let nested_plan =
            preflight(&Path::new(&first.path).join(&plan.offset), None, None).unwrap();
        let nested = create(&nested_plan, &uuid::Uuid::new_v4().to_string()).unwrap();
        assert!(Path::new(&nested.path).starts_with(&first.path));
        assert_eq!(
            report(&nested).unwrap().head_commit.as_deref(),
            Some(before.as_str())
        );
        let after = run_git(
            repo,
            &["rev-parse", "--verify", "--end-of-options", "HEAD"],
            READ_TIMEOUT,
        )
        .unwrap()
        .complete()
        .unwrap();
        assert_eq!(before, after);
    });
}

#[test]
fn native154_git_rapport_zero_deux_soixante_commits_et_tete_detachee() {
    with_repo(|repo| {
        let record = create(
            &preflight(repo, None, None).unwrap(),
            &uuid::Uuid::new_v4().to_string(),
        )
        .unwrap();
        let worktree = Path::new(&record.path);
        let zero = report(&record).unwrap();
        assert_eq!(zero.commit_count, Some(0));
        assert_eq!(zero.dirty, Some(false));
        assert_eq!(zero.head_on_branch, Some(true));
        assert_eq!(zero.state, WorkspaceState::Kept);
        for index in 0..60 {
            let subject = if index == 0 {
                format!("contrôle\u{0007}\u{202e} {}", "é".repeat(230))
            } else {
                format!("commit {index}")
            };
            fixture_git(worktree, &["commit", "--allow-empty", "-qm", &subject]);
            if index == 1 {
                let two = report(&record).unwrap();
                assert_eq!(two.commit_count, Some(2));
                assert_eq!(two.commits.len(), 2);
                assert!(two.commits[1].subject.chars().count() <= 200);
                assert!(!two.commits[1].subject.chars().any(is_disallowed_control));
                assert!(!two.truncated);
            }
        }
        let sixty = report(&record).unwrap();
        assert_eq!(sixty.commit_count, Some(60));
        assert_eq!(sixty.commits.len(), 50);
        assert!(sixty.truncated);
        fixture_git(worktree, &["config", "status.showUntrackedFiles", "no"]);
        fs::write(worktree.join("untracked"), "kept\n").unwrap();
        assert_eq!(report(&record).unwrap().dirty, Some(true));
        fixture_git(worktree, &["checkout", "--detach", "-q"]);
        fixture_git(worktree, &["commit", "--allow-empty", "-qm", "detached"]);
        let detached = report(&record).unwrap();
        assert_eq!(detached.head_on_branch, Some(false));
        assert_ne!(detached.head_commit, detached.branch_commit);
        fixture_git(worktree, &["checkout", "-b", "another-branch", "-q"]);
        let changed_branch = report(&record).unwrap();
        assert_eq!(changed_branch.head_on_branch, Some(false));
        assert_eq!(changed_branch.branch_commit, sixty.branch_commit);
        assert_eq!(changed_branch.head_commit, detached.head_commit);
        assert_ne!(changed_branch.head_commit, changed_branch.branch_commit);
    });
}

#[test]
fn native154_git_rapport_ignore_gitfile_enfant_et_parent_emboite() {
    with_repo(|repo| {
        let record = create(
            &preflight(repo, None, None).unwrap(),
            &uuid::Uuid::new_v4().to_string(),
        )
        .unwrap();
        let worktree = Path::new(&record.path);
        fixture_git(worktree, &["commit", "--allow-empty", "-qm", "child head"]);
        let expected = report(&record).unwrap().head_commit.unwrap();
        let nested = create(
            &preflight(worktree, None, None).unwrap(),
            &uuid::Uuid::new_v4().to_string(),
        )
        .unwrap();
        fixture_git(
            Path::new(&nested.path),
            &["commit", "--allow-empty", "-qm", "nested head"],
        );
        let nested_expected = report(&nested).unwrap().head_commit.unwrap();
        // Les deux .git sont désormais mensongers. Le registre du principal
        // reste autorité ; aucune lecture ne doit suivre ces fichiers.
        fs::write(
            worktree.join(".git"),
            format!("gitdir: {}\n", repo.join(".git").display()),
        )
        .unwrap();
        fs::write(
            Path::new(&nested.path).join(".git"),
            format!("gitdir: {}\n", repo.join(".git").display()),
        )
        .unwrap();
        assert_eq!(
            report(&record).unwrap().head_commit.as_deref(),
            Some(expected.as_str())
        );
        assert_eq!(
            report(&nested).unwrap().head_commit.as_deref(),
            Some(nested_expected.as_str())
        );

        // Choisir la cible en seconde position RÉELLE, sans hypothèse de tri
        // des noms. Les deux entrées administratives du registre restent
        // ensuite inchangées ; seuls leurs fichiers internes sont modifiés.
        let entries = fs::read_dir(repo.join(".git/worktrees"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 2, "fixture : deux worktrees enregistrés");
        let neighbor_gitdir = entries[0].join("gitdir");
        let target_gitdir = entries[1].join("gitdir");
        let neighbor_original = fs::read(&neighbor_gitdir).unwrap();
        let target_original = fs::read(&target_gitdir).unwrap();
        let target_gitfile = PathBuf::from(
            String::from_utf8(target_original.clone())
                .unwrap()
                .trim_end(),
        );
        let (target, target_head) = if target_gitfile == Path::new(&record.path).join(".git") {
            (&record, expected.as_str())
        } else {
            assert_eq!(target_gitfile, Path::new(&nested.path).join(".git"));
            (&nested, nested_expected.as_str())
        };
        fs::remove_file(&neighbor_gitdir).unwrap();
        assert_eq!(
            report(target)
                .expect("voisin administratif incomplet ignoré")
                .head_commit
                .as_deref(),
            Some(target_head)
        );
        std::os::unix::fs::symlink(&target_gitdir, &neighbor_gitdir).unwrap();
        assert_eq!(
            report(target)
                .expect("gitdir symbolique voisin ignoré")
                .head_commit
                .as_deref(),
            Some(target_head)
        );
        fs::remove_file(&neighbor_gitdir).unwrap();
        fs::write(&neighbor_gitdir, [0xff]).unwrap();
        assert_eq!(
            report(target)
                .expect("gitdir non UTF-8 voisin ignoré")
                .head_commit
                .as_deref(),
            Some(target_head)
        );
        fs::write(&neighbor_gitdir, &neighbor_original).unwrap();
        fs::remove_file(&target_gitdir).unwrap();
        assert!(
            report(target).is_err(),
            "la cible sans ancre vérifiable doit rester refusée"
        );
        fs::write(&target_gitdir, &target_original).unwrap();
    });
}

#[test]
fn worktree171_branch_mirror_import_isolation_and_refusals() {
    with_repo(|repo| {
        let base = resolve_source_commit(repo, None).unwrap();
        let root = repo.parent().unwrap();
        let mirror = root.join("mirror.git");
        let target = root.join("target");
        fixture_git(repo, &["clone", "--bare", "--quiet", repo.to_str().unwrap(), mirror.to_str().unwrap()]);
        fixture_git(repo, &["clone", "--quiet", mirror.to_str().unwrap(), target.to_str().unwrap()]);
        fixture_git(repo, &["checkout", "-qb", "feature171"]);
        fs::write(repo.join("sub/tracked"), "feature171\n").unwrap();
        fixture_git(repo, &["commit", "-qam", "feature"]);
        let feature = resolve_source_commit(repo, None).unwrap();
        fixture_git(repo, &["push", "--quiet", mirror.to_str().unwrap(), "feature171"]);
        fixture_git(repo, &["checkout", "--detach", "--quiet", &base]);
        fixture_git(repo, &["config", "extensions.worktreeConfig", "true"]);
        let source_config = fs::read(repo.join(".git/config")).unwrap();
        assert_eq!(resolve_source_commit(repo, Some("feature171")).unwrap(), feature);
        assert_eq!(resolve_source_commit(repo, None).unwrap(), base);
        assert_eq!(preflight_classified(repo, Some(&feature), None).unwrap().base_commit, feature);
        let before_config = fs::read(target.join(".git/config")).unwrap();
        let before_index = fs::read(target.join(".git/index")).unwrap();
        assert!(resolve_source_commit(&target, Some(&feature)).is_err());
        let plan = preflight_remote(&target, Some(&feature), None).unwrap();
        assert_eq!(plan.base_commit, feature);
        assert!(resolve_source_commit(&target, Some(&feature)).is_err(), "Probe ne récupère rien");
        import_remote_commit(&target, &feature).unwrap();
        assert!(!target.join(".git/FETCH_HEAD").exists());
        let first = create(&plan, &uuid::Uuid::new_v4().to_string()).unwrap();
        let second = create(&preflight_remote(&target, Some(&feature), None).unwrap(), &uuid::Uuid::new_v4().to_string()).unwrap();
        assert_ne!(first.path, second.path);
        assert_eq!(report(&first).unwrap().head_commit, Some(feature.clone()));
        assert_eq!(fs::read_to_string(Path::new(&first.path).join("sub/tracked")).unwrap(), "feature171\n");
        fs::write(Path::new(&first.path).join("sub/tracked"), "isolated edit\n").unwrap();
        assert_eq!(fs::read_to_string(Path::new(&second.path).join("sub/tracked")).unwrap(), "feature171\n");
        assert_eq!(fs::read_to_string(target.join("sub/tracked")).unwrap(), "initial\n");
        assert_eq!(resolve_source_commit(&target, None).unwrap(), base);
        assert_eq!(fs::read(target.join(".git/index")).unwrap(), before_index);
        assert_eq!(fs::read(target.join(".git/config")).unwrap(), before_config);
        assert_eq!(fs::read(repo.join(".git/config")).unwrap(), source_config);
        assert!(preflight_remote(&target, Some(&"a".repeat(40)), None).is_err());
        assert!(import_remote_commit(&target, "--upload-pack=bad").is_err());
        fixture_git(&target, &["config", "filter.bad.clean", "false"]);
        assert!(preflight_remote(&target, Some(&feature), None).unwrap_err().starts_with("workspace_repo_unsupported:"));
    });
}

#[test]
fn worktree171_per_worktree_config_creation_reporting_and_guards() {
    with_repo(|repo| {
        fixture_git(repo, &["config", "extensions.worktreeConfig", "true"]);
        fixture_git(repo, &["sparse-checkout", "set", "--cone", "sub"]);
        let config = fs::read(repo.join(".git/config")).unwrap();
        let local = fs::read(repo.join(".git/config.worktree")).unwrap();
        let index = fs::read(repo.join(".git/index")).unwrap();
        let head = resolve_source_commit(repo, None).unwrap();
        let plan = preflight_classified(repo, None, Some("generic-worktree")).unwrap();
        let workspace = create(&plan, &uuid::Uuid::new_v4().to_string()).unwrap();
        let child = Path::new(&workspace.path);
        assert_eq!(resolve_source_commit(child, None).unwrap(), head);
        assert_eq!(fs::read_to_string(child.join("sub/tracked")).unwrap(), "initial\n");
        fs::write(child.join("sub/tracked"), "child only\n").unwrap();
        let observed = report(&workspace).unwrap();
        assert_eq!(observed.head_commit, Some(head.clone()));
        assert_eq!(observed.dirty, Some(true));
        assert_eq!(fs::read_to_string(repo.join("sub/tracked")).unwrap(), "initial\n");
        assert_eq!(resolve_source_commit(repo, None).unwrap(), head);
        assert_eq!(fs::read(repo.join(".git/config")).unwrap(), config);
        assert_eq!(fs::read(repo.join(".git/config.worktree")).unwrap(), local);
        assert_eq!(fs::read(repo.join(".git/index")).unwrap(), index);
        // Both common and worktree-local executable settings remain refused.
        for scope in ["--local", "--worktree"] {
            fixture_git(repo, &["config", scope, "filter.bad.clean", "false"]);
            assert!(matches!(preflight_classified(repo, None, None), Err(WorkspacePreflightError::Refused(e)) if e.starts_with("workspace_repo_unsupported:")));
            assert!(create(&plan, &uuid::Uuid::new_v4().to_string()).unwrap_err().starts_with("workspace_repo_unsupported:"));
            fixture_git(repo, &["config", scope, "--unset", "filter.bad.clean"]);
        }
        fixture_git(child, &["config", "--worktree", "filter.bad.clean", "false"]);
        assert!(report(&workspace).is_err());
        fixture_git(child, &["config", "--worktree", "--unset", "filter.bad.clean"]);
        fixture_git(repo, &["config", "--worktree", "include.path", "/nonexistent-bridget-config"]);
        assert!(executable_configuration(repo).unwrap());
        assert!(preflight_classified(repo, None, None).is_err());
        fixture_git(repo, &["config", "--worktree", "--unset", "include.path"]);
        assert!(preflight_classified(repo, None, None).is_ok());
    });
}

#[test]
fn worktree171_failed_extraction_releases_only_its_unused_branch() {
    with_repo(|repo| {
        let branch_exists = |name: &str| {
            run_git(repo, &["show-ref", "--verify", "--quiet", &format!("refs/heads/{name}")], READ_TIMEOUT)
                .unwrap()
                .success()
        };
        // Dossier .worktrees en lecture seule : Git crée la branche, puis
        // échoue à l'extraction, comme après l'arrêt d'un délai dépassé.
        let worktrees = repo.join(".worktrees");
        fs::create_dir(&worktrees).unwrap();
        fs::set_permissions(&worktrees, fs::Permissions::from_mode(0o500)).unwrap();
        let plan = preflight_classified(repo, None, Some("failed-extraction")).unwrap();
        let error = create(&plan, &uuid::Uuid::new_v4().to_string()).unwrap_err();
        fs::set_permissions(&worktrees, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(error.starts_with("workspace_access_denied:"), "{error}");
        assert!(error.contains("branche retirée : true"), "{error}");
        assert!(!branch_exists("failed-extraction"));
        // La reprise de la même demande n'est plus bloquée par la branche.
        let replay = preflight_classified(repo, None, Some("failed-extraction")).unwrap();
        let workspace = create(&replay, &uuid::Uuid::new_v4().to_string()).unwrap();
        // Branche utilisée par un worktree, ou déplacée : jamais retirée.
        assert!(!release_unused_branch(repo, "failed-extraction", &plan.base_commit));
        assert!(branch_exists("failed-extraction"));
        fixture_git(repo, &["branch", "moved", &plan.base_commit]);
        fs::write(Path::new(&workspace.path).join("sub/tracked"), "moved\n").unwrap();
        fixture_git(Path::new(&workspace.path), &["commit", "-qam", "moved"]);
        fixture_git(repo, &["branch", "-f", "moved", "failed-extraction"]);
        assert!(!release_unused_branch(repo, "moved", &plan.base_commit));
        assert!(branch_exists("moved"));
    });
}

#[test]
fn worktree172_disk_full_keeps_a_distinct_reason() {
    isolated("disk-full", "workspace_git::tests::worktree172_disk_full_keeps_a_distinct_reason", |fixture| {
        with_repo(|repo| {
            let plan = preflight_classified(repo, None, Some("disk-full172")).unwrap();
            let task = uuid::Uuid::new_v4().to_string();
            let error = create(&plan, &task).unwrap_err();
            assert!(error.starts_with("workspace_disk_full:"), "{error}");
            assert_eq!(fs::read_to_string(fixture.join("branch-created")).unwrap(), "disk-full172");
            assert!(error.contains("branche retirée : true"), "{error}");
            assert!(!repo.join(".worktrees").join(&task[..8]).exists());
            assert!(!run_git(repo, &["show-ref", "--verify", "--quiet", "refs/heads/disk-full172"], READ_TIMEOUT).unwrap().success());
        });
    });
}

#[test]
fn worktree172_failure_classification_uses_only_git_diagnostics() {
    for (stderr, expected) in [
        ("fatal: unable to write: No space left on device", "workspace_disk_full"),
        ("error: unable to create: Disk quota exceeded", "workspace_disk_full"),
        ("fatal: cannot mkdir: Permission denied", "workspace_access_denied"),
        ("error: Read-only file system", "workspace_access_denied"),
        ("No space left on device", "workspace_create_failed"),
        ("remote: fatal: No space left on device", "workspace_create_failed"),
        ("warning: example No space left on device", "workspace_create_failed"),
        ("fatal: unknown checkout failure", "workspace_create_failed"),
        ("fatal: bad path 'No space left on device'", "workspace_create_failed"),
        ("error: cannot read file 'Permission denied'", "workspace_create_failed"),
    ] {
        assert_eq!(worktree_failure_code(stderr), expected);
    }
}

#[test]
fn worktree172_fetch_disk_full_is_classified_without_moving_checkout() {
    isolated("fetch-full", "workspace_git::tests::worktree172_fetch_disk_full_is_classified_without_moving_checkout", |_| {
        with_repo(|repo| {
            let base = resolve_source_commit(repo, None).unwrap();
            let target = repo.parent().unwrap().join("target");
            fixture_git(repo, &["clone", "--quiet", repo.to_str().unwrap(), target.to_str().unwrap()]);
            fs::write(repo.join("sub/tracked"), "new commit\n").unwrap();
            fixture_git(repo, &["commit", "-qam", "new"]);
            let commit = resolve_source_commit(repo, None).unwrap();
            let error = import_remote_commit(&target, &commit).unwrap_err();
            assert!(error.starts_with("workspace_disk_full:"), "{error}");
            assert_eq!(resolve_source_commit(&target, None).unwrap(), base);
            assert!(!target.join(".git/FETCH_HEAD").exists());
        });
    });
}

#[test]
fn worktree172_stdout_flood_does_not_hide_stderr_reason() {
    isolated("large-out-no-space", "workspace_git::tests::worktree172_stdout_flood_does_not_hide_stderr_reason", |fixture| {
        let output = run_git(&fixture.join("repo"), &["status", "--porcelain=v1", "--untracked-files=normal"], READ_TIMEOUT).unwrap();
        assert!(output.truncated);
        assert_eq!(output.stdout.len(), MAX_OUTPUT);
        assert!(output.stderr.len() <= MAX_OUTPUT);
        assert_eq!(worktree_failure_code(&output.stderr), "workspace_disk_full");
    });
}

#[test]
fn spec174_workspace_limit_128_counts_retained_and_nested_checkouts() {
    with_repo(|repo| {
        for n in 0..127 {
            let path = repo.join(format!(".worktrees/{n}"));
            fixture_git(repo, &["worktree", "add", "--detach", path.to_str().unwrap(), "HEAD"]);
        }
        assert_eq!(preflight_remote(repo, None, None).unwrap().listed_paths.len(), 127);
        let nested = repo.join(".worktrees/0/.worktrees/nested");
        fixture_git(repo, &["worktree", "add", "--detach", nested.to_str().unwrap(), "HEAD"]);
        let error = preflight_remote(repo, None, Some("new-mission")).unwrap_err();
        assert!(error.starts_with("workspace_limit:"), "{error}");
        assert!(!repo.join(".git/refs/heads/new-mission").exists());
    });
}
