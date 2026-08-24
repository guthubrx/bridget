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
