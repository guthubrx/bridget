use bridget_daemon::{
    project_runtime::{
        CONTAINER_INGRESS_DIRECTORY, CONTAINER_INGRESS_SOCKET, CONTAINER_STATE_ROOT, DockerCli,
        DockerRuntimeLaunch, ImageReferenceKind, ProjectEnvironment, ProjectMount,
        ProjectRuntimePolicy, RuntimeIngressEndpoint, bind_runtime_ingress, prepare_environment,
    },
    registry::AgentRegistry,
};
use bridget_transport::protocol::{DaemonToWrapper, WrapperToDaemon, decode, encode};
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc::{Receiver, SyncSender},
    thread,
    time::{Duration, Instant},
};

fn fixture_image_id() -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("racine dépôt");
    let iidfile = std::env::temp_dir().join(format!("r66-agents-{}.iid", uuid::Uuid::new_v4()));
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
    image_id
}

struct ContainerCleanup(Option<String>);

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

struct ExpectedAgent {
    name: String,
    instance_id: String,
    generation: u64,
}

#[allow(clippy::too_many_arguments)]
fn accept_agents(
    listener: UnixListener,
    project_id: String,
    binding_generation: u64,
    environment_epoch: u64,
    container_id: String,
    mut expected: Vec<ExpectedAgent>,
    registered: SyncSender<()>,
    stop: Receiver<()>,
) -> Result<(), String> {
    listener
        .set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut channels: Vec<(BufReader<UnixStream>, BufWriter<UnixStream>)> = Vec::new();
    while !expected.is_empty() {
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err("inscription runtime manquante".to_string());
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(error.to_string()),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .map_err(|error| error.to_string())?;
        let read = stream.try_clone().map_err(|error| error.to_string())?;
        let write = stream.try_clone().map_err(|error| error.to_string())?;
        let mut reader = BufReader::new(read);
        let mut writer = BufWriter::new(write);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        let hello = match decode(line.trim()).map_err(|error| error.to_string())? {
            WrapperToDaemon::RuntimeIngressHello { hello } => hello,
            other => return Err(format!("hello runtime inattendu: {other:?}")),
        };
        let expected_index = expected
            .iter()
            .position(|agent| agent.instance_id == hello.instance_id)
            .ok_or_else(|| "instance runtime non attendue".to_string())?;
        let agent = expected.remove(expected_index);
        if hello.project_id != project_id
            || hello.binding_generation != binding_generation
            || hello.environment_epoch != environment_epoch
            || hello.container_id != container_id
            || hello.agent_generation != agent.generation
        {
            return Err("identité runtime différente de la réservation".to_string());
        }
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::RuntimeIngressAccepted {
                project_id: project_id.clone(),
                binding_generation,
                environment_epoch,
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
                name: Some(name),
                instance_id: Some(instance_id),
                ..
            } if agent_type == "fixture"
                && name == agent.name
                && instance_id == agent.instance_id => {}
            other => return Err(format!("register runtime inattendu: {other:?}")),
        }
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered { name: agent.name })
                .map_err(|error| error.to_string())?
        )
        .map_err(|error| error.to_string())?;
        writer.flush().map_err(|error| error.to_string())?;
        registered.send(()).map_err(|error| error.to_string())?;
        channels.push((reader, writer));
    }
    stop.recv_timeout(Duration::from_secs(8))
        .map_err(|error| error.to_string())?;
    for (mut reader, _) in channels {
        let mut line = String::new();
        let _ = reader.read_line(&mut line);
    }
    Ok(())
}

fn policy(image_reference: String, state_root_parent: PathBuf) -> ProjectRuntimePolicy {
    ProjectRuntimePolicy {
        policy_id: "integration-agents".to_string(),
        policy_version: 1,
        image_reference_kind: ImageReferenceKind::LocalImageId,
        image_reference,
        run_as_uid: unsafe { libc::geteuid() },
        run_as_gid: unsafe { libc::getegid() },
        cpu_limit: 1.0,
        memory_limit_bytes: 128 * 1024 * 1024,
        pids_limit: 64,
        tmpfs: vec!["/tmp".to_string()],
        runtime_launcher: Some("/usr/local/bin/bridget".to_string()),
        runtime_executables: std::collections::BTreeMap::from([(
            "fixture".to_string(),
            "/usr/local/bin/fixture-agent".to_string(),
        )]),
        state_root_parent,
        digest: format!("sha256:{}", "c".repeat(64)),
    }
}

