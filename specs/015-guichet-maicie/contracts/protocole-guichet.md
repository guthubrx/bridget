# Contrat public v1 — Guichet Maicie

**Statut** : gelé pour G1501
**Version filaire** : `1`
**Propriétaire du transport** : Bridget
**Consommateur** : Maicie, uniquement par le protocole public Bridget

Ce contrat définit une boîte aux lettres durable, réservée à la cible
`maicie`. Elle rend cette cible joignable sans annoncer un processus Maicie
vivant et sans déplacer dans Bridget un objectif, une délégation ou une
décision Maicie.

La capacité `maicie_guichet` est une borne de protocole coopérative v1. Elle
évite qu'un nom déclaré ou un rôle historique donne accès à la relève, aux
réponses ou aux événements. Elle n'authentifie pas cryptographiquement deux
processus du même compte local ; cette limite est déclarée par le daemon et ne
doit jamais être présentée comme une identité opposable.

## 1. Négociation et matrice de rôle

Une connexion Maicie suit exactement cet ordre :

1. `RoleHandshake { role: service }` ;
2. `ServiceHello { version: 1, service: "maicie", capabilities:
   ["maicie_guichet"] }` ;
3. `ServiceWelcome { version: 1, capabilities: ["maicie_guichet"] }`.

`service` est un rôle distinct de `wrapper`, `attach` et `client`. Le daemon
ne sélectionne jamais ce rôle implicitement. Une capacité demandée mais non
reconnue, un rôle déjà fixé ou une version différente échoue avant toute
lecture ou écriture de guichet.

| Opération | Wrapper historique | Service sans capacité | Service + `maicie_guichet` |
|---|---:|---:|---:|
| dépôt `ServiceRequest` vers `maicie` | admis si l'émetteur enregistré correspond au champ `from` | refus | refus |
| `GuichetClaim` / `GuichetClaimed` | refus | `capability_required` | admis |
| `GuichetReply` | refus | `capability_required` | admis |
| réception `RequestLifecycleEvent` | refus | `capability_required` | admis |
| messages 007–014 non-guichet | comportement historique | refus hors matrice | refus hors matrice |

Le dépôt est volontairement séparé de la capacité : un équipier ne négocie pas
le droit de se faire passer pour Maicie pour déposer une demande. Le daemon le
contrôle contre la connexion wrapper déjà enregistrée ; le champ `from` est un
**émetteur déclaré**, jamais une preuve d'identité entre processus locaux.

## 2. Enveloppes canoniques

Chaque frame est une ligne JSON UTF-8. Les objets sont sérialisés sans espace,
avec les clés dans l'ordre indiqué ci-dessous, les chaînes en UTF-8 NFC et un
seul saut de ligne de transport hors des octets canoniques. Les octets
canoniques sont ceux de l'objet JSON sans ce saut de ligne. Toute autre forme
JSON qui représente la même valeur est rejetée : l'idempotence compare les
octets, pas une valeur reparsée.

Limites v1 : une frame complète fait au plus 64 Kio ; `request_id`,
`response_message_id` et `event_id` font de 1 à 128 octets ASCII
`[A-Za-z0-9._:-]`; les UUID et références métier font au plus 128 octets ; le
hash de livraison est un SHA-256 hexadécimal de 64 caractères. Un corps libre,
un tableau à la place d'un objet, un champ inconnu ou une clé dupliquée est un
refus filaire avant persistance.

### 2.1 Dépôt : `ServiceRequest`

```json
{"type":"service_request","v":1,"request_id":"req-01","from":"codex-1","to":"maicie","operation":"delivery_report","payload":{"objective_id":"obj-01","delegation_id":"del-01","delivery_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","in_reply_to":"msg-01"}}
```

Les seules valeurs de `operation` sont :

| Opération | `payload` canonique, dans cet ordre | Effet Bridget |
|---|---|---|
| `delivery_report` | `objective_id`, `delegation_id`, `delivery_hash`, `in_reply_to` | mise en attente ; la réponse liée peut greffer la demande `in_reply_to` |
| `mission_status` | `delegation_id` | mise en attente, sans mutation d'une demande suivie |
| `deadline_question` | `delegation_id` | mise en attente, sans mutation d'une demande suivie |

