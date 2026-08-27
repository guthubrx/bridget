# Plan 042 — Fixtures d'intégration éphémères

## Décision

Introduire dans le seul banc lourd une petite garde de racine, à durée de vie
lexicale. Sa destruction tente d'effacer la racine sans paniquer. La racine
utilise un UUID, indépendant du PID. Cette garde ne connaît ni daemon ni
processus : `DaemonGuard` reste responsable de l'enfant qu'il possède déjà.

## Pourquoi ce périmètre

- le seul répertoire mesuré à environ 1,2 Gio est le `target/` privé de
  `crates/bridget-daemon/tests/build_id_integration_test.rs` ;
- ce test a déjà une garde de processus correcte, mais aucun propriétaire de
  sa racine ;
- les répertoires de `sigkill_daemon...` sont plus petits, portent déjà un
  UUID et correspondent à un rouge connu : les modifier ici diluerait le
  correctif sans établir qu'ils ont produit les 6,3 Gio mesurés.

## Complexité et sûreté

La garde fait une suppression récursive unique à la sortie du banc : O(n), où
n est le nombre de fichiers strictement contenus dans sa propre fixture. Elle
n'accepte aucun chemin extérieur et n'effectue aucune suppression globale.

## Vérification

1. Tests légers : unicité et nettoyage après panique capturée.
2. Banc de couture existant : comportement normal inchangé.
3. Mutants : neutraliser `Drop` doit faire échouer le témoin de panique ;
   réduire l'identifiant au PID doit faire échouer le témoin d'unicité.
4. Construire les états base et tête avec des cibles de compilation distinctes
   avant de comparer les comptes de tests.
