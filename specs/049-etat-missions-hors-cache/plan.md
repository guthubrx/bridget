# Plan 049 — État des missions hors cache

## Pourquoi corriger le générateur

Maicie ne possède pas de chemin implicite : elle exige `database_path` dans sa
configuration. Ajouter une valeur par défaut au Rust créerait une seconde
source de vérité et laisserait l'installateur reproduire le défaut. Le bon
point de correction est donc le seul écrivain de la configuration initiale,
`scripts/install-k1.sh`.

## Dérivation du chemin neuf

1. Si `XDG_STATE_HOME` est absolu et non vide, l'utiliser comme racine d'état.
2. S'il est absent, vide ou relatif, utiliser `${HOME}/.local/state` ; le cas
   relatif produit un avertissement parce que la spécification XDG demande de
   l'ignorer.
3. Définir la base neuve sous `maicie/maicie.sqlite3` et créer ce répertoire en
   0700.
4. Conserver `CACHE_DIR` pour les artefacts réellement remplaçables et le
   socket runtime existant ; leur déplacement n'appartient pas au lot.

## Installation existante

Après le préflight réussi de la configuration figée, un lecteur JSON extrait
`database_path` et le compare aux racines de cache utilisateur connues :
`${HOME}/.cache` et un `XDG_CACHE_HOME` absolu lorsqu'il est défini.

Si le chemin se trouve sous l'une d'elles, l'installateur avertit. Il ne
réécrit pas le JSON, ne déplace aucun fichier et ne change pas le code retour.
La lecture a lieu après le préflight afin qu'un JSON invalide reste attribué au
gate de configuration plutôt qu'au diagnostic de stockage.

## Oracles et mutant

Le banc existant de l'installateur est étendu avec trois issues distinctes :

1. configuration absente : chemin durable exact et répertoire 0700 ;
2. configuration cache explicite : avertissement présent et octets conservés ;
3. configuration durable explicite : avertissement absent et octets conservés.

Un quatrième témoin donne un `XDG_STATE_HOME` relatif et exige le repli absolu.
Le contrôle positif vérifie chaque JSON réellement publié ; aucun succès ne
repose sur un fichier absent ou un filtre vide.

Le mutant remet uniquement `MAICIE_STATE_DIR` sous `CACHE_DIR`. Il doit faire
mourir le témoin de configuration absente à l'égalité du chemin, tandis que les
deux témoins de conservation restent verts.

## Portée de validation

Le delta fonctionnel est shell. Les gates sont le banc
`scripts/test-install-k1-preflight.sh`, `bash -n`, `shellcheck` lorsqu'il est
disponible et `git diff --check`. Aucun test Rust n'est nécessaire si aucun
fichier Rust n'est modifié. La base et la tête sont mesurées dans la même passe.
