# Validation r1 — tests UI client T3 Lineage 149

Date : 2026-10-10. Sous-agent test dédié (lots client). Périmètre : couche
état client-runtime, journal Bridget web, panneau Lineage web, journal Bridget
mobile. Aucun edit de production, aucun commit, aucun cochage de tâche, aucun
restart, aucune config de production modifiée.

Résultat global : **22/22 PASS** sur 4 fichiers de test (consolidé final).
Typechecks ciblés : **0 erreur dans mes 4 fichiers** sur les 3 packages.
Le typecheck reste bloqué par des erreurs d'autres lots, signalées sans patch
(§5). Aucun test n'est coché côté tasks.md.

---

## 1. Périmètre et ownership

Fichiers de test créés (ownership exclusif, aucun autre fichier touché) :

| Fichier (worktree T3 `149-sous-agents-lineage`) | Heure | Tests | Cible |
|---|---|---|---|
| `packages/client-runtime/src/state/orchestration.bridget149.test.ts` | 11:48 | 9 | Dérivation d'état : journal, watch, 9 statuts natifs, runless, détachement |
| `apps/web/src/components/BridgetTaskJournal149.test.tsx` | 10:58 | 7 | Journal web : flux, pagination, gaps, unsubscribe, arrêt |
| `apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx` | 11:51 | 4 | Panneau Lineage : stop natif, fermetures, relation, sidebar |
| `apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx` | 11:51 | 2 | Journal mobile : focus/arrière-plan/afterSeq, résultat paginé, arrêt |

Production couverte (toute lue, aucune modifiée) : `orchestration.ts`,
`threadExecution.ts`, `threadWorkflows.ts` (état) ; `BridgetTaskJournal.tsx`
web et mobile ; `ThreadRelationshipsControl.tsx` ; `Sidebar.logic.ts` ;
`subagentDisplay.ts` ; `ThreadDetailsSection.tsx`.

## 2. Commandes exactes et compteurs

Environnement : Node 24 toolchain `/Users/moi/.cache/t3-toolchains/148/`,
pnpm du checkout principal, deps du WT via symlinks préexistants (aucun
`pnpm install`, aucune duplication `node_modules`).

Run consolidé final (après toutes corrections TS) :

```
cd /Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage
./node_modules/.bin/vp test run \
  packages/client-runtime/src/state/orchestration.bridget149.test.ts \
  apps/web/src/components/BridgetTaskJournal149.test.tsx \
  apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx \
  apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx
```

Sortie : `Test Files 4 passed (4)` / `Tests 22 passed (22)` / Duration 1.87s.
Décomptes par fichier : 9/9, 7/7, 4/4, 2/2.

⚠️ **Exit code vp** : le binaire `vp` renvoie toujours 0, même en échec.
Les compteurs sont lus sur la sortie texte (`Test Files` / `Tests`), jamais
sur `$?`. Pour du CI, grepper la sortie.

Typechecks ciblés (`<app>/../../node_modules/.bin/tsc --noEmit` depuis chaque
package) :

| Package | Exit tsc | Erreurs dans mes fichiers | Erreurs totales |
|---|---|---|---|
| `packages/client-runtime` | 1 | **0** | 4 (autres lots, §5) |
| `apps/web` | 1 | **0** | 14 (autres lots + prod, §5) |
| `apps/mobile` | 1 | **0** | 2 (autres lots, §5) |

## 3. Comportements prouvés (résumé par lot)

**État client-runtime (9)** — pli du journal sans dédupliquer (le seq dupliqué
reste, la fusion est le travail du composant), éviction 4096 avec comptage
perdu, watch relancé 3 fois max sur erreur technique seulement (`store_unavailable`),
les 10 codes de refus d'autorité sans aucune relance, journal jamais relancé
(reprise par `afterSeq` + nouvelle visite), les 9 statuts natifs → 6
présentations sans run ni tour (3 champs publics exactement, `activeTurnId`
jamais inventé), secondes Unix → ISO avec absences à null et priorité tâche
Bridget sur le tour racine, `startedAt` runless seulement pour un travail
Bridget actif, détachement fermé sur un fil Bridget même avec session vivante.