`to` est exactement `maicie`. Le daemon lie le dépôt à une connexion wrapper
enregistrée et refuse si son nom courant ne correspond pas à `from`. Cette
vérification ne transforme pas le champ déclaré en authentification forte.

### 2.2 Relève : claim

```json
{"type":"guichet_claim","v":1,"request_id":"req-01"}
```

Le daemon répond soit par :

```json
{"type":"guichet_claimed","v":1,"request_id":"req-01","canonical_request":"<base64 des octets ServiceRequest>","claimed_at":1787500000}
```

soit par `guichet_empty`, soit par un refus typé. Le claim ne prend jamais une
décision Maicie. Une demande `claimed` non finalisée redevient relevable après
un redémarrage : elle garde le même `request_id` et les mêmes octets.

### 2.3 Réponse : `GuichetReply`

```json
{"type":"guichet_reply","v":1,"request_id":"req-01","response_message_id":"msg-02","in_reply_to":"msg-01","outcome":"accepted","payload":{"kind":"delivery_report","objective_id":"obj-01","delegation_id":"del-01","delivery_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}
```

`GuichetReply` est réservé à la connexion de service ayant négocié
`maicie_guichet`. `in_reply_to` est obligatoire pour `delivery_report` et doit
être identique au champ de la demande. Les valeurs fermées de `outcome` sont :

- `accepted` : réponse durable, avec livraison liée à effectuer ;
- `request_already_terminal` : la demande liée était déjà `cancelled` ou
  `timed_out`; le rapport reste durable et livrable, sans rouvrir cette demande ;
- `recipient_unavailable` : la réponse est durable mais son destinataire
  déclaré n'est pas joignable ;
- `refused` : refus déterministe de la relève, sans effet métier Bridget.

Une réponse déjà durable est reconstruite octet pour octet au retry. Un retry
avec le même `request_id` mais des octets différents retourne
`canonical_bytes_mismatch` sans modifier la demande, sa réponse ou son claim.

### 2.4 Événement terminal : `RequestLifecycleEvent`

```json
{"type":"request_lifecycle_event","v":1,"event_id":"evt-01","request_id":"req-01","state":"answered","observed_at":1787500001,"in_reply_to":"msg-01","response_message_id":"msg-02"}
```

Les états fermés sont `answered`, `cancelled` et `timed_out`. Seul Bridget
produit cet événement ; Maicie le reçoit parce que sa connexion de service a
négocié la capacité. Chaque transition terminale et son événement sont écrits
dans la même transaction Bridget. `answered` est déposé après l'accusé durable
de la réponse liée ; `cancelled` et `timed_out` ne sont jamais reconstitués par
Maicie depuis une horloge ou une absence de message.

## 3. États, idempotence et reprise

La machine de transport est indépendante du registre Maicie :

```text
queued ──claim──> claimed ──GuichetReply durable──> replied
  │                  │                                  │
  └──refus──> rejected└──crash/reprise──> queued          └──retry──> même réponse
```

`rejected` et `replied` sont terminaux. Le couple `(request_id,
canonical_request_bytes)` est la clé de dépôt. Les tombstones conservent les
octets, l'issue et les identifiants jusqu'à la rétention annoncée par le daemon.
Après expiration, `idempotency_expired` est terminal : Maicie ne crée jamais un
nouvel identifiant à la place du demandeur.

| Frontière | État durable requis au redémarrage | Réponse au retry |
|---|---|---|
| avant insertion | aucune demande ou refus explicite | dépôt unique possible |
| après dépôt, avant claim | `queued` + octets canoniques | même demande relevable |
| après claim, avant résultat | `claimed` relivable | même octets, sans seconde demande |
| après résultat, avant retour client | réponse + issue durables | même `GuichetReply` reconstruite |
| après terminal Bridget | événement unique durable | même `event_id`, jamais de seconde transition |

Pour un `delivery_report`, le rapport tardif après `cancelled` ou `timed_out`
reste relevable et reçoit `request_already_terminal`. Il ne peut ni rouvrir la
demande Bridget ni effacer son terminal. La corrélation inter-canaux est le
couple fermé `(in_reply_to, response_message_id)` : le rapport et l'événement
`answered` convergent vers un seul reçu Maicie, quel que soit leur ordre.

