//! Une limite constatée ouvre une question, jamais une nouvelle exécution.
use super::*;

fn failure(task: &Task) -> bool {
    task.state == "failed"
        && task.error.as_deref().is_some_and(|error| {
            error == "usage_limit"
                || serde_json::from_str::<Value>(error)
                    .ok()
                    .is_some_and(|value| value["code"] == "usage_limit")
        })
}

fn authorized(task: &Task) -> bool {
    matches!(
        &task.request,
        NativeDelegationRequest::Delegate {
            quota_fallback_authorized: Some(true),
            ..
        }
    )
}

fn id(task: &Task) -> String {
    Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        format!("bridget:quota-question246:{}", task.task_id).as_bytes(),
    )
    .to_string()
}

fn questions(task: &Task) -> Value {
    let (engine, machine) = match &task.request {
        NativeDelegationRequest::Delegate {
            agent_type,
            machine_id,
            ..
        } => (
            agent_type.as_str(),
            machine_id.as_deref().unwrap_or("local"),
        ),
        _ => ("fournisseur", "machine admise"),
    };
    form(&task.task_id, engine, machine)
}

fn form(subject: &str, engine: &str, machine: &str) -> Value {
    json!([{"id":"quota_recovery","header":"Quota atteint",
    "question":format!("Quota fournisseur atteint pour {engine} sur {machine}. Travail concerné : {}. Comment autorises-tu la suite ? Les travaux existants restent conservés. Une reprise attend la confirmation de l’arrêt, ou de l’absence, d’une exécution. Aucun compte nouveau, droit ou budget supplémentaire n’est accordé.",subject),
    "multiSelect":false,"allowCustomAnswer":true,"options":[
        {"label":"Attendre le même compte","value":"wait","description":"Garder le moteur et le compte. Une date inconnue reste inconnue ; aucun redémarrage n’est lancé par cette réponse."},
        {"label":"Autoriser un autre compte","value":"change_account","description":"Pour cette mission, utiliser un autre compte existant du même moteur, dans les droits et le budget déjà autorisés."},
        {"label":"Autoriser un autre moteur","value":"change_engine","description":"Pour cette mission, autoriser un autre moteur et une connexion existante, dans les droits et le budget déjà autorisés."}
    ]}])
}

fn valid_row(task: &Task, row: &Value) -> bool {
    row["owner"] == task.root_owner()
        && row["origin"] == "mcp"
        && row["callback"].is_null()
        && row["questions"] == questions(task)
}

pub(super) fn ensure(st: &DaemonState, task: &Task) -> Result<(), String> {
    if !failure(task) || authorized(task) || task.failure_sent {
        return Ok(());
    }
    let db = st.store.connection();
    let question = id(task);
    if let Some(row) = crate::user_input::get(db, &question)? {
        return if valid_row(task, &row) {
            Ok(())
        } else {
            Err("quota_question_conflict".into())
        };
    }
    let owner = task.root_owner();
    let Some(instance) = st
        .router
        .get_agent(owner)
        .and_then(|route| live_connection_identity(st, &route.connection_id))
        .filter(|(agent, _)| agent == owner)
        .map(|(_, instance)| instance)
    else {
        return Ok(());
    };
    let project = agent_communication_project(st, owner)
        .map(|p| serde_json::to_string(&p))
        .transpose()
        .map_err(|_| "question_project_unavailable")?;
    crate::user_input::create(
        db,
        owner,
        &instance,
        project.as_deref(),
        &question,
        &questions(task),
        "mcp",
        &Value::Null,
        unix_timestamp(),
    )?;
    st.delegation_store.wait_epoch().bump();
    if let Some(notifier) = &st.mobile_notifier {
        notifier.changed();
    }
    Ok(())
}

