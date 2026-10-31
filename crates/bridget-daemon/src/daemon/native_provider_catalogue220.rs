//! La découverte est un fait, pas une nouvelle autorité de modèle ou de compte.
use super::*;
use bridget_transport::provider_catalogue::*;
use serde_json::{Value, json};

pub(super) struct CachedCatalogue {
    pub sequence: u64,
    pub received: Instant,
    pub received_at: i64,
    pub value: ProviderCatalogue,
}

pub(super) fn ingest(
    st: &mut DaemonState,
    conn: &str,
    mut update: WorkerCatalogueUpdate,
) -> Option<DaemonToWrapper> {
    use bridget_transport::protocol::*;
    let rejected = |reason| Some(DaemonToWrapper::WorkerRejected { reason });
    let registry_path = st.registry.source().to_path_buf();
    let Some(worker) = st.remote_workers.values_mut().find(|w| w.conn_id == conn) else {
        return rejected(WorkerRefusal::Unauthorized);
    };
    if !worker
        .capabilities
        .contains(&WorkerCapability::ProviderCatalogueV1)
    {
        return rejected(WorkerRefusal::CapabilityRequired);
    }
    if update.worker != worker.identity || update.connection_epoch != worker.epoch {
        return rejected(WorkerRefusal::StaleConnection);
    }
    if !update.valid() {
        return rejected(WorkerRefusal::InvalidEnvelope);
    }
    if update.refresh_id != worker.catalogue_refresh_id {
        return None;
    }
    let registry = AgentRegistry::load_from_path_quiet(registry_path);
    let Ok(registry) = registry else {
        worker
            .catalogues
            .retain(|_, value| value.value.cause == Some(CatalogueCause::StopUnconfirmed));
        return None;
    };
    let Some(policy) = registry.routing_policy().filter(|p| p.enabled) else {
        worker
            .catalogues
            .retain(|_, value| value.value.cause == Some(CatalogueCause::StopUnconfirmed));
        return None;
    };
    worker.catalogues.retain(|id, cached| {
        cached.value.cause == Some(CatalogueCause::StopUnconfirmed) || policy
            .connections
            .iter()
            .any(|c| c.machine_id == worker.identity.machine_id && c.connection_id == *id)
    });
    let Some(connection) = policy.connections.iter().find(|c| {
        c.machine_id == worker.identity.machine_id
            && c.connection_id == update.catalogue.connection_id
    }) else {
        return None;
    };
    if registry
        .resolved_definition(&connection.agent_type)
        .map_or(true, |d| d.digest != update.catalogue.definition_digest)
    {
        // Keep only the closed mismatch diagnostic. An old definition must not
        // leave the reader waiting for an observation that cannot be accepted.
        update.catalogue.state = CatalogueState::Unavailable;
        update.catalogue.models.clear();
        update.catalogue.account_scope = None;
        if update.catalogue.cause != Some(CatalogueCause::StopUnconfirmed) {
            update.catalogue.cause = Some(CatalogueCause::DefinitionChanged);
        }
    }
    let key = update.catalogue.connection_id.clone();
    if worker
        .catalogues
        .get(&key)
        .is_some_and(|v| v.sequence >= update.sequence)
    {
        return None;
    }
    if !worker.catalogues.contains_key(&key) && worker.catalogues.len() >= MAX_CATALOGUE_CONNECTIONS
    {
        return rejected(WorkerRefusal::ResourceLimit);
    }
    worker.catalogues.insert(
        key,
        CachedCatalogue {
            sequence: update.sequence,
            received: Instant::now(),
            received_at: unix_timestamp(),
            value: update.catalogue,
        },
    );
    None
}

pub(super) fn project(st: &DaemonState, registry: &AgentRegistry) -> Vec<Value> {
    project_at(st, registry, unix_timestamp(), Instant::now())
}

