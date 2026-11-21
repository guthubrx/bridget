//! Entretien périodique du daemon : balayage des retenus, purges SQLite/flotte,
//! nettoyage des fichiers temporaires, rotation des journaux et restitution glibc.

use super::*;

/// Session 116 : cadence de l'entretien périodique du daemon.
pub(super) const MAINTENANCE_FIRST_DELAY: Duration = Duration::from_secs(120);
pub(super) const MAINTENANCE_INTERVAL: Duration = Duration::from_secs(3600);
/// Cadence de restitution des pages libres glibc vers le noyau Linux.
pub(super) const MEMORY_TRIM_INTERVAL: Duration = Duration::from_secs(300);
/// Marge après l'expiration d'une saga d'envoi avant sa purge : pendant elle,
/// un rejeu tardif reçoit encore « expiré » au lieu d'être réexpédié.
pub(super) const SEND_PURGE_GRACE_SECS: i64 = 30 * 24 * 3600;
/// Âge minimal d'un nom ou d'une preuve d'identité orphelins avant retrait.
pub(super) const IDENTITY_FILES_MIN_AGE: Duration = Duration::from_secs(7 * 24 * 3600);

/// Lance le fil détaché d'entretien périodique et de restitution mémoire.
pub(super) fn spawn_maintenance_thread(
    state: &Arc<Mutex<DaemonState>>,
    state_root: &std::path::Path,
) {
    let st_entretien = state.clone();
    let racine_entretien = state_root.to_path_buf();
    if let Err(error) = thread::Builder::new()
        .name("entretien".into())
        .spawn(move || {
            let mut next_maintenance = Instant::now() + MAINTENANCE_FIRST_DELAY;
            loop {
                let wait = next_maintenance
                    .saturating_duration_since(Instant::now())
                    .min(MEMORY_TRIM_INTERVAL);
                if sleep_until_shutdown(wait) {
                    return;
                }
                if Instant::now() >= next_maintenance {
                    run_maintenance(&st_entretien, &racine_entretien);
                    next_maintenance = Instant::now() + MAINTENANCE_INTERVAL;
                }
                let _ = crate::memory::trim_allocator_memory();
            }
        })
    {
        warn!("entretien: thread détaché impossible: {error}");
    }
}

