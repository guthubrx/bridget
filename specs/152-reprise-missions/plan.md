# Plan - Session152 reprise des missions natives

**Spec** : `specs/152-reprise-missions/spec.md`
**Branche** : `session-150-socle-delegation` (worktree unique 150 à 158)
**Date** : 2026-10-11
**Préalables livrés (hypothèses H1, H2)** : sessions 150 et 151.

## Résumé

Cinq corrections dans le moteur de délégation native. Elles partagent un seul
mécanisme : reprendre une mission avec son identifiant d'origine, en suivant la
chaîne d'exécutions de l'ADR 027.

1. Pause : suspendre au lieu d'échouer, reprendre quand l'exécution est
   `interrupted` (R1, R2, R3).
2. Horloges : geler l'échéance de mission, le suivi de demande et le lancement
   (R4, R5, R11).
3. Panne : reprendre au lieu de passer `failed/unreachable`, avec une borne (R6).
4. Annulation en cascade : classer d'abord, arrêter ensuite, ne rejouer que les
   échecs (R7, R9).
5. Reçus de refus définitifs (R8).

## Contexte technique

- Rust 2024, crate `bridget-daemon` (et `bridget-transport` si le contrat de
  statut change : non prévu).
- Stockage : payload JSON de la tâche (champs `serde(default)`), une table SQLite
  nouvelle pour les reçus de refus. Pas de nouvelle dépendance.
- Tests : recette `umask 077` (voir `quickstart.md`).

## Décisions architecturales

| # | Décision | Raison |
|---|---|---|
| D1 | La suspension est un fait dérivé (pause active et état gelé), pas un état ni un champ | Lineage 149 inchangé (R10) |
| D2 | Garde de pause en tête de `tick`, avant l'échéance | Objection 4 (R4) |
| D3 | `AutonomousEffect::NativeMissionDispatch` | Garde unique ADR 027 (R11) |
| D4 | `rejected()` ignore le refus de pilote si une ligne de pause existe | Objection 1 (R1) |
| D5 | L'état `interrupted` de pause n'échoue pas la tâche | Chemin B (R1) |
| D6 | Reprise pilotée par `tick`, exige `interrupted` | Objection 2 (R2) |
| D7 | `latest_in_chain` remplace `execution-{mission}` à 4 endroits | Objection 3 (R3) |
| D8 | `frozen_secs` = `pause_overlap` depuis la ligne durable de la demande | Idempotence (R4, R23) |
| D9 | `launch_issued_at` posé à la levée, sans saga existante | Objection 7 (R5) |
| D10 | `prepare_restart` garde la tâche si identité, échéance, borne OK | Défaut 1 (R6) |
| D11 | `native_child_env` extrait de `spawn_task` | Env natif à la reprise (R6) |
| D12 | `classify_stop` séparé de `prepare_managed_stop` | Vérifier d'abord (R7) |
| D13 | Table `native_delegation_refusals`, liste fermée de cinq codes | Reçus de refus (R8, R18) |
| D14 | Deux chemins de reprise (lancement en cours, enfant connecté) avec filiation ; commande dérivée du lien attesté | Objection 1 (R13, R24) |
| D15 | `restart_resumes` incrémenté à la décision de reprise (sur-compte seulement) | Objection 2 (R14, R22) |
| D16 | `settle_suspension` dans `prepare_restart` ; `interrupting` réconcilié en `interrupted` | Objection 3 (R15) |
| D17 | `rejected()` rend `pause_interruption` ; le gestionnaire général garde le suivi et se tait | Objection 4 (R16) |
| D18 | Dernière exécution = feuille de plus grande `generation` | Objection 5 (R17) |
| D19 | Gel du suivi calculé depuis la demande durable ; `mission_issued_at` | Objection 6 (R19, R23) |
| D21 | Temps de pause = fonction du journal `control_events` (`pause_overlap`) ; échéance effective sans réécriture | Ronde 2 point 3 (R25) |
| D22 | `restart_identities` non fatal : tâche irréconciliable `failed` nommée | Ronde 2 point 2 (R24) |
| D20 | Annulation : contrôle avant toute mutation, tick sur les racines seulement | Objection 7 (R20) |

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-daemon/src/delegation.rs` | champs `launch_issued_at`, `mission_issued_at`, `restart_resumes`, `cancel_stopped` ; table et lecture des reçus de refus |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | garde de pause dans `tick`, `rejected()`, `latest_in_chain`, `may_resume`, `prepare_restart`, `native_child_env`, `cancel_tree`, `task_view`, reçus de refus à l'admission |
| `crates/bridget-daemon/src/daemon.rs` | garde de reprise (`:4251`), `reserve_managed_recoveries`, `PendingReply.frozen_secs`, `collect_reminder_actions`, `restore_pending_for_agent`, `classify_stop`, appel de reprise depuis `tick` |
| `crates/bridget-daemon/src/execution_store.rs` | `latest_in_chain` (génération), `abandon_pause_interruption` |
| `crates/bridget-daemon/src/referent_control.rs` | variante `NativeMissionDispatch` |
| `crates/bridget-daemon/src/daemon.rs` (entretien) | purge des reçus de refus de plus de 30 jours dans le passage de la 150 |
| tests des modules, `tests/native_delegation_e2e.rs` | tests `native152_*` |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | pause, reprise, refus nouveaux |
| `docs/decisions/152-reprise-missions-natives.md` | ADR des décisions D1 à D13 |

Aucun nouveau module Rust.

## Étapes d'implémentation

1. Baseline et tests rouges (T001 à T007, T038 à T040) : pause chemin A complet,
   chemin B, ordre inverse, horloges, lancement après pause, panne (enfant
   connecté, deux redémarrages, imbriquée), panne pendant pause, gel mixte,
   annulation multi-ticks.
2. Fondations (T008 à T011) : champs de tâche, variante de garde, fonctions de
   magasin, `frozen_secs`.
3. US1 et US2 (T012 à T021, T044, T045) : pause, reprise, gel des horloges.
4. US3 (T022 à T025, T041 à T043) : reprise après panne.
5. US4 (T026 à T028, T046) : annulation en cascade.
6. US5 (T029 à T031) : reçus de refus.
7. Finitions (T032 à T037) : compatibilité, docs, ADR, suite complète, relecture.

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 152 prévu (T035) |
| VIII Forensic | OK | tests rouges qui reproduisent les chemins avant correction |
| XV Test-before-next | Adapté | pas de commit automatique ; chaque tâche testée avant d'être cochée |
| XVI Worktree | OK | worktree unique 150 à 158 |
| XVIII Complexité | OK | `latest_in_chain` O(c), c = pauses et reprises de la mission (non borné par une constante, petit en pratique) ; `classify_stop` O(m), m = membres de l'arbre (au plus 4096) ; gel des demandes O(p), p = demandes ouvertes |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Minimalisme (Article XIX)

- Réutilisés : reprise ADR 027 (`reconstruct_active_for_agent`), chaîne de
  continuations, `prepare_recovery`, garde unique, entretien de la 150.
- Pas de nouvel état, pas de nouveau module, pas de nouvelle dépendance.
- Deux extractions de fonctions existantes (`native_child_env`, `classify_stop`).
  Chacune a deux usages et supprime une duplication de logique d'effet.
- Rejetés : état `suspended` (R10), réécriture de `tracked_requests` (R4),
  retrait de `deadline_at` du canon de flotte (R5), nouveau mécanisme de reprise
  des missions.
- Reçus de refus limités aux refus qui dépendent de l'état du monde (R8).

Potentiel minimalisme : environ 40 lignes supprimées (branche `failed` de
`prepare_restart`, mise à `failed` des chemins A et B, double collecte d'état
d'arrêt).

### Responsabilité future (Article XX)

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- Le risque principal est l'environnement natif d'un enfant relancé (H4). Un
  test rouge le prouve avant le code.
- Non vérifiable dans ce dépôt : le comportement réel de T3 sur un fil `working`
  pendant la pause. La vue reste `working` (question 3).

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | mission sous pause : refus de remise (A), état `interrupted` (B), ordre `TurnState(false)` avant `interrupted` ; échéance qui expire en pause ; lancement après pause de 600 s ; redémarrage avec mission `working` ; annulation avec membre non arrêtable |
| Unitaire | `latest_in_chain`, `may_resume`, décalage idempotent, `frozen_secs`, `classify_stop`, reçus de refus (liste fermée) |
| Intégration | pause puis levée avec résultat rendu une fois ; redémarrage puis résultat ; 3e redémarrage |
| Non-régression | filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087` |
| Suite complète | recette `--workspace --no-fail-fast` avant clôture |

