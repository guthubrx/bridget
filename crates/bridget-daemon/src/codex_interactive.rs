//! Cycle de vie de l'interface Codex native. La communication reste dans le
//! wrapper existant ; aucun caractère n'est injecté dans le terminal.
use std::ffi::OsString;
use std::io::{self, IsTerminal};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub(crate) struct Launch {
    pub server_args: Vec<String>,
    pub tui_args: Vec<String>,
    pub model: Option<String>,
    pub resume_thread: Option<String>,
    pub display_name: Option<String>,
}

impl Launch {
    pub(crate) fn parse(args: &[String]) -> Result<Self, String> {
        let mut server_args = Vec::new();
        let mut tui_args = Vec::new();
        let mut model = None;
        let mut resume_thread = None;
        let mut display_name = None;
        let mut index = 0;
        let mut prompt = None;
        while index < args.len() {
            let arg = &args[index];
            let (option, inline) = arg
                .split_once('=')
                .map_or((arg.as_str(), None), |(k, v)| (k, Some(v)));
            match option {
                "--remote" | "--remote-auth-token-env" | "--last" | "--all" =>
                    return Err(format!("{option} incompatible : Bridget possède le serveur et le fil de cette session")),
                "--dangerously-bypass-approvals-and-sandbox" | "--yolo" | "--search" => {
                    if inline.is_some() { return Err(format!("{option} ne prend pas de valeur")); }
                    // Seulement les choix EXPLICITES de l'humain, jamais ceux
                    // de la définition gérée, ne changent ses permissions.
                    tui_args.push(if option == "--yolo" { "--dangerously-bypass-approvals-and-sandbox".into() } else { arg.clone() });
                    match option {
                        "--dangerously-bypass-approvals-and-sandbox" | "--yolo" => server_args.extend(["-c".into(), "approval_policy=\"never\"".into(), "-c".into(), "sandbox_mode=\"danger-full-access\"".into()]),
                        _ => server_args.extend(["-c".into(), "web_search=\"live\"".into()]),
                    }
                }
                "--no-alt-screen" => tui_args.push(arg.clone()),
                "-m" | "--model" | "-c" | "--config" | "-a" | "--ask-for-approval" | "-s" | "--sandbox" | "-p" | "--profile" | "--enable" | "--disable" => {
                    debug_assert!(crate::wrapper::codex_option_takes_value(option));
                    let value = match inline {
                        Some(value) => value.to_owned(),
                        None => { index += 1; args.get(index).cloned().ok_or_else(|| format!("valeur manquante pour {option}"))? }
                    };
                    tui_args.extend([option.to_owned(), value.clone()]);
                    match option {
                        "-m" | "--model" => {
                            model = Some(value.clone());
                            server_args.extend(["-c".into(), format!("model={}", serde_json::to_string(&value).unwrap())]);
                        }
                        "-a" | "--ask-for-approval" | "-s" | "--sandbox" => {
                            let key = if matches!(option, "-a" | "--ask-for-approval") { "approval_policy" } else { "sandbox_mode" };
                            server_args.extend(["-c".into(), format!("{key}={}", serde_json::to_string(&value).unwrap())]);
                        }
                        _ => server_args.extend([option.to_owned(), value]),
                    }
                }
                "-C" | "--cd" | "--add-dir" | "-i" | "--image" | "--oss" | "--local-provider" =>
                    return Err(format!("{option} non pris en charge dans cette première session partagée ; lancez depuis le répertoire voulu")),
                "--name" => {
                    let value = match inline {
                        Some(value) => value.to_owned(),
                        None => { index += 1; args.get(index).cloned().ok_or("valeur manquante pour --name")? }
                    };
                    if value.trim().is_empty() || value.chars().any(char::is_control)
                        || value.chars().count() > crate::agent_profile::MAX_DISPLAY_NAME_CHARS {
                        return Err("--name : nom non vide, sans caractère de contrôle, limité à 80 caractères".into());
                    }
                    if display_name.replace(value).is_some() { return Err("--name ne peut apparaître qu'une fois".into()); }
                }
                "resume" => {
                    if resume_thread.is_some() || prompt.is_some() || inline.is_some() {
                        return Err("resume <UUID> doit précéder le prompt et ne peut apparaître qu'une fois".into());
                    }
                    index += 1;
                    let value = args.get(index).ok_or("resume exige l'UUID explicite du fil Codex")?;
                    let id = uuid::Uuid::parse_str(value).map_err(|_| "resume exige l'UUID explicite du fil Codex")?;
                    resume_thread = Some(id.to_string());
                }
                "fork" | "app-server" | "exec" =>
                    return Err("sous-commande non prise en charge ; utilisez resume <UUID> pour choisir le fil initial".into()),
                _ if arg.starts_with('-') => return Err(format!("option Codex non prise en charge : {arg}")),
                _ => {
                    if prompt.replace(arg.clone()).is_some() { return Err("un seul prompt initial est accepté (entre guillemets)".into()); }
                }
            }
            index += 1;
        }
        if let Some(prompt) = prompt {
            tui_args.push(prompt);
        }
        server_args.push("app-server".into());
        Ok(Self {
            server_args,
            tui_args,
            model,
            resume_thread,
            display_name,
        })
    }

