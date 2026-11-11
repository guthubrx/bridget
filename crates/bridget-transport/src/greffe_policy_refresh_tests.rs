use super::*;
use crate::greffe_authorization::{
    GreffeAuthorizationGate, GreffeDepositAuthorization, GreffeMutationAction,
};
use serde_json::json;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::sync::atomic::{AtomicU64, Ordering};

const NOW: i64 = 1_788_200_000;
const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    root: PathBuf,
    policy: PathBuf,
    audit: PathBuf,
    source: MarkerSource,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "bridget-greffe-refresh-{label}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let root = fs::canonicalize(root).unwrap();
        let source = MarkerSource {
            host: "cartae-test".to_string(),
            marker_directory: root.join("agent-pids"),
        };
        let fixture = Self {
            policy: root.join("policy.json"),
            audit: root.join("audit.jsonl"),
            root,
            source,
        };
        fixture.write_policy(7, true);
        fixture
    }

    fn write_policy(&self, generation: u64, annotated: bool) {
        let source = annotated.then(|| self.source.clone());
        let content = json!({
            "version": 1,
            "generation": generation,
            "attestation_key": KEY,
            "principals": [
                {
                    "principal": "agent-vivant",
                    "marker_source": source,
                    "actions": ["delegate", "registre_add"],
                    "instances": [
                        {
                            "instance_id": "instance-ancienne-a",
                            "expires_at": NOW + 600,
                            "revoked": false
                        },
                        {
                            "instance_id": "instance-ancienne-b",
                            "expires_at": NOW + 600,
                            "revoked": false
                        }
                    ]
                },
                {
                    "principal": "agent-arrete",
                    "marker_source": source,
                    "actions": ["objective_close"],
                    "instances": [{
                        "instance_id": "instance-conservee",
                        "expires_at": NOW + 900,
                        "revoked": true
                    }]
                }
            ]
        });
        fs::write(&self.policy, serde_json::to_vec_pretty(&content).unwrap()).unwrap();
        fs::set_permissions(&self.policy, fs::Permissions::from_mode(0o600)).unwrap();
    }

    fn inventory(&self, instance_id: &str) -> MarkerInventory {
        MarkerInventory {
            version: MARKER_INVENTORY_VERSION,
            source: self.source.clone(),
            observed_at: NOW - 1,
            complete: true,
            live: vec![
                LiveMarker {
                    principal: "agent-vivant".to_string(),
                    instance_id: instance_id.to_string(),
                    pid: 42,
                    birth: 420,
                },
                LiveMarker {
                    principal: "agent-inconnu".to_string(),
                    instance_id: "instance-inconnue".to_string(),
                    pid: 43,
                    birth: 430,
                },
            ],
            stale: Vec::new(),
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn regenere_seulement_l_instance_vivante_et_la_garde_accepte_son_effet() {
    let fixture = Fixture::new("nominal");
    let original = fs::read(&fixture.policy).unwrap();
    let inventory = fixture.inventory("instance-nouvelle");

    let preview = refresh_policy(
        &fixture.policy,
        std::slice::from_ref(&inventory),
        NOW,
        false,
    )
    .unwrap();
    assert!(!preview.applied);
    assert_eq!(preview.previous_generation, 7);
    assert_eq!(preview.next_generation, 8);
    assert_eq!(preview.refreshed_principals, ["agent-vivant"]);
    assert_eq!(preview.dead_principals, ["agent-arrete"]);
    assert_eq!(preview.unapproved_principals, ["agent-inconnu"]);
    assert_eq!(fs::read(&fixture.policy).unwrap(), original);

    let applied = refresh_policy(&fixture.policy, &[inventory], NOW, true).unwrap();
    assert!(applied.applied);
    assert_eq!(applied.next_generation, 8);
    assert_eq!(
        fs::metadata(&fixture.policy).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let policy: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
    assert_eq!(policy["generation"], 8);
    assert_eq!(
        policy["principals"][0]["instances"],
        json!([{
            "instance_id": "instance-nouvelle",
            "expires_at": NOW + 600,
            "revoked": false
        }])
    );
    assert_eq!(
        policy["principals"][1]["instances"],
        json!([{
            "instance_id": "instance-conservee",
            "expires_at": NOW + 900,
            "revoked": true
        }])
    );
    assert!(
        !fs::read_to_string(&fixture.policy)
            .unwrap()
            .contains("agent-inconnu")
    );

    let gate = GreffeAuthorizationGate::new(&fixture.policy, &fixture.audit);
    let durable = fixture.root.join("durable-effect");
    fs::write(&durable, b"unchanged").unwrap();
    gate.authorize_deposit_then(
        GreffeDepositAuthorization {
            canonical_name: Some("agent-vivant"),
            canonical_instance_id: Some("instance-nouvelle"),
            declared_from: Some("agent-vivant"),
            action: GreffeMutationAction::Delegate,
            issuer_scope: "038_scope_0123456789abcdef0123456789abcdef",
            request_id: "request-refresh-positive",
            request_issued_at: NOW - 1,
            canonical_request: br#"{"operation":"delegate"}"#,
            observed_at: NOW,
        },
        |_| fs::write(&durable, b"mutated").unwrap(),
    )
    .unwrap();
    assert_eq!(fs::read(&durable).unwrap(), b"mutated");
}

#[test]
fn politique_historique_reste_lisible_par_la_garde_mais_pas_regenerable() {
    let fixture = Fixture::new("historical");
    fixture.write_policy(7, false);
    let gate = GreffeAuthorizationGate::new(&fixture.policy, &fixture.audit);
    let accepted = gate.authorize_deposit(GreffeDepositAuthorization {
        canonical_name: Some("agent-vivant"),
        canonical_instance_id: Some("instance-ancienne-a"),
        declared_from: Some("agent-vivant"),
        action: GreffeMutationAction::Delegate,
        issuer_scope: "038_scope_0123456789abcdef0123456789abcdef",
        request_id: "request-historical",
        request_issued_at: NOW - 1,
        canonical_request: br#"{"operation":"delegate"}"#,
        observed_at: NOW,
    });
    assert!(accepted.is_ok());

    assert_eq!(
        refresh_policy(&fixture.policy, &[], NOW, true).unwrap_err(),
        PolicyRefreshError::MissingMarkerSource {
            principal: "agent-vivant".to_string()
        }
    );
}

#[test]
fn chemin_lie_ne_peut_pas_creer_un_second_verrou_sur_la_meme_politique() {
    let fixture = Fixture::new("linked-policy-path");
    let linked = fixture.root.join("linked-policy.json");
    symlink(&fixture.policy, &linked).unwrap();

    assert_eq!(
        refresh_policy(
            &linked,
            &[fixture.inventory("instance-nouvelle")],
            NOW,
            true
        )
        .unwrap_err(),
        PolicyRefreshError::PolicyPathNotCanonical
    );
}

#[test]
fn politique_a_plusieurs_entrees_est_refusee_avant_le_plan() {
    let fixture = Fixture::new("hard-linked-policy");
    let alias = fixture.root.join("policy-alias.json");
    fs::hard_link(&fixture.policy, &alias).unwrap();
    let original = fs::read(&fixture.policy).unwrap();

    assert_eq!(
        refresh_policy(&fixture.policy, &[], NOW, true).unwrap_err(),
        PolicyRefreshError::PolicyPathNotUnique
    );
    assert_eq!(fs::read(&fixture.policy).unwrap(), original);
    assert_eq!(fs::read(alias).unwrap(), original);
}

#[test]
fn verrou_a_plusieurs_entrees_est_refuse_avant_le_plan() {
    let fixture = Fixture::new("hard-linked-lock");
    let lock = fixture.root.join(".policy.json.refresh.lock");
    let alias = fixture.root.join("lock-alias");
    fs::write(&lock, b"").unwrap();
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o600)).unwrap();
    fs::hard_link(&lock, &alias).unwrap();
    let original = fs::read(&fixture.policy).unwrap();

    assert_eq!(
        refresh_policy(&fixture.policy, &[], NOW, true).unwrap_err(),
        PolicyRefreshError::LockPathNotUnique
    );
    assert_eq!(fs::read(&fixture.policy).unwrap(), original);
}

#[test]
fn exemple_038_adapte_est_regenerable_sans_ouvrir_de_nouveau_droit() {
    let fixture = Fixture::new("documented-refresh-policy");
    let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../specs/038-regeneration-politique/contracts/greffe-authorization-refresh.example.json",
        );
    let mut example: serde_json::Value =
        serde_json::from_slice(&fs::read(example_path).unwrap()).unwrap();
    example["generation"] = json!(7);
    example["attestation_key"] = json!(KEY);
    example["principals"][0]["principal"] = json!("agent-vivant");
    example["principals"][0]["marker_source"] = json!(fixture.source);
    example["principals"][0]["instances"][0]["instance_id"] = json!("instance-ancienne");
    example["principals"][0]["instances"][0]["expires_at"] = json!(NOW + 600);
    fs::write(
        &fixture.policy,
        serde_json::to_vec_pretty(&example).unwrap(),
    )
    .unwrap();
    fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();

    let report = refresh_policy(
        &fixture.policy,
        &[fixture.inventory("instance-nouvelle")],
        NOW,
        true,
    )
    .unwrap();

    assert_eq!(report.refreshed_principals, ["agent-vivant"]);
    assert_eq!(report.unapproved_principals, ["agent-inconnu"]);
    let policy = fs::read_to_string(&fixture.policy).unwrap();
    assert!(policy.contains("instance-nouvelle"));
    assert!(!policy.contains("instance-inconnue"));
}

