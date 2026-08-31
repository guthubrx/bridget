# Audit de réutilisation de l'existant - Environnement Docker partagé par projet

## Decision

Statut: PASS contre `main` 74234641fe04293c007717b8f1e3879823f5d382, REVALIDATION REQUIRED si la tête change
Date: 2026-08-30
Feature dir: /home/moi/bridget-referent/.worktrees/session-066-environnement-partage-projet/specs/066-environnement-partage-projet

Conclusion courte: aucun backend conteneur n'existe dans Bridget. Le cycle de
vie de processus, le cwd, l'environnement filtré, la flotte, les générations et
la reconnexion existent et doivent rester les autorités. Un seul nouveau module
de machine d'état runtime est justifié; Docker CLI évite une dépendance.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 17 |
| Items audites | 17 |
| Reutilisations deja prevues | 13 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Cwd et identité de lancement | `ManagedLaunch` | `crates/bridget-daemon/src/managed_process.rs` | Conserver identité/env/cwd, changer l'exécuteur. |
| Spawn process | `spawn_managed_bootstrap` | `crates/bridget-daemon/src/managed_process.rs` | Réutiliser bornes et supervision pour host. |
| Env minimal | `build_environment` | `crates/bridget-daemon/src/lifecycle.rs` | Pas de secret supplémentaire en 066. |
| Admission | `prepare_spawn_parts` | `crates/bridget-daemon/src/lifecycle.rs` | Brancher backend après validation projet. |
| Commande durable | `SpawnOrder.cwd` | `crates/bridget-daemon/src/fleet.rs` (`SpawnOrder`) | Ajouter project_id/backend figés. |
| Saga de flotte | `SpawnOrder`, `SpawnLease` | `crates/bridget-daemon/src/fleet.rs` | Une génération reste l'autorité d'un agent. |
| État désiré | `DesiredEquipier` | `crates/bridget-daemon/src/desired_state.rs` | Ajouter la référence environnement. |
| Reconnexion wrapper | boucle socket | `crates/bridget-daemon/src/wrapper.rs` | Réutiliser via runtime ingress dédié. |
| Contrats providers | `ManagedSession` | `crates/bridget-transport/src/managed_session.rs` | Aucun adaptateur provider spécifique Docker. |
| Autorité des exécutables Docker | aucune autorité existante adaptée | [AgentDefinition.command] est un chemin hôte | Créer dans la policy runtime un mapping fermé type vers exécutable interne; aucun fallback hôte. |
| Arrêt/reaper | managed supervisor et reaper | `crates/bridget-daemon/src/reaper.rs` | Étendre la corrélation, pas un second superviseur complet. |
| Persistance | Store Bridget | `crates/bridget-daemon/src/store.rs` | ProjectEnvironment dans le store existant. |
| Projection UI | UI existante | `crates/bridget-daemon/src/ui.rs` | Étendre status, pas nouveau dashboard. |
| Projet | ProjectBinding SPEC-065 | `specs/065-registre-identite-projets/data-model.md` | Autorité du backend et de la racine. |
| Pilotage | SPEC-063 | `specs/063-interruption-pilotage-tour-humain/plan.md` | Même transport et mêmes oracles. |
| Exécution | SPEC-064 | `specs/064-plan-controle-bridget-maicie/plan.md` | ProjectEnvironment ne remplace pas Execution. |
| Incidents délégués | store, curseur, frame et accusé SPEC-068 | `crates/bridget-daemon/src/idempotency.rs`, `crates/bridget-transport/src/protocol.rs` | Conserver exactement la même route sous Docker. |
| Commandes externes | `std::process::Command` | `crates/bridget-daemon/src/managed_process.rs` | Docker CLI sans shell ni dépendance. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| ProjectEnvironment | DesiredFleet | `crates/bridget-daemon/src/desired_state.rs` | Lier les états, ne pas confondre conteneur et agents. |
| Docker exec suivi | ManagedProcess | `crates/bridget-daemon/src/managed_process.rs` | Réutiliser identité et supervision, ajouter l'attestation exec. |
| Runtime ingress | socket wrapper actuelle | `crates/bridget-daemon/src/wrapper.rs` (`socket_path`) | Ajouter un socket dédié, ne pas monter le cache parent. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucune | Aucun backend conteneur 1:1 | N/A | Maintenir le gate avant tout nouveau helper. |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/home/moi/.speckit/constitution.md` | Minimalisme, isolation, réversibilité | Backend opt-in et host conservé. |
| `/home/moi/.speckit/ref/standards-infra-k8s.md` | Infra déclarative | Politique et image versionnées, pas de kubectl. |
| `/home/moi/.speckit/ref/standards-tests.md` | Gherkin et tests techniques | Feature 066 et tests Rust/Docker. |
| `/home/moi/.speckit/ref/standards-observability.md` | Logs, métriques, traces | Machine d'état et raisons observables. |
| `/home/moi/.speckit/ref/code-quality-details.md` | Validation frontières et erreurs | Parse strict de Docker inspect. |
| `/home/moi/.speckit/ref/adversarial-review.md` | Review hostile infrastructure | Montages, privilèges, réseau et health réels. |
| `/home/moi/.speckit/ref/agent-orchestration.md` | Agents coordonnés | Partage projet, worktrees seulement par sujet. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-063 | Steering/interruption dans transport | Parité host/docker obligatoire. |
| SPEC-064 | Exécution et génération | Ne pas créer une seconde identité d'exécution. |
| SPEC-065 | Identité et liaison projet | Précondition du backend. |
| SPEC-068 | Incident runtime délégué durable | Parité host/docker du store, rejeu, remise et accusé. |
| SPEC-009 | Saga de spawn | Réutiliser réservation, lease et rollback. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n "docker|container" crates plugins` | codebase | Aucun runtime Docker productif. |
| `rg -n "ManagedLaunch|current_dir|env_clear" crates/bridget-daemon` | processus | Seam de lancement existante. |
| `rg -n "SpawnOrder|SpawnLease|DesiredEquipier" crates/bridget-daemon` | flotte | Saga et génération existantes. |
| `rg -n "socket_path" crates/bridget-daemon` | transport local | Socket dans cache partagé, montage parent à éviter. |
| `rg -n "pass_env|forbidden_env" crates/bridget-daemon` | environnement | Allowlist et garde de clés existantes. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Granularité | un conteneur par projet | Frontière utile, collaboration simple. | 2026-08-29 |
| SDK Docker | ne pas ajouter | CLI suffisante et déjà compatible avec Command. | 2026-08-29 |
| Nouveau module runtime | créer | Machine d'état avec plus de trois consommateurs. | 2026-08-29 |
| Socket cache complète | ne pas monter | Exposerait des états inter-projets. | 2026-08-29 |
| Secret en 066 | refuser | Séparer preuve runtime et propagation sensible. | 2026-08-29 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Complément cible RC8

