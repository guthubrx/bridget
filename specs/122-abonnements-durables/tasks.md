# Tâches 122
Statut: Implemented - 4/4, livré le 2026-09-25 à 08:13.
- [x] T001 `Observations::restore` reprend les abonnements (`restored`, écart ouvert au démarrage) ; champ `interrupted` retiré ; `owner_returned` annonce la reprise une fois (`observation.rs`) ; catalogue `coverage.lifetime` mis à jour (`daemon.rs`).
- [x] T002 Tests : `spec122_restore_resumes_and_absolute_ttl_is_preserved` (source absente puis revenue) ; test d'intégration avec vrai redémarrage adapté (reprise sans `sub`, fin de tour remise).
- [x] T003 Documentation : `docs/reference-communication.md`, `skills/bridget/references/commandes.md`, ADR 043, README (47 ADR).
- [x] T004 Recette réunie 121-123 : 1556 réussis, 0 échec ; livraison et vérification en production.