/// Horloges capturées une seule fois ; le point d'entrée et les tests partagent
/// exactement les mêmes bornes, sans sommeil ni horloge globale de test.
pub(super) fn project_at(
    st: &DaemonState,
    registry: &AgentRegistry,
    now: i64,
    instant: Instant,
) -> Vec<Value> {
    use bridget_transport::protocol::{WorkerCapability, WorkerContactState};
    let Some(policy) = registry.routing_policy().filter(|p| p.enabled) else {
        return Vec::new();
    };
    let mut connection_counts = std::collections::BTreeMap::<&str, usize>::new();
    for connection in &policy.connections {
        *connection_counts
            .entry(connection.machine_id.as_str())
            .or_default() += 1;
    }
    let mut result = Vec::new();
    for connection in &policy.connections {
        let worker = st.remote_workers.get(&connection.machine_id);
        let cached = worker.and_then(|w| w.catalogues.get(&connection.connection_id));
        let valid_definition = registry.resolved_definition(&connection.agent_type).ok();
        let (status, cause, observation, age) = if !connection.enabled {
            ("disabled", None, None, None)
        } else if registry
            .machines()
            .get(&connection.machine_id)
            .is_some_and(|m| !m.enabled)
        {
            ("machine_disabled", None, None, None)
        } else if connection_counts
            .get(connection.machine_id.as_str())
            .copied()
            .unwrap_or(0)
            > MAX_CATALOGUE_CONNECTIONS
        {
            (
                "connection_limit",
                Some(CatalogueCause::OutputLimit),
                None,
                None,
            )
        } else if connection.engine != "codex" {
            (
                "unsupported_provider",
                Some(CatalogueCause::UnsupportedProvider),
                None,
                None,
            )
        } else if worker.is_none() && registry.machine(&connection.machine_id).is_err() {
            ("observation_source_unavailable", None, None, None)
        } else if worker.is_none_or(|w| {
            w.contact != WorkerContactState::Connected
                || instant.saturating_duration_since(w.last_contact)
                    >= Duration::from_secs(bridget_transport::protocol::WORKER_STALE_SECS as u64)
        }) {
            ("worker_unavailable", None, None, None)
        } else if !worker
            .unwrap()
            .capabilities
            .contains(&WorkerCapability::ProviderCatalogueV1)
        {
            ("unverified_peer", None, None, None)
        } else if let Some(cached) = cached {
            if (instant.saturating_duration_since(cached.received)
                > Duration::from_secs(CATALOGUE_TTL_SECS)
                || now.saturating_sub(cached.value.observed_at) > CATALOGUE_TTL_SECS as i64
                || cached.value.observed_at > now + 30)
                && cached.value.cause != Some(CatalogueCause::StopUnconfirmed)
            {
                (
                    "stale",
                    None,
                    None,
                    Some(
                        instant
                            .saturating_duration_since(cached.received)
                            .as_secs()
                            .max(now.saturating_sub(cached.value.observed_at).max(0) as u64),
                    ),
                )
            } else if valid_definition
                .as_ref()
                .is_none_or(|d| d.digest != cached.value.definition_digest)
            {
                (
                    "definition_changed",
                    Some(CatalogueCause::DefinitionChanged),
                    None,
                    None,
                )
            } else {
                let value = &cached.value;
                (
                    match value.state {
                        CatalogueState::Refreshing => "refreshing",
                        CatalogueState::Observed => "reported_models",
                        CatalogueState::SignInRequired => "sign_in_required",
                        CatalogueState::Unavailable => "unavailable",
                        CatalogueState::Disabled => "disabled",
                    },
                    value.cause,
                    Some(value),
                    Some(
                        instant
                            .saturating_duration_since(cached.received)
                            .as_secs()
                            .max(now.saturating_sub(cached.value.observed_at).max(0) as u64),
                    ),
                )
            }
        } else {
            ("not_observed", None, None, None)
        };
        // O(d log d + m log d), d/m<=256 : une seule indexation par connexion.
        let declared = policy
            .model_catalog
            .get(&connection.connection_id)
            .into_iter()
            .flatten()
            .map(|model| (model.model.as_str(), model))
            .collect::<std::collections::BTreeMap<_, _>>();
        let models=observation.filter(|o|o.state==CatalogueState::Observed).map(|o|o.models.iter().map(|m| {
            let policy_model=declared.get(m.model.as_str());
            let supported=m.supported_efforts.iter().collect::<std::collections::BTreeSet<_>>();
            let missing=policy_model.into_iter().flat_map(|p|p.supported_efforts.iter())
                .filter(|effort|!supported.contains(effort)).collect::<Vec<_>>();
            let default_announced=policy_model.and_then(|p|p.default_effort.as_ref())
                .map(|effort|supported.contains(effort));
            let effort_status=match policy_model {
                None=>"outside_policy",
                Some(p) if p.supported_efforts.is_empty() && p.default_effort.is_none()=>"not_configured",
                Some(_) if !missing.is_empty() || default_announced==Some(false)=>"not_announced",
                Some(_)=>"announced",
            };
            json!({"model":m.model,"supported_efforts":m.supported_efforts,"default_effort":m.default_effort,
                "declared_in_policy":policy_model.is_some(),
                "policy_effort_status":effort_status,"declared_efforts_not_announced":missing,
                "declared_default_effort_announced":default_announced})
        }).collect::<Vec<_>>()).unwrap_or_default();
        let source_age = cached.map(|v| now.saturating_sub(v.value.observed_at).max(0) as u64);
        let received_age = cached.map(|v| instant.saturating_duration_since(v.received).as_secs());
        let remaining = age.map(|age| CATALOGUE_TTL_SECS.saturating_sub(age));
        let freshness_cause = if status == "stale" {
            cached.map(|v| {
                if v.value.observed_at > now.saturating_add(30) {
                    "source_clock_ahead"
                } else if source_age.unwrap_or(0) > CATALOGUE_TTL_SECS
                    && instant.saturating_duration_since(v.received)
                        > Duration::from_secs(CATALOGUE_TTL_SECS)
                {
                    "source_and_receipt_expired"
                } else if instant.saturating_duration_since(v.received)
                    > Duration::from_secs(CATALOGUE_TTL_SECS)
                {
                    "receipt_expired"
                } else {
                    "source_expired"
                }
            })
        } else {
            None
        };
        let expires_at = if status == "reported_models" {
            remaining.map(|left| now.saturating_add(left as i64))
        } else {
            None
        };
        let guidance = match status {
            "connection_limit" => {
                "Ce serveur dépasse la borne de 32 connexions observées. Aucune sonde ne part sur ce worker. Faire qualifier la configuration par le responsable Bridget ; ne pas interpréter cette limite comme un refus du compte."
            }
            "machine_disabled" => {
                "Ce serveur est désactivé dans Bridget. Demander au responsable si son activation est autorisée, puis qualifier le lien existant. Ne pas installer un autre worker ni changer de compte pour contourner cette décision."
            }
            "disabled" => {
                "Cette connexion est désactivée. Une nouvelle connexion au fournisseur ne l’active pas. Demander au responsable de ce serveur de vérifier la décision d’activation ; aucun changement automatique."
            }
            "unsupported_provider" => {
                "La découverte automatique n’est pas prise en charge pour ce fournisseur. Le catalogue déclaré et les gardes habituelles restent applicables ; ne pas annoncer un accès vérifié."
            }
            "observation_source_unavailable" => {
                "Aucun worker observateur n’est déclaré pour cette connexion. Demander au responsable Bridget de qualifier son mode d’observation local ou un worker provisionné. Ce fait ne prouve pas une panne de lien ni un défaut du compte."
            }
            "worker_unavailable" => {
                "Le worker de ce serveur n’est pas joignable. Rétablir son lien natif avant de demander une nouvelle observation ; ce constat ne ferme ni ne relance ses missions."
            }
            "unverified_peer" => {
                "Ce worker ne négocie pas l’observation des modèles. Avant tout changement de version, faire vérifier les marqueurs privés et les processus de sonde sur ce serveur par le responsable Bridget : ce statut ne confirme aucun arrêt. Ne pas effacer un marqueur ni créer une mission pour diagnostiquer. Charger ensuite une version compatible selon le parcours de maintenance. Aucun accès au modèle ne peut être déduit de l’ancien catalogue. Ce contrôle concerne l’observation de cette connexion ; les autres travaux autorisés suivent leur propre état."
            }
            "sign_in_required" => {
                "Codex ne rapporte aucun compte connecté sur ce serveur. Demander à l’utilisateur de s’y connecter avec le mécanisme officiel et le compte prévu ; aucune copie de secrets. La prochaine observation est automatique."
            }
            "definition_changed" => {
                "La définition configurée pour cette connexion diffère de celle de la dernière observation. Faire qualifier la définition prévue par le responsable Bridget, puis demander une nouvelle observation sur ce serveur. Ne pas changer de compte ou de droits ni rejouer une mission pour résoudre cet écart. Les autres travaux autorisés suivent leur propre état."
            }
            "reported_models" => {
                "Le programme a annoncé cette liste lors de la lecture datée. Son cache fournisseur peut rester ancien ; ce n’est pas une garantie d’accès au modèle. Les modèles hors du profil Bridget restent non admissibles. Ne pas changer de compte, modèle ou droits sans décision autorisée."
            }
            "stale" if freshness_cause == Some("source_clock_ahead") => {
                "La date annoncée est trop en avance sur l’horloge du coordinateur. Faire vérifier les horloges de ce serveur et du coordinateur, puis obtenir une nouvelle observation ; ne pas déduire un défaut du compte ou du réseau. Aucun arrêt de sonde n’est confirmé par ce refus : vérifier ses marqueurs privés avant un changement de version. Ce constat concerne cette observation ; les autres travaux autorisés suivent leur propre état."
            }
            "stale"
                if freshness_cause == Some("source_expired")
                    && received_age.is_some_and(|age| age < 30) =>
            {
                "Cette observation vient d’être reçue mais porte déjà une date ancienne. Faire vérifier la date de lecture, les horloges du serveur et du coordinateur, puis le prochain contrôle. Ce seul écart ne distingue pas une horloge en retard d’une remise différée ; ne pas inventer une panne de compte ou de réseau. L’arrêt d’une sonde n’est pas confirmé : vérifier ses marqueurs privés avant tout changement de version. Les autres travaux autorisés suivent leur propre état."
            }
            "refreshing" | "not_observed" | "stale" => {
                "Attendre la fin du contrôle ou sa prochaine échéance (objectif de deux minutes par connexion ; les contrôles sont séquentiels et peuvent dépasser cet objectif en charge ou après échec), puis relire une fois le catalogue. Une liste ancienne n’est pas présentée comme actuelle. Si l’observation reste indisponible, faire vérifier ce worker ; ne pas relancer une mission d’issue inconnue. Ce statut ne confirme aucun arrêt de sonde. Avant tout changement de version, faire vérifier ses marqueurs privés et processus sur ce serveur par le responsable Bridget ; ne pas effacer un marqueur. Ce contrôle concerne l’observation de cette connexion ; les autres travaux autorisés suivent leur propre état."
            }
            _ if cause == Some(CatalogueCause::StopUnconfirmed) => {
                "L’arrêt d’une sonde de métadonnées n’est pas confirmé sur ce serveur. Aucune autre sonde n’est lancée. Demander une maintenance ciblée de cette sonde ; ne pas supprimer son marqueur ni affirmer l’arrêt. Les missions existantes restent séparées."
            }
            _ => {
                "L’observation native est indisponible pour cette connexion. Faire vérifier le programme et sa configuration sur le serveur nommé, avec ce code. La cause de compte ou d’abonnement n’est pas déduite de ce seul échec. Aucun rejeu de mission ni changement de droits."
            }
        };
        let guidance = if cached
            .is_some_and(|value| value.value.cause == Some(CatalogueCause::StopUnconfirmed))
            && cause != Some(CatalogueCause::StopUnconfirmed)
        {
            format!(
                "{guidance} Un arrêt de sonde reste non confirmé : faire vérifier son marqueur privé et son groupe avant toute activation ou modification de version ; ce statut ne le clôt pas."
            )
        } else {
            guidance.to_owned()
        };
        result.push(json!({"machine_id":connection.machine_id,"engine":connection.engine,"connection_id":connection.connection_id,
            "status":status,"cause":cause,"scope":"advertised_by_program","entitlement":"unknown","provider_cache":"not_attested",
            "auth":match status {"sign_in_required"=>"reported_absent","reported_models" if observation.and_then(|o|o.account_scope.as_ref()).is_some()=>"reported_present",_=>"not_verified"},
            "account_scope":observation.and_then(|o|o.account_scope.as_ref()),"observed_at":observation.map(|o|o.observed_at),"age_secs":age,"ttl_secs":CATALOGUE_TTL_SECS,"refresh_target_secs":120,"scheduling":"sequential_per_connection",
            "last_observed_at":cached.map(|v|v.value.observed_at),"received_at":cached.map(|v|v.received_at),
            "source_age_secs":source_age,"received_age_secs":received_age,"expires_at":expires_at,
            "expiry_basis":"local_estimate_from_remaining_source_and_receipt_budget",
            "freshness_cause":freshness_cause,
            "policy_effort_guidance":"not_announced signale un écart entre profil et liste annoncée ; ce n’est pas une preuve d’interdiction du fournisseur. Faire qualifier le programme et le profil avant toute modification autorisée. Aucun droit, compte, modèle ou effort de mission déjà admise n’est changé ; ne pas relancer automatiquement.",
            "account_change_detection":if connection.engine=="codex" {json!({
                "signals":["auth_file_metadata","config_file_metadata","program_metadata","definition_digest","worker_reconnection"],
                "external_login_without_file_change":"periodic_check_only","refresh_target_secs":120,
                "refresh_target_is_guarantee":false})} else {json!({"signals":[],"status":"unsupported_provider"})},
            "probe_constraint":cached.filter(|v|v.value.cause==Some(CatalogueCause::StopUnconfirmed)).map(|v|json!({"state":"stop_unconfirmed","observed_at":v.value.observed_at,"next_action":"Faire vérifier le marqueur privé probe de ce worker et son groupe de processus par le responsable Bridget. Le statut de connexion ou de serveur ne constate pas leur arrêt. Ne pas effacer le marqueur ni relancer de mission pour diagnostiquer."})),
            "models":models,"next_action":guidance,"other_work":"Ce constat porte sur cette connexion de ce serveur. Les autres travaux autorisés suivent leur propre état."}));
    }
    result.sort_by(|a, b| {
        a["connection_id"]
            .as_str()
            .cmp(&b["connection_id"].as_str())
    });
    result
}

