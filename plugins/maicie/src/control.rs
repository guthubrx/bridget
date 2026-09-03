//! État de contrôle du référent vu par Maicie (SPEC-087).
//!
//! Maicie ne possède pas cet état : elle le lit une fois par relève auprès du
//! daemon et le garde en mémoire le temps de la commande. Tout puits d'effet
//! autonome de Maicie passe par [`admit_autonomous_effect`] ; un puits qui ne
//! l'appelle pas est un défaut, pas une limite acceptée (ADR 027).

use crate::bridget_client::{
    BridgetClientError, BridgetClientLimits, ControlStateClient, ControlStateReading,
};
use std::path::Path;
use std::time::Instant;

/// Instantané de l'état de contrôle pour une relève.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSnapshot {
    /// Aucune relève n'a encore lu l'état : chemin des commandes locales et
    /// des tests qui n'ouvrent pas la socket. Le journal le dit ; la
    /// production pose toujours un snapshot lu avant tout puits.
    Unread,
    /// Le daemon était injoignable à la relève : on ne sait pas si le référent
    /// a demandé la pause, donc on ne produit rien de nouveau.
    Unknown,
    Read {
        paused: bool,
        auto_objectives_cap: u32,
        inbox_open_count: u32,
        read_at: i64,
    },
}

impl Default for ControlSnapshot {
    fn default() -> Self {
        Self::Unread
    }
}

impl ControlSnapshot {
    pub fn from_reading(reading: &ControlStateReading, read_at: i64) -> Self {
        Self::Read {
            paused: reading.state.paused,
            auto_objectives_cap: reading.state.auto_objectives_cap,
            inbox_open_count: reading.inbox_open_count,
            read_at,
        }
    }

    pub fn paused(self) -> bool {
        matches!(self, Self::Read { paused: true, .. })
    }

    /// Plafond d'objectifs auto-générés ; `None` quand l'état n'a pas été lu
    /// (le budget ne peut alors pas être appliqué, l'ouverture est différée
    /// par la garde si l'état est inconnu).
    pub fn auto_objectives_cap(self) -> Option<u32> {
        match self {
            Self::Read {
                auto_objectives_cap,
                ..
            } => Some(auto_objectives_cap),
            Self::Unread | Self::Unknown => None,
        }
    }
}

/// Effets autonomes de Maicie. Fermé : ajouter un puits oblige à le classer
/// ici et donc à le tester.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomousEffect {
    /// Ouverture d'un objectif par une routine.
    RoutineOpening,
    /// Création d'une génération de réassignation.
    Reassignment,
    /// Rejeu d'une outbox de délégation en attente.
    OutboxReplay { origin_auto: bool },
    /// Matérialisation d'une outbox quand un prérequis se clôt.
    DependencyRelease,
    /// Rejeu d'un dispatch différé.
    DeferredDispatch,
}

/// Verdict de la garde unique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    Admitted,
    Deferred { motif: &'static str },
}

/// Garde unique des puits d'effet autonome de Maicie. Pure : table de vérité.
///
/// `Unread` admet : c'est le chemin des commandes qui n'ont pas relevé (aucun
/// puits ne s'y exécute en production, la relève lit toujours avant). `Unknown`
/// diffère tout : un daemon injoignable ne vaut jamais « pas de pause ».
pub fn admit_autonomous_effect(effect: AutonomousEffect, snapshot: ControlSnapshot) -> Admission {
    match snapshot {
        ControlSnapshot::Unread => Admission::Admitted,
        ControlSnapshot::Unknown => Admission::Deferred {
            motif: "controle_inconnu",
        },
        ControlSnapshot::Read { paused: false, .. } => Admission::Admitted,
        ControlSnapshot::Read { paused: true, .. } => match effect {
            // Une outbox d'origine humaine est le référent qui parle : la
            // pause ne le bride jamais.
            AutonomousEffect::OutboxReplay { origin_auto: false } => Admission::Admitted,
            AutonomousEffect::RoutineOpening
            | AutonomousEffect::Reassignment
            | AutonomousEffect::OutboxReplay { origin_auto: true }
            | AutonomousEffect::DependencyRelease
            | AutonomousEffect::DeferredDispatch => Admission::Deferred { motif: "pause" },
        },
    }
}

