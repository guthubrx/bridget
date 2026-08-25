use maicie::catalogue::Severity;
use maicie::review::{
    CitationSource, ContractDocument, EvidenceKind, F38_FIXED_REGIME, FileChange, RegistryFinding,
    RepositorySnapshot, ReviewRegime, SeedRule, TrackedPath, calculate_criticality,
};

fn path(path: &str, line_count: usize) -> TrackedPath {
    TrackedPath {
        path: path.to_string(),
        line_count,
    }
}

fn change(path: &str, patch: &str, head_content: &str) -> FileChange {
    FileChange {
        old_path: Some(path.to_string()),
        new_path: Some(path.to_string()),
        patch: patch.to_string(),
        head_content: Some(head_content.to_string()),
    }
}

fn finding(id: &str, severity: Severity, text: &str) -> RegistryFinding {
    RegistryFinding {
        id: id.to_string(),
        severity,
        text: text.to_string(),
    }
}

fn snapshot(paths: Vec<TrackedPath>, changes: Vec<FileChange>) -> RepositorySnapshot {
    RepositorySnapshot {
        paths,
        changes,
        contracts: Vec::new(),
        open_findings: Vec::new(),
    }
}

fn elected_paths(snapshot: &RepositorySnapshot) -> Vec<String> {
    calculate_criticality(snapshot)
        .unwrap()
        .zones
        .into_iter()
        .map(|zone| zone.path)
        .collect()
}

#[test]
fn t2505_registre_resout_exact_unique_et_ligne_sans_elire_les_homonymes() {
    let paths = vec![
        path("crates/bridget-daemon/src/daemon.rs", 900),
        path("crates/bridget-daemon/src/store.rs", 80),
        path("plugins/maicie/src/store.rs", 7_600),
    ];

    let mut exact = snapshot(paths.clone(), Vec::new());
    exact.open_findings = vec![finding(
        "blocker-exact",
        Severity::Blocker,
        "défaut dans plugins/maicie/src/store.rs:412",
    )];
    assert_eq!(elected_paths(&exact), ["plugins/maicie/src/store.rs"]);

    let mut unique = snapshot(paths.clone(), Vec::new());
    unique.open_findings = vec![finding(
        "blocker-unique",
        Severity::Blocker,
        "la frontière daemon.rs:230 est concernée",
    )];
    assert_eq!(
        elected_paths(&unique),
        ["crates/bridget-daemon/src/daemon.rs"]
    );

    let mut ambiguous = snapshot(paths.clone(), Vec::new());
    ambiguous.open_findings = vec![finding(
        "blocker-ambigu",
        Severity::Blocker,
        "store.rs doit être repris",
    )];
    let map = calculate_criticality(&ambiguous).unwrap();
    assert!(map.zones.is_empty(), "un nom ambigu n'élit aucune zone");
    assert_eq!(map.unresolved_citations.len(), 1);
    assert_eq!(
        map.unresolved_citations[0].candidates,
        [
            "crates/bridget-daemon/src/store.rs",
            "plugins/maicie/src/store.rs"
        ]
    );

    let mut line_resolved = snapshot(paths, Vec::new());
    line_resolved.open_findings = vec![finding(
        "blocker-ligne",
        Severity::Blocker,
        "store.rs:6135 échoue",
    )];
    assert_eq!(
        elected_paths(&line_resolved),
        ["plugins/maicie/src/store.rs"]
    );
}

#[test]
fn t2505_contrat_n_ancre_qu_un_chemin_complet_exact() {
    let mut input = snapshot(
        vec![
            path("crates/bridget-transport/src/protocol.rs", 420),
            path("crates/bridget-daemon/src/store.rs", 80),
            path("plugins/maicie/src/store.rs", 7_600),
            path(
                "specs/015-guichet-maicie/contracts/protocole-guichet.md",
                200,
            ),
        ],
        Vec::new(),
    );
    input.contracts = vec![ContractDocument {
        source_path: "specs/011-maicie-orchestration/contracts/protocol.md".to_string(),
        content: "Ancre `crates/bridget-transport/src/protocol.rs`; référence croisée `specs/015-guichet-maicie/contracts/protocole-guichet.md`; store.rs est incomplet."
            .to_string(),
    }];

    let map = calculate_criticality(&input).unwrap();
    assert_eq!(
        map.zones
            .iter()
            .map(|zone| zone.path.as_str())
            .collect::<Vec<_>>(),
        ["crates/bridget-transport/src/protocol.rs"]
    );
    assert!(
        map.zones[0]
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::Contract)
    );
    assert_eq!(map.unresolved_citations.len(), 1);
    assert_eq!(
        map.unresolved_citations[0].source_kind,
        CitationSource::Contract
    );
    assert_eq!(map.unresolved_citations[0].candidates.len(), 2);
}

#[test]
fn t2506_un_blocker_ou_deux_majors_distincts_elisent_mais_pas_une_repetition() {
    let paths = vec![path("src/domain.rs", 500)];

    let mut repeated = snapshot(paths.clone(), Vec::new());
    repeated.open_findings = vec![finding(
        "major-1",
        Severity::Major,
        "src/domain.rs:20 puis src/domain.rs:30",
    )];
    assert!(elected_paths(&repeated).is_empty());

    let mut two_majors = repeated.clone();
    two_majors
        .open_findings
        .push(finding("major-2", Severity::Major, "src/domain.rs:40"));
    assert_eq!(elected_paths(&two_majors), ["src/domain.rs"]);

    let mut blocker = snapshot(paths, Vec::new());
    blocker.open_findings = vec![finding("blocker-1", Severity::Blocker, "src/domain.rs")];
    assert_eq!(elected_paths(&blocker), ["src/domain.rs"]);
}

