# Rapport lint et format T3 - session 149 - Haiku 5.5 (r1)

## Résumé

- Le lint échoue : 14 erreurs, dont 13 nouvelles introduites par 149. Une erreur existe déjà dans le checkout racine.
- Le lint échoue aussi sur 924 avertissements, contre 901 en racine (+23 liés à 149).
- Le format échoue : 37 fichiers sur 48 modifiés ou nouveaux par 149 ne respectent pas `vp fmt`. Deux échecs existent déjà en racine.
- Cause du blocage initial : le plugin de lint `oxlint-plugin-t3code` ne trouvait ni `@oxlint/plugins` ni `effect` dans le worktree. J'ai ajouté deux liens ciblés. Aucune dépendance n'a été installée ni copiée.
- Aucun fichier source, test, manifeste ou lock n'a été modifié. Aucun format ni fix automatique n'a été lancé.
- Aucune commande Git, Cargo, modèle réel, install ou redémarrage.

## Environnement

- Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`
- Racine (baseline) : `/Users/moi/11.Repositories/t3code-local`
- Toolchain Node : `/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin` (Node v24.13.1, vérifié par `verification.json`)
- Commande : `export PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:$PATH`
- `vp` n'est pas dans le PATH. J'ai utilisé `./node_modules/.bin/vp` (lien vers la racine).
- Dossier des logs (hors dépôt) : `/Users/moi/.cache/t3-lint149-PaLNaU`

## Liens ajoutés (réversibles)

Ces liens existent seulement dans le worktree. Ils pointent vers le store pnpm de la racine. Ils sont couverts par `node_modules` dans `.gitignore`.

| Lien | Cible |
|---|---|
| `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/oxlint-plugin-t3code/node_modules/@oxlint/plugins` | `/Users/moi/11.Repositories/t3code-local/node_modules/.pnpm/@oxlint+plugins@1.68.0/node_modules/@oxlint/plugins` |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/oxlint-plugin-t3code/node_modules/effect` | `/Users/moi/11.Repositories/t3code-local/node_modules/.pnpm/effect@4.0.1_patch_hash=a33cba07c41f32374c2aaa86ea4a84d3ab9a872a78c94b65e929b19f8361856f/node_modules/effect` |

Justification : la racine a les mêmes liens dans `oxlint-plugin-t3code/node_modules`. Le `package.json` du plugin demande `@oxlint/plugins` `^1.63.0`, et 1.68.0 le satisfait. Les règles du plugin importent seulement `@oxlint/plugins` et `effect`, vérifié par lecture des fichiers hors tests.

Pour retirer les liens : `rm` sur ces deux chemins de lien, sans suivre la cible.

## Commandes et résultats

| # | Commande (depuis le dossier cible) | RC | Log |
|---|---|---|---|
| 1 | `./node_modules/.bin/vp lint --report-unused-disable-directives` (WT, avant liens) | 1 | `lint-wt.log` : `ERR_MODULE_NOT_FOUND` pour `effect` |
| 2 | Idem, WT, après les deux liens | 1 | `lint-wt-2.log` |
| 3 | Idem, racine (baseline) | 1 | `lint-root-baseline.log` |
| 4 | `./node_modules/.bin/vp fmt --check <48 fichiers 149>` (WT) | 1 | `fmt-check-149.log` |
| 5 | Idem sur les 29 fichiers 149 qui existent en racine | 1 | `fmt-check-root-baseline.log` |

Liste des 48 fichiers 149 : `/Users/moi/.cache/t3-lint149-PaLNaU/files149.txt`. Elle reprend le `git status` lu au début (fichiers modifiés et non suivis, `apps/` et `packages/`).

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

Aucun test unitaire relancé. Les tsc baseline (16 server et 10 web) ne sont pas refaits. Les 104 PASS du rapport `t3-final-boundary-tests-sonnet-r1.md` ne sont pas revérifiés ici.

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
