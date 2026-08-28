use maicie::app::{DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::config::DurationClasses;
use maicie::domain::{
    ClasseDuree, ModeObjectif, ObjectifCoordonne, ObjectiveOrigin, SuiteObjective,
};
use maicie::store::MaicieStore;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use uuid::Uuid;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("maicie-origin-{label}-{}", Uuid::new_v4()))
}

#[test]
fn spec_056_nouvel_objectif_porte_une_origine_automatique_explicitement() {
    let objectif =
        ObjectifCoordonne::nouveau("mesurer la provenance", ModeObjectif::Delegue, 10).unwrap();
    let value = serde_json::to_value(objectif).unwrap();

    assert_eq!(value["origin"], json!({"kind": "auto_generated"}));
}

#[test]
fn spec_056_payload_historique_sans_origine_reste_inconnu() {
    let historique = json!({
        "id": Uuid::new_v4(),
        "but": "objectif antérieur",
        "mode": "delegue",
        "etat": "ouvert",
        "cree_at": 1,
        "mis_a_jour_at": 1,
        "synthese": null,
        "decision_en_attente_id": null
    });

    let decoded: ObjectifCoordonne = serde_json::from_value(historique).unwrap();
    assert_eq!(decoded.origin, ObjectiveOrigin::LegacyUnknown);
    let roundtrip = serde_json::to_value(decoded).unwrap();

    assert!(
        roundtrip.get("origin").is_none(),
        "un payload historique doit rester byte-compatible sans champ ajouté"
    );
}

#[test]
fn spec_056_delegate_persiste_l_origine_automatique() {
    let root = root("delegate");
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "rc5".to_string(),
        tags: Vec::new(),
        available: true,
        dnd: false,
    }];
    let request = DelegateRequest {
        goal: "ouvrir avec une provenance",
        opening_permit: maicie::domain::ObjectiveOpeningPermit::auto_generated(),
        explicit_target: Some("rc5"),
        required_tags: &[],
        duration: ClasseDuree::Normale,
        reply: false,
        constat_id: None,
        review_target: None,
        suite: SuiteObjective::Aucune,
        depends_on: &[],
        references: &[],
        idempotency_key: "spec-056-origin",
        now: 10,
        retry_until: 20,
        dedup_retained_until: 20,
        max_frame_bytes: 256 * 1024,
    };
    let DelegateResult::Created(created) = delegate(
        &mut store,
        DurationClasses {
            short_secs: 30,
            normal_secs: 60,
            long_secs: 90,
        },
        "maicie",
        &candidates,
        &request,
    )
    .unwrap() else {
        panic!("création attendue")
    };
    drop(store);

    let connection = Connection::open(&database).unwrap();
    let (payload, canonical_request_bytes): (Vec<u8>, Vec<u8>) = connection
        .query_row(
            "SELECT o.payload_json, i.canonical_request_bytes
             FROM objectives o
             JOIN delegate_idempotency i ON i.objective_id = o.id
             WHERE o.id = ?1",
            [created.objective_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let value: Value = serde_json::from_slice(&payload).unwrap();
    assert_eq!(value["origin"], json!({"kind": "auto_generated"}));
    let canonical: Value = serde_json::from_slice(&canonical_request_bytes).unwrap();
    assert_eq!(canonical["v"], 2);
    assert!(
        canonical.get("origin").is_none(),
        "la voie automatique doit conserver les octets canoniques historiques"
    );

    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec_056_toutes_les_frontieres_d_ouverture_exigent_un_permit() {
    let app = include_str!("../src/app.rs");
    let domain = include_str!("../src/domain.rs");
    let greffe = include_str!("../src/greffe_service.rs");
    let main = include_str!("../src/main.rs");
    let store = include_str!("../src/store.rs");
    let routines = include_str!("../src/routines.rs");

    let request_start = app
        .find("pub struct DelegateRequest<'a> {")
        .expect("DelegateRequest absent");
    let request_block = &app[request_start..request_start + 1_500];
    assert!(
        request_block.contains("pub opening_permit: ObjectiveOpeningPermit"),
        "la frontière delegate n'exige aucun permit typé"
    );
    assert_eq!(domain.matches("pub fn auto_generated() -> Self").count(), 1);
    assert!(
        !domain.contains("pub fn human_request"),
        "la voie humaine ne doit pas être constructible avant l'attestation daemon"
    );

    for function in [
        "pub fn lookup_or_reserve_delegate(",
        "pub fn lookup_or_reserve_delegate_observed(",
        "pub fn lookup_or_reserve_waiting_delegate(",
    ] {
        let start = store.find(function).expect("fonction créatrice absente");
        let signature = &store[start..store[start..].find(" {").unwrap() + start];
        assert!(
            signature.contains("opening_permit"),
            "frontière sans permit : {function}"
        );
    }

    let delegate_start = app.find("pub fn delegate(").expect("delegate absent");
    let delegate_end = app[delegate_start..]
        .find("\nfn validate_suite_and_citations(")
        .expect("fin de delegate absente")
        + delegate_start;
    let delegate_body = &app[delegate_start..delegate_end];
    assert!(delegate_body.contains("ObjectifCoordonne::nouveau_avec_permit("));
    assert_eq!(
        delegate_body.matches("&request.opening_permit").count(),
        3,
        "le constructeur et les deux réservations doivent consommer le même permit"
    );

    let insert = "INSERT INTO objectives(id, state, payload_json)";
    assert_eq!(
        store.matches(insert).count(),
        1,
        "une seule insertion neuve"
    );
    let insert_at = store.find(insert).unwrap();
    let guard_start = insert_at.saturating_sub(1_500);
    assert!(
        store[guard_start..insert_at].contains("opening_permit"),
        "la branche INSERT ne vérifie aucun permit"
    );

    for (producer, source, marker) in [
        (
            "guichet",
            greffe,
            "let delegate_request = DelegateRequest {",
        ),
        ("CLI", main, "let request = DelegateRequest {"),
        ("routine", routines, "let request = DelegateRequest {"),
    ] {
        let request = source
            .find(marker)
            .unwrap_or_else(|| panic!("construction {producer} absente"));
        let block = &source[request..request + 1_500];
        assert!(
            block.contains("opening_permit: ObjectiveOpeningPermit::auto_generated()"),
            "{producer} ne déclare pas son origine automatique"
        );
    }

    let helper_start = store
        .find("pub fn create_prepared_delegation_observed(")
        .expect("helper de store absent");
    let helper_end = store[helper_start..]
        .find("\n    /// Retrouve objective_id")
        .expect("fin du helper de store absente")
        + helper_start;
    let helper = &store[helper_start..helper_end];
    assert!(helper.contains("ObjectiveOpeningPermit::auto_generated()"));
    assert!(helper.contains("insert_prepared(&tx, prepared, &opening_permit)"));
}
