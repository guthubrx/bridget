# Recherche - Session151 attente et boîte de fin de tâche

Date : 2026-10-11 (révisée après la contre-revue Codex, voir
`adversarial-review-codex.md`). Source : lecture du dépôt principal non modifié
(`/Users/moi/Nextcloud/10.Scripts/64.bridget`, base `1f2c661d`) et de
`specs/150-socle-delegation/` (acquis supposé livré). Les lignes citées sont
celles de cette base. Mécanismes T3 : `specs/150-socle-delegation/research-t3-bridget.md`
(A1, B1, B3, B4). Aucun test ni build lancé.

## R1 - Deux outils d'attente dédiés, en lecture pure

- **Décision** : `bridget_task_wait` (tâches) et `bridget_request_wait` (demandes
  suivies). `bridget_delegate` et `bridget_send` ne changent pas.
- **Raison** : la tâche est durable dès l'admission (`native_delegation.rs:496-504`) et
  l'envoi rend `id` et `issued_at` tout de suite. Un second appel, relancé à volonté,
  suffit. Séparer l'attente de l'envoi évite de cacher `id` et `issued_at` pendant
  l'attente : après une coupure de l'hôte, rien ne pousse l'agent à renvoyer
  (`mcp.rs:725-740`, contre-revue objection 7). Cela évite aussi toute négociation de
  capacité de client.
