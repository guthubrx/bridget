use maicie::app::{
    LocalProfileApproval, ProfileActivationError, ProfileActivationProposalRequest,
    approve_profile_activation, propose_profile_activation,
};
use maicie::domain::EtatActivationOutbox;
use maicie::store::MaicieStore;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use uuid::Uuid;

const DEFINITION_DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const CHANGED_DEFINITION_DIGEST: &str =
    "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

#[test]
fn proposition_et_approbation_locale_figent_le_contexte_et_l_outbox() {
    let fixture = Fixture::new();
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let request = proposal_request(Uuid::new_v4());

    let proposal = propose_profile_activation(&mut store, &request).unwrap();
    assert_eq!(proposal.decision.objectif_id, request.objective_id);
    assert_eq!(proposal.decision.proposee_par, "maicie");
    assert_eq!(proposal.approval.profile_id, "claude-review");
    assert_eq!(proposal.approval.profile_hash, vec![7; 32]);
    assert_eq!(proposal.approval.context_hash, hex_bytes(DEFINITION_DIGEST));
    assert_eq!(proposal.approval.context_scope, "objectif:minimal");
    assert_eq!(proposal.approval.actor, "local_human");
    assert_eq!(proposal.approval.expires_at, 200);
    assert_eq!(
        proposal.approval.parameters.as_bytes(),
        proposal.spawn_order_bytes.as_slice()
    );
    assert!(
        String::from_utf8_lossy(&proposal.spawn_order_bytes)
            .contains(&proposal.approval.command_id.to_string())
    );
    assert!(
        String::from_utf8_lossy(&proposal.spawn_order_bytes).contains("\"agent_type\":\"claude\"")
    );
    assert!(store.pending_activation_outboxes().unwrap().is_empty());

    let stale_definition = LocalProfileApproval {
        approval_id: proposal.approval.id,
        now: 10,
        profile_hash: &[7; 32],
        resolved_definition_digest: CHANGED_DEFINITION_DIGEST,
    };
    assert!(matches!(
        approve_profile_activation(&mut store, &proposal, &stale_definition),
        Err(ProfileActivationError::Store(_))
    ));
    assert!(store.pending_activation_outboxes().unwrap().is_empty());

    let activation = approve_profile_activation(
        &mut store,
        &proposal,
        &LocalProfileApproval {
            approval_id: proposal.approval.id,
            now: 10,
            profile_hash: &[7; 32],
            resolved_definition_digest: DEFINITION_DIGEST,
        },
    )
    .unwrap();
    assert_eq!(activation.command_id, proposal.approval.command_id);
    assert_eq!(activation.spawn_order_bytes, proposal.spawn_order_bytes);
    assert_eq!(activation.etat, EtatActivationOutbox::Dispatching);

    let replay = approve_profile_activation(
        &mut store,
        &proposal,
        &LocalProfileApproval {
            approval_id: proposal.approval.id,
            now: 11,
            profile_hash: &[7; 32],
            resolved_definition_digest: DEFINITION_DIGEST,
        },
    )
    .unwrap();
    assert_eq!(replay.command_id, activation.command_id);
    assert_eq!(replay.spawn_order_bytes, activation.spawn_order_bytes);
    assert_eq!(store.pending_activation_outboxes().unwrap().len(), 1);
}

#[test]
fn une_approbation_expiree_ne_cree_aucune_outbox() {
    let fixture = Fixture::new();
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let proposal =
        propose_profile_activation(&mut store, &proposal_request(Uuid::new_v4())).unwrap();

    let result = approve_profile_activation(
        &mut store,
        &proposal,
        &LocalProfileApproval {
            approval_id: proposal.approval.id,
            now: proposal.approval.expires_at,
            profile_hash: &[7; 32],
            resolved_definition_digest: DEFINITION_DIGEST,
        },
    );
    assert!(matches!(result, Err(ProfileActivationError::Store(_))));
    assert!(store.pending_activation_outboxes().unwrap().is_empty());
}

#[test]
fn le_digest_de_definition_doit_etre_un_sha256_hexadecimal() {
    let fixture = Fixture::new();
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let mut request = proposal_request(Uuid::new_v4());
    request.resolved_definition_digest = "not-a-definition-digest";

    assert_eq!(
        propose_profile_activation(&mut store, &request),
        Err(ProfileActivationError::Invalid(
            "digest de définition invalide"
        ))
    );
    assert!(store.pending_activation_outboxes().unwrap().is_empty());
}

fn proposal_request(objective_id: Uuid) -> ProfileActivationProposalRequest<'static> {
    ProfileActivationProposalRequest {
        objective_id,
        profile_id: "claude-review",
        agent_type: "claude",
        profile_hash: &[7; 32],
        resolved_definition_digest: DEFINITION_DIGEST,
        context_scope: "objectif:minimal",
        cwd: "/tmp/maicie-profile-approval",
        persistent: true,
        now: 10,
        spawn_deadline_at: 100,
        approval_expires_at: 200,
        retry_until: 80,
        dedup_retained_until: 300,
        reason: "profil absent compatible avec l'objectif",
    }
}

fn hex_bytes(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("maicie-profile-approval-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            database: root.join("maicie.sqlite3"),
            root,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
