# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/implementation.md](../../../../../../specs/156-guidage-file/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

QueueList lit seulement les remises idempotentes dispatching non expirées,
par rowid, au plus 200 entrées et aperçu 120 caractères. La borne est 64 T3/null
sinon. Destinataire/parent/Attach/UI attestée voient la projection ; les autres
voient leurs messages. Attach ne négocie que [QueueListV1], sans changement
de rôle ni identité agent. LIMIT200 borne la sortie, pas le scan O(log n+s).

<a id="extrait-2"></a>

## Extrait 2

T3 conserve ses lots et exige mid_turn_ok. Claude seul possède une capacité
native activeSteering prouvée dans le bundle149. Cursor occupé attend même
flagtrue ; son steer est refusé. REST/V1 ferme toute intention explicite.
L’immédiat utilise la connexion source ; recovery ne déduit jamais une route
UI/humaine de from. Sans preuve UI persistée, attente normale ; liens UUID
agents recalculés. Cancel retire en attente ; l’interruption active passe la
garde. ACP retire atomiquement sous le verrou existant.

<a id="extrait-3"></a>

## Extrait 3

Réemplois : MessageIntent/canon, reçus idempotents, SendDelivery, ReplyTracking,
DeliveryExecutionLink, transaction SQLite, identité149, liens et Task, pause087,
ManagedSession, ACK/reprise et dispatch T3. Créations demandées : guidance pur,
guidance_events/deux index, index recipient/phase, lectures de rythme/file,
QueueListV1/trames et NotPermitted. Extensions : flags facultatifs de remise,
interrupt_active et cancel_queued. Aucun service, dépendance ou framework nouveau.
T013 reprend le montage E2E prévu dans son seul fichier, pour trois scénarios.
