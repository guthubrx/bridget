# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-lineage-contract-tests-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-lineage-contract-tests-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-10. Sous-agent test dédié. Périmètre : premier incrément T3
contrats/RPC/scopes Lineage 149. Aucun edit de production, aucun commit,
aucun cochage de tâche, aucun restart, aucune config de production modifiée.

<a id="extrait-2"></a>

## Extrait 2

| Fichier (worktree T3) | État au test | Contenu testé |
|---|---|---|
| `packages/contracts/src/bridgetLineage.ts` | nouveau, 09:29 | Schémas fermés, UUID, séquence sûre, marqueur, bornes, refus, mapping présentation |
| `packages/contracts/src/rpc.ts` | modifié 149 | 4 méthodes `bridget.lineage.*`, union d'erreurs, groupe WS |
| `packages/contracts/src/clientRpcPermissions.ts` | modifié 09:16 | `bridgetLineageCancel` → `AuthOrchestrationOperateScope` seul |
| `packages/contracts/src/index.ts` | modifié 149 | Export `bridgetLineage.ts` (vérifié par le typecheck package) |
| `apps/server/src/auth/RpcAuthorization.ts` | modifié 149 | Lecture seule du diff : scopes serveur (§6) |

<a id="extrait-3"></a>

## Extrait 3

Fichier de test créé (ownership exclusif du sous-agent) :
`packages/contracts/src/bridgetLineage149.test.ts`. Aucun autre fichier
modifié dans le worktree T3. Le fichier `packages/contracts/src/bridgetPermissions.ts`
(09:32:20, t3permissions149) n'a pas été lu comme base d'oracle et pas touché.

<a id="extrait-4"></a>

## Extrait 4

- Worktree T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`
  (chemin réel `/Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/...`,
  volume 8TB2 à 204 Gio libres ; disque interne ~14 Gio libres, inchangé).
- **Aucun `node_modules` n'existait dans le worktree.** Aucun `pnpm install`
  lancé (interdit par la mission, risque de duplication). Solution : symlinks
  vers les node_modules existants du checkout principal
  `/Users/moi/11.Repositories/t3code-local/node_modules` — 1 lien racine +
  389 liens par package recréés en miroir de la structure pnpm (mêmes cibles
  relatives) + 2 liens reconstruits (`effect`, `@effect/vitest`) dans
  `packages/contracts/node_modules`. Empreinte : des dizaines de Ko de liens,
  zéro duplication de volume, zéro écriture dans le store pnpm.
- **Signal pour t3149 et t3permissions149** : leurs worktrees n'ont pas de
  node_modules. Toute exécution de test exige soit le même montage de liens,
  soit un `pnpm install` (coût disque à arbitrer avant). À décider par le
  principal avant les lots serveur.
- **Cache mission** : `/Users/moi/.cache/t3-final148.XnkCUZ` ne contient aucun
  node_modules (artefacts de build desktop uniquement, 8,1 Gio). Aucun symlink
  vers ce cache n'était présent ni possible pour les deps ; les deps utiles
  sont celles du checkout principal. Rien n'a été supprimé dans ce cache.
- Toolchain : `vp` absent du PATH global, appelé via
  `./node_modules/.bin/vp` (vite-plus 5.0.1). Node v26.9.0. `effect` 4.0.1
  (version patchée du repo, identique au main).
- **Incident corrigé et consigné** : un premier `ln -sfn` avait transformé
  `packages/contracts/node_modules` du worktree en symlink vers le main ; le
  mirroring a alors écrit à travers lui dans le repo principal. Pollution
  exacte : 1 symlink `$MAIN/node_modules/effect` (n'existait pas avant,
  vérifié) + `$MAIN/node_modules/.vite-temp` vide (artefact du run vp). Les
  deux supprimés ; `$MAIN/node_modules/@effect/` préexistant laissé intact
  (contenu vérifié différent de la source et non modifié). Le miroir a ensuite
  été refait proprement dans des dossiers réels du worktree.

<a id="extrait-5"></a>

## Extrait 5

Depuis la racine du worktree T3. Sorties réelles, aucune troncature de compteur.

<a id="extrait-6"></a>

## Extrait 6

Les 2 échecs de E1 venaient de mes hypothèses, pas du code : j'affirmais la
fermeture de structs de **sortie** alors que seuls les **inputs** T3 sont
fermés (`onExcessProperty: "error"`). Les 2 tests ont été réécrits pour figer
le comportement réel documenté, avec commentaire dans le fichier :
