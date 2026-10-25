# Plan 134 — Noms humains dans les messages Bridget

Statut : Correctif livré, recette de production vérifiée.
Branche initiale : session-134-noms-humains-messages.
Branche de reprise : fix-134-profils-satures.
Spec : specs/134-noms-humains-messages/spec.md.

## Contexte technique

### Correctif de saturation — 2026-10-05

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/plan.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/plan.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/plan.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/plan.md#extrait-4).

Aucune dépendance, table, migration globale, socket ou boucle résidente nouvelle
n’est nécessaire.

La branche part de `main` après l’intégration de la SPEC-133. Le champ de
provenance déléguée et son rendu existent donc avant les changements de la
SPEC-134.

## Constitution et périmètre

- PASS : session et worktree isolés.
- PASS : l’UUID reste l’unique identité de sécurité et de routage.
- PASS : la solution étend deux fonctions existantes et ne crée aucun service.
- PASS : le repli historique reste disponible si le nom manque.
- PASS : la réparation est transactionnelle, idempotente et progressive.
- PASS : aucun contenu libre du message n’entre dans le profil ou les logs.
- PASS Article XVIII : le rendu est O(1). La réparation est O(n), avec n égal au
  nombre d’identifiants fournis à l’enregistrement.
- PASS Articles XIX/XX : aucune abstraction ni dépendance nouvelle. Les tests
  décrivent les invariants sans contexte caché.

## Réutilisation de l’existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/plan.md#extrait-5).

## Modifications prévues

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/plan.md#extrait-6).

## Contrat de présentation

Le format visible est défini dans
`specs/134-noms-humains-messages/contracts/sender-label.md`.

Le champ `from_display_name` reste optionnel dans le message. Il n’est jamais
consulté pour une décision d’autorisation ou de routage. Aucun changement du
protocole persistant n’est nécessaire.

## Stratégie de tests

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/plan.md#extrait-7).

## Déploiement et retour arrière

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/plan.md#extrait-8).

## Ordre d’exécution

1. Verrouiller le contrat et les régressions par des tests rouges.
2. Corriger le libellé central et le rendu des lots.
3. Réparer l’invariant identité-profil à l’enregistrement.
4. Exécuter les validations ciblées et globales.
5. Faire une contre-revue adverse, converger et auditer.
6. Fusionner, construire, déployer, vérifier, pousser et nettoyer.
