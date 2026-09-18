//! État de contrôle du référent (SPEC-087) : pause de l'autonomie et plafond
//! d'objectifs auto-générés.
//!
//! Le daemon est le seul écrivain. le service compagnon lit l'état à chaque relève et n'en
//! garde aucune copie. Tout puits d'effet autonome du daemon passe par
//! [`admit_autonomous_effect`] : un puits qui ne l'appelle pas est un défaut,
//! pas une limite acceptée (ADR 027).

use bridget_transport::protocol::{
    AgentPosture, CONTROL_STATE_CONTRACT_VERSION, ControlFocusFrame, ControlStateFrame,
    ControlStateRefusal,
};
use rusqlite::{Connection, OptionalExtension, params};

pub const DEFAULT_AUTO_OBJECTIVES_CAP: u32 = 5;
pub const MIN_AUTO_OBJECTIVES_CAP: u32 = 1;
pub const MAX_AUTO_OBJECTIVES_CAP: u32 = 100;
/// Un motif de pause est une phrase, pas un document.
pub const MAX_REASON_CHARS: usize = 512;

/// Libellés d'émetteur admis comme principal humain pour les mutations.
///
/// La borne est celle de l'ADR-011 : la façade MCP ne négocie jamais la
/// capacité `ControlStateV1`, et la CLI exige un terminal interactif. Un
/// processus hostile du même compte n'est pas arrêté par cette liste ; il est
/// rendu visible par l'item `human_route_replaced` de la boîte.
pub const HUMAN_PRINCIPAL_LABELS: [&str; 2] = ["bridget-ui-control", "bridget-control-cli"];

/// Rend l'acteur à journaliser si le périmètre d'émetteur est un principal
/// humain, sinon `None`.
pub fn human_principal_actor(issuer_scope: &str) -> Option<&'static str> {
    HUMAN_PRINCIPAL_LABELS
        .iter()
        .find(|label| crate::communication::issuer_scope(label) == issuer_scope)
        .map(|label| match *label {
            "bridget-ui-control" => "humain",
            _ => "cli",
        })
}

/// Journal fermé des mutations. Le libellé SQL dérive de `as_sql`, la clause
/// `CHECK` est construite à partir de `ALL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlEventKind {
    PauseOn,
    PauseOff,
    BudgetSet,
    /// SPEC-088 : posture d'agent ou réassignation automatique modifiée.
    RightsSet,
}

impl ControlEventKind {
    pub const ALL: [Self; 4] = [
        Self::PauseOn,
        Self::PauseOff,
        Self::BudgetSet,
        Self::RightsSet,
    ];

    pub fn as_sql(self) -> &'static str {
        match self {
            Self::PauseOn => "pause_on",
            Self::PauseOff => "pause_off",
            Self::BudgetSet => "budget_set",
            Self::RightsSet => "rights_set",
        }
    }

    fn sql_in_clause() -> String {
        let quoted: Vec<String> = Self::ALL
            .iter()
            .map(|kind| format!("'{}'", kind.as_sql()))
            .collect();
        format!("IN ({})", quoted.join(", "))
    }
}

/// Ligne du journal, relisible par `bridget control status --history`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlEvent {
    pub at: i64,
    pub actor: String,
    pub kind: &'static str,
    pub reason: Option<String>,
    pub generation_after: u64,
    pub command_id: String,
}

/// Mutation demandée par le principal humain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlMutation<'a> {
    pub command_id: &'a str,
    pub expected_generation: u64,
    pub paused: Option<bool>,
    pub auto_objectives_cap: Option<u32>,
    pub reason: Option<&'a str>,
    pub actor: &'a str,
    pub now: i64,
    /// SPEC-088.
    pub agent_posture: Option<AgentPosture>,
    pub auto_reassignment: Option<bool>,
}

/// Effets autonomes du plan de contrôle Bridget. Fermé : ajouter un puits
/// oblige à le classer ici, et donc à le tester.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomousEffect {
    /// Réveil de ronde par projet.
    ProjectRound,
    /// Rappel d'une demande suivie (paliers 1 et 2).
    ReminderNudge,
    /// Continuation automatique d'une exécution.
    Continuation,
}

