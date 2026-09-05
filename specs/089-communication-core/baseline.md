# Baseline 089 — carte de l'extraction

Référence immuable : `dfa2134dcfe2a2522e3ae77d93561e6ae72556b3`. Lecture du 2026-09-05 ; chemins et lignes ci-dessous relatifs à cette référence. C'est une disposition avant découplage, **pas une permission de suppression**.

## Dépendances constatées

Cargo.toml:4–8 inclut core, transport, daemon et plugins/maicie.
crates/bridget-daemon/Cargo.toml:33 dépend directement de Maicie en production (pas seulement en dev-dependencies). reqwest:34 et serde_yaml:23 doivent être réévalués par leurs consommateurs ; ne pas les supprimer au seul motif « web ».
Transport ne dépend localement que de core. Maicie dépend de core/transport : absence de cycle Cargo, mais couplage de produit obligatoire.
Toolchain : rust-toolchain.toml, 1.92.0. Aucun changement de version nécessaire.

## Tous les modules du daemon (45)

Base des références de cette table : crates/bridget-daemon/src/. « Extraire » signifie module mixte : déplacer ses politiques/présentations après caractérisation, jamais tout effacer.

| Module | Disposition | Preuve | Frontière utile |
|---|---|---|---|
| agent_profile | extraire | lib.rs:1 ; agent_profile.rs:1 | Identité/alias/instructions conservés ; préférences visuelles et attention UI sorties. |
| artifact_blob_store | conserver | lib.rs:2 ; artifact_blob_store.rs:1 | Blobs privés adressés par contenu ; indépendants du renderer. |
| artifact_fetch | extraire | lib.rs:3 ; artifact_fetch.rs:1 | Collecte HTTP explicite hors noyau minimal ; aucune suppression de contenu déjà publié. |
| artifact_policy | conserver | lib.rs:4 ; artifact_policy.rs:1 | Bornes et autorisation des contenus échangés, pas préférences d'affichage. |
| artifact_service | extraire | lib.rs:5 ; artifact_service.rs:1 | Publication atomique conservée ; dissocier le projet UI sans élargir les droits. |
| artifact_store | conserver | lib.rs:6 ; artifact_store.rs:1 | Références/versions immuables nécessaires aux échanges. |
| artifact_types | extraire | lib.rs:7 ; artifact_types.rs:1 | Contrats de contenu gardés ; séparer modèles de prévisualisation. |
| attach | conserver | lib.rs:8 ; attach.rs:1 | Observation/curseur/fraîcheur sans GUI. |
| build_identity | conserver | lib.rs:9 ; build_identity.rs:1 | Empreinte exacte des binaires et invalidation Git. |
| build_info | conserver | lib.rs:10 ; build_info.rs:1 | Comparaison build client/daemon. |
| cli | extraire | lib.rs:11 ; cli.rs:1 | Façade communication/lifecycle gardée ; dispatch UI/projet retiré. |
| connection_channel | conserver | lib.rs:12 ; connection_channel.rs:1 | Canal borné partagé ; pas d'adaptation sémantique. |
| control_settings | retirer | lib.rs:13 ; control_settings.rs:1 | Catalogue réglages serveur UI/projet ; ne contient pas le contrôle d'accès des messages. |
| daemon | extraire | lib.rs:14 ; daemon.rs:1 | Dispatch et autorisation communication ; sortir orchestration de projets. |
| desired_state | extraire | lib.rs:15 ; desired_state.rs:1 | Persistance des sessions gardée ; séparer références de projet et politiques. |
| disk_hygiene | extraire | lib.rs:16 ; disk_hygiene.rs:1 | Nettoyage de ressources propres gardé ; balayage des worktrees de l'hôte retiré. |
| disk_trend | retirer | lib.rs:17 ; disk_trend.rs:1 | Tendance disque et prédiction hors communication ; aucune suppression des quotas du journal. |
| execution_store | extraire | lib.rs:18 ; execution_store.rs:1 | Corrélation soumission/effet/session et reprise gardées ; séparer budgets d'orchestration. |
| fleet | conserver | lib.rs:22 ; fleet.rs:1 | Flotte et générations des processus gérés. |
| greffe_policy_refresh | extraire | lib.rs:23 ; greffe_policy_refresh.rs:1 | Politique de greffe extérieure ; préservation des vérifications d'autorisation de service. |
| human_inbox | extraire | lib.rs:24 ; human_inbox.rs:1 | Reçus/approbations existants à la frontière humaine gardés ; aucune auto-approbation de remplacement. |
| idempotency | conserver | lib.rs:25 ; idempotency.rs:1 | Réservation, canon, ACK, rejouabilité et scopes durables. |
| identity_migration | conserver | lib.rs:26 ; identity_migration.rs:1 | Migration d'identité stable, séparée du nom affiché. |
| ledger | conserver | lib.rs:27 ; ledger.rs:1 | Lecture neutre unique, globale/destinataire et bornée. |
| lifecycle | conserver | lib.rs:28 ; lifecycle.rs:1 | Préflight et ordres durables du cycle de vie. |
| managed_process | conserver | lib.rs:29 ; managed_process.rs:1 | Process group, stderr privé et attente bornée. |
| managed_supervisor | extraire | lib.rs:30 ; managed_supervisor.rs:1 | Supervision réelle gardée ; isoler continuations gouvernées non transport. |
| managers | conserver | lib.rs:31 ; managers.rs:1 | Annuaire/connexion/request routing. |
| mcp | extraire | lib.rs:35 ; mcp.rs:1 | Façade de la même autorité ; isoler outils projets sans casser send/reply/publish. |
| mcp_identity | conserver | lib.rs:36 ; mcp_identity.rs:1 | Résolution vérifiée par appel et scope stable. |
| mission_projection | extraire | lib.rs:37 ; mission_projection.rs:1 | Lecture d'état Maicie extérieure ; retirer dépendance au verdict métier dans wrapper. |
| project_policy | extraire | lib.rs:38 ; project_policy.rs:1 | Politiques de projet sorties ; contrôles de chemins des contenus gardés ailleurs avant coupe. |
| project_runtime | retirer | lib.rs:39 ; project_runtime.rs:1 | Docker/ingress du runtime de projet non requis pour une session gérée locale. |
| project_workspace | retirer | lib.rs:40 ; project_workspace.rs:1 | Exploration de dossiers pour la GUI. |
| reaper | conserver | lib.rs:41 ; reaper.rs:1 | Nettoyage des propriétaires/processus morts, borné. |
| receipt_store | conserver | lib.rs:42 ; receipt_store.rs:1 | Reçus Seen/Acked côté wrapper, au cœur de l'injection unique. |
| recovery_trace | conserver | lib.rs:43 ; recovery_trace.rs:1 | Diagnostic durable de reconnexion, sans déduire une réussite. |
| referent_control | extraire | lib.rs:44 ; referent_control.rs:1 | Commandes humaines attestées gardées ; plafond d'objectifs et focus métier extérieurs. |
| registry | conserver | lib.rs:45 ; registry.rs:1 | Définitions figées, capacités, droits/facturation et commandes absolues. |
| reprise | extraire | lib.rs:46 ; reprise.rs:1 | Carte transport gardée ; liens greffe et GUI seulement comme consommateurs extérieurs. |
| runtime | conserver | lib.rs:47 ; runtime.rs:1 | Faits d'observation fournisseur ; ce module n'est PAS la configuration des chemins. |
| store | extraire | lib.rs:48 ; store.rs:1 | Transactions transport gardées ; tables projets isolées, jamais retirées en bloc. |
| test_sync | conserver | lib.rs:50 ; test_sync.rs:1 | Barrières test-support ; no-op sans activation explicite. |
| ui | retirer | lib.rs:51 ; ui.rs:1 | Relais HTTP et rendu, sans retirer leurs sources de données communicables. |
| wrapper | extraire | lib.rs:52 ; wrapper.rs:1 | Session/pilote/journal/reçus gardés ; sortir vérifications Maicie et runtime Docker. |

