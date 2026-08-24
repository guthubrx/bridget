//! Gate de session 017 — greffière du catalogue.
//!
//! Ce n'est PAS une suite de tests unitaires : un seul scénario d'intégration
//! enchaîne les cinq promesses de la session. Chaque assert nomme la promesse
//! et la mutation qui la ferait échouer — si l'assert disparaît ou devient
//! trivial, le gate ne prouve plus rien.
//!
//! Exécution :
//! `cargo test -p maicie --test catalogue_session_gate -- --nocapture`

use maicie::catalogue::{
    AppendOutcome, ArbitrationLink, AttestedClosure, CatalogueEntry, CatalogueJournal,
    CoveredFactKind, MissionSource, MissionSourceKind, ObservedFact, Severity,
    TranscriptionOutcome, parse_journal_bytes, project_registre,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

struct GateRoot {
    root: PathBuf,
}

impl GateRoot {
    fn new() -> Self {
        let n = NEXT.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "maicie-017-session-gate-{}-{}",
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

impl Drop for GateRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Gate unique : les cinq promesses, dans l'ordre, sur un journal réel.
#[test]
fn gate_session_017_cinq_promesses() {
    let root = GateRoot::new();
    let journal_path = root.path("catalogue.jsonl");
    let prose_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/catalogue-migration/prose-corpus.jsonl");

    // ── (1) SURVIE ──────────────────────────────────────────────────────────
    // Mutation qui ferait ÉCHOUER : refuser le journal entier dès qu'une
    // dernière ligne est sans LF (parse_journal_bytes qui traite la queue
    // arrachée comme corruption centrale) ; OU écrire ligne puis LF en deux
    // write_all (fenêtre de corruption).
    {
        let mut journal = CatalogueJournal::open(&journal_path).expect("P1 open");
        journal
            .consign_observed_fact(&ObservedFact {
                kind: CoveredFactKind::GateFailed.as_str().into(),
                source_id: "G-session".into(),
                date: "2026-08-24T04:00:00Z".into(),
                text: "gate session rouge — doit survivre".into(),
            })
            .expect("P1 consign gate");
        journal
            .consign_observed_fact(&ObservedFact {
                kind: CoveredFactKind::ReviewAmender.as_str().into(),
                source_id: "r-session".into(),
                date: "2026-08-24T04:01:00Z".into(),
                text: "AMENDER session — doit survivre".into(),
            })
            .expect("P1 consign amender");
        // Troisième ligne destinée à être arrachée.
        journal
            .append_add(maicie::catalogue::AddEntry {
                v: 1,
                kind: maicie::catalogue::AddKind::Add,
                id: "a-torn-victim".into(),
                date: "2026-08-24T04:02:00Z".into(),
                mission_source: MissionSource {
                    kind: MissionSourceKind::Incident,
                    id: "inc-torn".into(),
                    failed: None,
                },
                severity: Severity::Info,
                recurrence_of: None,
                text: "victime de troncature volontaire".into(),
            })
            .expect("P1 append victime");
    }
    let intact = fs::read(&journal_path).expect("P1 read intact");
    assert!(
        intact.ends_with(b"\n"),
        "PROMESSE(1) SURVIE — MUTATION: write sans LF final unique ; \
         l'append doit terminer chaque enregistrement par un seul write ligne+LF"
    );
    let text = String::from_utf8(intact).expect("P1 utf8");
    let without_final_lf = text.trim_end_matches('\n');
    let last_start = without_final_lf.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let cut = last_start + (without_final_lf.len() - last_start) / 2;
    assert!(
        cut > last_start,
        "PROMESSE(1) SURVIE — la troncature doit couper AU MILIEU de la dernière ligne"
    );
    fs::write(&journal_path, &without_final_lf.as_bytes()[..cut]).expect("P1 truncate");

    let after_crash = parse_journal_bytes(&fs::read(&journal_path).expect("P1 reread"))
        .expect("PROMESSE(1) SURVIE — MUTATION: parse qui refuse le journal entier \
                 sur queue arrachée ; les entrées antérieures DOIVENT rester lisibles");
    assert!(
        after_crash.torn_tail_warning.is_some(),
        "PROMESSE(1) SURVIE — MUTATION: ignorer la queue arrachée sans avertissement \
         (silence = promesse non observée)"
    );
    let survived_ids: Vec<_> = after_crash
        .entries
        .iter()
        .filter_map(|e| match e {
            CatalogueEntry::Add(a) => Some(a.id.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        survived_ids.contains(&"gate_failed:G-session"),
        "PROMESSE(1) SURVIE — entrée antérieure gate_failed perdue après crash simulé"
    );
    assert!(
        survived_ids.contains(&"review_amender:r-session"),
        "PROMESSE(1) SURVIE — entrée antérieure review_amender perdue après crash simulé"
    );
    assert!(
        !survived_ids.iter().any(|id| id.contains("torn")),
        "PROMESSE(1) SURVIE — la ligne arrachée ne doit PAS être acceptée comme add valide"
    );

    // Réécrire un journal sain pour la suite du gate (même fichier déclaré).
    fs::write(&journal_path, b"").expect("reset");
    let mut journal = CatalogueJournal::open(&journal_path).expect("reopen");

    // ── (2) MIGRATION REJOUABLE ─────────────────────────────────────────────
    // Mutation qui ferait ÉCHOUER : supprimer le dédup par provenance_id
    // (second passage append N lignes de plus).
    let first = journal
        .migrate_prose_file(&prose_path)
        .expect("PROMESSE(2) MIGRATION — premier passage");
    assert!(
        first.read >= 40 && first.appended == first.read && first.skipped == 0,
        "PROMESSE(2) MIGRATION — corpus réel attendu (~40) : read={} appended={} skipped={}",
        first.read,
        first.appended,
        first.skipped
    );
    let texts_after_first: Vec<Vec<u8>> = journal
        .read_entries()
        .expect("P2 read")
        .into_iter()
        .filter_map(|e| match e {
            CatalogueEntry::PendingQualification(p) => Some(p.text.into_bytes()),
            _ => None,
        })
        .collect();
    assert_eq!(
        texts_after_first.len(),
        first.appended,
        "PROMESSE(2) MIGRATION — chaque ligne migrée doit être un pending"
    );

    let second = journal
        .migrate_prose_file(&prose_path)
        .expect("PROMESSE(2) MIGRATION — second passage");
    assert_eq!(
        second.read, first.read,
        "PROMESSE(2) MIGRATION — le corpus lu doit être stable"
    );
    assert_eq!(
        second.appended, 0,
        "PROMESSE(2) MIGRATION — MUTATION: omettre le dédup provenance ; \
         un second passage ne doit RIEN appendre (got appended={})",
        second.appended
    );
    assert_eq!(
        second.skipped, first.read,
        "PROMESSE(2) MIGRATION — MUTATION: compter skipped=0 au rejeu ; \
         toutes les provenances doivent être skipped"
    );
    let pending_count = journal
        .read_entries()
        .expect("P2 recount")
        .iter()
        .filter(|e| matches!(e, CatalogueEntry::PendingQualification(_)))
        .count();
    assert_eq!(
        pending_count, first.appended,
        "PROMESSE(2) MIGRATION — MUTATION: doubler les pending au rejeu"
    );

    // ── (3) QUALIFICATION SANS RÉÉCRITURE ───────────────────────────────────
    // Mutation qui ferait ÉCHOUER : qualify_pending qui modifie/normalise le
    // texte (trim, réécriture, troncature) avant l'add.
    let (pending_id, verbatim): (String, Vec<u8>) = journal
        .read_entries()
        .expect("P3 read")
        .into_iter()
        .find_map(|e| match e {
            CatalogueEntry::PendingQualification(p) => Some((p.id, p.text.into_bytes())),
            _ => None,
        })
        .expect("PROMESSE(3) QUALIF — au moins un pending après migration");
    assert!(
        !verbatim.is_empty(),
        "PROMESSE(3) QUALIF — texte migré non vide attendu"
    );
    journal
        .qualify_pending(
            &pending_id,
            Severity::Minor,
            MissionSource {
                kind: MissionSourceKind::Incident,
                id: "qualif-humaine".into(),
                failed: None,
            },
            "2026-08-24T05:00:00Z".into(),
        )
        .expect("PROMESSE(3) QUALIF — qualify");
    let after_text = journal
        .read_entries()
        .expect("P3 reread")
        .into_iter()
        .find_map(|e| match e {
            CatalogueEntry::Add(a) if a.id == pending_id => Some(a.text.into_bytes()),
            _ => None,
        })
        .expect("PROMESSE(3) QUALIF — l'add de qualification doit exister");
    assert_eq!(
        after_text, verbatim,
        "PROMESSE(3) QUALIF — MUTATION: réécrire/normaliser le texte migré ; \
         octet pour octet exigé (len avant={}, après={})",
        verbatim.len(),
        after_text.len()
    );

    // ── (4) TABLE FR-1711 : DÉRIVE, N'INVENTE PAS ───────────────────────────
    // Mutation qui ferait ÉCHOUER : mapper review_approve → info/minor ;
    // OU gate_failed → major ; OU hors-table → add au lieu de pending.
    let (gate_out, _) = journal
        .consign_observed_fact(&ObservedFact {
            kind: "gate_failed".into(),
            source_id: "G-gate".into(),
            date: "2026-08-24T05:10:00Z".into(),
            text: "gate G-gate rouge".into(),
        })
        .expect("P4 gate");
    match gate_out {
        TranscriptionOutcome::CoveredAdd(add) => {
            assert_eq!(
                add.severity,
                Severity::Blocker,
                "PROMESSE(4) TABLE — MUTATION: dériver gate_failed vers autre chose que blocker"
            );
        }
        TranscriptionOutcome::Pending(_) => {
            panic!("PROMESSE(4) TABLE — gate_failed DOIT être couvert, pas pending")
        }
    }

    let (amender_out, _) = journal
        .consign_observed_fact(&ObservedFact {
            kind: "review_amender".into(),
            source_id: "r-amend".into(),
            date: "2026-08-24T05:11:00Z".into(),
            text: "AMENDER nuit".into(),
        })
        .expect("P4 amender");
    match amender_out {
        TranscriptionOutcome::CoveredAdd(add) => {
            assert_eq!(
                add.severity,
                Severity::Major,
                "PROMESSE(4) TABLE — MUTATION: dériver review_amender vers autre chose que major"
            );
        }
        TranscriptionOutcome::Pending(_) => {
            panic!("PROMESSE(4) TABLE — review_amender DOIT être couvert, pas pending")
        }
    }

    let (approve_out, approve_outcome) = journal
        .consign_observed_fact(&ObservedFact {
            kind: "review_approve".into(),
            source_id: "r-ok".into(),
            date: "2026-08-24T05:12:00Z".into(),
            text: "APPROVE hors table".into(),
        })
        .expect("P4 approve");
    assert_eq!(approve_outcome, AppendOutcome::Appended);
    match approve_out {
        TranscriptionOutcome::Pending(pending) => {
            assert!(
                pending.provenance_id.starts_with("uncovered:"),
                "PROMESSE(4) TABLE — MUTATION: inventer une sévérité pour hors-table ; \
                 provenance uncovered exigée, got {}",
                pending.provenance_id
            );
        }
        TranscriptionOutcome::CoveredAdd(add) => {
            panic!(
                "PROMESSE(4) TABLE — MUTATION: traiter hors-table comme couvert \
                 (severity={:?}) ; le défaut est l'attente",
                add.severity
            );
        }
    }

    // ── (5) VUE D'AUTORITÉ = VÉRITÉ DU JOURNAL ──────────────────────────────
    // Mutation qui ferait ÉCHOUER : footer hardcodé ; compter les delivered
    // comme ouverts ; oublier P ; inventer K sans gate failed.
    //
    // Oracle d'ÉTAT uniquement : on recompte depuis les entrées brutes, hors
    // de `project_registre`, puis on compare. Aucun libellé humain du rendu
    // n'est consulté (un merge a déjà cassé un gate fondateur pour cette
    // raison exacte : oracle sur texte obsolète).
    let entries = journal.read_entries().expect("P5 entries");
    let independent = authority_counts_from_entries(&entries);
    let view = project_registre(&entries);

    assert_eq!(
        view.footer.ouverts, independent.ouverts,
        "PROMESSE(5) VUE — MUTATION: footer.ouverts désynchronisé du journal \
         (vue={}, oracle={})",
        view.footer.ouverts, independent.ouverts
    );
    assert_eq!(
        view.footer.recurrents, independent.recurrents,
        "PROMESSE(5) VUE — MUTATION: footer.recurrents inventé"
    );
    assert_eq!(
        view.footer.gates_rates, independent.gates_rates,
        "PROMESSE(5) VUE — MUTATION: footer K inventé ou oublié"
    );
    assert_eq!(
        view.footer.pending_qualification, independent.pending_qualification,
        "PROMESSE(5) VUE — MUTATION: footer P inventé ou oublié"
    );
    assert_eq!(
        view.ouverts.len(),
        independent.ouverts,
        "PROMESSE(5) VUE — MUTATION: liste ouverts et footer.ouverts divergent"
    );
    assert_eq!(
        view.attente.len(),
        independent.pending_qualification,
        "PROMESSE(5) VUE — MUTATION: liste attente et footer.P divergent"
    );
    assert!(
        independent.gates_rates >= 1,
        "PROMESSE(5) VUE — au moins un gate failed ouvert attendu (K={})",
        independent.gates_rates
    );
    assert!(
        independent.pending_qualification >= 1,
        "PROMESSE(5) VUE — l'APPROVE hors table doit compter dans P (P={})",
        independent.pending_qualification
    );

    // Mutation critique : compter un constat delivered comme encore ouvert.
    // On clôt `gate_failed:G-gate` via transition attestée, puis on vérifie
    // que N et K baissent d'un et que l'id disparaît des ouverts.
    let delivered_id = "gate_failed:G-gate";
    assert!(
        view.ouverts.iter().any(|c| c.id == delivered_id),
        "PROMESSE(5) VUE — précondition : {delivered_id} doit être ouvert avant transition"
    );
    let before = independent;
    journal
        .append_delivered_for_attested_closure(
            &ArbitrationLink {
                constat_id: delivered_id.into(),
                objective_id: "obj-gate-session".into(),
            },
            &AttestedClosure {
                objective_id: "obj-gate-session".into(),
                observed_at: "2026-08-24T05:30:00Z".into(),
            },
        )
        .expect("P5 transition delivered");
    let after_entries = journal.read_entries().expect("P5 after transition");
    let after = authority_counts_from_entries(&after_entries);
    let after_view = project_registre(&after_entries);
    assert_eq!(
        after.ouverts,
        before.ouverts - 1,
        "PROMESSE(5) VUE — MUTATION: compter delivered comme ouvert ; N doit baisser de 1"
    );
    assert_eq!(
        after.gates_rates,
        before.gates_rates - 1,
        "PROMESSE(5) VUE — MUTATION: garder un gate delivered dans K"
    );
    assert_eq!(
        after.delivered,
        before.delivered + 1,
        "PROMESSE(5) VUE — MUTATION: ignorer la transition dans le décompte delivered"
    );
    assert!(
        !after_view.ouverts.iter().any(|c| c.id == delivered_id),
        "PROMESSE(5) VUE — MUTATION: {delivered_id} encore listé ouvert après delivered"
    );
    assert_eq!(
        after_view.footer.ouverts, after.ouverts,
        "PROMESSE(5) VUE — footer post-transition encore synchronisé"
    );

    eprintln!(
        "gate_session_017 OK — N={} M={} K={} P={} delivered={}",
        after.ouverts, after.recurrents, after.gates_rates, after.pending_qualification, after.delivered
    );
}

/// Oracle indépendant de `project_registre` : mêmes règles, autre code.
///
/// Si la projection mente ou hardcode le pied, le gate échoue parce que ce
/// compteur ne lit que les entrées brutes.
fn authority_counts_from_entries(entries: &[CatalogueEntry]) -> AuthorityCounts {
    let mut delivered: BTreeSet<String> = BTreeSet::new();
    let mut adds: BTreeMap<String, &maicie::catalogue::AddEntry> = BTreeMap::new();
    let mut pendings: BTreeMap<String, &maicie::catalogue::PendingQualificationEntry> =
        BTreeMap::new();

    for entry in entries {
        match entry {
            CatalogueEntry::Add(add) => {
                adds.insert(add.id.clone(), add);
            }
            CatalogueEntry::Transition(transition) => {
                delivered.insert(transition.constat_id.clone());
            }
            CatalogueEntry::PendingQualification(pending) => {
                pendings.insert(pending.id.clone(), pending);
            }
        }
    }

    let ouverts: Vec<_> = adds
        .values()
        .filter(|add| !delivered.contains(&add.id))
        .collect();
    let recurrents = ouverts
        .iter()
        .filter(|add| add.recurrence_of.is_some())
        .count();
    let gates_rates = ouverts
        .iter()
        .filter(|add| {
            add.mission_source.kind == MissionSourceKind::Gate
                && add.mission_source.failed == Some(true)
        })
        .count();
    let pending_qualification = pendings
        .values()
        .filter(|pending| !adds.contains_key(&pending.id))
        .count();

    AuthorityCounts {
        ouverts: ouverts.len(),
        recurrents,
        gates_rates,
        pending_qualification,
        delivered: delivered.len(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AuthorityCounts {
    ouverts: usize,
    recurrents: usize,
    gates_rates: usize,
    pending_qualification: usize,
    delivered: usize,
}
