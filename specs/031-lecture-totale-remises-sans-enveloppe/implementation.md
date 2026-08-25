# Preuves d'implémentation

## Références

- Parent empilé : `7ea339b789efe076d2a5d194067e431060f5ec6c`
- Base commune avec `origin/main` au démarrage :
  `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`
- Branche : `session-031-lecture-totale-remises-sans-enveloppe`
- Objectif : `cc47cfc5-a7d4-46f1-afe3-ece4ccbe864b`
- Délégation : `bd7c1d3f-aa61-42b4-84ea-0dbc76e2ca7c`

## Ordre de preuve

1. Compiler la tête parente avec `cargo test --no-run`.
2. Appliquer les deux oracles du jury et mesurer leurs rouges sans correctif.
3. Ajouter un oracle indépendant pour P1, P2 et P3.
4. Implémenter la reprise, puis l'accusé, dans des commits séparés.
5. Tuer séparément les trois mutants causaux et restaurer l'arbre après chacun.
6. Rejouer les mêmes commandes, puis les gates du workspace.

## Mesures

Les comptes avant/après, la plateforme, la charge, les mutations et les limites
de mesure seront consignés ici au fur et à mesure, sans anticiper leur résultat.
