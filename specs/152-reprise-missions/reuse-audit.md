# Audit de reutilisation de l'existant - 152-reprise-missions

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/152-reprise-missions

Conclusion courte: Le plan réutilise la reprise ADR 027, la chaîne de
continuations et la reprise de flotte. Il ne crée aucun module ni nouvel état.
Deux fonctions existantes sont séparées en deux (`native_child_env`,
`classify_stop`), chacune avec deux usages. Une seule table est nouvelle (reçus
de refus). Aucune duplication évidente.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 14 |
| Items audites | 14 |
| Reutilisations deja prevues | 9 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 6 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Reprise d'une mission après pause | `reconstruct_active_for_agent` et `resume_execution_id` | `crates/bridget-daemon/src/execution_store.rs:1298-1440`, `:1422` | message d'origine, donc même identifiant de mission |
| Suivi de la chaîne d'exécutions | CTE `lineage` du budget | `crates/bridget-daemon/src/execution_store.rs:818` | même requête, nouvelle racine |
| Trace de la pause | table `control_pause_interruptions` | `crates/bridget-daemon/src/execution_store.rs:1488-1505` | aucune table pour la pause |
| Garde de pause | `admit_autonomous_effect` | `crates/bridget-daemon/src/referent_control.rs:105-133` | une variante de plus |
| Reprise après panne | `prepare_recovery` et `reserve_managed_recoveries` | `crates/bridget-daemon/src/lifecycle.rs:313-339`, `crates/bridget-daemon/src/daemon.rs:3886-3935` | retrait de l'exclusion des enfants natifs |
| Boucle de reprise | `tick` à 1 s | `crates/bridget-daemon/src/daemon.rs:4694-4705` | pas de nouveau thread |
| Environnement natif de l'enfant | bloc de `spawn_task` | `crates/bridget-daemon/src/daemon/native_delegation.rs:585-603` | extrait en `native_child_env` |
| Arrêt d'un membre | `prepare_managed_stop`, `await_managed_stop` | `crates/bridget-daemon/src/daemon.rs:9255-9380` | classification extraite |
| Entretien et grâce de 30 jours | passage d'entretien de la 150 | `specs/150-socle-delegation/tasks.md` T011 | purge des reçus de refus au même endroit |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Gel du suivi de demande | paliers différés en pause (`ReminderNudge`) | `crates/bridget-daemon/src/daemon.rs:6853-6871` | Réutiliser la variable `paused` et le différé ; ajouter seulement `frozen_secs` et l'exclusion des demandes de mission |
| Reçus de refus | pierres tombales de la 150 | `specs/150-socle-delegation/data-model.md` | Non réutilisable : elles prouvent un arbre purgé, pas un refus. Table séparée justifiée (research R8) |
| Relance d'un enfant arrêté | `submit_relaunch_from_resolved` | `crates/bridget-daemon/src/lifecycle.rs:278` | Écarté tant que le test T004 ne l'exige pas |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | `latest_in_chain` O(c), `classify_stop` O(m) |
| `~/.speckit/constitution.md` XIX | minimalisme | pas d'état, pas de module, extractions à deux usages |
| `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` | garde unique, reprise des remises acceptées admise | `NativeMissionDispatch` ; résultat acquis toujours remis (H3) |
| `docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md` | suspension transférée à la 152 | périmètre de cette session |
| `specs/150-socle-delegation/adversarial-review-codex.md` | objections 1 à 4 et 7 | R1 à R5 |
| Mémoire `bridget-tests-umask-077-env-prive` | recette de test privée | `quickstart.md` |
