# Archives techniques de l’adaptateur T3

Document d’origine : [specs/125-fin-de-tour-rapide/tasks.md](../../../../../../specs/125-fin-de-tour-rapide/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

# Tâches 125
Statut: Implemented - 4/4, livré le 2026-09-25.
- [x] T001 Tour non dernier, clos, origine relevée : `turn_end` observable (`stop_reason: ended`, identifiant = origine, ou `bridget-observation:` si l'origine est une notification) ; exclu du calcul de lacune (`t3code.rs`).
- [x] T002 Test `spec125_tour_suivi_de_pres_fin_observable_si_origine_connue` (origine connue, inconnue, notification) ; échoue sur l'ancien code.
- [x] T003 Documentation : référence, skill, CHANGELOG (section Non publié).
- [x] T004 fmt, clippy ; recette 1557 réussis, 0 échec (charge 54-68) ; livraison du pont seul.
