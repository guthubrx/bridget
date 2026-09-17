# Plan 102 — Fils inter-agents

État : préparation documentaire, sans implémentation. Spécification : spec.md.
Racine de travail : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents
Base Git examinée : 1738a072 ; complément examiné en lecture seule :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3

## 1. Résultat attendu et règle de lecture

Un seul stockage partagé dans le daemon ; des alertes courtes uniquement aux
membres explicitement visés ; les agents récupèrent les nouveautés par un outil.
Les trois faits suivants ne doivent jamais être confondus :

1. `posted` : la contribution est enregistrée durablement.
2. `dispatched` : l'alerte a été injectée dans la conversation destinataire.
3. `acknowledged` : le membre a confirmé une plage fournie par `read`.

Aucun de ces faits n'atteste une compréhension ou une tâche terminée.

## 2. Contexte technique et dépendances

Rust 2024, crates bridget-core / bridget-transport / bridget-daemon, JSONL sur
socket Unix, SQLite rusqlite, serde/serde_json, UUID et SHA-256 déjà disponibles.
Pas de nouveau service, broker, dépendance, interface Web ou appel LLM.

Socle fonctionnel : 089 (identités/routage/journaux), 094 (parité), 099 (preuve
auxiliaire et idempotence de remise), 100 (lecture bornée et observations).
Compatibilité obligatoire : 097 (Claude interactif), 098 (T3), 101 (identité MCP
T3 et observations attestées). Les modifications non commitées de 101 ne sont PAS
présentes dans cette branche. T001 doit identifier leur commit intégré ou obtenir
un arbitrage d'intégration avant de modifier les composants communs. Ne pas
réimplémenter 101, copier son arbre, lancer un rebase sur une branche sale ou
confondre version installée et version Git. Cela n'empêche pas la préparation.

## 3. Périmètre technique fixé

- Un outil MCP `bridget_thread` et une famille CLI `bridget thread` ; actions
  `create`, `list`, `show`, `post`, `read`, `ack`, `history`, `close`.
- Un contrat versionné `ThreadRequest` / `ThreadResult`, avec enums fermées.
- Un module métier `threads.rs` pour validation, appartenance, curseurs et
  sollicitations ; un sous-module SQL `store/threads.rs` sur la connexion Store.
  Cette séparation suit l'organisation réelle de Store ; pas de couche repository
  générique, service façade, plugin ou bus d'événements supplémentaire.
- Six tables métier dans la base existante : `discussion_threads`,
  `discussion_members`, `discussion_entries`, `thread_operations`,
  `thread_reads`, `thread_wakes`. Détails et invariants : data-model.md.
- Extension de `BridgetMessage` par métadonnée typée facultative
  `thread_notice`, absente des DM historiques ; capacité de remise
  `ThreadNoticeV1` négociée explicitement par les wrappers compatibles.
- Pas d'opération `summary` : la synthèse est un usage documenté de `history`
  et éventuellement `post`, effectué par l'agent déjà présent.

## 4. Réutilisation de l'existant

Preuves détaillées dans reuse-audit.md. Points de départ vérifiés (lignes main,
sauf mention 101) :

| Besoin | Existant à étendre | Ce qu'il ne faut pas faire |
|---|---|---|
| Identité CLI/MCP | communication/client.rs:766, mcp_identity.rs:109 ; daemon.rs `live_connection_identity` (101:7339) | accepter `actor`, `owner` ou un simple nom comme preuve |
| Client commun | communication/client.rs:19 `observation_request`, budget10s | doubler règles dans CLI et MCP |
| Persistance | store.rs:32 et organisation store/ledger_requests.rs | ouvrir une nouvelle base de fils |
| Remise | wrapper.rs:572 `IdempotentDeliveryTracker`, ReceiptStore ; idempotency/send_delivery.rs | réinventer transport/reçu, ou assimiler ACK d'injection à lecture |
| Lecture | attach.rs:30, pagination bornée et signalement d'incomplétude | détourner le journal d'exécution en conversation collective |
| Façades | cli.rs:178, mcp.rs:438 | exposer toutes les commandes via un outil shell générique |
| Annuaire | bridget_who, router.rs:129 | router à partir d'un nom fournisseur ou d'un texte `@` |
| Sécurité du protocole | daemon.rs:9530/9610, matrices de rôles | laisser la nouvelle requête accessible aux rôles publics |

