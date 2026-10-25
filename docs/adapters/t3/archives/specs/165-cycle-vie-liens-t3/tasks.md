# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/tasks.md](../../../../../../specs/165-cycle-vie-liens-t3/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T001 Figer le diagnostic initial et les compteurs de fils, liens, sockets et fichiers dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/validation/runtime-before.md`
- [x] T002 Clore la contre-revue du plan et consigner chaque objection dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/adversarial-review.md`

<a id="extrait-2"></a>

## Extrait 2

- [x] T003 [P] Ajouter les oracles RED de filiation V2, de compatibilité V1 et de validation des relations dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code_contract.rs` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code_contract_v2.rs`
- [x] T004 [P] Ajouter les oracles RED de décision racine, fork, sous-agent inactif, actif, terminal, avec obligation et sans obligation dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`

<a id="extrait-3"></a>

## Extrait 3

- [x] T005 [US1] Porter le chemin réel vérifié `threads[].lineage.relationshipToParent` dans `ThreadSummary`, sans heuristique, bâtir la fixture RED sur sa forme expurgée et journaliser les comptes par relation dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code_contract.rs` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code_contract_v2.rs`
- [x] T006 [US1] Ajouter une décision pure qui réutilise `is_live()` et `turn_is_final()` et tient compte d'une obligation persistée dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`
- [x] T007 [US1] Extraire une décision pure de réconciliation, puis réconcilier en deux temps avec dernier Tick, `drained_key`, horloge `drain_since` non repoussable, file et journal vides, ACK ou borne `turn_wait` ; couvrir le cas silencieux sur cinq lectures et 1 000 enfants terminaux dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`
- [x] T008 [US1] Conserver le lecteur et un clone de fermeture dans `Link`, ajouter une construction à partir de morceaux, finaliser aussi les liens morts et fermer la socket avant de joindre le lecteur ; prouver qu'une panique du worker ne bloque pas la fermeture et que la finalisation reste idempotente dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`

<a id="extrait-4"></a>

## Extrait 4

- [x] T009 [US2] Couvrir préparation, file, démarrage, exécution, attente, `activeTurnId`, invalidation de la quiescence par une remise reçue après publication et réactivation avant fermeture dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`
- [x] T010 [US2] Scanner le premier snapshot réussi via `load_thread_state(id)`, reprendre `pending` ou un tour connu non projeté, réinscrire une obligation créée après ce scan si le lien meurt, limiter à 32 drains simultanés, tenter une fois par processus, couvrir le redémarrage du daemon et ne pas modifier `drive_queue()` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`

<a id="extrait-5"></a>

## Extrait 5

- [x] T011 [US3] Appliquer la décision au statut détaillé du pont et couvrir racine, fork, fil V1, archive et rangement dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/crates/bridget-daemon/src/t3code.rs`
- [x] T012 [US3] Vérifier les régressions des parseurs et des remises V1/V2 avec les tests ciblés du paquet `bridget-daemon`

<a id="extrait-6"></a>

## Extrait 6

- [x] T013 Documenter la décision et ses invariants dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/docs/decisions/165-cycle-vie-liens-t3.md`
- [x] T014 Exécuter format, tests ciblés, tests du crate affecté et contrôles workspace proportionnés ; consigner les commandes et codes retour dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/validation/tests.md`
- [x] T015 Relire le diff complet, supprimer le code sans effet utile et fermer la contre-revue code dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/validation/code-review.md`
- [x] T016 Intégrer et pousser la source validée avec un message Git conforme, puis vérifier la référence distante et la propreté du dépôt
- [x] T017 Construire le binaire signé, préparer le rollback, l'installer, retirer l'override local à 16 384, relancer seulement le pont après contrôle du PID et prouver la limite normale de 4 096 dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/165-cycle-vie-liens-t3/specs/165-cycle-vie-liens-t3/validation/runtime-after.md`
- [x] T018 Finaliser le reçu, cocher uniquement les preuves acquises et retirer sans force le worktree et la branche de session devenus inutiles dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/165-cycle-vie-liens-t3/validation/livraison165.md`