La cible observée utilise encore HOME pour localiser le socket Bridget et
plusieurs fichiers internes du wrapper. Ce comportement est réutilisable pour
le backend host mais ne convient pas au conteneur sans montage du home. Le plan
prévoit donc un champ de socket explicite et un HOME/XDG projet, tout en gardant
le wrapper, la reconnexion et ManagedLaunch existants.

Docker Engine 29.6.2 est joignable par le compte UID/GID 1002 et ne se déclare
pas rootless. Le plan n'ajoute pas un second moteur ni une promesse d'isolation
hostile. Le préflight doit prouver les droits réels de cet utilisateur sur les
montages fixtures avant activation.

La fixture locale peut être épinglée par l'image ID écrite avec `--iidfile`;
aucun registre externe, SDK Docker ou tag autoritatif n'est requis.

## Rejeu sur main 74234641fe04293c007717b8f1e3879823f5d382 après SPEC-068

SPEC-068 ajoute une route durable d'incident enfant vers parent avec curseur,
rejeu après reconnexion et accusé idempotent. Le backend Docker doit réutiliser
ce canal, pas créer une seconde remontée. FR-022, le plan, T017 et T026 couvrent
désormais explicitement warning, failed, ProjectReference, ordre et accusé.

Les preuves du tableau utilisent des chemins et symboles stables. T003 reste
obligatoire si la tête d'implémentation avance après `74234641fe04293c007717b8f1e3879823f5d382`.


