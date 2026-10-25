# Plan 170

1. Vérifier les chemins `capture_remote_result`, `commit_remote_result`,
   `mark_answered` et l’entretien `tick`.
2. Ajouter une clôture corrélée après acceptation distante, avec reprise bornée
   par l’entretien et conservation indépendante de l’ACK transport.
3. Étendre le banc `remote162` : succès avant deadline, rejeu identique,
   échec SQL réessayable et absence/expiration conservée.
4. Exécuter uniquement formatage et tests ciblés Cartae, puis consigner les
   SHA-256 et limites dans `validation/`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/170-delais-reponses-distantes/plan.md#extrait-1).
