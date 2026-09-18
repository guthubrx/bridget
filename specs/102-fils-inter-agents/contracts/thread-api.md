# Contrat public 102 — version1

Ce contrat décrit des commandes FUTURES ; elles ne sont pas disponibles avant
implémentation. Même logique daemon pour MCP `bridget_thread` et CLI `bridget thread`.

## Envelope, identité et validation

Requête socket : `ThreadRequest { version:1, request:ThreadAction }` ; résultat
`ThreadResult { version:1, result:ThreadOutcome }`. Requête seulement pour un rôle
CLI/MCP auxiliaire authentifié et une identité active attestée, jamais public.
Les wrappers n'accèdent pas aux fils par un UUID libre ; ils portent uniquement
des alertes typées après négociation de capacité.

MCP : `arguments` contient l'action et ses paramètres ; version1 fixée par le
client, non paramétrable à chaque appel. Chaque variante refuse les champs
inconnus et incompatibles. Résultat structuré JSON également lisible en texte,
enveloppe MCP `isError:true` pour une erreur, jamais un succès avec message d'échec.
Pas de paramètre from/actor/owner/socket/path ; identité et autorité viennent de099/101.

## Actions fermées

| Action | Paramètres | Résultat utile |
|---|---|---|
| create | title, members:UUID[], operation_id:UUID | created, thread_id, membres effectifs, titre, étatopen ; zéro notification |
| list | limit?=20, after_thread_id? | fils dont l'appelant est membre, next_after ; métadonnées sans corps, ordre technique UUID |
| show | thread_id | titre, créateur, membres, état, last_seq, own_acked_seq, own_wake, bornes/capacités des membres |
| post | thread_id, body, notify:UUID[] ou "all", operation_id:UUID, reply_to_seq?, ack_receipt? | posted, message_id, seq, cibles effectives, état initial par cible ; durable avant retour |
| read | thread_id, limit?=50 | page nouvelle, receipt ou null, base_seq, through_seq, snapshot_seq, has_more, expires_at |
| ack | thread_id, receipt | acknowledged ou already_acknowledged, acked_seq, état propre de sollicitation |
| history | thread_id, from_seq?=1, to_seq?, limit?=50 | page sans reçu, from_seq, through_seq, snapshot_seq, has_more, next_from_seq |
| close | thread_id, operation_id:UUID | closed, closed_at ; annule les sollicitations non parties, conserve historique |

`operation_id` est obligatoire pour create/post/close en MCP. CLI : `--id`
obligatoire aussi ; pas de génération cachée après une erreur. Les exemples de
skill expliquent comment préparer une clé avant l'appel. Aucun issued_at public
nécessaire au dépôt : il n'est pas une livraison099 et son idempotence n'expire
pas avant l'historique. Le dispatcher prépare séparément sa clé/échéance099.

Close avec une clé nouvelle sur un fil déjà clos retourne thread_closed, sans
nouvelle ligne d'opération ; même clé que la clôture réussie rend le reçu original.
Cela borne thread_operations à nombre d'entrées+2 par fil. Une clé déjà liée à
une autre opération reste envelope_mismatch, vérifiée avant l'état de clôture.

`members` et `notify` n'acceptent que des UUID au MCP. Le CLI peut proposer des
noms explicitement saisis via --member/--notify, résolus avec l'annuaire avant
requête ; inconnu/ambigu = refus, jamais choix du premier. Le daemon revalide
l'appartenance avec les UUID. Même affichage qu'un renommage094 : le nom n'est
pas une adresse. `notify:[]` est le choix explicite silencieux ; champ manquant
refusé pour ne pas laisser la discipline au hasard.

`notify:[soi]` donne zéro cible avec notice `self_mention_ignored` ; les autres
cibles sont dédupliquées. `all` exclut l'auteur. Une cible non-membre annule tout
le post, même si les autres sont valides. `reply_to_seq` sert de référence dans
le fil et ne sollicite PAS implicitement l'auteur cité ; notify fait seul foi.

## Pages et reçus

Chaque entrée : `{seq,message_id,author_id,created_at,body,notify,reply_to_seq}`.
Dans une entrée en sortie, notify est `{mode:"none"|"targets"|"all",targets:UUID[]}` ;
targets porte l'ensemble effectif. reply_to_seq est entier positif ou null.
La borne through_seq est la dernière entrée effectivement transmise ; page vide :
through_seq=base_seq pour read, receipt=null. `has_more` concerne le snapshot,
pas des publications postérieures. Après ACK, read repart du curseur courant et
capture un nouveau snapshot. On ne bloque pas la lecture en attendant un ACK
d'alerte ni la publication en attendant un ACK de lecture.