## Tous les modules core/transport (22)

| Module | Disposition | Déclaration vérifiée |
|---|---|---|
| bridget-core/circuit_breaker | conserver : limitation des boucles | crates/bridget-core/src/lib.rs:5 |
| bridget-core/dedup | conserver : dédup historique | crates/bridget-core/src/lib.rs:6 |
| bridget-core/envelope | conserver : enveloppe et garde | crates/bridget-core/src/lib.rs:7 |
| bridget-core/execution | extraire : faits d'exécution, sans politique métier | crates/bridget-core/src/lib.rs:8 |
| bridget-core/host | conserver : hôte attesté | crates/bridget-core/src/lib.rs:9 |
| bridget-core/message | conserver : corrélation et références | crates/bridget-core/src/lib.rs:10 |
| bridget-core/router | conserver : routage et refus | crates/bridget-core/src/lib.rs:11 |
| bridget-core/text_guards | conserver : validation des données/affichage terminal | crates/bridget-core/src/lib.rs:12 |
| bridget-transport/acp | conserver : pilote ACP | crates/bridget-transport/src/lib.rs:5 |
| bridget-transport/act_kind | conserver : schéma de journal fermé | crates/bridget-transport/src/lib.rs:6 |
| bridget-transport/claude_provider_session | conserver : reprise réelle fournisseur | crates/bridget-transport/src/lib.rs:7 |
| bridget-transport/claude_stream_json | conserver : pilote natif | crates/bridget-transport/src/lib.rs:8 |
| bridget-transport/codex_app_server | conserver : pilote natif | crates/bridget-transport/src/lib.rs:9 |
| bridget-transport/fsutil | conserver : créations privées/atomiques | crates/bridget-transport/src/lib.rs:10 |
| bridget-transport/greffe_authorization | extraire : garde de service conservée, politique métier externe | crates/bridget-transport/src/lib.rs:11 |
| bridget-transport/greffe_policy_refresh | extraire : mécanisme de politique extérieure, pas nouvelle autorité | crates/bridget-transport/src/lib.rs:12 |
| bridget-transport/journal | conserver : append durable, raw et séquences | crates/bridget-transport/src/lib.rs:13 |
| bridget-transport/managed_session | conserver : contrat commun corrigé | crates/bridget-transport/src/lib.rs:14 |
| bridget-transport/protocol | extraire : familles de contrat séparées, bytes préservés | crates/bridget-transport/src/lib.rs:15 |
| bridget-transport/refusals | conserver : refus typés | crates/bridget-transport/src/lib.rs:16 |
| bridget-transport/tmux | retirer de la voie principale après preuve native ; aucune dépendance requise | crates/bridget-transport/src/lib.rs:17 |
| bridget-transport/transport | conserver : interface de livraison | crates/bridget-transport/src/lib.rs:18 |

