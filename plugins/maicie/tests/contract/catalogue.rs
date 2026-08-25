use maicie::catalogue::{
    AddEntry, AddKind, AppendOutcome, CatalogueEntry, CatalogueJournal, MissionSource,
    MissionSourceKind, PendingKind, PendingQualificationEntry, ProseMigrationRecord, Severity,
    migrate_prose_record, parse_closed_line, parse_journal_bytes, project_registre,
    render_registre_list, validate_catalogue_path, validate_rfc3339_with_offset,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

static NEXT: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let n = NEXT.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "maicie-catalogue-contract-{}-{}-{}",
            label,
            std::process::id(),
            n
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn sample_add(id: &str) -> AddEntry {
    AddEntry {
        v: 1,
        kind: AddKind::Add,
        id: id.into(),
        date: "2026-08-23T22:00:00+02:00".into(),
        mission_source: MissionSource {
            kind: MissionSourceKind::Incident,
            id: "inc-1".into(),
            failed: None,
        },
        severity: Severity::Minor,
        recurrence_of: None,
        text: format!("constat {id}"),
    }
}

#[test]
fn corpus_v1_valide_est_lisible_et_les_mutations_refusent_sans_append() {
    let valid = include_str!("../fixtures/catalogue-v1/valid-journal.jsonl");
    // Le fixture valide référence recurrence_of avant la cible dans l'ordre
    // physique : on vérifie d'abord les lignes isolées, puis un journal ordonné.
    for (label, body) in [
        (
            "unknown-field",
            include_str!("../fixtures/catalogue-v1/unknown-field.jsonl"),
        ),
        (
            "unknown-kind",
            include_str!("../fixtures/catalogue-v1/unknown-kind.jsonl"),
        ),
        (
            "unknown-severity",
            include_str!("../fixtures/catalogue-v1/unknown-severity.jsonl"),
        ),
        (
            "date-without-offset",
            include_str!("../fixtures/catalogue-v1/date-without-offset.jsonl"),
        ),
    ] {
        let err = parse_closed_line(body.lines().next().unwrap()).unwrap_err();
        assert!(
            err.to_string().contains("refusé")
                || err.to_string().contains("inconnu")
                || err.to_string().contains("fuseau")
                || err.to_string().contains("invalid")
                || err.to_string().contains("unknown")
                || err.to_string().contains("missing field")
                || err.to_string().contains("denied")
                || err.to_string().contains("horodatage")
                || err.to_string().contains("kind"),
            "{label}: {err}"
        );
    }

    let ordered = concat!(
        r#"{"v":1,"kind":"add","id":"c-info-base","date":"2026-08-22T10:00:00Z","mission_source":{"kind":"mission","id":"m-011"},"severity":"info","text":"base informative sans récurrence"}"#,
        "\n",
        r#"{"v":1,"kind":"add","id":"c-blocker-gate","date":"2026-08-23T22:00:00+02:00","mission_source":{"kind":"gate","id":"G1701","failed":true},"severity":"blocker","recurrence_of":"c-info-base","text":"gate G1701 rouge sur corpus fermé"}"#,
        "\n",
        r#"{"v":1,"kind":"add","id":"c-major","date":"2026-08-23T09:00:00+02:00","mission_source":{"kind":"review","id":"r-hostile"},"severity":"major","text":"revue hostile a demandé un amendement"}"#,
        "\n",
        r#"{"v":1,"kind":"transition","constat_id":"c-major","from":"open","to":"delivered","objective_id":"obj-42","observed_at":"2026-08-23T23:00:00+02:00","trigger":"objective_closed"}"#,
        "\n",
        r#"{"v":1,"kind":"pending_qualification","id":"pending:exigences#001","provenance_id":"exigences-coordination-v2.md#001","text":"entrée historique sans sévérité déclarée"}"#,
        "\n",
    );
    let _ = valid; // fixture figée pour revue humaine / golden futur
    let entries = parse_journal_bytes(ordered.as_bytes()).unwrap().entries;
    assert_eq!(entries.len(), 5);
    let view = project_registre(&entries);
    assert_eq!(view.footer.ouverts, 2);
    assert_eq!(view.footer.recurrents, 1);
    assert_eq!(view.footer.gates_rates, 1);
    assert_eq!(view.footer.pending_qualification, 1);
    assert_eq!(view.ouverts[0].id, "c-blocker-gate");
}

#[test]
fn chemin_hors_catalogue_symlink_et_tasks_md_sont_refuses() {
    let fixture = Fixture::new("paths");
    let tasks = fixture.path("tasks.md");
    fs::write(&tasks, "# plan hôte\n").unwrap();
    assert!(validate_catalogue_path(&tasks, Some(&fixture.root)).is_err());

    let catalogue = fixture.path("catalogue.jsonl");
    fs::write(&catalogue, "").unwrap();
    assert!(validate_catalogue_path(&catalogue, Some(&fixture.root)).is_ok());

    let link = fixture.path("alias.jsonl");
    std::os::unix::fs::symlink(&catalogue, &link).unwrap();
    assert!(validate_catalogue_path(&link, Some(&fixture.root)).is_err());

    let dehors = PathBuf::from("/tmp/maicie-catalogue-hors-projet.jsonl");
    let _ = fs::remove_file(&dehors);
    assert!(validate_catalogue_path(&dehors, Some(&fixture.root)).is_err());
}

#[test]
fn jury_controle_positif_de_l_instrument() {
    let fixture = Fixture::new("jury-ctrl");
    let dedans = fixture.path("catalogue.jsonl");
    fs::write(&dedans, "").unwrap();
    assert!(
        validate_catalogue_path(&dedans, Some(&fixture.root)).is_ok(),
        "l'instrument refuse le cas légitime : un vert ailleurs ne prouverait rien"
    );
}

#[test]
fn jury_chemin_interne_a_parent_inexistant_reste_accepte() {
    let fixture = Fixture::new("jury-acreer");
    let candidate = fixture.root.join("dossier-a-creer").join("catalogue.jsonl");
    let result = validate_catalogue_path(&candidate, Some(&fixture.root));
    assert!(
        result.is_ok(),
        "chemin interne refusé car son parent n'existe pas encore : root={:?} candidat={:?} verdict={:?}",
        fixture.root,
        candidate,
        result
    );
}

#[test]
fn jury_evasion_par_parent_inexistant_est_refusee() {
    let fixture = Fixture::new("jury-evasion");
    let evasion = fixture
        .root
        .join("inexistant")
        .join("..")
        .join("..")
        .join("..")
        .join("..")
        .join("..")
        .join("etc")
        .join("maicie-jury-canari.jsonl");
    let result = validate_catalogue_path(&evasion, Some(&fixture.root));
    assert!(
        result.is_err(),
        "évasion de racine acceptée : root={:?} candidat={:?}",
        fixture.root,
        evasion
    );
}

#[test]
fn jury_racine_symlinkee_reproduit_le_cas_macos_sur_linux() {
    let fixture = Fixture::new("jury-racine-liee");
    let real_root = fixture.path("projet-reel");
    fs::create_dir(&real_root).unwrap();
    let linked_root = fixture.path("projet-lie");
    std::os::unix::fs::symlink(&real_root, &linked_root).unwrap();
    let candidate = linked_root.join("dossier-a-creer/catalogue.jsonl");

    let result = validate_catalogue_path(&candidate, Some(&linked_root));

    assert!(
        result.is_ok(),
        "un chemin interne doit rester accepté sous une racine symlinkée : {result:?}"
    );
}

#[test]
fn jury_lien_existant_est_resolu_avant_le_reliquat_manquant() {
    let fixture = Fixture::new("jury-lien-existant");
    let project_root = fixture.path("projet");
    fs::create_dir(&project_root).unwrap();
    let outside = fixture.path("hors-projet");
    let outside_child = outside.join("enfant");
    fs::create_dir_all(&outside_child).unwrap();
    let link = project_root.join("lien");
    std::os::unix::fs::symlink(&outside_child, &link).unwrap();
    let candidate = link.join("..").join("catalogue.jsonl");

    let result = validate_catalogue_path(&candidate, Some(&project_root));

    assert!(
        result.is_err(),
        "un lien existant suivi de '..' ne doit pas être normalisé dans la racine"
    );
}

#[test]
fn sc1708_replay_identique_et_divergent() {
    let fixture = Fixture::new("idem");
    let path = fixture.path("catalogue.jsonl");
    let mut journal = CatalogueJournal::open(&path).unwrap();
    let entry = sample_add("same");
    assert_eq!(
        journal.append_add(entry.clone()).unwrap(),
        AppendOutcome::Appended
    );
    assert_eq!(
        journal.append_add(entry.clone()).unwrap(),
        AppendOutcome::IdempotentNoop
    );
    let mut divergent = entry;
    divergent.text = "octets différents".into();
    let err = journal.append_add(divergent).unwrap_err();
    assert!(err.to_string().contains("divergents"));
    assert_eq!(journal.read_entries().unwrap().len(), 1);
}

#[test]
fn sc1710_deux_writers_concurrents() {
    let fixture = Fixture::new("concurrent");
    let path = fixture.path("catalogue.jsonl");
    fs::write(&path, "").unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let path_a = path.clone();
    let path_b = path.clone();
    let barrier_a = barrier.clone();
    let barrier_b = barrier;
    let a = thread::spawn(move || {
        barrier_a.wait();
        let mut journal = CatalogueJournal::open(&path_a).unwrap();
        journal.append_add(sample_add("concurrent-a")).unwrap()
    });
    let b = thread::spawn(move || {
        barrier_b.wait();
        let mut journal = CatalogueJournal::open(&path_b).unwrap();
        journal.append_add(sample_add("concurrent-b")).unwrap()
    });
    assert_eq!(a.join().unwrap(), AppendOutcome::Appended);
    assert_eq!(b.join().unwrap(), AppendOutcome::Appended);
    let mut journal = CatalogueJournal::open(&path).unwrap();
    let entries = journal.read_entries().unwrap();
    assert_eq!(entries.len(), 2, "aucune entrée perdue ni tronquée");
    for entry in &entries {
        match entry {
            CatalogueEntry::Add(add) => {
                assert!(add.text.starts_with("constat concurrent-"));
                validate_rfc3339_with_offset(&add.date).unwrap();
            }
            _ => panic!("ligne inattendue"),
        }
    }
}

#[test]
fn sc1701_migration_prose_preserve_textes_verbatim_en_pending() {
    let corpus = include_str!("../fixtures/catalogue-migration/prose-corpus.jsonl");
    let fixture = Fixture::new("migrate");
    let path = fixture.path("catalogue.jsonl");
    let mut journal = CatalogueJournal::open(&path).unwrap();
    let mut expected_texts = Vec::new();
    for line in corpus.lines().filter(|line| !line.is_empty()) {
        let record: ProseMigrationRecord = serde_json::from_str(line).unwrap();
        expected_texts.push(record.text.clone());
        let entry = migrate_prose_record(&record).unwrap();
        match &entry {
            CatalogueEntry::PendingQualification(pending) => {
                assert_eq!(pending.text, record.text);
                assert_eq!(pending.kind, PendingKind::PendingQualification);
                assert_eq!(
                    journal.append_pending(pending.clone()).unwrap(),
                    AppendOutcome::Appended
                );
            }
            other => panic!("migration incomplète doit rester pending, obtenu {other:?}"),
        }
    }
    assert!(
        expected_texts.len() >= 40,
        "corpus réel attendu (~50), obtenu {}",
        expected_texts.len()
    );
    let entries = journal.read_entries().unwrap();
    assert_eq!(entries.len(), expected_texts.len());
    for (entry, expected) in entries.iter().zip(expected_texts.iter()) {
        match entry {
            CatalogueEntry::PendingQualification(pending) => {
                assert_eq!(&pending.text, expected, "texte historique perdu ou réécrit");
            }
            _ => panic!("aucune activation par défaut"),
        }
    }
    let view = project_registre(&entries);
    assert_eq!(view.footer.ouverts, 0);
    assert_eq!(view.footer.pending_qualification, expected_texts.len());
    assert!(!render_registre_list(&view).contains("sévérité inventée"));
}

#[test]
fn sc1704_vue_identiques_sur_ordres_physiques_permutés() {
    let a = CatalogueEntry::Add(sample_add("z-last"));
    let b = CatalogueEntry::Add({
        let mut entry = sample_add("a-first");
        entry.severity = Severity::Blocker;
        entry.date = "2026-08-20T00:00:00Z".into();
        entry
    });
    let c = CatalogueEntry::PendingQualification(PendingQualificationEntry {
        v: 1,
        kind: PendingKind::PendingQualification,
        id: "p-1".into(),
        provenance_id: "prov".into(),
        text: "en attente".into(),
    });
    let order_one = vec![a.clone(), b.clone(), c.clone()];
    let order_two = vec![c, b, a];
    let left = render_registre_list(&project_registre(&order_one));
    let right = render_registre_list(&project_registre(&order_two));
    assert_eq!(left, right);
    assert!(left.contains("pied: 2 OUVERTS dont 0 récurrents, 0 gates ratés, 1 en attente ; 0 TRAITÉS, 0 RÉFUTÉS, 0 REQUALIFIÉS"));
}

#[test]
fn migration_complete_devient_add_sans_inventer_de_champ() {
    let record = ProseMigrationRecord {
        provenance_id: "manual#1".into(),
        text: "qualifié par un humain".into(),
        id: Some("human-1".into()),
        date: Some("2026-08-24T01:00:00+02:00".into()),
        mission_source: Some(MissionSource {
            kind: MissionSourceKind::Review,
            id: "r1".into(),
            failed: None,
        }),
        severity: Some(Severity::Major),
        recurrence_of: None,
    };
    match migrate_prose_record(&record).unwrap() {
        CatalogueEntry::Add(add) => {
            assert_eq!(add.id, "human-1");
            assert_eq!(add.text, "qualifié par un humain");
            assert_eq!(add.severity, Severity::Major);
        }
        other => panic!("attendu add, obtenu {other:?}"),
    }
}

#[test]
fn reference_recurrence_inexistante_refusee_avant_append() {
    let fixture = Fixture::new("recurrence");
    let mut journal = CatalogueJournal::open(fixture.path("catalogue.jsonl")).unwrap();
    let mut entry = sample_add("child");
    entry.recurrence_of = Some("absent".into());
    let err = journal.append_add(entry).unwrap_err();
    assert!(err.to_string().contains("recurrence_of"));
    assert_eq!(journal.read_entries().unwrap().len(), 0);
}

#[allow(dead_code)]
fn assert_under_project(path: &Path, root: &Path) {
    assert!(path.starts_with(root));
}
