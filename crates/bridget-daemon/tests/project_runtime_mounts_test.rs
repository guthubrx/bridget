use bridget_daemon::project_runtime::{CONTAINER_STATE_ROOT, resolve_project_mounts};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .expect("git disponible");
    assert!(status.success(), "git {:?} doit réussir", args);
}

#[test]
fn spec_066_worktrees_du_meme_common_dir_sont_montes_sans_elargir_le_projet() {
    let root = std::env::temp_dir().join(format!("bridget-066-mounts-{}", uuid::Uuid::new_v4()));
    let project = root.join("project");
    let linked = root.join("linked");
    let state = root.join("state");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    git(&project, &["init"]);
    git(
        &project,
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(&project, &["config", "user.name", "Fixture"]);
    std::fs::write(project.join("README"), "fixture").unwrap();
    git(&project, &["add", "README"]);
    git(&project, &["commit", "-m", "fixture"]);
    git(
        &project,
        &[
            "worktree",
            "add",
            "-b",
            "bridget-066-linked",
            linked.to_str().unwrap(),
        ],
    );

    let mounts = resolve_project_mounts(&project, &state).expect("layout commun admis");
    let actual = mounts
        .iter()
        .filter(|mount| mount.container_path != CONTAINER_STATE_ROOT)
        .map(|mount| mount.host_path.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual,
        BTreeSet::from([
            std::fs::canonicalize(&project).unwrap(),
            std::fs::canonicalize(&linked).unwrap(),
        ])
    );
    assert!(mounts.iter().any(|mount| {
        mount.host_path == state && mount.container_path == CONTAINER_STATE_ROOT && mount.writable
    }));

    git(
        &project,
        &["worktree", "remove", "--force", linked.to_str().unwrap()],
    );
    let _ = std::fs::remove_dir_all(root);
}
