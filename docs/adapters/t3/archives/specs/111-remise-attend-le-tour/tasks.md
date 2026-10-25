# Archives techniques de l’adaptateur T3

Document d’origine : [specs/111-remise-attend-le-tour/tasks.md](../../../../../../specs/111-remise-attend-le-tour/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T001 Écrire les tests dans crates/bridget-daemon/src/t3code.rs ; attendu : rouges avant T002.
- [x] T002 Retirer la péremption par turn_wait aux deux points de filtrage ; attendu : T001 verts.
- [x] T003 Borner la file à l'insertion avec un avertissement distinct ; attendu : test de borne vert.
- [x] T004 fmt, clippy, recette complète ; attendu : 0 échec.
- [ ] T005 Livrer en production, vérifier les remises en attente et le journal ; consigner dans specs/111-remise-attend-le-tour/implementation.md.