/// Verdict de la garde unique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    Admitted,
    Deferred { motif: &'static str },
}

/// Garde unique des puits d'effet autonome du daemon. Pure : elle ne lit
/// aucune base et se teste par table de vérité.
pub fn admit_autonomous_effect(effect: AutonomousEffect, state: &ControlStateFrame) -> Admission {
    if state.paused {
        return Admission::Deferred { motif: "pause" };
    }
    match effect {
        AutonomousEffect::ProjectRound
        | AutonomousEffect::ReminderNudge
        | AutonomousEffect::Continuation => Admission::Admitted,
    }
}

pub fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    // Acquérir le droit d'écriture AVANT la première lecture de schéma.
    // Une transaction deferred lecture→ALTER peut échouer immédiatement par
    // SQLITE_BUSY malgré busy_timeout lorsque deux ouvertures se croisent.
    // L'observation neuf/existant et les droits migrés forment un seul fait.
    let tx = rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate)?;
    let conn = &tx;
    // SPEC-088 : « base existante » = la ligne de contrôle existait déjà
    // avant cette passe, quelle que soit sa génération.
    let pre_existing_row: bool = conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'control_state'")?
        .exists([])?
        && conn
            .prepare("SELECT 1 FROM control_state WHERE id = 1")?
            .exists([])?;
    conn.execute_batch(&format!(
        "CREATE TABLE IF NOT EXISTS control_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            generation INTEGER NOT NULL,
            paused INTEGER NOT NULL CHECK (paused IN (0, 1)),
            paused_since INTEGER,
            paused_by TEXT,
            pause_reason TEXT,
            auto_objectives_cap INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS control_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            at INTEGER NOT NULL,
            actor TEXT NOT NULL,
            kind TEXT NOT NULL CHECK (kind {kinds}),
            reason TEXT,
            generation_after INTEGER NOT NULL,
            command_id TEXT NOT NULL UNIQUE
        );
        INSERT OR IGNORE INTO control_state (
            id, generation, paused, paused_since, paused_by, pause_reason,
            auto_objectives_cap, updated_at
        ) VALUES (1, 0, 0, NULL, NULL, NULL, {cap}, 0);
        CREATE TABLE IF NOT EXISTS control_focus_projection (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            objective_id TEXT NOT NULL,
            goal TEXT NOT NULL,
            project_id TEXT NOT NULL,
            updated_at INTEGER NOT NULL CHECK (updated_at > 0)
        );",
        kinds = ControlEventKind::sql_in_clause(),
        cap = DEFAULT_AUTO_OBJECTIVES_CAP,
    ))?;
    migrate_rights_columns(conn, pre_existing_row)?;
    tx.commit()
}

/// SPEC-088 : colonnes de droits. Base neuve (aucune génération encore
/// écrite) ⇒ Prudent (découverte, réassignation différée). Base existante ⇒
/// le comportement observable d'avant la migration (complète, réassignation
/// active), pour ne pas changer un serveur en production à son redémarrage.
fn migrate_rights_columns(
    conn: &rusqlite::Transaction<'_>,
    pre_existing_row: bool,
) -> rusqlite::Result<()> {
    let has_column = |name: &str| -> rusqlite::Result<bool> {
        conn.prepare("SELECT 1 FROM pragma_table_info('control_state') WHERE name = ?1")?
            .exists([name])
    };
    let events_accept_rights: bool = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'control_events'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map(|sql| sql.contains("rights_set"))
        .unwrap_or(false);
    if has_column("agent_posture")? && has_column("auto_reassignment")? && events_accept_rights {
        return Ok(());
    }
    let (posture, reassignment) = if pre_existing_row {
        (AgentPosture::Complete, 1)
    } else {
        (AgentPosture::Discovery, 0)
    };
    // Une seule transaction : les deux colonnes ET la contrainte du journal,
    // sinon une interruption laisse une base que personne ne répare.
    if !has_column("agent_posture")? {
        conn.execute_batch(&format!(
            "ALTER TABLE control_state ADD COLUMN agent_posture TEXT NOT NULL DEFAULT '{posture}'
                CHECK (agent_posture IN ('discovery', 'complete'));",
            posture = posture.as_sql(),
        ))?;
    }
    if !has_column("auto_reassignment")? {
        conn.execute_batch(&format!(
            "ALTER TABLE control_state ADD COLUMN auto_reassignment INTEGER NOT NULL DEFAULT {reassignment}
                CHECK (auto_reassignment IN (0, 1));"
        ))?;
    }
    if !events_accept_rights {
        // SQLite ne modifie pas une contrainte CHECK : la table est
        // reconstruite avec la clause dérivée de `ControlEventKind::ALL`, et
        // ses lignes recopiées à l'identique.
        conn.execute_batch(&format!(
            "CREATE TABLE control_events_v088 (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                at INTEGER NOT NULL,
                actor TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind {kinds}),
                reason TEXT,
                generation_after INTEGER NOT NULL,
                command_id TEXT NOT NULL UNIQUE
            );
            INSERT INTO control_events_v088 (id, at, actor, kind, reason, generation_after, command_id)
                SELECT id, at, actor, kind, reason, generation_after, command_id FROM control_events;
            DROP TABLE control_events;
            ALTER TABLE control_events_v088 RENAME TO control_events;",
            kinds = ControlEventKind::sql_in_clause(),
        ))?;
    }
    Ok(())
}

