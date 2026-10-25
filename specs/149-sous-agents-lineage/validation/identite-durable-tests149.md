# Tests du correctif durable d'identité 149

Date : 2026-10-11. Rôle : sous-agent de tests. Aucun fichier de production touché.
Le binaire global 149 n'est pas utilisé comme preuve : tout part des sources des deux worktrees.

## Fichiers dont je suis propriétaire

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-durable-tests149.md#extrait-1).

## Résultats

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-durable-tests149.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-durable-tests149.md#extrait-3).

## Typecheck exact (`tsc --noEmit -p apps/server/tsconfig.json`)

75 lignes `error TS` au total. Aucune n'est dans un fichier modifié, sauf une :

- `src/provider/Drivers/CodexMcp.ts(22,27)` TS377030 : « unknown in the error channel ». Préexistant : la ligne `const config = yield* input.readConfig().pipe(Effect.flatMap(decodeConfig));` est identique dans HEAD (ligne 22). Le changement de Sol n'y touche pas.
- `src/mcp/BridgetRustInterop*.ts` : 15 erreurs, fichiers non modifiés (règles Effect sur `http`, `fs`, `Date.now`).
- `../../scripts/**` : 59 erreurs, fichiers non modifiés (modules `effect/*` et `@effect/vitest` introuvables depuis la racine).
- `ClaudeMcp.test.ts`, `CodexMcp.test.ts`, `BridgetMcp.ts`, `ClaudeMcp.ts` : 0 erreur TypeScript (seulement des suggestions Effect non bloquantes). Une erreur TS2783 que j'avais créée (`type` en double dans une fixture de test) a été corrigée.

Je n'ai pas masqué ni élargi ces diagnostics. Sortie brute : `/tmp/tsc149c.txt` (volatile).

## Ce que les tests prouvent

### Cause 2 : identifiants de fil opaques (Rust)

Un serveur loopback réel (HTTP et WebSocket pour la V2) lit la cible brute de chaque requête. Il décode le segment une seule fois et ne sert le fil que si le résultat égale l'identifiant d'origine.

- Identifiants acceptés : UUID, `thread:delegated-task:command%3Amcp%3Ax`, `delegated%3A`, `a%2Fb`, `a%25b`, `a%zz`, `a/b`, `a?b`, `a#f`, espace, `+`, antislash, guillemet, unicode (`é日本語😀`), `..%2F..%2Fetc`, longueur 2048 exacte.
- `%3A` devient `%253A` sur le fil. Aucun `/ ? # espace` brut, aucun octet non ASCII brut dans le segment.
- Routes : V1 `?turnLimit=N` ; V2 `/bounded` pour 1 et 20, route complète pour 21 et 200.
- V2 : l'identifiant d'origine (non encodé) atteint `parse_detail`, qui le compare à `projection.thread.id`.
- Refus sans aucune connexion réseau (compteur de connexions à 0) : vide, `.`, `..`, `\n`, `\r`, NUL, tabulation, DEL, U+0085, U+202E, U+200B, U+2066, longueur 2049. Erreur exacte `InvalidShape { thread_detail, threadId }`.
- `.` et `..` intégrés (`a/..`, `../a`, `%2e%2e`, `..a`) restent littéraux.
- Un 404 serveur reste `Http { 404 }`, jamais un succès masqué.

### Cause 1 : preuve fraîche pour le binaire officiel global (adaptateur de bureau)

Fixtures privées uniquement : fichiers exécutables temporaires, lien symbolique `~/.local/bin/bridget`, aucun secret réel.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-durable-tests149.md#extrait-4).

## Limites réelles

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/identite-durable-tests149.md#extrait-5).

## Minimalisme et charge future

- Lignes ajoutées : 406 en Rust, environ 800 en TypeScript. Une grande partie sont des tables de refus, relues une fois et exécutées en 1,3 s.
- Aucun nouveau paquet. Rust utilise `tungstenite`, `serde_json` et `minreq`, déjà dépendances de la crate.
- Fixtures TypeScript `officialFixture` et `canonicalEnv` dupliquées dans les deux fichiers de test : je n'ai le droit de modifier que ces deux fichiers. Un fichier partagé serait possible plus tard, avec trois usages réels.
- Aucun test ne vérifie un mock : le serveur Rust lit les octets réels de la requête, les tests TypeScript lisent le `realPath` et les fichiers réels de la fixture.

## Auto-revue

- Diff relu en entier. Une assertion décorative supprimée côté Rust, un doublon `type` corrigé côté TypeScript, formats appliqués (`rustfmt`, `vp fmt`).
- Hypothèses : `ROOT_SOURCE_READY2` décrit la production finale ; mes tests ont été exécutés contre elle (124 PASS).
- Non vérifié : voir « Limites réelles ».
