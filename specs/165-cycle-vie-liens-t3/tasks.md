# Tâches — SPEC-165 Cycle de vie des liens T3

**Statut :** Implemented
**Progression :** 18/18
**Pipeline :** Livré.

Chaque tâche reste ouverte jusqu'à sa preuve. Les tests qui définissent le comportement précèdent le code correspondant.

## Phase 1 — Préparation

- [x] T001 Figer le diagnostic initial et les compteurs de fils, liens, sockets et fichiers dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/validation/runtime-before.md`
- [x] T002 Clore la contre-revue du plan et consigner chaque objection dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/adversarial-review.md`

## Phase 2 — Fondations

- [x] T003 [P] Ajouter les oracles RED de filiation V2, de compatibilité V1 et de validation des relations dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code_contract.rs` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code_contract_v2.rs`
- [x] T004 [P] Ajouter les oracles RED de décision racine, fork, sous-agent inactif, actif, terminal, avec obligation et sans obligation dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`

## Phase 3 — US1 Libérer les sous-agents terminés

**But :** le nombre de liens ne croît plus avec l'historique enfant.  
**Test indépendant :** 1 000 sous-agents terminés ne produisent pas 1 000 liens.

- [x] T005 [US1] Porter le chemin réel vérifié `threads[].lineage.relationshipToParent` dans `ThreadSummary`, sans heuristique, bâtir la fixture RED sur sa forme expurgée et journaliser les comptes par relation dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code_contract.rs` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code_contract_v2.rs`
- [x] T006 [US1] Ajouter une décision pure qui réutilise `is_live()` et `turn_is_final()` et tient compte d'une obligation persistée dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`
- [x] T007 [US1] Extraire une décision pure de réconciliation, puis réconcilier en deux temps avec dernier Tick, `drained_key`, horloge `drain_since` non repoussable, file et journal vides, ACK ou borne `turn_wait` ; couvrir le cas silencieux sur cinq lectures et 1 000 enfants terminaux dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`
- [x] T008 [US1] Conserver le lecteur et un clone de fermeture dans `Link`, ajouter une construction à partir de morceaux, finaliser aussi les liens morts et fermer la socket avant de joindre le lecteur ; prouver qu'une panique du worker ne bloque pas la fermeture et que la finalisation reste idempotente dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`

## Phase 4 — US2 Garder le travail actif joignable

**But :** un enfant actif reste exposé et un enfant repris retrouve son lien.  
**Test indépendant :** la même fixture passe de terminal à actif et redevient éligible au rafraîchissement suivant.

- [x] T009 [US2] Couvrir préparation, file, démarrage, exécution, attente, `activeTurnId`, invalidation de la quiescence par une remise reçue après publication et réactivation avant fermeture dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`
- [x] T010 [US2] Scanner le premier snapshot réussi via `load_thread_state(id)`, reprendre `pending` ou un tour connu non projeté, réinscrire une obligation créée après ce scan si le lien meurt, limiter à 32 drains simultanés, tenter une fois par processus, couvrir le redémarrage du daemon et ne pas modifier `drive_queue()` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`

## Phase 5 — US3 Préserver les conversations principales

**But :** les racines, forks et anciens snapshots restent joignables.  
**Test indépendant :** une racine terminale et un fil V1 restent exposés tandis qu'un enfant terminal ne l'est plus.

- [x] T011 [US3] Appliquer la décision au statut détaillé du pont et couvrir racine, fork, fil V1, archive et rangement dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`
- [x] T012 [US3] Vérifier les régressions des parseurs et des remises V1/V2 avec les tests ciblés du paquet `bridget-daemon`

## Phase 6 — Qualité et livraison

- [x] T013 Documenter la décision et ses invariants dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/docs/decisions/165-cycle-vie-liens-t3.md`
- [x] T014 Exécuter format, tests ciblés, tests du crate affecté et contrôles workspace proportionnés ; consigner les commandes et codes retour dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/validation/tests.md`
- [x] T015 Relire le diff complet, supprimer le code sans effet utile et fermer la contre-revue code dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/validation/code-review.md`
- [x] T016 Intégrer et pousser la source validée avec un message Git conforme, puis vérifier la référence distante et la propreté du dépôt
- [x] T017 Construire le binaire signé, préparer le rollback, l'installer, retirer l'override local à 16 384, relancer seulement le pont après contrôle du PID et prouver la limite normale de 4 096 dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/validation/runtime-after.md`
- [x] T018 Finaliser le reçu, cocher uniquement les preuves acquises et retirer sans force le worktree et la branche de session devenus inutiles dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/165-cycle-vie-liens-t3/validation/livraison165.md`

## Dépendances

`T001 → T002 → (T003 || T004) → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T013 → T014 → T015 → T016 → T017 → T018`

## Charge de maintenance

- T005 à T012 étendent trois fichiers existants. Aucun service, format de stockage ou dépendance n'est ajouté.
- T008 ferme une ressource déjà possédée. Elle réduit le risque de fuite future.
- T010 réutilise l'état durable existant. Le scan est unique et évite une lecture disque par cycle.
- T013 rend l'invariant visible pour éviter une nouvelle hausse de limite comme correction principale.
