# Corpus 089 — gel vérifié avant extraction

17 fichiers épinglés, SHA-256 et comparaison aux objets Git ; LF inclus.
Deux origines distinctes, jamais confondues :

- dfa2134dcfe2a2522e3ae77d93561e6ae72556b3 : 13 fixtures historiques copiées à l'identique.
- bd1cbe0e04d83a1cb5e258bf7df3ed0c6c2fbc14 : scénarios de caractérisation, 42 sorties du codec inchangé lors de la capture, et deux lignes natives extraites des émetteurs de tests historiques.

Le second commit matérialise ce qui n'existait pas en fichiers autonomes dans le
premier. Il ne prétend pas que ces nouveaux chemins existaient dans dfa2134.
Le vérificateur épingle les deux commits et exige leur ascendance ; changer
ensemble un golden et son hash ne suffit pas à le tromper.

## Commandes

Depuis la racine du worktree :

```sh
sh scripts/verify-089-contracts.sh --self-test --require-complete
PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline -p bridget-transport --test core_089_wire_test
```

Les cinq familles matérialisées dans wire-reference.jsonl sont comptées d'après
les lignes effectivement présentes, pas seulement leur étiquette de manifeste :
messages/idempotence, annuaire/ledger, lifecycle, attach, guichet claim/reply.
Les 42 cas incluent les sept issues idempotentes, les cinq issues stop, corps
riche et corrélation, une présence native, un ledger non vide et le token/lease.

Chaque ligne contient la direction et une chaîne wire : **cette chaîne garde
l'ordre et les octets du codec**, contrairement à une structure JSON qui serait
triée/réencodée avant comparaison. Le test compare l'émission aux bytes figés,
puis les décode et les réémet par le codec réel. Aucun DTO parallèle.

## Provenance de matérialisation

Commande exécutée AVANT les changements de production :

```sh
PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline -p bridget-transport --test core_089_wire_test core_089_materialize_reference -- --ignored --nocapture
```

Les lignes CAPTURE de stdout ont été archivées telles quelles, préfixe technique
retiré. Le générateur n'écrit aucun fichier et est ignoré par défaut. À cet
instant protocol.rs et core/message.rs étaient identiques au commit source.
Les corrections Clippy ultérieures n'ont changé ni encode/decode, ni les champs
des trames : Default dérivé identique et suppression d'un format de test.

Les lignes natives proviennent des émetteurs shell de tests de dfa2134 :
codex_app_server.rs:3303 et claude_stream_json.rs:1467. Le consommateur vérifie
raw/source/origine ; seul le délimiteur LF est retiré explicitement. Les vrais
pilotes lisent les flux de faux fournisseurs : **pas une recette de compte réel**.

## Limites du gel

C'est une caractérisation de chaque famille conservée, pas une preuve de tous
ses comportements, schémas fournisseur exhaustifs ou canons SQL. T014/T018
exercent CLI/MCP/daemon/store réels ; T020/T021 valident les fournisseurs réels.
Les tests de clients externes encore logés dans Maicie sont classés à déplacer
par T003 et doivent survivre à son retrait.

provider-codex-0.150.1.jsonl reste une fixture historique de forme : son champ
jsonrpc n'est PAS promu comme contrat du pilote natif qui l'interdit. Le témoin
native-codex-delta.jsonl préserve une vraie ligne de son émetteur de test sans ce
champ. Aucun changement du pilote pour satisfaire une fixture périmée.

L'auto-test altère seulement une copie privée : octet modifié, hash réécrit,
source absente, fixture absente, famille effacée et commit existant non ancêtre
sont refusés. L'intégrité du corpus ne remplace jamais les gates d'exécution.