pub fn read_focus(conn: &Connection) -> rusqlite::Result<Option<ControlFocusFrame>> {
    conn.query_row(
        "SELECT objective_id, goal, project_id, updated_at
         FROM control_focus_projection WHERE id = 1",
        [],
        |row| {
            Ok(ControlFocusFrame {
                objective_id: row.get(0)?,
                goal: row.get(1)?,
                project_id: row.get(2)?,
                updated_at: row.get(3)?,
            })
        },
    )
    .optional()
}

pub fn publish_focus(conn: &Connection, focus: Option<&ControlFocusFrame>) -> rusqlite::Result<()> {
    match focus {
        Some(focus) => {
            conn.execute(
                "INSERT INTO control_focus_projection(id, objective_id, goal, project_id, updated_at)
                 VALUES (1, ?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                    objective_id = excluded.objective_id,
                    goal = excluded.goal,
                    project_id = excluded.project_id,
                    updated_at = excluded.updated_at",
                params![focus.objective_id, focus.goal, focus.project_id, focus.updated_at],
            )?;
        }
        None => {
            conn.execute("DELETE FROM control_focus_projection WHERE id = 1", [])?;
        }
    }
    Ok(())
}

pub fn read(conn: &Connection) -> rusqlite::Result<ControlStateFrame> {
    conn.query_row(
        "SELECT generation, paused, paused_since, paused_by, pause_reason,
                auto_objectives_cap, updated_at, agent_posture, auto_reassignment
         FROM control_state WHERE id = 1",
        [],
        |row| {
            Ok(ControlStateFrame {
                version: CONTROL_STATE_CONTRACT_VERSION,
                generation: row.get::<_, i64>(0)?.max(0) as u64,
                paused: row.get::<_, i64>(1)? == 1,
                paused_since: row.get(2)?,
                paused_by: row.get(3)?,
                pause_reason: row.get(4)?,
                auto_objectives_cap: row.get::<_, i64>(5)?.clamp(0, i64::from(u32::MAX)) as u32,
                agent_posture: AgentPosture::from_sql(&row.get::<_, String>(7)?),
                auto_reassignment: Some(row.get::<_, i64>(8)? == 1),
                updated_at: row.get(6)?,
            })
        },
    )
}

/// Applique une mutation. Un rejeu du même `command_id` rend l'état courant
/// sans rien réécrire ; une génération inattendue est refusée.
pub fn set(
    conn: &Connection,
    mutation: ControlMutation<'_>,
) -> rusqlite::Result<Result<ControlStateFrame, ControlStateRefusal>> {
    let tx = conn.unchecked_transaction()?;
    let replayed: Option<i64> = tx
        .query_row(
            "SELECT generation_after FROM control_events WHERE command_id = ?1",
            params![mutation.command_id],
            |row| row.get(0),
        )
        .optional()?;
    if replayed.is_some() {
        let current = read(&tx)?;
        tx.commit()?;
        return Ok(Ok(current));
    }
    let current = read(&tx)?;
    if current.generation != mutation.expected_generation {
        return Ok(Err(ControlStateRefusal::GenerationMismatch {
            current: current.generation,
        }));
    }
    if let Some(cap) = mutation.auto_objectives_cap
        && !(MIN_AUTO_OBJECTIVES_CAP..=MAX_AUTO_OBJECTIVES_CAP).contains(&cap)
    {
        return Ok(Err(ControlStateRefusal::BudgetOutOfRange {
            min: MIN_AUTO_OBJECTIVES_CAP,
            max: MAX_AUTO_OBJECTIVES_CAP,
        }));
    }
    let pause_change = mutation.paused.filter(|paused| *paused != current.paused);
    let cap_change = mutation
        .auto_objectives_cap
        .filter(|cap| *cap != current.auto_objectives_cap);
    let posture_change = mutation
        .agent_posture
        .filter(|posture| Some(*posture) != current.agent_posture);
    let reassignment_change = mutation
        .auto_reassignment
        .filter(|value| Some(*value) != current.auto_reassignment);
    if pause_change.is_none()
        && cap_change.is_none()
        && posture_change.is_none()
        && reassignment_change.is_none()
    {
        return Ok(Err(ControlStateRefusal::NothingToChange));
    }
    let reason = mutation
        .reason
        .map(|text| text.chars().take(MAX_REASON_CHARS).collect::<String>());
    let generation_after = current.generation + 1;
    if let Some(paused) = pause_change {
        if paused {
            tx.execute(
                "UPDATE control_state SET paused = 1, paused_since = ?1, paused_by = ?2,
                        pause_reason = ?3, generation = ?4, updated_at = ?1 WHERE id = 1",
                params![
                    mutation.now,
                    mutation.actor,
                    reason,
                    generation_after as i64
                ],
            )?;
        } else {
            tx.execute(
                "UPDATE control_state SET paused = 0, paused_since = NULL, paused_by = NULL,
                        pause_reason = NULL, generation = ?1, updated_at = ?2 WHERE id = 1",
                params![generation_after as i64, mutation.now],
            )?;
        }
        tx.execute(
            "INSERT INTO control_events (at, actor, kind, reason, generation_after, command_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                mutation.now,
                mutation.actor,
                if paused {
                    ControlEventKind::PauseOn.as_sql()
                } else {
                    ControlEventKind::PauseOff.as_sql()
                },
                reason,
                generation_after as i64,
                mutation.command_id,
            ],
        )?;
    }
    if let Some(cap) = cap_change {
        tx.execute(
            "UPDATE control_state SET auto_objectives_cap = ?1, generation = ?2, updated_at = ?3
             WHERE id = 1",
            params![i64::from(cap), generation_after as i64, mutation.now],
        )?;
        if pause_change.is_none() {
            tx.execute(
                "INSERT INTO control_events (at, actor, kind, reason, generation_after, command_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    mutation.now,
                    mutation.actor,
                    ControlEventKind::BudgetSet.as_sql(),
                    reason,
                    generation_after as i64,
                    mutation.command_id,
                ],
            )?;
        }
    }
    if posture_change.is_some() || reassignment_change.is_some() {
        let posture = posture_change
            .or(current.agent_posture)
            .unwrap_or(AgentPosture::Discovery);
        let reassignment = reassignment_change
            .or(current.auto_reassignment)
            .unwrap_or(false);
        tx.execute(
            "UPDATE control_state SET agent_posture = ?1, auto_reassignment = ?2,
                    generation = ?3, updated_at = ?4 WHERE id = 1",
            params![
                posture.as_sql(),
                i64::from(reassignment),
                generation_after as i64,
                mutation.now
            ],
        )?;
        if pause_change.is_none() && cap_change.is_none() {
            tx.execute(
                "INSERT INTO control_events (at, actor, kind, reason, generation_after, command_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    mutation.now,
                    mutation.actor,
                    ControlEventKind::RightsSet.as_sql(),
                    reason,
                    generation_after as i64,
                    mutation.command_id,
                ],
            )?;
        }
    }
    let state = read(&tx)?;
    tx.commit()?;
    Ok(Ok(state))
}

