# Vérification manuelle - SPEC-068

1. Lancer un parent géré puis créer un enfant avec une propriété de délégation.
2. Provoquer dans l'enfant un refus d'outil normalisé.
3. Vérifier dans la conversation du parent une notification système contenant:
   enfant, catégorie `avertissement`, code et référence redacted.
4. Vérifier que l'enfant peut ensuite produire une réponse normale et que le
   parent n'a reçu aucun texte affirmant l'échec de la tâche.
5. Provoquer un terminal fournisseur en erreur dans une autre exécution enfant.
6. Vérifier une notification `échec terminal` distincte.
7. Déconnecter le parent avant un nouveau fait, reconnecter le même parent et
   vérifier la remise une seule fois.
8. Vérifier qu'aucun état Maicie n'a changé: le fait apparaît uniquement comme
   diagnostic runtime Bridget.
