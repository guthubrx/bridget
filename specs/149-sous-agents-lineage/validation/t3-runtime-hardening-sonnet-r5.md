# Session 149 - validation ciblée r5 (après qualité production de Sol)

Testeur : Sonnet 5.5 normal, effort high. Date : 2026-10-10.
Base T3 : `33f6d04e116430bf7f0011902d6af3868163f2d1`. WT : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

## Verdict : APPROVE

Aucun finding actif. Aucune édition de production ni de test unitaire T3. Aucun Cargo, Git, installation, redémarrage, modèle réel. Aucune case cochée (T036/T037/T038/T043-T045 non touchées).

Limite à lire d'abord : le daemon Bridget de la recette navigateur est SIMULÉ (`bridget_fixture.mjs`, magasin JSON privé) et les données sont synthétiques (`[recette149]`). Cette ronde ne valide ni SC001, ni T037, ni le MCP privé, ni T036/T039 natifs. Je n'ai pas utilisé le binaire natif. Je n'ai pas rejoué la matrice réseau 134 tâches, inchangée : les deltas de Sol ne touchent ni `BridgetLineage.ts` ni le serveur de projection.

## Preuves de non-modification

| Mesure | Avant | Après |
|---|---|---|
| sha256 des 32 fichiers de production (liste triée, `shasum`) | `26fba25527ad0247` | `26fba25527ad0247` (identique) |
| sha256 des 16 fichiers de tests149 | identique | identique |
| `git diff HEAD` (fichiers suivis) | `59a54267ce7c1d2a` | `59a54267ce7c1d2a` |

Les 32 fichiers de production : 26 lineage, 6 permissions, au sens de la liste `git diff --name-only HEAD` + fichiers non suivis hors tests. Aucune source n'a bougé pendant ma fenêtre.

Empreintes des 4 fichiers livrés par Sol (16 premiers caractères) :
- `apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts` : `497bcdbeebe0b98d`
- `packages/contracts/src/bridgetPermissions.ts` : `df03fd3fb71dc3e2`
- `packages/provider-core/src/server/mcpSession.ts` : `2e95140365bef3f8`
- `apps/web/src/components/BridgetTaskJournal.tsx` : `08a4796729bf80f1`

## Compteurs exécutés

| Volet | PASS | FAIL | Note |
|---|---|---|---|
| Permissions et consommateurs (6 fichiers) | 128 | 0 | `bridgetPermissions149` 30, `mcpSession149` 14, `OrchestratorMcpService.bridgetPermissions149` 9, `BridgetPermissions149` (G3) 21, `Orchestrator.bridget149` (G5) 48, `BridgetSession` 6 |
| UI/client (4 fichiers) | 24 | 0 | web journal 9, ThreadRelationshipsControl 4, client-runtime 9, mobile journal 2 |
| `tsc --noEmit` serveur | - | - | 16 erreurs, toutes baseline : 12 `BridgetRustInterop.testkit.ts`, 2 `BridgetRustInterop.test.ts`, 1 `BridgetRustInteropObserver.test.ts`, 1 `CodexMcp.ts` (TS377030, hors périmètre). 0 nouvelle. |
| `tsc --noEmit` web | - | - | 10 erreurs, toutes baseline (`MessagesTimeline.logic.test.ts` 7, `MessagesTimeline.test.tsx` 3). 0 nouvelle. |
| Bornes C0/C1 sur le schéma réel (`ui-recipes/boundary149.ts`) | 65 536 unités UTF-16 | 0 écart | voir ci-dessous |
| Navigateur preview T3 | 13 mesures | 0 | voir ci-dessous |
| SQL privé en fin de recette | runs 0, run_attempts 0, provider_turns 0, provider_sessions 0, provider_threads 0, bindings 0, runtime_requests 0, effect_outbox 0, workflows 0, messages 0 | | `projection_threads` = 6 (hôte + 4 tâches virtuelles + 1 fil auto du projet `ui-recipes`) |

Aucun test unitaire n'a été modifié. Les commandes ont donc été lancées telles quelles sur les fichiers déjà formatés/corrigés par Haiku r2 et Sol.

## Audit ciblé des 4 deltas de Sol

