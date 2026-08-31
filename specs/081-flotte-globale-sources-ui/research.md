# Recherche - SPEC-081 Flotte globale et sources Desktop

**Date** : 2026-08-31  
**Base inspectée** : `origin/main` 29c81424b28d2607d871a0feed67e3941717e2f9

## Décisions vérifiées

| Sujet | Décision | Preuve |
|---|---|---|
| Agrégation | Desktop agrège localement les instantanés de chaque tunnel connecté. | Le host MCP est le point de contrôle de connexions isolées. Desktop possède déjà sessions et tunnels par profil. |
| Conversation | Une seule WebView distante reste ouverte, pour l'agent sélectionné. | Le relais fournit déjà la conversation et le registre limite volontairement à un panneau. |
| Coque visible | Sources et flotte sont rendues localement. La WebView enfant n'occupe que la zone conversation. | Le panneau enfant couvre aujourd'hui toute la fenêtre et cache le bouton Serveurs. |
| Identité | Une carte est identifiée par `source_id + agent_name`. | Les noms ne sont uniques qu'à l'intérieur d'une source. |
| Source locale | Découvrir dynamiquement le relais local avec la commande constante `bridget ui endpoint --json`, sans profil persistant ni jeton saisi. | Les anciens profils local exigeaient un jeton manuel et sont explicitement exclus du store actuel. Le relais actuel publie déjà un contrat endpoint versionné. |
| Filtres | Source et projet créent des chips indépendants. | On peut retirer séparément chaque restriction. |
| Tri | Les critères sont ordonnés, réordonnables et inversables. Le regroupement en découle. | Évite un état de regroupement contradictoire avec le tri. |
| Coordinateur | Pré-épingler seulement sur `agent_link.role = "coordinator"`. Les anciens agents non marqués restent manipulables manuellement. | Le code actuel n'offre aucun autre signal fiable. Une heuristique par nom serait fausse. |
| Chemins | Ne projeter que `project_id`, `display_name` et `state`. | `GET /v1/projects` porte `canonical_path`, inutile et non souhaitable dans la flotte globale. |
| Panne isolée | Une source défaillante reste visible mais n'affecte pas les autres. | Chaque tunnel est une frontière d'origine. |

## Réutilisation identifiée

| Élément | Emplacement | Usage |
|---|---|---|
| Sessions et tunnels | `apps/bridget-desktop/src-tauri/src/lib.rs` | Capturer les sources actives hors mutex avant les lectures réseau. |
| Transport HTTP local | `request_relay_json` dans `apps/bridget-desktop/src-tauri/src/lib.rs` | Lire `/v1/snapshot` et `/v1/projects`, sans pile HTTP nouvelle. |
| Encodage de relais | `apps/bridget-desktop/src-tauri/src/connection.rs` | Construire les chemins authentifiés et l'URL compacte côté Rust. |
| Endpoint local versionné | `bridget ui endpoint --json` et `RelayEndpoint` | Découvrir `Cet ordinateur` sans sauvegarder un profil local, puis réutiliser le transport loopback existant. |
| Snapshot UI | `crates/bridget-daemon/src/ui.rs` | Réutiliser agents, alertes, projet et activité. |
| Panneau isolé | `apps/bridget-desktop/src-tauri/src/panels.rs` | Conserver un seul panneau et déplacer seulement sa géométrie. |
| Agent ciblé | `crates/bridget-daemon/assets/ui/app.js` | Réutiliser `agent` et ajouter `desktop_shell=1`. |
| Préférences locales | `apps/bridget-desktop/src-tauri/src/preferences_store.rs` | Garder épingles et tris localement, sans serveur. |

## Recherche externe

| Source | Enseignement | Application |
|---|---|---|
| [MCP Architecture](https://modelcontextprotocol.io/specification/2025-06-18/architecture/index) | Le host isole des connexions client-serveur multiples. | Desktop agrège les lectures, chaque action reste dans sa source. |
| [Microsoft - Guidelines for Human-AI Interaction](https://www.microsoft.com/en-us/research/publication/guidelines-for-human-ai-interaction/) | L'état et l'incertitude doivent être visibles. | Une source indisponible est explicite, pas fusionnée à une source saine. |
| [Microsoft - Tools for Thought](https://www.microsoft.com/en-us/research/publication/tools-for-thought/) | Le contexte implicite accroît la charge cognitive. | Source et projet sont textuels sur chaque carte, pas seulement colorés. |

## Options écartées

- Plusieurs WebViews : réintroduirait le défaut de recouvrement explicitement corrigé.
- Une agrégation dans une WebView distante : elle ne doit jamais recevoir les privilèges Desktop ou les données d'autres sources.
- Le nom `coordinateur` comme signal : non fiable.
- La transmission de `canonical_path` au JavaScript Desktop : sans nécessité fonctionnelle.
