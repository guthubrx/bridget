# Audit de réutilisation de l'existant - Registre et identité des projets

## Decision

Statut: PASS contre `main` 04fd9c6, REVALIDATION REQUIRED si la tête change
Date: 2026-08-30
Feature dir: /home/moi/bridget-referent/.worktrees/session-065-registre-identite-projets/specs/065-registre-identite-projets

Conclusion courte: aucun registre général de projets n'existe. Le domaine
Bridget et `review_project` Maicie sont proches mais n'ont ni la même autorité
ni le même cycle de vie. Le plan étend les stores, outboxes, contrats et
lancements existants sans créer de service ou de crate supplémentaire.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 15 |
| Items audites | 15 |
| Reutilisations deja prevues | 12 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 6 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Commande idempotente Maicie | outbox et résultats durables | `plugins/maicie/src/store.rs` (`command_id`, outbox) | Étendre la saga, ne pas ajouter un coordinateur. |
| Identité métier | domaine Maicie | `plugins/maicie/src/domain.rs` | Ajouter l'entité dans le domaine existant. |
| Persistance métier | store SQLite Maicie | `plugins/maicie/src/store.rs` | Réutiliser migrations et permissions privées. |
| Contrat public | `WrapperToDaemon` | `crates/bridget-transport/src/protocol.rs` | Ajouter commandes et issues versionnées. |
| Persistance technique | store SQLite Bridget | `crates/bridget-daemon/src/store.rs` | Ajouter liaisons et historique dans le store existant. |
| Répertoire de lancement | `SpawnOrder.cwd` | `crates/bridget-daemon/src/fleet.rs` (`SpawnOrder`) | Ajouter la référence projet optionnelle. |
| Validation avant spawn | `prepare_spawn_parts` | `crates/bridget-daemon/src/lifecycle.rs` | Vérifier la liaison avant le chemin provider. |
| Génération durable | `SpawnLease` et définition figée | `crates/bridget-daemon/src/fleet.rs` | Épingler project_id et génération de liaison. |
| Environnement minimal | `build_environment` | `crates/bridget-daemon/src/lifecycle.rs` | Aucun changement de secret en 065. |
| Projection de fraîcheur | runtime Maicie existant | `plugins/maicie/src/runtime.rs` | Réutiliser Fresh/Gap/Ended/Unavailable. |
| Validation de racine | `ReviewProjectConfig` | `plugins/maicie/src/config.rs` | Réutiliser bornes lexicales, pas l'entité mono-projet. |
| Classification historique | `derive_domain` | `crates/bridget-daemon/src/wrapper.rs` | Conserver séparément, sans promotion en identité. |
| Filtrage flotte | filtre domain existant | `README.md` (`--domain`) | Ajouter un filtre project distinct. |
| Audit mutation | événements et raisons structurés | `crates/bridget-transport/src/greffe_authorization.rs` | Réutiliser le style, sans réutiliser l'autorité greffe. |
| Incidents runtime délégués | `DelegatedRuntimeEventRecord` et frame cursée | `crates/bridget-daemon/src/idempotency.rs`, `crates/bridget-transport/src/protocol.rs` | Ajouter ProjectReference aux faits, rejeux et accusés existants. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| ProjectIdentity | `ReviewProjectConfig.project_id` | `plugins/maicie/src/config.rs` (`ReviewProjectConfig`) | Créer l'entité générale; migrer explicitement le cas revue. |
| ProjectBinding | `domain` dérivé | `crates/bridget-daemon/src/wrapper.rs` (`derive_domain`) | Garder domain comme métadonnée, ne pas l'étendre en autorité. |
| Liaison racine | `SpawnOrder.cwd` | `crates/bridget-daemon/src/fleet.rs` (`SpawnOrder`) | Réutiliser le chemin par lancement mais persister la liaison séparément. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucune | Aucun équivalent 1:1 | N/A | Maintenir l'audit à l'implémentation. |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/home/moi/.speckit/constitution.md` | Réutiliser, isoler et rendre réversible | Worktree, migrations additives, backend host. |
| `/home/moi/.speckit/ref/speckit-workflow.md` | Statut Draft tant que non approuvé | Aucun statut Ready automatique. |
| `/home/moi/.speckit/ref/agent-orchestration.md` | Mandat et corrélation explicites | project_id opaque dans les contrats. |
| `/home/moi/.speckit/ref/standards-tests.md` | Tests techniques et Gherkin | Scénarios 065 et tests Rust. |
| `/home/moi/.speckit/ref/standards-observability.md` | Logs, métriques, corrélation | État de saga observable sans labels non bornés. |
| `/home/moi/.speckit/ref/code-quality-details.md` | Validation aux frontières | Canonicalisation et raisons structurées. |
| `/home/moi/.speckit/ref/adversarial-review.md` | Revue hostile des chemins et de l'état partagé | Crash, symlinks, collisions et non-destruction testés. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-046 | Connexion distincte de l'activité | Projet distinct du domaine et de la présence. |
| SPEC-048 | Identité d'instance daemon | Épingler la génération technique. |
| SPEC-052 | Délégation corrélée et rejeu | Réutiliser références opaques et outbox. |
| SPEC-063 | Pilotage humain | Ne pas perturber le chemin de remise. |
| SPEC-064 | Frontière mission/exécution | ProjectIdentity Maicie, ProjectBinding Bridget. |
| SPEC-068 | Incidents runtime délégués cursés | Propager ProjectReference dans liens, événements, frames, rejeu et accusé. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n "derive_domain|domain" crates/bridget-daemon/src/wrapper.rs` | domaine | Métadonnée dérivée, pas registre. |
| `rg -n "review_project|repository_root" plugins/maicie/src` | Maicie | Configuration mono-projet de revue. |
| `rg -n "SpawnOrder|cwd" crates/bridget-*` | lancement | Chemin absolu déjà durable par commande. |
| `rg -n "CREATE TABLE|Store" crates/bridget-daemon/src/store.rs` | persistance | Store SQLite extensible. |
| `rg -n "outbox|command_id" plugins/maicie/src/store.rs` | saga | Rejeu durable existant. |
| `rg -n "project_id|ProjectBinding" crates plugins` | codebase | Aucun registre général 1:1. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Domain contre project_id | créer distinct | Domain classe; project_id identifie. | 2026-08-29 |
| review_project | migrer explicitement | Config de revue trop étroite pour le registre. | 2026-08-29 |
| Nouveau service de registre | ne pas créer | Les deux stores et le contrat public suffisent. | 2026-08-29 |
| Enregistrement automatique | refuser | Risque de fausse identité et scope implicite. | 2026-08-29 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Complément cible RC8