#[test]
fn inventaire_incomplet_vide_ou_absent_preserve_exactement_la_politique() {
    let fixture = Fixture::new("closed-inputs");
    let original = fs::read(&fixture.policy).unwrap();
    let mut incomplete = fixture.inventory("instance-nouvelle");
    incomplete.complete = false;
    assert_eq!(
        refresh_policy(&fixture.policy, &[incomplete], NOW, true).unwrap_err(),
        PolicyRefreshError::IncompleteInventory {
            source: fixture.source.clone()
        }
    );
    assert_eq!(fs::read(&fixture.policy).unwrap(), original);

    let mut empty = fixture.inventory("instance-nouvelle");
    empty.live.clear();
    assert_eq!(
        refresh_policy(&fixture.policy, &[empty], NOW, true).unwrap_err(),
        PolicyRefreshError::EmptyInventory {
            source: fixture.source.clone()
        }
    );
    assert_eq!(fs::read(&fixture.policy).unwrap(), original);

    assert_eq!(
        refresh_policy(&fixture.policy, &[], NOW, true).unwrap_err(),
        PolicyRefreshError::MissingInventory {
            source: fixture.source.clone()
        }
    );
    assert_eq!(fs::read(&fixture.policy).unwrap(), original);
}

#[test]
fn ensemble_des_sources_est_exact_sans_oubli_ajout_ni_doublon() {
    let fixture = Fixture::new("exact-sources");
    let source_distante = MarkerSource {
        host: "hote-distant".to_string(),
        marker_directory: fixture.root.join("remote-agent-pids"),
    };
    let mut policy: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
    policy["principals"][1]["marker_source"] = json!(source_distante);
    fs::write(&fixture.policy, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
    fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();
    let local = fixture.inventory("instance-nouvelle");
    let remote = MarkerInventory {
        version: MARKER_INVENTORY_VERSION,
        source: source_distante.clone(),
        observed_at: NOW - 1,
        complete: true,
        live: vec![LiveMarker {
            principal: "agent-inconnu-distant".to_string(),
            instance_id: "instance-inconnue-distante".to_string(),
            pid: 44,
            birth: 440,
        }],
        stale: Vec::new(),
    };

    assert_eq!(
        refresh_policy(&fixture.policy, std::slice::from_ref(&local), NOW, false).unwrap_err(),
        PolicyRefreshError::MissingInventory {
            source: source_distante.clone()
        }
    );

    let unexpected_source = MarkerSource {
        host: "hote-inattendu".to_string(),
        marker_directory: fixture.root.join("unexpected-agent-pids"),
    };
    let mut unexpected = remote.clone();
    unexpected.source = unexpected_source.clone();
    assert_eq!(
        refresh_policy(
            &fixture.policy,
            &[local.clone(), remote.clone(), unexpected],
            NOW,
            false
        )
        .unwrap_err(),
        PolicyRefreshError::UnexpectedInventory {
            source: unexpected_source
        }
    );

    assert_eq!(
        refresh_policy(&fixture.policy, &[local.clone(), local, remote], NOW, false).unwrap_err(),
        PolicyRefreshError::DuplicateInventory {
            source: fixture.source.clone()
        }
    );
}

#[test]
fn principal_sur_mauvaise_source_ou_duplique_est_refuse() {
    let fixture = Fixture::new("wrong-or-duplicate-principal");
    let source_distante = MarkerSource {
        host: "hote-distant".to_string(),
        marker_directory: fixture.root.join("remote-agent-pids"),
    };
    let mut policy: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
    policy["principals"][1]["marker_source"] = json!(source_distante);
    fs::write(&fixture.policy, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
    fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();
    let mut local = fixture.inventory("instance-nouvelle");
    local.live.remove(0);
    let remote = MarkerInventory {
        version: MARKER_INVENTORY_VERSION,
        source: source_distante.clone(),
        observed_at: NOW - 1,
        complete: true,
        live: vec![LiveMarker {
            principal: "agent-vivant".to_string(),
            instance_id: "instance-nouvelle".to_string(),
            pid: 44,
            birth: 440,
        }],
        stale: Vec::new(),
    };
    assert_eq!(
        refresh_policy(&fixture.policy, &[local, remote], NOW, false).unwrap_err(),
        PolicyRefreshError::PrincipalOnWrongSource {
            principal: "agent-vivant".to_string(),
            expected: fixture.source.clone(),
            observed: source_distante
        }
    );

    let duplicate_fixture = Fixture::new("duplicate-principal");
    let mut duplicate = duplicate_fixture.inventory("instance-nouvelle");
    duplicate.live.push(LiveMarker {
        principal: "agent-vivant".to_string(),
        instance_id: "instance-concurrente".to_string(),
        pid: 45,
        birth: 450,
    });
    assert_eq!(
        refresh_policy(&duplicate_fixture.policy, &[duplicate], NOW, false).unwrap_err(),
        PolicyRefreshError::AmbiguousPrincipal {
            principal: "agent-vivant".to_string()
        }
    );
}

#[test]
fn inventaire_perime_ou_futur_est_refuse_avant_ecriture() {
    let fixture = Fixture::new("inventory-time");
    let original = fs::read(&fixture.policy).unwrap();
    let mut old = fixture.inventory("instance-nouvelle");
    old.observed_at = NOW - MAX_MARKER_INVENTORY_AGE_SECONDS - 1;
    assert_eq!(
        refresh_policy(&fixture.policy, &[old], NOW, true).unwrap_err(),
        PolicyRefreshError::InventoryTooOld {
            source: fixture.source.clone()
        }
    );
    assert_eq!(fs::read(&fixture.policy).unwrap(), original);

    let mut future = fixture.inventory("instance-nouvelle");
    future.observed_at = NOW + 1;
    assert_eq!(
        refresh_policy(&fixture.policy, &[future], NOW, true).unwrap_err(),
        PolicyRefreshError::InventoryFromFuture {
            source: fixture.source.clone()
        }
    );
    assert_eq!(fs::read(&fixture.policy).unwrap(), original);
}

#[test]
fn droits_divergents_et_generation_epuisee_ne_sont_jamais_arbitres() {
    let fixture = Fixture::new("divergent-grant");
    let mut policy: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
    policy["principals"][0]["instances"][1]["expires_at"] = json!(NOW + 601);
    fs::write(&fixture.policy, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
    fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        refresh_policy(
            &fixture.policy,
            &[fixture.inventory("instance-nouvelle")],
            NOW,
            false
        )
        .unwrap_err(),
        PolicyRefreshError::DivergentGrant {
            principal: "agent-vivant".to_string()
        }
    );

    fixture.write_policy(u64::MAX, true);
    assert_eq!(
        refresh_policy(
            &fixture.policy,
            &[fixture.inventory("instance-nouvelle")],
            NOW,
            true
        )
        .unwrap_err(),
        PolicyRefreshError::GenerationExhausted
    );
}

#[test]
fn generation_croit_et_remplacement_reste_atomique() {
    let fixture = Fixture::new("atomic-generation");
    let original = fs::read(&fixture.policy).unwrap();
    let inventory = fixture.inventory("instance-nouvelle");
    let mut observed = Vec::new();
    let report = refresh_policy_observed(&fixture.policy, &[inventory], NOW, |phase| {
        observed.push(phase);
        if phase == AtomicWritePhase::BeforeRename {
            assert_eq!(
                fs::read(&fixture.policy).unwrap(),
                original,
                "avant le renommage, seul le fichier original est visible"
            );
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(report.previous_generation, 7);
    assert_eq!(report.next_generation, 8);
    assert_eq!(
        observed,
        [
            AtomicWritePhase::BeforeRename,
            AtomicWritePhase::AfterRename
        ],
        "le chemin de production doit traverser les deux frontières atomiques"
    );
    let rewritten: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
    assert_eq!(rewritten["generation"], 8);
}

#[test]
fn erreur_avant_renommage_garde_l_original_et_apres_rend_l_issue_indeterminee() {
    let before = Fixture::new("failure-before-rename");
    let original = fs::read(&before.policy).unwrap();
    let before_error = refresh_policy_observed(
        &before.policy,
        &[before.inventory("instance-nouvelle")],
        NOW,
        |phase| {
            if phase == AtomicWritePhase::BeforeRename {
                return Err(io::Error::other("échec injecté avant renommage"));
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(
        before_error,
        PolicyRefreshError::Io("échec injecté avant renommage".to_string())
    );
    assert_eq!(fs::read(&before.policy).unwrap(), original);

    let after = Fixture::new("failure-after-rename");
    let after_error = refresh_policy_observed(
        &after.policy,
        &[after.inventory("instance-nouvelle")],
        NOW,
        |phase| {
            if phase == AtomicWritePhase::AfterRename {
                return Err(io::Error::other("échec injecté après renommage"));
            }
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(
        after_error,
        PolicyRefreshError::WriteOutcomeIndeterminate {
            observed: PostRenamePolicyState::PlannedPolicyPresent,
            message: "échec injecté après renommage".to_string(),
        }
    );
    let rewritten: serde_json::Value =
        serde_json::from_slice(&fs::read(&after.policy).unwrap()).unwrap();
    assert_eq!(rewritten["generation"], 8);
}

#[test]
fn instance_deja_alignee_ne_reecrit_ni_ne_rafraichit_la_generation() {
    let fixture = Fixture::new("already-current");
    refresh_policy(
        &fixture.policy,
        &[fixture.inventory("instance-nouvelle")],
        NOW,
        true,
    )
    .unwrap();
    let current = fs::read(&fixture.policy).unwrap();

    let report = refresh_policy_observed(
        &fixture.policy,
        &[fixture.inventory("instance-nouvelle")],
        NOW,
        |_| panic!("aucune écriture ne doit avoir lieu sans changement"),
    )
    .unwrap();
    assert!(!report.applied);
    assert_eq!(report.previous_generation, 8);
    assert_eq!(report.next_generation, 8);
    assert_eq!(report.unchanged_principals, ["agent-vivant"]);
    assert_eq!(fs::read(&fixture.policy).unwrap(), current);
}