## Coutures critiques avec consommateurs constatés

- **Identité et présentation** : mcp_identity.rs:116–123 résout BRIDGET_AGENT_ID_FILE et BRIDGET_AGENT_INSTANCE_ID puis les marqueurs du home. Ne pas rétablir la vieille identité fondée sur le nom. agent_profile.rs:173–225 mélange identités et préférences d'attention/UI.
- **Maicie dans wrapper** : wrapper.rs:27 importe mission_projection en production ; ses imports directs maicie:5606+ sont dans les tests. Retirer seulement la dépendance Cargo échouera aussi à compiler ces tests ; les scénarios transport doivent être portés, pas ignorés.
- **Runtime de projet** : wrapper.rs:1680–1700 lit les bindings d'environnement du conteneur ; daemon.rs:2812–2991 et :5275–5439 font le préflight Docker. Les variables fournisseur autorisées et la garde de facturation ne sont pas ce runtime.
- **Transaction partagée** : idempotency.rs:2403–2451 ouvre une transaction IMMEDIATE et appelle mark_answered_in_transaction pour l'ACK. Le ledger et les faits de clôture ne peuvent pas être déplacés vers un callback non transactionnel.
- **Journal** : managed_session.rs:76 porte raw/provenance ; protocol.rs:5113 teste JournalReady ; le gate d'attach ne doit pas redevenir un test de marque de fournisseur.
- **Contenu échangé** : artifact_blob_store.rs:1–9 et artifact_store.rs:1–9 sont des magasins de contenu privés ; artifact_service.rs:1–5 s'appuie sur identité/projet/conversation déjà attestés. Séparer le projet sans inventer une visibilité globale.
- **Projections** : ledger.rs est la lecture neutre ; ui.rs la consomme mais ne doit plus posséder l'état. mission_projection.rs lit un instantané JSON, pas le store métier directement : préserver cette frontière pour les consommateurs extérieurs.

