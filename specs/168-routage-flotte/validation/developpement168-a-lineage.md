# Développement168 — lot A Lineage

Date : 2026-10-15.
Owner : DEV168_A.
Portée : T029 Lineage et compatibilité Lineage/CLI de T034.

## Projection v4

- Les lectures `list` et `show` acceptent la version4.
- La version4 lit exclusivement `metadata_version=2` puis le seul objet
  `metadata.decision`, désérialisé comme `LineageDecisionV4` fermé.
- La version3 rend `decision:null` pour toute tâche168 ; elle conserve son
  placement. Une tâche163 reste inchangée en v3 et rend `decision:null` en v4 :
  `c01` n'est jamais interprété comme `c001`.
- Le metadata complet, la politique, les faits et secrets restent hors projection.

## Contrat consommé

`metadata.decision` vaut `null` dans le snapshot initial avant choix, puis un
objet atomiquement choisi avec :
`candidate_id`, `connection_id`, `machine_id`, `engine`, `model`, `effort`,
`source` et `reason` optionnelle. Toute forme invalide ferme la lecture par
`store_unavailable` ; aucun champ n'est deviné depuis les colonnes163.

## Oracles source ajoutés, non exécutés

- Projection v4 complète, v3 nul pour168, compatibilité163 et absence de fuite privée.
- Versions invalides, lecture show et conservation du placement.
- CLI v4 `list/show` : succès à la version demandée et refus sans downgrade.
- Parseur CLI v4 et refus de v4 hors lectures paginées.

## Contrôles réalisés

- `rustfmt --edition 2024 --config skip_children=true` ciblé : succès.
- `git diff --check` : succès.
- Aucun Cargo, rustc, build ou test exécuté.

## Raccord attendu

C/E doivent persister `decision:null` dans le metadata v2 initial, puis le seul
objet `LineageDecisionV4` validé dans `chosen_metadata` de
`choose_auto_admission_at`. Aucun raccord natif/e2e C n'a été modifié par ce lot.
