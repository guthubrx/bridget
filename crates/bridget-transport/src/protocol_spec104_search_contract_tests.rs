    use super::*;

    #[test]
    fn spec104_requete_de_recherche_stricte_et_sans_identite() {
        let request: LedgerSearchRequest = serde_json::from_value(serde_json::json!({
            "query": "pagination erreur",
            "peer": "11111111-1111-4111-8111-111111111111",
            "limit": 20
        }))
        .unwrap();
        assert_eq!(request.source, LedgerSearchSource::Messages);
        assert_eq!(request.limit, Some(20));
        assert!(request.cursor.is_none());
        // Aucun champ d'identité, d'instance ou de chemin n'est accepté.
        for forbidden in [
            "agent_id",
            "acting_agent",
            "instance_id",
            "db_path",
            "scope",
            "project_id",
        ] {
            let value = serde_json::json!({"query": "x", forbidden: "y"});
            assert!(
                serde_json::from_value::<LedgerSearchRequest>(value).is_err(),
                "champ {forbidden} accepté"
            );
        }
        // Types stricts : booléen ou flottant refusés pour since/limit.
        assert!(
            serde_json::from_value::<LedgerSearchRequest>(
                serde_json::json!({"query":"x","since":true})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<LedgerSearchRequest>(
                serde_json::json!({"query":"x","limit":1.5})
            )
            .is_err()
        );
    }

    #[test]
    fn spec104_trames_serialisees_par_nom_et_resultats_types() {
        let frame = WrapperToDaemon::LedgerSearch {
            request: LedgerSearchRequest {
                source: LedgerSearchSource::Thread,
                query: "décision".into(),
                author: None,
                peer: None,
                since: None,
                until: None,
                limit: None,
                cursor: None,
                thread_id: Some("22222222-2222-4222-8222-222222222222".into()),
            },
        };
        let json = serde_json::to_value(&frame).unwrap();
        assert_eq!(json["type"], "ledger_search");
        assert_eq!(json["request"]["source"], "thread");
        assert!(json["request"].get("peer").is_none());
        let back: WrapperToDaemon = serde_json::from_value(json).unwrap();
        assert!(matches!(back, WrapperToDaemon::LedgerSearch { .. }));

        let outcome = LedgerSearchOutcomeV1::Ok(LedgerSearchPage {
            source: LedgerSearchSource::Messages,
            hits: vec![LedgerSearchHit::Message {
                id: "m1".into(),
                target: "t".into(),
                sender: "s".into(),
                ts: 7,
                excerpt: "…".into(),
                match_offset: 3,
                body_digest: "ab".into(),
                body_bytes: 12,
            }],
            has_more: true,
            next_cursor: Some("00".into()),
            scanned_count: 1,
            scanned_bytes: 12,
            skipped_oversized: 0,
            stop_reason: "result_limit".into(),
            consistency: "live_bounded".into(),
            notices: vec![],
        });
        let json = serde_json::to_value(DaemonToWrapper::LedgerSearchResult { outcome }).unwrap();
        assert_eq!(json["type"], "ledger_search_result");
        assert_eq!(json["outcome"]["status"], "ok");
        assert_eq!(json["outcome"]["hits"][0]["kind"], "message");
        assert_eq!(json["outcome"]["hits"][0]["match_offset"], 3);

        let refus = serde_json::to_value(LedgerReadOutcomeV1::Error {
            code: "not_found_or_forbidden".into(),
            reason: "source indisponible".into(),
        })
        .unwrap();
        assert_eq!(
            refus,
            serde_json::json!({"status":"error","code":"not_found_or_forbidden","reason":"source indisponible"})
        );

        let read: LedgerReadRequest =
            serde_json::from_value(serde_json::json!({"id":"handoff-pagination-01","target":"t"}))
                .unwrap();
        assert_eq!(read.offset, 0);
        assert!(read.digest.is_none());
        assert!(
            serde_json::from_value::<LedgerReadRequest>(
                serde_json::json!({"id":"a","target":"b","offset":-1})
            )
            .is_err()
        );
    }

    #[test]
    fn spec104_ancien_contrat_ledger_inchange() {
        // Un client ou un daemon antérieur continue d'échanger la projection
        // historique : trame et champs identiques à la version précédente.
        let request = serde_json::to_value(WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Messages,
            limit: 20,
        })
        .unwrap();
        assert_eq!(
            request,
            serde_json::json!({"type":"LedgerProjection","scope":"Messages","limit":20})
        );
        // Un ancien daemon ne connaît pas la trame 104 : le client doit obtenir
        // une erreur explicite, jamais un repli. Ici : la trame reste
        // syntaxiquement distincte de toute trame historique.
        let unknown: Result<WrapperToDaemon, _> =
            serde_json::from_str(r#"{"type":"ledger_search_v2","request":{"query":"x"}}"#);
        assert!(unknown.is_err());
    }

    #[test]
    fn native150_delai_requete_sans_champ_garde_les_octets_149() {
        // Sans `reply_timeout`, la requête sérialise exactement comme en 149.
        let request = NativeDelegationRequest::Delegate {
            machine_id: None,
            project_id: None,
            request_id: "k".to_string(),
            agent_type: "a".to_string(),
            model: "m".to_string(),
            effort: None,
            task: "t".to_string(),
            cwd: "/c".to_string(),
            posture: None,
            reply_timeout: None,
            workspace: None,
            keep_alive_secs: None,
            brief: None,
            child_requests: None,
            quota_fallback_authorized: None,
            resume_on_limit: None,
            fallback: Vec::new(),
        };
        let expected = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/c","posture":null}"#;
        assert_eq!(serde_json::to_vec(&request).unwrap(), expected.to_vec());
        let reread: NativeDelegationRequest = serde_json::from_slice(expected).unwrap();
        assert_eq!(reread, request);
        let NativeDelegationRequest::Delegate { reply_timeout, .. } = serde_json::from_str(
            r#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/c","reply_timeout":120}"#,
        )
        .unwrap() else {
            panic!("variante attendue : Delegate");
        };
        assert_eq!(reply_timeout, Some(120));
    }

    #[test]
    fn native153_protocole_delegate_sans_champs_garde_octets150() {
        let wire = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/c","posture":null,"reply_timeout":120}"#;
        let request: NativeDelegationRequest = serde_json::from_slice(wire).unwrap();
        let NativeDelegationRequest::Delegate {
            keep_alive_secs,
            brief,
            ..
        } = &request
        else {
            panic!("variante attendue : Delegate");
        };
        assert_eq!(*keep_alive_secs, None);
        assert_eq!(*brief, None);
        assert_eq!(serde_json::to_vec(&request).unwrap(), wire.to_vec());
    }

    #[test]
    fn native154_contrat_sans_workspace_octets153() {
        let wire = r#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/c","posture":null,"reply_timeout":120,"keep_alive_secs":600,"brief":{"objective":"objectif","summary":"résultat"}}"#.as_bytes();
        let request: NativeDelegationRequest = serde_json::from_slice(wire).unwrap();
        assert_eq!(serde_json::to_vec(&request).unwrap(), wire.to_vec());
        assert!(
            serde_json::to_value(request)
                .unwrap()
                .get("workspace")
                .is_none()
        );
    }

    #[test]
    fn native154_contrat_workspace_objet_ferme() {
        let base = serde_json::json!({
            "operation":"delegate", "request_id":"k", "agent_type":"a",
            "model":"m", "task":"t", "cwd":"/c"
        });
        for workspace in [
            serde_json::json!({"strategy":"shared"}),
            serde_json::json!({"strategy":"worktree"}),
            serde_json::json!({"strategy":"worktree","base_ref":"main","branch":"bridget/refonte"}),
        ] {
            let mut wire = base.clone();
            wire["workspace"] = workspace;
            let request: NativeDelegationRequest =
                serde_json::from_value(wire.clone()).expect("stratégie workspace valide");
            assert_eq!(
                serde_json::to_value(request).unwrap()["workspace"],
                wire["workspace"]
            );
        }
        for workspace in [
            serde_json::json!({"strategy":"worktree","unknown":true}),
            serde_json::json!({"strategy":"shared","base_ref":"main"}),
            serde_json::json!({"strategy":"existing_worktree"}),
            serde_json::json!(null),
            serde_json::json!({"strategy":"worktree","base_ref":null}),
            serde_json::json!({"strategy":"worktree","branch":null}),
        ] {
            let mut wire = base.clone();
            wire["workspace"] = workspace;
            assert!(
                serde_json::from_value::<NativeDelegationRequest>(wire.clone()).is_err(),
                "workspace doit être fermé : {}",
                wire["workspace"]
            );
        }
    }

    #[test]
    fn native154_contrat_partage_explicite_enveloppe_distincte() {
        let absent: NativeDelegationRequest = serde_json::from_str(
            r#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/c"}"#,
        ).unwrap();
        let mut shared_wire = serde_json::to_value(&absent).unwrap();
        shared_wire["workspace"] = serde_json::json!({"strategy":"shared"});
        let shared: NativeDelegationRequest =
            serde_json::from_value(shared_wire).expect("partage explicite valide");
        let canonical = serde_json::to_vec(&shared).unwrap();
        assert_ne!(canonical, serde_json::to_vec(&absent).unwrap());
        assert_eq!(
            serde_json::from_slice::<NativeDelegationRequest>(&canonical).unwrap(),
            shared
        );
        assert_eq!(
            serde_json::to_value(shared).unwrap()["workspace"],
            serde_json::json!({"strategy":"shared"})
        );
    }

    #[test]
    fn native153_protocole_continue_aller_retour() {
        for wire in [
            r#"{"operation":"continue","task_id":"child","request_id":"round2","task":"continue","brief":{"objective":"objectif","summary":"résultat"}}"#,
            r#"{"operation":"continue","task_id":"child","request_id":"round2","task":"continue"}"#,
            r#"{"operation":"continue","task_id":"child","request_id":"round2","task":"continue","brief":null}"#,
        ] {
            let request: NativeDelegationRequest = serde_json::from_str(wire).unwrap();
            assert_eq!(serde_json::to_string(&request).unwrap(), wire);
            assert!(matches!(request, NativeDelegationRequest::Continue { .. }));
        }
    }

    #[test]
    fn native153_protocole_brief_serialisation_deterministe() {
        let first: NativeDelegationRequest = serde_json::from_str(
            r#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/c","keep_alive_secs":600,"brief":{"summary":"résultat","objective":"objectif"}}"#,
        )
        .unwrap();
        let second: NativeDelegationRequest = serde_json::from_str(
            r#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/c","keep_alive_secs":600,"brief":{"objective":"objectif","summary":"résultat"}}"#,
        )
        .unwrap();
        assert_eq!(first, second);
        let encoded = serde_json::to_vec(&first).unwrap();
        assert_eq!(encoded, serde_json::to_vec(&second).unwrap());
        let NativeDelegationRequest::Delegate {
            keep_alive_secs,
            brief,
            ..
        } = serde_json::from_slice(&encoded).unwrap()
        else {
            panic!("variante attendue : Delegate");
        };
        assert_eq!(keep_alive_secs, Some(600));
        assert_eq!(
            brief,
            Some(serde_json::json!({"objective":"objectif","summary":"résultat"}))
        );
    }

    #[test]
    fn native151_protocole_attente_aller_retour_et_champs_fermes() {
        let wire = r#"{"operation":"wait","task_ids":["one","two"],"wait_timeout":7}"#;
        let request: NativeDelegationRequest = serde_json::from_str(wire).unwrap();
        assert_eq!(serde_json::to_string(&request).unwrap(), wire);
        assert!(
            serde_json::from_str::<NativeDelegationRequest>(
                r#"{"operation":"wait","task_ids":["one"],"parent":"forged"}"#
            )
            .is_err()
        );
        let missing = r#"{"operation":"wait","task_ids":["one"]}"#;
        let request: NativeDelegationRequest = serde_json::from_str(missing).unwrap();
        assert_eq!(serde_json::to_string(&request).unwrap(), missing);
    }

    #[test]
    fn native151_protocole_demande_et_reponse_aller_retour() {
        let request = r#"{"type":"RequestWait","request_id":"request","wait_timeout":3}"#;
        let decoded: WrapperToDaemon = decode(request).unwrap();
        assert_eq!(encode(&decoded).unwrap(), request);
        let response = r#"{"type":"RequestWaitResult","state":"answered","timed_out_wait":false,"waited_secs":1,"reply":{"summary":"done"},"reply_reason":null}"#;
        let decoded: DaemonToWrapper = decode(response).unwrap();
        assert_eq!(encode(&decoded).unwrap(), response);
    }

    #[test]
    fn native151_protocole_identite_ancienne_liste_vide() {
        let report: DaemonToWrapper = decode(
            r#"{"type":"DaemonIdentityReport","host":"host","db_path":"db","instance_id":"instance"}"#
        ).unwrap();
        let value: serde_json::Value = serde_json::from_str(&encode(&report).unwrap()).unwrap();
        assert_eq!(value["features"], serde_json::json!([]));
    }

    // T009 : réemploi JSON public et serde_json::to_vec ; aucun canon alternatif.
    #[test]
    fn native160_protocole_delegate_sans_resume_on_limit_octets_identiques() {
        let wire = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/c","posture":null,"reply_timeout":120}"#;
        let request: NativeDelegationRequest = serde_json::from_slice(wire).unwrap();
        assert_eq!(serde_json::to_vec(&request).unwrap(), wire);
        assert!(
            serde_json::to_value(request)
                .unwrap()
                .get("resume_on_limit")
                .is_none()
        );
    }

    #[test]
    fn spec163_auto_delegate_et_probe_restent_des_contrats_fermes() {
        let auto = serde_json::json!({
            "request_id":"request-163", "task":"auditer", "cwd":"/project",
            "urgent":true, "deliverable":"rapport"
        });
        let request: AutoDelegateV1 = serde_json::from_value(auto.clone()).unwrap();
        assert_eq!(serde_json::to_value(request).unwrap(), auto);
        let mut native = auto.clone();
        native["operation"] = serde_json::json!("auto_delegate");
        let decoded: NativeDelegationRequest = serde_json::from_value(native.clone()).unwrap();
        assert!(matches!(decoded, NativeDelegationRequest::AutoDelegate(_)));
        assert_eq!(serde_json::to_value(decoded).unwrap(), native);
        let mut forged = auto;
        forged["owner"] = serde_json::json!("forged");
        assert!(serde_json::from_value::<AutoDelegateV1>(forged).is_err());

        let hello = WorkerHelloWithCapabilities {
            hello: WorkerHello {
                version: REMOTE_WORKER_VERSION,
                identity: WorkerIdentity {
                    machine_id: "machine-163".into(),
                    worker_id: "worker-163".into(),
                    provisioning_revision: 1,
                },
                credential: IdentityCredential::new("credential".into()),
                build: "build-163".into(),
                capability: ClientCapability::RemoteWorkerV1,
            },
            capabilities: vec![WorkerCapability::CandidateProbeV1],
        };
        assert_eq!(hello.hello.validate(), Ok(()));
        let encoded = serde_json::to_value(&hello).unwrap();
        assert_eq!(
            serde_json::from_value::<WorkerHelloWithCapabilities>(encoded).unwrap(),
            hello
        );
        let legacy = WorkerHello {
            version: REMOTE_WORKER_VERSION,
            identity: WorkerIdentity {
                machine_id: "machine-163".into(),
                worker_id: "worker-163".into(),
                provisioning_revision: 1,
            },
            credential: IdentityCredential::new("credential".into()),
            build: "build-163".into(),
            capability: ClientCapability::RemoteWorkerV1,
        };
        assert!(
            serde_json::from_value::<WorkerHello>(serde_json::json!({
                "version":legacy.version, "identity":legacy.identity,
                "credential":"credential", "build":legacy.build,
                "capability":"remote_worker_v1", "capabilities":["candidate_probe_v1"]
            }))
            .is_err()
        );
    }

    #[test]
    fn native160_protocole_delegate_resume_on_limit_faux_octets_distincts() {
        let legacy = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/c","posture":null,"reply_timeout":120}"#;
        let explicit = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/c","posture":null,"reply_timeout":120,"resume_on_limit":false}"#;
        let request: NativeDelegationRequest =
            serde_json::from_slice(explicit).expect("resume_on_limit explicite doit être accepté");
        let bytes = serde_json::to_vec(&request).unwrap();
        assert_eq!(bytes, explicit);
        assert_ne!(bytes, legacy);
        assert_eq!(
            serde_json::to_value(request).unwrap()["resume_on_limit"],
            false
        );
    }

    #[test]
    fn native157_protocole_delegate_sans_fallback_garde_octets150() {
        let wire = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/c","posture":null,"reply_timeout":120}"#;
        let request: NativeDelegationRequest = serde_json::from_slice(wire).unwrap();
        assert_eq!(serde_json::to_vec(&request).unwrap(), wire);
    }

    #[test]
    fn native155_protocol_delegate_sans_option_garde_octets154() {
        let wire = r#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/c","posture":null,"reply_timeout":120,"workspace":{"strategy":"shared"},"keep_alive_secs":600,"brief":{"objective":"objectif","summary":"résultat"}}"#.as_bytes();
        let request: NativeDelegationRequest = serde_json::from_slice(wire).unwrap();
        assert_eq!(serde_json::to_vec(&request).unwrap(), wire);
        assert!(
            serde_json::to_value(request)
                .unwrap()
                .get("child_requests")
                .is_none()
        );
    }

    #[test]
    fn native155_protocol_respond_refuse_champ_inconnu() {
        let wire = serde_json::json!({
            "operation":"respond", "task_id":"task", "request_id":"request",
            "decision":"accept", "answer":{"q1":["réponse"]}
        });
        let request: NativeDelegationRequest = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(request).unwrap(), wire);
        let mut unknown = wire;
        unknown["parent"] = serde_json::json!("forged");
        assert!(serde_json::from_value::<NativeDelegationRequest>(unknown).is_err());
    }

    #[test]
    fn native151_protocole_variantes_historiques_octets_stables() {
        for wire in [
            r#"{"operation":"catalogue"}"#,
            r#"{"operation":"status","task_id":"task"}"#,
            r#"{"operation":"cancel","task_id":"task"}"#,
        ] {
            let decoded: NativeDelegationRequest = serde_json::from_str(wire).unwrap();
            assert_eq!(serde_json::to_string(&decoded).unwrap(), wire);
        }
    }

    #[test]
    fn control158_transport_six_variantes_aller_retour() {
        for request in [
            WrapperToDaemon::DelegationOverviewRead {
                version: 1,
                state: "active".into(),
                limit: 50,
            },
            WrapperToDaemon::DelegationHaltSet {
                version: 1,
                command_id: "halt158".into(),
                halted: true,
                reason: Some("essai".into()),
                cancel_active: false,
            },
            WrapperToDaemon::DelegationCancel {
                version: 1,
                task_id: "task158".into(),
                request_id: "cancel158".into(),
            },
        ] {
            let value = serde_json::to_value(&request).unwrap();
            let reread: WrapperToDaemon = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(reread).unwrap(), value);
        }
        for response in [
            DaemonToWrapper::DelegationOverview {
                halted: false,
                since: None,
                by: None,
                reason: None,
                items: vec![serde_json::json!({
                "task_id":"task158","owner":"parent158","child":"child158","state":"working",
                "age_secs":10,"deadline_in_secs":590,"parent_task_id":null })],
                total_active: 1,
            },
            DaemonToWrapper::DelegationHaltState {
                halted: true,
                since: Some(100),
                by: Some("cli".into()),
                reason: Some("essai".into()),
                generation: 1,
                cancelled: 0,
                replayed: false,
            },
            DaemonToWrapper::DelegationControlRejected {
                reason: DelegationControlRefusal::HumanActorRequired,
            },
        ] {
            let value = serde_json::to_value(&response).unwrap();
            let reread: DaemonToWrapper = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(reread).unwrap(), value);
        }
        for (reason, code) in [
            (
                DelegationControlRefusal::UnsupportedVersion,
                "unsupported_version",
            ),
            (DelegationControlRefusal::InvalidRequest, "invalid_request"),
            (
                DelegationControlRefusal::NegotiationRequired,
                "negotiation_required",
            ),
            (
                DelegationControlRefusal::CapabilityNotNegotiated,
                "capability_not_negotiated",
            ),
            (
                DelegationControlRefusal::HumanActorRequired,
                "human_actor_required",
            ),
            (
                DelegationControlRefusal::TaskUnavailable,
                "task_unavailable",
            ),
            (
                DelegationControlRefusal::StoreUnavailable,
                "store_unavailable",
            ),
        ] {
            assert_eq!(
                serde_json::to_value(reason).unwrap(),
                serde_json::json!(code)
            );
            assert_eq!(
                serde_json::from_value::<DelegationControlRefusal>(serde_json::json!(code))
                    .unwrap(),
                reason
            );
        }
        assert!(
            serde_json::from_value::<DelegationControlRefusal>(serde_json::json!("foreign_reason"))
                .is_err()
        );
        // L'oracle canonique existant est répété ici sans modifier Delegate157.
        let historical = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/c","posture":null}"#;
        let request: NativeDelegationRequest = serde_json::from_slice(historical).unwrap();
        assert_eq!(serde_json::to_vec(&request).unwrap(), historical);
    }
