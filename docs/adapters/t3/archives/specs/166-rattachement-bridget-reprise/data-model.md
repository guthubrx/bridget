# Archives techniques de l’adaptateur T3

Document d’origine : [specs/166-rattachement-bridget-reprise/data-model.md](../../../../../../specs/166-rattachement-bridget-reprise/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Élément | Rôle | Changement |
|---|---|---|
| `ThreadId` | Identité T3 du fil | Aucun |
| `ProviderThreadId` | Identité du fil natif Codex | Aucun |
| `McpProviderSessionConfig` | Preuve privée courante du fil | Aucun |
| `bridgetCredentialsByThread` | Preuve effectivement montée par fil | Aucun schéma ; invariant renforcé |
| Fait `provider_turn` | Droits du run actif | Aucun format |
