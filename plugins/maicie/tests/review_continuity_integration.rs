use maicie::review_continuity::{ReviewHeadRelation, observe_reviewed_head};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

struct GitFixture {
    root: PathBuf,
    remote: PathBuf,
    author: PathBuf,
    observer: PathBuf,
}

impl GitFixture {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("maicie-review-continuity-{}", Uuid::new_v4()));
        let remote = root.join("remote.git");
        let author = root.join("author");
        let observer = root.join("observer");
        fs::create_dir_all(&root).unwrap();
        run_git(&root, &["init", "--bare", "-q", remote.to_str().unwrap()]);
        fs::create_dir_all(&author).unwrap();
        run_git(&author, &["init", "-q", "-b", "session-fixture"]);
        run_git(
            &author,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        fs::create_dir_all(&observer).unwrap();
        run_git(&observer, &["init", "-q"]);
        run_git(
            &observer,
            &["remote", "add", "origin", remote.to_str().unwrap()],
        );
        Self {
            root,
            remote,
            author,
            observer,
        }
    }

    fn commit(&self, content: &str, message: &str) -> String {
        fs::write(self.author.join("value.txt"), content).unwrap();
        run_git(&self.author, &["add", "value.txt"]);
        run_git(
            &self.author,
            &[
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--no-verify",
                "-q",
                "-m",
                message,
            ],
        );
        run_git(&self.author, &["rev-parse", "HEAD"])
    }

    fn push(&self, force: bool) {
        let mut args = vec!["push", "-q"];
        if force {
            args.push("--force");
        }
        args.extend(["origin", "HEAD:refs/heads/session-fixture"]);
        run_git(&self.author, &args);
    }

    fn seed_observer(&self) {
        run_git(
            &self.observer,
            &[
                "fetch",
                "-q",
                "origin",
                "refs/heads/session-fixture:refs/remotes/origin/session-fixture",
            ],
        );
    }
}

impl Drop for GitFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("LC_ALL", "C")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn spec_047_distingue_empilement_et_reecriture_de_la_tete_distante() {
    let fixture = GitFixture::new();
    let base = fixture.commit("base\n", "base");
    let judged = fixture.commit("judged\n", "judged");
    fixture.push(false);
    fixture.seed_observer();

    let unchanged =
        observe_reviewed_head(&fixture.observer, "origin/session-fixture", &judged).unwrap();
    assert_eq!(
        unchanged,
        ReviewHeadRelation::StillAncestor {
            observed_head: judged.clone(),
        }
    );

    let appended = fixture.commit("appended\n", "appended");
    fixture.push(false);
    let stacked =
        observe_reviewed_head(&fixture.observer, "origin/session-fixture", &judged).unwrap();
    assert_eq!(
        stacked,
        ReviewHeadRelation::StillAncestor {
            observed_head: appended,
        },
        "une simple égalité de SHA classerait à tort cet empilement comme réécriture"
    );

    run_git(&fixture.author, &["reset", "--hard", &base]);
    let sibling = fixture.commit("sibling\n", "sibling");
    fixture.push(true);
    let rewritten =
        observe_reviewed_head(&fixture.observer, "origin/session-fixture", &judged).unwrap();
    assert_eq!(
        rewritten,
        ReviewHeadRelation::Rewritten {
            observed_head: sibling,
        }
    );

    assert!(fixture.remote.is_dir());
}
