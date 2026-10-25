# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/tasks.md](../../../../../../specs/151-attente-fin-de-tache/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T014 [US4] Écrire `native151_groupe_trois_resultats_une_reponse` (trois enfants terminés dans le même intervalle : une réponse, trois entrées dans l'ordre de `task_ids`, `pending` vide) et, dans `crates/bridget-daemon/src/t3code.rs`, un test qui prouve que `batch_family` classe un message `native-result-*` (`reply = false`, sans `thread_notice`) en famille `Message` et que `batchable_prefix` en groupe trois. Corriger `Wait` si l'ordre ou le contenu diffère. Résultat : tests verts (SC-004).
