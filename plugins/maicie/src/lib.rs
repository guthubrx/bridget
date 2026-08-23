//! Domaine et interfaces publiques du compagnon Maicie.
//!
//! Ce crate ne dépend d'aucun module interne de Bridget : la future frontière
//! réseau restera limitée à son client public dédié.

pub mod app;
pub mod bridget_client;
pub mod config;
pub mod domain;
pub mod outbox;
pub mod reconcile;
pub mod runtime;
pub mod store;
pub mod telemetry;

/// Identité stable réservée au compagnon d'orchestration.
pub const MAICIE_IDENTITY: &str = "maicie";
