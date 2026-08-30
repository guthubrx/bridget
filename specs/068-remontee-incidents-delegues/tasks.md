# Tâches - SPEC-068 Remonter les incidents des délégations

**Entrées**: `spec.md`, `plan.md`, `research.md`, `data-model.md`,
`contracts/delegated-runtime-event-v1.md`, `reuse-audit.md`

## Dépendances

```text
Fondations -> US1 avertissement -> US2 échec terminal -> US3 reprise -> convergence
```

US2 dépend de la persistance et du pont déjà établis en US1. US3 dépend de
l'accusé introduit en US1. Aucun lot ne touche Maicie.

## Phase 1 - Préparation

- [x] T001 Vérifier les checklists, l'état Git et les commandes de test pertinentes dans `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/specs/068-remontee-incidents-delegues/`; résultat observable: aucun fichier utilisateur hors worktree et liste des tests ciblés.

## Phase 2 - Fondations

- [x] T002 Écrire les témoins rouges de sérialisation et d'autorisation des trois trames dans `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-transport/src/protocol.rs`; résultat observable: les tests exigent événement, remise et accusé sans détail brut.
- [x] T003 Écrire les témoins rouges de persistance, ordre et accusé dans `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/idempotency.rs`; résultat observable: un événement non accusé est relu, puis disparaît après accusé autorisé.
- [x] T004 Étendre les trames de `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-transport/src/protocol.rs` avec les types fermés `warning` et `failed`; résultat observable: round-trip JSON stable et refus des valeurs inconnues.
- [x] T005 Étendre `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/idempotency.rs` avec migration compatible, insertion dédupliquée, lecture cursée et accusé parent; résultat observable: T003 devient vert sans réécrire les liens antérieurs.
- [x] T006 Étendre `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/fleet.rs` pour exposer le fait runtime lié et réveiller les lecteurs; résultat observable: le lien réel, jamais une valeur client, détermine le parent.

## Phase 3 - User Story 1: avertissement d'outil (P1)

**Goal**: un coordinateur reçoit un avertissement durable, redacted et non
terminal quand son enfant rencontre un incident d'outil.

**Independent Test**: un enfant lié émet un refus Codex normalisé et le parent
reçoit le fait `warning`, tandis que l'exécution de l'enfant n'est pas
marquée `failed`.

- [x] T007 [US1] Ajouter un diagnostic typé redacted au refus fournisseur connu dans `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-transport/src/managed_session.rs` et `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-transport/src/codex_app_server.rs`; résultat observable: le test conserve code et référence mais aucune méthode ou charge brute.
- [x] T008 [US1] Publier le diagnostic de l'enfant lié depuis `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/wrapper.rs`; résultat observable: un enfant sans lien n'émet rien et un enfant lié produit exactement un `warning`.
- [x] T009 [US1] Persister et pousser le fait vers le parent autorisé dans `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/daemon.rs`; résultat observable: le parent reçoit une trame typée et aucun client ne peut choisir un autre parent.
- [x] T010 [US1] Réceptionner la trame parent dans `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/wrapper.rs`; résultat observable: la notification système entre dans la file normale sans `SteerCurrent`, `InterruptAndStart` ni réponse métier.

## Phase 4 - User Story 2: échec terminal distinct (P1)

**Goal**: un terminal enfant en erreur produit un fait `failed` indépendant des
avertissements.

**Independent Test**: un terminal `ManagedTerminal::Failed` produit une seule
notification d'échec au parent et ne modifie aucun état Maicie.

- [x] T011 [US2] Publier l'échec terminal corrélé depuis `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/wrapper.rs`; résultat observable: `warning` et `failed` restent deux catégories distinctes.
- [x] T012 [US2] Ajouter les oracles de terminaison, de redaction et de frontière Maicie dans `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/{wrapper.rs,daemon.rs}`; résultat observable: aucun appel de guichet ni état Maicie n'apparaît lors du test.

## Phase 5 - User Story 3: reprise sans double remise (P2)

**Goal**: un parent déconnecté reçoit à son retour les faits non accusés dans
l'ordre, une seule fois après son accusé.

**Independent Test**: déconnexion parent, événement enfant, reconnexion,
injection puis accusé, seconde reconnexion sans nouvelle remise.

- [x] T013 [US3] Raccorder la reprise à l'enregistrement et l'accusé à la connexion parent dans `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/daemon.rs` et `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/crates/bridget-daemon/src/wrapper.rs`; résultat observable: l'oracle de reconnexion prouve ordre, conservation avant accusé et absence de doublon après accusé.

## Phase 6 - Finition et preuves

- [x] T014 Exécuter les tests ciblés et la compilation des crates modifiés, mettre à jour `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/specs/068-remontee-incidents-delegues/{spec.md,tasks.md,implementation.md}`; résultat observable: commandes, résultats, non-vérifié et self-review XIX/XX sont consignés.
- [x] T015 Exécuter Converge puis l'audit final de la SPEC dans `/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/specs/068-remontee-incidents-delegues/`; résultat observable: toutes les exigences FR-6801 à FR-6810 ont une preuve code et test, ou une tâche de convergence explicite.

## Stratégie d'implémentation

MVP: Phases 1 à 4, qui livrent un avertissement et un échec distinct à un parent
connecté. La Phase 5 est obligatoire avant le statut final car la durabilité est
une exigence de la SPEC, pas une amélioration optionnelle.