**Journal web (7)** — souscription watch+journal à l'écran actif, pli et rendu,
gap transport affiché, pagination résultat 16 Kio UTF-8 multioctet, recharge
après `snapshot_changed` avec contenu conservé, unsubscribe à la fermeture et
au changement de sélection (reads en vol avortés), arrêt redirigé vers
`bridget-lineage-cancel` natif (jamais `interrupt` T3, jamais navigate),
« Dernier état connu » sans fausse disponibilité ni bouton Arrêter, scope
`canOperate` false → pas d'arrêt.

**Panneau Lineage web (4)** — stop bridget → `cancelNative` avec
`{projectId, threadId racine, taskId, requestId UUID}` (variante imbriquée :
sa propre référence, pas le parent), aucun `startedAt` exigé pour l'origine
`bridget_native` ; fermetures : `available:false`, statut terminal, sous-agent
`provider_native` sans `startedAt` → 0 bouton ; relation affichée des deux
côtés (vrai `<h3>` « Lineage » côté parent, hint « Open parent agent in this
chat » côté enfant, titre exact « Lineage · Bridget indisponible ») ;
sidebar : enfant subagent masquée, parent et fils ordinaires visibles.

**Journal mobile (2)** — écran actif → watch+journal souscrits avec
`afterSeq:0`, arrière-plan → désabonnement + reads avortés, retour → reprise
depuis `afterSeq:8` (nextSeq du dernier pli) ; résultat paginé 16 Kio
(`offset:0, limit:16384` → `offset:16384`), panne `snapshot_changed` en pleine
pagination avec contenu conservé, reprise jusqu'à la fin (bouton « Suite du
résultat » retiré), arrêt = annulation native UUID local, jamais un tour T3 ;
« Dernier état connu » / tâche terminale / `canOperate` false → pas d'arrêt.

## 4. Corrections faites dans mes fichiers (passe typecheck)

Les tests passaient ; ces 7 erreurs TS bloquaient la propreté. Toutes
corrigées dans MES fichiers seulement :

