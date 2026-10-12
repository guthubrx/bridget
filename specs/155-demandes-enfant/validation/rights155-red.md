# Rouge T006/T008

Les trois fonctions de test sont écrites avant la production155 : table de 31 cas de droits, refus des chemins sous workspaceWrite, affichage borné/nettoyé sans effet sur le verdict.

RED de compilation seulement : RC101 en 11,39 s. Les huit erreurs E0425 correspondent exclusivement à classify_request et format_cited_paths absents. Les sources sont identiques avant et après.
Aucun cas de la table n’a été exécuté dans cette phase rouge. Ce résultat ne prouve aucun comportement. La phase verte doit exécuter toutes les assertions.

Les fonctions de production sont autorisées après ce constat. Les tests restent figés.
