# Checklist qualité — Spécification 025

**Créée** : 2026-08-25
**Feature** : `specs/025-carte-criticite-regime/spec.md`

## Qualité du contenu

- [x] Le besoin et la valeur sont décrits avant l’implémentation.
- [x] Les décisions structurantes sont séparées des détails techniques du plan.
- [x] Toutes les sections obligatoires sont remplies.
- [x] Aucun marqueur de clarification ne subsiste.

## Complétude des exigences

- [x] Les dépendances 021, v18, 026/v19 et 023 sont explicites.
- [x] Les trois voies F38 et le germe F38-b sont testables.
- [x] L’angle de confidentialité est tranché et borné.
- [x] Les homonymes et citations sans chemin ont un comportement exact.
- [x] La décision du référent et ce qu’il perd sont explicités.
- [x] Aucun champ libre obligatoire n’existe.
- [x] Tout refus métier est durable et comptable.
- [x] La fermeture des écarts cite un fait exact, jamais un silence.
- [x] Le régime propre du mécanisme est fixe et non auto-calculé.
- [x] L’élection des relecteurs est hors périmètre et échoue explicitement.
- [x] La migration v20 ne peut pas certifier un DDL antérieur absent.

## Préparation à l’implémentation

- [x] Les scénarios principaux et limites possèdent un oracle indépendant.
- [x] Les critères de succès sont mesurables.
- [x] Le modèle de données et le contrat sont définis.
- [x] La stratégie de mutants distingue assertion et mise en place.
- [x] La confidentialité des journaux possède une sentinelle octet par octet.
