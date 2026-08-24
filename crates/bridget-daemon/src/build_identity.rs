use std::path::{Path, PathBuf};
use std::process::Command;

pub fn current_build_id(root: &Path) -> Option<String> {
    let head = git(root, &["rev-parse", "--short=12", "HEAD"])?;
    let dirty = git(root, &["status", "--porcelain", "--untracked-files=no"])?;
    Some(if dirty.is_empty() {
        head
    } else {
        format!("{head}-dirty")
    })
}

pub fn invalidation_paths(root: &Path) -> Vec<PathBuf> {
    let Some(git_dir) =
        git(root, &["rev-parse", "--path-format=absolute", "--git-dir"]).map(PathBuf::from)
    else {
        return Vec::new();
    };
    let mut paths = vec![
        git_dir.join("HEAD"),
        git_path(root, "index"),
        git_path(root, "packed-refs"),
        root.join("Cargo.toml"),
        root.join("Cargo.lock"),
        root.join("crates"),
        root.join("plugins"),
    ];
    if let Some(reference) = git(root, &["symbolic-ref", "-q", "HEAD"]) {
        paths.push(git_path(root, &reference));
    }
    paths
}

fn git_path(root: &Path, path: &str) -> PathBuf {
    git(
        root,
        &["rev-parse", "--path-format=absolute", "--git-path", path],
    )
    .map(PathBuf::from)
    .unwrap_or_else(|| root.join(".git").join(path))
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn run(root: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    }

    #[test]
    fn avance_head_et_marque_index_et_sources_sales() {
        let root = std::env::temp_dir().join(format!("bridget-build-id-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        run(&root, &["init", "--initial-branch=main"]);
        run(&root, &["config", "user.email", "test@example.invalid"]);
        run(&root, &["config", "user.name", "test"]);
        fs::write(root.join("tracked.txt"), "one\n").unwrap();
        run(&root, &["add", "tracked.txt"]);
        run(&root, &["commit", "-m", "first"]);
        let first = current_build_id(&root).unwrap();
        assert!(!first.ends_with("-dirty"));
        let watched = invalidation_paths(&root);
        assert!(watched.iter().any(|path| path.ends_with("refs/heads/main")));
        assert!(watched.iter().any(|path| path.ends_with("index")));

        fs::write(root.join("tracked.txt"), "two\n").unwrap();
        assert_eq!(current_build_id(&root).unwrap(), format!("{first}-dirty"));
        run(&root, &["add", "tracked.txt"]);
        run(&root, &["commit", "-m", "second"]);
        let second = current_build_id(&root).unwrap();
        assert_ne!(first, second, "avancer HEAD doit changer l'identifiant");
        assert!(!second.ends_with("-dirty"));
        let _ = fs::remove_dir_all(root);
    }
}
