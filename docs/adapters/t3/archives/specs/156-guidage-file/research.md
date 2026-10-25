# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/research.md](../../../../../../specs/156-guidage-file/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11, révisée après contre-revue (`adversarial-review-codex.md`).
Source : lecture du checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget`
(base `1f2c661d`, non modifié) et de T3 `origin/main`. La session150 est supposée livrée.

<a id="extrait-2"></a>

## Extrait 2

- ADR 011 : la garantie tient « par impossibilité » face à un agent qui exécute des commandes mais ne peut pas appuyer sur les touches d'un terminal ouvert de l'utilisateur. ADR 027 : la reconnaissance du principal humain « arrête un agent qui agit par ses outils déclarés, pas un processus hostile du même compte ».
- **156 protège contre un agent qui utilise ses outils Bridget** (MCP, CLI, pont T3, wrapper).
- **Hors modèle** (limite acceptée, non promise) : un processus du même compte Unix qui ouvre une socket brute et forge des trames. Preuves : rôle déclaratif sans preuve (`daemon.rs:14213`), `human_from_bare_client` (`daemon.rs:8604-8606`), pseudo-terminal propre pour `bridget attach` (`attach.rs:267`, `:405-415`).
- **Dans le modèle, corrigé** : (1) CLI sans preuve d'identité qui signe `human` (`cli.rs:3479`, `:4322`) ; (2) message de `ControlExecution` à l'origine non normalisée ; (3) guidage implicite du pont T3 ; (4) `bridget_cancel` sur une demande active.
- Les tests de trames brutes (T014, T023) sont de la défense en profondeur : le daemon est le seul point d'application, il applique la garde quel que soit le client.

<a id="extrait-3"></a>

## Extrait 3

- `deliverable_while_busy(provider, groupable_front)` (`t3code.rs:3126`) donne `steer_current = true` à `dispatch` (`t3code.rs:2206`) pour tout message groupable (sans réponse attendue) vers Claude ou Cursor occupé, sans regarder l'expéditeur. Un pair injecte ainsi du texte dans un tour actif.
- **Décision** : l'expéditeur doit pouvoir guider. Le daemon calcule `mid_turn_ok` (attach, UI attestée, systèmes internes, parent/enfant) aux constructions de DeliverIdempotent ET Deliver historique (envoi et reprise) et le pose sur la trame ; le pont l'exige. Deliver historique porte aussi mid_turn_ok ; aucune origine Human fabriquée. La remise immédiate utilise la connexion source réelle. À la récupération, from ne permet jamais d’emprunter une route UI ou humaine ; sans preuve UI persistée, la remise attend normalement. Pour les agents UUID, recalculer les liens depuis les faits actuels. UI attestée et systèmes internes reconnus par preuve/source, pas from=humain. Enfant vers parent reste permis : c'est le cas de la session 129 (coordinateur qui retenait 13 messages 8 h 45). Les pairs attendent la fin du tour. Changement visible : question 3a.
- Alternatives rejetées : laisser tel quel (guidage sans garde, sans pause ni rythme) ; décision dans le pont (il ne connaît pas la parenté).

<a id="extrait-4"></a>

## Extrait 4

- `CancelRequest` vérifie l'émetteur seulement (`daemon.rs:18128-18150`), puis envoie `CancelDelivery` aux présences ACP ou `t3code` (`:18155-18175`). `AcpTransport::cancel_delivery` annule le tour actif si l'identifiant correspond (`acp.rs:518-531`). Le pont T3 ne fait que retirer de sa file (`t3code.rs:1684-1692`).
- La description MCP de l'outil dit déjà : « n'arrête pas l'agent » (`mcp.rs` description de `bridget_cancel`). Le comportement ACP la contredit.
- **Décision** : `CancelDelivery` gagne `interrupt_active: bool` (défaut faux). `ManagedSession::cancel_queued(id, reason) -> bool` (défaut faux) retire sans toucher au tour actif ; ACP l'implémente sous le même verrou. Le daemon met `interrupt_active = true` seulement si l'appelant passe la garde commune (droits, pause, rythme, mission), et compte l'interruption. Un ancien wrapper garde l'ancien comportement jusqu'à sa relance (noté dans la doc).
- Hors périmètre : Codex et Claude gérés reçoivent un texte, pas `CancelDelivery`.

<a id="extrait-5"></a>

## Extrait 5

- Source : `send_deliveries` en phase `dispatching` (`idempotency.rs:527-540`), jointe à `idempotency_records` pour `issued_at`. Ordre `rowid`, conservé par la réaffectation qui met à jour en place (`send_delivery.rs:230`). Une ligne `dispatching` peut être en cours d'injection : état `pending_or_injecting`.
- Un index `(recipient_instance_id, phase)` évite le parcours complet (`send_delivery.rs:372-377`).
- `CancelDelivery` (`protocol.rs:4147-4151`) est sans résultat. `cancel_delivery` Codex annule d'abord le tour actif si l'identifiant correspond (`codex_app_server.rs:1081-1093`).
- **Décision (Art. XIX, contre-revue 5 à 8 et 10)** : pas d'édition de file dans 156. Le `CHECK` de `phase` n'admet ni `cancelled` ni `rejected` (`idempotency.rs:534`) ; la reprise relit toute remise `dispatching` (`send_delivery.rs:379-449`) donc un retrait mémoire sans réservation durable est réinjecté ; `promote` exige une saga durable. Prérequis de la future session : réservation durable avant émission, accusé durable, réconciliation avant relivraison, phase d'annulation (reconstruction de table), retrait réservé aux remises encore en file sous le verrou du transport (Claude, ACP, file Codex `steer`), saga de promotion.
- Pour une demande suivie (`reply=true`), `bridget_cancel` existe déjà (`daemon.rs:18125-18200`) et envoie `CancelDelivery` aux fils T3 et ACP. T022 documente et sépare le retrait de l'interruption selon la garde commune, sans édition de file nouvelle.

<a id="extrait-6"></a>

## Extrait 6

- `dispatch_command(command, steer_current: bool)` ne pose que `deliveryIntent: "steer"` (`t3code_contract_v2.rs:255-272`). T3 accepte `auto | steer | restart` (T3 `orchestrationV2.ts:2858`).
- T3 `decideSteeringExecution` (`CommandPolicy.ts:238-262`) : sans `forceRestart` et sans guidage actif, T3 choisit `interrupt_restart` si le fournisseur sait interrompre. Donc un `steer` vers un fournisseur sans guidage natif interromprait en silence.
- **Décision revalidée T002(c)** : `steer` envoyé seulement à Claude, dont activeSteering natif est prouvé par le bundle installé149 ; Cursor est hors liste car son repli peut interrompre ; sinon rejeté `steer_unavailable`. `interrupt` pose `restart`.
- `maybe_deliver` n'examine que `queue.front()` (`t3code.rs:1962-1975`). Un message de guidage doit être choisi dans toute la file et rester hors lot (`batch_family`, `t3code.rs:3150`). La remise des groupables vers Claude reste immédiate avec activeSteering natif prouvé pour attach/UI attestés, systèmes internes et parent/enfant ; les pairs attendent (FR-022).
- `QUEUE_BOUND` = 64 inchangé (`t3code.rs:44`) ; exposé par `bridget_queue`.

<a id="extrait-7"></a>

## Extrait 7

- T3 a six outils de file. Bridget en prend un, en lecture seule : `bridget_queue` avec `target`. Un futur `action` s'ajoutera avec la session d'édition.
- Identité : `ResolvedIdentity` (`mcp.rs:673-684`), preuve privée, `RoleHandshake` Client puis `ClientHello` avec `QueueListV1`.

<a id="extrait-8"></a>

## Extrait 8

Voir `adversarial-review-codex.md`. Supprimés : mode de repli, table de décision de l'effet, `intent_applied`, `QueueEdit`/`QueueControl`, `queue_rank`, phase `cancelled`, `cancel`/`reorder`/`promote`. Ajoutés : garde sur `Send` et `ControlExecution`, normalisation de la provenance, liste blanche T3, choix dans toute la file du pont, tests combinés.

<a id="extrait-9"></a>

## Extrait 9

1. Ce que reçoit l'émetteur quand un transport rejette un `steer`.
2. Provenance attach/UI attestée/systèmes internes ; aucun privilège du client nu.
3. Fournisseurs T3 à guidage natif prouvé.
4. Noms des champs de la tâche native (propriétaire, `child`).

<a id="extrait-10"></a>

## Extrait 10

- F1 : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:7758–7775 stamp30s sur tout steer ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs:1483–1497,1598–1605 repli sur toute deadline. Filtrer le repli à l'humain validé ; enlever seulement le stamp ne couvre pas une deadline fournie.
- F2 : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:21306 appelle negotiate_idempotent_client ; :23538–23568 fait RegisterAuxiliary avec credential. UI testé n'est donc pas BareClient. Validation : :8774–8788 ; identité UI réservée et instance ui : :16264–16274 ; rôle immuable : :14706–14730 ; alias de routage sans preuve : :8170–8178.
- F2 historique/système : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:4193 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:1782–1790,18748–18753. Garder groupable : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:3142–3155.
- F3/F6 : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs:861–865 pas de tour dédié steer, :897–915 FIFO.
- F5 : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:9204–9220 projet, :9323–9346 acceptation durable. Compter admitted après acceptation, sans promettre confirmation fournisseur.

<a id="extrait-11"></a>

## Extrait 11

Le principal confirme T002(c) par lecture du bundle T3 effectivement installé149 :
Claude supportsActiveSteering=true et query.offer ; Cursor
supportsActiveSteering=false et supportsSteeringByInterruptRestart=true ;
decideSteeringExecution peut convertir steer en interrupt_restart. Ce constat
porte sur les capacités du code installé, pas sur un nouveau tour de modèle
ou une recette active156. Les citations CommandPolicy.ts précédentes restent
historiques ; leur équivalent et le SHA du paquet sont attribués au rapport
frais de l’owner audit_existant151 lorsqu’il est reçu.
Décision : Claude seul pour steer T3 ; Cursor steer_unavailable et attente
implicite en busy même si mid_turn_ok=true. T012/T021/T029 existants couvrent
le changement visible accepté par root ; aucun nouveau test de confort.
Les comptages T006 retournent count+LASTcouple et count+THIRD_LATESTcible,
avec now transmis à la garde pure. QueueList est une action de lecture de la
même garde, sans audit/quotas/écriture de pause. Les31 IDs et noms sont conservés.

<a id="extrait-12"></a>

## Extrait 12

Codex DeliveryRejected : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs:925. Wrapper injection_rejected : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:830 → DeliveryIndeterminate ; event :6513 → failed et DeliveryRejected. Daemon : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:18676 relaie refus différé Attach :18687, puis avis pour demande reply suivie :18698. T3 report_discard : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:1868 émet indeterminate, et DeliveryRejected seulement si reply=true :1900. Un steer reply=false ne garantit aucun avis à son émetteur. Le reçu synchrone reste admission/effet non confirmé. Repli F1 nécessaire : stamp /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:7759 et earliest_unconfirmed_steer_deadline /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs:1803 lisent toute deadline ; filtrer Human validé même si deadline fournie, sans changer deadline du tour :1642.

<a id="extrait-13"></a>

## Extrait 13

Paquet installé /Applications/T3 Code (Local).app/Contents/Resources/app.asar, entrée apps/server/dist/binCli-B0p7vLio.mjs, SHA7764a465393168b82c9a2986e406436f46548b56f70bcf5623779363341072d9. Schema deliveryIntent :31851–31855 ; resolve :123140 ; decideSteeringExecution :123177–123180. Claude capabilities :184184/184206=true ; steerTurn :188998 → query.offer :189022. Cursor capabilities :224161/224183=false, interrupt_restart :224184=true ; adapter lit ces faits :224626. ROOT arbitre Claude seul pour guidage explicite ET implicite ; Cursor steer_unavailable et attente pendant busy. CommandPolicy historique absent du checkoutT3 actuel ; pas de nouveau fournisseur ni relance.

<a id="extrait-14"></a>

## Extrait 14

Constructeur idempotent production unique /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:4496/4545 defer_idempotent_delivery ; appels envoi9361, reprise4593, récupération4870, notices8514. Constructeurs Deliver historiques :1698 avis,1791 observation,18306 Send,18770 avis Cancel. CancelDelivery :5217 échéance et18764 CancelRequest ; cibles structurées ACP/T3 :18759. Récepteurs /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:4540 cancel_delivery et /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:1684 retrait file (également1408 à connexion). Tmux :2538/2572 lit remises mais aucune branche Cancel structurée. ACP /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/acp.rs:511–541 annule actif OU retire file sous verrou ; cancel_queued séparé nécessaire T022. Deliver est tuple /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:4309, serde tag type :3903 : proposer même variante avec message #[serde(flatten)] + mid_turn_ok Option omis si absent (patron existant :4811), sans projection ajoutée au canon BridgetMessage.

<a id="extrait-15"></a>

## Extrait 15

Package installé149 : /Applications/T3 Code (Local).app/Contents/Resources/app.asar.
L’extrait binCli-B0p7vLio.mjs de apps/server/dist est conservé dans
/Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/t3-installed149-excerpts.txt.
SHA256 extrait : 44fb816df22bb51006c0d679673f55f7efc5b6d0af1fce4aff9ab388b0669ac0.
Preuve du paquet : /Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/t3-installed149-evidence.json.
Les constats «T002 attendue» ci-dessus sont historiques ; tous ses axes sont
maintenant prouvés par lecture, sans nouvelle preuve runtime156.