fn mounts(
    project_root: PathBuf,
    state_root: PathBuf,
    ingress: &RuntimeIngressEndpoint,
) -> Vec<ProjectMount> {
    vec![
        ProjectMount {
            container_path: project_root.display().to_string(),
            host_path: project_root,
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
    ]
}

fn launch(
    environment: &ProjectEnvironment,
    policy: &ProjectRuntimePolicy,
    project_root: PathBuf,
    name: &str,
    generation: u64,
    instance_id: String,
) -> DockerRuntimeLaunch {
    let definition = AgentRegistry::from_json(
        r#"{"agents":{"fixture":{"command":"/never/on/the/host","protocol":"acp"}}}"#,
        "/tmp/fixture-only.json",
    )
    .expect("registre fixture")
    .resolved_definition("fixture")
    .expect("définition fixture");
    DockerRuntimeLaunch {
        reservation: environment.reserve_spawn().expect("réservation runtime"),
        container_id: environment.container_id.clone().expect("container attesté"),
        run_as_uid: policy.run_as_uid,
        run_as_gid: policy.run_as_gid,
        execution: policy
            .runtime_execution("fixture")
            .expect("commande interne"),
        exec_id: uuid::Uuid::new_v4().to_string(),
        agent_type: "fixture".to_string(),
        agent_name: name.to_string(),
        instance_id,
        agent_generation: generation,
        cwd: project_root,
        resolved_definition_json: serde_json::to_string(&definition).expect("définition gelée"),
    }
}

fn assert_exit(child: &mut Child, what: &str) {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if let Some(status) = child.try_wait().expect("état docker exec") {
            assert!(status.success(), "{what} doit sortir proprement: {status}");
            return;
        }
        assert!(Instant::now() < deadline, "{what} doit s'arrêter");
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn spec_066_deux_agents_partagent_un_conteneur_sans_melanger_deux_projets() {
    let root = PathBuf::from("/tmp").join(format!("r66-agents-{}", uuid::Uuid::new_v4().simple()));
    let project_a = root.join("project-a");
    let project_b = root.join("project-b");
    let state_parent = root.join("state");
    let state_a = state_parent.join("project-a");
    let state_b = state_parent.join("project-b");
    std::fs::create_dir_all(&project_a).unwrap();
    std::fs::create_dir_all(&project_b).unwrap();
    std::fs::create_dir_all(&state_a).unwrap();
    std::fs::create_dir_all(&state_b).unwrap();

    let policy = policy(fixture_image_id(), state_parent);
    let docker = DockerCli::new(PathBuf::from("docker"), Duration::from_secs(20));

    let mut environment_a = ProjectEnvironment::absent("project-a", 2, &policy).unwrap();
    let ingress_a = bind_runtime_ingress(&policy, &environment_a).unwrap();
    prepare_environment(
        &docker,
        &mut environment_a,
        &policy,
        &mounts(project_a.clone(), state_a, &ingress_a),
    )
    .unwrap();
    let _cleanup_a = ContainerCleanup(environment_a.container_id.clone());

    let mut environment_b = ProjectEnvironment::absent("project-b", 3, &policy).unwrap();
    let ingress_b = bind_runtime_ingress(&policy, &environment_b).unwrap();
    prepare_environment(
        &docker,
        &mut environment_b,
        &policy,
        &mounts(project_b.clone(), state_b, &ingress_b),
    )
    .unwrap();
    let _cleanup_b = ContainerCleanup(environment_b.container_id.clone());
    assert_ne!(environment_a.container_id, environment_b.container_id);
    assert_ne!(ingress_a.socket_path, ingress_b.socket_path);
    assert!(
        Command::new("docker")
            .args([
                "exec",
                environment_a.container_id.as_deref().unwrap(),
                "test",
                "-S",
                CONTAINER_INGRESS_SOCKET,
            ])
            .status()
            .unwrap()
            .success()
    );

    let first_instance = uuid::Uuid::new_v4().to_string();
    let second_instance = uuid::Uuid::new_v4().to_string();
    let first = launch(
        &environment_a,
        &policy,
        project_a.clone(),
        "fixture-agent-a",
        7,
        first_instance.clone(),
    );
    let second = launch(
        &environment_a,
        &policy,
        project_a,
        "fixture-agent-b",
        8,
        second_instance.clone(),
    );
    let first_name = first.agent_name.clone();
    let first_generation = first.agent_generation;
    let second_name = second.agent_name.clone();
    let second_generation = second.agent_generation;
    let (registered_tx, registered_rx) = std::sync::mpsc::sync_channel(2);
    let (stop_tx, stop_rx) = std::sync::mpsc::sync_channel(1);
    let server = thread::spawn({
        let project_id = environment_a.project_id.clone();
        let binding_generation = environment_a.binding_generation;
        let environment_epoch = environment_a.environment_epoch;
        let container_id = environment_a.container_id.clone().unwrap();
        move || {
            accept_agents(
                ingress_a.listener,
                project_id,
                binding_generation,
                environment_epoch,
                container_id,
                vec![
                    ExpectedAgent {
                        name: first_name,
                        instance_id: first_instance,
                        generation: first_generation,
                    },
                    ExpectedAgent {
                        name: second_name,
                        instance_id: second_instance,
                        generation: second_generation,
                    },
                ],
                registered_tx,
                stop_rx,
            )
        }
    });

    let mut first_child = docker
        .spawn_runtime_with_stderr(&first, Stdio::piped())
        .expect("premier docker exec");
    let mut second_child = docker
        .spawn_runtime_with_stderr(&second, Stdio::piped())
        .expect("second docker exec");
    registered_rx
        .recv_timeout(Duration::from_secs(8))
        .expect("premier agent inscrit");
    registered_rx
        .recv_timeout(Duration::from_secs(8))
        .expect("second agent inscrit");
    assert!(first_child.try_wait().unwrap().is_none());
    assert!(second_child.try_wait().unwrap().is_none());

    docker.stop_runtime(&first).expect("arrêt du premier agent");
    assert_exit(&mut first_child, "premier agent");
    assert!(
        second_child.try_wait().unwrap().is_none(),
        "l'arrêt ciblé du premier agent ne doit pas toucher le second"
    );
    docker.stop_runtime(&second).expect("arrêt du second agent");
    assert_exit(&mut second_child, "second agent");
    stop_tx.send(()).unwrap();
    assert!(server.join().unwrap().is_ok());

    drop(ingress_b);
    let _ = std::fs::remove_dir_all(root);
}
