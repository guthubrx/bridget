# Dégradations 149 - couche adaptateur de bureau (T039 partiel)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), r2. Aucun edit de production, aucun commit, aucune case cochée.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/degradation149.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/degradation149.md#extrait-2).

## Compteurs

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/degradation149.md#extrait-3).

Total : 13 PASS, 0 FAIL, 1 SKIP (groupe natif).

## Comptage des exécutions et des processus

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/degradation149.md#extrait-4).

## Constats pour l'owner de production (aucun patch de ma part)

1. **Réécriture O(n) à chaque mutation.** Un seul ajout de journal sur une tâche (4 tâches) produit 4 `subagent.updated` + 5 `thread.metadata-updated` ; les 3 autres tâches n'ont pas changé. Avec 134 tâches, une mutation ajoute environ 270 événements. Sur la recette, le journal d'événements totalise 2082 lignes (1147 `thread.metadata-updated`, 778 `subagent.updated`, 137 `thread.created`). À l'inverse, 100 lectures sur snapshot inchangé n'ajoutent rien. Reproduction : `scenario_step.mjs <store> journal`, puis comparer `select event_type,count(*) from orchestration_events where sequence > <max avant>`.
2. **Fils orphelins.** Après `scenario_step.mjs unseed` (130 tâches retirées du magasin, même génération), les 130 fils virtuels restent en base avec leur ancien marqueur (`seq 28`, statut `cancelled`), ni supprimés ni archivés. Le panneau Lineage affiche « Previous agents (133) » pour 4 tâches natives (capture 11). À confirmer : intentionnel (historique) ou fil à retirer sur snapshot complet vérifié.
3. **Indisponibilité d'une seule tâche = tout le lineage.** `journal_unavailable` sur T4 fait afficher « Lineage · Bridget indisponible » et « Dernier état connu » pour tout le fil racine, tâches saines comprises, jusqu'au prochain `list` réussi. Le contrat dit seulement « garde l'historique vérifié sans le présenter comme état courant » ; la portée n'est pas précisée.
4. Voir aussi `ui149.md` (bouton « Arrêter » masqué à 1280 px).

## Reprise

Les scénarios natifs se rejouent avec le binaire natif (reçu `native149-debug-receipt.json` frais). Les outils sont prêts : `scenario_step.mjs` (étapes `journal`, `nested-status`, `result`, `cancelled`, `generation`, `gap`, `journal-off`, `store-corrupt`, `hang`, `seed`, `unseed`, `cursor-bump`), `matrix149.mjs`, `db_counts.sh`. Étape `reset` : non implémentée (réexécuter `fixture_store_init.mjs`).