read avec reçu actif : réémettre la même plage/receipt, même si limit change ;
notice `pending_receipt_replayed` et requested_limit effectif. La réservation du
reçu expire à10min, pour laisser un autre client de la même identité reprendre.
L'échéance libère la réservation pour un nouveau read ; un ACK tardif reste accepté
si le reçu n'a pas été remplacé et que le curseur initial correspond. Après
remplacement, l'ancien est receipt_obsolete. Pas d'API « ack jusqu'à numéro
arbitraire ». ACK seulement avec receipt opaque.
ack_receipt de post est le même reçu, mêmes contrôles ; si ACK invalide, le dépôt
est refusé en entier. Un rejeu exact du post déjà commis rend son reçu même si
le read receipt a été supprimé depuis.

history n'a jamais de reçu ; le client continue avec next_from_seq et garde
snapshot_seq comme to_seq. `from_seq>snapshot_seq` produit une page vide ;
`from_seq=0` ou `to_seq<from_seq-1` est invalide. Demander une borne future
to_seq>last_seq est refusé (`range_unavailable`), pas silencieusement tronqué.

## Bornes V1

| Élément | Valeur et effet |
|---|---|
| Participants | 2–16 créateur inclus ; validation après déduplication |
| Titre | 1–160 caractères Unicode, trim pour affichage ; jamais une clé |
| Corps | 1–16Kio UTF-8, pas tout-blanc ; aucune coupe lors du dépôt |
| Page read/history | 50 entrées défaut, 1–200 ; 60Kio de JSON sérialisé maximum |
| Reçu | 10min, un actif par membre/fil |
| Liste | 20 défaut, 1–100 ; ordre UUID lexical stable, after_thread_id exclusif |
| Fils | 256 conservés par autorité, dont au plus32 ouverts créés par un même agent |
| Historique | 10000 entrées ou16Mio de corps par fil, et128Mio de corps tous fils |
| Départs d'alertes | 5/s global pour les fils, lot maximum16, équité des candidats |
| Livraison active | injection≤120s après réservation ; conservation des clés099 selon horizon existant7jours, distincte |

Ces limites techniques n'imposent ni quota de réponses utiles ni nombre de tours.
La limite60Kio porte sur le JSON ThreadOutcome complet, métadonnées de page et
reçu inclus ; pas sur le seul total des corps, ni sur l'enveloppe externe JSON-RPC.
Les corps et titres sont des données non fiables : JSON escape avant calcul du
budget. Même un seul corps16Kio peut gonfler à la sérialisation ; post doit aussi
refuser une entrée dont le JSON complet dépasserait48Kio (`entry_too_large`),
pour garantir qu'une page peut contenir au moins une entrée. La lecture n'efface
pas une entrée trop grosse et ne boucle pas éternellement avec une page vide.
Les12Kio restants réservent la place des métadonnées bornées de page. La page ne
répète pas les titres/noms d'affichage à chaque entrée. Valider la taille de
l'objet final. Quota global : SUM(body_bytes) sur au plus256 lignes de fils,
dans la transaction ; jamais un scan des corps. Aucun TTL d'historique ajouté.

## Erreurs et compatibilité

Format commun : `{status:"error",code,detail,retryable}`. detail n'inclut jamais
de corps/titre d'un fil auquel l'acteur n'a pas accès. CLI sortie JSON avec même
code, exit2 validation/refus et exit1 panne technique ; succès exit0.

Codes : `invalid_request`, `unsupported_version`, `identity_unavailable`,
`thread_unavailable`, `thread_closed`, `creator_required`, `unknown_member`,
`ambiguous_name`, `not_a_member`, `invalid_reply_reference`, `envelope_mismatch`,
`capacity_exceeded`, `entry_too_large`, `receipt_obsolete`,
`receipt_invalid`, `cursor_conflict`, `range_unavailable`, `storage_unavailable`,
`thread_notice_not_replyable` (raccourci CLI reply).
`not_a_member` concerne une cible d'un post par un membre autorisé ; l'appelant
non-membre reçoit toujours thread_unavailable.

