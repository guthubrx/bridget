# Archives techniques de l’adaptateur T3

Document d’origine : [specs/153-rondes-contexte/adversarial-review-codex.md](../../../../../../specs/153-rondes-contexte/adversarial-review-codex.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Note de l'agent revue (reprise)** : les validateurs 149 ferment la liste des
états sans imposer l'irréversibilité (`task_entry`, `delegation_lineage.rs:35`) :
le retour `result_available` vers `mission_pending` n'est pas bloqué côté
Bridget. La compatibilité du lecteur T3 reste à prouver (A2, non vérifiable dans
ce dépôt). L'expiration pendant une pause est cohérente avec 152 : une tâche
inactive est terminale, donc sans `suspended_at` (R11).

<a id="extrait-2"></a>

## Extrait 2

Statut du plan : les six objections sont traitées. Ouverts : le lecteur T3
(A2), le risque de faux refus de l'égalité de politique (A10) et la livraison
réelle de 150 et 151 (A1). Pas de
ronde 2 de contre-revue lancée ; elle reste recommandée avant T002.

<a id="extrait-3"></a>

## Extrait 3

Aucune tâche ajoutée : les trois corrections complètent T004, T008, T010, T011,
T013, T014 et T015. Total : 29 tâches. Ouverts : lecteur T3 (A2), stabilité de
l'égalité de politique à confirmer par le test de T014 (A15), livraison réelle de
150 et 151 (A1).
