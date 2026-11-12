# Recherche des spécialistes

Le schéma MCP exige déjà role_id pour search et le type transport est String.
La description facultatif est donc fausse. Le contrat est conservé : search
exige un rôle, list consulte le catalogue sans filtre. Le refus local nomme
le champ absent ou du mauvais type et donne l’action list.
Aucun changement de wire, de permission ou d’état. Les anciennes façades
peuvent conserver l’ancien guidage tant que T3 reste ouvert.

Revue indépendante non relancée : le précédent refus de tous les profils
de lecture seule n’a pas été résolu. Aucun contournement ou nouveau droit.
Preuves privées : /Users/moi/.cache/bridget287-role.

Un appel réel list avec version numérique et sans rôle réussit depuis la
session existante. La description chargée y conserve encore facultatif,
ce qui confirme la limite des façades anciennes. Le test de contrat vérifie
les champs requis de search et list, les refus locaux et les formes valides.

La recette cumulée est reconstruite et ses quatre fichiers comparés octet
par octet avec les fichiers de qualification. Aucun build de main.

Les 20 tests distincts du module delegation_mcp passent sur Mac et Linux.
Le test 287 est aussi lancé séparément ; il fait partie de ces 20 tests,
pas d’un vingt-et-unième cas. Le format du nouveau test et git diff --check
passent. Clippy et release Mac passent ; le binaire signé contient la nouvelle
description et plus le texte facultatif. Clippy strict et release Linux passent également. Le contrôle ABI demande
glibc 2.34, compatible avec Debian 12 (maximum 2.36).
L’APK 8051 est construit et sa signature vérifiée contre 0.8.50.

Un processus MCP neuf du binaire Mac 0.8.51 exécute initialize, initialized
puis tools/list. La description renvoyée contient role_id obligatoire et ne
contient plus facultatif. Aucun tools/call ni appel métier n’est émis par ce
harnais. Le processus se termine normalement sur EOF.
