# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-network-proofs-r4.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-network-proofs-r4.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Fixture de départ : racine réelle `waiting_for_children` avec résultat réel `fixture149-answer:e1` et exécution racine `completed`. Le descendant réel est `working`.
**SIMULÉ** (E2r4.b) : daemon arrêté, transaction exacte d'admission rejouée par `craft-queued.py` pour deux lignes `queued` : un clone du descendant (propriétaire = enfant natif, parent = racine) et un clone externe (propriétaire = fil T3 A). Après la relance, la recette n'écrit plus rien en base.