Les chemins abrégés de ce tableau sont relatifs à `crates/bridget-daemon/src/`,
sauf `router.rs` dans `crates/bridget-core/src/`.

Deux limites imposent de petits états spécialisés :

1. `idempotency_records` n'accepte que Send/Spawn ; `send_deliveries` porte un
   destinataire unique. Un dépôt de fil n'est pas un Send. `thread_operations`
   porte son idempotence transactionnelle, sans modifier le sens des opérations099.
2. `observation.once` est consommé avant une remise best effort, parfois perdue.
   L'appartenance au fil et les sollicitations durables ne peuvent pas s'appuyer
   dessus. On réutilise la livraison, pas le cycle de vie de l'abonnement.

## 5. Publication et atomicité — algorithme imposé

Le daemon est l'autorité de validation. Aucun accès réseau/fournisseur dans la
transaction SQL ni sous le verrou de l'état partagé.

```text
post(actor attesté, request)
  contrôler version + schéma fermé + tailles + UUID
  ouvrir transaction Store
  contrôler membre et rechercher (actor, operation_id)
  si rejeu exact : rendre le reçu sauvegardé, même si fil clos depuis
  si même clé autre enveloppe : refuser sans mutation
  contrôler fil ouvert + reply_to local au fil + mentions membres
  éventuellement confirmer ack_receipt (mêmes règles que ack)
  allouer seq = last_seq + 1, insérer une seule entrée
  pour chaque cible dédupliquée hors auteur : pending_seq = max(pending_seq, seq)
  sauvegarder reçu posted + empreinte enveloppe dans thread_operations
  commit
  signaler au dispatcher les paires destinataire/fil concernées
  rendre posted + état initial des sollicitations (pas une promesse de remise)
```

`Store` et `IdempotencyStore` ouvrent actuellement deux connexions SQLite vers
le même fichier (daemon.rs:2880/2886). Appeler l'un puis l'autre ne constitue PAS
une transaction commune. L'intention `thread_wakes` est commise avec l'entrée ;
la projection en livraison idempotente vient ensuite. Après crash entre les deux,
elle reprend la même génération/clé. Ne pas ajouter une transaction distribuée.

Projection099 concrète (source daemon.rs:4140–4220, à réutiliser comme patron
technique sans activer sa logique de supervision/reprise métier) :

- Utiliser l'issuer_scope interne durable fourni par IdempotencyStore::supervisor_scope,
  pas le scope d'une connexion MCP ni un namespace reconstruit au redémarrage.
- Clé métier de livraison `thread-wake:<thread_id>:<agent_id>:<generation>` ;
  OperationKind::Send convient ICI car il s'agit d'une alerte à une seule cible,
  contrairement au post collectif. Clé inaccessible comme autorité au client public.
- Figer dans thread_wakes avant projection : clé, delivery_id, instance destinataire,
  delivery_generation obtenue par next_delivery_generation(), issued_at et échéance.
  La génération du fil et delivery_generation099 sont deux champs distincts.
- Message from="bridget", origin=System, intent=TriggerTurn, reply=false,
  aucune in_reply_to/référence métier ; body neutre et thread_notice typée. Ne pas
  faire signer cette alerte par l'auteur de la dernière contribution.
- Réutiliser canonical_send, reserve, begin_send_delivery et le dispatch099 ;
  sérialiser l'enveloppe complète avec sa notice. issued_at/deadline sont figés,
  jamais recalculés au retry. Rejouer reserve/lookup selon leurs issues, sans
  appeler begin_send_delivery une seconde fois sur une remise déjà préparée.
