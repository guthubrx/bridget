use bridget_daemon::{
    project_runtime::{
        CONTAINER_HOME, CONTAINER_INGRESS_DIRECTORY, CONTAINER_INGRESS_SOCKET,
        CONTAINER_STATE_ROOT, DockerCli, DockerRuntimeLaunch, ImageReferenceKind,
        ProjectEnvironment, ProjectEnvironmentState, ProjectMount, ProjectRuntimePolicy,
        bind_runtime_ingress, prepare_environment, remove_environment, stop_environment,
    },
    registry::AgentRegistry,
};
use bridget_transport::protocol::{DaemonToWrapper, WrapperToDaemon, decode, encode};
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

fn fixture_image_id() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("racine dépôt");
    let iidfile =
        std::env::temp_dir().join(format!("runtime066-fixture-{}.iid", uuid::Uuid::new_v4()));
    let bridget_binary = PathBuf::from(env!("CARGO_BIN_EXE_bridget"));
    let binary_context = bridget_binary.parent().expect("répertoire binaire Bridget");
    let output = Command::new("docker")
        .args(["build", "--build-context"])
        .arg(format!("bridget-bin={}", binary_context.display()))
        .arg("--file")
        .arg(root.join("infra/project-runtime/Dockerfile"))
        .arg("--iidfile")
        .arg(&iidfile)
        .arg(root.join("infra/project-runtime"))
        .output()
        .expect("docker build fixture runtime");
    assert!(
        output.status.success(),
        "docker build fixture runtime: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let image_id = std::fs::read_to_string(&iidfile)
        .expect("iidfile fixture")
        .trim()
        .to_string();
    let _ = std::fs::remove_file(iidfile);
    assert!(image_id.starts_with("sha256:"));
    image_id
}

struct ContainerCleanup(Option<String>);

impl ContainerCleanup {
    fn remove(&mut self) {
        let Some(container_id) = self.0.take() else {
            return;
        };
        let removed = Command::new("docker")
            .args(["rm", "--force", &container_id])
            .output()
            .expect("docker rm fixture");
        assert!(removed.status.success());
    }
}

impl Drop for ContainerCleanup {
    fn drop(&mut self) {
        let Some(container_id) = self.0.take() else {
            return;
        };
        let _ = Command::new("docker")
            .args(["rm", "--force", &container_id])
            .output();
    }
}

fn docker_exec(container_id: &str, user: &str, arguments: &[&str]) -> std::process::Output {
    Command::new("docker")
        .args(["exec", "--user", user, container_id])
        .args(arguments)
        .output()
        .expect("docker exec fixture")
}

