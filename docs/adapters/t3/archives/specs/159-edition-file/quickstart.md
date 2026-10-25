# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/quickstart.md](../../../../../../specs/159-edition-file/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Tests du crate transport : même environnement, `cargo test -p bridget-transport <filtre>`.
Suite complète avant clôture : même environnement,
`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast`.
Un changement de trame, de reçu ou de file exige aussi tous les tests `t3code`,
`mcp::`, `guidance156` et transport, pas seulement les nouveaux.

<a id="extrait-2"></a>

## Extrait 2

Non-régression : filtres `spec_087`, `native148`, `native149`, `native150`, `guidance156`, `t3code`, `mcp::`.

<a id="extrait-3"></a>

## Extrait 3

La série autorisée permet build signé, installation et activation Bridget après vérification de zéro mission native non terminale. T3 reste protégé, sans relance. Aucun succès de recette B n’est annoncé ici. Après activation :

<a id="extrait-4"></a>

## Extrait 4

SC010 étend la recette ouverte existante : 24 appels privés, quatre acteurs,
trois mutations et deux façades. Aucun T3 réel n’est impliqué. Les délais
d’attente tiennent compte de la réservation FR024 (ACK + 30 s) ; un timeout
de fixture trop court n’est pas une preuve de libération défectueuse. Les
24 assertions des façades ont réussi dans la recette ouverte privée.
La recette ouverte a réussi au run final ; la recette fermée avait réussi
dans le lot précédent. Deux recettes distinctes PASS cumulées, pas un run
unique 2/2. Le FULL unique et ses replays sont conservés séparément dans le reçu.
La livraison reste distincte des gates techniques. Reçus :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/tests-159.md
