# Session151 - Attente et boîte de fin de tâche

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 151-attente-fin-de-tache
Titre: Attente d'un résultat dans le même tour (lecture pure), boîte de fin de tâche, résultat structuré
Statut: Implemented
Priorité: P1
Tâches: 27 (27 cochées)
Tests: W004 avant T027 : 2046 PASS, 8 échecs hérités, 72 ignorés ; T027 distinct : 1 PASS, 0 FAIL, 0 ignoré (voir validation/tests-151.md)

Résumé:
- Contexte: Un parent Bridget ne peut pas attendre un résultat. Il délègue, rend la main, puis reçoit un message plus tard. Pour attendre la réponse d'un `bridget_send`, il n'a aucun outil non plus. La remise du résultat ne dit pas son état exact.
- Objectif: Attendre sans polling (déléguer puis attendre, ou envoyer puis attendre) par des lectures pures, garder une boîte de fin de tâche dont l'état « remis » repose sur l'accusé existant, rendre un résultat structuré.
- Dépendances: SPEC-148, SPEC-149, SPEC-150 (délai par tâche `reply_timeout`, preuve `delivery_acked`, lecture de phase de remise, pause à l'admission)

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-151-attente-fin-de-tache` (worktree dédié, socle 150 copié sans commit)
**Created**: 2026-10-11
**Status**: Implemented
**Priority**: P1
**Dependencies**: SPEC-148, SPEC-149, SPEC-150

État après T027 : 27 tâches cochées sur 27. Le test SC-009 mesure le retour
après le vrai `DeliverAcked` normal en 406 µs et 357 µs dans deux scopes qui
partagent un identifiant de réponse et un destinataire. Les résumés sont exacts
et stables. Converge 1 avait trouvé ce manque de preuve, sans bug de production
prouvé. W004 reste la suite complète avant T027 ; son total ne devient pas 2047.
Le format global reste RC1 avec549 hunks anciens. Dix-huit corrections de
style dans dix fichiers ont une normalisation rustfmt inchangée. Le principal
a qualifié les deux contextes restants. Le GO des captures r2 est reçu.

Date : 2026-10-11. Accord utilisateur : feuille de route « Bridget cinq étoiles »
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`). Critères
visés : Attente 2 vers 5, Livraison 4 vers 5, Suivi 4 vers 5. Source de la
comparaison : `specs/150-socle-delegation/research-t3-bridget.md` (mécanismes
A1, B1, B3, B4). Contre-revue : `adversarial-review-codex.md`.