#[test]
fn t2506_les_trois_voies_elisent_des_chemins_complets_et_durcissent_le_diff() {
    let mut input = snapshot(
        vec![
            path("db/migrations/v20.sql", 30),
            path("src/domain.rs", 500),
            path("src/wire.rs", 240),
        ],
        vec![
            change(
                "db/migrations/v20.sql",
                "+ALTER TABLE review ADD COLUMN state TEXT;",
                "ALTER TABLE review ADD COLUMN state TEXT;",
            ),
            change(
                "src/domain.rs",
                "+pub fn changed() {}",
                "pub fn changed() {}",
            ),
            change(
                "src/wire.rs",
                "+pub const V2: u8 = 2;",
                "pub const V2: u8 = 2;",
            ),
        ],
    );
    input.open_findings = vec![finding(
        "blocker-domain",
        Severity::Blocker,
        "src/domain.rs:42",
    )];
    input.contracts = vec![ContractDocument {
        source_path: "specs/001-wire/contracts/wire.md".to_string(),
        content: "Le format est défini par `src/wire.rs`.".to_string(),
    }];

    let map = calculate_criticality(&input).unwrap();
    assert_eq!(
        map.zones
            .iter()
            .map(|zone| zone.path.as_str())
            .collect::<Vec<_>>(),
        ["db/migrations/v20.sql", "src/domain.rs", "src/wire.rs"]
    );
    assert!(map.zones.iter().any(|zone| {
        zone.evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::DiffContent)
    }));
    assert!(map.zones.iter().any(|zone| {
        zone.evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::Registry)
    }));
    assert!(map.zones.iter().any(|zone| {
        zone.evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::Contract)
    }));
    assert_eq!(map.proposed_regime, ReviewRegime::JuryOnePlusOne);
    assert_eq!(map.critical_changed_paths.len(), 3);
}

#[test]
fn t2506_la_carte_nait_avec_les_quatre_germes_generiques() {
    let map = calculate_criticality(&snapshot(Vec::new(), Vec::new())).unwrap();
    assert_eq!(
        map.seed_rules,
        [
            SeedRule::PersistenceSchema,
            SeedRule::ProtocolFormat,
            SeedRule::AuthorizationPermissions,
            SeedRule::ExternalInputDurableWrite,
        ]
    );
    assert_eq!(map.seed_rules.len(), 4, "l'univers des germes est non vide");
    assert_eq!(map.proposed_regime, ReviewRegime::Simple);
}

#[test]
fn t2506_chaque_germe_seul_propose_un_jury() {
    let cases = [
        (
            "src/model.rs",
            "+const SCHEMA_VERSION: i64 = 20;",
            "const SCHEMA_VERSION: i64 = 20;",
            SeedRule::PersistenceSchema,
        ),
        (
            "src/protocol.rs",
            "+pub struct EnvelopeV2;",
            "pub struct EnvelopeV2;",
            SeedRule::ProtocolFormat,
        ),
        (
            "src/permissions.rs",
            "+pub enum Capability { Read }",
            "pub enum Capability { Read }",
            SeedRule::AuthorizationPermissions,
        ),
        (
            "src/archive.rs",
            "+let frame = socket.recv()?; db.execute(\"INSERT INTO traces\", [frame])?;",
            "let frame = socket.recv()?; db.execute(\"INSERT INTO traces\", [frame])?;",
            SeedRule::ExternalInputDurableWrite,
        ),
    ];

    for (changed_path, patch, content, expected_rule) in cases {
        let input = snapshot(
            vec![path(changed_path, 20)],
            vec![change(changed_path, patch, content)],
        );
        let map = calculate_criticality(&input).unwrap();
        assert_eq!(
            map.proposed_regime,
            ReviewRegime::JuryOnePlusOne,
            "le mutant du germe {expected_rule:?} doit rougir ici"
        );
        let zone = map
            .zones
            .iter()
            .find(|zone| zone.path == changed_path)
            .expect("le fichier du germe est une zone critique");
        assert!(zone.evidence.iter().any(|evidence| {
            evidence.kind == EvidenceKind::Seed && evidence.rule == Some(expected_rule)
        }));
    }
}

#[test]
fn t2506_un_diff_ordinaire_reste_simple_et_le_noyau_reste_fixe_en_2x2() {
    let ordinary = snapshot(
        vec![path("src/render.rs", 20)],
        vec![change(
            "src/render.rs",
            "+pub fn render() {}",
            "pub fn render() {}",
        )],
    );
    assert_eq!(
        calculate_criticality(&ordinary).unwrap().proposed_regime,
        ReviewRegime::Simple
    );

    for fixed_path in [
        "plugins/maicie/src/review.rs",
        "plugins/maicie/src/review_git.rs",
        "specs/025-carte-criticite-regime/contracts/revue-lot-v1.md",
    ] {
        let self_change = snapshot(
            vec![path(fixed_path, 500)],
            vec![change(
                fixed_path,
                "+const RULE: u8 = 4;",
                "const RULE: u8 = 4;",
            )],
        );
        assert_eq!(
            calculate_criticality(&self_change).unwrap().proposed_regime,
            F38_FIXED_REGIME,
            "le périmètre fixé par le référent doit couvrir {fixed_path}"
        );
    }
    assert_eq!(F38_FIXED_REGIME, ReviewRegime::JuryTwoByTwo);
}