- `orchestration.bridget149.test.ts` : `instanceof BridgetLineageError`
  interdit sur un type Schema (TS377042, règle `effect(instanceOfSchema)`) →
  garde `Schema.is(BridgetLineageError)` en tête de module, idiome repo ;
  accès `.code` post-`Effect.flip` narrow par ce garde (l'union
  EnvironmentAuthorizationError etc. n'a pas de `.code`) ; fixture
  `v2Projection.thread` cast structurel vers le paramètre de
  `isBridgetSubagentThread` (le type du shell modélise `bridgetTaskRef` comme
  `X | undefined` requis, la fixture omet la clé — TS2379
  exactOptionalPropertyTypes).
- `ThreadRelationshipsControl.bridget149.test.tsx` : helper sidebar typé
  `OrchestrationV2AppThreadLineage` (les 3 champs requis :
  `parentThreadId`, `relationshipToParent`, `rootThreadId`) — corrige 3×
  TS2322 + 1× TS2339 (une fois `lineage` typé, le générique
  `filterSidebarV2VisibleThreads<T>` infère `T` avec `id`).
- `BridgetTaskJournal149.test.tsx` mobile : `children?: ReactNode` dans le
  mock Pressable (TS2769) ; type `LineageTarget` reserré
  (`input.context: Record<string, unknown>`) pour lire `.context.afterSeq`
  (TS2571).

Aucun oracle abaissé : les assertions attendues sont inchangées, seuls les
types des fixtures/mocks ont été alignés sur les types de production.

## 5. Erreurs restantes — autres lots, signalées sans patch

**`packages/contracts/src/bridgetPermissions.ts` l.87 et l.100 (TS2322)** —
signature de décodeur incompatible avec la cible (`ParseOptions` vs
`Declaration`). Fichier du lot t3permissions149 (Sol). Bloque le typecheck des
3 packages (contrats transite partout). Repro : `tsc --noEmit` depuis
`packages/client-runtime`, `apps/web` ou `apps/mobile`.

**`packages/client-runtime/src/state/threadExecution.ts` l.141-142
(TS377068, règle `effect(globalDate)`)** — `new Date()` en production 149
(t3149). « Convertit les secondes Unix en ISO » : représentation attendue =
`DateTime` Effect. Repro : `tsc --noEmit` depuis `packages/client-runtime`.

**`apps/web/src/components/BridgetTaskJournal.tsx` l.159 (TS377078,
`effect(cryptoRandomUUID)`)** — `crypto.randomUUID()` en production 149 web ;
le mobile passe par `expo-crypto`, la couche Effect attend le module `Crypto`.
Repro : `tsc --noEmit` depuis `apps/web`.

**`apps/web/src/components/chat/ThreadRelationshipsControl.tsx` l.328
(TS377078, même règle)** — `crypto.randomUUID()` du stop natif (production
149). Repro : idem.

**Préexistant 148** : `apps/web` comptait aussi 13 erreurs hors lineage :
`MessagesTimeline.logic.test.ts` ×7, `MessagesTimeline.test.tsx` ×3 (détail
disponible dans `/tmp/tsc-web-2.log`, non reproduit ici). Non touchées.

**Échec baseline préexistant (à repro à Sol)** :
`apps/web/src/components/chat/ThreadRelationshipsControl.agents.test.tsx`
> « shows readable models and only differing workspace details in agent
tooltips ». Prouvé identique sur le checkout principal PROPRE (33f6d04) avant
le diff 149 : ce n'est pas une régression 149. Non patché (ownership autre).
Repro : `./node_modules/.bin/vp test run
apps/web/src/components/chat/ThreadRelationshipsControl.agents.test.tsx`.

## 6. Découvertes structurelles (utiles pour la recette T040)

- **aria-label stop dupliqué** : le composite `ThreadDetailsControl` et son
  `<button>` hôte portent tous deux `Stop subagent ${title}`. Un sélecteur
  aria naïf compte 2 matchs pour 1 bouton réel. Filtrer `type === "button"`.
- **Graphe Lineage construit depuis les shells** : `deriveThreadRelationshipGraph`
  part des shells du cache, pas de la seule projection. Vue enfant = il faut
  les 2 shells (parent + enfant) dans le cache pour afficher le parent agent.
- **Labels de relation en infobulle/hint** : « Subagent » / « Parent agent »
  n'apparaissent pas en texte permanent, seulement dans le tooltip
  (« Open parent agent in this chat »). Un test de texte plein écran raterait.
- **Mobile : input enveloppé** : journal/watch portent
  `{context: {projectId, threadId, taskId, afterSeq}, visitId}` — pas un
  input plat comme le web.
- **Résultat show exige la clé `task`** : le composant teste `"task" in value`
  ; une fixture sans `task` rend un écran vide sans erreur.
- **Fin de pagination** : `result_next_offset: null` retire le bouton
  « Suite du résultat » (comportement correct, à ne pas prendre pour un bug).
- **`afterSeq` de reprise** = `nextSeq` du dernier pli reçu (pas le dernier
  seq affiché si des événements ont été fusionnés côté composant).

## 7. Limites

- **Mobile simulé** : `react-test-renderer` est absent des deps mobiles
  (aucun install autorisé). Idiome repo appliqué : environnement jsdom +
  `createRoot` + `act` + vi.mock partiel react-native (Pressable mappé sur
  `<button>`). Les 2 tests prouvent l'intégration et la logique d'écran, pas
  le geste natif réel. Aucun nouveau framework ajouté.
- **Typecheck ≠ recette navigateur** : T040 (recette UI complète) reste
  futur. Ces tests prouvent la logique rendue, pas le rendu réel navigateur.
- **vp exit code toujours 0** : les compteurs doivent être extraits de la
  sortie (§2).
- **Artefact nettoyé** : mes runs ont recréé
  `node_modules/.vite-temp` au checkout principal (via le symlink racine
  sanctionné). Supprimé en fin de mission.

## 8. État git

`git status --porcelain` du WT : mes 4 fichiers de test présents en
non-suivis, aucune modification de ma part ailleurs (production et tests
d'autres agents intacts). Aucun commit, aucun cochage tasks.md, aucun restart,
aucune config DB/provider réelle touchée.