pub(super) fn attach(st: &DaemonState, task: &Task, view: &mut Value) {
    if !failure(task) {
        return;
    }
    if authorized(task) {
        view["quota_recovery"] = json!({"state":"prior_authorization_declared","scope":"this_mission","automatic_restart":false,
            "next_action":"L’appelant a déclaré l’accord humain explicite de bascule pour cette mission. Ne pas redemander cet accord. Vérifier d’abord la clôture physique et le dossier ; conserver cette tâche et ne reprendre que le travail restant dans le périmètre déjà autorisé."});
    } else {
        let question = id(task);
        let row = crate::user_input::get(st.store.connection(), &question)
            .ok()
            .flatten()
            .filter(|row| valid_row(task, row));
        if row.is_none() && task.failure_sent {
            return;
        }

        view["quota_recovery"] = json!({"state":if row.is_some(){"human_question"}else{"question_pending"},"request_id":question,"scope":"this_mission","automatic_restart":false,
            "question":row.map(crate::user_input::projection),
            "next_action":"Attendre la réponse humaine avec bridget_ask_user action wait et le request_id indiqué, sans recréer la question. Le choix initial automatique ne vaut pas accord de bascule. Un accord explicite antérieur dans le fil reste applicable dans son périmètre : le parent peut fermer cette question devenue sans objet, sans fabriquer de réponse humaine. Avant toute reprise, vérifier la clôture physique et le dossier, conserver l’ancienne tâche et sa clé, puis ne reprendre que le travail restant. Aucun nouvel accès ni budget déduit."});
    }
}

/// Une sonde qui constate un quota peut aussi refuser avant toute création de Task.
/// Seuls les faits de quota observés activent ce producteur, pas un refus générique.
pub(super) fn pre_admission(
    st: &DaemonState,
    conn: &str,
    request: &NativeDelegationRequest,
    known_quota: bool,
    view: &mut Value,
) -> Result<(), String> {
    if !known_quota {
        return Ok(());
    }
    let (key, engine, machine, allowed) = match request {
        NativeDelegationRequest::AutoDelegateV2(r) => (
            r.request_id.as_str(),
            r.engine.as_deref().unwrap_or("moteur admissible"),
            r.machine_id.as_deref().unwrap_or("destination demandée"),
            r.quota_fallback_authorized == Some(true),
        ),
        NativeDelegationRequest::Delegate {
            request_id,
            agent_type,
            machine_id,
            quota_fallback_authorized,
            ..
        } => (
            request_id.as_str(),
            agent_type.as_str(),
            machine_id.as_deref().unwrap_or("local"),
            *quota_fallback_authorized == Some(true),
        ),
        _ => return Ok(()),
    };
    if allowed {
        view["quota_recovery"] = json!({"state":"prior_authorization_declared","scope":"this_mission","automatic_restart":false,"next_action":"L’accord de bascule pour cette mission est déjà déclaré : ne pas le redemander. Conserver le refus original et vérifier l’absence attestée d’admission avant de préparer la suite autorisée ; aucun autre compte ni moteur n’est prouvé disponible par ce seul refus."});
        return Ok(());
    }
    let (owner, instance) = live_connection_identity(st, conn).ok_or("identity_unavailable")?;
    let owner = st
        .delegation_store
        .tasks()?
        .into_iter()
        .find(|t| t.child_instance.as_deref() == Some(instance.as_str()))
        .map_or(owner, |t| t.root_owner().to_owned());
    let instance = st
        .router
        .get_agent(&owner)
        .and_then(|r| live_connection_identity(st, &r.connection_id))
        .map(|(_, i)| i)
        .ok_or("question_owner_unavailable")?;
    let canonical = serde_json::to_vec(request).map_err(|_| "invalid_request")?;
    let question = Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        format!(
            "bridget:quota-admission246:{owner}:{key}:{:x}",
            sha2::Sha256::digest(canonical)
        )
        .as_bytes(),
    )
    .to_string();
    let form = form(&format!("demande {key}, non admise"), engine, machine);
    let db = st.store.connection();
    let existing = crate::user_input::get(db, &question)?;
    if existing.as_ref().is_some_and(|r| {
        r["owner"] != owner
            || r["origin"] != "mcp"
            || !r["callback"].is_null()
            || r["questions"] != form
    }) {
        return Err("quota_question_conflict".into());
    }
    let project = agent_communication_project(st, &owner)
        .map(|p| serde_json::to_string(&p))
        .transpose()
        .map_err(|_| "question_project_unavailable")?;
    let row = match existing {
        Some(r) => r,
        None => {
            let row = crate::user_input::create(
                db,
                &owner,
                &instance,
                project.as_deref(),
                &question,
                &form,
                "mcp",
                &Value::Null,
                unix_timestamp(),
            )?;
            st.delegation_store.wait_epoch().bump();
            if let Some(n) = &st.mobile_notifier {
                n.changed();
            }
            row
        }
    };
    view["quota_recovery"] = json!({"state":"human_question","request_id":question,"scope":"this_mission","automatic_restart":false,"question":crate::user_input::projection(row),"next_action":"Une question humaine est ouverte. Relever sa réponse avec bridget_ask_user action wait et ce request_id, sans la recréer. Ne changer ni moteur ni compte avant son accord, sauf autorisation humaine explicite antérieure applicable. Conserver le refus et la clé d’origine ; avant une nouvelle enveloppe, vérifier l’absence attestée d’admission. Un choix initial libre n’autorise pas une bascule après blocage."});
    Ok(())
}