## 4. Refus fermés

| Code | Frontière | Mutation interdite |
|---|---|---|
| `service_role_required` | role absent ou non `service` | aucune demande ni réponse |
| `capability_required` | claim, réponse ou événement sans `maicie_guichet` | aucune lecture métier ni transition |
| `reserved_target_required` | cible autre que `maicie` | aucune persistance guichet |
| `declared_sender_mismatch` | `from` différent du wrapper enregistré | aucune demande |
| `unsupported_version` | `v != 1` ou négociation incompatible | aucune demande |
| `frame_too_large` | dépassement 64 Kio | aucune allocation persistante |
| `invalid_envelope` | type, opération, champ, référence ou caractère invalide | aucune demande |
| `canonical_bytes_mismatch` | même `request_id`, octets différents | aucune mutation du record existant |
| `request_already_terminal` | claim/réponse incompatible avec un terminal | terminal conservé |
| `idempotency_expired` | tombstone hors rétention | aucune réémission |
| `transition_invalid` | transition hors machine guichet | état durable inchangé |

La matrice est fermée : une variante future de `ServiceRequest`,
`GuichetReply` ou `RequestLifecycleEvent` est refusée jusqu'à une nouvelle
version négociée. Les messages Bridget antérieurs, y compris attach et le
client idempotent, conservent exactement leur matrice historique lorsqu'aucune
capacité de guichet n'est négociée.

## 5. Corpus filaire normatif v1

Les lignes suivantes sont le corpus figé que T1502 matérialise ensuite en
fixtures producteur↔consommateur. Les valeurs d'exemple font partie de
l'ordre canonique, pas d'un format de rendu humain.

| Cas | Ligne canonique ou issue attendue |
|---|---|
| dépôt `delivery_report` | `{"type":"service_request","v":1,"request_id":"req-delivery","from":"codex-1","to":"maicie","operation":"delivery_report","payload":{"objective_id":"obj-01","delegation_id":"del-01","delivery_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","in_reply_to":"msg-01"}}` |
| dépôt `mission_status` | `{"type":"service_request","v":1,"request_id":"req-status","from":"codex-1","to":"maicie","operation":"mission_status","payload":{"delegation_id":"del-01"}}` |
| dépôt `deadline_question` | `{"type":"service_request","v":1,"request_id":"req-deadline","from":"codex-1","to":"maicie","operation":"deadline_question","payload":{"delegation_id":"del-01"}}` |
| événement `answered` | `{"type":"request_lifecycle_event","v":1,"event_id":"evt-answered","request_id":"req-delivery","state":"answered","observed_at":1787500001,"in_reply_to":"msg-01","response_message_id":"msg-02"}` |
| événement `cancelled` | `{"type":"request_lifecycle_event","v":1,"event_id":"evt-cancelled","request_id":"req-delivery","state":"cancelled","observed_at":1787500002}` |
| événement `timed_out` | `{"type":"request_lifecycle_event","v":1,"event_id":"evt-timeout","request_id":"req-delivery","state":"timed_out","observed_at":1787500003}` |
| relève sans capacité | `capability_required`, sans claim ni lecture de demande |
| retry divergent | même `request_id` que `req-status` avec une charge d'octets différente → `canonical_bytes_mismatch`, record inchangé |

Les fixtures associent chaque ligne à l'issue fermée indiquée et à une mutation
discriminante : retrait de la négociation, changement d'un octet canonique,
ou suppression de l'écriture atomique de l'événement terminal doivent faire
échouer leur oracle respectif.

## 6. Frontière Bridget / Maicie

Bridget valide la forme filaire, les tailles, le canon, les transitions du
guichet et la capacité. Maicie valide exclusivement les relations métier :
participant, délégation, objectif, hash et état de coordination. Maicie ne lit
ni `bridget.db`, ni une API interne, et ne peut ni créer une approbation ni
consommer une activation par ce chemin.

Les journaux du guichet portent `request_id`, et `objective_id` /
`delegation_id` lorsqu'ils existent, sans corps libre ni secret. Toute sortie
qualifie l'identité de service comme déclarative et coopérative v1.
