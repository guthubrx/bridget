# Plan148 — Identité et délégation

## Contexte technique

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/plan.md#extrait-1).

## Diagnostic prouvé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/plan.md#extrait-2).

## Architecture retenue

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/plan.md#extrait-3).

## Gates

Accord session et Codex high acquis. Identité et permissions : refus fermés,
tests négatifs obligatoires. Pas de nouveau framework d'orchestration. Toute
abstraction doit porter une règle d'identité, de rejeu ou de cycle de vie.
Complexité linéaire sur les inventaires bornés, lookup par identifiant pour les tâches.
Chaque incrément comporte test, auto-revue et commit. Pas de redémarrage réel.

## Ordre

US1 puis US3, US2, US4 et US5 ; US6 accompagne chaque incrément.
L'identité est validée avant toute délégation réelle. Les échecs de test sont
résolus avant progression. Recette de fournisseur réel isolée après validation
des contrats ; les limites sont consignées explicitement.

## Livraison

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/plan.md#extrait-4).
