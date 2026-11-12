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

Publication 0.8.51 : https://github.com/guthubrx/bridget/releases/tag/v0.8.51.
Commit de code : 13fc087c6bf099a18e3c60f657de1a36b2fb71ec. Les six actifs
publics sont téléchargés sans authentification et leurs empreintes vérifiées.
Les services prévus sur monordinateur, macbook-m1, Cartae, SYSLE2 et KS
chargent 287-role-specialistes-r1. Le mobile répond HTTP 200. Le retour
286 reste qualifié et les pointeurs de retour existants sont vérifiés.
Le passage du cœur dure 3,9 secondes. Sur SYSLE2, la sauvegarde attend des
entrées-sorties observées dans folio_wait_bit_common, puis termine normalement.
Les marqueurs des processus métier conservés sont vérifiés après activation.

Limite d’activation : T3 conserve PID 77762 et sa naissance du 11 novembre
à 14:10:41. Une ancienne façade rend encore le refus générique après passage.
Le nouveau guidage est donc actif pour les processus MCP neufs, pas pour
toutes les façades déjà ouvertes. Le rechargement reste différé conformément
au choix de garder T3 ouvert. La consultation list sans rôle fonctionne dans
la session existante. Les six propriétaires prévenus reçoivent un avis de fin
conservé sans réveil ; stored ne prouve pas sa lecture.

Ménage : le parcours installed contrôle les caches Mac et SYS, sans cache
reconnu restant lors du dernier contrôle. Aucun worktree ou branche éligible
au retrait dans le rapport natif. Une ancienne image est retirée sur KS
(45 389 784 octets) et sur M1 (37 517 776 octets). Les images Mac encore
utilisées restent présentes ; SYS conserve l’image 285 observée ouverte.
Cartae conserve les anciennes images faute d’observation complète des usages.
Les copies de transport propres au lot sont retirées après alignement des
sources ; archives publiques, journaux et preuves restent dans le dossier privé.