## Commandes : disposition du dispatch réel

Source : crates/bridget-daemon/src/cli.rs:129–174, aliases interactifs :101 et wrapper générique :106.

| Conserver | Extraire (partie utile seulement) | Retirer du noyau |
|---|---|---|
| daemon, managed-bootstrap, managed-wrapper, mcp, attach, spawn, stop, relaunch, decommission, adopt-stopped, send, cancel, requests, rename, runtime, identity, dnd, hook, install-hooks, reply, who, agents, discover, status, ledger, version/help | guichet (surface de service, pas son traitement métier), domain (identité/routage vs projet), control/inbox (approbations et sécurité vs orchestration), reprise/reaper/cleanup (ressources propres uniquement), aliases interactifs et -- (sans tmux requis) | ui, project-runtime, project-round, managed-runtime-wrapper, managed-runtime-stop |

MCP : src/mcp.rs:1643 publication de contenu, :1679 send, :1697 who, :1706 ledger. Les outils guichet dynamiques doivent conserver leur négociation sans embarquer l'application Maicie. La publication d'un document n'est pas son exécution HTML.

## Chemins et bornes : constat avant changement

- daemon.rs:612–630 : défaut socket/DB/log sous dirs_cache ; fenêtres 180 s, limite de circuit 8, quarantaine 3600 s, rétention 7 jours.
- daemon.rs:649–676 : HOME/.cache/bridget, **mkdir comme effet de bord**, simple warning de permissions ; fallback /tmp/bridget si HOME absent. À remplacer en T007 par résolution indépendante et refus sûr, pas à appeler pour « juste voir le chemin ».
- daemon.rs:636–695 : BRIDGET_ARTIFACT_ROOT et racines XDG/macOS distinctes du cache ; doivent suivre l'isolation elles aussi.
- cli.rs:37 : borne de corps 10 000 bytes via String::len(), alors que le diagnostic parle de caractères. Ne pas élargir le canon en « corrigeant le libellé » ; tester Unicode à la borne.
- mcp.rs:24–26 : MCP 2025-06-18, budget global 10 s, huit appels ; identité par appel.
- scripts/federate-ssh.sh:34 : socket historique imposée ; :39 keepalive/forwarding ; :55 **rm de socket distante à l'installation**. Script interdit sur la flotte pour les essais 089.
- protocol.rs:120–141 : client/service v1, coordination events v1 et stream v2, contrôle/inbox v1. Les versions de projets ne deviennent pas celles de la communication.
- scripts/deploy-remote.sh : déploiement et configuration persistants ; conservation de la voie client-only, mais pas d'exécution avant préflight et namespace dédiés.

## Tables et migrations — inventaire des autorités

Toutes les sources déclarant CREATE TABLE dans le daemon sont : store.rs, idempotency.rs, execution_store.rs, artifact_store.rs, agent_profile.rs, human_inbox.rs et referent_control.rs. Aucun de ces stores n'est ouvert pour cet inventaire.

