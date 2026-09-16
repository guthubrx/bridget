# Recette 100

Racine de la version livrée : /Users/moi/Nextcloud/10.Scripts/64.bridget
Ne pas employer le daemon historique. Les tests doivent créer des répertoires
temporaires et configurer explicitement BRIDGET_HOME et BRIDGET_SOCKET.

Commandes de validation à exécuter depuis cette racine :

```sh
export TMPDIR="$(mktemp -d /tmp/bg100-check.XXXXXX)"
export CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target
export CARGO_INCREMENTAL=0
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec100 -- --test-threads=1
/Users/moi/.cargo/bin/cargo test -p bridget-transport --lib spec100 -- --test-threads=1
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test spec100_observation_test
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec099 -- --test-threads=1
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib mcp::tests -- --test-threads=1
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib attach::tests -- --test-threads=1
/Users/moi/.cargo/bin/cargo test -p bridget-transport --lib journal::tests
/Users/moi/.cargo/bin/cargo test -p bridget-transport --lib protocol::tests
/Users/moi/.cargo/bin/cargo test -p bridget-core --lib
/Users/moi/.cargo/bin/cargo fmt --all -- --check
/Users/moi/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
```

Un TMPDIR court évite de dépasser la limite des chemins de sockets Unix macOS.
Les tests CLI créent eux-mêmes home/socket privés ; les tests unitaires utilisent
des sockets simulées ou des paires de sockets et aucun daemon historique.
Ne pas lancer `/Users/moi/.cargo/bin/cargo test --workspace` sans revoir les
harnais existants : support/idempotent.rs et les tests de crash utilisent des
SIGKILL de groupes, interdits par les consignes de cette session. Cette commande
reste BLOQUÉE pour ce motif ; compilation all-targets et suites ciblées ne sont
pas une exécution intégrale du workspace.

Cas manuels couverts par tests : partager 50 entrées Unicode, journal absent,
abonnement ponctuel fin de tour, permission, désabonnement, mauvais propriétaire,
collision entre deux agents, lecture simple sans alerte, destinataire lent.
Une fin de tour n'est pas une preuve de succès. Aucun test ne lance de fournisseur.

Exemples MCP (remplacer les identifiants par les UUID de l'annuaire) :

```json
{"name":"bridget_journal","arguments":{"agent":"UUID_SOURCE","tail":50,"to":"UUID_RELECTEUR","reply":true}}
{"name":"bridget_events","arguments":{"action":"sub","event":"turn_ended","agent":"UUID_SOURCE","once":true,"ttl_secs":1800}}
{"name":"bridget_events","arguments":{"action":"sub","event":"file_collision","file":"/projet/src/*"}}
{"name":"bridget_events","arguments":{"action":"list"}}
{"name":"bridget_events","arguments":{"action":"unsub","id":"ID_ABONNEMENT"}}
```

Les abonnements ne survivent pas au redémarrage du daemon. `once` signifie une
occurrence, pas une livraison garantie. Voir contracts/observation.md pour les
limites et la couverture par intégration. Installation/rechargement hors recette.

## Version installée le 2026-09-16

Le code100 est fusionné dans main ; build installé c680c6ce36c5.
Commande habituelle : /Users/moi/.local/bin/bridget
Source des skills déjà partagée avec les profils :
/Users/moi/Nextcloud/10.Scripts/64.bridget/skills/bridget/SKILL.md

Un nouveau client MCP expose bridget_journal et bridget_events parmi les
14 outils Bridget. Un client MCP déjà ouvert peut conserver l'ancien catalogue :
renouveler sa connexion par le mécanisme du client ou utiliser une nouvelle
session, sans tuer une conversation en cours. Les abonnements demandent
l'identité réelle d'un agent Bridget enregistré ; pas d'identité à inventer.

Le partage d'un journal T3 a été vérifié sur l'installation. Les événements de
fin de tour/écriture restent limités aux producteurs structurés documentés ;
l'installation n'ajoute pas cette observation au pont T3 natif.

Sauvegarde et retour arrière :
/Users/moi/.cache/bridget-adoptions/100-20260916.iMZZbR/receipt.md
