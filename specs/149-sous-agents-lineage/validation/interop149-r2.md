# Interop réseau natif 149 - T036 - ronde r2

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production, aucun modèle.

## Verdict : APPROVE sur le périmètre exécuté (65/65 PASS)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-2).

## Binaire et sources testés

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-3).

## Couches : réel ou simulé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-4).

Pas de `bridget_fixture.mjs`, pas de faux Orchestrator HTTP.

## Harnais : précision des ACK (obligatoire, conservée)

`delivery_generation` est un u64 de 19 chiffres. `JSON.parse` l'arrondirait. Le harnais (`fx.mjs`, classe `JsonLines` et `Peer`) relit les chiffres exacts dans le texte de la ligne et écrit l'ACK à la main. Preuve : R9.1b (aucune remise déjà accusée n'est rejouée après relance) passe, et aucune remise ne se répète dans les 3 recettes.

## Incident de recette à connaître (premier essai de `unified149.ts`)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-5).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-6).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-7).

## Résultats détaillés

### T036 partie 1 - hôte MCP adaptateur de bureau + daemon natif (43)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-8).

### T036 partie 2 - serveur adaptateur de bureau complet + daemon natif (15)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-9).

### T036 partie 3 - instance adaptateur de bureau unique (7)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-10).

## Ce que prouve l'instance unique (levée de la réserve r1)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-11).

## Limites et non prouvé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r2.md#extrait-12).