Un ancien daemon ne comprenant pas ThreadRequest doit produire/refletéter une
capacité absente/erreur de protocole ; aucune réécriture en DM ou observation.
Les anciennes opérations restent au contrat099 actuel. La négociation nouvelle
ne doit pas empêcher un ancien client de communiquer par ses anciens outils.

## Contrat d'alerte interne

`BridgetMessage.thread_notice = {version:1,thread_id,through_seq,generation}`
facultatif, absent par défaut et omis à la sérialisation sur les DM. Métadonnée
réservée au daemon ; `send` et les entrées externes ne peuvent pas l'injecter.
L'alerte a reply=false, aucune référence de demande suivie ; ID déterministe de
la paire fil/membre/génération, stable et distinct du message_id de la contribution.
Vérifier enveloppe complète au rejeu. Les wrappers déclarent ThreadNoticeV1 via
leur enregistrement complété, et refusent une version inconnue :

- **Amendement d'implémentation (T005)** : la capacité est annoncée par un fait
  de connexion `WrapperToDaemon::ThreadNoticeCapability { versions: [u16] }`,
  envoyé par le wrapper juste après `Registered`, comme `DiskSpace`/`JournalReady`,
  au lieu de champs ajoutés à `Register`/`Registered`. Motif : ajouter un champ
  aux deux variantes aurait modifié ~105 constructions littérales dans 30 fichiers
  pour le même effet ; le fait post-enregistrement conserve « sans nouveau
  handshake », la sélection liée à la connexion et la renégociation à chaque
  reconnexion. Au plus 8 valeurs lues ; seule la version 1 est sélectionnée ; un
  client auxiliaire n'est jamais éligible.
- Sélection liée à la connexion wrapper ; déconnexion et remplacement d'instance
  effacent ; reprise renégocie. Le daemon n'écho pas la sélection : le wrapper ne
  reçoit que des alertes de la version qu'il a annoncée ; `show` expose la version
  acceptée par membre.
- Ancien wrapper n'annonce rien et ne reçoit aucune alerte de fil
  (`capability_unavailable`) ; ancien daemon ignore la variante. DM historiques
  inchangés. Décodages testés (`spec102_v27_capacite_d_alerte_annoncee_apres_enregistrement`).
- Ce n'est ni ClientCapability ni ObservationCapabilities ; l'annonce est faite
  par le helper commun connect_and_register_at, donc par Codex/Claude gérés et
  interactifs et par le pont T3.

Le contexte du raccourci reply reçoit un marqueur JSON
`{"kind":"thread_notice","thread_id":"UUID"}` dans son fichier existant propre
à l'agent ; aucun expéditeur synthétique n'y est enregistré. CLI reply refuse
alors avec thread_notice_not_replyable ; l'ancien format DM reste lu. Une réponse
explicitement adressée à une vraie demande par send reste permise.

Corps neutre généré côté daemon, sans citation des contributions :
« Sollicitation dans le fil <thread_id>, nouveautés jusqu'à <through_seq>.
Lis les nouveautés avec bridget_thread/read. Confirme la plage reçue, puis publie
dans le fil si utile. Ne réponds pas par message direct à cette alerte. »

Pour la recette nominale, vérifier séparément l'accès réel à bridget_thread dans
le catalogue MCP de la session : une capacité d'injection ne le prouve pas.
Catalogue ancien : signaler outil indisponible, pas de bascule automatique vers
CLI ou DM, pas de relance fournisseur implicite. Les adaptateurs n'interprètent pas le texte
pour décider du relais ; ils utilisent thread_notice. Le mot all contenu dans
une alerte ou une entrée n'est pas réinterprété.

## Exemples MCP normatifs

Ces UUID sont synthétiques ; ne pas les utiliser sur le daemon réel.

```json
{"action":"create","title":"Relecture sécurité","members":["11111111-1111-4111-8111-111111111111","22222222-2222-4222-8222-222222222222"],"operation_id":"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"}
```

```json
{"action":"post","thread_id":"33333333-3333-4333-8333-333333333333","body":"Constat disponible, sans demande de relecture immédiate.","notify":[],"operation_id":"bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"}
```

```json
{"action":"post","thread_id":"33333333-3333-4333-8333-333333333333","body":"Peux-tu vérifier ce point ?","notify":["22222222-2222-4222-8222-222222222222"],"operation_id":"cccccccc-cccc-4ccc-8ccc-cccccccccccc"}
```