- Ne pas confondre conservation de clé099 (horizon existant7jours) et délai
  d'injection de l'alerte (120s). Réserver selon l'horizon099 existant ; borner
  SendDelivery.expires_at à min(échéance120s, expiration de réservation), et fixer
  message.deadline_at au même délai d'injection. Les tests prouvent qu'aucune
  injection nouvelle n'arrive après ce délai ; les reçus historiques restent
  consultables selon leur horizon. Ne jamais réduire le TTL global099 à120s.

La projection stocke son résultat dans thread_wakes après la transaction099 ;
un crash dans cette fenêtre récupère par la même clé et la même instance. Aucun
reroutage d'une remise possiblement injectée vers une autre instance du membre.

Deux adaptations099 sont indispensables, vérifiées dans le code existant :

1. communication.rs:64 construit canonical_send champ par champ, pas par serde
   automatique. Ajouter à la FIN, seulement si thread_notice est Some, un domaine
   distinct `bridget/thread-notice/v1` puis version/thread_id/through_seq/generation
   encodés par les helpers de champs existants. None garde EXACTEMENT les anciens
   octets. Test golden historique inchangé et changement d'un champ notice change
   le canon. Sans cela, le rejet d'enveloppe différente serait incomplet.
2. idempotency/send_delivery.rs:176 propose une réaffectation générale des remises
   dispatching vers une nouvelle instance (appels daemon.rs:4044/5949, lignesmain).
   Exclure les remises portant une thread_notice de cette réaffectation, en
   inspectant leur enveloppe durable typée, et garder le comportement des DM.
   Faire la sélection et l'UPDATE de façon transactionnelle/batchée, pas N+1.
   Aucun lien vers une nouvelle table ou nouveau schéma099 requis : message_bytes
   est déjà présent. Après remplacement d'instance, issue inconnue/orpheline
   visible ; nouvelle mention ultérieure suit la règle de génération décrite.
   V25/V27 doivent provoquer une reconnexion avec une AUTRE instance pour vérifier
   cette exclusion, pas seulement une reconnexion réseau du même wrapper.

Une erreur de précondition ne consomme pas operation_id ; un succès est conservé
dans thread_operations pendant toute la vie du fil. Création et clôture suivent
la même règle d'enveloppe canonique et de rejeu. L'empreinte inclut action,
version, thread_id, corps exact, cibles normalisées, reply_to et ack_receipt ;
ne pas normaliser le corps en changeant espaces ou Unicode. Si close reçoit une
nouvelle clé pour un fil déjà clos, retourner thread_closed sans enregistrer
une nouvelle opération. Seul le rejeu de la clé de clôture originale renvoie
closed : au plus entrées+2 opérations conservées par fil (create, posts, close).

## 6. Sollicitations ciblées, coalescence et livraison

`post.notify` est obligatoire : `[]`, liste d'UUID, ou chaîne `"all"`. Le daemon
ne parse JAMAIS le corps pour extraire des `@`. Le CLI peut résoudre des noms
explicites fournis à `--notify`, via l'annuaire, avec refus d'ambiguïté. Dans les
exemples humains `@B` signifie cette cible structurée, pas une regex sur le texte.

La ligne thread_wakes est unique par fil/membre, avec au plus une génération de
livraison active et un watermark pending_seq. Avant le départ, plusieurs mentions
se regroupent. À la réservation du départ, figer `active_through_seq`,
`generation`, `delivery_id`, `issued_at`, `expires_at` ; l'enveloppe ne change plus.
Une mention postérieure augmente pending_seq mais ne modifie pas l'envoi figé.

