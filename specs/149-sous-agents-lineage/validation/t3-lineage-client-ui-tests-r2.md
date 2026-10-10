# Validation r2 — tests UI client T3 Lineage 149 (après corrections Sol)

Date : 2026-10-10. Sous-agent test r2, ownership exclusif des 4 fichiers de test
r1. Objet : vérifier les 4 corrections ciblées de Sol, relancer les 22 tests et
les typechecks, clore les 4 groupes d'erreurs du r1. Aucun edit de production,
aucun commit, aucun cochage de tâche, aucun nettoyage d'artefact partagé.

Résultat global : **22/22 PASS** sur 4 fichiers de test (compteurs parsés sur
la sortie, pas sur l'exit code). Typechecks : **contracts 0 erreur,
client-runtime 0 erreur, mobile 0 erreur, web exit 1** avec 10 erreurs toutes
baseline préexistante 148 (preuve fraîche sur base propre 33f6d04, §4).
**Les 4 groupes d'erreurs du r1 sont fermés.** Un renforcement d'oracle
« UUID frais par action » a été ajouté dans 3 fichiers de test (§5).

---

## 1. Corrections Sol vérifiées (relues puis prouvées)

| Correction | Reliée | Preuve r2 |
|---|---|---|
| `packages/contracts/src/bridgetPermissions.ts` l.86-107 : `Schema.declareConstructor` avec décodeur wrapper `(input) => decode(input)` et `SchemaParser.decodeUnknownEffect(codec, { onExcessProperty: "error" })`, sur `BridgetPermissions` et `BridgetSessionIdentity` | ✓ | `tsc --noEmit` contracts : **exit 0, 0 erreur** (les TS2322 l.87/100 ont disparu). Le contrat ne bloque plus les 3 packages consommateurs |
| `packages/client-runtime/src/state/threadExecution.ts` l.141-142 : `DateTime.formatIso(DateTime.makeUnsafe(unixSeconds * 1000))` pour `started_at`/`completed_at`, plus de `new Date()` | ✓ | Typecheck client-runtime : **exit 0**. Comportement prouvé par le test 7 « convertit les secondes Unix en ISO » : `1700000000` → `"2023-11-14T22:13:20.000Z"`, absences à `null` (oracle inchangé) |
| `apps/web/src/components/BridgetTaskJournal.tsx` l.160 : `requestId: randomUUID()` du helper repo `apps/web/src/lib/utils.ts:36` (UUID v4 maison, idiome `newProjectId`/`newThreadId`), appelé dans le handler `stop` — généré neuf par action | ✓ | Test renforcé (§5) : 2 clics « Arrêter » → 2 requestId UUID valides **et différents**. Aucun module Crypto Effect réclamé : le helper repo est correct et préexistant |
| `apps/web/src/components/chat/ThreadRelationshipsControl.tsx` l.329 : même helper, même idiome, dans le handler d'arrêt natif | ✓ | Test renforcé : stop imbriqué → requestId UUID valide (plus `expect.any(String)`) et ≠ celui du premier stop. Reader F1/F2 et C01/C02 non concernés par mes fichiers (lot serveur) |

Mobile : le composant passe par `apps/mobile/src/lib/uuid.ts` (`uuidv4` →
`expo-crypto`), appelé dans le handler (l.111). Prouvé par compteur (§5).

## 2. Commandes exactes et compteurs

Environnement identique au r1 : Node 24 `/Users/moi/.cache/t3-toolchains/148/`,
deps du WT via symlinks préexistants, aucun install, aucune duplication
`node_modules`. Runs de validation exécutée ; grande revue T040 reste future.

Run entrant (avant mes édits, preuve de l'état reçu) :

```
cd /Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage
./node_modules/.bin/vp test run \
  packages/client-runtime/src/state/orchestration.bridget149.test.ts \
  apps/web/src/components/BridgetTaskJournal149.test.tsx \
  apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx \
  apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx
```

Sortie : `Test Files 4 passed (4)` / `Tests 22 passed (22)` / 2.20s.

Runs par fichier en `--reporter=verbose` (décomptes exacts, comptés sur les
ticks `✓`/`×` du stdout, jamais sur `$?`) :

| Fichier | Tests | Verts | Rouges |
|---|---|---|---|
| `orchestration.bridget149.test.ts` | 9 passed (9) | 9 | 0 |
| `BridgetTaskJournal149.test.tsx` (web) | 7 passed (7) | 7 | 0 |
| `ThreadRelationshipsControl.bridget149.test.tsx` | 4 passed (4) | 4 | 0 |
| `BridgetTaskJournal149.test.tsx` (mobile) | 2 passed (2) | 2 | 0 |

Run final (après renforcement oracle §5) : **`Test Files 4 passed (4)` /
`Tests 22 passed (22)` / 1.97s**, aucun `FAIL`, aucun `×`.

⚠️ Exit code vp non fiable (constat r1 confirmé) : sur le run intermédiaire,
`exit=1` reflétait bien l'échec, mais la règle reste de parser les compteurs
texte. En CI : grepper `Test Files`/`Tests`.

## 3. Typechecks r2 — closure des 4 groupes d'erreurs

Commande par package : `cd <pkg> && ../../node_modules/.bin/tsc --noEmit`.

| Package | Exit | Erreurs | Détail |
|---|---|---|---|
| `packages/contracts` | 0 | **0** | Groupe 1 fermé : TS2322 l.87/100 disparues |
| `packages/client-runtime` | 0 | **0** | Groupe 2 fermé : TS377068 l.141-142 disparues (r1 : 4 erreurs) |
| `apps/web` | 1 | 10 | **Toutes baseline 148** (§4). Groupes 3 et 4 fermés : plus aucune erreur dans `BridgetTaskJournal.tsx` ni `ThreadRelationshipsControl.tsx` (r1 : 14 erreurs) |
| `apps/mobile` | 0 | **0** | Rien depuis le r1 (r1 : 2 erreurs = transit contracts) |

Les 4 groupes d'erreurs du r1 sont donc clos sans aucun edit de production de
ma part — tout vient des corrections Sol, relues ligne à ligne avant run.

## 4. Preuve fraîche baseline (pas régression 149)

Les 10 erreurs web restantes, toutes dans 2 fichiers hors lineage :

```
MessagesTimeline.logic.test.ts(197,33) TS2345   (×7 : 197, 201, 203, 206, 211, 217, 223)
MessagesTimeline.test.tsx(948,67) TS2322        (×3 : 948, 996, 1031)
```

Ces erreurs mentionnent `structuredPayload`/`subagent`, et `orchestrationV2.ts`
est touché par le diff 149 — la preuve r1 ne suffisait plus telle quelle.
Reproductibilité fraîche sur le checkout principal **propre à 33f6d04**
(`git status` vide, HEAD vérifié) : `tsc --noEmit` depuis `apps/web` produit
**exactement les 10 mêmes erreurs, mêmes fichiers, mêmes lignes, mêmes codes**.
Diff 149 ⇒ 0 erreur TypeScript nouvelle. Aucun patch de ma part (ownership
autre, baseline étrangère).

## 5. Renforcement d'oracle r2 (mes 4 fichiers seulement)

Motif : le brief r2 demande de vérifier « newUUID / généré neuf par action ».
L'oracle r1 prouvait le format UUID mais pas la fraîcheur par action. Aucune
assertion r1 abaissée ; uniquement des preuves ajoutées :

- **Web journal** : constante `UUID_RE` + helper `requestIdOf` (la regex r1 est
  réutilisée telle quelle) ; après le second clic « Arrêter » (déjà présent dans
  le scénario r1) : `cancelCalls.length === 2`, second requestId UUID valide et
  `!==` premier.
- **Relationships** : `expect.any(String)` du stop imbriqué remplacé par
  `expect.stringMatching(UUID_RE)` (renforcement, pas abaissement) ; capture des
  deux requestId réels via `state.cancelNative.mock.calls[i][0]` et assertion
  d'unicité.
- **Mobile** : mock `expo-crypto` passé à un compteur `vi.hoisted`
  (`mobile-test-request-${++count}`) ; assertion `requestId === "mobile-test-request-1"`
  et, en fin de scénario, `count === 1` — l'UUID n'est produit que par l'action
  d'arrêt, jamais par les arrêts fermés (indisponible, terminale, sans scope).
- **State (9 tests)** : aucun changement — l'oracle de conversion ISO est déjà
  exact et la couche ne manipule pas d'UUID.

Une itération de correction sur mes propres ajouts : première version du
renforcement relationships a lu `mock.calls[0]` (tuple) au lieu de
`mock.calls[0][0]` (1er argument) → `TypeError` sur `requestId`, corrigé en une
passe. Ce n'est pas un échec de production ; le comportement testé n'a jamais
varié (22/22 avant, 22/22 après).

## 6. Limites

- **vp exit code** : fiabilité non garantie, compteurs texte only (§2).
- **Baseline MessagesTimeline ×10** : reste ouverte, hors lineage, prouvée
  préexistante ; à repro au lot 148, pas à Sol ni au diff 149.
- **Mobile simulé** (inchangé r1) : jsdom + mock react-native ; le compteur
  prouve l'appel `uuidv4()` par action, pas la randomness d'`expo-crypto`.
- **Artefacts** : `.vite-temp` et caches vite laissés en l'état (consigne :
  aucun nettoyage pendant les runs parallèles des autres lots).
- **Périmètre git** : mes 4 fichiers de test restent non suivis, aucune
  modification de production de ma part, aucun commit, aucun cochage tasks.md,
  aucune config réelle touchée.