/// Journal, du plus récent au plus ancien.
pub fn history(conn: &Connection, limit: u32) -> rusqlite::Result<Vec<ControlEvent>> {
    let mut statement = conn.prepare(
        "SELECT at, actor, kind, reason, generation_after, command_id
         FROM control_events ORDER BY id DESC LIMIT ?1",
    )?;
    let rows = statement.query_map(params![i64::from(limit)], |row| {
        let kind: String = row.get(2)?;
        Ok(ControlEvent {
            at: row.get(0)?,
            actor: row.get(1)?,
            kind: ControlEventKind::ALL
                .iter()
                .map(|candidate| candidate.as_sql())
                .find(|candidate| *candidate == kind)
                .unwrap_or("inconnu"),
            reason: row.get(3)?,
            generation_after: row.get::<_, i64>(4)?.max(0) as u64,
            command_id: row.get(5)?,
        })
    })?;
    rows.collect()
}

/// Rendu d'une ligne pour le pied de `bridget who` et `bridget control status`.
pub fn summary_line(state: &ControlStateFrame, now: i64) -> String {
    let pause = if state.paused {
        let since = state.paused_since.unwrap_or(now);
        let elapsed = now.saturating_sub(since).max(0);
        format!("pause depuis {}", format_duration(elapsed))
    } else {
        "autonomie active".to_string()
    };
    format!(
        "Contrôle : {pause} · plafond objectifs automatiques {}",
        state.auto_objectives_cap
    )
}

