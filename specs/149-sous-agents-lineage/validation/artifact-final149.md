# Contrôle du paquet final 149

## Périmètre

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/artifact-final149.md#extrait-1).

Aucune installation, aucun redémarrage, aucun GUI, aucun serveur, aucun modèle et aucune base de production n'ont été utilisés.

## Résultats

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/artifact-final149.md#extrait-2).

Détails du smoke SQLite : Electron 44.4.5, Node 24.21.0, modules 149, SQLite 3.53.4. `guiStarted`, `serverStarted` et `modelCalled` valent `false`.

## Incident de manipulation

Mon premier appel du smoke SQLite a utilisé `--help` comme chemin de base. Le fichier `./--help` a donc été créé dans `/Users/moi/Nextcloud/10.Scripts/64.bridget`. Ce fichier de 8192 octets venait de mon appel. Je l'ai supprimé. Le smoke final a ensuite été relancé avec la base privée décrite plus haut. Ce résultat n'a pas été utilisé comme preuve.

## Limites

- Le vérificateur indique lui-même que la signature doit être contrôlée par une commande séparée. Cette commande a été lancée et a réussi.
- Le smoke SQLite ne lance pas l'application complète.
- Les erreurs anciennes de lint et de typecheck (serveur 16, web 10) restent la référence de base. Aucun nouveau lint ni typecheck n'a été lancé.
- Les recettes Rust 1840 et les autres recettes déjà validées n'ont pas été rejouées.
- Aucun contrôle git, cargo ni build n'a été lancé.