#[test]
fn spec_066_prepare_docker_atteste_image_montages_abi_et_limites() {
    let unique = format!("runtime066-{}", uuid::Uuid::new_v4());
    let root = std::env::temp_dir().join(&unique);
    let project_root = root.join("project");
    let state_parent = root.join("state");
    let state_root = state_parent.join("project-066");
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::create_dir_all(&state_root).unwrap();
    let source_file = project_root.join("source.txt");
    let durable_file = state_root.join("durable.txt");
    let private_fixture = state_root.join("private-fixture");
    std::fs::write(&source_file, "source-before-runtime").unwrap();
    std::fs::write(&durable_file, "state-before-runtime").unwrap();
    std::fs::write(&private_fixture, "private-fixture-content").unwrap();
    std::fs::set_permissions(&private_fixture, std::fs::Permissions::from_mode(0o600)).unwrap();

    let policy = ProjectRuntimePolicy {
        policy_id: "integration-local".to_string(),
        policy_version: 1,
        image_reference_kind: ImageReferenceKind::LocalImageId,
        image_reference: fixture_image_id(),
        run_as_uid: unsafe { libc::geteuid() },
        run_as_gid: unsafe { libc::getegid() },
        cpu_limit: 1.0,
        memory_limit_bytes: 128 * 1024 * 1024,
        pids_limit: 64,
        tmpfs: vec!["/tmp".to_string()],
        network_mode: "bridge".to_string(),
        runtime_launcher: None,
        runtime_executables: std::collections::BTreeMap::new(),
        state_root_parent: state_parent,
        digest: format!("sha256:{}", "a".repeat(64)),
    };
    let mut environment = ProjectEnvironment::absent("project-066", 1, &policy).unwrap();
    let ingress = bind_runtime_ingress(&policy, &environment).expect("ingress privé");
    let mounts = vec![
        ProjectMount {
            host_path: project_root.clone(),
            container_path: project_root.display().to_string(),
            writable: true,
        },
        ProjectMount {
            host_path: state_root.clone(),
            container_path: CONTAINER_STATE_ROOT.to_string(),
            writable: true,
        },
        ProjectMount {
            host_path: ingress.mount_directory.clone(),
            container_path: CONTAINER_INGRESS_DIRECTORY.to_string(),
            writable: false,
        },
    ];
    let docker = DockerCli::new(PathBuf::from("docker"), Duration::from_secs(20));
    let inspection =
        prepare_environment(&docker, &mut environment, &policy, &mounts).expect("prepare docker");
    let mut cleanup = ContainerCleanup(environment.container_id.clone());

    assert_eq!(environment.state, ProjectEnvironmentState::Ready);
    let first_id = environment.container_id.clone().expect("container id");
    let expected_user = format!("{}:{}", policy.run_as_uid, policy.run_as_gid);
    assert_eq!(
        inspection
            .pointer("/Config/User")
            .and_then(serde_json::Value::as_str),
        Some(expected_user.as_str())
    );
    let env = inspection
        .pointer("/Config/Env")
        .and_then(serde_json::Value::as_array)
        .expect("environment");
    assert!(
        env.iter()
            .any(|entry| entry.as_str() == Some(&format!("HOME={CONTAINER_HOME}")))
    );
    assert!(
        env.iter().any(|entry| entry.as_str()
            == Some(&format!("XDG_STATE_HOME={CONTAINER_STATE_ROOT}/xdg/state")))
    );
    assert_eq!(
        inspection.pointer("/HostConfig/ReadonlyRootfs"),
        Some(&serde_json::Value::Bool(true))
    );
    let uid = docker_exec(&first_id, &expected_user, &["id", "-u"]);
    let gid = docker_exec(&first_id, &expected_user, &["id", "-g"]);
    assert!(uid.status.success() && gid.status.success());
    assert_eq!(
        String::from_utf8(uid.stdout).unwrap().trim(),
        policy.run_as_uid.to_string()
    );
    assert_eq!(
        String::from_utf8(gid.stdout).unwrap().trim(),
        policy.run_as_gid.to_string()
    );
    let runtime_env = docker_exec(&first_id, &expected_user, &["env"]);
    assert!(runtime_env.status.success());
    let runtime_env = String::from_utf8(runtime_env.stdout).unwrap();
    for expected in [
        format!("HOME={CONTAINER_HOME}"),
        format!("XDG_CONFIG_HOME={CONTAINER_STATE_ROOT}/xdg/config"),
        format!("XDG_CACHE_HOME={CONTAINER_STATE_ROOT}/xdg/cache"),
        format!("XDG_DATA_HOME={CONTAINER_STATE_ROOT}/xdg/data"),
        format!("XDG_STATE_HOME={CONTAINER_STATE_ROOT}/xdg/state"),
        format!("BRIDGET_RUNTIME_SOCKET={CONTAINER_INGRESS_SOCKET}"),
    ] {
        assert!(runtime_env.lines().any(|line| line == expected));
    }
    let private_read = docker_exec(
        &first_id,
        &expected_user,
        &["cat", "/var/lib/bridget-project/private-fixture"],
    );
    assert!(private_read.status.success());
    assert_eq!(private_read.stdout, b"private-fixture-content");
    assert!(
        docker_exec(
            &first_id,
            &expected_user,
            &["touch", "/var/lib/bridget-project/agent-wrote"],
        )
        .status
        .success()
    );
    assert!(state_root.join("agent-wrote").is_file());
    assert!(
        docker_exec(
            &first_id,
            &expected_user,
            &["test", "-S", CONTAINER_INGRESS_SOCKET],
        )
        .status
        .success()
    );

    stop_environment(&docker, &mut environment).expect("arrêt Docker explicite");
    assert_eq!(environment.state, ProjectEnvironmentState::Stopped);
    assert_eq!(environment.container_id.as_deref(), Some(first_id.as_str()));
    assert!(
        !docker_exec(&first_id, &expected_user, &["true"])
            .status
            .success(),
        "le conteneur arrêté ne doit plus accepter d'exec"
    );
    remove_environment(&docker, &mut environment).expect("suppression Docker explicite");
    assert_eq!(environment.state, ProjectEnvironmentState::Absent);

    let recreated =
        prepare_environment(&docker, &mut environment, &policy, &mounts).expect("recreate docker");
    cleanup.0 = environment.container_id.clone();
    assert_eq!(environment.state, ProjectEnvironmentState::Ready);
    assert!(recreated.get("Id").is_some());
    assert_eq!(
        std::fs::read_to_string(&source_file).unwrap(),
        "source-before-runtime"
    );

    assert_eq!(
        std::fs::read_to_string(&durable_file).unwrap(),
        "state-before-runtime"
    );

    cleanup.remove();
    drop(ingress);
    let _ = std::fs::remove_dir_all(root);
}
#[test]
fn spec_066_docker_exec_lance_bridget_et_un_fournisseur_de_fixture_isole() {
    let unique = format!("r66e-{}", uuid::Uuid::new_v4().simple());
    let root = PathBuf::from("/tmp").join(&unique);
    let project_root = root.join("project");
    let state_parent = root.join("state");
    let state_root = state_parent.join("project-exec-066");
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::create_dir_all(&state_root).unwrap();

    let policy = ProjectRuntimePolicy {
        policy_id: "integration-exec".to_string(),
        policy_version: 1,
        image_reference_kind: ImageReferenceKind::LocalImageId,
        image_reference: fixture_image_id(),
        run_as_uid: unsafe { libc::geteuid() },
        run_as_gid: unsafe { libc::getegid() },
        cpu_limit: 1.0,
        memory_limit_bytes: 128 * 1024 * 1024,
        pids_limit: 64,
        tmpfs: vec!["/tmp".to_string()],
        network_mode: "bridge".to_string(),
        runtime_launcher: Some("/usr/local/bin/bridget".to_string()),
        runtime_executables: std::collections::BTreeMap::from([(
            "fixture".to_string(),
            "/usr/local/bin/fixture-agent".to_string(),
        )]),
        state_root_parent: state_parent.clone(),
        digest: format!("sha256:{}", "b".repeat(64)),
    };
    let mut environment = ProjectEnvironment::absent("project-exec-066", 2, &policy).unwrap();
    let ingress = bind_runtime_ingress(&policy, &environment).unwrap();
    let mounts = vec![
        ProjectMount {
            host_path: project_root.clone(),
            container_path: project_root.display().to_string(),
            writable: true,
        },
        ProjectMount {
            host_path: state_root,
            container_path: CONTAINER_STATE_ROOT.to_string(),
            writable: true,
        },
        ProjectMount {
            host_path: ingress.mount_directory.clone(),
            container_path: CONTAINER_INGRESS_DIRECTORY.to_string(),
            writable: false,
        },
    ];
    let docker = DockerCli::new(PathBuf::from("docker"), Duration::from_secs(20));
    prepare_environment(&docker, &mut environment, &policy, &mounts).unwrap();
    let mut cleanup = ContainerCleanup(environment.container_id.clone());
    let reservation = environment.reserve_spawn().unwrap();
    let container_id = environment.container_id.clone().unwrap();
    let definition = AgentRegistry::from_json(
        r#"{"agents":{"fixture":{"command":"/never/on/the/host","protocol":"acp"}}}"#,
        "/tmp/fixture-only.json",
    )
    .unwrap()
    .resolved_definition("fixture")
    .unwrap();
    let instance_id = uuid::Uuid::new_v4().to_string();
    let launch = DockerRuntimeLaunch {
        reservation: reservation.clone(),
        container_id: container_id.clone(),
        run_as_uid: policy.run_as_uid,
        run_as_gid: policy.run_as_gid,
        execution: policy.runtime_execution("fixture").unwrap(),
        exec_id: uuid::Uuid::new_v4().to_string(),
        agent_type: "fixture".to_string(),
        agent_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        instance_id: instance_id.clone(),
        agent_generation: 7,
        cwd: project_root,
        resolved_definition_json: serde_json::to_string(&definition).unwrap(),
        process_env: Vec::new(),
    };

    let (registered_tx, registered_rx) = std::sync::mpsc::sync_channel(1);
    let server_container_id = container_id.clone();
    let listener = ingress.listener;
    let server = thread::spawn(move || -> Result<(), String> {
        listener
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        let deadline = Instant::now() + Duration::from_secs(5);
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err("connexion runtime absente".to_string());
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(error.to_string()),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|error| error.to_string())?;
        let read = stream.try_clone().map_err(|error| error.to_string())?;
        let mut reader = BufReader::new(read);
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        match decode(line.trim()).map_err(|error| error.to_string())? {
            WrapperToDaemon::RuntimeIngressHello { hello }
                if hello.project_id == reservation.project_id
                    && hello.binding_generation == reservation.binding_generation
                    && hello.container_id == server_container_id
                    && hello.environment_epoch == reservation.environment_epoch
                    && hello.agent_generation == 7 => {}
            other => return Err(format!("hello runtime inattendu: {other:?}")),
        }
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::RuntimeIngressAccepted {
                project_id: reservation.project_id,
                binding_generation: reservation.binding_generation,
                environment_epoch: reservation.environment_epoch,
            })
            .map_err(|error| error.to_string())?
        )
        .map_err(|error| error.to_string())?;
        writer.flush().map_err(|error| error.to_string())?;
        line.clear();
        reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        match decode(line.trim()).map_err(|error| error.to_string())? {
            WrapperToDaemon::Register {
                agent_type,
                instance_id: Some(observed_instance),
                ..
            } if agent_type == "fixture" && observed_instance == instance_id => {}
            other => return Err(format!("register runtime inattendu: {other:?}")),
        }
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                agent_id: "fixture-runtime-066".to_string(),
            })
            .map_err(|error| error.to_string())?
        )
        .map_err(|error| error.to_string())?;
        writer.flush().map_err(|error| error.to_string())?;
        registered_tx.send(()).map_err(|error| error.to_string())?;
        line.clear();
        reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        Ok(())
    });

    let mut child = docker
        .spawn_runtime_with_stderr(&launch, Stdio::piped())
        .unwrap();
    registered_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("le wrapper runtime doit s'inscrire");
    docker.stop_runtime(&launch).expect("arrêt runtime ciblé");
    let exit_deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        assert!(
            Instant::now() < exit_deadline,
            "wrapper runtime doit se terminer"
        );
        thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "wrapper runtime: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let server_result = server.join().unwrap();
    assert!(server_result.is_ok(), "serveur ingress: {server_result:?}");
    // Le répertoire ingress est monté, pas l'inode de la socket : après la
    // reprise du daemon, le conteneur voit le nouvel endpoint au même chemin.
    let rebound = bind_runtime_ingress(&policy, &environment).expect("ingress recréé");
    assert_eq!(
        rebound.mount_directory,
        state_parent.join("project-exec-066/runtime/2")
    );
    let visible = docker_exec(
        &container_id,
        &format!("{}:{}", policy.run_as_uid, policy.run_as_gid),
        &["test", "-S", CONTAINER_INGRESS_SOCKET],
    );
    assert!(
        visible.status.success(),
        "socket recréée visible dans le conteneur"
    );
    drop(rebound);
    cleanup.remove();
    let _ = std::fs::remove_dir_all(root);
}