fn format_duration(secs: i64) -> String {
    let minutes = secs / 60;
    if minutes < 60 {
        return format!("{minutes} min");
    }
    let hours = minutes / 60;
    if hours < 48 {
        return format!("{hours} h {:02}", minutes % 60);
    }
    format!("{} j", hours / 24)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        conn
    }

    fn mutation<'a>(command_id: &'a str, generation: u64) -> ControlMutation<'a> {
        ControlMutation {
            command_id,
            expected_generation: generation,
            paused: None,
            auto_objectives_cap: None,
            reason: None,
            actor: "humain",
            now: 1_788_400_000,
            agent_posture: None,
            auto_reassignment: None,
        }
    }

    #[test]
    fn spec_088_base_neuve_est_prudente_et_base_existante_garde_son_comportement() {
        // Neuve : découverte, réassignation différée.
        let fresh = read(&conn()).unwrap();
        assert_eq!(fresh.agent_posture, Some(AgentPosture::Discovery));
        assert_eq!(fresh.auto_reassignment, Some(false));
        // Existante : la table 087 sans les colonnes, avec sa ligne ⇒ complète, active.
        // Schéma EXACT de a931a8ac (SPEC-087) : la contrainte CHECK du journal
        // ne connaît que trois kinds, et une ligne d'historique existe.
        let legacy = Connection::open_in_memory().unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE control_state (
                    id INTEGER PRIMARY KEY CHECK (id = 1), generation INTEGER NOT NULL,
                    paused INTEGER NOT NULL CHECK (paused IN (0, 1)), paused_since INTEGER, paused_by TEXT,
                    pause_reason TEXT, auto_objectives_cap INTEGER NOT NULL, updated_at INTEGER NOT NULL);
                 CREATE TABLE control_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT, at INTEGER NOT NULL, actor TEXT NOT NULL,
                    kind TEXT NOT NULL CHECK (kind IN ('pause_on', 'pause_off', 'budget_set')),
                    reason TEXT, generation_after INTEGER NOT NULL, command_id TEXT NOT NULL UNIQUE);
                 INSERT INTO control_state VALUES (1, 1, 0, NULL, NULL, NULL, 5, 0);
                 INSERT INTO control_events (at, actor, kind, reason, generation_after, command_id)
                    VALUES (10, 'humain', 'budget_set', NULL, 1, 'c-hist');",
            )
            .unwrap();
        ensure_schema(&legacy).unwrap();
        let migrated = read(&legacy).unwrap();
        assert_eq!(migrated.agent_posture, Some(AgentPosture::Complete));
        assert_eq!(migrated.auto_reassignment, Some(true));
        assert_eq!(
            history(&legacy, 5).unwrap().len(),
            1,
            "l'historique est recopié"
        );
        // Un changement de droits est réellement accepté par le journal migré.
        let state = set(
            &legacy,
            ControlMutation {
                auto_reassignment: Some(false),
                ..mutation("r-legacy", 1)
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(state.generation, 2);
        assert_eq!(history(&legacy, 1).unwrap()[0].kind, "rights_set");
        // Idempotent : une seconde passe ne change rien.
        ensure_schema(&legacy).unwrap();
        assert_eq!(read(&legacy).unwrap(), state);
    }

    #[test]
    fn spec_088_droits_journalises_sous_la_meme_generation() {
        let conn = conn();
        let state = set(
            &conn,
            ControlMutation {
                agent_posture: Some(AgentPosture::Complete),
                ..mutation("r1", 0)
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(state.generation, 1);
        assert_eq!(state.agent_posture, Some(AgentPosture::Complete));
        assert_eq!(
            state.auto_reassignment,
            Some(false),
            "la réassignation ne bouge pas seule"
        );
        let events = history(&conn, 5).unwrap();
        assert_eq!(events[0].kind, "rights_set");
        assert_eq!(events[0].generation_after, 1);
        // Rejeu du même command_id : état rendu, pas de génération.
        let replayed = set(
            &conn,
            ControlMutation {
                agent_posture: Some(AgentPosture::Complete),
                ..mutation("r1", 0)
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(replayed.generation, 1);
        // Même valeur, autre commande ⇒ rien à changer.
        assert_eq!(
            set(
                &conn,
                ControlMutation {
                    agent_posture: Some(AgentPosture::Complete),
                    ..mutation("r2", 1)
                },
            )
            .unwrap()
            .unwrap_err(),
            ControlStateRefusal::NothingToChange
        );
        // Posture + plafond dans la même mutation : une seule génération, journal du plafond.
        let both = set(
            &conn,
            ControlMutation {
                auto_reassignment: Some(true),
                auto_objectives_cap: Some(20),
                ..mutation("r3", 1)
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(both.generation, 2);
        assert_eq!(both.auto_reassignment, Some(true));
        assert_eq!(both.auto_objectives_cap, 20);
        assert_eq!(history(&conn, 1).unwrap()[0].generation_after, 2);
        // La pause n'est jamais touchée par une mutation de droits.
        assert!(!both.paused);
    }

    #[test]
    fn etat_par_defaut_actif_avec_plafond_cinq() {
        let state = read(&conn()).unwrap();
        assert!(!state.paused);
        assert_eq!(state.generation, 0);
        assert_eq!(state.auto_objectives_cap, DEFAULT_AUTO_OBJECTIVES_CAP);
        assert_eq!(state.paused_since, None);
    }

    #[test]
    fn pause_puis_reprise_journalisees_avec_generation() {
        let conn = conn();
        let state = set(
            &conn,
            ControlMutation {
                paused: Some(true),
                reason: Some("revue"),
                ..mutation("c1", 0)
            },
        )
        .unwrap()
        .unwrap();
        assert!(state.paused);
        assert_eq!(state.generation, 1);
        assert_eq!(state.paused_since, Some(1_788_400_000));
        assert_eq!(state.paused_by.as_deref(), Some("humain"));
        assert_eq!(state.pause_reason.as_deref(), Some("revue"));
        let state = set(
            &conn,
            ControlMutation {
                paused: Some(false),
                ..mutation("c2", 1)
            },
        )
        .unwrap()
        .unwrap();
        assert!(!state.paused);
        assert_eq!(state.generation, 2);
        assert_eq!(state.paused_since, None);
        let events = history(&conn, 10).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "pause_off");
        assert_eq!(events[1].kind, "pause_on");
        assert_eq!(events[1].reason.as_deref(), Some("revue"));
    }

    #[test]
    fn generation_inattendue_refusee_sans_ecriture() {
        let conn = conn();
        let refusal = set(
            &conn,
            ControlMutation {
                paused: Some(true),
                ..mutation("c1", 7)
            },
        )
        .unwrap()
        .unwrap_err();
        assert_eq!(
            refusal,
            ControlStateRefusal::GenerationMismatch { current: 0 }
        );
        assert!(!read(&conn).unwrap().paused);
        assert!(history(&conn, 10).unwrap().is_empty());
    }

    #[test]
    fn plafond_hors_bornes_et_absence_de_changement_refuses() {
        let conn = conn();
        let refusal = set(
            &conn,
            ControlMutation {
                auto_objectives_cap: Some(0),
                ..mutation("c1", 0)
            },
        )
        .unwrap()
        .unwrap_err();
        assert!(matches!(
            refusal,
            ControlStateRefusal::BudgetOutOfRange { min: 1, max: 100 }
        ));
        let refusal = set(
            &conn,
            ControlMutation {
                paused: Some(false),
                auto_objectives_cap: Some(DEFAULT_AUTO_OBJECTIVES_CAP),
                ..mutation("c1", 0)
            },
        )
        .unwrap()
        .unwrap_err();
        assert_eq!(refusal, ControlStateRefusal::NothingToChange);
    }

    #[test]
    fn rejeu_du_meme_command_id_rend_l_etat_sans_nouvel_evenement() {
        let conn = conn();
        let first = set(
            &conn,
            ControlMutation {
                auto_objectives_cap: Some(9),
                ..mutation("c1", 0)
            },
        )
        .unwrap()
        .unwrap();
        let replay = set(
            &conn,
            ControlMutation {
                auto_objectives_cap: Some(9),
                ..mutation("c1", 0)
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(first, replay);
        assert_eq!(history(&conn, 10).unwrap().len(), 1);
        assert_eq!(history(&conn, 10).unwrap()[0].kind, "budget_set");
    }

    #[test]
    fn etat_relu_apres_reouverture() {
        let dir = std::env::temp_dir().join(format!("rc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("control.db");
        {
            let conn = Connection::open(&path).unwrap();
            ensure_schema(&conn).unwrap();
            set(
                &conn,
                ControlMutation {
                    paused: Some(true),
                    ..mutation("c1", 0)
                },
            )
            .unwrap()
            .unwrap();
        }
        let conn = Connection::open(&path).unwrap();
        ensure_schema(&conn).unwrap();
        let state = read(&conn).unwrap();
        assert!(state.paused, "la pause survit à la réouverture");
        assert_eq!(state.generation, 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn garde_unique_table_de_verite() {
        let mut active = read(&conn()).unwrap();
        for effect in [
            AutonomousEffect::ProjectRound,
            AutonomousEffect::ReminderNudge,
            AutonomousEffect::Continuation,
        ] {
            assert_eq!(
                admit_autonomous_effect(effect, &active),
                Admission::Admitted
            );
        }
        active.paused = true;
        for effect in [
            AutonomousEffect::ProjectRound,
            AutonomousEffect::ReminderNudge,
            AutonomousEffect::Continuation,
        ] {
            assert_eq!(
                admit_autonomous_effect(effect, &active),
                Admission::Deferred { motif: "pause" }
            );
        }
    }

    #[test]
    fn principal_humain_reconnu_par_perimetre_seulement() {
        assert_eq!(
            human_principal_actor(&crate::communication::issuer_scope("bridget-ui-control")),
            Some("humain")
        );
        assert_eq!(
            human_principal_actor(&crate::communication::issuer_scope("bridget-control-cli")),
            Some("cli")
        );
        assert_eq!(
            human_principal_actor(&crate::communication::issuer_scope("bridget-ui")),
            None
        );
        assert_eq!(human_principal_actor("n-importe-quoi"), None);
    }

    #[test]
    fn resume_lisible() {
        let mut state = read(&conn()).unwrap();
        assert_eq!(
            summary_line(&state, 10),
            "Contrôle : autonomie active · plafond objectifs automatiques 5"
        );
        state.paused = true;
        state.paused_since = Some(0);
        assert_eq!(
            summary_line(&state, 2 * 3600 + 600),
            "Contrôle : pause depuis 2 h 10 · plafond objectifs automatiques 5"
        );
    }
}