- **Alternatives** : `mode:"wait"` dans `bridget_delegate` comme T3 (rejetée :
  champ d'enveloppe de plus, aucun gain) ; `wait_timeout` dans `bridget_send` avec
  `id` et `issued_at` imposés (rejetée : `issued_at` doit être proche de l'heure
  du daemon, un agent ne la connaît pas de façon fiable) ; polling de
  `bridget_task_status` (le défaut à corriger).
- **Impact mainteneur** : deux outils de plus dans les listes de R9. Contrat
  d'envoi intact.

## R2 - L'attente vit dans le daemon, sur le thread de la connexion

- **Décision** : `NativeDelegationRequest::Wait` est traité dans `handle_with_fact`
  (`native_delegation.rs:280-320`) avant la prise du verrou d'état. La boucle prend le
  verrou le temps d'une lecture, puis le relâche.
- **Preuve** : un thread par connexion (`daemon.rs:4965-4970`, `handle_connection`
  `:4988`). `handle_with_fact` relâche déjà le verrou avant `tick`
  (`native_delegation.rs:313-319`).
- **Alternatives** : `bridget_events` avec `once` (`mcp.rs:2001`) observe des fins de
  tour, pas des résultats de tâche ; abonnement Lineage (`native_lineage.rs`) est la
  voie humaine et exige `--t3-thread`.
- **À vérifier (T001)** : aucune coupure d'inactivité sur une connexion client du
  daemon. La lecture de trame a un délai de 10 s seulement après le premier octet
  (`daemon.rs:5040`).

## R3 - Curseur d'événements : compteur, condvar, ordre des verrous

- **Décision** : un compteur `WaitEpoch` (`Arc<(Mutex<u64>, Condvar)>`) dans
  `delegation.rs`. `save_projected` et `insert_projected`
  (`delegation_lineage.rs:161-175`) l'incrémentent à chaque écriture, que la
  projection Lineage change ou non.
- **Boucle d'attente** (invariant) :
  1. `seen = epoch.get()` (le verrou du compteur est pris et relâché aussitôt) ;
  2. prendre le verrou d'état, lire les tâches (au plus 16), le relâcher ;
  3. si une tâche est terminée : rendre ; sinon `epoch.wait_changed(seen, tranche)`.
  Ainsi `seen` est relu AVANT chaque lecture, et le verrou du compteur n'est jamais
  tenu pendant l'accès à l'état. Ordre unique des verrous : état puis compteur
  (les écritures de tâche incrémentent sous le verrou d'état). Jamais compteur puis
  état.
- **Raison** : `committed()` (`delegation_lineage.rs:107`) ne signale que les
  changements visibles de Lineage et son canal est à un seul consommateur
  (`change_receiver`). Un résultat peut changer sans changement visible.
- **Filet** : tranche de `min(5 s, délai restant de l'attente)` (pour une demande, aussi
  bornée par l'échéance `deadline_at`). Une tranche fixe de 5 s ferait durer une
  attente de 1 s jusqu'à 5 s (contre-revue ronde 2, point 3). La tranche est un
  paramètre de la boucle : les tests la règlent à 60 s pour prouver le réveil par
  événement. Détecte un client fermé et l'arrêt du daemon, et rattrape une écriture qui
  n'aurait pas incrémenté le compteur.
- **Complexité** : par réveil O(k), k au plus 16 lectures indexées ; au plus 64
  attentes ; une écriture réveille 64 attentes au plus. Ces lectures se font sous
  le verrou d'état, brièvement. `tick` garde ce verrou pendant toute sa boucle
  (`native_delegation.rs:959-1295`) : les attentes patientent pendant `tick`. Le test
  SC-008 mesure que les appels de contrôle progressent.
- **Alternatives** : second récepteur de `change_receiver` (impossible) ; module neuf
  (rejeté : 25 lignes, tient dans `delegation.rs`) ; sommeil fixe de 1 s (rejeté).

## R4 - Expiration : jamais d'annulation

- **Décision** : à l'expiration, `timed_out: true` et les tâches telles qu'elles sont.
- **Raison** : T3 ne tue pas l'enfant à l'expiration (A1). Le délai d'attente est
  indépendant de `mission_deadline_at` (150 : `reply_timeout`).
- **Réveil garanti** : l'attente n'écrit rien (R6) et `tick` ne la connaît pas. Le message
  de résultat part donc exactement comme sans attente. Le test
  `native151_boite_message_apres_attente_expiree` le prouve.

## R5 - La boîte de fin est une vue calculée, sans table neuve

- **Décision** : `mailbox.state` se calcule avec des faits existants :
  - `pending` : `result_available` avec `result_sent = false`, ou `failed` avec
    `failure_sent = false` ;
  - `claimed` : envoi fait, phase de remise `dispatching` ou `indeterminate`
    (lecture de phase de 150-T005, qui lit `send_deliveries.phase`,
    `idempotency/send_delivery.rs:311`), `delivery_acked = false` ;
  - `delivered` : phase `acked` ou `delivery_acked = true` ;
  - `acknowledged` (P2) : `acknowledged_at` présent ;
  - `disposed` : tâche purgée par 150 (`task_unavailable`).
- **Raison** : la tâche durable (`delegation.rs:20-61`) et les remises à phases
  (`idempotency.rs:527`) existent. Une table de plus créerait une divergence possible.
- **Alternatives** : table outbox à cinq états comme T3 (rejetée) ; état stocké à chaque
  transition (rejeté : `delivered` vit déjà dans `send_deliveries`).

## R6 - Simplification retenue : l'attente est une lecture pure

- **Décision** : `bridget_task_wait`, `bridget_request_wait` et `bridget_task_status`
  n'écrivent rien de ce qui touche la livraison. Pas de `result_read_at`, pas de
  verrou d'attente dans `tick`, pas de suppression du message, pas de nettoyage
  déclenché par la lecture, pas de transition de remise partagée.
- **Raison** : la première version marquait « lu » et `delivery_acked` avant l'écriture
  de la réponse sur la connexion. Le daemon écrit la réponse après le retour du
  gestionnaire (`daemon.rs:5164-5173`). Une coupure entre les deux supprimait le
  message de secours et autorisait la purge 150 sans résultat reçu. L'annulation MCP
  ne ferme pas l'exécution (R8). Et `tick` peut envoyer le résultat avant le second
  appel d'attente (`native_delegation.rs:313-319` et envoi du résultat dans le même
  passage), rendant « jamais envoyé » impossible. Seul l'accusé de la remise prouve
  la réception.
- **Coût assumé** : un doublon possible, jamais un résultat perdu (spec FR-021).
  Le résumé de 2048 octets limite le doublon à l'attente ; le texte entier arrive par
  le message.
- **Ce qui disparaît** : champ `result_read_at`, table `task_waits`, branche de `tick`,
  paramètre `include_result`, tests de lecture/suppression (une dizaine de tests).
- **Suppression future** : elle devrait reposer sur un accusé explicite du parent
  (`bridget_task_ack`, R12), jamais sur l'écriture d'une réponse.
- **Impact sur 153** : voir `adversarial-review-codex.md`, section « Impact sur 153 ».

## R7 - Résultat structuré

- **Décision** : ajouter à `task_view` (`native_delegation.rs:68-86`) `work_state`,
  `summary`, `summary_truncated`, `result_bytes`, `detail`, `mailbox`. Le résumé est
  le début du `result`, 2048 octets au plus, sur une frontière de caractère. Les
  entrées d'attente omettent `result`.
- **Raison** : le résultat est figé après `result_available`
  (`docs/delegation-native.md:43`) : une fonction pure de ce texte est stable.
- **Compatibilité** : les clés actuelles sont inchangées (contrat 148,
  `specs/148-identite-delegation/contracts/delegation.md:22-24`). `status` rend
  toujours `result` en entier.
- **À vérifier (T001)** : aucun test ne compare l'objet complet de `task_view`.

## R8 - Façade MCP : créneaux et annulation

- **Constat** : la façade accepte 8 appels en vol par processus
  (`MAX_IN_FLIGHT_TOOL_CALLS = 8`, `mcp.rs:33`) et rend `busy` au neuvième
  (`mcp.rs:259`). Une notification `notifications/cancelled` n'est enregistrée que
  pour un appel en vol (`mcp.rs:147-158`) ; le fil d'exécution continue, puis jette sa
  réponse (`mcp.rs:185-205`). L'exécution et le socket du daemon ne sont pas fermés.
- **Décision** :
  - 4 attentes en cours au plus par processus (`MAX_WAITING_TOOL_CALLS = 4`) ; la
    cinquième rend `wait_capacity` sans occuper un créneau : 4 créneaux restent aux
    appels de contrôle ;
  - registre d'annulation unique `appel -> {annulé: bool, socket: Option<UnixStream>}`
    sous un seul verrou. `serve` crée l'entrée au moment où il enregistre l'appel en vol
    (avant `scope.spawn`). L'exécuteur d'attente (identifiant d'appel posé dans une
    variable de fil par le fil lancé par `serve`) enregistre un clone de son socket
    DÈS la connexion, avant l'authentification auxiliaire et la sonde (qui sont des
    échanges bloquants), et teste le drapeau « annulé » dans la même section critique.
    La notification d'annulation, sous le même verrou, pose le drapeau et appelle
    `shutdown(Both)` si un socket est déjà enregistré. Trois cas : annulation avant
    l'enregistrement (l'exécuteur voit le drapeau et s'arrête), pendant (le socket est
    fermé, l'échange bloquant échoue), après (l'entrée a disparu, sans effet) ;
  - à l'EOF ou à une erreur de lecture, une garde (`Drop`) déclarée dans la fermeture de
    `thread::scope` (`mcp.rs:131`) appelle `cancel_all` : drapeau « fermeture » global,
    tous les drapeaux à vrai, `shutdown(Both)` sur tous les sockets enregistrés.
    Elle s'exécute au retour de la fermeture, donc AVANT la jonction des fils
    (`mcp.rs:134` sort de la boucle sur EOF, `:213` ferme la portée qui attend les
    exécutions). Un enregistrement tardif voit le drapeau global et s'arrête ;
  - l'attente du daemon se termine, il la libère en 6 s (R3).
