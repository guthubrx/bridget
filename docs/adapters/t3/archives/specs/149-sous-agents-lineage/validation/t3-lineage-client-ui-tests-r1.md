# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-lineage-client-ui-tests-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-lineage-client-ui-tests-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Fichier (worktree T3 `149-sous-agents-lineage`) | Heure | Tests | Cible |
|---|---|---|---|
| `packages/client-runtime/src/state/orchestration.bridget149.test.ts` | 11:48 | 9 | Dérivation d'état : journal, watch, 9 statuts natifs, runless, détachement |
| `apps/web/src/components/BridgetTaskJournal149.test.tsx` | 10:58 | 7 | Journal web : flux, pagination, gaps, unsubscribe, arrêt |
| `apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx` | 11:51 | 4 | Panneau Lineage : stop natif, fermetures, relation, sidebar |
| `apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx` | 11:51 | 2 | Journal mobile : focus/arrière-plan/afterSeq, résultat paginé, arrêt |

<a id="extrait-2"></a>

## Extrait 2

Environnement : Node 24 toolchain `/Users/moi/.cache/t3-toolchains/148/`,
pnpm du checkout principal, deps du WT via symlinks préexistants (aucun
`pnpm install`, aucune duplication `node_modules`).

<a id="extrait-3"></a>

## Extrait 3

```
cd /Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage
./node_modules/.bin/vp test run \
  packages/client-runtime/src/state/orchestration.bridget149.test.ts \
  apps/web/src/components/BridgetTaskJournal149.test.tsx \
  apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx \
  apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx
```

<a id="extrait-4"></a>

## Extrait 4

**Journal web (7)** — souscription watch+journal à l'écran actif, pli et rendu,
gap transport affiché, pagination résultat 16 Kio UTF-8 multioctet, recharge
après `snapshot_changed` avec contenu conservé, unsubscribe à la fermeture et
au changement de sélection (reads en vol avortés), arrêt redirigé vers
`bridget-lineage-cancel` natif (jamais `interrupt` T3, jamais navigate),
« Dernier état connu » sans fausse disponibilité ni bouton Arrêter, scope
`canOperate` false → pas d'arrêt.

<a id="extrait-5"></a>

## Extrait 5

**Journal mobile (2)** — écran actif → watch+journal souscrits avec
`afterSeq:0`, arrière-plan → désabonnement + reads avortés, retour → reprise
depuis `afterSeq:8` (nextSeq du dernier pli) ; résultat paginé 16 Kio
(`offset:0, limit:16384` → `offset:16384`), panne `snapshot_changed` en pleine
pagination avec contenu conservé, reprise jusqu'à la fin (bouton « Suite du
résultat » retiré), arrêt = annulation native UUID local, jamais un tour T3 ;
« Dernier état connu » / tâche terminale / `canOperate` false → pas d'arrêt.