Cycle : pending → in_flight → dispatched / refused / outcome_unknown.
Absence, DND ou capacité manquante donnent un motif d'attente, sans lancement.
Une réception fournisseur douteuse reste outcome_unknown ; pas de nouvel ID ou
de nouvelle génération pour rejouer LA MÊME sollicitation. Les résultats099 font
autorité. Après expires_at, figer cette issue ; seulement un pending_seq STRICTEMENT
supérieur à active_through_seq et au curseur peut ouvrir une génération suivante.
C'est une nouvelle mention, pas un retry. Garder la dernière borne/génération
inconnue pour show ; son ACK tardif ne change jamais l'état de la nouvelle.
Sans nouvelle mention, aucun réveil supplémentaire. Une lecture confirmée couvrant
la borne inconnue satisfait la consultation sans inventer une remise réussie.

Après dispatch confirmé, aucun rappel périodique pour la même borne ; une mention
plus récente peut produire une nouvelle génération. Après une lecture confirmée
jusqu'à N, les sollicitations ≤N sont satisfaites (sans devenir « livrées » si
elles ne l'étaient pas). Un active_through_seq>N ou pending_seq>N reste intact.
Un envoi déjà en vol peut encore arriver : alerte courte devenue obsolète, lecture
vide, aucun travail requis. La clôture annule les intentions pas encore parties.

Déclencheurs de réévaluation : commit post/ack/close, connexion ou capacité
modifiée, fin DND et maintenance bornée existante. Pas de polling par un agent.
Au plus 5 départs/s pour les fils et un lot de16 candidats par tick ; rotation
équitable par last_attempt pour qu'un absent ne bloque pas les autres.
Une erreur déterministe avant injection peut être réévaluée après changement de
la condition (connexion/DND/capacité), jamais en boucle serrée. Une issue inconnue
attend une preuve ou une consultation explicite du fil ; la règle de nouvelle
mention après échéance ci-dessus évite de bloquer définitivement les suivantes.

Si099 prouve un refus AVANT injection, une nouvelle génération pour la même
pending_seq est permise uniquement après changement du motif de refus ; conserver
la précédente dans les reçus099 et préparer une nouvelle clé de tentative. Ce
cas n'est pas outcome_unknown. Un refus permanent de schéma/identité reste visible
sans retry automatique. Rejouer une clé terminale099 ne peut pas débloquer sa
livraison : il ne fait que relire son résultat. Les tests V10/V25 distinguent les
deux voies, pour éviter soit un doublon soit un blocage permanent après DND.

Ne pas introduire de messages de fil dans le ledger global point à point : seules
les métadonnées de l'alerte technique peuvent y apparaître, sans titre/corps du fil.
Cette restriction évite qu'une lecture ledger large contourne l'appartenance.

## 7. Lecture incrémentale — algorithme imposé

Le curseur appartient à (fil, agent UUID), pas au nom ni à une connexion éphémère.
Il indique une réception confirmée par l'agent, non sa mémoire fournisseur.
Un seul reçu actif par paire évite les confirmations de pages chevauchantes.

```text
read(actor, fil, limit)
  authentifier et vérifier appartenance
  si reçu actif non expiré : rendre exactement sa plage, sans nouveau reçu
  sinon capturer snapshot_seq = last_seq et base_seq = acked_seq
  lire en ordre croissant à partir de base_seq+1, <= snapshot_seq
  couper avant dépassement de limit ou du budget JSON, jamais au milieu d'une entrée
  si vide : receipt=null, aucun curseur modifié
  sinon persister reçu opaque, base_seq, through_seq réel, snapshot_seq, expires_at
  rendre page + receipt + has_more (through_seq < snapshot_seq)

ack(actor, fil, receipt)
  authentifier et vérifier appartenance au fil
  si receipt = last_ack_receipt : retourner already_acknowledged sans mutation
  vérifier reçu actif lié à ce fil et cet acteur, toujours courant
  comparer acked_seq == base_seq ; sinon cursor_conflict sans mutation
  avancer jusqu'à through_seq, jamais snapshot_seq si page incomplète
  conserver last_ack_receipt, supprimer reçu actif, satisfaire les mentions <=through_seq
  commit
```

