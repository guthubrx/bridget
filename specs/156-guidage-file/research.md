# Recherche - Session156 guidage, interruption et lecture de file

Date : 2026-10-11, révisée après contre-revue (`adversarial-review-codex.md`).
Source : lecture du checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget`
(base `1f2c661d`, non modifié) et de T3 `origin/main`. La session150 est supposée livrée.

## R0 - Cadre de menace (ADR 011, ADR 027)

- ADR 011 : la garantie tient « par impossibilité » face à un agent qui exécute des commandes mais ne peut pas appuyer sur les touches d'un terminal ouvert de l'utilisateur. ADR 027 : la reconnaissance du principal humain « arrête un agent qui agit par ses outils déclarés, pas un processus hostile du même compte ».
- **156 protège contre un agent qui utilise ses outils Bridget** (MCP, CLI, pont T3, wrapper).
- **Hors modèle** (limite acceptée, non promise) : un processus du même compte Unix qui ouvre une socket brute et forge des trames. Preuves : rôle déclaratif sans preuve (`daemon.rs:14213`), `human_from_bare_client` (`daemon.rs:8604-8606`), pseudo-terminal propre pour `bridget attach` (`attach.rs:267`, `:405-415`).
- **Dans le modèle, corrigé** : (1) CLI sans preuve d'identité qui signe `human` (`cli.rs:3479`, `:4322`) ; (2) message de `ControlExecution` à l'origine non normalisée ; (3) guidage implicite du pont T3 ; (4) `bridget_cancel` sur une demande active.
- Les tests de trames brutes (T014, T023) sont de la défense en profondeur : le daemon est le seul point d'application, il applique la garde quel que soit le client.

## R1 - Le protocole et les transports savent déjà faire

- `MessageIntent` : `QueueOnly`, `TriggerTurn`, `SteerCurrent`, `InterruptAndStart`, `ControlOnly` (`bridget-core/src/message.rs:54-60`).
- `ExecutionControlOperation` : seules `SteerCurrent` et `Interrupt` sont implémentées (`protocol.rs:219-227`, `daemon.rs:9025-9035`, `wrapper.rs:5603-5640`).
- Codex : `deliver` accepte `SteerCurrent` si un tour est actif et `steering_open` (`codex_app_server.rs:865-899`), interrompt pour `InterruptAndStart` (`:901-925`). Un `turn/steer` refusé remet le message en file (`:1700-1760`).
- Claude : `SteerCurrent` refusé (`claude_stream_json.rs:310-317`), `InterruptAndStart` accepté (`:329`). ACP : idem (`acp.rs:620-628`, `:630`).
- **Décision** : exposer, ne rien réinventer. L'intention demandée devient `message.intent`.

## R2 - Chemin d'envoi : `SendIdempotent`

- `ControlExecution` exige `generation` et `revision` (`daemon.rs:9161-9165`), pas pratique pour un agent.
- `SendIdempotent` porte clé, rejeu, garde de projet et admission d'exécution pour `TriggerTurn`/`InterruptAndStart` (`daemon.rs:8828-8850`). Le canon réserve l'intention AVANT le routage (`daemon.rs:8654`).
- **Décision** : l'intention passe par `bridget_send`. Le canon contient l'intention demandée. Le daemon ne la réécrit jamais.

## R3 - Failles existantes (contre-revue 1 et 2, vérifiées)

- `Send` historique et `SendIdempotent` ne retirent ni `intent` ni `origin` du client (`daemon.rs:17521-17560`, `:8584-8640`, seul `thread_notice` est neutralisé).
- Les transports interrompent tout message `origin=Human` si un tour est actif (`codex_app_server.rs:865-866`, `claude_stream_json.rs:329`, `acp.rs:630`).
- `Interrupt` de `ControlExecution` n'a pas de message, donc pas de contrôle d'identité (`daemon.rs:9002-9035`). Capacité `ExecutionControlV1` négociable par tout client (`daemon.rs:16174`). Génération et révision doivent être connues (`:9161-9165`), à la portée d'un client qui observe l'exécution.
- Aucun producteur de `origin=Human` hors tests dans `crates/` sauf le dépôt de service (`daemon.rs:15834`, ledger seul). Les producteurs réels (connexion attach, relais UI) sont hors des crates ; T002(b) les inventorie.
- **Décision** : une seule garde, trois chemins, plus la normalisation de la provenance validée par le daemon. Exception interne explicite pour l'interruption de pause (`handle_execution_control(None, …)`, `daemon.rs:9667`).

## R3b - Autorité humaine (ronde 2, point 1)

- `RoleHandshake` accepte tout rôle sans preuve (`daemon.rs:14213`). Le chemin attach pose `from="humain"` (`daemon.rs:17541`). Le client nu `from=human` est « humain » pour `SendIdempotent` (`:8604-8606`).
- La CLI signe `human` quand aucune preuve (fichier, variable, filiation) n'est trouvée (`cli.rs:3479`) et envoie alors comme client nu (`cli.rs:4322`). Un agent qui vide son environnement, ou rompt sa filiation, y arrive par l'outil. **Dans le modèle.**
- **Décision** : pour le contrôle explicite (`steer`, `interrupt`), l'humain attesté = connexion de rôle attach. Un client nu garde l'envoi en file. Une connexion qui a une identité d'agent (`conn_names`) ne devient pas humaine : test de reconnexion. Le pseudo-terminal propre reste hors modèle (R0). T002(e) revalide l'immutabilité du rôle sur la même connexion déjà identifiée ; une nouvelle connexion Attach brute reste hors modèle.
- Conséquence : `bridget send --intent` depuis un terminal nu est refusé pour le contrôle (question 3b). Alternative rejetée : exiger un terminal côté CLI (contrôle côté client, donc contournable).

## R3c - Origine du message de `ControlExecution` (ronde 2, point 2)

- `handle_execution_control` encode la commande telle quelle (`daemon.rs:9060-9075`) puis `wrapper.rs:5629-5640` appelle `transport.deliver(message)`. Codex interrompt si `origin=Human` et tour actif (`codex_app_server.rs:866`, `:915`). Un parent autorisé à guider obtient donc guidage ET interruption, hors `mission_active` et limites.
- **Décision** : normaliser `origin` du message de commande avant le canon et avant la garde ; Pour ControlExecution, reserve_control_command conserve canon et rejeu AVANT la garde. La garde d’une commande neuve précède mark_control_dispatched, le comptage et l’émission ; le rejeu conserve sa réponse existante.  `Human` toujours retirée (le rôle Client n'est pas attach, `daemon.rs:14195-14213`).

## R3d - Guidage implicite du pont T3 (ronde 2, point 3)

- `deliverable_while_busy(provider, groupable_front)` (`t3code.rs:3126`) donne `steer_current = true` à `dispatch` (`t3code.rs:2206`) pour tout message groupable (sans réponse attendue) vers Claude ou Cursor occupé, sans regarder l'expéditeur. Un pair injecte ainsi du texte dans un tour actif.
- **Décision** : l'expéditeur doit pouvoir guider. Le daemon calcule `mid_turn_ok` (attach, UI attestée, systèmes internes, parent/enfant) aux constructions de DeliverIdempotent ET Deliver historique (envoi et reprise) et le pose sur la trame ; le pont l'exige. Deliver historique porte aussi mid_turn_ok ; aucune origine Human fabriquée. La remise immédiate utilise la connexion source réelle. À la récupération, from ne permet jamais d’emprunter une route UI ou humaine ; sans preuve UI persistée, la remise attend normalement. Pour les agents UUID, recalculer les liens depuis les faits actuels. UI attestée et systèmes internes reconnus par preuve/source, pas from=humain. Enfant vers parent reste permis : c'est le cas de la session 129 (coordinateur qui retenait 13 messages 8 h 45). Les pairs attendent la fin du tour. Changement visible : question 3a.
- Alternatives rejetées : laisser tel quel (guidage sans garde, sans pause ni rythme) ; décision dans le pont (il ne connaît pas la parenté).

## R3e - `bridget_cancel` et interruption active (ronde 2, point 4)

- `CancelRequest` vérifie l'émetteur seulement (`daemon.rs:18128-18150`), puis envoie `CancelDelivery` aux présences ACP ou `t3code` (`:18155-18175`). `AcpTransport::cancel_delivery` annule le tour actif si l'identifiant correspond (`acp.rs:518-531`). Le pont T3 ne fait que retirer de sa file (`t3code.rs:1684-1692`).
- La description MCP de l'outil dit déjà : « n'arrête pas l'agent » (`mcp.rs` description de `bridget_cancel`). Le comportement ACP la contredit.
- **Décision** : `CancelDelivery` gagne `interrupt_active: bool` (défaut faux). `ManagedSession::cancel_queued(id, reason) -> bool` (défaut faux) retire sans toucher au tour actif ; ACP l'implémente sous le même verrou. Le daemon met `interrupt_active = true` seulement si l'appelant passe la garde commune (droits, pause, rythme, mission), et compte l'interruption. Un ancien wrapper garde l'ancien comportement jusqu'à sa relance (noté dans la doc).
- Hors périmètre : Codex et Claude gérés reçoivent un texte, pas `CancelDelivery`.

## R4 - Intention demandée, pas d'intention effective

- Un état d'exécution n'existe que pour les envois `TriggerTurn`/`InterruptAndStart` (`execution_store.rs:527-531` projette les transitions) : un tour démarré par un envoi ordinaire est invisible du daemon.
- Le support de guidage et l'issue du `turn/steer` sont connus du transport seul, et de façon asynchrone.
- **Décision** : pas de table de décision à l'envoi, pas de `steered`/`started`/`interrupted_restarted`. Le résultat dit « admis, effet non confirmé ». L'effet est celui du transport (accusé de remise, journal). Alternative rejetée : prédire l'effet dans le daemon (fausse promesse). Les cas « tour changé » restent gérés par le transport (`expectedTurnId` dans `turn/steer`).

## R5 - Matrice de droits

Fonction pure `authorize(facts)`, testable par table (patron de `admit_autonomous_effect`, `referent_control.rs:122-133`).

| Action | Humain attesté | Parent de la cible | Destinataire (soi) | Pair même projet | Autre projet |
|---|:-:|:-:|:-:|:-:|:-:|
| envoi en file | oui | oui | - | oui | règle interprojet existante |
| `steer` / `interrupt` | oui | oui | non | non | non |
| `bridget_queue` | toute la file | toute la file | toute la file | ses messages | ses messages |

- Parent : `agent_link_for_child(child_instance_id)` puis `parent_instance_id` (`agent_links.rs:79-94`), ou tâche native dont l'appelant est propriétaire et la cible `child`.
- Provenance humaine ordinaire : attach ou UI avec credential/instance primaire réservée de type ui. Client nu signé human : aucun privilège. Rôle Client non promu en Attach ; limite ADR027 inchangée.
- Pour le contrôle, « humain attesté » = rôle attach (R3b) ; un client nu n'en fait pas partie.
- Guidage implicite (message ordinaire déjà groupable remis en cours de tour) : humain attaché, UI attestée, systèmes internes, parent, enfant de la cible. Pair ou client nu : non.
- La garde de projet existante (`communication::project_scope`) précède l'admission de guidage. SendIdempotent inscrit admitted dans la même transaction begin_send_delivery_with_guidance que remise/suivi/lien/journal. Historique/Control/Cancel inscrivent l’audit accepté durable après admission et avant émission physique. Un INSERT échoué interdit le contrôle ; un crash après admission et avant émission peut consommer le quota de façon conservatrice. Un refus tardif ne consomme pas le rythme.
- La preuve enfant (ADR 044) est refusée pour `steer`, `interrupt` et `bridget_queue`.

## R6 - Anti-boucle et audit

- Table `guidance_events` (audit et compteur). Comptage `COUNT` indexé, O(log n + k), k entrées candidates de fenêtre parcourues ; outcome est filtré après le parcours d’index. Cooldown 30 s par couple, plafond 3 par 10 min par cible. Humain exempt. Pause interne non comptée.
- **Ce n'est pas un reçu d'idempotence** (contre-revue 8) : le rejeu relit `idempotency_records` (7 jours, `daemon.rs:466`). Purge à 24 h seulement pour l'audit.
- Les refus de la garde sur `SendIdempotent` passent par `reject_idempotent_send` (`daemon.rs:7694-7718`) : mémorisés jusqu'à l'expiration, nouvelle clé pour réessayer. Même comportement que tous les refus d'envoi existants.

## R7 - Pause et mission native

- ADR 027 : `admit_autonomous_effect` classe chaque puits. `steer`/`interrupt` par un agent devient `AgentGuidance`. La reprise des remises déjà acceptées reste admise.
- La pause interrompt par `Interrupt` sans connexion (`daemon.rs:9667`) et mémorise la reprise (`control_pause_interruptions`, `execution_store.rs:1605`). Cette interruption n'est ni refusée par la garde ni comptée.
- Mission native : `interrupted` est lu comme un échec définitif (`daemon/native_delegation.rs:1193-1203`, spec 150). Interrupt agent vers enfant en mission : mission_active tant que l'interruption utilisateur est terminale, même après152 (pause/panne). Humain explicite exempt ; steer agent permis sans repli automatique interrupt.

## R8 - Lecture de la file, sans édition

- Source : `send_deliveries` en phase `dispatching` (`idempotency.rs:527-540`), jointe à `idempotency_records` pour `issued_at`. Ordre `rowid`, conservé par la réaffectation qui met à jour en place (`send_delivery.rs:230`). Une ligne `dispatching` peut être en cours d'injection : état `pending_or_injecting`.
- Un index `(recipient_instance_id, phase)` évite le parcours complet (`send_delivery.rs:372-377`).
- `CancelDelivery` (`protocol.rs:4147-4151`) est sans résultat. `cancel_delivery` Codex annule d'abord le tour actif si l'identifiant correspond (`codex_app_server.rs:1081-1093`).
- **Décision (Art. XIX, contre-revue 5 à 8 et 10)** : pas d'édition de file dans 156. Le `CHECK` de `phase` n'admet ni `cancelled` ni `rejected` (`idempotency.rs:534`) ; la reprise relit toute remise `dispatching` (`send_delivery.rs:379-449`) donc un retrait mémoire sans réservation durable est réinjecté ; `promote` exige une saga durable. Prérequis de la future session : réservation durable avant émission, accusé durable, réconciliation avant relivraison, phase d'annulation (reconstruction de table), retrait réservé aux remises encore en file sous le verrou du transport (Claude, ACP, file Codex `steer`), saga de promotion.
- Pour une demande suivie (`reply=true`), `bridget_cancel` existe déjà (`daemon.rs:18125-18200`) et envoie `CancelDelivery` aux fils T3 et ACP. T022 documente et sépare le retrait de l'interruption selon la garde commune, sans édition de file nouvelle.

## R9 - Pont T3

- `dispatch_command(command, steer_current: bool)` ne pose que `deliveryIntent: "steer"` (`t3code_contract_v2.rs:255-272`). T3 accepte `auto | steer | restart` (T3 `orchestrationV2.ts:2858`).
- T3 `decideSteeringExecution` (`CommandPolicy.ts:238-262`) : sans `forceRestart` et sans guidage actif, T3 choisit `interrupt_restart` si le fournisseur sait interrompre. Donc un `steer` vers un fournisseur sans guidage natif interromprait en silence.
- **Décision revalidée T002(c)** : `steer` envoyé seulement à Claude, dont activeSteering natif est prouvé par le bundle installé149 ; Cursor est hors liste car son repli peut interrompre ; sinon rejeté `steer_unavailable`. `interrupt` pose `restart`.
- `maybe_deliver` n'examine que `queue.front()` (`t3code.rs:1962-1975`). Un message de guidage doit être choisi dans toute la file et rester hors lot (`batch_family`, `t3code.rs:3150`). La remise des groupables vers Claude reste immédiate avec activeSteering natif prouvé pour attach/UI attestés, systèmes internes et parent/enfant ; les pairs attendent (FR-022).
- `QUEUE_BOUND` = 64 inchangé (`t3code.rs:44`) ; exposé par `bridget_queue`.

## R10 - Un outil MCP

- T3 a six outils de file. Bridget en prend un, en lecture seule : `bridget_queue` avec `target`. Un futur `action` s'ajoutera avec la session d'édition.
- Identité : `ResolvedIdentity` (`mcp.rs:673-684`), preuve privée, `RoleHandshake` Client puis `ClientHello` avec `QueueListV1`.

## R11 - Hors périmètre constaté

- `QueueOnly` du chemin historique `Send` accuse sans remettre (`daemon.rs:17634-17660`). `execution_queue` sans consommateur de production (`execution_store.rs:456`). Dettes notées.
- `cancel_delivery` Codex peut annuler le tour actif (voir R8) : à traiter dans la session d'édition.

## R12 - Complexité

- Garde : O(1). Comptages et dates de fenêtre : O(log n + k), k entrées candidates parcourues. `bridget_queue` : parcours O(log n + s), s lignes dispatching examinées dont expirées, plus jointures indexées ; LIMIT200 borne la sortie seulement. Pont : choix du message de guidage O(n), n ≤ 64.

## R13 - Contre-revue : ce qui a changé

Voir `adversarial-review-codex.md`. Supprimés : mode de repli, table de décision de l'effet, `intent_applied`, `QueueEdit`/`QueueControl`, `queue_rank`, phase `cancelled`, `cancel`/`reorder`/`promote`. Ajoutés : garde sur `Send` et `ControlExecution`, normalisation de la provenance, liste blanche T3, choix dans toute la file du pont, tests combinés.

## Vérifications ouvertes (levées par T002)

1. Ce que reçoit l'émetteur quand un transport rejette un `steer`.
2. Provenance attach/UI attestée/systèmes internes ; aucun privilège du client nu.
3. Fournisseurs T3 à guidage natif prouvé.
4. Noms des champs de la tâche native (propriétaire, `child`).

5. Rôle immuable sur la même connexion ; reconnexion avec preuve d'agent.
6. DeliverIdempotent ET Deliver historique, envoi/reprise et CancelDelivery.

## Prélecture R1 du2026-10-12, à revalider par T002 sur155 livrée

Le contexte initial ci-dessus est historique. Les preuves suivantes viennent
du ROOT actuel, pas de la future base155. Aucun Analyze, test ni T002 complet exécuté.

- F1 : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:7758–7775 stamp30s sur tout steer ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs:1483–1497,1598–1605 repli sur toute deadline. Filtrer le repli à l'humain validé ; enlever seulement le stamp ne couvre pas une deadline fournie.
- F2 : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:21306 appelle negotiate_idempotent_client ; :23538–23568 fait RegisterAuxiliary avec credential. UI testé n'est donc pas BareClient. Validation : :8774–8788 ; identité UI réservée et instance ui : :16264–16274 ; rôle immuable : :14706–14730 ; alias de routage sans preuve : :8170–8178.
- F2 historique/système : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:4193 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:1782–1790,18748–18753. Garder groupable : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:3142–3155.
- F3/F6 : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs:861–865 pas de tour dédié steer, :897–915 FIFO.
- F5 : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:9204–9220 projet, :9323–9346 acceptation durable. Compter admitted après acceptation, sans promettre confirmation fournisseur.

Réemploi des oracles prévus T007/T010/T013/T018/T020/T021 et des témoins
Codex G/H, du témoin d'échéance humaine, du témoin UI credential/route, de
spec129_remise_pendant_un_tour_seulement_si_pilotable_et_sans_reponse et
spec138_steer_guard_refuses_crossproject_before_provider. Aucun scénario de confort.

## Décisions fraîches de reprise sur155 livrée

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

## Préflight et baseline de reprise156

Sélection explicite : SPECIFY_FEATURE=156-guidage-file, dossier de feature
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/specs/156-guidage-file.
Le principal a déjà observé RC127 sur check-prerequisites.sh absent ; la lecture
locale confirme l’absence du script ET de son répertoire. Aucun deuxième essai,
outil installé ou pont officiel réécrit. Fallback manuel : spec, plan, tasks,
research, data-model, contrat, quickstart, reuse-audit, analyze-report présents.
La checklist actuelle compte 16/16 cases de qualité cochées ; elle ne prouve
aucun test ou gate source T002. Extensions before/after_analyze absentes.

Base livrée : source 155 59da3968c7d14dbd7acd1370875dd9d135ba7dcb ; métadonnées
be7957ee00d3801a2cc127e2cd2bd2c8da8c4c4c selon le reçu du principal.
Baseline T001 reçue : 774 PASS, 0 FAIL, 7 ignored ; 214 empreintes code avant/après
égales, toutes les commandes RC0. Seul root a coché T001 ; 31 IDs et cases des
autres tâches conservés. Aucune exécution d’implémentation156 n’est déduite.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/specs/156-guidage-file/validation/baseline156.json.
SHA256 : b8eeb67e7ddc1d0004798f1cb81b8c59c61416816dbea95b2583a96737b854cd.

## Limite de compatibilité REST/V1 du pont T3

La voie REST/V1 Client.dispatch ignore l’ancien booléen et ne sait pas exprimer
deliveryIntent. Elle refuse donc localement toute intention explicite avant
émission : steer_unavailable pour steer, motif de capacité explicite pour
interrupt, avec les erreurs/report_discard existants. L’envoi ordinaire None
reste inchangé. Le paquet installé149 est V2 : le trajet de production visé
n’est pas changé par cette fermeture V1. Aucun protocole, API, type ou nom
d’oracle supplémentaire ; T012 étend seulement les cas déjà prévus.

## Vérifications levées — T002 sur155 livrée

Les six axes a–f sont levés statiquement par audit_existant151 sur17 sources,
confrontées aux blobs15559da3968 ; HEADbe7957ee ne change que les métadonnées.
La collecte est close, lecture ciblée seulement. Aucun runtime fournisseur,
modèle, UI ou nouveau test. Le présent append ne coche pas T002 : root fait
sa relecture avant le seul cochage autorisé.

### Six réponses source attribuées à l’owner

Base155 livrée : 59da3968c7d14dbd7acd1370875dd9d135ba7dcb. HEAD clôture : be7957ee00d3801a2cc127e2cd2bd2c8da8c4c4c.
Lecture source uniquement ; aucun test/code/cochage/service. Borne15minutes atteinte ; collecte close, rédaction finale seulement.

## a — levé par lecture

Codex DeliveryRejected : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs:925. Wrapper injection_rejected : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:830 → DeliveryIndeterminate ; event :6513 → failed et DeliveryRejected. Daemon : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:18676 relaie refus différé Attach :18687, puis avis pour demande reply suivie :18698. T3 report_discard : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:1868 émet indeterminate, et DeliveryRejected seulement si reply=true :1900. Un steer reply=false ne garantit aucun avis à son émetteur. Le reçu synchrone reste admission/effet non confirmé. Repli F1 nécessaire : stamp /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:7759 et earliest_unconfirmed_steer_deadline /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs:1803 lisent toute deadline ; filtrer Human validé même si deadline fournie, sans changer deadline du tour :1642.

## b — levé par lecture

Attach prend rôleAttach (attach.rs:267/910), Send pose from=humain (/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:18118/18128). UI : identité réservée /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:97 ; primary instance typeui :16289 ; register_auxiliary :8768 valide credential, instance et route primaire vivante via live_connection_identity :8607. Garder Client immuable. Producteurs System internes : observation :1785, notice :8475 et native_child_requests.rs:363. deliver_to_agent :1691 et avis Cancel :18770 sont internes sans originSystem : conserver la source au producteur, pas inférer from=bridget. UI externe et trafic réel non lus.

## c — levé par lecture et arbitrage ROOT

Paquet installé /Applications/T3 Code (Local).app/Contents/Resources/app.asar, entrée apps/server/dist/binCli-B0p7vLio.mjs, SHA7764a465393168b82c9a2986e406436f46548b56f70bcf5623779363341072d9. Schema deliveryIntent :31851–31855 ; resolve :123140 ; decideSteeringExecution :123177–123180. Claude capabilities :184184/184206=true ; steerTurn :188998 → query.offer :189022. Cursor capabilities :224161/224183=false, interrupt_restart :224184=true ; adapter lit ces faits :224626. ROOT arbitre Claude seul pour guidage explicite ET implicite ; Cursor steer_unavailable et attente pendant busy. CommandPolicy historique absent du checkoutT3 actuel ; pas de nouveau fournisseur ni relance.

## d — levé par lecture

Task : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation.rs:174 owner,175 owner_instance,176 origin_owner_instance,167 parent_task_id,193 child,194 child_instance,195 mission,199 state. Réutiliser Task/liens existants. /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs:106 may_resume reprend interrupted seulement sous pause_covers :123. Tick :2734–2765 maintient working si pause, sinon failed/interrupted. Interruption utilisateur hors pause demeure terminale malgré152.

## e — levé par lecture

RoleHandshake /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:14722 refuse une nouvelle sélection si rôle déjà fixé ; premier message compatible fixe Wrapper :14712. register_auxiliary :8768 ne modifie aucun rôle et exige credential/instance/route primaire vivante. CLI : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs:4358 choisit Client ; :4369 authentifie seulement si identité présente. Client nu env-i ne reçoit pas autorité humaine de contrôle via from=human. Une nouvelle socket Attach brute du mêmeUID reste hors modèle ADR011/027.

## f — levé par lecture

Constructeur idempotent production unique /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:4496/4545 defer_idempotent_delivery ; appels envoi9361, reprise4593, récupération4870, notices8514. Constructeurs Deliver historiques :1698 avis,1791 observation,18306 Send,18770 avis Cancel. CancelDelivery :5217 échéance et18764 CancelRequest ; cibles structurées ACP/T3 :18759. Récepteurs /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:4540 cancel_delivery et /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:1684 retrait file (également1408 à connexion). Tmux :2538/2572 lit remises mais aucune branche Cancel structurée. ACP /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/acp.rs:511–541 annule actif OU retire file sous verrou ; cancel_queued séparé nécessaire T022. Deliver est tuple /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:4309, serde tag type :3903 : proposer même variante avec message #[serde(flatten)] + mid_turn_ok Option omis si absent (patron existant :4811), sans projection ajoutée au canon BridgetMessage.

## Limites

Aucun runtime fournisseur, UI ou modèle lancé. Des gros fichiers seules les plages critiques sont lues ; sorties tronquées non comptées comme lecture intégrale. Pas audit global155 ni Analyze officielle exécutée.

Provenance vérifiée :
- /Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/research-T002.md : SHA256 b5f74b863238a31c36b8bd5351b7ad8e1584686c411a6315fcbe4521d77d2324.
- /Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/reuse-audit.md : SHA256 5a184496ba498c5b2e8344dcd48da0cc4046da9ee9a807de7fa5415fbe74af52.
- /Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/manifest.json : SHA256 dac18d38653c204b6631e4eb9f51345b61efa91f830123942b516365342b99d7.

Package installé149 : /Applications/T3 Code (Local).app/Contents/Resources/app.asar.
L’extrait binCli-B0p7vLio.mjs de apps/server/dist est conservé dans
/Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/t3-installed149-excerpts.txt.
SHA256 extrait : 44fb816df22bb51006c0d679673f55f7efc5b6d0af1fce4aff9ab388b0669ac0.
Preuve du paquet : /Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/t3-installed149-evidence.json.
Les constats «T002 attendue» ci-dessus sont historiques ; tous ses axes sont
maintenant prouvés par lecture, sans nouvelle preuve runtime156.