    pub(crate) fn check_terminal() -> Result<(), String> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Err("Codex interactif exige stdin et stdout sur un terminal ; utilisez bridget spawn pour un agent détaché".into());
        }
        Ok(())
    }
}

/// Le parent restaure les attributs même si le fournisseur quitte sans rendre
/// son terminal. Il ne lit ni ne réécrit la saisie de l'utilisateur.
pub(crate) struct NativeTui {
    child: Arc<Mutex<Child>>,
    stopped: Arc<AtomicBool>,
    monitor: Option<thread::JoinHandle<()>>,
    termios: libc::termios,
    signals: Vec<signal_hook::SigId>,
    exit_code: Arc<AtomicI32>,
}

impl NativeTui {
    pub(crate) fn start(
        binary: &str,
        launch: &Launch,
        socket: &Path,
        thread_id: &str,
        environment: &[(OsString, OsString)],
        alive: Arc<AtomicBool>,
    ) -> io::Result<Self> {
        let mut termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut termios) } != 0 {
            return Err(io::Error::last_os_error());
        }
        let mut command = Command::new(binary);
        command
            .args([
                "resume",
                thread_id,
                "--remote",
                &format!("unix://{}", socket.display()),
            ])
            .args(&launch.tui_args)
            .envs(environment.iter().cloned())
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let stopped = Arc::new(AtomicBool::new(false));
        let exit_code = Arc::new(AtomicI32::new(i32::MIN));
        let mut signals = Vec::new();
        for signal in [libc::SIGHUP, libc::SIGTERM, libc::SIGINT] {
            match signal_hook::flag::register(signal, stopped.clone()) {
                Ok(id) => signals.push(id),
                Err(error) => {
                    for id in signals {
                        signal_hook::low_level::unregister(id);
                    }
                    return Err(error);
                }
            }
        }
        let child = match command.spawn() {
            Ok(child) => Arc::new(Mutex::new(child)),
            Err(error) => {
                for id in signals {
                    signal_hook::low_level::unregister(id);
                }
                return Err(error);
            }
        };
        let watch_child = child.clone();
        let watch_stop = stopped.clone();
        let observed_exit = exit_code.clone();
        let monitor = thread::spawn(move || {
            while !watch_stop.load(Ordering::SeqCst) && alive.load(Ordering::SeqCst) {
                match watch_child
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .try_wait()
                {
                    Ok(None) => thread::sleep(Duration::from_millis(25)),
                    Ok(Some(status)) => {
                        observed_exit.store(status.code().unwrap_or(1), Ordering::SeqCst);
                        break;
                    }
                    Err(_) => {
                        observed_exit.store(1, Ordering::SeqCst);
                        break;
                    }
                }
            }
            alive.store(false, Ordering::SeqCst);
        });
        Ok(Self {
            child,
            stopped,
            monitor: Some(monitor),
            termios,
            signals,
            exit_code,
        })
    }

    pub(crate) fn result(&self) -> Result<(), String> {
        match self.exit_code.load(Ordering::SeqCst) {
            0 => Ok(()),
            i32::MIN if self.stopped.load(Ordering::SeqCst) => Ok(()),
            i32::MIN => Err("serveur Codex arrêté ; session interactive terminée".into()),
            code => Err(format!(
                "interface Codex terminée en erreur (code {code}), session Bridget fermée"
            )),
        }
    }
}

