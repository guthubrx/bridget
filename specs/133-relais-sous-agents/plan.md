# Plan 133 — Relais Bridget pour les sous-agents internes

Statut : Implémenté. Branche : session-133-relais-sous-agents.
Spec : specs/133-relais-sous-agents/spec.md.

## Contexte technique

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/plan.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/plan.md#extrait-2).

## Choix et réutilisation de l'existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/plan.md#extrait-3).

## Fichiers impactés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/plan.md#extrait-4).

## Modèle de menace et confidentialité

Le processus enfant est coopératif et tourne sous le même compte système. Il ne
reçoit pas la preuve privée du parent dans son message. Le serveur MCP réutilise
la preuve locale existante seulement après résolution du marqueur privé. Le modèle
ne prétend pas isoler deux processus hostiles du même compte.

La référence enfant est une empreinte bornée. Aucun identifiant de session natif,
corps de message ou secret n'entre dans les diagnostics. Les chemins et fichiers
sont absolus, privés, bornés, non symboliques et validés avant lecture.

## Stratégie de validation

Écrire les tests de refus avant le code. Tester : rattachement nominal, parent
absent, ambiguïté, PID recyclé, marqueur non privé, symlink, dépassement de taille,
nettoyage, propriété concurrente, outil permis, outil interdit, CLI refusée,
provenance visible, réponse routée au parent et ancien message compatible.
Ajouter le cas causal : marqueur enfant périmé avec marqueur principal vivant
plus haut donne `identity_not_found`, jamais l'autorité principale. Ajouter aussi
un garde-fou qui inventorie les commandes CLI sensibles et leur résolution.
Reproduire aussi l'incident de production avec un grand nombre de fichiers de
session ouverts. Vérifier que la sortie de `lsof` est vidée pendant l'exécution,
reste bornée et ne dépasse pas le délai de l'inventaire.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/plan.md#extrait-5).

## Gates constitutionnels et Article XIX/XX

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/plan.md#extrait-6).

## Ordre d'exécution

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/plan.md#extrait-7).
