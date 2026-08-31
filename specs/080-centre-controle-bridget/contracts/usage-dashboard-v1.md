# Contrat relay - Usage Dashboard v1

## `GET /v1/control/usage`

Paramètres:

- `from` et `to`: secondes Unix inclusives, période bornée.
- `timezone`: `system` n'est jamais envoyé au serveur. Une IANA explicite peut définir les libellés journaliers.
- `providers`, `models`, `projects`: filtres répétés facultatifs, validés contre les dimensions retournées.

Réponse:

- `coverage`: premières et dernières observations, nombre d'échantillons et dimensions inconnues.
- `totals`: jetons entrée, sortie, création cache, lecture cache et tours.
- `groups`: regroupements par fournisseur, modèle et projet uniquement si attestés.
- `daily`: agrégats par jour et fuseau déclaré.
- `estimated_cost`: montant, devise, tarifs appliqués ou état `unpriced` avec raisons.

Invariants:

- aucune observation ne devient zéro par absence;
- aucun total coût n'est retourné si un groupe pertinent ne possède pas de tarif applicable;
- la source, la version de tarif et la couverture restent visibles;
- aucun prompt, message, sortie, argument, secret ou clé n'est inclus.
