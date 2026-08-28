//! Domaine et interfaces publiques du compagnon Maicie.
//!
//! Ce crate ne dépend d'aucun module interne de Bridget : la future frontière
//! réseau restera limitée à son client public dédié.

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
