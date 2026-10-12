# Revue158B R1 — preuves natives

Lecture seule fraîche. Aucun test, processus fournisseur, accès DB, Cargo ou mutation Git. Les réservations099887–952 déjà lues par ROOT ne sont pas relues. Les conclusions portent sur les sources présentes, pas sur un runtime158B.

## 1. Self Discovery et durée de vie native

FR104 et le contrat permettent `schedule self` sous WritesOwnState, y compris Discovery. Cette admission reste valide. Elle ne doit pas transformer la programmation en droit de prolonger une mission native terminée.

Preuves (tous les chemins ci-dessous sont absolus) :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/contracts/wake-158B.md:24–28 : self typé, propriétaire attesté, plafond Discovery ; droits revérifiés au départ. Ligne42 : route/instance/capacité/DND fraîches.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon/native_delegation.rs:1448–1460 : CallerScope utilise `DelegationStore::for_child(owner)` ; cleanup_done enlève le plafond de la façade. Ce booléen n'atteste ni mission active ni capacité de redémarrer un enfant.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/delegation_lineage.rs:146–149 : `for_child(&self, child: &str) -> Result<Option<Task>, String>` retourne aussi une tâche terminale conservée ; absence de filtre cleanup/state.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon/native_delegation.rs:3395–3447 : arrêt confirmé, clôture de l'exécution exacte, puis cleanup_done seulement après rapport/garde/demandes soldés. Lignes3479–3545 : purge refusée si route, présence vivante, spawn/saga ou état Fleet Running ; décommission obligatoire avant purge. Cleanup et présence routable ne sont donc pas un même fait.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon/native_child_requests.rs:31–83,87–143 : précédent de revérification au puits : identité live, lien native_delegate, instance, commande, définition épinglée, ancêtres, états/deadline, révocation. Cette fonction exige forward/Codex : ne pas l'appeler artificiellement pour tous les wakes ; réemployer les primitives pertinentes.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs:9898–9928 : TriggerTurn crée/admet une exécution. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/guidance.rs:115–120 : la garde mission active concerne Interrupt, pas tout TriggerTurn. Ce n'est pas une preuve automatique de sûreté pour un timer nouveau.

Recommandation minimale : au vrai départ agent, avant le commit begin_send_delivery099 et de nouveau après toute préparation hors verrou, lire le propriétaire actuel, sa route/instance et son éventuelle Task native via les primitives existantes. Si native : identité/lien/commande/définition/mission courants, tâche et ancêtres encore actifs, cleanup_done=false et exécution courante compatible sont nécessaires ; source manquante ou stockage illisible ferme le départ. Un self ne démarre aucun processus ni nouvelle ronde native à lui seul. Une programmation peut rester visible/annulable après fin de mission, mais elle ne réanime pas cet enfant. Il faut fixer explicitement dans la garde ScheduledWake le résultat d'une fin de vie avant départ, plutôt que déduire une permission de CallerScope=false. Aucune interdiction de créer self Discovery n'est nécessaire.

## 2. Limite send existante

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs:933–943 définit par défaut8 échanges/180s. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-core/src/circuit_breaker.rs:8–12,24–94 : `CircuitBreaker::check(from,to)` puis `record(from,to)` ; paire TRIÉE donc symétrique, horloge Instant, fenêtre strictement récente, mémoire non durable.

Le chemin réel vérifie après portée projet à daemon.rs:9474–9483 ; l'idempotent enregistre après préparation durable à10026–10036 ; l'historique après acceptation durable à19408–19410. Réemploi recommandé : propriétaire attesté comme expéditeur logique, cible canonique, même check/record sous coordination d'état. Origine System reste une provenance du daemon, pas une identité qui contourne le compteur. Aucun quota métierB additionnel. Cette limite concerne la remise agent ; ne pas lui faire certifier un commit de fil. Un refus peut différer dans la grâce initiale selon le contrat:119. Ne pas annoncer un quota persistant après restart.

## 3. CLI-fil : solder099 sans fausse remise socket

API existantes, aucune nouvelle méthode nécessaire :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/idempotency.rs:962–993 : `transition(&self, key: &IdempotencyKey, from: RecordState, to: RecordState) -> Result<(), IdempotencyError>` autorise Prepared→Dispatching seulement, par CAS.
- Même fichier:1004–1042 : `finalize(&self, key: &IdempotencyKey, result: PublicResult) -> Result<(), IdempotencyError>` exige Dispatching ; Accepted/Rejected/Orphaned deviennent terminaux. Pas de finale Accepted directement depuis Prepared.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/store/threads.rs:1067–1087,1259–1273 : `thread_post(&self, request: PostEntry<'_>) -> Result<ThreadTxOutcome, StoreError>` retrouve le reçu UUID/canon puis commit entrée, intentions et reçu d'opération102 ensemble. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/threads.rs:1051–1083 montre le rejeu et le dépôt réels.

Séquence du contrat:104 : UUID102/canon/issued_at gelés → reçu099 Prepared → transition099 Dispatching AVANT post102 → preuve102 Done/Replayed committée → finalize099 Accepted. Après crash, lookup099 puis même UUID/canon102 ; si le dépôt était committé, achever099 sans republier. Relecture terminale rend le résultat enregistré, sans refaire finalize. Erreur SQL ne prouve aucune absence. Dispatching désigne ici la commande CLI en traitement ; aucune ligne send_deliveries ni ACK wrapper n'est fabriquée. `acknowledge_send_delivery` (send_delivery.rs:768–829) est réservé à une vraie remise physique : ne pas le détourner.

Limites : ni mesures de performance ni exécution des scénarios158B. Revue ciblée, pas audit exhaustif du daemon. Les nouvelles gardes ScheduledWake ne sont pas encore démontrées par ce code historique.

Capture SHA256 — 2026-10-12T10:59:38.767010+00:00

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/contracts/wake-158B.md : b298703db69452bd30f33512d8026b881a1314020443f827ccaaec43300fe9db
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon/native_delegation.rs : 4ea51409dad43bf864bb8048d0d60b807265bb10d8f31ad2368a1f8b8a43aeed
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/delegation_lineage.rs : de49575af4fdbd68d104579c5b824210c92b747f72a357526cf5b062f910dda3
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon/native_child_requests.rs : 3be800076fe7523adc083dce913f4d38988dc7da540fb6d4fa6bc3c2adb2af51
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs : a4ec4a19c6904e616fd40b8ee3cb46650eceb4cbc9e77816dd98bfe836e5aa7b
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/guidance.rs : 592cbd364b9ff207d6f73c5b2e35cf95835e308070af9239d322a0e999a61968
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-core/src/circuit_breaker.rs : 7ae6d1c7e69854b46840f3eff39f6bc5b8cdf92da80b9f090eb8800e30267ec4
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/idempotency.rs : b467937682e8769ab5ccf27d2eb024fff22fe4d7c59d43ddd1ebdcfc5a31c5f7
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/idempotency/send_delivery.rs : f1b71c8516447a9d20c9a50b58497ec75c58029c344189439870f68377130d04
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/store/threads.rs : 1e34ef1b3d5b6059e2846010c1cbb44d623b49685f16f9e09072e658c89a93e6
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/threads.rs : 7675231ea83720d4d87c52a3d7dd88d12aead9047fd8940a8ffab9cb4fd1c43d
