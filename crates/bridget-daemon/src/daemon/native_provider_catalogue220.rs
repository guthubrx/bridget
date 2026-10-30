//! La découverte est un fait, pas une nouvelle autorité de modèle ou de compte.
use super::*;
use bridget_transport::provider_catalogue::*;
use serde_json::{Value, json};

pub(super) struct CachedCatalogue {
    pub sequence: u64,
    pub received: Instant,
    pub value: ProviderCatalogue,
}

pub(super) fn ingest(
    st: &mut DaemonState,
    conn: &str,
    update: WorkerCatalogueUpdate,
) -> Option<DaemonToWrapper> {
    use bridget_transport::protocol::*;
    let rejected = |reason| Some(DaemonToWrapper::WorkerRejected { reason });
    let registry = AgentRegistry::load_from_path_quiet(st.registry.source().to_path_buf());
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
    let Ok(registry) = registry else {
        worker.catalogues.clear();
        return None;
    };
    let Some(policy) = registry.routing_policy().filter(|p| p.enabled) else {
        worker.catalogues.clear();
        return None;
    };
    worker.catalogues.retain(|id, _| {
        policy
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
        return None;
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
            value: update.catalogue,
        },
    );
    None
}

pub(super) fn project(st: &DaemonState, registry: &AgentRegistry) -> Vec<Value> {
    use bridget_transport::protocol::{WorkerCapability, WorkerContactState};
    let Some(policy) = registry.routing_policy().filter(|p| p.enabled) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for connection in &policy.connections {
        let worker = st.remote_workers.get(&connection.machine_id);
        let cached = worker.and_then(|w| w.catalogues.get(&connection.connection_id));
        let valid_definition = registry.resolved_definition(&connection.agent_type).ok();
        let (status, cause, observation, age) = if !connection.enabled {
            ("disabled", None, None, None)
        } else if policy
            .connections
            .iter()
            .filter(|c| c.machine_id == connection.machine_id)
            .count()
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
                || w.last_contact.elapsed()
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
            if (cached.received.elapsed() > Duration::from_secs(CATALOGUE_TTL_SECS)
                || unix_timestamp().saturating_sub(cached.value.observed_at)
                    > CATALOGUE_TTL_SECS as i64
                || cached.value.observed_at > unix_timestamp() + 30)
                && cached.value.cause != Some(CatalogueCause::StopUnconfirmed)
            {
                (
                    "stale",
                    None,
                    None,
                    Some(
                        cached.received.elapsed().as_secs().max(
                            unix_timestamp()
                                .saturating_sub(cached.value.observed_at)
                                .max(0) as u64,
                        ),
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
                        cached.received.elapsed().as_secs().max(
                            unix_timestamp()
                                .saturating_sub(cached.value.observed_at)
                                .max(0) as u64,
                        ),
                    ),
                )
            }
        } else {
            ("not_observed", None, None, None)
        };
        let models=observation.filter(|o|o.state==CatalogueState::Observed).map(|o|o.models.iter().map(|m| {
            let declared=policy.model_catalog.get(&connection.connection_id).is_some_and(|v|v.iter().any(|v|v.model==m.model));
            json!({"model":m.model,"supported_efforts":m.supported_efforts,"default_effort":m.default_effort,"declared_in_policy":declared})
        }).collect::<Vec<_>>()).unwrap_or_default();
        let guidance = match status {
            "connection_limit" => {
                "Ce serveur dépasse la borne de 32 connexions observées. Aucune sonde ne part sur ce worker. Faire qualifier la configuration par le responsable Bridget ; ne pas interpréter cette limite comme un refus du compte."
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
                "Ce worker ne négocie pas l’observation des modèles. Charger une version compatible de Bridget sur ce serveur ; aucun accès au modèle ne peut être déduit de l’ancien catalogue."
            }
            "sign_in_required" => {
                "Codex ne rapporte aucun compte connecté sur ce serveur. Demander à l’utilisateur de s’y connecter avec le mécanisme officiel et le compte prévu ; aucune copie de secrets. La prochaine observation est automatique."
            }
            "reported_models" => {
                "Le programme a annoncé cette liste lors de la lecture datée. Son cache fournisseur peut rester ancien ; ce n’est pas une garantie d’accès au modèle. Les modèles hors du profil Bridget restent non admissibles. Ne pas changer de compte, modèle ou droits sans décision autorisée."
            }
            "refreshing" | "not_observed" | "stale" => {
                "Attendre la fin du contrôle ou sa prochaine échéance (objectif de deux minutes par connexion ; les contrôles sont séquentiels et peuvent dépasser cet objectif en charge ou après échec), puis relire une fois le catalogue. Une liste ancienne n’est pas présentée comme actuelle. Si l’observation reste indisponible, faire vérifier ce worker ; ne pas relancer une mission d’issue inconnue."
            }
            _ if cause == Some(CatalogueCause::StopUnconfirmed) => {
                "L’arrêt d’une sonde de métadonnées n’est pas confirmé sur ce serveur. Aucune autre sonde n’est lancée. Demander une maintenance ciblée de cette sonde ; ne pas supprimer son marqueur ni affirmer l’arrêt. Les missions existantes restent séparées."
            }
            _ => {
                "L’observation native est indisponible pour cette connexion. Faire vérifier le programme et sa configuration sur le serveur nommé, avec ce code. La cause de compte ou d’abonnement n’est pas déduite de ce seul échec. Aucun rejeu de mission ni changement de droits."
            }
        };
        result.push(json!({"machine_id":connection.machine_id,"engine":connection.engine,"connection_id":connection.connection_id,
            "status":status,"cause":cause,"scope":"advertised_by_program","entitlement":"unknown","provider_cache":"not_attested",
            "auth":match status {"sign_in_required"=>"reported_absent","reported_models" if observation.and_then(|o|o.account_scope.as_ref()).is_some()=>"reported_present",_=>"not_verified"},
            "account_scope":observation.and_then(|o|o.account_scope.as_ref()),"observed_at":observation.map(|o|o.observed_at),"age_secs":age,"ttl_secs":CATALOGUE_TTL_SECS,"refresh_target_secs":120,"scheduling":"sequential_per_connection",
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
        worker.catalogues.clear();
        let conn = worker.conn_id.clone();
        let message = DaemonToWrapper::ProviderCatalogueRefresh {
            worker: worker.identity.clone(),
            connection_epoch: worker.epoch.clone(),
            refresh_id: worker.catalogue_refresh_id.clone(),
        };
        defer_control(st, &conn, message, controls);
    }
    Ok(
        json!({"version":1,"status":"requested","workers":selected.into_iter().map(|(machine,status)|json!({"machine_id":machine,"status":status})).collect::<Vec<_>>(),
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
