# Progrès T004–T006 159A

Reçu de progrès en cache. Aucune tâche cochée. T003 reste ouverte avant T011. Aucune suite globale ni GREEN global.

| Exécution | Résultat constaté | Durée | RC |
|---|---|---:|---:|
| T004-rank-GREEN-adopt-T006-temporal-RED | Daemon interrompu : 17 statuts terminés (10 PASS,7 FAIL),1 cas bloqué, aucun résumé final daemon. Transport : 1 PASS. |179,79s|101|
| T006-periodic-RED-fixture-replay | RED réel :0 PASS,1 FAIL sur la réattestation périodique. |11,90s|101|
| T004-T006-GREEN | Daemon : 17 PASS,1 FAIL ; transport : 1 PASS. T004 : 10 PASS ; T006 : 8/9 PASS. |25,40s|101|
| T006-force-expiry-GREEN-fixture-replay | Cas exact de sortie forcée : 1 PASS,0 FAIL. |19,85s|0|

Le blocage initial est observé dans `__sendto`. Le sample identifie le processus de test 7577. ROOT atteste sa terminaison individuelle par SIGTERM ; le journal confirme le signal 15. Ce run incomplet ne constitue pas une exécution daemon terminée.

La fixture périodique effectue 601 passages et, selon ROOT, dix trames. Elle ne pousse pas 600 événements socket. Son replay RED échoue sur «réattestation même sous trafic continu», puis le groupe terminé passe ce cas.

La sortie forcée échoue dans le groupe terminé. Le replay exact passe après montage du vrai BRIDGET_HOME privé et du registre à une seconde. HOME seul était ignoré. La cause appelle une correction de fixture, pas un changement de production. La restauration T011 et les autres tâches ne sont pas créditées par ce reçu.

Chaque reçu compare 222 empreintes avant/après, toutes égales pendant son run. Entre le groupe et le replay, daemon.rs, idempotency/send_delivery.rs et protocol.rs changent ; leurs empreintes sont conservées dans le JSON. Le succès isolé ne transforme pas le groupe précédent en un run unique réussi.

Les 19 cas prescrits sont ainsi réussis cumulativement : dix T004 dans le dernier groupe, huit T006 dans ce groupe et un T006 dans le replay isolé. Ce constat ne promet aucune couverture globale des sources actuelles.

Provenance des reçus et journaux :

/Users/moi/.cache/bridget-tests159A-20261012/T004-rank-GREEN-adopt-T006-temporal-RED.json
SHA256 reçu : d025f36cf2c5e5c8d7e821721dcad43d5026a0765cb7b6f977d70ab3ebaaf05d
/Users/moi/.cache/bridget-tests159A-20261012/T004-rank-GREEN-adopt-T006-temporal-RED.log
SHA256 journal : 07f7457d85c45f64aa8eea32bdcfe86a1207d60d311950c334e370aa32950898

/Users/moi/.cache/bridget-tests159A-20261012/T006-periodic-RED-fixture-replay.json
SHA256 reçu : 2482e3faf9307856a2dc310e086854148d9888113bade896894d8dc81bd4ec97
/Users/moi/.cache/bridget-tests159A-20261012/T006-periodic-RED-fixture-replay.log
SHA256 journal : d80d375cfddf10dea28daddbfbc255b63ae480f151ae1703518d08bd5cdd9615

/Users/moi/.cache/bridget-tests159A-20261012/T004-T006-GREEN.json
SHA256 reçu : 47ed5bbcee967d808b3c6815000f81588249b316b4cb8d5366efe0ed75ee4c25
/Users/moi/.cache/bridget-tests159A-20261012/T004-T006-GREEN.log
SHA256 journal : 39bf808b5f99e07c1f2ed1286ccecfc336a2d768d1b92aae82f12247fb75c8e8

/Users/moi/.cache/bridget-tests159A-20261012/T006-force-expiry-GREEN-fixture-replay.json
SHA256 reçu : 13e68f834b11ba096df105b08cd275f1dcabb2dfe99361648395be58ab4f0b85
/Users/moi/.cache/bridget-tests159A-20261012/T006-force-expiry-GREEN-fixture-replay.log
SHA256 journal : 2bbea69e022c50540943f954ce3116bacf529c1ddc40f695c28a1c1692b05056

/Users/moi/.cache/bridget-code159A-T006-20261012/RED-periodic-sample.txt
SHA256 sample : 88f31e7e37bf01cf1138478d792200682ddaca2a3d9a54a55e2efd93c18c85d8

La liste exacte des cas, les commandes, namespaces privés et empreintes complètes par run figurent dans :
/Users/moi/.cache/bridget-tests159A-20261012/T004-T006-progress159A.json
