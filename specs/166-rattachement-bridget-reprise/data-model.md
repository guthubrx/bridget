# Modèle de données — SPEC-166

## État existant utilisé

| Élément | Rôle | Changement |
|---|---|---|
| `ThreadId` | Identité T3 du fil | Aucun |
| `ProviderThreadId` | Identité du fil natif Codex | Aucun |
| `McpProviderSessionConfig` | Preuve privée courante du fil | Aucun |
| `bridgetCredentialsByThread` | Preuve effectivement montée par fil | Aucun schéma ; invariant renforcé |
| Fait `provider_turn` | Droits du run actif | Aucun format |

## Invariant

Avant `turn/start`, si une preuve privée courante existe, elle doit être identique à la preuve mémorisée comme montée pour ce `ThreadId`.

## Transitions

1. Le fil natif est chargé avec A.
2. La session privée tourne vers B.
3. Le début du tour observe A différent de B.
4. `thread/resume` recharge le même fil avec B.
5. La carte mémorise B.
6. Les droits du run sont publiés pour B.
7. `turn/start` démarre.

Aucune table, migration ou donnée persistante nouvelle n’est requise.
