# Preuve US2 - administration non destructive

Date: 2026-08-30

## Scénario exécuté

Le test d'intégration de registre ouvre uniquement un daemon temporaire et un
répertoire temporaire. Il écrit une sentinelle dans la racine initiale, puis
exécute `list`, `status`, `rebind`, `review-project reconcile --confirm` et
`disable` au travers du contrat local négocié.

- `list` rend une seule liaison, sans créer d'identité au démarrage ni pendant
  l'ouverture/migration SQLite;
- `status` rend `active`, génération 1, puis `disabled`;
- `rebind` vers une autre racine fait passer la génération à 2 et son rejeu
  rend l'issue durable exacte;
- la réconciliation explicite sur cette racine garde la génération 2, écrit un
  audit `review_project_reconcile` et son rejeu n'écrit pas de second audit;
- `disable` est idempotent, projette Maicie en `disabled` et n'arrête aucun
  processus dans ce test;
- les quatre audits sont exactement `register`, `rebind`,
  `review_project_reconcile` et `disable`;
- les représentations textuelles des audits ne contiennent aucune racine
  canonique, ni ancienne ni nouvelle;
- le contenu de la sentinelle est identique avant et après les opérations.

## Commandes et résultats

```text
/home/moi/.cargo/bin/cargo test -p bridget-daemon --test project_registration_e2e
1 passed, 0 failed, 0 ignored

/home/moi/.cargo/bin/cargo test -p maicie --bin maicie -- --test-threads=1
25 passed, 0 failed, 0 ignored

/home/moi/.cargo/bin/cargo test -p maicie --lib -- --test-threads=1
76 passed, 0 failed, 1 ignored
```

## Limite de cette preuve

La conservation d'une exécution active sur son ancienne génération est une
propriété qui sera prouvée avec la propagation de `ProjectReference` de US3.
US2 prouve ici que `rebind` et `disable` ne demandent aucun arrêt implicite et
ne modifient aucun fichier du dépôt.
