# Archives techniques de l’adaptateur T3

Document d’origine : [specs/101-abonnements-t3/reuse-audit.md](../../../../../../specs/101-abonnements-t3/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Statut: PASS
Date: 2026-09-16
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3/specs/101-abonnements-t3

<a id="extrait-2"></a>

## Extrait 2

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Abonnements/filtres/TTL/collisions | Observations | crates/bridget-daemon/src/observation.rs:43 | Étendre, pas de second moteur |
| Persistance interruption | Connexion Store | crates/bridget-daemon/src/store.rs:31 | Table dédiée nécessaire, aucun KV équivalent |
| Notifications | observation_output/observe_fact | crates/bridget-daemon/src/daemon.rs:1280 | File bornée, DND, origine système |
| Capacités source | Protocole observation | crates/bridget-transport/src/protocol.rs:2148 | Nouveau fait primaire, pas registre autonome |
| Autorité source | ObservedActivity/live_connection_identity | crates/bridget-daemon/src/daemon.rs:9866 | Même interdiction des auxiliaires |
| Identité T3 | Marqueurs/process_birth | crates/bridget-daemon/src/mcp_identity.rs:268 ; crates/bridget-daemon/src/managed_process.rs:772 | Nouveau module d'adaptateur justifié |
| Rollouts ouverts | Inventaire lsof existant | crates/bridget-daemon/src/runtime.rs:179 | Exposer tous les candidats, pas le dernier |
| Journal T3 | ThreadState/JournalLiveFeed/relay | crates/bridget-daemon/src/t3code.rs:201 | Étendre curseurs et corrélation |
| Écritures confirmées | confirmed_write_payload | crates/bridget-transport/src/journal.rs:578 | Réutiliser payload commun |
| SQLite | rusqlite | crates/bridget-daemon/Cargo.toml:27 | Aucune dépendance nouvelle |

<a id="extrait-3"></a>

## Extrait 3

Aucun doublon fonctionnel de moteur ou service. Mesure finale jscpd sur
t3code_identity.rs, observation.rs et spec101_observation_test.rs : un clone
de10lignes, 0,47% de2112lignes. Lecture privée commune entre read_marker et
rollout_session : doublon court assumé, parsers et limites différents, pas de
helper générique supplémentaire pour déplacer seulement ces vérifications.
Ce taux concerne ces trois fichiers, pas le dépôt entier.

<a id="extrait-4"></a>

## Extrait 4

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 098 | Adaptateur T3 borné | Étendre le contrat local sans modifier T3 |
| 099 | Preuve auxiliaire et identité | Réutiliser, ne pas affaiblir |
| 100 | Observations/journal non bloquants | Étendre état et producteurs |

<a id="extrait-5"></a>

## Extrait 5

| Requete | Portee | Resultat |
|---|---|---|
| rg Observation, subscription | crates, specs | Moteur100 unique |
| rg CREATE TABLE, metadata, settings | store.rs/store_schema.rs | Pas de table d'abonnements/KV adapté |
| rg process_parent, process_birth, write_marker | crates/bridget-daemon/src | Primitives099 réutilisables |
| rg open_session_file, lsof | runtime.rs | Inventaire existe ; mtime interdit pour identité |
| rg ThreadDetail, project_journal, pending | t3code*.rs | Projection et curseur existants |
| rg approval.requested, file_change, tool.completed | source T3 locale | Contrats attestés, types à filtrer |

<a id="extrait-6"></a>

## Extrait 6

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| t3code_identity.rs | créer module spécifique | Frontière de sécurité OS/SQLite propre à T3 ; réutilise primitives existantes | 2026-09-16 |
| Publication exclusive du marqueur | étendre mcp_identity | write_marker remplace l'existant ; write_marker_if_absent réutilise AgentPidMarker et fsutil mais publie par lien dur sans écrasement concurrent. Justification : autorité et propriété, pas de wrapper gratuit. | 2026-09-16 |
| Reprise des marqueurs après crash du pont | étendre t3code_identity | Drop ne s'exécute pas sur SIGTERM ; le nom privé atteste PID + naissance + nonce du propriétaire. Seuls les marqueurs du namespace exact d'un propriétaire mort/recyclé sont retirés ; tiers/vivants/inconnus préservés. Aucun arrêt de fournisseur. | 2026-09-16 |
| Table abonnements | créer dans Store existant | Survie de la trace d'interruption, pas de file durable ni stockage Maicie | 2026-09-16 |
| Annonce capacités | étendre protocole | Les capacités existantes ne portent pas ce fait source | 2026-09-16 |
| Lacune de faits amont (T013) | étendre protocole et Observations | Recherche `ObservationGap`, `facts_lost`, `report_gap`, `observation_gap` : seul wrapper.rs:2688 observe le compteur perdu ; Gap existant concerne une vue attach et exige subscription_id. ObservationGap porte uniquement le nombre de faits perdus d'une source primaire, réutilise la file de notifications et les abonnements bornés ; compteur distinct des remises perdues. | 2026-09-16 |
| Écriture SQLite sans attente (T013) | réutiliser save_observation_snapshot | store.rs:48 utilise déjà l'UPSERT atomique ; délai temporaire nul puis restauration du délai précédent pour ne pas attendre sous le verrou daemon. Aucun worker, store ou connexion de production ajouté. | 2026-09-16 |

<a id="extrait-7"></a>

## Extrait 7

La table des abonnements persiste TTL absolu, pas Instant. Création/once/unsub
cohérentes avec la sauvegarde. Pas d'E/S pour chaque fait non pertinent.
SQLite T3 ouverte READ_ONLY directement, jamais par Store::open migrateur.
