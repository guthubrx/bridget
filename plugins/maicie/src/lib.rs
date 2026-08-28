//! Domaine et interfaces publiques du compagnon Maicie.
//!
//! Ce crate ne dépend d'aucun module interne de Bridget : la future frontière
//! réseau restera limitée à son client public dédié.

/// Portées d'émetteur employées par Maicie face au daemon.
///
/// Le daemon impose une longueur minimale (`MIN_ISSUER_SCOPE_LEN`) depuis le
/// socle d'idempotence. Les portées littérales précédentes étaient toutes plus
/// courtes : chaque ClientHello repartait en `invalid_issuer_scope`, refus qui
/// remontait déguisé en « annuaire indisponible ». Elles sont nommées ici pour
/// qu'un témoin unique garde la règle.
pub const LOCALITY_GUARD_ISSUER_SCOPE: &str = "maicie-locality-guard-cli";
pub const STATUS_ISSUER_SCOPE: &str = "maicie-status-reporter";
pub const ROUTINES_ISSUER_SCOPE: &str = "maicie-routines-scheduler";
pub const USAGE_ISSUER_SCOPE: &str = "maicie-usage-collector";
/// Reprise de la règle du daemon, qui vit dans un autre crate.
pub const MIN_ISSUER_SCOPE_LEN: usize = 22;

pub mod app;
pub mod bridget_client;
pub mod catalogue;
pub mod citation;
pub mod config;
pub mod domain;
pub mod greffe_service;
pub mod install_publish;
pub mod outbox;
pub mod preuve;
pub mod profiles;
pub mod reconcile;
pub mod review;
pub mod review_continuity;
pub mod review_git;
pub mod routines;
pub mod runtime;
pub mod store;
pub mod telemetry;
pub mod ui_projection;

/// Identité stable réservée au compagnon d'orchestration.
pub const MAICIE_IDENTITY: &str = "maicie";