- **Preuve du défaut EOF** : `mcp.rs:134` fait `break` sur EOF ; `thread::scope` attend
  ensuite tous les fils lancés (`:213`). Un appel d'attente d'une heure garde donc le
  processus MCP et ses sockets ouverts pendant une heure après la fermeture de
  l'entrée.
- **Raison** : sans cela, un parent qui lance 8 attentes ne peut plus annuler ses
  tâches, et une annulation MCP ou la fermeture de l'hôte laisse une attente d'une
  heure ouverte.
- **Alternatives** : annuler en laissant le daemon deviner (impossible) ; augmenter 8
  (rejeté : déplace la limite).

## R9 - Délai d'outil des hôtes et listes d'outils

- **Constat** : le client MCP a un budget de 10 s par échange
  (`communication/client.rs:21`, `connect` `:937-940`). La session 132 a déjà ouvert une
  connexion avec un délai propre pour le lancement. L'attente exige
  `connect_until(now + wait_timeout + 10 s)` (`registered_connection_until`,
  `client.rs:1159`, aujourd'hui privée).
- **Constat** : montage Codex de Bridget : `wrapper.rs:5252-5270`, sans
  `tool_timeout_sec`. Liste des outils sûrs : `BRIDGET_SAFE_MCP_TOOLS: [&str; 20]`
  (`wrapper.rs:5035-5056`).
- **Décision** : ajouter les outils nouveaux à cette liste et `tool_timeout_sec=3700` au
  montage Codex de Bridget. Pour un hôte hors montage Bridget, documenter que
  `wait_timeout` doit rester sous le délai d'outil de l'hôte (question 1).
- **Si l'hôte coupe l'appel** : l'attente est une lecture, elle se rappelle sans effet.
- **À vérifier (T001)** : valeurs par défaut de Codex et de Claude Code.

## R10 - Attendre la réponse d'un `bridget_send`

- **Constat** : une demande suivie est une ligne `tracked_requests`
  (`idempotency.rs:580-590`, création `store/ledger_requests.rs:231-255`). Elle passe à
  `answered` à l'accusé de la réponse (`acknowledge_send_delivery`,
  `idempotency/send_delivery.rs:698-768`, appel `daemon.rs:7656-7690`), par le chemin
  hérité (`daemon.rs:17778`) ou par le guichet (`store/service_events.rs:380`, sans
  message de réponse).