Reçu actif : réservation10min ; après cette durée un nouveau read peut le
remplacer depuis le curseur inchangé. Un ACK tardif reste accepté tant que le
reçu n'a pas été remplacé et que base_seq correspond au curseur. Un ancien reçu,
autre que le dernier confirmé, est `receipt_obsolete` : rejouer est sans effet,
mais on ne promet pas un cache illimité de toutes les réponses ACK historiques.

Après chaque page, l'agent appelle `ack`, ou joint `ack_receipt` à son `post`
suivant (même transaction). Publier sans ce champ n'avance JAMAIS le curseur.
Les contributions propres apparaissent aussi en lecture pour garder une plage
contiguë ; les retirer compliquerait les preuves et les synthèses pour peu de gain.

`history(from_seq,to_seq,limit)` lit une plage sans reçu et sans déplacer le
curseur. Au premier appel, figer `to_seq` au last_seq renvoyé ; les pages suivantes
conservent cette borne. Une perte de contexte doit utiliser history explicitement,
pas prétendre que le curseur persistant prouve un contexte fournisseur intact.

## 8. Adaptateurs : piège des réponses automatiques

Ajouter `thread_notice` typé au message, uniquement constructible par le chemin
interne de fil. Un corps commençant par « Bridget thread » ne suffit jamais.
Contenu de l'alerte : version, thread_id, through_seq, génération et conduite de
lecture ; aucun historique, aucun texte posté, aucun titre dans le ledger général.

Pour T3 (101 t3code.rs:1659 et :2375), ne pas ajouter cette remise au `pending`
des DM qui relayent automatiquement la réponse finale. Produire une enveloppe
« Consulte ce fil via bridget_thread/read ; publie dans le fil si utile ; ne fais
pas de réponse directe à cette alerte ». Même exigence dans les injections Codex
gérées/interactives et Claude gérées/interactives. Une réponse finale ordinaire
reste dans la conversation fournisseur ; elle n'est pas extraite et publiée.

Ne pas utiliser le préfixe `bridget-observation:` : il signifie observation sans
réponse requise et entraîne des exclusions101 différentes. Les vraies activités
du tour sollicité restent observables selon101 ; pas de nouveau faux événement
« mission terminée ». Les observations ne créent pas de post de fil.

La capacité ThreadNoticeV1 doit être explicitement annoncée par chaque adaptateur
testé. Ancien adaptateur : publication et lecture disponibles mais alerte en état
capability_unavailable ; pas de dégradation en DM pour masquer l'incompatibilité.
Ne pas annoncer une recette nominale si l'outil read/post n'est pas accessible
au destinataire. La capacité de transport ne prouve pas le catalogue d'un serveur
MCP déjà vivant : vérifier celui-ci séparément lors de la recette de version.

Négociation concrète, sans nouveau handshake : Register.thread_notice_versions
(Vec<u16>, default vide) ; Registered.thread_notice_version (Option<u16>, absent
par défaut). Adaptateurs implémentés annoncent [1] ; daemon choisit1 ouNone.
Sélection par connexion wrapper, effacée à déconnexion, revérifiée avant chaque
remise ou récupération. Adapter connect_and_register_at (wrapper.rs:1570) et son
appel T3 (101:1070). Ne pas utiliser ClientCapability (client auxiliaire) ni
ObservationCapabilities101 (sources de faits) pour cette capacité de réception.

Le raccourci `bridget reply` ne doit pas viser un ancien DM après cette alerte.
Dans le contexte last-sender propre à l'agent, écrire un marqueur de dernier
événement `thread_notice` avec thread_id (format JSON discriminé, anciennes lignes
expéditeur/tabulation toujours lisibles). Ce n'est pas un nouvel expéditeur.
CLI reply retourne `thread_notice_not_replyable` et indique thread/post ; le
prochain vrai DM remet son contexte normal. Couvrir les chemins concernés sans
effacer les demandes suivies099 ni empêcher une réponse explicitement adressée.

## 9. Identité, confidentialité, erreurs

