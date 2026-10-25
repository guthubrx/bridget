# Archives techniques de l’adaptateur T3

Document d’origine : [specs/113-fiabilite-remises/tasks.md](../../../../../../specs/113-fiabilite-remises/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

# Tâches 113
Statut: Implemented — 5/5, livré en production le 2026-09-19 à 21:06.
- [x] T001 Tests spec113 dans crates/bridget-daemon/src/t3code.rs (saturation, échéance, demande close).
- [x] T002 `Discard`, `discard_cause`, `report_discard`, `discard_by_id` ; quatre points d'écartement rapportés.
- [x] T003 Documentation : attente sans borne de durée, file bornée, écartement rapporté ; routes dans un terminal.
- [x] T004 fmt, clippy, recette complète (1524 réussis, 0 échec, 52 ignorés).
- [x] T005 Fusion dans main, reconstruction, relance du pont, vérification en production, implementation.md.
