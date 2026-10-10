# Validation r1 — tests de contrats T3 Lineage 149

Date : 2026-10-10. Sous-agent test dédié. Périmètre : premier incrément T3
contrats/RPC/scopes Lineage 149. Aucun edit de production, aucun commit,
aucun cochage de tâche, aucun restart, aucune config de production modifiée.

Résultat global : **22/22 PASS (tests 149)** + **47/47 PASS (régression ciblée
bridget 148)**. Typecheck ciblé : **0 erreur dans les fichiers lineage 149** ;
le typecheck du package reste bloqué par 2 erreurs dans
`src/bridgetPermissions.ts`, fichier d'un autre agent (t3permissions149),
signalé sans patch. Aucun test n'est coché côté tasks.md.

---

## 1. Périmètre testé et ownership

Fichiers de production couverts (tous lus, aucun modifié) :

| Fichier (worktree T3) | État au test | Contenu testé |
|---|---|---|
| `packages/contracts/src/bridgetLineage.ts` | nouveau, 09:29 | Schémas fermés, UUID, séquence sûre, marqueur, bornes, refus, mapping présentation |
| `packages/contracts/src/rpc.ts` | modifié 149 | 4 méthodes `bridget.lineage.*`, union d'erreurs, groupe WS |
| `packages/contracts/src/clientRpcPermissions.ts` | modifié 09:16 | `bridgetLineageCancel` → `AuthOrchestrationOperateScope` seul |
| `packages/contracts/src/index.ts` | modifié 149 | Export `bridgetLineage.ts` (vérifié par le typecheck package) |
| `apps/server/src/auth/RpcAuthorization.ts` | modifié 149 | Lecture seule du diff : scopes serveur (§6) |

Fichier de test créé (ownership exclusif du sous-agent) :
`packages/contracts/src/bridgetLineage149.test.ts`. Aucun autre fichier
modifié dans le worktree T3. Le fichier `packages/contracts/src/bridgetPermissions.ts`
(09:32:20, t3permissions149) n'a pas été lu comme base d'oracle et pas touché.

Sources d'oracle : `specs/149-sous-agents-lineage/contracts/lineage.md` (v1,
2026-10-10), `specs/149-sous-agents-lineage/test-strategy.md` (r2 + amendements
G-L r1 et G-P r1), style du test 148 existant `bridget.test.ts`.

---

## 2. Environnement — prérequis résolus et signalés

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

---

## 3. Exécutions — commandes exactes, compteurs, exit codes

Depuis la racine du worktree T3. Sorties réelles, aucune troncature de compteur.

| # | Commande | Résultat | Exit |
|---|---|---|---|
| E0 | `./node_modules/.bin/vp test run packages/contracts/src/bridget.test.ts` (avant écriture, état 148 inchangé) | 47 passed / 47 | 0 |
| E1 | `./node_modules/.bin/vp test run packages/contracts/src/bridgetLineage149.test.ts` (1re exécution) | 2 failed / 20 passed (22) — hypothèses de fermeture de sorties, voir §5 | 0 |
| E2 | idem après alignement des 2 oracles (§5) | **22 passed / 22** (6 describe) | 0 |
| E3 | `cd packages/contracts && ../../node_modules/.bin/tsc --noEmit` (état initial du test) | 5 erreurs TS2339/TS7053 dans `bridgetLineage149.test.ts` + 2 TS2322 dans `bridgetPermissions.ts` + 2 suggestions TS377112 préexistantes | 1 |
| E4 | idem après corrections du test | **0 erreur dans `bridgetLineage149.test.ts`, `bridgetLineage.ts`, `rpc.ts`, `clientRpcPermissions.ts`, `auth.ts`, `index.ts`** ; restent 2 erreurs TS2322 `bridgetPermissions.ts` l.87 et l.100 | 1 |
| E5 | `./node_modules/.bin/vp test run packages/contracts/src/bridgetLineage149.test.ts packages/contracts/src/bridget.test.ts` (état final) | **2 fichiers, 69 passed / 69** (22 + 47) | 0 |

Durées observées : E2 ≈ 0,94 s total (39 ms de tests) ; E5 ≈ 0,73 s.
Aucun re-run après PASS sur code inchangé (article XXI) : E5 est le run de
consolidation finale demandé pour la preuve combinée, il n'a pas été répété.