**Principe directeur (révision).** Une attente ne change jamais l'état de
livraison. Elle lit. Seule la remise par message prouve la réception, avec
l'accusé qui existe déjà (phase `acked`). Le parent peut donc voir un résultat
deux fois (retour de l'attente, puis message). Il ne le voit jamais zéro fois.

## Besoin et périmètre

Aujourd'hui, la délégation native est asynchrone seulement. Le parent appelle
`bridget_delegate`, rend la main, et reçoit plus tard un message
`native-result-<task>` (`crates/bridget-daemon/src/daemon/native_delegation.rs`,
fonction `tick`, `:954-1295`). Trois défauts en découlent.

**Défaut 1 - Aucune attente dans le même tour.** Aucun outil MCP n'attend un
résultat. Le seul moyen est de relire `bridget_task_status`
(`crates/bridget-daemon/src/delegation_mcp.rs:13`), donc de faire du polling.
`bridget_send` rend la main à l'admission
(`crates/bridget-daemon/src/mcp.rs:673-860`) et rien n'attend la réponse.

**Défaut 2 - L'état de la remise est illisible.** La tâche porte `result_sent`
et `failure_sent`. Ces champs deviennent vrais même quand la remise est
indéterminée (contre-revue 150, ronde 1, objection 5). Le parent ne voit pas si
son résultat est à envoyer, en vol ou accusé.

**Défaut 3 - Résultat brut.** `task_view`
(`native_delegation.rs:68-86`) rend le texte entier du résultat et l'état
interne. Il n'y a ni résumé stable, ni état de travail simple, ni référence au
détail.

Cette session ajoute :

1. `bridget_task_wait` : attente d'une ou plusieurs tâches, pilotée par
   événements, 10 min par défaut, 60 min au plus, lecture pure ;
2. `bridget_request_wait` : attente de la réponse d'une demande suivie
   (`bridget_send` avec `reply: true`), lecture pure ;
3. une boîte de fin de tâche calculée à partir de faits durables ;
4. un résultat structuré (résumé stable, `work_state`, référence au détail) ;
5. le regroupement des résultats arrivés ensemble dans une réponse d'attente ;
6. (P2, en dernier) `bridget_task_ack` : accusé de lecture explicite du parent,
   informatif.

**Hors périmètre** :

- toute suppression du message de résultat ou de l'avis d'échec, tout
  nettoyage ou toute preuve de remise déclenchés par une attente ;
- reprise après panne, gel des délais pendant la pause (session152) ;
- remontée des demandes de l'enfant, état `waiting_for_input` (session155) ;
- guidage, interruption, file d'attente exposée, suppression de la copie d'une
  réponse injectée (session156) ;
- liste globale des boîtes d'un parent, vue d'ensemble (session158) ;
- mode `wait` dans `bridget_delegate` et attente fusionnée dans `bridget_send`
  (deux appels suffisent ; voir `research.md` R1 et R10) ;
- CLI humaine : aucune commande nouvelle.

## Scénarios utilisateur

### US1 - Déléguer puis attendre dans le même tour (P1)

Un parent appelle `bridget_delegate`, puis `bridget_task_wait` avec le
`task_id` reçu. L'appel reste ouvert. Dès que l'enfant a rendu son résultat et
que ses propres enfants ont fini, l'appel rend l'état de la tâche avec son
résumé. Le parent n'a fait aucun appel de relecture. Pour le texte entier, il
appelle `bridget_task_status` ou attend le message de résultat.

Si le délai expire, l'appel rend `timed_out: true`. L'enfant n'est pas annulé.
Quand il finit plus tard, le parent reçoit le résultat par message, comme avant.

### US2 - Résultat structuré (P1)

Chaque tâche rendue par `bridget_task_wait` et `bridget_task_status` porte un
`work_state` (`working`, `waiting_for_children`, `result_available`, `failed`,
`cancelled`), un résumé stable de 2048 octets au plus, la taille du résultat et
une référence au détail. Les clés des contrats 148 à 150 restent inchangées.

### US3 - Boîte de fin de tâche (P1)

La boîte d'une tâche conservée rend des états lisibles : `pending` (à remettre),
`claimed` (remise engagée), `delivered` (accusée par le destinataire).
Le parent lit cet état dans la réponse. Seule la remise par message change ces
faits. L'attente et le statut les lisent sans les écrire.

Après sa purge par la rétention 150, la tâche n'est plus observable.
Son identifiant rend `task_unavailable`, sans entrée de boîte.
La purge n'ajoute aucune valeur à `mailbox.state`.

### US4 - Regrouper les résultats arrivés ensemble (P2)

Un parent attend trois enfants. Trois résultats arrivent pendant le même
intervalle. Une seule réponse d'attente porte les trois. Hors attente, les
messages `native-result-*` restent regroupés en un seul tour par le pont
(session 114 : 8 messages ou 32 Kio, `t3code.rs:50-51`, `:2073-2092`).

### US5 - Attendre la réponse d'un `bridget_send` (P2)

Un agent envoie une demande avec `reply: true` par `bridget_send`. Il reçoit
`id` et `issued_at` tout de suite. Il appelle ensuite `bridget_request_wait`
avec cet `id`. L'appel rend l'état de la demande (`answered`, `cancelled`,
`timed_out`, `open`) et, si elle est répondue, un résumé figé de la réponse.
Si l'hôte coupe l'appel, l'agent rappelle `bridget_request_wait` : rien n'est
renvoyé.

### US6 - Aucun changement pour l'existant (P1)

Un appel sans paramètre nouveau garde le comportement 150. Les tâches en cours
au moment de la mise à jour restent lisibles : les champs nouveaux sont
optionnels. Un client 151 face à un daemon plus ancien refuse l'attente en
moins de 5 s au lieu de rester bloqué.

### US7 - Accusé de lecture explicite (P2, en dernier)

Le parent appelle `bridget_task_ack` avec les tâches qu'il a lues. La boîte
passe à `acknowledged`. Cet accusé est informatif : il ne supprime aucun
message, ne prouve aucune remise, ne déclenche aucun nettoyage et ne change pas
la purge.

## Exigences fonctionnelles

**Attente d'une tâche (`bridget_task_wait`)**

- **FR-001** : `bridget_task_wait` accepte `task_ids` (1 à 16 chaînes, sans doublon)
  et `wait_timeout` (entier, secondes, 1 à 3600, défaut 600). Tout autre champ est
  refusé.
- **FR-002** : Seul le propriétaire attend ses tâches. Un identifiant inconnu,
  purgé ou d'un autre parent rend `task_unavailable` pour tout l'appel. Aucun
  résultat partiel.
- **FR-003** : L'appel rend la main dès qu'au moins une tâche demandée est
  terminée (`result_available`, `failed` ou `cancelled`). Il rend alors toutes
  les tâches terminées à cet instant, dans l'ordre de `task_ids`, et la liste
  `pending` des autres.
- **FR-004** : L'attente ne fait AUCUNE écriture : ni état de tâche, ni
  `result_sent`, `failure_sent`, `delivery_acked`, ni nettoyage, ni message.
- **FR-005** : Le curseur d'événements est relu avant CHAQUE lecture de tâche, sans
  tenir le verrou du curseur pendant l'accès à l'état. Un résultat publié entre
  une lecture et le sommeil suivant n'est jamais manqué.
- **FR-006** : À l'expiration, l'appel rend `timed_out: true`. Aucune tâche n'est
  annulée ni modifiée.
- **FR-007** : Le réveil suit les écritures de tâche (événement). Un contrôle
  interne sert de filet et de détection de client fermé. Chaque sommeil dure
  `min(5 s, délai restant de l'attente)`, et pour une demande
  `min(5 s, délai restant, échéance restante de la demande)`. Une attente de 1 s
  rend donc la main en moins de 1,5 s. L'agent ne fait aucun polling.
- **FR-008** : Une valeur `wait_timeout` hors 1 à 3600 rend
  `wait_timeout_out_of_range`, sans attente. Plus de 64 attentes simultanées dans
  le daemon (tâches et demandes) : `wait_capacity`.
- **FR-009** : Si le client ferme sa connexion ou si le daemon s'arrête, l'attente
  se termine dans les 6 s.
- **FR-010** : L'attente reste possible pendant la pause du référent.
- **FR-011** : Le client MCP garde sa connexion ouverte pendant `wait_timeout`
  plus 10 s, au lieu du budget fixe de 10 s
  (`crates/bridget-daemon/src/communication/client.rs:21`).

**Façade MCP**

- **FR-012** : La façade MCP d'un processus accepte 4 appels d'attente en cours au
  plus (`MAX_WAITING_TOOL_CALLS`). Un cinquième rend `wait_capacity` aussitôt. Les
  8 créneaux d'appels en vol (`mcp.rs:33`) gardent donc 4 créneaux pour les
  appels de contrôle (`bridget_task_cancel`, `bridget_task_status`, `bridget_cancel`).
- **FR-013** : Une notification MCP `notifications/cancelled` sur un appel
  d'attente ferme la connexion daemon de cet appel. L'exécution s'arrête, le
  créneau est libéré, l'attente du daemon se termine. Cela vaut aussi quand la
  notification arrive avant que l'exécution ait enregistré son socket : le
  registre (appel, drapeau d'annulation, socket) est mis à jour sous un seul
  verrou, le socket est enregistré dès la connexion, avant l'authentification et la
  sonde, et l'exécuteur vérifie le drapeau dans la même section critique.
- **FR-038** : À la fin de l'entrée MCP (EOF ou erreur de lecture), la façade marque
  toutes les attentes annulées et ferme leurs sockets AVANT de joindre les fils
  d'exécution. Un socket enregistré après cette fermeture est refusé aussitôt. Le
  processus MCP se termine en 6 s au plus, quelle que soit la durée de l'attente.

**Résultat structuré**

- **FR-014** : `task_view` et les entrées d'attente ajoutent `work_state`,
  `summary`, `summary_truncated`, `result_bytes`, `detail` et `mailbox`. Les clés
  existantes gardent leur forme. Les entrées d'attente ne portent pas `result` :
  le texte entier se lit par `bridget_task_status`.
- **FR-015** : `work_state` vaut `working` pour `queued`, `starting`,
  `mission_pending`, `working` et `cancelling` ; `waiting_for_children`,
  `result_available`, `failed` et `cancelled` pour les états du même nom.
- **FR-016** : `summary` est le début du résultat, 2048 octets au plus, coupé
  sur une frontière UTF-8. Il n'existe que pour `result_available`. Le résultat
  ne change plus après publication : le résumé est donc stable.
- **FR-017** : `detail` vaut `{"tool":"bridget_task_status","task_id":…}` pour les
  tâches `result_available` et `failed`, sinon `null`.

**Boîte de fin de tâche**

- **FR-018** : `mailbox.state` est calculé : `pending` (résultat ou avis prêt, non
  remis), `claimed` (remise engagée, phase non accusée), `delivered` (phase
  accusée ou `delivery_acked`). Une tâche non terminée ou `cancelled` n'a pas de
  boîte (`mailbox` vaut `null`). Une tâche purgée par la rétention 150 n'est
  plus observable ; son identifiant rend `task_unavailable`, sans entrée de boîte.
- **FR-019** : Seule la remise par message écrit `result_sent`, `failure_sent` et
  `delivery_acked`. Les appels de la session 151 (`bridget_task_wait`,
  `bridget_request_wait`, `bridget_task_status`) n'écrivent aucun de ces champs.
- **FR-020** : Le message `native-result-<task>` (ou `native-failure-<task>`) part
  par le chemin 150, qu'une attente soit ouverte, fermée ou expirée. `tick` ne
  connaît pas les attentes.
- **FR-021** : Le parent peut recevoir un résultat deux fois (attente, puis
  message). Le message n'est jamais supprimé. Cette limite est documentée.
- **FR-022** : Un résultat remis ou non ne dépend pas de l'attente pour la purge
  150 : la condition reste `delivery_acked` posé par l'accusé de la remise.

**Regroupement**

- **FR-023** : Les tâches terminées au même moment sont rendues ensemble dans une
  seule réponse d'attente (FR-003).
- **FR-024** : Les messages `native-result-*` restent de la famille « message sans
  réponse attendue » du pont T3. Un test fige ce fait.

**Attente d'une réponse de `bridget_send` (`bridget_request_wait`)**

- **FR-025** : `bridget_request_wait` accepte `request_id` (l'`id` rendu par
  `bridget_send`) et `wait_timeout` (1 à 3600, défaut 600). `bridget_send` ne
  change pas.
- **FR-026** : Seul l'émetteur de la demande l'attend. Sinon `request_unavailable`.
- **FR-027** : L'appel rend `state` (`answered`, `open`, `timed_out`, `cancelled`),
  `timed_out_wait` et `reply`. Il rend la main quand la demande quitte `open` ou
  quand son échéance `deadline_at` passe.
- **FR-028** : Le résumé de réponse est écrit dans `tracked_requests.reply_summary`
  dans la transaction qui passe la demande à `answered` (id du message, émetteur,
  1024 octets de corps au plus, drapeau de troncature). Aucune recherche dans le
  ledger. Une demande `answered` sans résumé (réponse via guichet, ligne antérieure
  à 151) rend `reply: null` et `reply_reason: "reply_unavailable"`.
- **FR-039** : Toute écriture qui change l'état d'une demande suivie incrémente le
  compteur d'événements après son commit : accusé idempotent, chemin hérité,
  annulation, et réponse du guichet (`daemon.rs:16074`, qui passe par
  `store/service_events.rs:380`). Une attente d'envoi se réveille sur la réponse du
  guichet en moins de 2 s, sans le filet.
- **FR-029** : Un rappel de `bridget_request_wait` après une coupure de l'hôte
  reprend l'attente de la même demande. Aucun envoi n'a lieu.
- **FR-030** : Limite connue : la réponse est déjà injectée au demandeur par le
  chemin normal avant que la demande passe à `answered`. Elle est visible deux fois.

**Compatibilité**

- **FR-031** : Le daemon annonce `features: ["wait_v1"]` dans `DaemonIdentityReport`
  (champ ajouté, absent chez un ancien daemon). Avant toute attente, le client
  interroge `DaemonIdentityRequest` avec un budget de 3 s. Sans `wait_v1`, l'appel
  est refusé aussitôt (`wait_unsupported`) et aucune trame nouvelle n'est envoyée.
- **FR-032** : `bridget_capabilities` annonce `wait_timeout_default_secs` (600) et
  `wait_timeout_max_secs` (3600).
- **FR-033** : Les outils nouveaux figurent dans la liste des outils sûrs montés
  par Bridget. Le montage Codex de Bridget fixe `tool_timeout_sec` à 3700.
- **FR-034** : Un appel sans paramètre nouveau reste identique à 150. Les clés
  existantes de `task_view` ne changent pas.
- **FR-035** : Les outils, refus et limites nouveaux sont documentés dans
  `docs/delegation-native.md` et dans la skill Bridget.

**Accusé de lecture explicite (P2)**

- **FR-036** : `bridget_task_ack` accepte `task_ids` (1 à 16). Il écrit
  `acknowledged_at` (une fois) sur les tâches `result_available` ou `failed` du
  propriétaire ; `task_unavailable` ou `task_not_ready` sinon, pour tout l'appel.
- **FR-037** : `acknowledged_at` fait passer `mailbox.state` à `acknowledged`. Il ne
  change ni la remise, ni le message, ni le nettoyage, ni la purge, ni la
  projection Lineage.

## Critères de succès

- **SC-001** : Un parent qui délègue puis attend obtient l'état et le résumé avec 0
  appel de relecture. Délai entre la capture de la réponse enfant et le retour de
  l'attente : 2 s au plus.
- **SC-002** : Après une attente expirée à 1 s, l'enfant continue, termine, et le
  parent reçoit exactement 1 message de résultat.
- **SC-003** : Après une attente qui a rendu un résultat, le message de résultat
  part quand même exactement 1 fois (jamais 0). Les champs `result_sent`,
  `failure_sent` et `delivery_acked` ne sont écrits que par `tick` et l'accusé.
- **SC-004** : Trois tâches terminées dans le même intervalle d'attente : 1
  réponse, 3 entrées.
- **SC-005** : 100 répétitions d'un résultat publié entre deux lectures : 100
  retours, 0 expiration.
- **SC-006** : Les états `pending`, `claimed`, `delivered` sont observés dans cet
  ordre sur un test de bout en bout.
- **SC-007** : Les tests 148, 149 et 150 restent verts. Les clés de `task_view`
  d'avant 151 sont toutes présentes.
- **SC-008** : Avec 64 attentes ouvertes et des écritures de tâches étrangères
  continues, un appel `bridget_task_cancel` et un `bridget_task_status` aboutissent
  en moins de 500 ms. Sans événement, une attente ne lit la base qu'une fois par
  tranche de 5 s.
- **SC-009** : `bridget_request_wait` rend `answered` 2 s au plus après l'accusé de
  la réponse, avec la bonne réponse même si deux émetteurs de scopes différents
  utilisent le même identifiant de message vers le même destinataire.
- **SC-010** : Un client qui ferme sa connexion ou annule l'appel MCP libère
  l'attente du daemon en 6 s au plus, que l'annulation arrive avant, pendant ou
  après l'enregistrement du socket ; 4 attentes ouvertes laissent 4 créneaux aux
  appels de contrôle de la façade.
- **SC-012** : À l'EOF de l'entrée MCP avec 4 attentes d'une heure ouvertes, le
  processus MCP se termine en 6 s au plus et les 4 attentes du daemon sont
  libérées.
- **SC-013** : Une attente de 1 s rend la main en moins de 1,5 s. Une attente de
  demande se réveille sur une réponse du guichet en moins de 2 s avec un filet réglé à
  60 s.
- **SC-011** : Un client 151 contre un daemon de la base 150 rend `wait_unsupported`
  en moins de 5 s (mesuré avec le vrai binaire ancien).

## Entités et hypothèses

**Entités**

- **Attente** : appel ouvert d'un propriétaire, lecture pure, avec délai.
- **Boîte de fin de tâche** : vue calculée du résultat ou de l'avis d'échec d'une tâche.
- **Résultat structuré** : `work_state`, résumé, taille, référence au détail.
- **Curseur d'événements** : compteur partagé incrémenté à chaque écriture de tâche.
- **Résumé de réponse** : copie figée du début de la réponse, écrite avec le passage à `answered`.
- **Accusé de lecture (P2)** : déclaration explicite du parent, informative.

**Hypothèses**

- La session 150 est livrée : `reply_timeout`, `delivery_acked`, lecture de phase
  d'une remise (tâche T005 de 150), refus `referent_paused`, rétention.
- Le délai de 600 s par défaut et 3600 s au plus reprend T3 (10 min, 60 min).
- Un hôte peut couper un appel d'outil avant `wait_timeout` (Codex : 60 s par
  défaut selon sa documentation, à vérifier en T001). L'appel suivant reprend
  l'attente ; rien n'est perdu puisque l'attente ne consomme rien.
- Le résultat publié ne change plus (docs/delegation-native.md:43).
- Les tests suivent la recette `umask 077` (quickstart).
- La livraison suit les sessions 147 à 150 : fichiers sur disque, sans relance de service.

## Questions pour l'utilisateur

1. **Délai par défaut et délai des hôtes.** Valeur choisie : 600 s par défaut,
   comme T3. Un parent Codex monté par T3, hors du lanceur Bridget, garde le
   délai d'outil de sa configuration (60 s par défaut) et verra l'appel coupé
   avant 600 s ; il rappelle l'attente. Faut-il demander à T3 de transmettre
   `tool_timeout_sec` ? Défaut retenu : documenter, et fixer 3700 s seulement dans
   le montage Bridget.
2. **Garder l'accusé de lecture explicite `bridget_task_ack` (US7) ?** Sans lui, la
   boîte distingue « à remettre », « en vol » et « remis » mais pas « lu ».
   Défaut retenu : oui, en dernière priorité (T024 et T025), supprimable sans
   effet sur le reste.
3. **Accepter le doublon (attente puis message) ?** Défaut retenu : oui. Une
   suppression automatique du message exigerait une preuve de réception que
   l'attente ne peut pas donner (réponse écrite avant d'être reçue, appel annulé
   par l'hôte). Une suppression future devrait reposer sur un accusé explicite.
