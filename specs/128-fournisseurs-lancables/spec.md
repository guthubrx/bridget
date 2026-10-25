# Spécification 128 - GLM et second compte Codex lançables par Bridget

## Fiche synthèse
Spec: 128-fournisseurs-lancables | Statut: In Progress | Priorité: P1 | Date: 2026-10-02
Branche: session-128-fournisseurs-lancables | Accord de l'utilisateur pour GLM « et tout autre provider ».

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/128-fournisseurs-lancables/spec.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/128-fournisseurs-lancables/spec.md#extrait-2).

## Critères de succès
- **SC-001** : après relance du daemon, `bridget spawn glm` et `bridget spawn codex-pro` sans
  `--posture`, depuis une session sans terminal, sont acceptés en lecture seule, répondent à une
  demande, puis s'arrêtent.
