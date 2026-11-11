use super::*;
use serde_json::json;
use std::os::unix::fs::PermissionsExt;
use std::process::ExitStatus;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
#[test]
fn remote162_drain_pidfd_pipe_herite_et_barriere_child_wait() {
    const CHILD_MODE: &str = "BRIDGET_REMOTE162_DRAIN_CHILD";
    if std::env::var_os(CHILD_MODE).is_none() {
        let status = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "managed_process::tests::remote162_drain_pidfd_pipe_herite_et_barriere_child_wait",
                "--nocapture",
            ])
            .env(CHILD_MODE, "1")
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }
    use std::io::Read;
    remote_drain::arm().unwrap();
    assert!(remote_drain::checked_pidfd(std::process::id()).is_err());
    let root = std::env::temp_dir().join(format!("drain162-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let script = r#"import os,signal,sys,time
root=sys.argv[1]
def late_fork(signum, frame):
 if os.fork()==0:
  signal.signal(signal.SIGTERM,signal.SIG_IGN)
  os.setsid()
  if os.fork()!=0: os._exit(0)
  os.setsid()
  open(root+'/late.tmp','w').write(str(os.getpid()))
  os.replace(root+'/late.tmp',root+'/late')
  while not os.path.exists(root+'/release'): time.sleep(.02)
  os._exit(0)
 os._exit(0)
signal.signal(signal.SIGTERM,late_fork)
if os.fork()==0:
 os.setsid()
 if os.fork()!=0: os._exit(0)
 os.setsid()
 signal.signal(signal.SIGTERM,signal.SIG_IGN)
 open(root+'/ready.tmp','w').write(str(os.getpid()))
 os.replace(root+'/ready.tmp',root+'/ready')
 while not os.path.exists(root+'/release'): time.sleep(.02)
 os._exit(0)
while True: time.sleep(.02)
"#;
    let mut child = Command::new("/usr/bin/python3")
        .args(["-c", script])
        .arg(&root)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !root.join("ready").exists() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    let descendant: u32 = fs::read_to_string(root.join("ready"))
        .unwrap()
        .parse()
        .unwrap();
    let birth = process_birth(descendant).unwrap();
    let mut pipe = child.stdout.take().unwrap();
    let (done, received) = std::sync::mpsc::channel();
    let owner = thread::spawn(move || {
        let status = child.wait().unwrap();
        done.send(status).unwrap();
        let mut text = String::new();
        pipe.read_to_string(&mut text).unwrap();
    });
    // Signalisation uniquement : Child::wait reste l'unique récolteur ici.
    remote_drain::signal_pass().unwrap();
    received.recv_timeout(Duration::from_secs(5)).unwrap();
    // Le nouveau double-fork est déclenché PAR TERM, pendant l'arrêt.
    while !root.join("late").exists() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    let late: u32 = fs::read_to_string(root.join("late"))
        .unwrap()
        .parse()
        .unwrap();
    let late_birth = process_birth(late).unwrap();
    while remote_drain::checked_pidfd(late).is_err() {
        assert!(
            Instant::now() < deadline,
            "enfant tardif non adopté par le subreaper"
        );
        thread::sleep(Duration::from_millis(10));
    }
    remote_drain::signal_pass().unwrap();
    assert_eq!(process_birth(late).unwrap(), late_birth);
    assert_eq!(process_birth(descendant).unwrap(), birth);
    assert!(
        !owner.is_finished(),
        "le pipe hérité doit retenir le lecteur"
    );
    fs::write(root.join("release"), []).unwrap();
    owner.join().unwrap(); // barrière réelle, aucune récolte ciblée avant elle
    loop {
        remote_drain::signal_pass().unwrap();
        if remote_drain::reap_pass_after_owners().unwrap() {
            break;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(remote_drain::CADENCE);
    }
    assert!(process_birth(descendant).is_err());
    assert!(process_birth(late).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "Recette privée explicite root, sans fournisseur ni fichier métier"]
fn spec221_drain_reaps_owned_foreign_uid_zombie_after_signal_refusal() {
    const HELPER: &str = "BRIDGET_SPEC221_DRAIN_HELPER";
    assert_eq!(
        bridget_transport::fsutil::effective_uid(),
        0,
        "cette recette exige root"
    );
    if std::env::var_os(HELPER).is_none() {
        let status = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "managed_process::tests::spec221_drain_reaps_owned_foreign_uid_zombie_after_signal_refusal", "--ignored", "--nocapture", "--test-threads=1"])
                .env(HELPER, "1")
                .status().unwrap();
        assert!(status.success(), "recette enfant UID différent refusée");
        return;
    }
    remote_drain::arm().unwrap();
    assert!(remote_drain::checked_pidfd(std::process::id()).is_err());
    let child = Command::new("/bin/sh")
        .args(["-c", "exit 0"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let pid = child.id();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status = fs::read_to_string(format!("/proc/{pid}/status")).unwrap();
        if status
            .lines()
            .any(|line| line.starts_with("State:") && line.contains('Z'))
        {
            break;
        }
        assert!(Instant::now() < deadline, "l'enfant privé doit terminer");
        thread::sleep(Duration::from_millis(10));
    }
    let birth = process_birth(pid).unwrap();
    drop(child); // Barrière : aucun propriétaire Child ne peut encore wait ce PID.
    assert_eq!(unsafe { libc::setgroups(0, std::ptr::null()) }, 0);
    assert_eq!(unsafe { libc::setgid(65534) }, 0);
    assert_eq!(unsafe { libc::setuid(65534) }, 0);
    assert_eq!(process_birth(pid).unwrap(), birth);
    let signal = remote_drain::signal_pass();
    let drained = remote_drain::reap_pass_after_owners();
    // Nettoyage même sur l'ancienne garde fautive avant l'assertion RED.
    if !matches!(drained, Ok(true)) {
        let mut status = 0;
        assert_eq!(
            unsafe { libc::waitpid(pid as i32, &mut status, 0) },
            pid as i32
        );
    }
    assert!(
        signal.is_err(),
        "le parent non privilégié ne signale pas son enfant root"
    );
    assert!(
        matches!(drained, Ok(true)),
        "le refus de signal ne doit pas interdire la récolte : {drained:?}"
    );
    assert!(process_birth(pid).is_err(), "aucun zombie privé restant");
    println!("SPEC221_FOREIGN_UID_REAP_OK");
}

const BOOTSTRAP_CHILD_ENV: &str = "BRIDGET_T903_BOOTSTRAP_CHILD";
const BOOTSTRAP_REQUEST_ENV: &str = "BRIDGET_T903_BOOTSTRAP_REQUEST";
const FD_PROBE_ENV: &str = "BRIDGET_T903_FD_PROBE";
const CONTROLLER_ENV: &str = "BRIDGET_T903_CRASH_CONTROLLER";
const CONTROLLER_STAGE_ENV: &str = "BRIDGET_T903_CRASH_STAGE";
const CONTROLLER_ROOT_ENV: &str = "BRIDGET_T903_CRASH_ROOT";

fn test_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-t903-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn identity() -> ManagedIdentity {
    ManagedIdentity {
        instance_id: "instance-test".to_string(),
        command_id: "command-test".to_string(),
        generation: 7,
    }
}

#[test]
fn spec221_c19_bootstrap_checks_owned_child_identity_birth_and_group_before_release() {
    let mut ready = spawn_test_bootstrap().wait_ready().unwrap();
    let original = ready.ready.clone();
    let expected = ManagedIdentity {
        instance_id: original.instance_id.clone(),
        command_id: original.command_id.clone(),
        generation: original.generation,
    };
    assert!(ready.belongs_to(&expected));
    ready.ready.command_id = "foreign".into();
    assert!(!ready.belongs_to(&expected));
    ready.ready = original.clone();
    ready.ready.pid = std::process::id();
    assert!(!ready.belongs_to(&expected));
    ready.ready = original.clone();
    // SAFETY: getpgrp ne déréférence aucun pointeur ; ses arguments sont des entiers.
    ready.ready.pgid = unsafe { libc::getpgrp() } as u32;
    assert!(!ready.belongs_to(&expected));
    ready.ready = original.clone();
    ready.ready.birth += 1;
    assert!(!ready.belongs_to(&expected));
    ready.ready = original;
    assert!(ready.belongs_to(&expected));
    assert_eq!(
        ready.abandon().wait().unwrap().code(),
        Some(ABANDONED_EXIT_CODE)
    );
}

fn current_test_executable() -> PathBuf {
    let executable = std::env::current_exe().unwrap();
    assert_ne!(
        executable.file_name().and_then(|name| name.to_str()),
        Some("firefox")
    );
    executable
}

fn helper_command(test_name: &str) -> Command {
    let mut command = Command::new(current_test_executable());
    command
        .arg("--exact")
        .arg(test_name)
        .arg("--ignored")
        .arg("--nocapture")
        .arg("--test-threads=1");
    command
}

pub(crate) fn spawn_test_bootstrap() -> ManagedChild {
    spawn_test_bootstrap_with_wrapper(
        current_test_executable(),
        vec![
            "--exact".to_string(),
            "managed_process::tests::fd_probe_child".to_string(),
            "--ignored".to_string(),
            "--nocapture".to_string(),
            "--test-threads=1".to_string(),
        ],
    )
}

fn spawn_test_bootstrap_with_wrapper(
    wrapper_executable: PathBuf,
    wrapper_args: Vec<String>,
) -> ManagedChild {
    let request = BootstrapRequest {
        identity: identity(),
        wrapper_executable,
        wrapper_args,
    };
    let needs_fd_probe = request
        .wrapper_args
        .iter()
        .any(|argument| argument == "managed_process::tests::fd_probe_child");
    let mut command = helper_command("managed_process::tests::bootstrap_child");
    command.env(BOOTSTRAP_CHILD_ENV, "1").env(
        BOOTSTRAP_REQUEST_ENV,
        serde_json::to_string(&json!({
            "instance_id": request.identity.instance_id,
            "command_id": request.identity.command_id,
            "generation": request.identity.generation,
            "wrapper_executable": request.wrapper_executable,
            "wrapper_args": request.wrapper_args,
        }))
        .unwrap(),
    );
    if needs_fd_probe {
        command.env(FD_PROBE_ENV, "1");
    }
    spawn_bootstrap_command(command, Stdio::null()).unwrap()
}

pub(crate) fn spawn_catalogue_test_bootstrap(launch: &ManagedLaunch) -> ManagedChild {
    spawn_catalogue_test_bootstrap_with_stderr(launch, Stdio::null())
}
pub(crate) fn spawn_catalogue_test_bootstrap_with_stderr(
    launch: &ManagedLaunch,
    stderr: Stdio,
) -> ManagedChild {
    let mut command = helper_command("managed_process::tests::bootstrap_child");
    command.current_dir(&launch.cwd).env_clear().envs(&launch.env)
            .env(BOOTSTRAP_CHILD_ENV,"1").env(BOOTSTRAP_REQUEST_ENV,serde_json::to_string(&json!({
                "instance_id":launch.identity.instance_id,"command_id":launch.identity.command_id,
                "generation":launch.identity.generation,"wrapper_executable":launch.wrapper_executable,"wrapper_args":launch.wrapper_args})).unwrap());
    spawn_bootstrap_command(command, stderr).unwrap()
}

#[test]
#[ignore = "processus enfant lancé par un autre test (variable BOOTSTRAP_CHILD_ENV) ; sans effet lancé seul"]
fn bootstrap_child() {
    if std::env::var(BOOTSTRAP_CHILD_ENV).ok().as_deref() != Some("1") {
        return;
    }
    if std::env::var_os("B221_VIEW_TEST_NO_READY").is_some() {
        use std::os::unix::fs::OpenOptionsExt;
        let root = PathBuf::from(std::env::var_os("B221_VIEW_PROOFS").unwrap());
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe {
            libc::signal(libc::SIGTERM, libc::SIG_IGN);
        }
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("unready-started"))
            .unwrap();
        let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while !root.join("allow-bootstrap-exit").exists() && std::time::Instant::now() < until {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        std::process::exit(0);
    }
    let value: serde_json::Value =
        serde_json::from_str(&std::env::var(BOOTSTRAP_REQUEST_ENV).unwrap()).unwrap();
    let request = BootstrapRequest {
        identity: ManagedIdentity {
            instance_id: value["instance_id"].as_str().unwrap().to_string(),
            command_id: value["command_id"].as_str().unwrap().to_string(),
            generation: value["generation"].as_u64().unwrap(),
        },
        wrapper_executable: PathBuf::from(value["wrapper_executable"].as_str().unwrap()),
        wrapper_args: value["wrapper_args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|argument| argument.as_str().unwrap().to_string())
            .collect(),
    };
    run_bootstrap(request).unwrap();
}

#[test]
#[ignore = "processus enfant lancé par un autre test (variable FD_PROBE_ENV) ; sans effet lancé seul"]
fn fd_probe_child() {
    if std::env::var(FD_PROBE_ENV).ok().as_deref() != Some("1") {
        return;
    }
    // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
    let status_open = unsafe { libc::fcntl(STATUS_FD, libc::F_GETFD) } >= 0;
    // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
    let release_open = unsafe { libc::fcntl(RELEASE_FD, libc::F_GETFD) } >= 0;
    write_all_fd(
        STATUS_FD,
        format!(
            "{}\n",
            json!({
                "event": "fd_probe",
                "status_open": status_open,
                "release_open": release_open,
            })
        )
        .as_bytes(),
    )
    .unwrap();
    let mut byte = [0_u8; 1];
    while read_one(STATUS_FD, &mut byte).unwrap_or(0) != 0 {}
}

fn read_probe(child: &mut RunningManagedChild) -> serde_json::Value {
    let mut line = String::new();
    child.status_reader().read_line(&mut line).unwrap();
    serde_json::from_str(line.trim_end()).unwrap()
}

#[test]
#[ignore = "processus enfant lancé par un autre test (variable BRIDGET_TEST_BOOTSTRAP_ERROR162) ; sans effet lancé seul"]
fn bootstrap_error_child() {
    let Ok(mode) = std::env::var("BRIDGET_TEST_BOOTSTRAP_ERROR162") else {
        return;
    };
    if mode == "failed" {
        write_status_line(&ManagedStatus::StartupFailed {
            kind: "fixture".into(),
            reason: "avant ready".into(),
            instance_id: identity().instance_id,
            command_id: identity().command_id,
            generation: identity().generation,
        })
        .unwrap();
    } else {
        assert_eq!(mode, "eof");
        // SAFETY: close ne déréférence aucun pointeur ; ses arguments sont des entiers.
        unsafe {
            libc::close(STATUS_FD);
        }
    }
    // Le statut ne prouve pas la mort : ce processus attend encore EOF
    // du contrôleur sur RELEASE avant de terminer.
    let mut byte = [0_u8; 1];
    let _ = read_one(RELEASE_FD, &mut byte);
}

#[test]
fn spec243_delai_sur_canal_ferme_reste_un_eof() {
    let (reader, peer) = UnixStream::pair().unwrap();
    drop(peer);
    set_status_timeout(&reader, Duration::from_millis(10)).unwrap();
    let mut line = String::new();
    assert_eq!(BufReader::new(reader).read_line(&mut line).unwrap(), 0);
}

#[test]
fn worker162_bootstrap_erreur_garde_handle_jusqua_arret() {
    for mode in ["eof", "failed"] {
        let mut command = helper_command("managed_process::tests::bootstrap_error_child");
        command.env("BRIDGET_TEST_BOOTSTRAP_ERROR162", mode);
        let bootstrap = spawn_bootstrap_command(command, Stdio::null()).unwrap();
        let pid = bootstrap.inner.child.id();
        if mode == "eof" {
            // Cas observé sous charge : le canal est déjà fermé quand le
            // contrôleur règle son délai (EINVAL sur macOS). Le rendre sûr.
            thread::sleep(Duration::from_millis(500));
        }
        let (error, mut child) =
            match bootstrap.wait_ready_or_cancel_retained(|| false, Duration::from_millis(10)) {
                Err(error) => error,
                Ok(_) => panic!("aucune ready attendue"),
            };
        assert!(
            matches!(error, ManagedProcessError::InvalidStatus(_)),
            "{mode}: {error:?}"
        );
        assert_eq!(child.id(), pid);
        assert!(
            child.try_wait().unwrap().is_some(),
            "arrêt réel, pas seule trame de statut"
        );
    }
}

#[test]
fn worker162_marqueur_refuse_conserve_bootstrap_abandonne() {
    let root = test_root("retained-marker");
    let store = ManagedMarkerStore::at_directory(root.join("managed"));
    let ready = spawn_test_bootstrap().wait_ready().unwrap();
    let pid = ready.ready().pid;
    let (error, mut child) = match ready.persist_marker_retained(&store, "../invalide") {
        Err(error) => error,
        Ok(_) => panic!("nom marqueur invalide accepté"),
    };
    assert!(matches!(error, ManagedProcessError::InvalidArgument(_)));
    assert_eq!(child.id(), pid);
    assert_eq!(child.wait().unwrap().code(), Some(ABANDONED_EXIT_CODE));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn worker162_release_erreur_conserve_processus_et_marqueur_jusqua_recolte() {
    let root = test_root("release-retained");
    let store = ManagedMarkerStore::at_directory(root.join("managed"));
    let marked = spawn_test_bootstrap()
        .wait_ready()
        .unwrap()
        .persist_marker(&store, "worker162")
        .unwrap();
    let pid = marked.inner.child.id();
    let marker = marked.marker().marker().clone();
    let marker_path = marked.marker().path().to_path_buf();
    // EOF déclenche l'abandon après READY et ferme l'écriture RELEASE.
    // Aucune mort n'est présumée ni récoltée avant le retour du handle.
    marked
        .inner
        .release
        .as_ref()
        .unwrap()
        .shutdown(std::net::Shutdown::Write)
        .unwrap();
    let (error, mut retained) = match marked.release_retained() {
        Err(error) => error,
        Ok(_) => panic!("RELEASE sur un canal fermé ne doit pas réussir"),
    };
    assert!(matches!(error, ManagedProcessError::Io(_)));
    assert_eq!(retained.child_mut().id(), pid);
    assert_eq!(retained.marker().marker(), &marker);
    assert!(marker_path.exists());
    assert!(retained.try_status().is_err());
    assert!(!retained.status_is_open());
    assert_eq!(
        retained.child_mut().wait().unwrap().code(),
        Some(ABANDONED_EXIT_CODE)
    );
    assert!(retained.reap_terminated_group().unwrap());
    assert!(!marker_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn eof_abandonne_tandis_que_release_exec_la_sonde() {
    let abandoned = spawn_test_bootstrap().wait_ready().unwrap();
    assert_eq!(abandoned.ready().pid, abandoned.ready().pgid);
    assert_eq!(
        abandoned.ready().birth,
        process_birth(abandoned.ready().pid).unwrap()
    );
    let mut child = abandoned.abandon();
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(ABANDONED_EXIT_CODE));

    let root = test_root("release");
    let store = ManagedMarkerStore::at_directory(root.join("managed"));
    let mut released = spawn_test_bootstrap()
        .wait_ready()
        .unwrap()
        .persist_marker(&store, "codex-1")
        .unwrap()
        .release()
        .unwrap();
    assert!(released.try_wait().unwrap().is_none());
    let probe = read_probe(&mut released);
    assert_eq!(probe["event"], "fd_probe");
    assert_eq!(probe["status_open"], true);
    assert_eq!(probe["release_open"], false);
    released.close_status();
    assert!(released.child_mut().wait().unwrap().success());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn annulation_avant_marqueur_termine_le_bootstrap_sans_exec() {
    let child = spawn_test_bootstrap();
    let pid = child.inner.child.id();
    let outcome = child
        .wait_ready_or_cancel(|| true, Duration::from_millis(10))
        .unwrap();

    assert!(outcome.is_none());
    wait_pid_gone(pid);
}

#[test]
fn stop_force_termine_l_intermediaire_npx_qui_ignore_l_annulation_et_son_descendant() {
    let root = test_root("stop-group");
    fs::create_dir_all(&root).unwrap();
    let descendant_path = root.join("descendant.pid");
    let fake_npx = root.join("npx");
    fs::write(&fake_npx, "#!/bin/sh\nsleep 30 &\necho $! > \"$1\"\nwait\n").unwrap();
    fs::set_permissions(&fake_npx, fs::Permissions::from_mode(0o700)).unwrap();
    let store = ManagedMarkerStore::at_directory(root.join("managed"));
    let mut child = spawn_test_bootstrap_with_wrapper(
        fake_npx,
        vec![descendant_path.to_string_lossy().into_owned()],
    )
    .wait_ready()
    .unwrap()
    .persist_marker(&store, "codex-1")
    .unwrap()
    .release()
    .unwrap();
    let descendant = wait_for_path(&descendant_path, |content| {
        std::str::from_utf8(content)
            .ok()?
            .trim()
            .parse::<u32>()
            .ok()
    });

    assert!(matches!(
        child
            .stop_group(
                Duration::ZERO,
                Duration::from_secs(2),
                Duration::from_millis(10),
            )
            .unwrap(),
        ManagedStopResult::StoppedForced { survivors_killed }
            if survivors_killed >= 2
    ));
    assert!(matches!(
        store.load("codex-1"),
        Err(ManagedProcessError::Io(ref error))
            if error.kind() == io::ErrorKind::NotFound
    ));
    wait_pid_gone(descendant);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn marqueur_perime_est_retire_sans_signaler_le_pid_recycle() {
    let root = test_root("stale-marker");
    let store = ManagedMarkerStore::at_directory(root.join("managed"));
    // Même avec le mutant qui ignore la naissance, le seul groupe
    // signalable est cet enfant possédé, JAMAIS le groupe du runner.
    let child = spawn_test_bootstrap().wait_ready().unwrap();
    let mut ready = child.ready().clone();
    assert_eq!(ready.pid, ready.pgid);
    // SAFETY: getpgrp ne déréférence aucun pointeur ; ses arguments sont des entiers.
    assert_ne!(ready.pgid, unsafe { libc::getpgrp() } as u32);
    ready.birth = ready.birth.saturating_add(1);
    store.persist("codex-1", &ready).unwrap();
    let result = store.stop_current_group(
        "codex-1",
        Duration::from_millis(50),
        Duration::from_millis(5),
    );
    let status = child.abandon().wait().unwrap();
    assert_eq!(result.unwrap(), None);
    assert_eq!(
        status.code(),
        Some(ABANDONED_EXIT_CODE),
        "aucun signal au PID périmé"
    );
    assert!(matches!(
        store.load("codex-1"),
        Err(ManagedProcessError::Io(ref error))
            if error.kind() == io::ErrorKind::NotFound
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn reconciliation_termine_un_groupe_reel_avant_de_supprimer_son_marqueur() {
    let root = test_root("reconcile-real-group");
    fs::create_dir_all(&root).unwrap();
    let store = ManagedMarkerStore::at_directory(root.join("managed"));
    let mut running = spawn_test_bootstrap_with_wrapper(
        PathBuf::from("/bin/sh"),
        vec![
            "-c".to_string(),
            "trap '' TERM; while :; do sleep 1; done".to_string(),
        ],
    )
    .wait_ready()
    .unwrap()
    .persist_marker(&store, "codex-reconcile")
    .unwrap()
    .release()
    .unwrap();
    let pgid = running.marker().marker().pgid;
    let reaper = thread::spawn(move || running.child_mut().wait().unwrap());

    assert_eq!(
        store
            .reconcile_stale_groups(Duration::from_secs(2), Duration::from_millis(10))
            .unwrap(),
        ["codex-reconcile"]
    );
    assert!(!group_exists(pgid).unwrap());
    let _ = reaper.join().unwrap();
    assert!(matches!(
        store.load("codex-reconcile"),
        Err(ManagedProcessError::Io(ref error))
            if error.kind() == io::ErrorKind::NotFound
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stderr_est_prive_separe_par_generation_et_purge_par_age() {
    let root = test_root("stderr");
    let store = ManagedStderrStore::at_directory(root.join("stderr"));
    let (mut file, first) = store.open("codex-1", &identity()).unwrap();
    writeln!(file, "diagnostic privé").unwrap();
    file.flush().unwrap();
    let second_identity = ManagedIdentity {
        generation: identity().generation + 1,
        ..identity()
    };
    let (_second, second) = store.open("codex-1", &second_identity).unwrap();
    assert_ne!(first.parent(), second.parent());
    for directory in [
        first.parent().unwrap(),
        first.parent().unwrap().parent().unwrap(),
        first.parent().unwrap().parent().unwrap().parent().unwrap(),
    ] {
        assert_eq!(
            fs::metadata(directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    assert_eq!(
        fs::metadata(&first).unwrap().permissions().mode() & 0o777,
        0o600
    );
    drop(file);
    assert_eq!(store.purge_older_than_days(0).unwrap(), 2);
    assert!(!first.exists());
    assert!(!second.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stderr_refuse_un_repertoire_existant_aux_permissions_trop_larges() {
    let root = test_root("stderr-permissions");
    let directory = root.join("stderr");
    fs::create_dir_all(&directory).unwrap();
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).unwrap();
    let store = ManagedStderrStore::at_directory(&directory);

    let error = store.open("codex-1", &identity()).unwrap_err();
    assert!(matches!(
        error,
        ManagedProcessError::Io(ref source)
            if source.kind() == io::ErrorKind::PermissionDenied
    ));
    assert_eq!(
        fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
        0o755
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stderr_refuse_un_fichier_existant_non_regulier_ou_trop_permissif() {
    let root = test_root("stderr-file-permissions");
    let store = ManagedStderrStore::at_directory(root.join("stderr"));
    let (file, path) = store.open("codex-1", &identity()).unwrap();
    drop(file);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

    let error = store.open("codex-1", &identity()).unwrap_err();
    assert!(matches!(
        error,
        ManagedProcessError::Io(ref source)
            if source.kind() == io::ErrorKind::PermissionDenied
    ));
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o644
    );
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(matches!(
        store.open("codex-1", &identity()),
        Err(ManagedProcessError::Io(ref source))
            if source.kind() == io::ErrorKind::PermissionDenied
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn sous_mode_parse_une_commande_wrapper_sans_confondre_ses_arguments() {
    let request = parse_bootstrap_args(&[
        "--instance-id".to_string(),
        "instance-9".to_string(),
        "--command-id".to_string(),
        "command-9".to_string(),
        "--generation".to_string(),
        "9".to_string(),
        "--".to_string(),
        "/usr/bin/agent".to_string(),
        "--name".to_string(),
        "codex-9".to_string(),
    ])
    .unwrap();
    assert_eq!(request.identity.generation, 9);
    assert_eq!(request.wrapper_executable, Path::new("/usr/bin/agent"));
    assert_eq!(request.wrapper_args, ["--name", "codex-9"]);
}

#[test]
fn marqueur_prive_roundtrip_et_refuse_un_nom_traversant() {
    let root = test_root("marker");
    let store = ManagedMarkerStore::at_directory(root.join("managed"));
    let ready = BootstrapReady {
        pid: 42,
        pgid: 42,
        birth: 123,
        instance_id: identity().instance_id,
        command_id: identity().command_id,
        generation: identity().generation,
    };
    let marker = store.persist("codex-1", &ready).unwrap();
    let path = marker.path().to_path_buf();
    assert_eq!(store.load("codex-1").unwrap(), ManagedMarker::from(&ready));
    assert_eq!(
        fs::metadata(path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(store.persist("../escape", &ready).is_err());
    store.remove("codex-1").unwrap();
    assert!(!path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
#[ignore = "processus enfant lancé par un autre test (variable CONTROLLER_ENV) ; sans effet lancé seul"]
fn crash_controller_child() {
    if std::env::var(CONTROLLER_ENV).ok().as_deref() != Some("1") {
        return;
    }
    let stage = std::env::var(CONTROLLER_STAGE_ENV).unwrap();
    let root = PathBuf::from(std::env::var_os(CONTROLLER_ROOT_ENV).unwrap());
    let ready = spawn_test_bootstrap().wait_ready().unwrap();
    let ready_info = ready.ready().clone();
    let marker_store = ManagedMarkerStore::at_directory(root.join("managed"));
    match stage.as_str() {
        "before_marker" => {
            write_crash_boundary(&root, &ready_info);
            let _held_ready = ready;
            loop {
                thread::park();
            }
        }
        "after_marker" => {
            let marked = ready.persist_marker(&marker_store, "codex-1").unwrap();
            write_crash_boundary(&root, &ready_info);
            let _held_marked = marked;
            loop {
                thread::park();
            }
        }
        "after_release" => {
            let mut running = ready
                .persist_marker(&marker_store, "codex-1")
                .unwrap()
                .release()
                .unwrap();
            let probe = read_probe(&mut running);
            assert_eq!(probe["status_open"], true);
            assert_eq!(probe["release_open"], false);
            write_crash_boundary(&root, &ready_info);
            let _held_running = running;
            loop {
                thread::park();
            }
        }
        unknown => panic!("frontière inconnue: {unknown}"),
    }
}

fn write_crash_boundary(root: &Path, ready: &BootstrapReady) {
    write_private_file_atomic(
        &root.join("boundary.json"),
        serde_json::to_string(ready).unwrap().as_bytes(),
    )
    .unwrap();
}

fn wait_for_path<T>(path: &Path, parse: impl Fn(&[u8]) -> Option<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(content) = fs::read(path)
            && !content.is_empty()
            && let Some(value) = parse(&content)
        {
            return value;
        }
        assert!(
            Instant::now() < deadline,
            "barrière absente: {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_pid_gone(pid: u32) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
        let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
        if result < 0 && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "processus descendant {pid} encore vivant"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn terminate_controller(child: &mut Child) -> ExitStatus {
    // Ce PID est celui du processus de test créé juste au-dessus ; son
    // exécutable a été vérifié différent de Firefox avant le spawn.
    assert_eq!(
        // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
        unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) },
        0
    );
    child.wait().unwrap()
}

#[test]
fn crash_reel_aux_trois_frontieres_preserve_la_preuve_attendue() {
    for stage in ["before_marker", "after_marker", "after_release"] {
        let root = test_root(stage);
        let boundary = root.join("boundary.json");
        let mut controller = helper_command("managed_process::tests::crash_controller_child");
        controller
            .env(CONTROLLER_ENV, "1")
            .env(CONTROLLER_STAGE_ENV, stage)
            .env(CONTROLLER_ROOT_ENV, &root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut controller = controller.spawn().unwrap();
        let ready: BootstrapReady =
            wait_for_path(&boundary, |content| serde_json::from_slice(content).ok());
        let status = terminate_controller(&mut controller);
        assert!(!status.success());
        wait_pid_gone(ready.pid);

        let marker = root.join("managed/codex-1.json");
        assert_eq!(
            marker.exists(),
            stage != "before_marker",
            "frontière {stage}"
        );
        let _ = fs::remove_dir_all(root);
    }
}