impl Drop for NativeTui {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl NativeTui {
    pub(crate) fn close(&mut self) -> io::Result<()> {
        self.stopped.store(true, Ordering::SeqCst);
        if let Some(monitor) = self.monitor.take() {
            let _ = monitor.join();
        }
        let mut child = self.child.lock().unwrap_or_else(|e| e.into_inner());
        let result = bridget_transport::managed_session::stop_owned_child(
            &mut child,
            false,
            Duration::from_secs(2),
        );
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.termios);
        }
        for signal in self.signals.drain(..) {
            signal_hook::low_level::unregister(signal);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn options_natives_conservees_sans_bypass_implicite() {
        let parsed = Launch::parse(&args(&[
            "-m",
            "gpt-5.6-terra",
            "-a",
            "on-request",
            "-s",
            "read-only",
            "bonjour",
        ]))
        .unwrap();
        assert_eq!(parsed.model.as_deref(), Some("gpt-5.6-terra"));
        assert!(
            parsed
                .server_args
                .contains(&"approval_policy=\"on-request\"".into())
        );
        assert!(
            !parsed
                .server_args
                .iter()
                .any(|v| v.contains("bypass") || v.contains("danger-full"))
        );
        assert_eq!(parsed.tui_args.last().unwrap(), "bonjour");
        assert_eq!(Launch::parse(&[]).unwrap().server_args, ["app-server"]);
    }
    #[test]
    fn yolo_resume_et_nom_bridget_ne_sont_pas_des_arguments_perdus() {
        let id = "01a027a9-046e-7cf1-9e75-c689c3bdf451";
        for values in [
            vec!["--name", "coderBridget", "--yolo", "resume", id],
            vec!["resume", id, "--name=coderBridget", "--yolo"],
        ] {
            let parsed = Launch::parse(&args(&values)).unwrap();
            assert_eq!(parsed.resume_thread.as_deref(), Some(id));
            assert_eq!(parsed.display_name.as_deref(), Some("coderBridget"));
            let long =
                Launch::parse(&args(&["--dangerously-bypass-approvals-and-sandbox"])).unwrap();
            assert_eq!(parsed.server_args, long.server_args);
            assert_eq!(parsed.tui_args, long.tui_args);
        }
        assert!(Launch::parse(&[]).unwrap().display_name.is_none());
        assert!(Launch::parse(&[]).unwrap().resume_thread.is_none());
        for values in [
            vec!["resume"],
            vec!["--name"],
            vec!["--name", " "],
            vec!["--name", "nom\nmenteur"],
            vec!["--yolo=false"],
            vec!["--name", "a", "--name", "b"],
            vec!["resume", id, "resume", id],
        ] {
            assert!(Launch::parse(&args(&values)).is_err(), "{values:?}");
        }
    }
    #[test]
    fn option_non_relayee_et_changement_de_fil_refuses() {
        for values in [
            &["--remote", "unix:///tmp/other"][..],
            &["--model"],
            &["--cd", "/tmp"],
            &["resume", "thread"],
            &["--inventee"],
        ] {
            assert!(Launch::parse(&args(values)).is_err(), "{values:?}");
        }
    }
}