Détail E5 (seule sortie consolidée) :

```
 Test Files  2 passed (2)
      Tests  69 passed (69)
```

---

## 4. Oracles couverts — 22 tests, 6 groupes

| Groupe (fichier test) | Ce qui est prouvé | Scénario |
|---|---|---|
| task contract (5) | Entrée native exacte (20 clés exactes après décodage, aucun champ provider exposé) ; UUID non canoniques / 32 chars / majuscules / vides refusés ; séquence hors `0..9007199254740991` refusée (`-1`, `0.5`, `2^53`, `NaN`) ; `UnixSeconds` borné à `8_640_000_000_000` ; statuts natifs fermés aux 9 littéraux du contrat (`running`, `pending`, `completed`, `WORKING` refusés) ; posture fermée (`discovery`/`development`, `inherit` refusé comme effectif) ; protocole fermé à `codex_app_server`/`claude_stream_json` ; bornes `title` 256, `error` 1024, `cwd` 4096, `model` 512, `child_instance_id` 2048 | S149-16, S149-27 |
| task ref marker (3) | Marqueur camelCase accepté racine (`parentTaskId: null`) et imbriqué ; `version` fermé à 1 ; UUID/génération/thread canoniques exigés ; séquence sûre ; statut natif exigé ; champ manquant, type faux, majuscule refusés ; snake_case et champs en excès strippés (limite documentée §5) | S149-16, S149-22 (part schéma TS) |
| snapshot/detail/journal (4) | Snapshot v1 `status:"ok"` obligatoires, séquence sûre jusqu'à `MAX_SAFE_INTEGER` accepté ; page bornée : 4096 entrées acceptées, 4097 refusées ; `next_cursor` ≤ 2048 octets ; détail : `result` ≤ 16384, `result_offset` ≥ 0, `result_total_bytes` ≤ 262 144 ; journal : ≤ 100 événements, `v:1` exigé par événement, `gap` structuré `{from_seq ≥ 0, to_seq, reason}` ou null ; `BridgetLineageView` reconnaît les trois formes | S149-27 |
| inputs fermés (5) | `list` sans `limit`/`cursor`, tout champ en excès refusé (y compris `owner`, `child`, `path`, `posture`, `model`, `provider_session`, `permissions`, `agentId` — le client humain ne peut forger aucune autorité) ; `show` : `taskId` UUID requis, `offset` ≥ 0, `limit` 1..16384 ; action `journal` : `afterSeq` ≥ 0, `limit` 1..100 ; watch : seul le contexte, `taskId`/`follow` refusés ; journal RPC dédié : `taskId` + `afterSeq` seul ; cancel : `taskId` + `requestId` UUID, rien d'autre (`force` refusé) ; contexte non vide et trimmé exigé | S149-16, contrat §Commandes CLI fermées |
| namespace + scopes (2) | `WS_METHODS` expose exactement les 4 clés `bridgetLineage*` aux valeurs `bridget.lineage.read/watch/journal/cancel` ; les clés `bridget*` préexistantes restent `bridgetRead`/`bridgetWatch` (aucune régression 148) ; client : `bridgetLineageCancel` → `AuthOrchestrationOperateScope` (`"orchestration:operate"` vérifié), read/watch/journal **non** gardés côté client (`clientRpcRequiredScopes` → `[]`) | S149-16, S149-25 (part contrats) |
| refus + reçu cancel (2) | Les 16 codes de `BridgetLineageErrorCode` acceptés (les 11 fermés du contrat + 5 codes reader 148), codes inconnus (`forbidden`, `task_not_found`, `internal`, vide) refusés, message expose le code ; reçu cancel v1 : `task_id` UUID + statut natif, `running` et `version:2` refusés | Contrat §Refus fermés, §Annulation humaine |
| présentation (1) | Mapping `bridgetTaskStatus` exhaustif : 9 états natifs → exactement 6 valeurs de présentation (`pending`×3, `running`, `waiting`×2, `completed`, `failed`, `cancelled`) ; aucune valeur hors l'ensemble fermé ; aucune notion de provider dans la sortie | S149-16 (« presentation, not configured provider ») |

---

## 5. Deux oracles alignés en cours d'exécution (E1 → E2)

