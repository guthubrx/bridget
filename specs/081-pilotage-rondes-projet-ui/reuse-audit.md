# Audit de reutilisation de l'existant - pilotage des rondes par projet

## Decision

Statut: PASS
Date: 2026-08-31
Feature dir: `specs/081-pilotage-rondes-projet-ui`

Conclusion courte: Le backend de politique, la cadence, la génération, l'idempotence, la projection projets et le menu contextuel existent déjà. Le plan les étend de façon ciblée. Aucun service, table, scheduler, écran ou dépendance 1:1 n'est recréé.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 10 |
| Items audites | 10 |
| Reutilisations deja prevues | 8 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 2 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Contrat de lecture et mutation | `ProjectRoundRequest` | `crates/bridget-transport/src/protocol.rs:526` | Les quatre opérations fermées existent. |
| Projection de politique | `ProjectRoundProjection` | `crates/bridget-transport/src/protocol.rs:558` | Ajouter seulement les faits d'observabilité optionnels. |
| Cadence | `PROJECT_ROUND_INTERVAL_SECS` | `crates/bridget-transport/src/protocol.rs:584` | Aucune nouvelle constante UI. |
| Persistance | `project_round_policies` | `crates/bridget-daemon/src/store.rs:712` | Colonnes additives, aucune table nouvelle. |
| Mutation idempotente | `apply_project_round_mutation` | `crates/bridget-daemon/src/store.rs:1449` | Le relais ne touche pas SQLite directement. |
| Projection de projets | `read_projects` | `crates/bridget-daemon/src/ui.rs:2363` | Enrichissement par jointure O(p). |
| Menu du projet | `openProjectContextMenu` | `crates/bridget-daemon/assets/ui/app.js:7639` | Même accès trois points, clic droit et clavier. |
| Rafraîchissement confirmé | `refreshProjects` | `crates/bridget-daemon/assets/ui/app.js:7804` | Relire après succès, pas d'état optimiste. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Dernier passage | Ledger et messages `origin=routine` | `crates/bridget-daemon/src/daemon.rs:8660` | Ne pas analyser le texte ou le ledger; stocker seulement la projection fermée. |
| Négociation locale | Client `SendIdempotent` du relais | `crates/bridget-daemon/src/ui.rs:3210` | Reprendre le même handshake client avec la capacité ronde, sans helper générique prématuré. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun | Recherche complète ci-dessous | Aucune |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/home/moi/.speckit/constitution.md` Article III | cycle SpecKit complet | Tous les artefacts et gates sont produits. |
| `/home/moi/.speckit/constitution.md` Article XVIII | complexité non négociable | Jointure HashMap O(p), mutation O(1). |
| `/home/moi/.speckit/constitution.md` Article XIX | réutiliser avant de créer | Aucun second store, scheduler ou écran. |
| `/home/moi/.speckit/constitution.md` Article XX | responsabilité future | États fermés, migration et commandes vérifiables documentés. |
| `/home/moi/.speckit/ref/standards-tests.md` | traçabilité par ID de spec | Scénario et tests portent 081. |
| `/home/moi/.speckit/ref/standards-observability.md` | état observable sans secret | Dernier dispatch minimal et structuré. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-079 | politique par `project_id + binding_generation`, scheduler unique, désactivé par défaut | Autorité réutilisée sans modification de cadence. |
| SPEC-080 | colonne Projets, menu contextuel et routes relay versionnées | Surface UI étendue, pas de nouvelle navigation. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n "ProjectRound|project_round" crates specs/079-*` | transport, daemon, store, CLI, specs | contrat, table, mutation, dispatch et tests existants identifiés. |
| `rg -n "UiProjectListEntryV1|read_projects" crates/bridget-daemon/src/ui.rs` | relais UI | liste existante à enrichir. |
| `rg -n "openProjectContextMenu|refreshProjects" crates/bridget-daemon/assets/ui/app.js` | navigateur | menu et rafraîchissement confirmé existants. |
| `rg -n "project-context-menu" crates/bridget-daemon/assets/ui/theme.css` | styles | surface CSS existante. |
| `rg -n "ronde|menu contextuel" specs/079-* specs/080-*` | specs livrées | frontière métier et surface canonique confirmées. |
| `rg -n "CREATE TABLE|ALTER TABLE" crates/bridget-daemon/src/store.rs` | schéma | migration additive conforme aux patterns locaux. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Source du dernier passage | étendre la politique existante | évite analyse de texte et nouveau journal | 2026-08-31 |
| Lecture UI | enrichir `/v1/projects` | évite N+1 et état navigateur redondant | 2026-08-31 |
| Prochain passage | délai maximal, pas d'heure exacte | le timer effectif n'est pas attesté par le relais | 2026-08-31 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