La comparaison avec la tête `main` observée au 2026-08-30 montre que
`ExecutionSnapshot`, `DesiredEquipier`, `SpawnOrder`, `SpawnLease`, les ordres
Maicie persistés, `ExecutionReference` et `ExecutionProjection` sont des
consommateurs réels de ProjectReference. Le plan et T024/T026 les énumèrent
désormais explicitement.

Le flux `ServiceRequest` existant est orienté Bridget vers Maicie. Le registre
utilisera des variantes locales dédiées dans le protocole existant plutôt que
de créer un second transport ou de détourner cette direction. Aucun nouveau
crate, daemon ou store n'est justifié.

Ce passage ne vaut pas audit de toute future tête d'implémentation: T003 impose
toujours un rejeu si `main` avance après `04fd9c6`.

## Rejeu T003 sur main a63cf97

Le worktree d'implémentation est rebased exactement sur `main` à `a63cf97`.
Le delta depuis `ae2b71a`, tête contrôlée avant la création du worktree, ne
contient que les commits UI `3364d90` et `a63cf97`; aucun fichier de contrat,
store, daemon ou transport de SPEC-065 n'a changé dans ce delta.

L'inventaire contre la tête courante confirme les porteurs déjà identifiés:
`SpawnOrder`, `SpawnLease`, `DesiredEquipier`, `ExecutionSnapshot`,
`ExecutionReference`, `ExecutionProjection`, `AgentLinkRecord`,
`AgentLinkEvent`, `DelegatedRuntimeEventInput`, `DelegatedRuntimeEventRecord`,
`AgentLinkEventFrame` et `DelegatedRuntimeEventFrame`. Les routes cursées et
les accusés SPEC-068 restent dans `protocol.rs`, `idempotency.rs`, `fleet.rs`
et `daemon.rs`.

Une recherche des symboles `ProjectIdentity`, `ProjectBinding`,
`ProjectAuditEvent`, `ProjectRegistryRequest`, `ProjectRegistryOutcome` et
`project_registry_v1` sur `crates/` et `plugins/` ne retourne aucun équivalent.
Le registre général, la liaison, l'audit et le contrat dédié restent donc des
créations justifiées, tandis que leurs stores, transports et porteurs runtime
réutilisent les points d'extension existants. Aucun nouveau crate, daemon,
transport parallèle ou dépendance n'est justifié.

Le finding externe sur ProjectAuditEvent est fermé par un modèle explicite,
une persistance transactionnelle et les tâches T009, T014, T019, T021 et T023.
Les preuves du tableau utilisent maintenant chemins et symboles stables plutôt
que des numéros de ligne périssables.

## Rejeu T003 sur main 04fd9c6

Le worktree d'implémentation est rebased exactement sur `main` à `04fd9c6`.
Le delta depuis `a63cf97` ne concerne que SPEC-071 et l'interface daemon:
`crates/bridget-daemon/src/ui.rs`, ses assets et les documents SPEC-071. Aucun
store, protocole, transport, daemon de registre, domaine Maicie ou contrat de
SPEC-065 n'est modifié.

La recherche contre cette tête confirme que les porteurs à étendre pour T012 et
la suite restent les mêmes: outbox et commandes dans `plugins/maicie/src/store.rs`,
contrat dans `crates/bridget-transport/src/protocol.rs`, store Bridget,
`SpawnOrder`, `SpawnLease`, `DesiredEquipier`, `ExecutionSnapshot`,
`ExecutionReference`, `ExecutionProjection`, liens d'agents et événements
délégués. Aucun symbole du registre général de projet n'existe sur `main`.

La nouvelle identité d'exécution affichée par SPEC-071 est une métadonnée UI;
elle ne constitue ni une identité de projet ni une liaison de racine. T028
devra s'y raccorder lors de l'affichage, sans créer de second porteur. Aucun
nouveau crate, transport, store ou dépendance n'est justifié avant T012.