Les 2 échecs de E1 venaient de mes hypothèses, pas du code : j'affirmais la
fermeture de structs de **sortie** alors que seuls les **inputs** T3 sont
fermés (`onExcessProperty: "error"`). Les 2 tests ont été réécrits pour figer
le comportement réel documenté, avec commentaire dans le fichier :

1. `result_available` hors état éponyme : le schéma transport accepte la
   combinaison `status:"queued"` + `result_available:true`. La règle
   « vrai seulement pour l'état éponyme » appartient au snapshot daemon
   (lineage.md §Snapshot). Test documentatif `not.toThrow` + commentaire.
2. Marqueur avec `root_thread_id`/`provider` en excès : strippé, pas refusé.
   Le refus ferme du marqueur malformé exigé par lineage.md §Projection
   appartient au connecteur Rust (S149-22). Le test vérifie que le décodé
   n'expose aucun champ excédentaire.

**À arbitrer par le principal (aucun patch demandé de ma part)** : si une
défense en profondeur au niveau schéma TS est souhaitée (refus ferme des
excès dans `BridgetTaskRef` et des sorties), c'est un retour vers t3149.
Les refus exigibles actuels (champ manquant, type faux, UUID non canonique,
séquence hors safe, version/statut inconnu) passent tous.

---

## 6. Non couvert par ce lot — limites explicites

1. **`RPC_REQUIRED_SCOPES` serveur (`RpcAuthorization.ts`)** : vérifié par
   lecture du diff — `bridgetLineageRead/Watch/Journal` →
   `AuthOrchestrationReadScope` ; `bridgetLineageCancel` →
   `AuthOrchestrationOperateScope` via le spread `...CLIENT_GUARDED_RPC_SCOPES`
   (une seule source de vérité, testée côté contracts). Aucune assertion
   automatisée : le fichier est dans `apps/server`, hors de mon ownership
   exclusif (`packages/contracts/src/bridgetLineage149.test.ts`).
2. **Partie serveur de S149-16** (fils virtuels `thread:bridget-task:<id>`,
   refus `message.dispatch`/fork/rollback, `activeTurnId` null, journal live,
   annulation réelle) : appartient au lot serveur de t3149
   (`apps/server/src/bridget/BridgetLineage.ts`, non testé ici).
3. **Aucune suite workspace** lancée (code en cours dans le worktree) ; aucun
   test Rust, aucun test serveur, aucun réseau, aucun processus modèle.
4. **Suggestions TS377112** sur `auth.test.ts` (l.18) et `baseSchemas.test.ts`
   (l.46) : préexistantes aux fichiers 148 inchangés, non bloquantes, hors
   périmètre.
5. Les scénarios restants (S149-01..15, 17..31) ne sont pas adressés par ce
   lot ; aucun compteur 148 n'est crédité aux tests 149 (correction 7).

---

## 7. Prérequis bloquants signalés (aucune action de ma part)

| Objet | Détail | Owner attendu |
|---|---|---|
| Typecheck package rouge | `src/bridgetPermissions.ts` l.87 et l.100 : TS2322 — le second paramètre de la fonction de décodage passée à un `declareConstructor` ne peut pas être typé `ParseOptions` (la signature attendue reçoit `self: Declaration`) | t3permissions149 (fichier en cours, 09:32:20) |
| node_modules des worktrees agents | Absents ; montage par symlinks prouvé et décrit §2, ou `pnpm install` à arbitrer (disque) | principal + t3149/t3permissions149 |
| Écarts §5 (fermeture de sorties/marqueur au niveau TS) | Décision de défense en profondeur à prendre | principal → t3149 si retenu |

## 8. Verdict

- Contrats lineage 149 (`bridgetLineage.ts`, surfaces `rpc.ts`,
  `clientRpcPermissions.ts`, export `index.ts`) : **22/22 PASS**, typecheck
  propre sur tous les fichiers 149. Le lot contrats tient le contrat
  `lineage.md` sur les points testés, y compris les bornes et refus fermés.
- Régression ciblée contracts 148 : **47/47 PASS** (aucune régression).
- Le typecheck package complet reste **rouge pour cause de
  `bridgetPermissions.ts`** (t3permissions149) — attente explicite, ne
  pénalise pas ce lot.
- Aucun cochage de tâche, aucun commit, aucune modification de production.
  Rapport produit exclusivement dans ce fichier.
