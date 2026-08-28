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
    let historique = r#"{"id":"00000000-0000-0000-0000-000000000056","but":"objectif antérieur","mode":"delegue","etat":"ouvert","cree_at":1,"mis_a_jour_at":1,"synthese":null,"decision_en_attente_id":null}"#;

    let decoded: ObjectifCoordonne = serde_json::from_slice(historique.as_bytes()).unwrap();
    assert_eq!(decoded.origin, ObjectiveOrigin::LegacyUnknown);
    assert_eq!(
        serde_json::to_vec(&decoded).unwrap(),
        historique.as_bytes(),
        "un payload historique doit rester identique octet par octet"
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
        !domain.contains("fn human_request"),
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

    let compact_store: String = store.chars().filter(|c| !c.is_whitespace()).collect();
    let insert = "INSERTINTOobjectives(id,state,payload_json)";
    assert_eq!(
        compact_store.matches(insert).count(),
        2,
        "l'inventaire doit voir l'ouverture durable et la sonde de migration annulée"
    );

    let open_start = store.find("fn open_objective(").unwrap();
    let update_start = store.find("fn update_objective(").unwrap();
    let open_block = &store[open_start..update_start];
    assert!(
        open_block.contains("if &objective.origin != opening_permit.origin()")
            && open_block.contains("INSERT INTO objectives(id, state, payload_json)"),
        "l'ouverture durable ne vérifie pas le permit au point INSERT"
    );
    let transition_start = store.find("fn transition_existing_objective(").unwrap();
    assert!(
        !store[update_start..transition_start].contains("INSERT INTO objectives"),
        "la mise à jour peut encore créer implicitement un objectif"
    );

    let preflight_start = store
        .find("fn verify_local_delegate_refusals_shape_v18(")
        .unwrap();
    let preflight_end = store[preflight_start..]
        .find("fn migrate_guichet_refusal_vocabulary_v19(")
        .unwrap()
        + preflight_start;
    let preflight = &store[preflight_start..preflight_end];
    assert!(
        preflight.contains("SAVEPOINT maicie_v19_preflight_v18")
            && preflight.contains("INSERT INTO objectives(id,state,payload_json)")
            && preflight.contains("ROLLBACK TO maicie_v19_preflight_v18")
            && preflight.contains("objectives_before != objectives_after"),
        "la seconde insertion doit rester une sonde annulée dont le cardinal est contrôlé"
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
