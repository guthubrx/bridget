# Revue de sources F2/O4 - session 149 - Sonnet 5.5 - ronde r1

Date : 2026-10-10. Relecteur : sous-agent de revue indépendant (Sonnet 5.5).
Cette revue lit les sources seulement. Elle n'édite aucun fichier de code, de test ou de documentation. Elle n'exécute ni Cargo, ni test, ni build, ni service, ni Git. Elle ne lance aucun modèle. Elle n'affirme aucun comportement d'exécution.

## 1. Verdict

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-f2-o4-source-review-sonnet-r1.md#extrait-1).

Ce verdict couvre l'état de disque décrit à la section 2. Une nouvelle modification de production rouvre cette revue pour les fichiers touchés.

Le delta F2/O4 est plus récent que tous les binaires et reçus connus. Les reçus r6 et r7 portent l'empreinte de production `6cc9a2be1dd3e60d...`. L'empreinte actuelle est différente (section 2). Les binaires immuables `620e...` (debug) et `abfb...` (release) ne contiennent donc pas ce code. La ronde r8 doit reconstruire avant toute preuve.

## 2. Code exact relu

Racine : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`. Base `6807c22b`. L'arbre n'est pas committé.

Empreinte calculée avec `validation/native-r5-fingerprint.py` (lecture seule) :

- 198 fichiers `.rs`, `.toml`, `Cargo.lock` : `bc916f3bbd833dce3a0078769f6fbd82fd3d84a9aa06d6127bf5865d1acde5e9`.
- 111 fichiers de production : `3943009ca82db59d850d913b55b18c7a0a17c91c3143ffaeb39ee6cdd600526d`.
- Fichiers de production modifiés après la revue finale `review-final149.md` (14:48) : `native_delegation.rs` (17:19), `codex_app_server.rs` (17:17), `managed_session.rs` (17:17), `wrapper.rs` (17:17), `daemon.rs` (17:07).

Digests individuels (16 premiers caractères) :

| Fichier | Digest |
|---|---|
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | `ea4e4316a796f62a` |
| `crates/bridget-daemon/src/daemon.rs` | `7815b19971f4eaa7` |
| `crates/bridget-daemon/src/wrapper.rs` | `28e92599eee1bb03` |
| `crates/bridget-transport/src/managed_session.rs` | `42dfa0a2fce380fd` |
| `crates/bridget-transport/src/codex_app_server.rs` | `b9b68ad5cb386483` |

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-f2-o4-source-review-sonnet-r1.md#extrait-2).

Les chemins Rust ci-dessous sont relatifs à `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/`.

## 3. Revue point par point

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-f2-o4-source-review-sonnet-r1.md#extrait-3).

## 4. Findings

**Aucun finding bloquant ou majeur.**

### Écarts optionnels, non bloquants

Chaque écart donne le déclencheur concret et la preuve dans les sources. Le principal décide s'il les ferme.

**E1 - `descendants_busy` en erreur fait échouer une racine qui a son résultat** (`native_delegation.rs:1008-1016`)

- Déclencheur : une erreur de lecture pendant le tick, sur une racine `waiting_for_children`. Sources d'erreur : `descendant_limit` (permanent) ou une erreur SQLite au-delà du délai de 2 s (`execution_store.rs:216`).
- Effet : la racine passe `failed/native_descendants_unavailable:...`. `task_view` n'affiche le résultat que pour `result_available` (`:85`). Le résultat capturé n'est plus lisible. Avant le delta, l'erreur gardait l'attente (`unwrap_or(true)`).
- Pourquoi non bloquant : le choix « ferme en cas d'erreur » est cohérent avec la consigne. Le déclencheur transitoire est rare.
- Test minimal pour r8 (optionnel) : une erreur de lecture simulée sur une racine retenue, puis constat de l'état voulu.

**E2 - une tâche `queued` dont le propriétaire est un enfant natif ne repart jamais après redémarrage** (`:829-868`, `:1035-1041`)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-f2-o4-source-review-sonnet-r1.md#extrait-4).

**E3 - descendant `cancelled`, `cancelling` ou `failed` pour une autre raison, avec exécution restée `running`** (`:1264-1274`)

- Déclencheur possible : annulation individuelle d'un petit-enfant, puis redémarrage avant la fin. `prepare_restart` ne ferme l'exécution que pour `failed/unreachable`. `descendants_busy` exige exactement ce couple pour ignorer l'exécution.
- Non prouvé : en marche normale, le wrapper envoie ses événements finaux après l'arrêt (`wrapper.rs:4693-4703`), ce qui ferme l'exécution. Le cas problématique demande un redémarrage dans la fenêtre de l'annulation.
- Test minimal pour r8 (optionnel) : tâche `cancelling` avec exécution `running`, redémarrage, puis état de la racine en attente au-dessus.

**E4 - fenêtre résiduelle de l'orphelin Codex avant la connaissance du `turn_id`** (`codex_app_server.rs:1117-1140`)

- Déclencheur : SIGTERM entre l'envoi de `turn/start` et l'enregistrement du `turn_id`. Dans ce cas il n'y a pas d'interruption. `shutdown` envoie SIGTERM au groupe puis fait `child.wait()` sans borne (`:755-830`). Si l'application serveur draine le tour en cours, le wrapper attend sa fin. Le daemon n'utilise jamais SIGKILL (`managed_process.rs:402`).
- Effet : même risque que l'arrêt ordinaire d'aujourd'hui. Ce n'est pas une régression. Il reste une fenêtre étroite.
- Non traité par ce delta : un wrapper tué par SIGKILL laisse son fournisseur en vie. macOS n'offre pas d'équivalent de `PDEATHSIG`. Le fournisseur finit à la fin naturelle de son tour.

### Observations mineures (information)

- Un `result_available` non remis garde un `error` transitoire (`native_result_owner_offline`, `native_result_delivery_pending`, `:1037`, `:1194`). Il est effacé à la remise (`:1157`). L'interface l'affiche en alerte (`apps/web/src/components/BridgetTaskJournal.tsx:380`). Le message est vrai : la remise au parent est en attente.
- `Rejected`, `Orphaned`, `EnvelopeMismatch`, `IdempotencyExpired` rendent maintenant la tâche `failed` (`:1169-1193`). Avant, la boucle de relance était infinie. Le nouveau comportement est cohérent car la clé idempotente est figée sur ce refus.
- `for_mission` lit par `json_extract` sans index (`delegation.rs:195`). `schedule_idempotent_delivery_recovery` l'appelle une fois par remise en cours. Coût O(N), N au plus 4096, hors chemin chaud. À noter pour l'Article XVIII.
- Dépendance `sha2` avec la fonction `asm` (`Cargo.toml`) : hors du delta F2/O4, motivée par `hash-latency149.md`.
- Documentation : les en-têtes de `tasks.md` et `plan.md` disent encore « G-P REQUEST_CHANGES ciblé r3 ». `permissions-contract-deltas-r4.md` donne APPROVE. L'ADR149 et `contracts/permissions.md` portent encore « proposé pour gate GLM ». Le principal arbitre ces statuts.

## 5. O1 - clarification de G-P-07(b)

Faits lus dans les sources :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-f2-o4-source-review-sonnet-r1.md#extrait-5).

Formulation proposée (à reprendre par le principal ; cette revue n'édite aucun document) :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-f2-o4-source-review-sonnet-r1.md#extrait-6).

Passages à aligner si le principal adopte ce texte : `contracts/permissions.md` lignes 44-46 et 440-448 (G-P-07(b)), `test-strategy.md` S149-32(b). Le comportement observé en réel est dans `recovery149.md`, contrôle R8.4.

## 6. Ce que la ronde r8 doit prouver pour fermer cette revue

La ronde r8 possède les tests et Cargo. Ces cibles viennent de la lecture :

1. Reconstruire depuis l'empreinte de production `3943009c...`, sans avertissement de compilation (le delta a été écrit après le dernier build connu).
2. Les tests F2 de `native_delegation_permissions149_tests.rs` : exécution terminale non rouverte, agent étranger refusé sans modification, CAS périmé refusé, autorité qui recoupe chaque champ, message externe sans capacité, chaîne complète « réponse capturée puis descendant perdu puis une seule remise ».
3. `native_provider_death149_test.rs` : la mort du fournisseur et le SIGTERM du wrapper natif (un seul PID tué à la fois, après identification non Firefox).
4. Une exécution réseau de R9.2 et R9.4 sur le nouveau binaire : tâche `failed/unreachable` dès le démarrage, racine retenue remise une fois, fournisseur arrêté avec le wrapper (corrige O4).
5. Un contrôle négatif de l'ordinaire : un agent persistant hors saga garde sa reprise (le test `native149_f2_mission_engagee_est_fermee...` le prévoit).
6. Optionnel : E1 et E3.

## 7. Limites de cette revue

- Aucune compilation, aucun test, aucun processus lancé. Les types et les emprunts du code ajouté ont été vérifiés par lecture seulement (variantes `IdempotencyIssue`, champs de `CodexTurnDetail`, imports par `use super::*`). Une erreur de compilation reste possible tant que r8 n'a pas construit.
- Aucun fournisseur réel. Le comportement d'un vrai Codex à l'arrêt (drain du serveur) reste non prouvé (E4).
- Les rapports de recette (`recovery149.md`, `native-network-proofs-r2.md`) décrivent le binaire r5. Ils ne prouvent pas le correctif lu ici.
- Aucun secret n'est copié dans ce rapport.