/// Périmètre d'émetteur de la lecture de contrôle : distinct des périmètres
/// humains du daemon, pour qu'une lecture ne soit jamais prise pour un
/// principal humain.
pub const CONTROL_ISSUER_SCOPE: &str = "maicie-control-snapshot-v1-read-only";

/// Lit l'état une fois. Injoignable ⇒ `Unknown`, jamais une erreur : la
/// relève continue et diffère ses effets autonomes.
pub fn read_control_snapshot(
    socket: &Path,
    limits: BridgetClientLimits,
    now: i64,
) -> ControlSnapshot {
    let deadline = Instant::now() + limits.connect_timeout + limits.io_timeout;
    let mut client = match ControlStateClient::connect_with_limits_until(
        socket,
        CONTROL_ISSUER_SCOPE,
        limits,
        deadline,
    ) {
        Ok(client) => client,
        Err(BridgetClientError::ClientRejected { .. }) => {
            // Daemon antérieur à SPEC-087 : la capacité n'existe pas. Il ne
            // porte pas de pause non plus ; on le dit plutôt que de tout
            // différer sur un daemon qui ne sait pas être en pause.
            return ControlSnapshot::Unread;
        }
        Err(_) => return ControlSnapshot::Unknown,
    };
    match client.read_control_state() {
        Ok(reading) => ControlSnapshot::from_reading(&reading, now),
        Err(_) => ControlSnapshot::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EFFECTS: [AutonomousEffect; 6] = [
        AutonomousEffect::RoutineOpening,
        AutonomousEffect::Reassignment,
        AutonomousEffect::OutboxReplay { origin_auto: true },
        AutonomousEffect::OutboxReplay { origin_auto: false },
        AutonomousEffect::DependencyRelease,
        AutonomousEffect::DeferredDispatch,
    ];

    fn read(paused: bool) -> ControlSnapshot {
        ControlSnapshot::Read {
            paused,
            auto_objectives_cap: 5,
            inbox_open_count: 0,
            read_at: 1,
        }
    }

    #[test]
    fn un_etat_lu_actif_admet_tout() {
        for effect in EFFECTS {
            assert_eq!(
                admit_autonomous_effect(effect, read(false)),
                Admission::Admitted
            );
        }
    }

    #[test]
    fn la_pause_differe_tout_sauf_la_parole_du_referent() {
        for effect in EFFECTS {
            let expected = if effect == (AutonomousEffect::OutboxReplay { origin_auto: false }) {
                Admission::Admitted
            } else {
                Admission::Deferred { motif: "pause" }
            };
            assert_eq!(
                admit_autonomous_effect(effect, read(true)),
                expected,
                "{effect:?}"
            );
        }
    }

    #[test]
    fn un_daemon_injoignable_ne_vaut_jamais_pas_de_pause() {
        for effect in EFFECTS {
            assert_eq!(
                admit_autonomous_effect(effect, ControlSnapshot::Unknown),
                Admission::Deferred {
                    motif: "controle_inconnu"
                },
                "{effect:?}"
            );
        }
    }

    #[test]
    fn l_absence_de_releve_admet_et_se_lit_comme_telle() {
        assert_eq!(ControlSnapshot::default(), ControlSnapshot::Unread);
        assert_eq!(
            admit_autonomous_effect(AutonomousEffect::RoutineOpening, ControlSnapshot::Unread),
            Admission::Admitted
        );
        assert_eq!(ControlSnapshot::Unread.auto_objectives_cap(), None);
        assert_eq!(read(true).auto_objectives_cap(), Some(5));
        assert!(read(true).paused());
        assert!(!ControlSnapshot::Unknown.paused());
    }

    #[test]
    fn une_socket_absente_rend_inconnu() {
        let snapshot = read_control_snapshot(
            Path::new("/nonexistent/bridget-087.sock"),
            BridgetClientLimits {
                connect_timeout: std::time::Duration::from_millis(50),
                io_timeout: std::time::Duration::from_millis(50),
                max_frame_bytes: 64 * 1024,
            },
            42,
        );
        assert_eq!(snapshot, ControlSnapshot::Unknown);
    }
}
