use maicie::catalogue::Severity;
use maicie::review::{RegistryFinding, ReviewRegime, calculate_criticality};
use maicie::review_git::{
    GitMeasurementRequest, LimitKind, ReviewGitError, ReviewGitLimits, measure_repository,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

struct FixtureRepository {
    root: PathBuf,
}

impl FixtureRepository {
    fn new(label: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("maicie-review-git-{label}-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let fixture = Self { root };
        fixture.git(&["init", "-q", "-b", "fixture"]);
        fixture
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn git(&self, arguments: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(arguments)
            .env("LC_ALL", "C")
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_COMMITTER_NAME", "Fixture")
            .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    fn commit(&self, message: &str) -> String {
        self.git(&["add", "--all"]);
        self.git(&[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--no-verify",
            "-q",
            "-m",
            message,
        ]);
        self.git(&["rev-parse", "HEAD"])
    }
}

impl Drop for FixtureRepository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn request<'a>(
    root: &'a Path,
    branch_ref: &'a str,
    base: &'a str,
    head: &'a str,
    limits: ReviewGitLimits,
) -> GitMeasurementRequest<'a> {
    GitMeasurementRequest {
        repository_root: root,
        branch_ref,
        base,
        head,
        open_findings: &[],
        limits,
    }
}

#[test]
fn t2509_mesure_les_objets_exacts_sans_lire_le_worktree() {
    let repo = FixtureRepository::new("objets-exacts");
    repo.write("src/render.rs", "pub fn value() -> u8 { 1 }\n");
    let base = repo.commit("base");
    repo.write("src/render.rs", "pub fn value() -> u8 { 2 }\n");
    let head = repo.commit("head");

    let first = measure_repository(&request(
        &repo.root,
        "refs/heads/fixture",
        &base,
        &head,
        ReviewGitLimits::default(),
    ))
    .unwrap();
    repo.write(
        "src/render.rs",
        "const SCHEMA_VERSION: i64 = 99;\npub fn value() -> u8 { 3 }\n",
    );
    repo.write("src/permissions.rs", "pub enum Permission { Root }\n");
    let second = measure_repository(&request(
        &repo.root,
        "refs/heads/fixture",
        &base,
        &head,
        ReviewGitLimits::default(),
    ))
    .unwrap();

    assert_eq!(first, second, "le worktree ne participe pas à la mesure");
    assert_eq!(first.changes.len(), 1);
    assert_eq!(first.changes[0].new_path.as_deref(), Some("src/render.rs"));
    assert!(!first.changes[0].patch.contains("SCHEMA_VERSION"));
    assert_eq!(
        calculate_criticality(&first).unwrap().proposed_regime,
        ReviewRegime::Simple
    );
}

#[test]
fn t2509_ref_deplacee_et_base_non_ancetre_sont_refusees() {
    let repo = FixtureRepository::new("coherence");
    repo.write("src/value.rs", "pub const VALUE: u8 = 1;\n");
    let root = repo.commit("root");
    repo.write("src/value.rs", "pub const VALUE: u8 = 2;\n");
    let first_head = repo.commit("first-head");
    repo.write("src/value.rs", "pub const VALUE: u8 = 3;\n");
    let moved_head = repo.commit("moved-head");
    assert_ne!(first_head, moved_head);

    let moved = measure_repository(&request(
        &repo.root,
        "refs/heads/fixture",
        &root,
        &first_head,
        ReviewGitLimits::default(),
    ));
    assert!(matches!(moved, Err(ReviewGitError::BranchHeadMismatch)));

    repo.git(&["switch", "-q", "-c", "side", &root]);
    repo.write("src/side.rs", "pub const SIDE: bool = true;\n");
    let side_head = repo.commit("side");
    let divergent = measure_repository(&request(
        &repo.root,
        "refs/heads/side",
        &moved_head,
        &side_head,
        ReviewGitLimits::default(),
    ));
    assert!(matches!(divergent, Err(ReviewGitError::BaseNotAncestor)));
}

#[test]
fn t2509_un_renommage_conserve_ancien_et_nouveau_chemin() {
    let repo = FixtureRepository::new("rename");
    repo.write("src/protocol.rs", "pub struct EnvelopeV1;\n");
    let base = repo.commit("base");
    repo.git(&["mv", "src/protocol.rs", "src/wire.rs"]);
    let head = repo.commit("rename");

    let measured = measure_repository(&request(
        &repo.root,
        "refs/heads/fixture",
        &base,
        &head,
        ReviewGitLimits::default(),
    ))
    .unwrap();
    assert_eq!(measured.changes.len(), 1);
    assert_eq!(
        measured.changes[0].old_path.as_deref(),
        Some("src/protocol.rs")
    );
    assert_eq!(measured.changes[0].new_path.as_deref(), Some("src/wire.rs"));
    let map = calculate_criticality(&measured).unwrap();
    assert_eq!(map.proposed_regime, ReviewRegime::JuryOnePlusOne);
    assert_eq!(map.critical_changed_paths, ["src/wire.rs"]);
}

#[test]
fn t2509_charge_les_contrats_du_commit_et_borne_l_univers() {
    let repo = FixtureRepository::new("contracts-limits");
    repo.write("src/wire.rs", "pub const WIRE: u8 = 1;\n");
    repo.write(
        "specs/001-wire/contracts/wire.md",
        "Le format est ancré par `src/wire.rs`.\n",
    );
    let base = repo.commit("base");
    repo.write("src/wire.rs", "pub const WIRE: u8 = 2;\n");
    let head = repo.commit("head");

    let measured = measure_repository(&request(
        &repo.root,
        "refs/heads/fixture",
        &base,
        &head,
        ReviewGitLimits::default(),
    ))
    .unwrap();
    assert_eq!(measured.contracts.len(), 1);
    assert_eq!(
        measured.contracts[0].source_path,
        "specs/001-wire/contracts/wire.md"
    );
    assert!(
        calculate_criticality(&measured)
            .unwrap()
            .zones
            .iter()
            .any(|zone| zone.path == "src/wire.rs")
    );

    let limited = measure_repository(&request(
        &repo.root,
        "refs/heads/fixture",
        &base,
        &head,
        ReviewGitLimits {
            max_paths: 1,
            ..ReviewGitLimits::default()
        },
    ));
    assert!(matches!(
        limited,
        Err(ReviewGitError::SnapshotLimitExceeded {
            kind: LimitKind::PathCount,
            ..
        })
    ));

    let findings = [RegistryFinding {
        id: "blocker-1".to_string(),
        severity: Severity::Blocker,
        text: "src/wire.rs".to_string(),
    }];
    let registry_limited = measure_repository(&GitMeasurementRequest {
        repository_root: &repo.root,
        branch_ref: "refs/heads/fixture",
        base: &base,
        head: &head,
        open_findings: &findings,
        limits: ReviewGitLimits {
            max_findings: 0,
            ..ReviewGitLimits::default()
        },
    });
    assert!(matches!(
        registry_limited,
        Err(ReviewGitError::SnapshotLimitExceeded {
            kind: LimitKind::FindingCount,
            ..
        })
    ));
}

#[test]
fn t2509_ref_courte_sha_non_canonique_et_racine_relative_refusent_avant_mesure() {
    let repo = FixtureRepository::new("frontiere");
    repo.write("src/value.rs", "pub const VALUE: u8 = 1;\n");
    let base = repo.commit("base");
    repo.write("src/value.rs", "pub const VALUE: u8 = 2;\n");
    let head = repo.commit("head");

    assert!(matches!(
        measure_repository(&request(
            &repo.root,
            "fixture",
            &base,
            &head,
            ReviewGitLimits::default(),
        )),
        Err(ReviewGitError::BranchRefNotFull)
    ));
    assert!(matches!(
        measure_repository(&request(
            &repo.root,
            "refs/heads/fixture",
            &base.to_ascii_uppercase(),
            &head,
            ReviewGitLimits::default(),
        )),
        Err(ReviewGitError::InvalidCommit { field: "base" })
    ));
    assert!(matches!(
        measure_repository(&request(
            Path::new("relative/repository"),
            "refs/heads/fixture",
            &base,
            &head,
            ReviewGitLimits::default(),
        )),
        Err(ReviewGitError::RepositoryUnavailable)
    ));
}

#[test]
fn t2511_les_variables_git_heritees_ne_changent_pas_la_paire_mesuree() {
    const ROOT_ENV: &str = "MAICIE_025_FIXTURE_ROOT";
    const BASE_ENV: &str = "MAICIE_025_FIXTURE_BASE";
    const HEAD_ENV: &str = "MAICIE_025_FIXTURE_HEAD";
    if let (Ok(root), Ok(base), Ok(head)) = (
        std::env::var(ROOT_ENV),
        std::env::var(BASE_ENV),
        std::env::var(HEAD_ENV),
    ) {
        let measured = measure_repository(&request(
            Path::new(&root),
            "refs/heads/fixture",
            &base,
            &head,
            ReviewGitLimits::default(),
        ))
        .unwrap();
        assert_eq!(measured.changes.len(), 1);
        assert_eq!(
            calculate_criticality(&measured).unwrap().proposed_regime,
            ReviewRegime::Simple
        );
        return;
    }

    let repo = FixtureRepository::new("environment");
    repo.write("src/render.rs", "pub fn value() -> u8 { 1 }\n");
    let base = repo.commit("base");
    repo.write("src/render.rs", "pub fn value() -> u8 { 2 }\n");
    let head = repo.commit("head");
    let executable = std::env::current_exe().unwrap();
    let output = Command::new(executable)
        .args([
            "--exact",
            "t2511_les_variables_git_heritees_ne_changent_pas_la_paire_mesuree",
            "--nocapture",
        ])
        .env(ROOT_ENV, &repo.root)
        .env(BASE_ENV, &base)
        .env(HEAD_ENV, &head)
        .env("GIT_DIR", "/defaut/interdit")
        .env("GIT_WORK_TREE", "/defaut/interdit")
        .env("GIT_EXTERNAL_DIFF", "/defaut/interdit")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "diff.external")
        .env("GIT_CONFIG_VALUE_0", "/defaut/interdit")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "le sous-processus hostile échoue : {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "l'univers enfant doit contenir un test réel"
    );
}