- **Pourquoi pas le ledger** : la clé du ledger est `(id, target)` avec
  `INSERT OR REPLACE` (`store/ledger_requests.rs:687`). Les identifiants d'envoi
  sont uniques par `issuer_scope` seulement (`IdempotencyKey::new(scope, Send, id)`,
  `daemon.rs:8636`) : deux émetteurs peuvent réutiliser le même `id` vers le même
  destinataire, et une autre émission peut remplacer la ligne. Un lien `reply_message_id`
  vers le ledger n'est donc pas fiable.
- **Décision** : colonne `reply_summary TEXT` (nullable) sur `tracked_requests`, JSON
  `{id, from, summary, summary_truncated}` écrit dans le même `UPDATE ... SET
  state='answered'` que le passage de la demande à `answered`
  (`mark_answered_in_transaction`, `store/ledger_requests.rs:665-678`). Les chemins
  idempotent et hérité passent le résumé calculé depuis leur message ; le guichet passe
  `NULL`.
- **Réponse sans résumé** : `answered` avec `reply: null` et
  `reply_reason: "reply_unavailable"` (guichet, lignes antérieures à 151, enveloppe
  manquante : `send_delivery.rs:741-768`). Le contrat ne promet pas de réponse.
- **Attente** : trame `WrapperToDaemon::RequestWait { request_id, wait_timeout }`,
  réponse `RequestWaitResult`. Même compteur (R3). Incréments APRÈS commit à tous les
  écrivains de l'état d'une demande suivie : accusé idempotent (`daemon.rs:7672`),
  chemin hérité (`:17778`), annulation, et réponse du guichet. Ce dernier écrit
  `answered` dans la transaction du `Store` (`store/service_events.rs:380`), qui ne
  voit pas le compteur : le site d'appel `st.store.reply_guichet(...)`
  (`daemon.rs:16074`) incrémente après son retour (contre-revue ronde 2, point 4).
  T001(g) énumère tous les `UPDATE tracked_requests` pour ne rien oublier. L'échéance `deadline_at` de la demande borne l'attente, sans
  attendre le balayage des délais.
