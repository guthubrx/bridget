use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn fixture_root(label: &str) -> PathBuf {
    let base = if Path::new("/private/tmp").is_dir() {
        Path::new("/private/tmp")
    } else {
        Path::new("/tmp")
    };
    base.join(format!(
        "b96-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ))
}

fn isolated_binary(root: &Path) -> PathBuf {
    fs::create_dir(root).unwrap();
    fs::set_permissions(root, fs::Permissions::from_mode(0o700)).unwrap();
    let binary = root.join("bridget-alone");
    fs::copy(env!("CARGO_BIN_EXE_bridget"), &binary).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
    binary
}

fn run(binary: &Path, root: &Path, args: &[&str]) -> Output {
    Command::new(binary)
        .args(args)
        .env_clear()
        .env("HOME", root.join("home"))
        .env("TMPDIR", root.join("tmp"))
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn spec096_embedded_help_works_without_adjacent_script_and_cleans_temp() {
    let root = fixture_root("standalone");
    let binary = isolated_binary(&root);
    fs::create_dir(root.join("home")).unwrap();
    fs::create_dir(root.join("tmp")).unwrap();
    let output = run(&binary, &root, &["federate", "--help"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bridget federate"));
    assert!(stdout.contains("Le port vaut 22 par défaut"));
    assert!(!root.join("scripts/federate-ssh.sh").exists());
    assert_eq!(fs::read_dir(root.join("tmp")).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec096_invalid_destination_preserves_argument_error() {
    let root = fixture_root("invalid");
    let binary = isolated_binary(&root);
    fs::create_dir(root.join("home")).unwrap();
    fs::create_dir(root.join("tmp")).unwrap();
    let output = run(&binary, &root, &["federate", "http://exemple.test"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("ssh://"));
    assert_eq!(fs::read_dir(root.join("tmp")).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec096_top_level_help_exposes_federate() {
    let root = fixture_root("help");
    let binary = isolated_binary(&root);
    fs::create_dir(root.join("home")).unwrap();
    fs::create_dir(root.join("tmp")).unwrap();
    let output = run(&binary, &root, &["help"]);
    assert!(String::from_utf8_lossy(&output.stderr).contains("federate"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec096_concurrent_help_invocations_leave_no_shared_temp_artifact() {
    let root = fixture_root("parallel");
    let binary = isolated_binary(&root);
    fs::create_dir(root.join("home")).unwrap();
    fs::create_dir(root.join("tmp")).unwrap();
    let first = Command::new(&binary)
        .args(["federate", "--help"])
        .env_clear()
        .env("HOME", root.join("home"))
        .env("TMPDIR", root.join("tmp"))
        .env("PATH", "/usr/bin:/bin")
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let second = Command::new(&binary)
        .args(["federate", "--help"])
        .env_clear()
        .env("HOME", root.join("home"))
        .env("TMPDIR", root.join("tmp"))
        .env("PATH", "/usr/bin:/bin")
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    assert!(first.wait_with_output().unwrap().status.success());
    assert!(second.wait_with_output().unwrap().status.success());
    assert_eq!(fs::read_dir(root.join("tmp")).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec096_preserves_the_embedded_manager_exit_code() {
    let root = fixture_root("exit-code");
    let binary = isolated_binary(&root);
    for directory in ["home", "tmp", "tools", "local"] {
        fs::create_dir(root.join(directory)).unwrap();
    }
    let local_root = root.join("local");
    fs::set_permissions(&local_root, fs::Permissions::from_mode(0o700)).unwrap();
    let identity = root.join("identity");
    let known_hosts = root.join("known_hosts");
    for file in [&identity, &known_hosts] {
        fs::write(file, "fixture\n").unwrap();
        fs::set_permissions(file, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let ssh = root.join("tools/ssh");
    fs::write(&ssh, "#!/bin/sh\nexit 37\n").unwrap();
    fs::set_permissions(&ssh, fs::Permissions::from_mode(0o700)).unwrap();
    let systemctl = root.join("tools/systemctl");
    fs::write(
        &systemctl,
        "#!/bin/sh\ncase \"$*\" in\n  *'show --property=LoadState --value'*) echo not-found ;;\n  *) exit 0 ;;\nesac\n",
    )
    .unwrap();
    fs::set_permissions(&systemctl, fs::Permissions::from_mode(0o700)).unwrap();
    let socket = local_root.join("master.sock");
    let _listener = UnixListener::bind(&socket).unwrap();
    fs::set_permissions(&socket, fs::Permissions::from_mode(0o600)).unwrap();
    let remote_root = root.join("remote");
    let remote_socket = remote_root.join("peer.sock");
    let output = Command::new(&binary)
        .args([
            "federate",
            "ssh://fixture.invalid",
            "--label",
            "fixture",
            "--user",
            "fixture",
            "--identity",
            identity.to_str().unwrap(),
            "--known-hosts",
            known_hosts.to_str().unwrap(),
            "--root",
            local_root.to_str().unwrap(),
            "--socket",
            socket.to_str().unwrap(),
            "--remote-root",
            remote_root.to_str().unwrap(),
            "--remote-socket",
            remote_socket.to_str().unwrap(),
        ])
        .env_clear()
        .env("HOME", root.join("home"))
        .env("TMPDIR", root.join("tmp"))
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", root.join("tools").display()),
        )
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(37),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_dir(root.join("tmp")).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}
