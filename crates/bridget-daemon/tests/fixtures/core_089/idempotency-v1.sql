-- Schéma historique v1 matérialisé depuis le harnais épinglé
-- dfa2134dcfe2a2522e3ae77d93561e6ae72556b3,
-- crates/bridget-daemon/src/idempotency.rs:4328,
-- migration_v1_classe_les_remises_sans_payload_sans_bloquer_les_valides.
-- Données ajoutées par la recette, aucun export de données utilisateur.
CREATE TABLE idempotency_records (
    issuer_scope TEXT NOT NULL,
    operation_kind TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    canonical_bytes BLOB NOT NULL,
    state TEXT NOT NULL,
    public_result_kind TEXT,
    public_result_category TEXT,
    public_result_reason TEXT,
    issued_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    PRIMARY KEY (issuer_scope, operation_kind, idempotency_key)
);
CREATE TABLE send_deliveries (
    delivery_id TEXT PRIMARY KEY,
    issuer_scope TEXT NOT NULL,
    operation_kind TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    recipient_instance_id TEXT NOT NULL,
    delivery_generation INTEGER NOT NULL,
    phase TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    message_bytes BLOB
);