```json
{"action":"read","thread_id":"33333333-3333-4333-8333-333333333333","limit":50}
```

```json
{"action":"ack","thread_id":"33333333-3333-4333-8333-333333333333","receipt":"dddddddd-dddd-4ddd-8ddd-dddddddddddd"}
```

## Résultats de référence

Le résultat socket enveloppe ThreadOutcome dans `{version:1,result:...}` ; le
structuredContent MCP expose directement ce ThreadOutcome, également rendu en
texte JSON. Le CLI imprime le même objet. Discriminant fermé `status` :
created/listed/shown/posted/read/history/acknowledged/already_acknowledged/closed/error.
Chaque résultat visant un fil inclut thread_id ; created/closed ajoutent
created_at/closed_at Unix respectivement. Les champs optionnels documentés sont
null quand inapplicables, pas renommés différemment entre CLI et MCP.

Read ci-dessous représente une page partielle : ACK ne peut avancer que jusqu'à1,
pas jusqu'au snapshot2. has_more=true entraîne une autre lecture après ACK.

```json
{"status":"read","thread_id":"33333333-3333-4333-8333-333333333333","base_seq":0,"through_seq":1,"snapshot_seq":2,"has_more":true,"receipt":"dddddddd-dddd-4ddd-8ddd-dddddddddddd","expires_at":1789585800,"requested_limit":1,"notices":[],"entries":[{"seq":1,"message_id":"eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee","author_id":"11111111-1111-4111-8111-111111111111","created_at":1789585100,"body":"Constat partagé.","notify":{"mode":"none","targets":[]},"reply_to_seq":null}]}
```

```json
{"status":"acknowledged","thread_id":"33333333-3333-4333-8333-333333333333","acked_seq":1,"own_wake":{"state":"none","reason":null,"pending_seq":0,"generation":0,"active_through_seq":null,"dispatched_seq":0,"last_uncertain_generation":null,"last_uncertain_seq":null}}
```

```json
{"status":"error","code":"thread_unavailable","detail":"Fil indisponible pour cette identité.","retryable":false}
```

Read vide : entries=[], receipt/expires_at=null, through_seq=base_seq,
has_more=false ; pas d'ACK à envoyer. History remplace base_seq/receipt/expires_at
par from_seq et next_from_seq ; next_from_seq=through_seq+1 si has_more, sinon null.
Les autres champs de page et notices sont communs. Résultat posted contient
message_id,seq,targets:UUID[],notices:[],wakes:[{agent_id,state,reason}] ; ces
états initiaux ne sont pas mis à jour dans le reçu idempotent après livraison.
Show renvoie own_wake au format ci-dessus, état open/closed et les métadonnées
de fil décrites dans la table ; members est une liste d'objets {agent_id,
connected,thread_notice_version}, valeurs unknown représentées par null.
List renvoie threads:[{thread_id,title,creator_id,state,last_seq}],next_after
(UUID ou null). Created renvoie members:UUID[] et state:open ; closed renvoie
closed_at. Aucun résultat ne contient les reçus ou curseurs des autres membres.

Les codes de validation/autorisations ont retryable=false. storage_unavailable
peut avoir retryable=true, mais un create/post/close doit conserver la même clé
et enveloppe : un timeout ne prouve pas que la transaction a échoué.

## Projection CLI à implémenter

Toutes les sorties réussies sont JSON ; `--help` explique la lecture et les
limites. Formes :

```text
bridget thread create --title TITRE --member UUID [--member UUID] --id UUID
bridget thread list [--limit N] [--after UUID]
bridget thread show THREAD
bridget thread post THREAD --silent --id UUID -- TEXTE
bridget thread post THREAD --notify UUID [--notify UUID] --id UUID [--reply-to N] [--ack RECU] -- TEXTE
bridget thread post THREAD --all --id UUID -- TEXTE
bridget thread read THREAD [--limit N]
bridget thread ack THREAD RECU
bridget thread history THREAD [--from-seq N] [--to-seq N] [--limit N]
bridget thread close THREAD --id UUID
```

`--silent`, `--notify` et `--all` mutuellement exclusifs, exactement un requis.
`--reply-to` et `--ack` sont permis pour les trois formes post. Refuser flags
répétés hors --member/--notify, inconnus, valeurs manquantes et positionnels
superflus. Ne pas accepter un script, fichier local ou commande à exécuter.