## Risques

| Risque | Parade |
|---|---|
| L'enfant relancé n'a pas l'environnement natif ni la filiation (H4, R13) | test rouge T004 sur les deux chemins ; `native_child_env`, `ownership_from_desired`, commande dérivée du lien |
| Lien de flotte refusé pour un parent d'instance disparue (mission imbriquée) | test rouge T004 (d) ; sinon échec `unreachable` documenté |
| Panne pendant une pause : échéance jugée brute, `interrupting` sans terminal | `settle_suspension` au démarrage, réconciliation `interrupting` (T041, T042) |
| Descendants annulés un par un malgré le refus global | contrôle avant mutation, tick sur les racines (T027, T046) |
| Reprise avant l'état `interrupted` | reprise pilotée par `tick`, exige `interrupted` ; test de l'ordre inverse |
| Décalage d'échéance perdu ou doublé par un crash | aucune écriture : échéance effective = enregistrée + `pause_overlap` (T017, T050) |
| Compteur de reprises sous-compté par un crash | incrément sauvegardé avant la reprise, au même endroit (T023, T025) |
| Daemon qui ne démarre pas sur une identité incohérente | tâche `failed` nommée, démarrage continue (T043) |
| Saga de lancement qui expire pendant une longue pause | limite documentée (FR-012) ; test qui prouve le comportement |
| Boucle de reprise sur une mission qui tue le daemon | borne de 2 reprises ; échec `native_resume_exhausted` |
| Mission annulée ressuscitée par la levée | `may_resume` refuse ; `abandon_pause_interruption` ; test dédié |
| Refus transitoire figé par erreur | liste fermée ; test qui vérifie que chaque code transitoire n'écrit rien |
| Conflits d'édition avec 151 et 153 sur `native_delegation.rs` | exécution séquentielle, un seul worktree |

## Livraison

Code sur disque dans le worktree, sans commit automatique ni relance de service.
Diff cumulé enregistré dans `validation/diff-152.patch`.