pub(super) fn notice(st: &DaemonState, task: &Task) -> String {
    if !failure(task) {
        return String::new();
    }
    let mut view = Value::Null;
    attach(st, task, &mut view);
    format!(
        "\nReprise quota : {}\nrequest_id de question : {}. Les autres travaux autorisés suivent leur propre état.",
        view["quota_recovery"]["next_action"]
            .as_str()
            .unwrap_or("Conserver la tâche et vérifier sa question native."),
        view["quota_recovery"]["request_id"]
            .as_str()
            .unwrap_or("sans question, accord préalable déclaré")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (DaemonState, Task, UnixStream) {
        let (mut st, _) = crate::daemon::presence_tests::state_with_registered_agent("quota246");
        let peer = spec094_live_test_connection(&mut st, "conn-1");
        let owner = st.conn_names["conn-1"].clone();
        let instance = st.conn_instances["conn-1"].clone();
        let task=serde_json::from_value(json!({"task_id":Uuid::new_v4().to_string(),"owner":owner,
            "owner_instance":instance,"origin_owner_instance":instance,
            "request":{"operation":"delegate","request_id":"quota246","agent_type":"claude","model":"fixture","task":"Mission fictive","cwd":"/tmp"},
            "definition":st.registry.resolved_definition("claude").unwrap(),"cwd":"/tmp",
            "child":Uuid::new_v4().to_string(),"mission":Uuid::new_v4().to_string(),
            "created_at":unix_timestamp(),"state":"failed","error":"usage_limit","result_sent":false})).unwrap();
        (st, task, peer)
    }
    #[test]
    fn spec246_question_unique_humaine_et_releve_sans_muter_la_tache() {
        let (st, task, _peer) = fixture();
        let before = serde_json::to_vec(&task).unwrap();
        ensure(&st, &task).unwrap();
        ensure(&st, &task).unwrap();
        let question = id(&task);
        let row = crate::user_input::get(st.store.connection(), &question)
            .unwrap()
            .unwrap();
        crate::user_input::validate_questions(&row["questions"]).unwrap();
        assert_eq!(row["state"], "pending");
        assert_eq!(row["owner"], task.owner);
        let command = Uuid::new_v4().to_string();
        let answer = json!({"quota_recovery":{"selected":["change_engine"],"text":""}});
        crate::user_input::respond(
            st.store.connection(),
            &question,
            row["revision"].as_str().unwrap(),
            &command,
            "human-test246",
            &answer,
            unix_timestamp(),
        )
        .unwrap();
        ensure(&st, &task).unwrap();
        let mut view = json!({});
        attach(&st, &task, &mut view);
        assert_eq!(view["quota_recovery"]["question"]["state"], "submitted");
        assert_eq!(view["quota_recovery"]["question"]["answers"], answer);
        assert_eq!(serde_json::to_vec(&task).unwrap(), before);
        assert_eq!(
            st.store
                .connection()
                .query_row("SELECT COUNT(*) FROM user_input_requests", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn spec246_refus_quota_avant_admission_pose_question_sans_tache() {
        let (st, task, _peer) = fixture();
        let mut view = json!({"status":"refused","admission":{"state":"not_recorded"}});
        pre_admission(&st, "conn-1", &task.request, false, &mut view).unwrap();
        assert!(view.get("quota_recovery").is_none());
        pre_admission(&st, "conn-1", &task.request, true, &mut view).unwrap();
        assert_eq!(view["quota_recovery"]["question"]["state"], "pending");
        let first = view["quota_recovery"]["request_id"].clone();
        pre_admission(&st, "conn-1", &task.request, true, &mut view).unwrap();
        assert_eq!(view["quota_recovery"]["request_id"], first);
        assert_eq!(view["admission"]["state"], "not_recorded");
        assert!(st.delegation_store.tasks().unwrap().is_empty());
    }
    #[test]
    fn spec246_echec_deja_signale_ne_cree_pas_de_question_retroactive() {
        let (st, mut task, _peer) = fixture();
        task.failure_sent = true;
        let before = serde_json::to_vec(&task).unwrap();
        ensure(&st, &task).unwrap();
        assert!(
            crate::user_input::get(st.store.connection(), &id(&task))
                .unwrap()
                .is_none()
        );
        let mut view = json!({});
        attach(&st, &task, &mut view);
        assert!(view.get("quota_recovery").is_none());
        assert_eq!(serde_json::to_vec(&task).unwrap(), before);
    }

    #[test]
    fn spec246_question_etrangere_ne_fuit_pas_vers_la_tache() {
        let (st, task, _peer) = fixture();
        let key = id(&task);
        crate::user_input::create(
            st.store.connection(),
            "other-owner",
            "other-instance",
            None,
            &key,
            &questions(&task),
            "mcp",
            &Value::Null,
            unix_timestamp(),
        )
        .unwrap();
        assert_eq!(ensure(&st, &task).unwrap_err(), "quota_question_conflict");
        let mut view = json!({});
        attach(&st, &task, &mut view);
        assert!(view["quota_recovery"]["question"].is_null());
        assert!(!view.to_string().contains("other-owner"));
    }
    #[test]
    fn spec246_accord_declare_epargne_question_mais_pas_nouveaux_droits() {
        let (st, mut task, _peer) = fixture();
        if let NativeDelegationRequest::Delegate {
            quota_fallback_authorized,
            ..
        } = &mut task.request
        {
            *quota_fallback_authorized = Some(true);
        }
        ensure(&st, &task).unwrap();
        assert!(
            crate::user_input::get(st.store.connection(), &id(&task))
                .unwrap()
                .is_none()
        );
        let mut view = json!({});
        attach(&st, &task, &mut view);
        assert_eq!(
            view["quota_recovery"]["state"],
            "prior_authorization_declared"
        );
        assert_eq!(view["quota_recovery"]["scope"], "this_mission");
        assert_eq!(view["quota_recovery"]["automatic_restart"], false);
        assert!(notice(&st, &task).contains("Ne pas redemander cet accord"));
    }
    #[test]
    fn spec246_seul_refus_quota_constate_pose_question() {
        let (st, mut task, _peer) = fixture();
        task.error = Some("api_error".into());
        ensure(&st, &task).unwrap();
        assert!(
            crate::user_input::get(st.store.connection(), &id(&task))
                .unwrap()
                .is_none()
        );
        task.state = "working".into();
        task.error = Some("usage_limit".into());
        ensure(&st, &task).unwrap();
        assert!(
            crate::user_input::get(st.store.connection(), &id(&task))
                .unwrap()
                .is_none()
        );
        task.state = "failed".into();
        task.error = Some(r#"{"code":"usage_limit","limit_reset_at":9999999999}"#.into());
        ensure(&st, &task).unwrap();
        assert!(
            crate::user_input::get(st.store.connection(), &id(&task))
                .unwrap()
                .is_some()
        );
    }
}