Réutiliser RegisterAuxiliary + preuve avant ClientHello et contrôle instance de099/101.
CLI humain : lecture/écriture de fil dans cette V1 exige une identité de session
attestée comme le MCP ; le chemin « humain idempotent »099 reste réservé à ses
usages existants et n'accorde pas une usurpation de membre.

Fil inexistant ou non-membre : même erreur `thread_unavailable`, aucun titre,
liste de membres ou état révélés. Une fois membre : causes spécifiques.
Toutes les requêtes SQL paramétrées ; pages structurées sans exécution HTML,
shell, templates ni liens lus automatiquement. L'émetteur conserve l'autorité
d'un message utilisateur/inter-agent, jamais d'un système ou développeur.

Ne pas journaliser corps, titres, jetons de preuve ni reçus opaques. Compteurs et
logs : thread_id, actor_id, nombre d'entrées/octets, code d'issue, génération,
nombre de cibles, durée ; pas d'analytics externe. Les journaux opérationnels
locaux ne remplacent pas les contrôles d'accès ni la protection du compte système.

## 10. Bornes, complexité et compatibilité SQL

Valeurs V1 explicites dans contracts/thread-api.md ; constantes partagées du
contrat métier, pas huit options de configuration. Index (thread_id,seq) pour
lecture O(log E + P), publication O(log E + M), M≤16, aucune lecture d'historique
au dispatch. `list` indexé par membre ; pas de scan de tous les corps.

Migration additive dans store/threads.rs appelée par Store::init_schema, avec
version propre aux fils. Ne pas réaffecter la version idempotence6 de store_schema.rs.
Préflight vérifie tables/index attendus ; migration transactionnelle, ouverture
répétée sans perte, anciennes lignes ledger et demandes099 inchangées.
Un ancien binaire peut ignorer les tables additives ; cela ne prouve PAS qu'une
cohabitation avec de nouveaux wrappers est sûre. Documenter la compatibilité,
pas de downgrade automatique ni modification de la base production ici.

Plafonds d'espace logique contrôlés transactionnellement ; pas de purge automatique
de l'historique dans102. À saturation : lecture/ack/close continuent, nouveaux
create/post refusés explicitement. Aucun quota sur le nombre de tours métier.

## 11. Séquence de construction et validation future

1. Intégrer/vérifier le socle101 et préparer fixtures isolées ; aucun déploiement.
2. Contrat/protocole + tests de refus ; schéma SQL et idempotence create/post.
3. Fil silencieux, appartenance et outils CLI/MCP (US1).
4. Intentions, remise ciblée typée et tests adaptateurs (US2).
5. Lecture/reçus/ACK/pagination/reprise (US3).
6. Recettes de synthèse + docs et skill (US4), tests end-to-end simulés et charge.
7. Relecture, Analyze/Converge/audit après implémentation ; commit/install uniquement
   sur nouvelle autorisation, pas héritée des sessions100/101.

Les tâches explicites feront foi. Aucun test102 n'a été exécuté pendant la
préparation. La suite de recette devra inclure les scénarios BDD puis leurs tests
Rust natifs ; pas de dépendance Python/pytest introduite dans ce workspace Rust.

## 12. Contrôle constitutionnel et charge future

Articles I/III/X/XVI : français, nouvelle session isolée, cycle arrêté avant code
sur instruction utilisateur explicite. VII : ADR038 proposée. XVIII : bornes et
index vérifiables. XIX/XX : un outil, un module métier, même DB, mêmes identités
et livraisons ; tables nouvelles justifiées par des invariants non couverts.
Pas de framework agents, LLM routeur, résumé automatique ou permissions métier.

Les détails de reçus et de watermarks ajoutent une complexité réelle mais nécessaire
aux coupures et à l'économie de lectures ; elle est rendue testable par transitions
et exemples. Les noms de fonctions privées peuvent évoluer ; les contrats publics,
invariants, codes d'erreur et attentes de tests ne doivent pas être improvisés.
