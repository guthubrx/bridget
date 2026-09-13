# Recette 094

Après installation ET activation des versions compatibles du daemon, du wrapper
et du serveur MCP de la session, découvrir les outils dans le catalogue. Un MCP
nouveau sous une ancienne liste d'autorisations fournisseur peut rester refusé.
Ne pas provoquer de redémarrage des gérés sans accord humain. Utiliser ensuite :

```json
{"name":"bridget_rename","arguments":{"display_name":"gui"}}
{"name":"bridget_dnd","arguments":{"mode":"on","duration":"30m"}}
{"name":"bridget_dnd","arguments":{"mode":"off"}}
{"name":"bridget_domain","arguments":{"domain":"mon-projet"}}
{"name":"bridget_domain","arguments":{"reset":true}}
{"name":"bridget_status","arguments":{}}
{"name":"bridget_control_status","arguments":{"history_limit":5}}
```

Comparer l'UUID avant/après par bridget_who, jamais le déduire du nom.
bridget_runtime déclare une information connue, ne sélectionne pas un modèle.
Ne pas l'appeler pour tester avec une valeur inventée sur une présence vivante.
Ne pas activer DND sur un autre agent pour une recette.

Validation développeur ciblée : `cargo test -p bridget-daemon --features test-support spec094_` ;
TMPDIR privé existant et PATH cargo explicite. Les tests qui nécessitent des
processus utilisent leurs homes/sockets privés et arrêtent seulement leurs
enfants. La consolidation n'exige aucun redémarrage du daemon utilisateur.