fn fresh(
    st: &DaemonState,
) -> Result<(bridget_transport::protocol::RoutingCatalogueV1, Vec<Value>), String> {
    let registry = AgentRegistry::load_from_path_quiet(st.registry.source().to_path_buf())
        .map_err(|_| "routing_catalogue_unavailable".to_string())?;
    Ok((
        registry.routing_catalogue(unix_timestamp())?,
        project(st, &registry),
    ))
}

pub(super) fn view(st: &DaemonState) -> Result<Value, String> {
    let (catalogue, observations) = fresh(st)?;
    let mut value = serde_json::to_value(catalogue).map_err(|_| "routing_catalogue_unavailable")?;
    value["connection_catalogues"] = json!(observations);
    Ok(value)
}

pub(super) fn enrich(st: &DaemonState, result: &mut Value, origin: &str) {
    match fresh(st) {
        Ok((declared, observations)) => {
            super::native_delegation::catalogue_scope212(result, Ok(declared), origin);
            result["fleet"]["connection_catalogues"] = json!(observations);
        }
        Err(code) => super::native_delegation::catalogue_scope212(result, Err(code), origin),
    }
}

pub(super) fn refresh(
    st: &mut DaemonState,
    machine: Option<&str>,
    controls: &mut Vec<DeferredControl>,
) -> Result<Value, String> {
    use bridget_transport::protocol::WorkerCapability;
    if machine.is_some_and(|m| !catalogue_token(m)) {
        return Err("provider_catalogue_machine_invalid".into());
    }
    let registry = AgentRegistry::load_from_path_quiet(st.registry.source().to_path_buf())
        .map_err(|_| "routing_catalogue_unavailable".to_owned())?;
    let policy = registry
        .routing_policy()
        .filter(|policy| policy.enabled)
        .ok_or_else(|| "routing_policy_unavailable".to_owned())?;
    let mut connection_counts = std::collections::BTreeMap::<&str, usize>::new();
    for connection in &policy.connections {
        *connection_counts
            .entry(connection.machine_id.as_str())
            .or_default() += 1;
    }
    let mut selected = Vec::new();
    for (id, worker) in &st.remote_workers {
        if machine.is_some_and(|m| m != id) {
            continue;
        }
        let status = if worker.contact != bridget_transport::protocol::WorkerContactState::Connected
            || !st.connections.contains_key(&worker.conn_id)
        {
            "worker_unavailable"
        } else if !worker
            .capabilities
            .contains(&WorkerCapability::ProviderCatalogueV1)
        {
            "unverified_peer"
        } else if connection_counts.get(id.as_str()).copied().unwrap_or(0)
            > MAX_CATALOGUE_CONNECTIONS
        {
            "connection_limit"
        } else if worker
            .catalogue_refresh_at
            .is_some_and(|t| t.elapsed() < Duration::from_secs(30))
        {
            "already_requested"
        } else {
            "requested"
        };
        selected.push((id.clone(), status));
    }
    if selected.is_empty() {
        return Err("provider_catalogue_worker_unavailable".into());
    }
    for (id, status) in &selected {
        if *status != "requested" {
            continue;
        }
        let worker = st.remote_workers.get_mut(id).unwrap();
        worker.catalogue_refresh_id = uuid::Uuid::new_v4().to_string();
        worker.catalogue_refresh_at = Some(Instant::now());
        worker
            .catalogues
            .retain(|_, v| v.value.cause == Some(CatalogueCause::StopUnconfirmed));
        let conn = worker.conn_id.clone();
        let message = DaemonToWrapper::ProviderCatalogueRefresh {
            worker: worker.identity.clone(),
            connection_epoch: worker.epoch.clone(),
            refresh_id: worker.catalogue_refresh_id.clone(),
        };
        defer_control(st, &conn, message, controls);
    }
    Ok(
        json!({"version":1,"status":"requested","workers":selected.into_iter().map(|(machine,status)|json!({"machine_id":machine,"status":status,
            "next_action":if status=="connection_limit" {
                "Ce serveur dépasse la borne de 32 connexions configurées, désactivées comprises. Aucun contrôle demandé. Faire qualifier cet inventaire par le responsable Bridget ; ne pas changer de compte ou de droits. Les missions existantes suivent leur propre état."
            } else {"Lire ce statut puis le catalogue du serveur ; une demande n'est pas une lecture terminée."}})).collect::<Vec<_>>(),
        "next_action":"La demande ne prouve pas que la lecture est terminée. Relire une fois le catalogue après quelques secondes. Aucun compte, profil ni mission n’est modifié.",
        "other_work":"Les autres travaux autorisés suivent leur propre état."}),
    )
}

// Refresh metadata uses the existing bounded writer, independently of historic
// control frames. An incomplete write is unconfirmed, never a completed read.
pub(super) fn send_controls(controls: Vec<DeferredControl>) -> Vec<DeferredControl> {
    let deadline = Instant::now() + Duration::from_secs(1);
    controls
        .into_iter()
        .filter(|c| push_control_message_until(&c.writer, &c.message, deadline).is_err())
        .collect()
}
