    use super::*;
    use serde_json::json;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::atomic::{AtomicU64, Ordering};

    const NOW: i64 = 1_788_000_000;
    const ISSUER_SCOPE: &str = "026_scope_0123456789abcdef0123456789abcdef";
    const CANONICAL_REQUEST: &[u8] = br#"{"operation":"delegate","goal":"A"}"#;
    const CANONICAL_REQUEST_SHA256: &str =
        "beff6e8b9bca6da6bbaa448dc70686181c56f485c44544f61dca46fe690d5d59";
    const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    struct Fixture {
        root: PathBuf,
        policy: PathBuf,
        audit: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "bridget-greffe-authorization-{label}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            let fixture = Self {
                policy: root.join("policy.json"),
                audit: root.join("audit.jsonl"),
                root,
            };
            fixture.write_policy(false, NOW + 300);
            fixture
        }

        fn write_policy(&self, revoked: bool, expires_at: i64) {
            let content = json!({
                "version": 1,
                "generation": 7,
                "attestation_key": KEY,
                "principals": [
                    {
                        "principal": "agent-autorise",
                        "actions": ["delegate", "registre_add", "objective_close"],
                        "instances": [{
                            "instance_id": "instance-autorisee",
                            "expires_at": expires_at,
                            "revoked": revoked
                        }]
                    },
                    {
                        "principal": "cli-send-123",
                        "actions": ["delegate", "registre_add", "objective_close"],
                        "instances": [{
                            "instance_id": "instance-autorisee",
                            "expires_at": expires_at,
                            "revoked": revoked
                        }]
                    }
                ]
            });
            fs::write(&self.policy, serde_json::to_vec(&content).unwrap()).unwrap();
            fs::set_permissions(&self.policy, fs::Permissions::from_mode(0o600)).unwrap();
        }

        fn gate(&self) -> GreffeAuthorizationGate {
            GreffeAuthorizationGate::new(&self.policy, &self.audit)
        }

        fn deposit(
            &self,
            name: Option<&str>,
            instance_id: Option<&str>,
            declared_from: Option<&str>,
            action: GreffeMutationAction,
        ) -> Result<GreffeAuthorizationAttestation, GreffeAuthorizationRefusal> {
            self.gate().authorize_deposit(GreffeDepositAuthorization {
                canonical_name: name,
                canonical_instance_id: instance_id,
                declared_from,
                action,
                issuer_scope: ISSUER_SCOPE,
                request_id: "request-1",
                request_issued_at: NOW - 1,
                canonical_request: CANONICAL_REQUEST,
                observed_at: NOW,
            })
        }

        fn effect(
            &self,
            attestation: Option<&GreffeAuthorizationAttestation>,
            action: GreffeMutationAction,
        ) -> Result<GreffePrincipal, GreffeAuthorizationRefusal> {
            self.gate().authorize_effect(GreffeEffectAuthorization {
                attestation,
                action,
                issuer_scope: ISSUER_SCOPE,
                request_id: "request-1",
                request_issued_at: NOW - 1,
                canonical_request: CANONICAL_REQUEST,
                observed_at: NOW,
            })
        }

        fn audit_lines(&self) -> Vec<serde_json::Value> {
            fs::read_to_string(&self.audit)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect()
        }

        fn assert_deposit_refused_before_mutation(
            &self,
            name: Option<&str>,
            instance_id: Option<&str>,
            declared_from: Option<&str>,
            expected: GreffeAuthorizationRefusal,
        ) {
            // Cet oracle prouve le traitement du principal fourni à la garde,
            // pas l'authenticité du `Register` qui alimente les cartes daemon.
            let durable_state = self.root.join("durable-business-state");
            fs::write(&durable_state, b"unchanged").unwrap();
            let result = self.gate().authorize_deposit_then(
                GreffeDepositAuthorization {
                    canonical_name: name,
                    canonical_instance_id: instance_id,
                    declared_from,
                    action: GreffeMutationAction::Delegate,
                    issuer_scope: ISSUER_SCOPE,
                    request_id: "request-1",
                    request_issued_at: NOW - 1,
                    canonical_request: CANONICAL_REQUEST,
                    observed_at: NOW,
                },
                |_| fs::write(&durable_state, b"mutated").unwrap(),
            );

            assert_eq!(
                fs::read_to_string(&durable_state).unwrap(),
                "unchanged",
                "la mutation durable doit rester en aval de la garde"
            );
            assert_eq!(result.unwrap_err(), expected);
            assert_eq!(
                expected.public_reason(),
                GREFFE_AUTHORIZATION_PUBLIC_REFUSAL
            );
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn hmac_sha256_est_confronte_a_un_vecteur_rfc4231_litteral() {
        let key = [0x0b_u8; 20];
        assert_eq!(
            hex(&hmac_sha256(&key, b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn agent_autorise_est_atteste_au_depot_et_reverifie_avant_effet() {
        let fixture = Fixture::new("nominal");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::Delegate,
            )
            .unwrap();
        let principal = fixture
            .effect(Some(&attestation), GreffeMutationAction::Delegate)
            .unwrap();

        assert_eq!(principal.name, "agent-autorise");
        assert_eq!(principal.instance_id, "instance-autorisee");
        assert_eq!(attestation.request_id, "request-1");
        assert_eq!(
            attestation.canonical_request_sha256,
            CANONICAL_REQUEST_SHA256
        );
        assert_eq!(attestation.signature.len(), 64);
        let audit = fixture.audit_lines();
        assert_eq!(audit.len(), 2);
        assert_eq!(audit[0]["stage"], "deposit");
        assert_eq!(audit[1]["stage"], "effect");
        assert_eq!(audit[0]["principal"]["name"], "agent-autorise");
        assert_eq!(audit[0]["action"], "delegate");
        assert_eq!(audit[0]["request_id"], "request-1");
        assert_eq!(audit[0]["allowed"], true);
    }

    #[test]
    fn exemple_de_politique_adapte_autorise_exactement_les_trois_mutations() {
        let fixture = Fixture::new("documented-policy");
        let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../specs/026-operations-greffe-central/contracts/greffe-authorization.example.json",
        );
        let mut example: serde_json::Value =
            serde_json::from_slice(&fs::read(example_path).unwrap()).unwrap();
        example["attestation_key"] = json!(KEY);
        example["principals"][0]["principal"] = json!("agent-autorise");
        example["principals"][0]["instances"][0]["instance_id"] = json!("instance-autorisee");
        example["principals"][0]["instances"][0]["expires_at"] = json!(NOW + 300);
        fs::write(&fixture.policy, serde_json::to_vec(&example).unwrap()).unwrap();
        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();

        for action in GreffeMutationAction::ALL {
            let attestation = fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    action,
                )
                .unwrap();
            assert_eq!(attestation.action, action);
        }
        assert_eq!(fixture.audit_lines().len(), GreffeMutationAction::ALL.len());
    }

    #[test]
    fn nom_canonique_absent_est_refuse_avant_mutation_durable() {
        Fixture::new("missing-name").assert_deposit_refused_before_mutation(
            None,
            Some("instance-autorisee"),
            Some("agent-autorise"),
            GreffeAuthorizationRefusal::PrincipalNameMissing,
        );
    }

    #[test]
    fn instance_canonique_absente_est_refusee_avant_mutation_durable() {
        Fixture::new("missing-instance").assert_deposit_refused_before_mutation(
            Some("agent-autorise"),
            None,
            Some("agent-autorise"),
            GreffeAuthorizationRefusal::PrincipalInstanceMissing,
        );
    }

    #[test]
    fn from_forge_est_refuse_avant_mutation_durable() {
        Fixture::new("forged-from").assert_deposit_refused_before_mutation(
            Some("agent-autorise"),
            Some("instance-autorisee"),
            Some("agent-forge"),
            GreffeAuthorizationRefusal::DeclaredPrincipalMismatch,
        );
    }

    #[test]
    fn connexion_cli_send_est_refusee_avant_mutation_durable() {
        Fixture::new("cli-send").assert_deposit_refused_before_mutation(
            Some("cli-send-123"),
            Some("instance-autorisee"),
            Some("cli-send-123"),
            GreffeAuthorizationRefusal::EphemeralCliForbidden,
        );
    }

    #[test]
    fn instance_remplacee_est_refusee_avant_mutation_durable() {
        Fixture::new("replaced-instance").assert_deposit_refused_before_mutation(
            Some("agent-autorise"),
            Some("instance-remplacee"),
            Some("agent-autorise"),
            GreffeAuthorizationRefusal::InstanceUnknown,
        );
    }

    #[test]
    fn defaut_deny_couvre_principal_et_action_non_declares() {
        let fixture = Fixture::new("default-deny");
        let unknown = fixture
            .deposit(
                Some("autre-agent"),
                Some("autre-instance"),
                Some("autre-agent"),
                GreffeMutationAction::Delegate,
            )
            .unwrap_err();
        assert_eq!(unknown, GreffeAuthorizationRefusal::PrincipalUnknown);

        let mut policy: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
        policy["principals"][0]["actions"] = json!(["delegate"]);
        fs::write(&fixture.policy, serde_json::to_vec(&policy).unwrap()).unwrap();
        let denied = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::ObjectiveClose,
            )
            .unwrap_err();
        assert_eq!(denied, GreffeAuthorizationRefusal::ActionDenied);
    }

    #[test]
    fn expiration_et_revocation_serveur_sont_reverifiees_avant_effet() {
        let fixture = Fixture::new("server-state");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::RegistreAdd,
            )
            .unwrap();

        fixture.write_policy(true, NOW + 300);
        assert_eq!(
            fixture
                .effect(Some(&attestation), GreffeMutationAction::RegistreAdd)
                .unwrap_err(),
            GreffeAuthorizationRefusal::GrantRevoked
        );
        fixture.write_policy(false, NOW);
        assert_eq!(
            fixture
                .effect(Some(&attestation), GreffeMutationAction::RegistreAdd)
                .unwrap_err(),
            GreffeAuthorizationRefusal::GrantExpired
        );
    }

    #[test]
    fn attestation_absente_fausse_ou_rejouee_sur_une_autre_action_est_refusee() {
        let fixture = Fixture::new("attestation-negatives");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::Delegate,
            )
            .unwrap();
        assert_eq!(
            fixture
                .effect(None, GreffeMutationAction::Delegate)
                .unwrap_err(),
            GreffeAuthorizationRefusal::AttestationMissing
        );

        let mut forged = attestation.clone();
        forged.signature = "signature-disjointe".to_string();
        assert_eq!(
            fixture
                .effect(Some(&forged), GreffeMutationAction::Delegate)
                .unwrap_err(),
            GreffeAuthorizationRefusal::AttestationInvalid
        );
        assert_eq!(
            fixture
                .effect(Some(&attestation), GreffeMutationAction::ObjectiveClose)
                .unwrap_err(),
            GreffeAuthorizationRefusal::AttestationMismatch
        );
    }

    #[test]
    fn issuer_scope_copie_est_refuse_avant_mutation_durable() {
        let fixture = Fixture::new("copied-issuer-scope");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::Delegate,
            )
            .unwrap();
        let durable_state = fixture.root.join("durable-effect-state");
        fs::write(&durable_state, b"unchanged").unwrap();

        let result = fixture.gate().authorize_effect_then(
            GreffeEffectAuthorization {
                attestation: Some(&attestation),
                action: GreffeMutationAction::Delegate,
                issuer_scope: "026_scope_ffffffffffffffffffffffffffffffff",
                request_id: "request-1",
                request_issued_at: NOW - 1,
                canonical_request: CANONICAL_REQUEST,
                observed_at: NOW,
            },
            |_| fs::write(&durable_state, b"mutated").unwrap(),
        );

        assert_eq!(
            fs::read_to_string(&durable_state).unwrap(),
            "unchanged",
            "une attestation ne s'étend jamais à une seconde portée idempotente"
        );
        assert_eq!(
            result.unwrap_err(),
            GreffeAuthorizationRefusal::AttestationMismatch
        );
    }

    #[test]
    fn octet_canonique_substitue_est_refuse_avant_mutation_durable() {
        let fixture = Fixture::new("canonical-request-substitution");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::Delegate,
            )
            .unwrap();
        let mut substituted = CANONICAL_REQUEST.to_vec();
        let changed = substituted
            .iter()
            .position(|byte| *byte == b'A')
            .expect("le témoin contient l'octet discriminant A");
        substituted[changed] = b'B';
        assert_eq!(
            CANONICAL_REQUEST
                .iter()
                .zip(&substituted)
                .filter(|(left, right)| left != right)
                .count(),
            1,
            "le témoin substitue exactement un octet canonique"
        );
        let durable_state = fixture.root.join("canonical-request-effect-state");
        fs::write(&durable_state, b"unchanged").unwrap();

        let result = fixture.gate().authorize_effect_then(
            GreffeEffectAuthorization {
                attestation: Some(&attestation),
                action: GreffeMutationAction::Delegate,
                issuer_scope: ISSUER_SCOPE,
                request_id: "request-1",
                request_issued_at: NOW - 1,
                canonical_request: &substituted,
                observed_at: NOW,
            },
            |_| fs::write(&durable_state, b"mutated").unwrap(),
        );

        assert_eq!(
            fs::read_to_string(&durable_state).unwrap(),
            "unchanged",
            "la substitution des octets relus doit être refusée avant l'effet"
        );
        assert_eq!(
            result.unwrap_err(),
            GreffeAuthorizationRefusal::AttestationMismatch
        );
    }

    #[test]
    fn politique_absente_invalide_ou_non_privee_refuse_et_journalise() {
        let fixture = Fixture::new("policy-negatives");
        fs::remove_file(&fixture.policy).unwrap();
        assert_eq!(
            fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    GreffeMutationAction::Delegate,
                )
                .unwrap_err(),
            GreffeAuthorizationRefusal::PolicyUnavailable
        );

        fixture.write_policy(false, NOW + 300);
        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    GreffeMutationAction::Delegate,
                )
                .unwrap_err(),
            GreffeAuthorizationRefusal::PolicyInvalid
        );
        let audit = fixture.audit_lines();
        assert_eq!(audit.len(), 2);
        assert_eq!(audit[0]["allowed"], false);
        assert_eq!(audit[0]["reason"], "policy_unavailable");
        assert_eq!(audit[1]["allowed"], false);
        assert_eq!(audit[1]["reason"], "policy_invalid");
    }

    #[test]
    fn politique_et_audit_refusent_les_liens_symboliques() {
        let fixture = Fixture::new("nofollow");
        let real_policy = fixture.root.join("real-policy.json");
        fs::rename(&fixture.policy, &real_policy).unwrap();
        symlink(&real_policy, &fixture.policy).unwrap();
        assert_eq!(
            fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    GreffeMutationAction::Delegate,
                )
                .unwrap_err(),
            GreffeAuthorizationRefusal::PolicyUnavailable
        );

        fs::remove_file(&fixture.policy).unwrap();
        fs::rename(&real_policy, &fixture.policy).unwrap();
        let target = fixture.root.join("real-audit.jsonl");
        fs::write(&target, b"").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
        fs::remove_file(&fixture.audit).unwrap();
        symlink(&target, &fixture.audit).unwrap();
        assert_eq!(
            fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    GreffeMutationAction::Delegate,
                )
                .unwrap_err(),
            GreffeAuthorizationRefusal::AuditUnavailable
        );
        assert!(fs::read(&target).unwrap().is_empty());
    }
