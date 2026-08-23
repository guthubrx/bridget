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
2. `ServiceHello { version: 1, service: "maicie", issuer_scope,
   capabilities: ["maicie_guichet"] }` ;
3. `ServiceWelcome { version: 1, horizon_secs, issued_at_tolerance_secs,
   capabilities: ["maicie_guichet"] }`.

`service` est un rôle distinct de `wrapper`, `attach` et `client`. Le daemon
ne sélectionne jamais ce rôle implicitement. Une capacité demandée mais non
reconnue, un rôle déjà fixé ou une version différente échoue avant toute
lecture ou écriture de guichet.

`issuer_scope` est opaque, stable et validé comme au contrat 012 (au moins 128
bits et son alphabet publié). Il isole deux émetteurs qui choisiraient le même
`request_id`; un nom d'agent, même renommé, n'est jamais un substitut de scope.
`horizon_secs` et `issued_at_tolerance_secs` sont négociés : le daemon valide
les dates contre son horloge, puis calcule et fige `expires_at = issued_at +
horizon_secs` au premier dépôt.

| Opération | Wrapper historique | Service sans capacité | Service + `maicie_guichet` |
|---|---:|---:|---:|
| dépôt `ServiceRequest` vers `maicie` | admis si l'émetteur enregistré correspond au champ `from` | refus | refus |
| `GuichetClaimNext`, `GuichetClaim` / `GuichetClaimed` | refus | `capability_required` | admis |
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
{"type":"service_request","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-01","issued_at":1787500000,"from":"codex-1","to":"maicie","operation":"delivery_report","payload":{"objective_id":"obj-01","delegation_id":"del-01","delivery_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","in_reply_to":"msg-01"}}
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
`issuer_scope`, `request_id`, `issued_at`, `from`, `to`, l'opération et toute
la charge font partie des octets canoniques. `issued_at` est immuable : une
valeur future au-delà de la tolérance est `invalid_issued_at`; un premier dépôt
déjà hors horizon est `idempotency_expired`, jamais la création silencieuse
d'une nouvelle demande.

Après un dépôt accepté, le daemon retourne :

```json
{"type":"guichet_result","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-01","issue":"queued","expires_at":1787500300}
```

### 2.2 Relève : amorçage et claim

Une relève commence obligatoirement par `GuichetClaimNext`, qui ne porte pas
de clé inconnue du compagnon :

```json
{"type":"guichet_claim_next","v":1}
```

Le daemon sélectionne **une seule** demande relivable dans l'ordre FIFO
durable `(deposited_sequence ASC)`. Une demande `claimed` dont la connexion de
service a disparu ou dont le claim n'a pas de résultat durable redevient
relivable avec sa séquence de dépôt d'origine : un crash ne change donc pas
l'ordre. La réponse est `guichet_claimed` (et révèle alors `issuer_scope` et
`request_id`) ou `guichet_empty`.

La pagination est implicitement bornée à un élément par appel. Le client répète
`GuichetClaimNext` seulement jusqu'à son échéance globale négociée ; le daemon
ne boucle jamais, ne scanne jamais au-delà du prochain index FIFO et ne retient
aucun curseur de session. Ainsi un dépôt fait pendant l'absence de Maicie est
amorcé à sa prochaine commande sans connaître de clé préalable.

Après cette première réponse, un retry précis peut employer `GuichetClaim` :

```json
{"type":"guichet_claim","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-01"}
```

Le daemon répond soit par :

```json
{"type":"guichet_claimed","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-01","canonical_request":"<base64 des octets ServiceRequest>","claimed_at":1787500000,"expires_at":1787500300}
```

Le claim ne prend jamais une décision Maicie. Une demande `claimed` non
finalisée redevient relevable après un redémarrage : elle garde le même
`request_id` et les mêmes octets.

### 2.3 Consultation et retry : `GuichetLookup`

```json
{"type":"guichet_lookup","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-01"}
```

La clé de lookup est exactement `(issuer_scope, "service_request",
request_id)`. Une issue terminale rejoue son `guichet_result` durable avec le
même `expires_at`; `queued` ou `claimed` retourne `outcome_unknown` avec ce
même `expires_at`; une clé absente ou purgée retourne `idempotency_expired`.
Le retry re-soumet exactement les mêmes octets, y compris `issuer_scope` et
`issued_at`, et ne recrée jamais une demande après la rétention.

### 2.4 Réponse : `GuichetReply`

```json
{"type":"guichet_reply","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-01","response_message_id":"msg-02","in_reply_to":"msg-01","outcome":"accepted","payload":{"kind":"delivery_report","objective_id":"obj-01","delegation_id":"del-01","delivery_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}
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
L'accusé de `GuichetReply` est un `guichet_result` avec le `expires_at` figé
de la demande : le client ne recalcule jamais cette échéance depuis sa
configuration courante.

### 2.5 Événement terminal : `RequestLifecycleEvent`

```json
{"type":"request_lifecycle_event","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","event_id":"evt-01","request_id":"req-01","state":"answered","observed_at":1787500001,"in_reply_to":"msg-01","response_message_id":"msg-02"}
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

`rejected` et `replied` sont terminaux. La clé de dépôt est
`(issuer_scope, "service_request", request_id)` ;
`canonical_request_bytes` et `issued_at` y sont immuables. Les tombstones
conservent les octets, l'issue, `issued_at` et `expires_at` jusqu'à la rétention
annoncée par le daemon. Après expiration, `idempotency_expired` est terminal :
Maicie ne crée jamais un nouvel identifiant à la place du demandeur.