- **Alternatives** : lien vers le ledger (rejeté, ci-dessus) ; balayage de
  `send_deliveries` (O(n)) ; résumé dans une table neuve (rejeté : une colonne suffit).
- **Limite (FR-030)** : la réponse est déjà injectée au demandeur ; elle sera visible
  deux fois. Session156.

## R11 - Compatibilité avec un ancien daemon

- **Constat** : un daemon qui ne décode pas une trame journalise et passe à la
  suivante, sans rien répondre (`daemon.rs:5140-5156`, sauf trame du guichet). Une
  variante inconnue de `NativeDelegationRequest` est donc muette. Une capacité de
  client inconnue est aussi muette : `ClientCapability` est une énumération fermée
  (`protocol.rs:187-190`). Le client attendrait `wait_timeout + 10 s`.
- **Décision** : la capacité n'est pas négociée par `ClientHello`. Le daemon annonce
  `features: ["wait_v1"]` dans `DaemonIdentityReport` (`protocol.rs:4187`, champ
  `#[serde(default)]` ajouté). Le client envoie d'abord `DaemonIdentityRequest`
  (trame ancienne, sans enregistrement, `daemon.rs:14384`), budget 3 s. Sans
  `wait_v1`, `wait_unsupported` aussitôt. Les trames nouvelles ne partent qu'après.
- **Preuve avec le vrai décodeur** : T018 compile le daemon de la base 150 dans un
  dossier temporaire hors du dépôt et exécute le client 151 contre lui.
- **Alternatives** : essayer la trame et attendre (rejeté) ; négocier dans `ClientHello`
  (rejeté, muet) ; Catalogue comme sonde (rejeté : lourd et peut refuser).

## R12 - Accusé de lecture explicite (P2)

- **Décision** : `bridget_task_ack` écrit `acknowledged_at` (une fois) ; état de boîte
  `acknowledged`. Effets : aucun sur la remise, le message, le nettoyage, la purge, Lineage.
- **Raison** : la demande utilisateur distingue « remis » et « lu ». Seul le parent peut
  dire qu'il a lu ; il le dit par un appel explicite fait après avoir reçu le contenu.
- **Alternatives** : lecture implicite par l'attente (rejetée, R6) ; ne pas faire « lu »
  (possible : question 2).
- **Priorité** : dernière (T024, T025), supprimable.

## R13 - Droits, pause, purge

- **Droits** : `Wait` suit `Status` : `can_own_task` (`native_delegation.rs:136-171`),
  mêmes `owner` et `instance`. Un refus rend `task_unavailable` pour tout l'appel.
- **Pause** : l'attente ne passe pas par `admit_autonomous_effect` (ADR 027), car elle
  n'écrit rien.
- **Capacité** : 64 attentes au plus dans le daemon, du même ordre que les 128
  observateurs Lineage (`native_lineage.rs`, `watch`).
- **Purge** : la purge 150 reste inchangée. `disposed` est lu comme `task_unavailable`.

## R14 - Hors périmètre constaté

- `bridget_task_cancel` reste inchangé
  (`native148_cancel_stops_native_descendants_before_parent`).
- Liste des boîtes d'un parent sans identifiants : session158.
- Reprise des attentes après un redémarrage du daemon : un appel ouvert meurt avec le
  daemon ; le client le rappelle. Reprise des missions : session152.

## Revalidation T001 des délais d'hôtes (2026-10-11)

Inspection locale préalable : configuration Codex présente, `tool_timeout_sec`
observé pour un serveur étranger ; elle ne fixe pas le montage Bridget. Les
documentations officielles confirment :
- Codex : délai par outil MCP de 60 s par défaut, réglable par serveur avec
  `tool_timeout_sec`. Source : https://developers.openai.com/codex/config-reference/
- Claude Code : `MCP_TOOL_TIMEOUT` de 100000000 ms par défaut ; le stdio a aussi
  un délai d'inactivité de 1800000 ms dans les versions documentées. Le transport
  stdio n'a pas le minuteur HTTP de 60 s. Une attente longue peut donc encore être
  interrompue par l'hôte et rappelée sans effet.
  Source : https://code.claude.com/docs/en/env-vars
