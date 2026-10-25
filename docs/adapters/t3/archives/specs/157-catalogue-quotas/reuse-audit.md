# Archives techniques de l’adaptateur T3

Document d’origine : [specs/157-catalogue-quotas/reuse-audit.md](../../../../../../specs/157-catalogue-quotas/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | O(1) candidats, O(log n) mémoire, mesure du catalogue |
| `~/.speckit/constitution.md` XIX | minimalisme | un module, pas de table, pas de CLI |
| `~/.speckit/constitution.md` XXII | mesurer avant d'optimiser | T001 et T012 |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart |
| `AGENTS.md` (checkpoints 149) | checkpoints historiques ; autorisation actuelle152–160 | livraison et activation Bridget, T3 inchangé |
| contrat 148 | `model_unavailable` sans substitution | `model_unavailable` jamais remplacé |

<a id="extrait-2"></a>

## Extrait 2

| Requete | Portee | Resultat |
|---|---|---|
| `rg rate_limits` | `crates/` | `daemon.rs:566`, `:6641`, `protocol.rs:4686`, `codex_app_server.rs:2643` |
| `rg claude-statusline` | `crates/` | `cli.rs:4859-5126`, statut fixe `unknown` |
| `rg permission_source_unavailable` | `crates/` | `native_permissions.rs:228`, `:254` : cause du constat |
| `rg launch_context` | `crates/` | optionnel côté Claude (`native_permissions.rs:75` transport) |
| `rg command_exists` | `lifecycle.rs` | `:218-226`, `:400`, `:542` |
| `rg credential\|unauthenticated` | `crates/bridget-daemon/src` | aucune sonde d'authentification fournisseur |
| `rg "fn which"` | `cli.rs` | `cli.rs:642`, usage hors délégation, non réutilisé |
| `git show origin/main:.../OrchestratorMcpService.ts` | T3 | `providerConstraints` partagé (`:264-280`, `:1146`) |
| `git show origin/main:.../UsageLimitRecoveryWorker.ts` | T3 | gardé pour la 160 (identité `thread:run:reset`) |