1. **Prédicat C0/C1 (`bridgetPermissions.ts` et `ClaudeAdapterV2.ts`).** Les deux fonctions `hasPermissionControlCharacters` sont textuellement identiques (`diff` vide). Elles testent les unités UTF-16 : `code <= 0x1f || (code >= 0x7f && code <= 0x9f)`. Preuve empirique : `boundary149.ts` décode le vrai `BridgetSessionIdentity` (v1) avec chacune des 65 536 unités UTF-16 et compare à l'ancienne règle `/[\u0000-\u001f\u007f-\u009f]/`. Résultat : 0 écart, 65 refusées (32 C0 + 33 de `0x7f` à `0x9f`), 65 471 acceptées. Bornes : `0x1f` refusé, `0x20` accepté, `0x7e` accepté, `0x7f` refusé, `0x9f` refusé, `0xa0` accepté. Une demi-paire de substitution seule et un caractère astral passent, comme avec l'ancienne règle sans drapeau `u`. Les tests existants ne couvrent que `\u0000` : je n'ai pas ajouté de test (propriété du principal), la preuve est portée par mon script jetable.
2. **`HostProcessPlatform` (ClaudeAdapterV2).** Une seule lecture `yield* HostProcessPlatform` (ligne 1004), passée aux trois captures de contexte de lancement (lignes 1052, 1103, 1160). Plus aucun `process.platform` dans le fichier. Le garde `platform !== "darwin"` (ligne 540) utilise donc la plateforme injectée. G3 (21/21) passe avec la plateforme fournie par le test. Limite : aucune exécution non-darwin native n'a été faite.
3. **Codecs compilés au niveau module.** `encodePermissionLaunchJson = Schema.encodeEffect(Schema.fromJsonString(Schema.Unknown))` (ClaudeAdapterV2) et `decodeBridgetPermissions = Schema.decodeUnknownOption(BridgetPermissions)` (mcpSession) : construits une fois, comportement identique, 14/14 et 21/21.
4. **Journal web (`BridgetTaskJournal.tsx`).** La classe `scrollbar-gutter-both` est sur le conteneur défilant ; le `paddingBottom` mesuré (`calc(0.75rem + Npx)`) et les insets de voie sont inchangés ; pas de `z-index`.

## Recette navigateur (couches réelles et simulées)

| Couche | Statut |
|---|---|
| Serveur T3 du WT (`apps/server/src/bin.ts`, Node 24.13.1, `127.0.0.1:14777`), auth, RPC, `BridgetLineage`, projection SQL privée | RÉEL |
| UI web du WT (Vite+, `localhost:15737`), navigateur preview T3 (onglet à moi, profil incognito, fermé) | RÉEL |
| Daemon Bridget (`bridget_fixture.mjs` sur `store.json` privé) | SIMULÉ, nommé |
| Données de tâches et journaux (T1 36 lignes, T2 31, T3 22) | SYNTHÉTIQUES |

Mesures de la dernière ligne, de la barre « lecture seule » et du bouton « Arrêter » au défilement maximal (locator : `role=button[name="Arrêter"]`, `elementFromPoint` au centre du bouton) :

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

Dans tous les cas : pas de débordement horizontal, zéro champ de saisie (`textarea`, `[role=textbox]`, `contenteditable`), aucun `<form>`.
Les écarts de gouttière asymétriques à 1280 avec la carte ouverte viennent du padding de voie (`pr` 324 px contre `pl` 48 px), fixe et voulu ; l'écart supplémentaire de chaque côté de la zone est de 6 px, donc la réserve de barre de défilement est symétrique (`scrollbar-gutter: stable both-edges`). Carte fermée : 144 / 144.

Clic réel sur « Arrêter » de l'imbriqué T2 (titre long, 1280, carte ouverte) : le clic passe, le magasin passe `T2` à `cancelling`, un reçu d'annulation est créé, aucun Run ni ProviderTurn (compteurs SQL ci-dessus).

Je n'ai pas rejoué 1024 et 768, ni le mobile natif ou iOS. Détail du résultat T3 : le texte du résultat est en tête de page (au-dessus de la liste) ; en bas, le journal reste au-dessus de la barre à 12 px.

## Captures (privées, absolues)

Dossier : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r5/`
1. `01-racine-1280-scroll-max-carte-details-ouverte.png`
2. `02-imbrique-titre-long-1280-carte-ouverte.png`
3. `03-imbrique-480-scroll-max-sans-composeur.png`
4. `04-imbrique-375-scroll-max-sans-composeur.png`
5. `05-terminale-resultat-journal-1280-scroll-max.png`

## Observations sans correction demandée

- Ma commande `auth session issue ... | tail -1` rendait un jeton vide : la sortie du CLI se termine par une ligne vide. Corrigé dans ma procédure (filtre `grep`), pas un défaut du produit.
- Les tests actuels ne couvrent que `\u0000` pour les caractères de contrôle ; la borne `0x7f`-`0x9f` repose sur mon script jetable. Si le principal veut un garde permanent, un test de bornes sur `0x1f`, `0x7f`, `0x9f` et `0xa0` suffirait (non demandé).
- Limites de la fixture inchangées : elle ignore `--t3-thread` (S149-19 non prouvé) et refuse le namespace `mcp`.

## Commandes

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

Sorties brutes des tests, de `tsc` et des empreintes : `/Users/moi/.cache/bridget149-tmp/r5/` (`tests-perm.log`, `perm.json`, `ui.json`, `tsc-server.log`, `tsc-web.log`, `prod-before.sha`, `prod-after.sha`).

Nouvel outil (jetable) : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipes/boundary149.ts` (`5e6737c04b988717`).

## Nettoyage

Onglet preview fermé (`t3_preview_close`). Arrêt en SIGTERM, un PID à la fois, cwd et ligne de commande vérifiés, pas Firefox : listener Vite 59618 et serveur T3 46865 ; le shell Vite 59584 s'était déjà terminé. Ports 14777 et 15737 libres, 0 processus fixture. Jetons bearer, jeton de recette, jeton d'appairage et pidfiles supprimés. Aucun job différé. Conservés sous `/Users/moi/.cache/bridget149-ui.r5` : DB privée `t3home`, `store.json`, `logs/`.
