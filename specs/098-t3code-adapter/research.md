# Recherche 098 — faits et décisions

## Environnement constaté le 2026-09-14

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/research.md#extrait-1).

## Faits adaptateur de bureau (lecture du code, fichier:ligne dans le dépôt adaptateur de bureau)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/research.md#extrait-2).

## Faits du poste (2026-09-14)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/research.md#extrait-3).

## Faits Bridget (sondes locales du 2026-09-14)

- Claude Code transmet `CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_ENTRYPOINT` et `CLAUDECODE` à ses serveurs MCP (observé sur quatre processus `bridget mcp` fils de `claude`). Codex app-server ne transmet aucune variable de fil à ses serveurs MCP (observé sur trois processus fils de `codex`).
- Le serveur MCP Bridget résout son identité par `BRIDGET_AGENT_ID_FILE` + `BRIDGET_AGENT_INSTANCE_ID`, ou par un marqueur `agent-pids/<pid>` d'un ancêtre (`crates/bridget-daemon/src/mcp_identity.rs:116-131, 202-270`). Sans wrapper, `IdentityNotFound`.
- La remise idempotente interactive (`deliver_idempotent_to_interactive`, tracker de reçus par instance) et la commande d'administration embarquée de la 096 (`crates/bridget-daemon/src/federate.rs`, 96 lignes) sont réutilisables telles quelles.

## Sonde T001 sur l'application réelle (2026-09-14 09:28–09:31, adaptateur de bureau Code 0.0.40, CLI `adaptateur de bureau` 0.0.40 installé par npm)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/research.md#extrait-4).

## Sonde T013 sur l'application réelle (recette, 0.0.41-nightly, 2026-09-14)

Trois écarts avec la sonde T001, tous corrigés dans le code et couverts par un test :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/research.md#extrait-5).

Autres faits confirmés en conditions réelles : `thread.create` refuse `runtimeMode: "local"` et `interactionMode: "chat"` (littéraux `approval-required|auto-accept-edits|auto|full-access` et `default|plan`) ; les quatre fils du poste sont apparus dans `bridget who` sous leur titre avec le bon type ; un message remis a produit un tour puis une réponse liée qui a clos la demande.

## Décisions (révision 2 après contre-revue adverse)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/research.md#extrait-6).

## Alternatives écartées

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/research.md#extrait-7).

## Risques nommés (révision 2)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/research.md#extrait-8).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/research.md#extrait-9).
