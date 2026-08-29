# Preuve T006-T008 - corrélation Codex par `clientId`

Date de validation : 2026-08-29
Portée : test isolé du transport Codex. Aucun daemon, agent ou service de production n'a été démarré, redémarré ou déployé.

## Binaire et schémas réellement interrogés

Horodatage de génération : `2026-08-29T07:49:17Z`.

```text
commande : /home/moi/.local/bin/codex app-server generate-json-schema --out <tempdir>
binaire invoqué : /home/moi/.local/bin/codex
résolution : /home/moi/.local/lib/node_modules/@openai/codex/bin/codex.js
version : codex-cli 0.150.1
sha256 binaire : 134063e133f0b4244fa3b251acf973d4fe4b4aeeacbdc135211bf480f59f1477
sha256 v2/TurnSteerParams.json : 4a52eb76e7a717bb388484ccd7538737fca0df35481fc30a21e259f1bfe96e37
sha256 v2/ItemStartedNotification.json : 632d9d70cf866e2515c9c661da6cad1ecc3cfb59abfad0098938a74d22b9fbb9
```

Le schéma généré par ce binaire déclare `TurnSteerParams.clientUserMessageId` et, pour le `ThreadItem` de type `userMessage` porté par `ItemStartedNotification`, deux champs distincts : `id` obligatoire et `clientId` optionnel. Le contrat ne permet donc pas d'utiliser l'identifiant interne `item.id` comme corrélation Bridget.

## Événement et règle appliquée

La fixture de transport émet désormais un événement isolé de la forme suivante, avec deux identifiants volontairement différents :

```json
{"method":"item/started","params":{"threadId":"thread-native","turnId":"turn-native","startedAtMs":1,"item":{"id":"provider-item-<client-message-id>","clientId":"<client-message-id>","type":"userMessage","content":[]}}}
```

L'adaptateur n'enregistre une preuve de visibilité que si les quatre conditions sont réunies :

1. l'événement est `item/started` ;
2. l'item est un `userMessage` ;
3. `item.clientId` est présent et non vide ;
4. `threadId` et `turnId` correspondent exactement au tour actif.

`consume_accepted_steers` ne remet ensuite un `PromptDispatched` qu'au message Bridget dont l'identifiant est ce `clientId`. La consommation retire la preuve du tour : une même preuve ne peut donc pas générer un second acquittement. Le repli borné reste couvert par le témoin G et l'accusé `turn/steer` seul reste insuffisant, couvert par le témoin H.

## Tests observables

Avant le correctif, après séparation de `provider-item-...` et du `clientId`, la commande ciblée a échoué sur quatre assertions : le témoin d'acquittement I, la distinction `clientId`/`id` J, le témoin d'événement tardif K et l'absence de `clientId` L.

Après le correctif :

```text
début : 2026-08-29T07:50:59Z
commande : cargo test -p bridget-transport codex_app_server::tests::TEMOIN_ -- --nocapture
résultat : 16 passed, 0 failed, 0 ignored, 193 filtered out
fin : 2026-08-29T07:51:03Z
format ciblé : rustfmt --edition 2024 --check crates/bridget-transport/src/codex_app_server.rs = OK
```
La suite complète `cargo test -p bridget-transport --quiet` a aussi réussi : 208 tests passés, 1 ignoré, 0 échec.


Les témoins J, K et L prouvent respectivement la séparation `clientId`/`id` et l'acquittement unique, le refus d'un `clientId` inconnu ainsi que d'un tour tardif, et l'absence totale de preuve sans `clientId`.

## Non vérifié

- Aucun tour fournisseur réel avec contenu utilisateur n'a été exécuté.
- Aucun chemin de remise d'un daemon de production n'a été exercé.
- Aucune activation ni compatibilité de version autre que `codex-cli 0.150.1` n'est revendiquée.
- Le test prouve le contrat de schéma et le comportement isolé de l'adaptateur, pas une livraison de production de bout en bout.