/// FR006 : deux puits (tick des relances et entretien), un retrait transactionnel.
/// O(N + B + H(log N + log R) + L + P), N lignes SQL, R demandes, H retenus,
/// P suivis mémoire ; les identifiants retirés sont regroupés pour un seul retain.
/// B enveloppes et L dépôts liés ; le balayage global n’est pas un scan borné par LIMIT.
/// Le caller garde State ; la mémoire ne change qu'après le COMMIT de settle_held.
pub(super) fn sweep_held(st: &mut DaemonState, now: i64) -> Result<(), String> {
    let held = {
        let mut statement = st
            .store
            .connection()
            .prepare(
                "SELECT d.delivery_id, d.message_bytes, d.expires_at, r.expires_at
             FROM send_deliveries d JOIN idempotency_records r
               ON r.issuer_scope=d.issuer_scope AND r.operation_kind=d.operation_kind
                 AND r.idempotency_key=d.idempotency_key
             WHERE d.phase='held' ORDER BY d.queue_rank",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Vec<u8>>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?
    };
    let mut settled_ids = HashSet::new();
    // Même pattern Result local que wait_request : les erreurs SQL restent inchangées.
    let result = (|| -> Result<(), String> {
        for (delivery_id, bytes, delivery_expires, record_expires) in held {
            let message: bridget_core::BridgetMessage = serde_json::from_slice(&bytes)
                .map_err(|error| format!("retenu {delivery_id} illisible: {error}"))?;
            let reason = if delivery_expires <= now || record_expires <= now {
                Some("expired_in_queue")
            } else if message.reply {
                let request = st
                    .store
                    .get_request(&message.id)
                    .map_err(|error| error.to_string())?;
                (!request.is_some_and(|request| {
                    request.state == "open"
                        && request.sender == message.from
                        && request.target == message.to
                        && request.deadline_at > now
                }))
                .then_some("request_closed_in_queue")
            } else {
                None
            };
            if let Some(reason) = reason
                && let Some(notice) = st
                    .idempotency
                    .settle_held(&delivery_id, reason, now)
                    .map_err(|error| error.to_string())?
            {
                st.held_ids.remove(&notice.message_id);
                settled_ids.insert(notice.message_id);
            }
        }
        Ok(())
    })();
    // Une erreur tardive ne doit pas conserver les suivis des COMMIT déjà réussis.
    if !settled_ids.is_empty() {
        st.pending_replies
            .retain(|pending| !settled_ids.contains(&pending.msg_id));
    }
    result
}

/// Entretien des magasins SQLite, sous le verrou de l'état. Ordre imposé (session 150) :
/// la recopie des accusés natifs précède `purge_expired_sends`, qui efface les remises
/// quelle que soit leur phase ; la purge des arbres suit, jamais au démarrage.
pub(super) fn maintain_stores(st: &mut DaemonState, now: i64) {
    if let Err(error) = sweep_held(st, now) {
        warn!("entretien: balayage retenu refusé, purges suspendues: {error}");
        return;
    }
    match st.idempotency.settle_expired_dispatching(now) {
        Ok(0) => {}
        Ok(count) => info!(
            "entretien: {count} remise(s) expirée(s) passée(s) de « en vol » à « sort inconnu »"
        ),
        Err(error) => warn!("entretien: remises expirées : {error}"),
    }
    // Sans preuve enregistrée, les remises ne sont pas effacées dans ce passage.
    match native_delegation::copy_delivery_acks(st) {
        Ok(acked) => {
            if acked > 0 {
                info!("entretien: {acked} accusé(s) de remise native recopié(s) dans les tâches");
            }
            match st
                .idempotency
                .purge_expired_sends(now, SEND_PURGE_GRACE_SECS)
            {
                Ok(0) => {}
                Ok(count) => {
                    info!("entretien: {count} envoi(s) expiré(s) depuis plus de 30 jours purgé(s)")
                }
                Err(error) => warn!("entretien: purge des envois : {error}"),
            }
        }
        Err(error) => {
            warn!("entretien: accusés natifs non recopiés, purge des envois suspendue : {error}")
        }
    }
    native_delegation::maintain_retention(st, now);
    if let Err(error) = st.idempotency.purge_guidance_events(now) {
        warn!("entretien: purge du guidage : {error}");
    }
    if let Err(error) = st.idempotency.purge_queue_edits(now) {
        warn!("entretien: purge des reçus d'édition : {error}");
    }
    st.store.maintain_storage();
}

/// Session 116 : un passage d'entretien. Chaque étape est indépendante : un
/// échec est journalisé. Un retrait retenu refusé suspend les purges dépendantes.
/// State couvre SQL et les notices historiques ; les remises normales restent
/// différées hors State. Les fichiers sont traités hors verrou.
pub(super) fn run_maintenance(state: &Arc<Mutex<DaemonState>>, root: &std::path::Path) {
    let Some(_update_activity) = update_activity(state) else {
        return;
    };
    let now = unix_now_secs();
    let connected: HashSet<String> = {
        let mut st = state.lock().recover_poison();
        maintain_stores(&mut st, now);
        // Réemploi des notices durables ; ce chemin historique prend le writer sous State.
        st.flush_pending_orphan_emitter_notices();
        st.conn_instances.values().cloned().collect()
    };
    let identite = crate::mcp_identity::purge_stale_identity_files(
        root,
        &connected,
        IDENTITY_FILES_MIN_AGE,
        std::time::SystemTime::now(),
    );
    if identite != crate::mcp_identity::IdentityPurge::default() {
        info!(
            "entretien: état d'identité des disparus retiré : {} marqueur(s), {} nom(s), {} preuve(s)",
            identite.markers, identite.names, identite.proofs
        );
    }
    let tmp = crate::disk_hygiene::purge_orphan_bridget_tmp(&root.join("tmp"));
    if !tmp.deleted.is_empty() {
        info!(
            "entretien: {} temporaire(s) privé(s) retiré(s)",
            tmp.deleted.len()
        );
    }
    // Garder les configurations MCP : seule la fermeture de leur propriétaire
    // autorise leur retrait. Un fournisseur peut encore les lire avant register.
    let home = std::env::var_os("HOME").map(PathBuf::from);
    for log in crate::disk_hygiene::service_logs(root, home.as_deref()) {
        match crate::disk_hygiene::rotate_log_if_large(&log, crate::disk_hygiene::LOG_ROTATE_BYTES)
        {
            Ok(true) => info!(
                "entretien: journal {} tourné (génération précédente en .1)",
                log.display()
            ),
            Ok(false) => {}
            Err(error) => warn!("entretien: rotation de {} : {error}", log.display()),
        }
    }
}
