# Rapport lint et format adaptateur de bureau - session 149 - Haiku 5.5 (r1)

## Résumé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r1.md#extrait-1).

## Environnement

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r1.md#extrait-2).

## Liens ajoutés (réversibles)

Ces liens existent seulement dans le worktree. Ils pointent vers le store pnpm de la racine. Ils sont couverts par `node_modules` dans `.gitignore`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r1.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r1.md#extrait-4).

Pour retirer les liens : `rm` sur ces deux chemins de lien, sans suivre la cible.

## Commandes et résultats

| # | Commande (depuis le dossier cible) | RC | Log |
|---|---|---|---|
| 1 | `./node_modules/.bin/vp lint --report-unused-disable-directives` (WT, avant liens) | 1 | `lint-wt.log` : `ERR_MODULE_NOT_FOUND` pour `effect` |
| 2 | Idem, WT, après les deux liens | 1 | `lint-wt-2.log` |
| 3 | Idem, racine (baseline) | 1 | `lint-root-baseline.log` |
| 4 | `./node_modules/.bin/vp fmt --check <48 fichiers 149>` (WT) | 1 | `fmt-check-149.log` |
| 5 | Idem sur les 29 fichiers 149 qui existent en racine | 1 | `fmt-check-root-baseline.log` |

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r1.md#extrait-5).

## Lint : erreurs

Erreurs du WT (14) :

| Fichier | Règle | Nombre | Statut |
|---|---|---|---|
| `apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts` | `namespace-node-imports` (2, 3, 4, 5) | 4 | nouveau |
| `apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts` | `no-global-process-runtime` (468, 479, 490) | 3 | nouveau |
| `apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts` | `no-test-in-loop` (1137, 1179, 1219, 1249) | 4 | nouveau |
| `apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts` | `no-test-in-loop` (1159) | 1 | nouveau |
| `apps/web/src/components/BridgetTaskJournal.tsx` | `require-centered-scroll-gutter` (197) | 1 | nouveau |
| `apps/server/src/mcp/BridgetSession.test.ts` | `no-test-in-loop` (93) | 1 | existe déjà en racine (ligne 64) |

Total : 13 nouvelles erreurs, 1 erreur déjà présente.

Avertissements : WT 924, racine 901. Écart de 23, réparti ainsi :
- `no-inline-schema-compile` : `BridgetReader.ts` (6), `OrchestratorMcpService.ts` (1), `ClaudeAdapterV2.ts` (1), `orchestration.ts` (1), `mcpSession.ts` (1)
- `no-unused-vars` : `BridgetReader149.test.ts` (1), `tools.ts` (1), `Orchestrator.bridget149.test.ts` (2), `ProjectionStore.bridget149.test.ts` (1), `ThreadLaunchService.bridget149.test.ts` (2), `BridgetTaskJournal149.test.tsx` (2)
- `no-control-regex` : `ClaudeAdapterV2.ts` (1), `bridgetPermissions.ts` (1)
- `no-array-fill-with-reference-type` : `bridgetLineage149.test.ts` (2)

Les avertissements de `ChatView.tsx`, `ChatComposer.tsx`, et des fichiers mobiles existent déjà en racine. Ils ne sont pas traités ici.

Limite : la racine n'est pas un checkout `main` vérifié (je n'ai pas utilisé Git). Elle sert de baseline à titre indicatif.

## Format : fichiers signalés

- Fichiers 149 en échec : 37 sur 48.
- Dont 2 déjà en échec en racine : `apps/server/src/mcp/BridgetSession.test.ts`, `apps/server/src/mcp/OrchestratorMcpService.ts`.
- Nouveaux échecs de format imputables à 149 : 35 fichiers.
- Les deux fichiers signalés par Sonnet sont confirmés : `apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts` et `apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts`.
- 19 fichiers 149 sont absents de la racine (fichiers nouveaux, non suivis). Ils ne figurent pas dans la comparaison de baseline.

Aucun fichier n'a été reformaté.

## Tests

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r1.md#extrait-6).

## Processus

Aucun processus temporaire à nettoyer. `pgrep` ne retourne aucun lint, fmt ou oxlint en cours. Aucun `kill` lancé.

## Prochain pas (propriétaires des fichiers)

1. Corriger les 13 erreurs introduites, sans modifier les autres fichiers :
   - `no-test-in-loop` : remplacer les boucles `it.effect` ou `it.live` par `it.effect.each` ou `it.live.each` (5 tests 149).
   - `namespace-node-imports` et `no-global-process-runtime` dans `ClaudeAdapterV2.ts` : import en namespace (`NodeFSP`, `NodePath`, `NodeOS`, `NodeCrypto`) et injection de `HostProcessPlatform`.
   - `require-centered-scroll-gutter` dans `BridgetTaskJournal.tsx` : ajouter `scrollbar-gutter-both`.
2. Lancer `./node_modules/.bin/vp fmt` seulement sur les fichiers 149 dont le propriétaire confirme le périmètre. Relire le diff avant commit.
3. Relancer `./node_modules/.bin/vp lint --report-unused-disable-directives` et `./node_modules/.bin/vp fmt --check` sur la liste 149. Le critère est 0 erreur nouvelle.
4. Mettre les deux liens du plugin dans le script de création de worktree. Sans eux, le lint échoue à chaque nouveau worktree.

Décision demandée : quel propriétaire corrige les 13 erreurs et applique le format. Je ne touche pas aux sources.
