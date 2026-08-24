# Frontière d'événements de coordination — extension 016 v1

T1603 ajoute le seul fait absent de 015 dont les consommateurs B et C ont
besoin : `reminder_sent`. Il est produit par Bridget, après l'écriture et le
flush réels du rappel. Maicie ne peut ni l'émettre, ni le fabriquer depuis un
texte, une horloge locale ou une absence de réponse.

## Corpus disponible

Le corpus de base est la copie octet pour octet de la négociation de service
015 v1 : `fixtures/service-negotiation-v1.jsonl`. L'extension versionnée est
matérialisée dans `fixtures/coordination-events-v1.jsonl` : elle ajoute la
capacité explicite `coordination_events_v1` après `maicie_guichet`, sans
modifier les cinq lignes 015. Un service 015 qui ne négocie que
`maicie_guichet` ne reçoit aucun fait 016.

Les seuls états de cycle de vie actuellement publiés par 015 sont :

- `answered` ;
- `cancelled` ;
- `timed_out`.

Ils restent des faits Bridget, persistés avec leur `event_id`, leur
`request_id`, leur instant `observed_at` et leur corrélation de réponse. Une
observation incomplète ne devient jamais un fait métier Maicie.

## Canon fermé `coordination_event` v1

```json
{"type":"coordination_event","v":1,"event_id":"evt-reminder-1","request_id":"request-1","kind":"reminder_sent","reminder_message_id":"message-reminder-1","recipient":"codex-1","generation":1,"observed_at":1787500003}
```

Tous les champs sont requis, dans cet ordre, et `kind` est une énumération
fermée dont la seule valeur v1 est `reminder_sent`. Un type de fait, un champ,
une capacité ou une corrélation inconnus sont refusés comme
`invalid_envelope` avant toute sélection ou mutation durable. Les faits ne
sont pas des trames de service entrantes : le daemon Bridget en est l'unique
producteur.

| Champ | Autorité | Lecteur identifié |
|---|---|---|
| `event_id` | SQLite Bridget, alloué une fois | B/T1606 déduplique le triplet persistant ; C/T1613 rejoue le même fait |
| `request_id` | demande suivie Bridget | B/T1605 rattache le fait à la délégation ; C/T1613 relève la même corrélation |
| `reminder_message_id` | `Deliver` réellement flushé | B/T1605 conserve la preuve de remise ; C/T1613 ne reconstruit jamais le message |
| `recipient` | destinataire résolu par Bridget | B/T1605 vérifie le destinataire déclaré ; C/T1613 rend la notification attestée |
| `generation` | palier de rappel transport, 1 puis 2 | B/T1606 refuse un rejeu de génération déjà consommée ; C/T1613 le relaie inchangé |
| `observed_at` | horloge du daemon après flush | B/T1605 journalise l'instant ; C/T1613 l'affiche sans le recalculer |

`generation` décrit le palier de relance du transport, pas une déduction de
priorité Maicie. Il est persistant et strictement positif. La persistance
indexe `(request_id, generation)` : un rejeu local relève donc le même
`event_id` et les mêmes bytes.

## Règle de couture

Le consommateur Maicie relira les bytes produits par Bridget ; il ne partage
ni le store Bridget ni un DTO local supposé équivalent. Tout curseur, toute
fraîcheur et toute déduplication devront être exposés par le protocole public
et conserver la reprise des mêmes bytes et du même `event_id`.

## Relève cursée v2 — T1604

La capacité `coordination_events_v2` remplace le rejeu implicite par une
commande explicite `coordination_subscribe`. Elle reste incompatible avec
`coordination_events_v1` sur une même connexion : la v1 conserve exactement
son comportement historique, tandis que la v2 est la seule voie qui donne une
frontière de fraîcheur publiquement vérifiable.

Le curseur est un entier opaque, alloué durablement par SQLite Bridget dans
l'ordre de persistance. Après `coordination_subscribe`, Bridget émet, dans cet
ordre : zéro ou plusieurs `coordination_event` v2 (chacun porte son `cursor`),
puis `coordination_snapshot_caught_up`. Avant cette dernière trame, et après
un `coordination_gap` ou `coordination_unavailable`, l'observation est non
fraîche. Seule une nouvelle souscription qui atteint son snapshot rend la
fraîcheur à la relève : un événement suivant ne répare jamais un trou.

| Trame v2 | Autorité | Lecteur identifié |
|---|---|---|
| `coordination_event.cursor` | watermark SQLite Bridget | C/T1613 le conserve comme curseur durable, B/T1606 déduplique `event_id` |
| `coordination_snapshot_caught_up` | lecture complète Bridget | C/T1613 peut seulement alors marquer l'observation fraîche |
| `coordination_gap` | séquence durable absente | B/T1606 bloque l'effet et journalise `observation_incomplète` |
| `coordination_unavailable` | erreur de lecture Bridget | B/T1606 bloque l'effet sans l'assimiler à une lacune |

Le daemon stocke un watermark monotone séparé des lignes. Ainsi, une ligne
absente entre deux curseurs est un `coordination_gap`, pas une vue vide. Une
erreur SQLite est `coordination_unavailable`; ces deux trames sont fermées et
distinctes. Le corpus `fixtures/coordination-stream-v2.jsonl` est normatif.
Le rejeu depuis le même curseur relit les mêmes octets JSONL et le même
`event_id`; Maicie n'ouvre jamais la base Bridget pour l'obtenir.
