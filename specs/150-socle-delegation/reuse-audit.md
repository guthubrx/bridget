# Audit de reutilisation de l'existant — 150-socle-delegation

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/150-socle-delegation

Conclusion courte: Le plan étend les modules existants et ne crée aucun
module. Deux réutilisations ont été ajoutées pendant l'audit. Le nom du champ
de délai suit `bridget_send` (`reply_timeout`). Le suivi de la reprise passe
par `control_pause_interruptions.resume_execution_id` et le CTE de
continuation existant. Aucune duplication évidente.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 15 |
| Items audites | 15 |
| Reutilisations deja prevues | 8 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Garde de pause à l'admission et au lancement | `admit_autonomous_effect` + enum `AutonomousEffect` | `crates/bridget-daemon/src/referent_control.rs:105-133` | ajout d'une variante, pas de nouvelle garde |
| Reprise d'une mission suspendue | chemin de reprise ADR 027 | `crates/bridget-daemon/src/daemon.rs:9675`, `execution_store.rs:1298-1440` | exclusion native levée seulement pour une pause |
| Trace de la pause | table `control_pause_interruptions` | `crates/bridget-daemon/src/execution_store.rs:1488-1505` | aucune table nouvelle pour la pause |
| Entretien horaire | `run_maintenance` | `crates/bridget-daemon/src/daemon.rs:7177-7240` | point d'appel de la purge |
| Purge par lots bornés | patron `MAX_ENTRIES_PER_PASS = 128` | `crates/bridget-daemon/src/disk_hygiene.rs:29` | même borne pour les arbres |
| Grâce de 30 jours | `purge_expired_sends` | `crates/bridget-daemon/src/daemon.rs:7169`, `send_delivery.rs:571` | même durée pour les pierres tombales |
| Génération de projection | `increment()` et méta de projection | `crates/bridget-daemon/src/delegation_lineage.rs:66-71` | nouvelle génération au lieu de seq+1 |
| Borne 4096 | contrats 148 et 149 | `specs/149-sous-agents-lineage/contracts/lineage.md` | valeur publiée réutilisée |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Champ `reply_timeout_secs` | paramètre `reply_timeout` de `bridget_send` | `crates/bridget-daemon/src/mcp.rs` (schéma `bridget_send`), MCP `reply_timeout` | Réutiliser le nom `reply_timeout` : appliqué |
| Suivi de la chaîne d'exécutions | `resume_execution_id` posé à la reprise ; CTE récursif `lineage` | `crates/bridget-daemon/src/execution_store.rs:1422`, `:818` | Réutiliser : `tick` lit `resume_execution_id` |
| Expiration idempotente | `IdempotencyExpired` (ADR 007) | `crates/bridget-daemon/src/idempotency.rs:872-876` | Non réutilisable sans `issued_at` : pierre tombale justifiée (research R3) |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | admission en comptage indexé, purge par lots |
| `~/.speckit/constitution.md` XIX | minimalisme | pas de module, pas d'enum de refus |
| `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` | garde unique par puits autonome | la délégation native devient un puits classé |
| `docs/decisions/007-socle-idempotence.md` | rejeu et expiration explicites | `request_expired` au lieu d'une nouvelle tâche |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart et gates |
| `AGENTS.md` (checkpoints 149) | livraison sans relance | aucune relance de service |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 148-identite-delegation | contrat `bridget_delegate`, rejeu par `request_id` | compatibilité des octets canoniques |
| 149-sous-agents-lineage | contrat Lineage, liste d'états fermée, 4096 | aucun nouvel état ; génération sur purge |
| 087 (pause) | interruption et reprise à la levée | étendue aux missions natives |
| 116 (entretien) | fil d'entretien horaire | purge branchée dessus |
| 089 (tests) | recette `umask 077` | gates de test |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -i tombstone` | `crates/` | aucun équivalent de pierre de clé de délégation |
| `rg "fn purge"` | `crates/` | `store.rs:425`, `disk_hygiene.rs:119-154` : patrons, pas de purge de délégation |
| `rg execution_continuations` | `crates/` | CTE `execution_store.rs:818`, `continuation_for` `:743` |
| `rg suspended` | `crates/` | `AttachView::suspended` (sans rapport) |
| `rg NativeDelegation` | `crates/` | variante de protocole, pas d'effet autonome |
| `rg reply_timeout_secs` | `crates/` | constantes sans rapport ; nom public existant `reply_timeout` |
| `rg spawn_base` | `crates/` | aucun |
| `rg "COUNT\(\*\) FROM native_delegations"` | `crates/` | aucun comptage existant |
| `rg IdempotencyExpired` | `crates/` | guichet et idempotence, liés à `issued_at` |
| `rg record_pause_interruption` | `crates/` | `execution_store.rs:1488` réutilisé |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Nom du champ de délai | reutiliser `reply_timeout` | cohérence avec `bridget_send` | 2026-10-11 |
| Suivi de la reprise | reutiliser `resume_execution_id` | lien déjà écrit à la reprise ADR 027 | 2026-10-11 |
| Retrait de la flotte avant purge | reutiliser `fleet.decommission` | `crates/bridget-daemon/src/fleet.rs:545`, contre-revue ronde 2 | 2026-10-11 |
| Lecture de la phase d'une remise | etendre `send_delivery.rs` d'une lecture explicite | `send_delivery()` masque les phases (`send_delivery.rs:311`) | 2026-10-11 |
| Pierre tombale | creer nouveau | pas d'`issued_at` dans `bridget_delegate` ; changer le contrat est hors périmètre | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
