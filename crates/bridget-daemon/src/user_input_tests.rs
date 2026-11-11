use super::*;
use serde_json::json;
fn questions() -> Value {
    json!([{"id":"q1","header":"Choix","question":"Quel chemin ?","options":[{"label":"A","description":"Premier chemin"},{"label":"B","description":"Second chemin"}],"multiSelect":false,"allowCustomAnswer":true}])
}
fn db() -> Connection {
    let c = Connection::open_in_memory().unwrap();
    ensure_schema(&c).unwrap();
    c
}
fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
#[test]
fn user_input181_retention_scrubs_once_and_preserves_tombstone() {
    let c = db();
    let request = id();
    create(
        &c,
        "agent",
        "instance",
        None,
        &request,
        &questions(),
        "mcp",
        &Value::Null,
        100,
    )
    .unwrap();
    finish(&c, &request, "agent", "instance", None, "cancelled", 101).unwrap();
    list_page(&c, None, RETENTION + 200, 50).unwrap();
    let row = get(&c, &request).unwrap().unwrap();
    assert_eq!(row["state"], "cancelled");
    assert!(row.get("questions").is_none());
    let changes = c.total_changes();
    list_page(&c, None, RETENTION + 201, 50).unwrap();
    assert_eq!(
        c.total_changes(),
        changes,
        "reading a scrubbed tombstone must not rewrite its data"
    );
}
#[test]
fn user_input181_active_question_blocks_maintenance_until_confirmed() {
    let c = db();
    let request = id();
    let row = create(
        &c,
        "agent",
        "instance",
        None,
        &request,
        &questions(),
        "mcp",
        &Value::Null,
        100,
    )
    .unwrap();
    assert!(has_active(&c, 101).unwrap());
    let command = id();
    respond(
        &c,
        &request,
        row["revision"].as_str().unwrap(),
        &command,
        "human",
        &json!({"q1":{"selected":["A"],"text":""}}),
        101,
    )
    .unwrap();
    assert!(has_active(&c, 102).unwrap());
    finish(
        &c,
        &request,
        "agent",
        "instance",
        Some(&command),
        "applied",
        103,
    )
    .unwrap();
    assert!(!has_active(&c, 104).unwrap());
}
#[test]
fn user_input181_preserves_options_and_free_complement() {
    let q = questions();
    assert!(validate_questions(&q).is_ok());
    assert!(
        validate_answers(
            &q,
            &json!({"q1":{"selected":["B"],"text":"Mais conserve les fichiers"}})
        )
        .is_ok()
    );
    assert!(
        validate_answers(
            &q,
            &json!({"q1":{"selected":[],"text":"Une autre réponse"}})
        )
        .is_ok()
    );
}
#[test]
fn user_input181_rejects_foreign_option_and_invalid_cardinality() {
    let q = questions();
    for a in [
        json!({"q1":{"selected":["C"],"text":""}}),
        json!({"q1":{"selected":["A","B"],"text":""}}),
        json!({"q1":{"selected":[],"text":""}}),
        json!({"foreign":{"selected":["A"],"text":""}}),
    ] {
        assert!(validate_answers(&q, &a).is_err());
    }
}
#[test]
fn user_input181_multiple_and_text_capability() {
    let mut q = questions();
    q[0]["multiSelect"] = json!(true);
    assert!(validate_answers(&q, &json!({"q1":{"selected":["A","B"],"text":"Précision"}})).is_ok());
    q[0]["allowCustomAnswer"] = json!(false);
    assert!(validate_answers(&q, &json!({"q1":{"selected":["A"],"text":"Précision"}})).is_err());
}
#[test]
fn user_input181_rejects_secret_unknown_and_duplicate_question_ids() {
    let mut q = questions();
    q[0]["isSecret"] = json!(true);
    assert!(validate_questions(&q).is_err());
    let q = questions();
    assert!(validate_questions(&json!([q[0].clone(), q[0].clone()])).is_err());
    let mut q = questions();
    q[0]["question"] = json!("Texte\u{0}secret");
    assert!(validate_questions(&q).is_err());
}
#[test]
fn user_input181_create_replay_and_collision_do_not_replace() {
    let c = db();
    let request = id();
    let q = questions();
    let first = create(
        &c,
        "agent",
        "instance",
        Some("project"),
        &request,
        &q,
        "mcp",
        &Value::Null,
        100,
    )
    .unwrap();
    let replay = create(
        &c,
        "agent",
        "instance",
        Some("project"),
        &request,
        &q,
        "mcp",
        &Value::Null,
        101,
    )
    .unwrap();
    assert_eq!(first, replay);
    let mut changed = q.clone();
    changed[0]["question"] = json!("Autre question");
    assert!(
        create(
            &c,
            "agent",
            "instance",
            Some("project"),
            &request,
            &changed,
            "mcp",
            &Value::Null,
            102
        )
        .is_err()
    );
    assert!(
        create(
            &c,
            "other",
            "instance",
            Some("project"),
            &request,
            &q,
            "mcp",
            &Value::Null,
            102
        )
        .is_err()
    );
}
#[test]
fn user_input181_response_replay_revision_and_competing_answer() {
    let c = db();
    let request = id();
    let q = create(
        &c,
        "agent",
        "instance",
        None,
        &request,
        &questions(),
        "mcp",
        &Value::Null,
        100,
    )
    .unwrap();
    let rev = q["revision"].as_str().unwrap();
    let cmd = id();
    let a = json!({"q1":{"selected":["B"],"text":"Complément"}});
    assert!(respond(&c, &request, "wrong", &cmd, "human", &a, 101).is_err());
    let first = respond(&c, &request, rev, &cmd, "human", &a, 101).unwrap();
    assert_eq!(first["state"], "submitted");
    assert_eq!(first["answers"], a);
    assert_eq!(
        respond(&c, &request, rev, &cmd, "human", &a, 102).unwrap(),
        first
    );
    assert!(
        respond(
            &c,
            &request,
            rev,
            &cmd,
            "human",
            &json!({"q1":{"selected":["A"],"text":""}}),
            102
        )
        .is_err()
    );
    assert!(respond(&c, &request, rev, &id(), "other-human", &a, 102).is_err());
}
