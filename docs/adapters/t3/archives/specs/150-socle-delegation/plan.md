# Archives techniques de l’adaptateur T3

Document d’origine : [specs/150-socle-delegation/plan.md](../../../../../../specs/150-socle-delegation/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- La contre-revue et le sort de chaque objection sont dans
  `adversarial-review-codex.md`.
- Non vérifiable dans ce dépôt : la réconciliation de liste côté T3 après purge.
  R9 contourne ce point par un snapshot complet, déjà exigé par le contrat 149.

<a id="extrait-2"></a>

## Extrait 2

| Risque | Parade |
|---|---|
| Purge d'un arbre avec remise en vol | exiger la phase `acked` ; test avec `OutcomeUnknown` |
| Purge d'un arbre dont l'enfant tourne | contrôle présence + flotte + lancement sous verrou ; test de relance après nettoyage |
| Lecteur T3 garde une tâche fantôme | nouvelle génération + signal après commit |
| Remises indéterminées qui saturent la borne | refus `retention_saturated` explicite, documenté |
