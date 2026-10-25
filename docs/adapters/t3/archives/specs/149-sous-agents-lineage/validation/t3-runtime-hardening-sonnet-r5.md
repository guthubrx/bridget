# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Testeur : Sonnet 5.5 normal, effort high. Date : 2026-10-10.
Base T3 : `33f6d04e116430bf7f0011902d6af3868163f2d1`. WT : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

<a id="extrait-2"></a>

## Extrait 2

Aucun finding actif. Aucune édition de production ni de test unitaire T3. Aucun Cargo, Git, installation, redémarrage, modèle réel. Aucune case cochée (T036/T037/T038/T043-T045 non touchées).

<a id="extrait-3"></a>

## Extrait 3

| Volet | PASS | FAIL | Note |
|---|---|---|---|
| Permissions et consommateurs (6 fichiers) | 128 | 0 | `bridgetPermissions149` 30, `mcpSession149` 14, `OrchestratorMcpService.bridgetPermissions149` 9, `BridgetPermissions149` (G3) 21, `Orchestrator.bridget149` (G5) 48, `BridgetSession` 6 |
| UI/client (4 fichiers) | 24 | 0 | web journal 9, ThreadRelationshipsControl 4, client-runtime 9, mobile journal 2 |
| `tsc --noEmit` serveur | - | - | 16 erreurs, toutes baseline : 12 `BridgetRustInterop.testkit.ts`, 2 `BridgetRustInterop.test.ts`, 1 `BridgetRustInteropObserver.test.ts`, 1 `CodexMcp.ts` (TS377030, hors périmètre). 0 nouvelle. |
| `tsc --noEmit` web | - | - | 10 erreurs, toutes baseline (`MessagesTimeline.logic.test.ts` 7, `MessagesTimeline.test.tsx` 3). 0 nouvelle. |
| Bornes C0/C1 sur le schéma réel (`ui-recipes/boundary149.ts`) | 65 536 unités UTF-16 | 0 écart | voir ci-dessous |
| Navigateur preview T3 | 13 mesures | 0 | voir ci-dessous |
| SQL privé en fin de recette | runs 0, run_attempts 0, provider_turns 0, provider_sessions 0, provider_threads 0, bindings 0, runtime_requests 0, effect_outbox 0, workflows 0, messages 0 | | `projection_threads` = 6 (hôte + 4 tâches virtuelles + 1 fil auto du projet `ui-recipes`) |

<a id="extrait-4"></a>

## Extrait 4

| Couche | Statut |
|---|---|
| Serveur T3 du WT (`apps/server/src/bin.ts`, Node 24.13.1, `127.0.0.1:14777`), auth, RPC, `BridgetLineage`, projection SQL privée | RÉEL |
| UI web du WT (Vite+, `localhost:15737`), navigateur preview T3 (onglet à moi, profil incognito, fermé) | RÉEL |
| Daemon Bridget (`bridget_fixture.mjs` sur `store.json` privé) | SIMULÉ, nommé |
| Données de tâches et journaux (T1 36 lignes, T2 31, T3 22) | SYNTHÉTIQUES |

<a id="extrait-5"></a>

## Extrait 5

| Cas | Écart dernière ligne / barre | Barre | `paddingBottom` | Gouttière G / D (contenu) | Composeur | « Arrêter » atteignable |
|---|---|---|---|---|---|---|
| Racine T1, 1280x800, carte « Thread details » ouverte | 12 px | 60 | 72 px | 54 / 330 (carte de 276 px à droite, voulu) | aucun | oui |
| Imbriqué T2 titre long, 1280, carte ouverte | 12 px | 60 | 72 px | 54 / 330 | aucun | oui |
| T2, 1280, carte fermée | 12 px | 60 | 72 px | 144 / 144 symétrique | aucun | oui |
| T2, 480x800 | 12 px | 54 | 66 px | 18 / 18 symétrique | aucun | oui |
| T2, 375x800 | 12 px | 54 | 66 px | 18 / 18 symétrique | aucun | oui |
| T1, 1280 -> 375 -> 1280 sur la même page (sans rechargement) | 12 px | 54 puis 60 | 66 puis 72 px (suit la barre) | 18 / 18 puis 54 / 330 | aucun | oui |
| T1, rechargement et retour d'historique (remontage complet) | 12 px | 60 | 72 px (pas doublé) | 54 / 330 | aucun | oui |
| T3 terminale, journal 22 lignes, résultat lu | 12 px | 60 | 72 px | - | aucun | absent (tâche terminée, voulu) |

<a id="extrait-6"></a>

## Extrait 6

Je n'ai pas rejoué 1024 et 768, ni le mobile natif ou iOS. Détail du résultat T3 : le texte du résultat est en tête de page (au-dessus de la liste) ; en bas, le journal reste au-dessus de la barre à 12 px.

<a id="extrait-7"></a>

## Extrait 7

- Ma commande `auth session issue ... | tail -1` rendait un jeton vide : la sortie du CLI se termine par une ligne vide. Corrigé dans ma procédure (filtre `grep`), pas un défaut du produit.
- Les tests actuels ne couvrent que `\u0000` pour les caractères de contrôle ; la borne `0x7f`-`0x9f` repose sur mon script jetable. Si le principal veut un garde permanent, un test de bornes sur `0x1f`, `0x7f`, `0x9f` et `0xa0` suffirait (non demandé).
- Limites de la fixture inchangées : elle ignore `--t3-thread` (S149-19 non prouvé) et refuse le namespace `mcp`.

<a id="extrait-8"></a>

## Extrait 8

```
export C=/Users/moi/.cache/bridget149-ui.r5 RECIPE149_CACHE=$C RECIPE149_T3_PORT=14777 RECIPE149_WEB_PORT=15737
PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:$PATH
# tests (racine du WT)
./node_modules/.bin/vp test run packages/contracts/src/bridgetPermissions149.test.ts packages/provider-core/src/server/mcpSession149.test.ts apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts apps/server/src/mcp/BridgetSession.test.ts
./node_modules/.bin/vp test run apps/web/src/components/BridgetTaskJournal149.test.tsx apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx packages/client-runtime/src/state/orchestration.bridget149.test.ts apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx
(cd apps/server && ../../node_modules/.bin/tsc --noEmit) ; (cd apps/web && ../../node_modules/.bin/tsc --noEmit)
# recette (depuis ui-recipes/)
node fixture_store_init.mjs $C/store.json $C/proj ; zsh run_recipe_server.sh start
node $WT/apps/server/src/bin.ts auth session issue --ttl 3h --token-only --scope ...   # read, operate, settings:write (sortie vers fichier)
node --no-warnings rpc149.ts setup $C/tok-operate.txt
node scenario_step.mjs $C/store.json tick <T1|T2|T3>   # x34, x30, x20 ; title ... pour le titre long de T2
(cd apps/web && PORT=15737 T3CODE_PORT=14777 vp dev) ; auth pairing create --ttl 20m --base-url http://localhost:15737
node --no-warnings boundary149.ts
```

<a id="extrait-9"></a>

## Extrait 9

Onglet preview fermé (`t3_preview_close`). Arrêt en SIGTERM, un PID à la fois, cwd et ligne de commande vérifiés, pas Firefox : listener Vite 59618 et serveur T3 46865 ; le shell Vite 59584 s'était déjà terminé. Ports 14777 et 15737 libres, 0 processus fixture. Jetons bearer, jeton de recette, jeton d'appairage et pidfiles supprimés. Aucun job différé. Conservés sous `/Users/moi/.cache/bridget149-ui.r5` : DB privée `t3home`, `store.json`, `logs/`.