| Frontière | État durable requis au redémarrage | Réponse au retry |
|---|---|---|
| avant insertion | aucune demande ou refus explicite | dépôt unique possible |
| après dépôt, avant claim | `queued` + scope + octets canoniques + `expires_at` | même demande relevable |
| après `ClaimNext`, avant résultat | `claimed` + `deposited_sequence` immuable | même élément FIFO relivable, sans seconde demande |
| après résultat, avant retour client | réponse + issue + `expires_at` durables | même `GuichetReply` reconstruite |
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

### 5.1 Trames valides

```json
{"type":"service_request","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-delivery","issued_at":1787500000,"from":"codex-1","to":"maicie","operation":"delivery_report","payload":{"objective_id":"obj-01","delegation_id":"del-01","delivery_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef","in_reply_to":"msg-01"}}
{"type":"service_request","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-status","issued_at":1787500000,"from":"codex-1","to":"maicie","operation":"mission_status","payload":{"delegation_id":"del-01"}}
{"type":"service_request","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-deadline","issued_at":1787500000,"from":"codex-1","to":"maicie","operation":"deadline_question","payload":{"delegation_id":"del-01"}}
{"type":"guichet_claim_next","v":1}
{"type":"request_lifecycle_event","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","event_id":"evt-answered","request_id":"req-delivery","state":"answered","observed_at":1787500001,"in_reply_to":"msg-01","response_message_id":"msg-02"}
{"type":"request_lifecycle_event","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","event_id":"evt-cancelled","request_id":"req-delivery","state":"cancelled","observed_at":1787500002}
{"type":"request_lifecycle_event","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","event_id":"evt-timeout","request_id":"req-delivery","state":"timed_out","observed_at":1787500003}
```

Ces sept lignes donnent respectivement `queued` avec `expires_at`, puis les
trois événements durables. La `guichet_claim_next` qui les suit rend
`req-delivery`, premier dépôt FIFO, sous forme de `guichet_claimed`. Le test
normatif dépose pendant l'absence de Maicie, ouvre une connexion de service
fraîche, appelle cette unique trame puis constate que la première demande est
relevée. Retirer l'écriture atomique de l'événement terminal fait échouer
l'oracle de redémarrage ; retirer `issuer_scope` fait échouer la validation de
la clé composite ; remplacer la sélection FIFO par une clé exigée fait échouer
l'amorçage.

### 5.2 Refus complets

Chaque trame suivante porte l'issue exacte et la mutation que sa fixture doit
détecter. Elle est envoyée à une connexion de dépôt déjà enregistrée, sauf le
cas de capacité qui utilise un service sans `maicie_guichet`.

1. Texte libre — issue `invalid_envelope`, aucune ligne guichet :

   ```text
   Maicie, ferme cette mission maintenant.
   ```

   Mutation discriminante : accepter une chaîne au lieu d'un objet JSON crée
   une demande et casse l'assertion de compteur nul.

2. Type inconnu — issue `invalid_envelope`, aucune ligne guichet :

   ```json
   {"type":"service_request","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-type","issued_at":1787500000,"from":"codex-1","to":"maicie","operation":"invented_operation","payload":{}}
   ```

   Mutation discriminante : ouvrir l'énumération d'opérations fait passer la
   fixture au lieu du refus fermé.

3. Champ inconnu — issue `invalid_envelope`, aucune ligne guichet :

   ```json
   {"type":"service_request","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-field","issued_at":1787500000,"from":"codex-1","to":"maicie","operation":"mission_status","payload":{"delegation_id":"del-01","extra":"non"}}
   ```

   Mutation discriminante : désactiver `deny_unknown_fields` conserve ce champ
   et fait échouer l'assertion de refus.

4. Référence absente — issue `invalid_envelope`, aucune ligne guichet :

   ```json
   {"type":"service_request","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-reference","issued_at":1787500000,"from":"codex-1","to":"maicie","operation":"delivery_report","payload":{"objective_id":"obj-01","delegation_id":"del-01","delivery_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}
   ```

   Mutation discriminante : rendre `in_reply_to` optionnel transforme ce refus
   en dépôt et casse le compteur nul.

5. Approbation interdite — issue `invalid_envelope`, aucune ligne guichet :

   ```json
   {"type":"service_request","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-approve","issued_at":1787500000,"from":"codex-1","to":"maicie","operation":"approve_profile_activation","payload":{"command_id":"cmd-01"}}
   ```

   Mutation discriminante : admettre cette opération créerait un chemin
   d'approbation distant et fait échouer l'oracle sans activation.

6. Enveloppe divergente — issue `canonical_bytes_mismatch`, record existant
   inchangé :

   ```json
   {"type":"service_request","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-status","issued_at":1787500000,"from":"codex-1","to":"maicie","operation":"mission_status","payload":{"delegation_id":"del-02"}}
   ```

   Cette trame suit la trame valide `req-status` de §5.1. Mutation
   discriminante : comparer une valeur reparsée ou remplacer le canon du record
   fait disparaître le refus et casse l'assertion d'octets inchangés.

7. Capacité absente — issue `capability_required`, aucune lecture ou claim :

   ```json
   {"type":"guichet_claim","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-delivery"}
   ```

   Mutation discriminante : déverrouiller le claim sur le seul rôle `service`
   rend une demande lisible et casse l'oracle de zéro relève.

## 6. Frontière Bridget / Maicie

Bridget valide la forme filaire, les tailles, le canon, les transitions du
guichet et la capacité. Maicie valide exclusivement les relations métier :
participant, délégation, objectif, hash et état de coordination. Maicie ne lit
ni `bridget.db`, ni une API interne, et ne peut ni créer une approbation ni
consommer une activation par ce chemin.

Les journaux du guichet portent `request_id`, et `objective_id` /
`delegation_id` lorsqu'ils existent, sans corps libre ni secret. Toute sortie
qualifie l'identité de service comme déclarative et coopérative v1.
