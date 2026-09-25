# Tâches 125
Statut: Implemented - 4/4, livré le 2026-09-25.
- [x] T001 Tour non dernier, clos, origine relevée : `turn_end` observable (`stop_reason: ended`, identifiant = origine, ou `bridget-observation:` si l'origine est une notification) ; exclu du calcul de lacune (`t3code.rs`).
- [x] T002 Test `spec125_tour_suivi_de_pres_fin_observable_si_origine_connue` (origine connue, inconnue, notification) ; échoue sur l'ancien code.
- [x] T003 Documentation : référence, skill, CHANGELOG (section Non publié).
- [x] T004 fmt, clippy ; recette 1557 réussis, 0 échec (charge 54-68) ; livraison du pont seul.