## Revalidation sur la tête courante - 2026-08-30

Tête observée : main à aba60f03a72c5b599cb2f8f2e5723cdaea684bf2.

- Les structures DelegatedRuntimeEventFrame et le store des incidents
  délégués SPEC-068 sont présents sur main : la parité prévue reste
  réutilisable.
- Aucun ProjectEnvironment, ProjectRuntimePolicy, RuntimeIngress,
  environment_epoch ou module project_runtime.rs n'est présent sur main :
  la création prévue conserve sa justification, sans doublon détecté.
- SPEC-075 a des modifications non intégrées dans daemon.rs, lifecycle.rs,
  fleet.rs, protocol.rs, l'UI et les stores associés. SPEC-066 ne doit pas
  modifier ces surfaces avant son intégration.
- Main contient le fichier non suivi préexistant
  docs/REPRISE-SUPERVISEUR-CLAUDE-VERS-CODEX-20260829.yaml; il n'est ni lu
  comme entrée de la SPEC ni modifié.

## Rejeu T003 sur la tête d'implémentation

Tête auditée : `74234641fe04293c007717b8f1e3879823f5d382` (`main` et
`origin/main` identiques le 2026-08-30). Le worktree d'implémentation est
`session-066-environnement-partage-projet`, créé directement depuis ce commit.

| Item du plan | Existant vérifié | Preuve | Décision |
|---|---|---|---|
| Backend de liaison | `ProjectBackend` ne possède que `Host` | `crates/bridget-transport/src/protocol.rs:265` | Étendre l'enum de manière additive, préserver la désérialisation host. |
| Persistance de liaison | `project_bindings` est contraint à `backend = 'host'` | `crates/bridget-daemon/src/store.rs:552` | Migrer le schéma et conserver les enregistrements host existants. |
| Préparation de spawn | `prepare_spawn_parts` et `build_environment` | `crates/bridget-daemon/src/lifecycle.rs:384`, `crates/bridget-daemon/src/lifecycle.rs:462` | Réutiliser pour host, insérer la décision Docker après validation de liaison. |
| Supervision de processus | `ManagedLaunch` et bootstrap supervisé | `crates/bridget-daemon/src/managed_process.rs:127`, `crates/bridget-daemon/src/managed_process.rs:575` | Conserver identité, génération, stderr et arrêt; adapter uniquement l'exécuteur Docker. |
| Cycle de vie 075 | daemon, fleet et lifecycle intégrés à la tête | commit `7423464` | Étendre cette autorité, sans second superviseur ni second cycle de vie. |
| Socket wrapper | chemin actuellement dérivé de HOME | `crates/bridget-daemon/src/wrapper.rs:902` | Ajouter une configuration explicite seulement pour Docker; garder host inchangé. |
| Incidents délégués 068 | store et frames persistants | `crates/bridget-daemon/src/idempotency.rs:372`, `crates/bridget-transport/src/protocol.rs:1152` | Réutiliser le même canal, les mêmes curseurs et accusés. |
| Commandes externes | `std::process::Command` est déjà employé | `crates/bridget-daemon/src/lifecycle.rs:23` | Construire Docker en arguments, sans shell ni dépendance SDK. |
| Runtime Docker proposé | aucun `ProjectEnvironment`, `ProjectRuntimePolicy`, `RuntimeIngress`, `environment_epoch` ou module `project_runtime.rs` | recherche T003 sur `crates/` | Créer un seul module productif, justifié par ses consommateurs daemon, lifecycle, store, CLI/UI et tests. |

Aucune duplication évidente n'est observée. SPEC-075 est intégrée. Le fichier
non suivi présent dans le worktree principal est hors périmètre et n'est ni lu,
ni modifié, ni inclus dans ce worktree.

Verdict : **PASS documentaire, BLOCKED avant implémentation**. Après
intégration de SPEC-075 sur une tête main propre, T003 doit rejouer l'audit
complet avec les nouveaux symboles et lignes.
