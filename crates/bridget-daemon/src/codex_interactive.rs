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
                        return Err("resume [UUID|nom] doit précéder le prompt et ne peut apparaître qu'une fois".into());
                    }
                    let target = args.get(index + 1).filter(|value| !value.starts_with('-'));
                    resume_thread = Some(match target {
                        Some(value) => {
                            if value.trim().is_empty() || value.chars().any(char::is_control) || value.len() > 1024 {
                                return Err("nom de conversation Codex invalide".into());
                            }
                            index += 1;
                            uuid::Uuid::parse_str(value).map_or_else(|_| value.clone(), |id| id.to_string())
                        }
                        None => String::new(), // Sélection humaine avant tout fil ou présence.
                    });
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

    /// Options passées à la TUI qui rejoint le fil distant. Les permissions
    /// (bypass, politique d'approbation, sandbox) vivent déjà dans la config du
    /// serveur privé ; Codex 0.154 refuse de les recevoir aussi sur
    /// `resume --remote` (« Permission overrides are not supported when
    /// resuming a remote task »). Elles sont donc retirées ici, sans changer
    /// ce que le serveur applique ni la commande de reprise affichée à l'humain.
    pub(crate) fn remote_tui_args(&self) -> Vec<String> {
        let mut args = Vec::with_capacity(self.tui_args.len());
        let mut index = 0;
        while let Some(arg) = self.tui_args.get(index) {
            match arg.as_str() {
                "--dangerously-bypass-approvals-and-sandbox" => {}
                "-a" | "--ask-for-approval" | "-s" | "--sandbox" => index += 1,
                _ => args.push(arg.clone()),
            }
            index += 1;
        }
        args
    }

    pub(crate) fn resume_command(&self, thread_id: &str) -> String {
        // Conserver les options explicites, jamais rejouer le prompt initial.
        let mut args = vec!["bridget".to_owned(), "codex".to_owned()];
        let mut index = 0;
        while let Some(option) = self.tui_args.get(index).filter(|arg| arg.starts_with('-')) {
            args.push(option.clone());
            if crate::wrapper::codex_option_takes_value(option) {
                index += 1;
                if let Some(value) = self.tui_args.get(index) {
                    args.push(value.clone());
                }
            }
            index += 1;
        }
        if let Some(name) = &self.display_name {
            args.extend(["--name".into(), name.clone()]);
        }
        args.extend(["resume".into(), thread_id.into()]);
        args.into_iter()
            .map(|arg| {
                if arg
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-._/".contains(&c))
                    && !arg.is_empty()
                {
                    arg
                } else {
                    format!("'{}'", arg.replace('\'', "'\\''"))
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub(crate) fn needs_selection(&self) -> bool {
        self.resume_thread
            .as_ref()
            .is_some_and(|target| uuid::Uuid::parse_str(target).is_err())
    }

    pub(crate) fn select_thread(
        &self,
        threads: &[bridget_transport::codex_app_server::CodexThreadSummary],
    ) -> Result<String, String> {
        let target = self
            .resume_thread
            .as_deref()
            .ok_or("sélection sans reprise")?;
        if !target.is_empty() {
            let matches = threads
                .iter()
                .filter(|thread| thread.name.as_deref() == Some(target))
                .collect::<Vec<_>>();
            return match matches.as_slice() {
                [thread] => Ok(thread.id.clone()),
                [] => Err(format!(
                    "conversation Codex « {target} » introuvable ; utilisez `bridget codex resume` pour choisir"
                )),
                _ => Err(format!(
                    "plusieurs conversations Codex portent « {target} » ; utilisez `bridget codex resume` pour choisir"
                )),
            };
        }
        select_thread_in_terminal(threads)
    }

    pub(crate) fn check_terminal() -> Result<(), String> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Err("Codex interactif exige stdin et stdout sur un terminal ; utilisez bridget spawn pour un agent détaché".into());
        }
        Ok(())
    }
}

fn select_thread_in_terminal(
    threads: &[bridget_transport::codex_app_server::CodexThreadSummary],
) -> Result<String, String> {
    use std::io::Write;
    Launch::check_terminal()?;
    if threads.is_empty() {
        return Err("aucune conversation Codex à reprendre".into());
    }
    // Mode canonique conservé : pas de capture de la TUI, ni de terminal raw.
    // Intercepter les signaux pendant l'attente permet au pilote de nettoyer
    // SON app-server si l'humain annule avant toute sélection.
    struct Signals(Vec<signal_hook::SigId>);
    impl Drop for Signals {
        fn drop(&mut self) {
            for id in self.0.drain(..) {
                signal_hook::low_level::unregister(id);
            }
        }
    }
    let cancelled = Arc::new(AtomicBool::new(false));
    let mut signals = Signals(Vec::new());
    for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        signals.0.push(
            signal_hook::flag::register(signal, cancelled.clone())
                .map_err(|error| error.to_string())?,
        );
    }
    let clean = |text: &str| {
        text.chars()
            .filter(|ch| !ch.is_control())
            .take(120)
            .collect::<String>()
    };
    let mut page = 0;
    loop {
        eprintln!(
            "Conversations Codex — choisir un numéro (n : suivantes, p : précédentes, q : annuler)"
        );
        for (i, thread) in threads.iter().enumerate().skip(page * 20).take(20) {
            eprintln!(
                "  {}. {} — {} [{}]",
                i + 1,
                clean(
                    thread
                        .name
                        .as_deref()
                        .filter(|name| !name.is_empty())
                        .or(thread.preview.as_deref())
                        .unwrap_or("sans titre")
                ),
                clean(&thread.cwd),
                thread.id
            );
        }
        eprint!("Choix : ");
        io::stderr().flush().map_err(|error| error.to_string())?;
        loop {
            if cancelled.load(Ordering::SeqCst) {
                return Err("reprise annulée : aucune session ouverte".into());
            }
            let mut fd = libc::pollfd {
                fd: libc::STDIN_FILENO,
                events: libc::POLLIN,
                revents: 0,
            };
            let result = unsafe { libc::poll(&mut fd, 1, 100) };
            if result > 0 {
                break;
            }
            if result < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
                return Err(io::Error::last_os_error().to_string());
            }
        }
        let mut choice = String::new();
        if io::stdin()
            .read_line(&mut choice)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Err("reprise annulée : terminal fermé".into());
        }
        match choice.trim() {
            "q" => return Err("reprise annulée : aucune session ouverte".into()),
            "n" if (page + 1) * 20 < threads.len() => page += 1,
            "p" if page > 0 => page -= 1,
            value => {
                if let Some(thread) = value
                    .parse::<usize>()
                    .ok()
                    .and_then(|number| number.checked_sub(1))
                    .and_then(|index| threads.get(index))
                {
                    return Ok(thread.id.clone());
                }
                eprintln!("Choix invalide, aucun fil ouvert.");
            }
        }
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
            .args(launch.remote_tui_args())
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
    fn commande_de_reprise_humaine_conserve_options_et_quote_le_nom_sans_prompt() {
        let launch = Launch::parse(&args(&[
            "--name",
            "l'agent humain",
            "--yolo",
            "-m",
            "fixture",
            "-c",
            "key=\"a b\"",
            "PROMPT-NE-PAS-REJOUER",
        ]))
        .unwrap();
        let command = launch.resume_command("90000000-0000-4000-8000-000000000001");
        assert_eq!(
            command,
            "bridget codex --dangerously-bypass-approvals-and-sandbox -m fixture -c 'key=\"a b\"' --name 'l'\\''agent humain' resume 90000000-0000-4000-8000-000000000001"
        );
        assert!(!command.contains("PROMPT-NE-PAS-REJOUER"));
        assert!(!command.contains("--agent-id"));
        assert!(!command.contains("--remote"));
    }

    #[test]
    fn reprise_par_nom_exact_ou_menu_sans_nom_bridget_invente() {
        use bridget_transport::codex_app_server::CodexThreadSummary;
        let named = Launch::parse(&args(&[
            "--yolo",
            "--name",
            "agent-visible",
            "resume",
            "horizon-original",
        ]))
        .unwrap();
        assert!(named.needs_selection());
        assert_eq!(named.display_name.as_deref(), Some("agent-visible"));
        let entry = CodexThreadSummary {
            id: "01a06f68-dbab-7b43-84b5-7e63b03d93b0".into(),
            name: Some("horizon-original".into()),
            preview: None,
            cwd: "/tmp".into(),
        };
        assert_eq!(
            named.select_thread(std::slice::from_ref(&entry)).unwrap(),
            entry.id
        );
        assert!(
            named
                .select_thread(&[])
                .unwrap_err()
                .contains("introuvable")
        );
        let mut other = entry.clone();
        other.id = "01a06f68-dbab-7b43-84b5-7e63b03d93b1".into();
        assert!(
            named
                .select_thread(&[entry, other])
                .unwrap_err()
                .contains("plusieurs")
        );
        for arguments in [
            vec!["resume"],
            vec!["resume", "--yolo", "--name", "agent-visible"],
        ] {
            let menu = Launch::parse(&args(&arguments)).unwrap();
            assert_eq!(menu.resume_thread.as_deref(), Some(""));
            assert!(menu.needs_selection());
        }
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
    fn la_tui_distante_ne_recoit_plus_les_surcharges_de_permissions() {
        // Codex 0.154 : « Permission overrides are not supported when resuming
        // a remote task ». Le serveur privé garde la politique, la TUI ne la
        // répète pas ; les autres options et le prompt restent relayés.
        let parsed = Launch::parse(&args(&[
            "--yolo",
            "-a",
            "never",
            "-s",
            "danger-full-access",
            "-m",
            "fixture",
            "--no-alt-screen",
            "resume",
            "01a027a9-046e-7cf1-9e75-c689c3bdf451",
        ]))
        .unwrap();
        assert!(
            parsed
                .server_args
                .iter()
                .any(|a| a == "approval_policy=\"never\"")
        );
        assert!(
            parsed
                .server_args
                .iter()
                .any(|a| a == "sandbox_mode=\"danger-full-access\"")
        );
        assert_eq!(
            parsed.remote_tui_args(),
            vec!["-m", "fixture", "--no-alt-screen"]
        );
        // La commande de reprise affichée à l'humain conserve son choix explicite.
        assert!(
            parsed
                .resume_command("01a027a9-046e-7cf1-9e75-c689c3bdf451")
                .contains("--dangerously-bypass-approvals-and-sandbox")
        );
    }

    #[test]
    fn option_non_relayee_et_changement_de_fil_refuses() {
        for values in [
            &["--remote", "unix:///tmp/other"][..],
            &["--model"],
            &["--cd", "/tmp"],
            &["resume", "\nthread"],
            &["--inventee"],
        ] {
            assert!(Launch::parse(&args(values)).is_err(), "{values:?}");
        }
    }
}
