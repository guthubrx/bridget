# Validation rapide

1. Lancer deux wrappers Bridget nommés `demandeur` et `executant`.
2. Depuis `demandeur`, envoyer une demande :

   ```bash
   bridget send --to executant --reply --timeout 9 "analyse ce point"
   ```

3. Noter l'identifiant retourné puis vérifier qu'il apparaît dans :

   ```bash
   bridget requests
   ```

4. Annuler avant le premier rappel :

   ```bash
   bridget cancel <id> --reason "priorité changée"
   ```

5. Vérifier que `executant` reçoit une notification sans réponse requise et qu'aucun rappel n'apparaît après neuf secondes.
6. Redémarrer le daemon, puis vérifier que `bridget requests` conserve l'état `cancelled`.
