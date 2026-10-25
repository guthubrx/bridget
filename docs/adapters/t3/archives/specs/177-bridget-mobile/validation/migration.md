# Archives techniques de l’adaptateur T3

Document d’origine : [specs/177-bridget-mobile/validation/migration.md](../../../../../../../specs/177-bridget-mobile/validation/migration.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le maître optimisé r6 est actif sur Cartae. Le maître Mac est arrêté et désactivé ; T3 reste chargé. La copie hors ligne récente contient 360 Tasks et 431 reçus. Toutes les tables applicatives ont le même nombre de lignes et le même SHA-256 de lignes avant et après copie, avant démarrage. quick_check est ok et foreign_key_check ne donne aucune erreur. Les tâches distantes conservent leurs placements ; les processus métier du worker n’ont pas été tués. Les anciens fichiers de cible sont conservés dans une sauvegarde privée distincte. Aucun maître n’a été actif simultanément.