| Source et emplacement | Tables / familles | Disposition |
|---|---|---|
| store.rs:547–656 | ledger, tracked_requests, request_events, guichet_requests, guichet_lifecycle_events, guichet_coordination_events, guichet_coordination_stream_state | Conserver transactions et bytes/curseurs. |
| store.rs:657 | usage_samples | Conserver les faits de consommation exposés, sans politique automatique. |
| store.rs:671–779 | project_bindings, project_system_roles, project_system_dogfooding, project_system_worktree_leases, project_audit_events, project_binding_attempts, project_admin_attempts, project_round_policies, project_round_commands | Sortir le runtime/projet ; préserver les références requises tant que les clés étrangères ne sont pas migrées. |
| idempotency.rs:506–637 | idempotency_records, send_deliveries, supervisor_identity, spawn_commands, idempotency_schema_migrations, tracked_requests, ledger, agent_links, agent_link_events, delegated_runtime_events | Conserver ; ledger/requests ont deux chemins d'initialisation à tester ensemble. |
| idempotency.rs:829–837 | orphan_emitter_notices, send_delivery_execution_links | Conserver terminaux/corrélations ; ne pas rendre l'orphelin rejouable par erreur. |
| execution_store.rs:1522–1598 | execution_schema_migrations, work_submissions, executions, provider_bindings, message_correlations, execution_control_commands, execution_queue, execution_continuations, execution_usage_totals, execution_continuation_reservations, control_pause_interruptions | Extraire les politiques ; conserver les faits, la reprise fournisseur et la pause attestée. |
| artifact_store.rs:187–285 | artifacts, artifact_versions, artifact_sources, artifact_blobs, artifact_version_blobs, artifact_publications, artifact_references, artifact_metric_counters | Conserver le stockage/référence du contenu sans renderer. |
| agent_profile.rs:173–271 | agent_identities, agent_profiles, agent_profile_labels, agent_profile_applications, attention_events, client_notification_preferences, client_attention_state, identity_migration_aliases | Identité gardée ; personnalisation/attention UI séparées. |
| human_inbox.rs:55 | human_inbox | Garder la frontière humaine attestée ; traitement métier externe. |
| referent_control.rs:146–179 | control_state, control_events, control_focus_projection | Séparer pause/autorisations des objectifs/focus métiers ; aucun bypass à la suppression. |

Les tables transitoires de migration (suffixes _v2, _v4, _v088, _next) et copies dans les tests ne sont pas de nouvelles autorités. Idempotency utilise son journal idempotency_schema_migrations, ExecutionStore le sien execution_schema_migrations ; Store::open appelle aussi ensure_schema de contrôle/inbox (store.rs:540–541). Il n'existe donc pas une unique version SQLite à décrémenter pour « revenir au noyau ».

## Points qui changent le plan sans changer le périmètre

1. T007 doit toucher **daemon.rs (DaemonConfig)** et la résolution des chemins des clients, pas seulement runtime.rs qui représente les faits fournisseur.
2. T010 ne peut pas retirer tous les modules artifact_* : les références de documents sont une communication. Le renderer est distinct.
3. Les nouveaux tests doivent **réutiliser les tests existants** quand ils portent déjà l'oracle demandé ; le préfixe core_089 ne justifie pas des duplications.
4. Le corpus 012 documentaire parle encore de RoleHello et de OutcomeUnknown « en vol » ; le code épinglé utilise RoleHandshake et distingue les statuts de façade. On gèle les octets attestés par les tests du code actuel, pas cette prose historique comme si elle était encore exacte.
5. La fixture provider-contracts/codex-0.150.1.jsonl porte jsonrpc alors que le pilote teste son absence sur le fil (codex_app_server.rs:5235). provider_contract_test.rs vérifie des noms de méthode et capacités synthétiques, pas la conformité d'une émission réelle. Elle est conservée comme fixture historique de forme, **pas promue en canon de pilote**.
6. Le canon d'envoi vit actuellement dans daemon.rs:7479–7540 ; le scope de la sonde réutilise mcp::issuer_scope à :14861. T008 doit déplacer les helpers au bon endroit sans en créer une deuxième version. Le sous-module protocol/project_profile.rs (protocol.rs:12) est une famille de projet à séparer, pas un module racine supplémentaire oublié.

## Preuve de couverture de l'inventaire

Les 45 déclarations mod du daemon et les 22 de core/transport ont chacune une disposition dans les tables. La liste est comparée aux trois lib.rs ; ajout inconnu = inventaire à compléter. Ceci ne prouve pas l'absence de toutes les dépendances transitives : le graphe du paquet extrait sera testé en T013.

Première caractérisation sûre : cargo test --offline -p bridget-transport --lib protocol:: -- --test-threads=4 → **65/65**, compilation 9,73 s, tests 0,03 s. Aucun daemon ni fournisseur lancé. La suite complète et les scénarios interserveurs restent à exécuter.
