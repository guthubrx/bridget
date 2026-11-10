    use super::*;
    const AGENT: &str = "89000000-0000-4000-8000-000000000102";
    fn human() -> SpecialistPrincipal {
        SpecialistPrincipal {
            agent_id: None,
            human_approval_ref: Some("bridget-control-cli:proof".into()),
            allowed_projects: vec!["*".into()],
        }
    }
    fn profile(id: &str) -> SpecialistRequest {
        serde_json::from_value(json!({"version":1,"action":"propose_profile","request_id":Uuid::new_v4().to_string(),
        "definition":{"profile_id":id,"name":"Recherche","kind":"profile","roles":["recherche"],"instructions":"Examiner et sourcer."}})).unwrap()
    }
    fn prepare(st: &DaemonState, id: &str, durable: bool) -> SpecialistRecord {
        let mut store = profiles(st).unwrap();
        store.specialist_apply(&profile(id), &human(), 10).unwrap();
        let approve=serde_json::from_value(json!({"version":1,"action":"approve_revision","request_id":Uuid::new_v4().to_string(),"profile_id":id,"revision":1})).unwrap();
        store.specialist_apply(&approve, &human(), 11).unwrap();
        let activate=serde_json::from_value(json!({"version":1,"action":"activate","request_id":Uuid::new_v4().to_string(),"specialist_agent_id":AGENT,"profile_id":id,"profile_revision":1,
            "expected_revision":0,"scope":{"projects":["project"]},"durable":durable,"auto_start":true,
            "launch_policy":{"agent_type":"claude","cwd":"/tmp","posture":"discovery"}})).unwrap();
        store.specialist_apply(&activate, &human(), 12).unwrap();
        store.get_specialist(AGENT).unwrap().unwrap()
    }
    #[test]
    fn spec222_release_checks_current_grant_and_scope_without_process() {
        let (st, _) = super::super::presence_tests::state_with_registered_agent("222-release");
        let profile = Uuid::new_v4().to_string();
        let record = prepare(&st, &profile, true);
        let grant = record.grant.unwrap();
        let canonical = br#"{"role_id":"recherche"}"#;
        let mut message = bridget_core::BridgetMessage::new("human", AGENT, "Test RELEASE");
        message.id = crate::execution_store::specialists::specialist_submission_id(
            "human", "release", canonical,
        );
        st.execution_store
            .reserve_specialist_request(
                "human",
                "release",
                canonical,
                AGENT,
                1,
                Some(&grant.grant_id),
                grant.revision,
                &message,
                &json!({"budget":{},"role_official":false}),
                unix_timestamp(),
            )
            .unwrap();
        let authority = crate::lifecycle::SpecialistLaunchAuthority {
            db_path: st.db_path.clone(),
            agent_id: AGENT.into(),
            grant_id: grant.grant_id,
            grant_revision: grant.revision,
            profile_revision: 1,
            profile_id: profile,
            project_id: "project".into(),
            submission_id: message.id,
            human_approval_ref: None,
        };
        authority.lock().unwrap().finish().unwrap();
        let disable = serde_json::from_value(
            json!({"version":1,"action":"disable","request_id":Uuid::new_v4().to_string(),
            "specialist_agent_id":AGENT,"expected_revision":1}),
        )
        .unwrap();
        profiles(&st)
            .unwrap()
            .specialist_apply(&disable, &human(), 13)
            .unwrap();
        assert!(
            authority.lock().is_err(),
            "la révocation entre réserve et RELEASE ferme le départ"
        );
        assert!(st.managed_by_instance.is_empty());
    }
    #[test]
    fn spec222_approval_requires_native_human_response_and_exact_operation() {
        let (mut st, _) = super::super::presence_tests::state_with_registered_agent("222-human");
        let (writer, _reader) = super::super::presence_tests::control_socket("222-human");
        st.connections.insert("conn-1".into(), writer);
        let id = Uuid::new_v4().to_string();
        let proposal = profile(&id);
        profiles(&st)
            .unwrap()
            .specialist_apply(&proposal, &human(), 10)
            .unwrap();
        let operation: SpecialistRequest =
            serde_json::from_value(json!({"version":1,"action":"approve_revision",
            "request_id":Uuid::new_v4().to_string(),"profile_id":id,"revision":1}))
            .unwrap();
        let principal = SpecialistPrincipal {
            agent_id: Some(AGENT.into()),
            human_approval_ref: None,
            allowed_projects: vec!["project".into()],
        };
        let key = Uuid::new_v4().to_string();
        let mut controls = Vec::new();
        assert_eq!(
            request_approval(
                &mut st,
                "conn-1",
                &principal,
                &key,
                &operation,
                &mut controls
            )
            .unwrap()["status"],
            "awaiting_human"
        );
        assert!(
            profiles(&st)
                .unwrap()
                .get_definition(&id, 1)
                .unwrap()
                .unwrap()
                .approved_ref
                .is_none()
        );
        let question = crate::user_input::get(st.store.connection(), &key)
            .unwrap()
            .unwrap();
        crate::user_input::respond(
            st.store.connection(),
            &key,
            question["revision"].as_str().unwrap(),
            &Uuid::new_v4().to_string(),
            "cli",
            &json!({"specialist_approval":{"selected":["approve"],"text":""}}),
            unix_timestamp(),
        )
        .unwrap();
        assert_eq!(
            request_approval(
                &mut st,
                "conn-1",
                &principal,
                &key,
                &operation,
                &mut controls
            )
            .unwrap()["status"],
            "ok"
        );
        assert!(
            profiles(&st)
                .unwrap()
                .get_definition(&id, 1)
                .unwrap()
                .unwrap()
                .approved_ref
                .is_some()
        );
        assert_eq!(
            crate::user_input::get(st.store.connection(), &key)
                .unwrap()
                .unwrap()["state"],
            "applied"
        );
        let before: u64 = st
            .store
            .connection()
            .query_row("SELECT COUNT(*) FROM specialist_mutations", [], |r| {
                r.get(0)
            })
            .unwrap();
        let approved_ref = profiles(&st)
            .unwrap()
            .get_definition(&id, 1)
            .unwrap()
            .unwrap()
            .approved_ref;
        assert_eq!(
            request_approval(
                &mut st,
                "conn-1",
                &principal,
                &key,
                &operation,
                &mut controls
            )
            .unwrap()["status"],
            "ok"
        );
        let after: u64 = st
            .store
            .connection()
            .query_row("SELECT COUNT(*) FROM specialist_mutations", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            before, after,
            "une décision applied ne réapplique pas la mutation"
        );
        assert_eq!(
            profiles(&st)
                .unwrap()
                .get_definition(&id, 1)
                .unwrap()
                .unwrap()
                .approved_ref,
            approved_ref
        );
        let changed = serde_json::from_value(
            json!({"version":1,"action":"approve_revision","request_id":Uuid::new_v4().to_string(),
            "profile_id":id,"revision":1}),
        )
        .unwrap();
        assert!(
            request_approval(&mut st, "conn-1", &principal, &key, &changed, &mut controls).is_err()
        );
    }

    #[test]
    fn spec239_legacy_mobile_admin_applies_only_the_recorded_human_decision() {
        use crate::mobile::store::{Scope,Store};
        let (mut st,_)=super::super::presence_tests::state_with_registered_agent_configured("239-legacy-approval",|config|{
            let root=config.db_path.with_extension("private");
            crate::environment::ensure_private_directory(&root).unwrap();
            config.db_path=root.join("bridget.db");
        });
        let (writer,_reader)=super::super::presence_tests::control_socket("239-legacy-approval");
        st.connections.insert("conn-1".into(),writer);
        let now=unix_timestamp();
        let mut devices=Store::open(&st.db_path.parent().unwrap().join("mobile-devices.db")).unwrap();
        let scope:Scope=serde_json::from_value(json!({"all_machines":true,"machines":[],"all_projects":true,"projects":[],
            "operations":["read","message","task_read","task_continue","task_cancel","task_respond","devices_manage"]})).unwrap();
        let pairing=devices.issue_pairing(&scope,now).unwrap();
        let device=devices.pair(&pairing.credential,&Uuid::new_v4().to_string(),
            &format!("{}{}",Uuid::new_v4().simple(),Uuid::new_v4().simple()),"Legacy administrator",now).unwrap();
        // La fixture émule une association ancienne renouvelée : sa naissance
        // précède la capacité, son expiration reste actuelle. Aucune base réelle.
        rusqlite::Connection::open(st.db_path.parent().unwrap().join("mobile-devices.db")).unwrap()
            .execute("UPDATE mobile_devices SET created_at=?1 WHERE device_id=?2",
                rusqlite::params![crate::mobile::store::SPECIALISTS_MANAGEMENT_INTRODUCED_AT-1,device.device_id]).unwrap();
        let actor=format!("mobile181:{}",device.device_id);
        let id=Uuid::new_v4().to_string();
        profiles(&st).unwrap().specialist_apply(&profile(&id),&human(),now).unwrap();
        let operation=serde_json::from_value(json!({"version":1,"action":"approve_revision",
            "request_id":Uuid::new_v4().to_string(),"profile_id":id,"revision":1})).unwrap();
        let principal=SpecialistPrincipal{agent_id:Some(AGENT.into()),human_approval_ref:None,allowed_projects:vec!["project".into()]};
        let key=Uuid::new_v4().to_string();let mut controls=Vec::new();
        assert_eq!(request_approval(&mut st,"conn-1",&principal,&key,&operation,&mut controls).unwrap()["status"],"awaiting_human");
        assert!(profiles(&st).unwrap().get_definition(&id,1).unwrap().unwrap().approved_ref.is_none());
        let question=crate::user_input::get(st.store.connection(),&key).unwrap().unwrap();
        crate::user_input::respond(st.store.connection(),&key,question["revision"].as_str().unwrap(),
            &Uuid::new_v4().to_string(),&actor,&json!({"specialist_approval":{"selected":["approve"],"text":""}}),now).unwrap();
        assert_eq!(request_approval(&mut st,"conn-1",&principal,&key,&operation,&mut controls).unwrap()["status"],"ok");
        let approved=profiles(&st).unwrap().get_definition(&id,1).unwrap().unwrap().approved_ref;
        assert!(approved.is_some());
        assert_eq!(crate::user_input::get(st.store.connection(),&key).unwrap().unwrap()["state"],"applied");
        assert_eq!(request_approval(&mut st,"conn-1",&principal,&key,&operation,&mut controls).unwrap()["status"],"ok");
        assert_eq!(profiles(&st).unwrap().get_definition(&id,1).unwrap().unwrap().approved_ref,approved);
        devices.revoke(&device.device_id,now).unwrap();
        assert_eq!(decision_projects(&st,&json!({"actor":actor})),Err("specialists_admin_scope_required".into()));
        let earlier=now-31*86_400;
        let pairing=devices.issue_pairing(&scope,earlier).unwrap();
        let expired=devices.pair(&pairing.credential,&Uuid::new_v4().to_string(),
            &format!("{}{}",Uuid::new_v4().simple(),Uuid::new_v4().simple()),"Expired administrator",earlier).unwrap();
        assert_eq!(decision_projects(&st,&json!({"actor":format!("mobile181:{}",expired.device_id)})),Err("specialists_admin_scope_required".into()));
        assert!(controls.is_empty());
        assert!(st.managed_by_instance.is_empty());
    }

    #[test]
    fn spec239_mobile_command_keeps_partial_and_new_associations_restricted() {
        use crate::mobile::store::{Device,Scope,SPECIALISTS_MANAGEMENT_INTRODUCED_AT};
        let (mut st,_)=super::super::presence_tests::state_with_registered_agent("239-mobile-boundary");
        st.client_negotiations.insert("mobile".into(),NegotiatedClient{version:CLIENT_CONTRACT_VERSION,
            issuer_scope:"test239".into(),capabilities:vec![ClientCapability::SpecialistsV1]});
        let profile_id=Uuid::new_v4().to_string();let record=prepare(&st,&profile_id,false);
        let command:SpecialistRequest=serde_json::from_value(json!({"version":1,"action":"configure_mandate",
            "request_id":Uuid::new_v4().to_string(),"specialist_agent_id":AGENT,"expected_revision":record.revision,
            "mandate":"Test","scope":{"projects":["project"],"transversal":false},"auto_start":false})).unwrap();
        let scope:Scope=serde_json::from_value(json!({"all_machines":true,"machines":[],"all_projects":true,"projects":[],
            "operations":["read","message","task_read","task_continue","task_cancel","task_respond","devices_manage"]})).unwrap();
        let legacy=Device{device_id:Uuid::new_v4().to_string(),label:"Legacy".into(),scope,
            created_at:SPECIALISTS_MANAGEMENT_INTRODUCED_AT-1,expires_at:i64::MAX,revoked_at:None};
        let shared=Arc::new(Mutex::new(st));
        let result=|device:&Device|match handle_mobile("mobile",&command,device,&shared){
            DaemonToWrapper::MobileOperationResult{result}=>result,_=>panic!("mobile result expected")};
        let mut restricted=legacy.clone();restricted.scope.all_projects=false;
        assert_eq!(result(&restricted)["code"],"specialists_admin_scope_required");
        restricted.scope.operations.push(crate::mobile::store::Operation::SpecialistsManage);
        assert_eq!(result(&restricted)["code"],"specialists_admin_scope_required","explicit mobile mutations retain the existing global-project guard");
        restricted=legacy.clone();restricted.created_at=SPECIALISTS_MANAGEMENT_INTRODUCED_AT;
        assert_eq!(result(&restricted)["code"],"specialists_admin_scope_required");
        assert_eq!(result(&legacy)["status"],"ok");
        let st=shared.lock().unwrap();
        let after=profiles(&st).unwrap().get_specialist(AGENT).unwrap().unwrap();
        assert_eq!(after.revision,record.revision+1);assert!(!after.auto_start);
        assert!(st.managed_by_instance.is_empty());
    }
    #[test]
    fn spec222_contact_is_idempotent_without_durable_activation_and_status_is_scoped() {
        let (mut st, _) = super::super::presence_tests::state_with_registered_agent("222-contact");
        let (writer, _reader) = super::super::presence_tests::control_socket("222-contact");
        st.connections.insert("conn-1".into(), writer);
        let mut store = profiles(&st).unwrap();
        let actor = SpecialistPrincipal {
            agent_id: Some(AGENT.into()),
            human_approval_ref: None,
            allowed_projects: vec!["project".into()],
        };
        let declaration=serde_json::from_value(json!({"version":1,"action":"declare_roles","request_id":"declaration","expected_revision":0,
            "roles":[{"role_id":"recherche","label":"Recherche","scope":{"projects":["project"]}}]})).unwrap();
        store.specialist_apply(&declaration, &actor, 10).unwrap();
        drop(store);
        let request=serde_json::from_value(json!({"version":1,"action":"request_role","request_id":"contact","role_id":"recherche","project_id":"project","task":"Expliquer la méthode."})).unwrap();
        st.presences.get_mut("instance-1").unwrap().state = "busy".into();
        let mut controls = Vec::new();
        let one = request_role(
            &mut st,
            "control",
            &human(),
            &request,
            "contact",
            "recherche",
            "project",
            "Expliquer la méthode.",
            &mut controls,
        )
        .unwrap();
        assert_eq!(one["status"], "submitted", "{one}");
        assert_eq!(st.presences["instance-1"].state, "busy");
        let controls_before = controls.len();
        let two = request_role(
            &mut st,
            "control",
            &human(),
            &request,
            "contact",
            "recherche",
            "project",
            "Expliquer la méthode.",
            &mut controls,
        )
        .unwrap();
        assert_eq!(two["status"], "known");
        assert_eq!(controls.len(), controls_before);
        assert!(
            !profiles(&st)
                .unwrap()
                .get_specialist(AGENT)
                .unwrap()
                .unwrap()
                .durable
        );
        let other = SpecialistPrincipal {
            agent_id: None,
            human_approval_ref: Some("mobile:proof".into()),
            allowed_projects: vec!["elsewhere".into()],
        };
        let status = SpecialistRequest::RequestStatus {
            version: 1,
            request_id: "contact".into(),
        };
        assert_eq!(
            apply(&mut st, "control", &status, &other, &mut controls).unwrap_err(),
            "role_scope_denied"
        );
    }
    #[test]
    fn spec222_constraints_and_no_budget_default_are_explicit() {
        let (st, _) = super::super::presence_tests::state_with_registered_agent("222-constraints");
        let record = prepare(&st, &Uuid::new_v4().to_string(), false);
        assert_eq!(record.budget, SpecialistBudget::default());
        assert_eq!(
            validate_authority(&record, "project", false).unwrap_err(),
            "human_reuse_approval_required"
        );
        assert!(validate_authority(&record, "project", true).is_ok());
        let prefs = SpecialistPreferences {
            machine_id: Some("other-machine".into()),
            ..Default::default()
        };
        assert_eq!(
            check_specialist_preferences(&st, &record, &prefs, true).unwrap_err(),
            "specialist_constraint_mismatch"
        );
    }
    #[test]
    fn spec222_team_partial_admission_pins_plan_and_reuses_native_members() {
        let (mut st, _) = super::super::presence_tests::state_with_registered_agent("222-team");
        let (writer, _reader) = super::super::presence_tests::control_socket("222-team");
        st.connections.insert("conn-1".into(), writer);
        let actor = SpecialistPrincipal {
            agent_id: Some(AGENT.into()),
            human_approval_ref: None,
            allowed_projects: vec!["project".into()],
        };
        let roles=serde_json::from_value(json!({"version":1,"action":"declare_roles","request_id":"claim","expected_revision":0,
            "roles":[{"role_id":"recherche","label":"Recherche","scope":{"projects":["project"]}}]})).unwrap();
        profiles(&st)
            .unwrap()
            .specialist_apply(&roles, &actor, 10)
            .unwrap();
        let id = Uuid::new_v4().to_string();
        let definition=serde_json::from_value(json!({"version":1,"action":"propose_profile","request_id":"team-definition",
            "definition":{"profile_id":id,"name":"Équipe","kind":"team","instructions":"Deux livrables distincts.",
                "members":[{"key":"source","role_id":"recherche","deliverable":"Sources"},{"key":"review","role_id":"recherche","deliverable":"Revue","depends_on":["source"]}]}})).unwrap();
        let mut store = profiles(&st).unwrap();
        store.specialist_apply(&definition, &human(), 10).unwrap();
        let approve=serde_json::from_value(json!({"version":1,"action":"approve_revision","request_id":"team-approve","profile_id":id,"revision":1})).unwrap();
        store.specialist_apply(&approve, &human(), 11).unwrap();
        drop(store);
        let request = SpecialistRequest::RequestTeam {
            version: 1,
            request_id: "team".into(),
            profile_id: id.clone(),
            revision: 1,
            project_id: "project".into(),
            task: "Comparer les sources.".into(),
        };
        let mut controls = Vec::new();
        let first = request_team(
            &mut st,
            "control",
            &human(),
            &request,
            "team",
            &id,
            1,
            "project",
            "Comparer les sources.",
            &mut controls,
        )
        .unwrap();
        assert_eq!(
            first["attempts"][0]["result"]["status"], "submitted",
            "{first}"
        );
        assert_eq!(first["attempts"][1]["status"], "waiting_dependencies");
        let source = first["data"]["members"][0]["native"]["native_submission_id"]
            .as_str()
            .unwrap();
        let mut answer = bridget_core::BridgetMessage::new(AGENT, "human", "Sources remises.");
        answer.in_reply_to = Some(source.into());
        assert!(capture_human_reply(&st, "foreign", &answer).is_err());
        assert!(capture_human_reply(&st, "conn-1", &answer).unwrap());
        let second = request_team(
            &mut st,
            "control",
            &human(),
            &request,
            "team",
            &id,
            1,
            "project",
            "Comparer les sources.",
            &mut controls,
        )
        .unwrap();
        assert_eq!(second["attempts"][0]["result"]["status"], "known");
        assert_eq!(
            second["attempts"][1]["result"]["status"], "submitted",
            "{second}"
        );
        let n: u64 = st
            .store
            .connection()
            .query_row("SELECT COUNT(*) FROM specialist_requests", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 2);
        let third = request_team(
            &mut st,
            "control",
            &human(),
            &request,
            "team",
            &id,
            1,
            "project",
            "Comparer les sources.",
            &mut controls,
        )
        .unwrap();
        assert_eq!(third["attempts"][1]["result"]["status"], "known");
        let changed = SpecialistRequest::RequestTeam {
            version: 1,
            request_id: "team".into(),
            profile_id: id.clone(),
            revision: 1,
            project_id: "project".into(),
            task: "Autre travail".into(),
        };
        assert!(
            request_team(
                &mut st,
                "control",
                &human(),
                &changed,
                "team",
                &id,
                1,
                "project",
                "Autre travail",
                &mut controls
            )
            .is_err()
        );
    }
    #[test]
    fn spec222_optional_budget_is_inherited_and_counts_native_boundaries() {
        let (st, _) = super::super::presence_tests::state_with_registered_agent("222-budget");
        let mut m = bridget_core::BridgetMessage::new("human", AGENT, "Budget explicite.");
        let canon = b"root-budget";
        m.id =
            crate::execution_store::specialists::specialist_submission_id("human", "budget", canon);
        st.execution_store.reserve_specialist_request("human","budget",canon,AGENT,0,None,0,&m,
            &json!({"budget":{"max_duration_secs":null,"max_facturable_tokens":null,"max_descendants":0}}),unix_timestamp()).unwrap();
        st.execution_store
            .accept_specialist_request("human", "budget", canon, 0, None, unix_timestamp())
            .unwrap();
        assert_eq!(
            guard_child_admission(&st, AGENT).unwrap_err(),
            "specialist_budget_reached"
        );
        let policy = budget_for_execution(&st, &format!("execution-{}", m.id)).unwrap();
        assert_eq!(policy.max_descendants, Some(0));
        let other = Uuid::new_v4().to_string();
        let mut child = bridget_core::BridgetMessage::new(AGENT, &other, "Sous-demande.");
        let canonical = b"child-budget";
        child.id = crate::execution_store::specialists::specialist_submission_id(
            AGENT, "child", canonical,
        );
        st.execution_store
            .reserve_specialist_request(
                AGENT,
                "child",
                canonical,
                &other,
                0,
                None,
                0,
                &child,
                &json!({"budget":{},"inherited_budget_executions":[format!("execution-{}",m.id)]}),
                unix_timestamp(),
            )
            .unwrap();
        st.execution_store
            .accept_specialist_request(AGENT, "child", canonical, 0, None, unix_timestamp())
            .unwrap();
        assert_eq!(
            guard_child_admission(&st, &other).unwrap_err(),
            "specialist_budget_reached"
        );
        assert_eq!(
            budget_for_execution(&st, &format!("execution-{}", child.id))
                .unwrap()
                .max_descendants,
            Some(0)
        );
    }
    #[test]
    fn spec222_old_client_cannot_declare_or_create_a_specialist() {
        let (mut st, _) = super::super::presence_tests::state_with_registered_agent("222-old");
        let count: usize = st
            .store
            .connection()
            .query_row("SELECT COUNT(*) FROM specialists", [], |r| r.get(0))
            .unwrap();
        let q = profile(&Uuid::new_v4().to_string());
        assert_eq!(
            handle_agent(&mut st, "conn-1", AGENT, None, &q, &mut Vec::new()).unwrap_err(),
            "specialists_capability_required"
        );
        let after: usize = st
            .store
            .connection()
            .query_row("SELECT COUNT(*) FROM specialists", [], |r| r.get(0))
            .unwrap();
        assert_eq!(after, count);
    }
    #[test]
    fn spec222_temporary_profile_is_frozen_before_native_admission_and_not_reused() {
        let (mut st, _) =
            super::super::presence_tests::state_with_registered_agent("222-temporary");
        st.client_negotiations.insert(
            "conn-1".into(),
            NegotiatedClient {
                version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "test222".into(),
                capabilities: vec![ClientCapability::SpecialistsV1],
            },
        );
        let (writer, _reader) = super::super::presence_tests::control_socket("222-temporary");
        st.connections.insert("conn-1".into(), writer);
        st.communication_projects.insert(
            "conn-1".into(),
            (
                Some(bridget_transport::protocol::CommunicationProject {
                    host: "macbook".into(),
                    root: "/tmp/project222".into(),
                }),
                None,
                false,
            ),
        );
        let id = Uuid::new_v4().to_string();
        let actor = SpecialistPrincipal {
            agent_id: Some(AGENT.into()),
            human_approval_ref: None,
            allowed_projects: vec!["project".into()],
        };
        profiles(&st)
            .unwrap()
            .specialist_apply(&profile(&id), &actor, 10)
            .unwrap();
        let value = json!({"version":1,"action":"request_temporary","request_id":"temporary","profile_id":id,"profile_revision":1,
            "delegation":{"request_id":"temporary","cwd":"/tmp/project222","task":"Revoir les sources.","posture":"discovery","workspace":{"strategy":"shared"}}});
        let request: SpecialistRequest = serde_json::from_value(value.clone()).unwrap();
        let native = prepare_temporary(&st, "conn-1", AGENT, &request).unwrap();
        assert!(native.task.contains("Examiner et sourcer."));
        assert!(native.task.contains("Revoir les sources."));
        assert_eq!(native.request_id, "temporary");
        assert_eq!(
            serde_json::to_value(prepare_temporary(&st, "conn-1", AGENT, &request).unwrap())
                .unwrap(),
            serde_json::to_value(&native).unwrap()
        );
        let mut reused = value.clone();
        reused["request_id"] = json!("another");
        reused["delegation"]["request_id"] = json!("another");
        assert_eq!(
            prepare_temporary(
                &st,
                "conn-1",
                AGENT,
                &serde_json::from_value(reused).unwrap()
            )
            .unwrap_err(),
            "human_reuse_approval_required"
        );
        let mut changed = value;
        changed["delegation"]["task"] = json!("Autre travail.");
        assert_eq!(
            prepare_temporary(
                &st,
                "conn-1",
                AGENT,
                &serde_json::from_value(changed).unwrap()
            )
            .unwrap_err(),
            "envelope_mismatch"
        );
        assert!(st.managed_by_instance.is_empty());
    }
