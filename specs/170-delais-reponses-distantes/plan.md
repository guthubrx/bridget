# Plan 170

1. Vérifier les chemins `capture_remote_result`, `commit_remote_result`,
   `mark_answered` et l’entretien `tick`.
2. Ajouter une clôture corrélée après acceptation distante, avec reprise bornée
   par l’entretien et conservation indépendante de l’ACK transport.
3. Étendre le banc `remote162` : succès avant deadline, rejeu identique,
   échec SQL réessayable et absence/expiration conservée.
4. Exécuter uniquement formatage et tests ciblés Cartae, puis consigner les
   SHA-256 et limites dans `validation/`.

5. Livrer0.4.1 depuis le même commit sur Mac et Cartae, sans relance T3.
6. Vérifier une recette distante et son rejeu, puis l’absence de rappel après échéance.
7. Publier les quatre actifs, les retélécharger et finaliser le nettoyage gardé.
