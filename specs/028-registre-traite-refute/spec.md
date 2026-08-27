# Spec 028 — Registre : fermé ≠ réfuté (et list lisible)

**Statut** : livré (tip d5d4b26, base a9353c1)  
**Branche** : `session-028-registre-ferme-refute`  
**Base** : `a9353c11291829ce762365b9d2af3cb267d73567`  
**Migration SQLite** : aucune (journal catalogue + projection).

## Propriété

L'état d'un constat au registre distingue **ouvert**, **fermé** (traité), **réfuté**
(et **requalifié**), et la **liste** le montre — pas seulement les données.

- **Fermé** : le défaut existait, il est corrigé → *ce fut vrai, ce ne l'est plus*.
- **Réfuté** : le défaut n'a jamais existé → *ce ne fut jamais vrai*.
- **Requalifié** : vrai, mais sévérité/portée changée → reste une charge réelle.
- Si le registre ne sait que « fermer », retirer une charge fausse **ment**.

## Append-only

`docs/catalogue-du-du.md` : une ligne JSON par entrée. Une fermeture / réfutation
est une **nouvelle** ligne `kind=transition` qui référence le constat — jamais
une réécriture. `app::reconcile_catalogue_from_store` pousse déjà store → journal.

## Critères d'acceptation

1. Gestes CLI `registre fermer|refuter|requalifier` avec raisons typées + `--ref` obligatoire.
2. `registre list` : **ouverts par défaut** ; `--fermes` / `--refutes` **restreignent** le corps à cet état (raison typée + réf) — pas un élargissement sous les ouverts ; `--attente` seul déplie ; pied O/F/R/Q toujours.
3. Oracles TEMOIN_A (ouvert), TEMOIN_B (fermé quitte le défaut, visible dans `--fermes` sans ouverts), TEMOIN_C (réfuté ≠ fermé), TEMOIN_D (fermé apparaît dans la vue des fermés, après preuve défaut non vide) — attentes en dur.
4. Fermeture/réfutation sans preuve refusée ; réfutation refuse `sha:` seul.
5. Suite maicie : comptes passed/failed annoncés ; taux concurrence mesuré (base comprise).
