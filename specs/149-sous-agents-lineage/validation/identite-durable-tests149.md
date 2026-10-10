# Tests du correctif durable d'identité 149

Date : 2026-10-11. Rôle : sous-agent de tests. Aucun fichier de production touché.
Le binaire global 149 n'est pas utilisé comme preuve : tout part des sources des deux worktrees.

## Fichiers dont je suis propriétaire

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-correctif-identite/crates/bridget-daemon/tests/t3code_opaque_identifiers149_test.rs` (nouveau, 406 lignes, API publique `Client::for_base_url` suffisante, aucun module `#[cfg(test)]` ajouté)
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-correctif-identite/apps/server/src/provider/Drivers/CodexMcp.test.ts` (433 lignes)
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-correctif-identite/apps/server/src/provider/Drivers/ClaudeMcp.test.ts` (963 lignes)

## Résultats

| Contrôle | Résultat | Durée |
| --- | --- | --- |
| Rust `t3code_opaque_identifiers149_test` (release, offline, 4 jobs, `CARGO_INCREMENTAL=0`, cible privée) | 6 PASS / 0 échec (dernier passage après `rustfmt`, 1,16 s de test) | build 45 à 87 s |
| Rust `--lib t3code_contract` (régression des tests V1/V2 existants) | 19 PASS, 1 ignoré (interop réelle, désactivée) | 73 s |
| Rust RED sur le source HEAD (copie `git archive`, hors worktree) | 6 sur 6 échecs | 81 s |
| `rustfmt --edition 2024 --check` sur mon fichier | OK | - |
| T3 `CodexMcp.test.ts` + `ClaudeMcp.test.ts` | 124 PASS, 1 skip, 0 échec (2 fichiers) | 1,3 s |
| T3 RED sur HEAD (copies temporaires des anciens `CodexMcp.ts` et `ClaudeMcp.ts`, supprimées) | 31 échecs sur 125 (série finale) ; 4 puis 20 échecs sur les séries intermédiaires | - |
| Mutation : exigence `source === "dynamic"` retirée dans une copie temporaire | 6 échecs ciblés (sources user, project, local, plugin, managed, absente) | - |
| `vp lint --report-unused-disable-directives` sur mes 2 tests et les 3 sources T3 | rc 0, 0 alerte | - |
| `vp fmt --check` sur mes 2 tests | OK | - |
| `vp fmt --check` sur `BridgetMcp.ts` et `ClaudeMcp.ts` (production) | 2 fichiers à reformater au dernier contrôle ; Sol corrige, sans changement sémantique. Je n'ai pas relancé le contrôle. | - |

Le `vp lint` du worktree échoue (plugin `@oxlint/plugins` introuvable depuis le chemin réel des dépendances liées). Je l'ai lancé depuis `/Users/moi/11.Repositories/t3code-local` avec des chemins du worktree. Une preuve négative (fichier avec `debugger` et variable inutilisée) confirme que ce lint détecte bien les alertes.

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

### Cause 1 : preuve fraîche pour le binaire officiel global (T3)

Fixtures privées uniquement : fichiers exécutables temporaires, lien symbolique `~/.local/bin/bridget`, aucun secret réel.

- Positif Codex et Claude : définition globale officielle (lien symbolique) → montage avec `command` égal au `realPath` absolu (jamais la chaîne de l'utilisateur), `args == ["mcp"]`, `env` strictement égal aux six clés canoniques, anciennes valeurs `BRIDGET_T3_MCP_*` écrasées, options sûres conservées (`startup_timeout_sec`, `tool_timeout_sec`).
- Sources Claude couvertes : `~/.claude.json`, `.mcp.json`, statut SDK avec `config`, `servers.bridget` fourni par l'hôte.
- Commande nue résolue par le `PATH` du processus, retour canonique.
- Rotation : deux sessions avec endpoint et autorisation distincts, aucune fuite de l'une vers l'autre, session révoquée sans preuve, fichiers et objets de configuration de l'utilisateur inchangés (comparaison avant/après), aucune clé de session dans l'environnement du processus.
- Refus qui laissent la définition de l'utilisateur intacte (Codex `undefined`, Claude code nommé) : autre binaire, lien symbolique vers un autre binaire, argument en plus, args vide ou absent, autre sous-commande, `enabled` faux ou texte, type http, URL, commande plus URL, exécutable absent, commande nue hors `PATH`, définition non objet ou nulle, env non texte, identité héritée (`BRIDGET_AGENT_ID_FILE`, `BRIDGET_AGENT_INSTANCE_ID`), `HOME`, `BRIDGET_HOME`, `BRIDGET_SOCKET`, `T3CODE_HOME` étrangers.
- Variables inconnues (`DYLD_INSERT_LIBRARIES`, `LD_PRELOAD`, `NODE_OPTIONS`, variable quelconque) : le test accepte un refus ou un montage canonique sans la variable. Il interdit le jeton à côté de la variable. La production actuelle refuse (`user_server`).
- Claude : une seule déclaration étrangère parmi plusieurs suffit à refuser ; opt-out `disabledMcpServers` et `disabledMcpjsonServers` prioritaires ; statut SDK `disabled` donne `disabled` ; `explicit_config`, identité héritée du processus et session révoquée gardent leur code.
- Claude, après montage : `source === "dynamic"` exigé. Une source `user`, `project`, `local`, `plugin`, `managed` ou absente, même connectée avec les outils, donne `catalogue_unavailable` et aucune règle de lecture appliquée. Les deux règles de lecture ne sont appliquées qu'après catalogue vérifié.
- Test existant modifié : les statuts SDK `disabled` d'un homonyme attendent désormais `disabled` au lieu de `user_server` (changement voulu par Sol). Les statuts simulés après montage portent `source: "dynamic"`.

## Limites réelles

- Aucun modèle, aucune session T3 réelle, aucun daemon, aucune base live n'ont été lancés. La reconnaissance de l'entrée réelle `source` du SDK repose sur la documentation du type `McpServerStatus` (`sdk.d.ts`), pas sur un CLI Claude réel.
- Le test natif réel du CLI Claude (existant) reste ignoré (1 skip, préexistant).
- Une variable inconnue dans l'`env` de l'utilisateur fait perdre le rafraîchissement : la session reste sans identité et le résultat est un refus visible, pas un montage dégradé. C'est le choix de Sol ; mes tests l'autorisent sans l'imposer.
- Les tests Rust utilisent un serveur de test minimal : ils prouvent le contrat de route et l'identifiant d'origine, pas un T3 réel.
- Les 20 puis 31 échecs RED sur HEAD T3 sont des échecs du comportement attendu (montage absent), pas des erreurs de compilation. Le détail par test n'est pas archivé.
- Le contrôle de format de `BridgetMcp.ts` et `ClaudeMcp.ts` après la correction de Sol n'a pas été relancé.

## Minimalisme et charge future

- Lignes ajoutées : 406 en Rust, environ 800 en TypeScript. Une grande partie sont des tables de refus, relues une fois et exécutées en 1,3 s.
- Aucun nouveau paquet. Rust utilise `tungstenite`, `serde_json` et `minreq`, déjà dépendances de la crate.
- Fixtures TypeScript `officialFixture` et `canonicalEnv` dupliquées dans les deux fichiers de test : je n'ai le droit de modifier que ces deux fichiers. Un fichier partagé serait possible plus tard, avec trois usages réels.
- Aucun test ne vérifie un mock : le serveur Rust lit les octets réels de la requête, les tests TypeScript lisent le `realPath` et les fichiers réels de la fixture.

## Auto-revue

- Diff relu en entier. Une assertion décorative supprimée côté Rust, un doublon `type` corrigé côté TypeScript, formats appliqués (`rustfmt`, `vp fmt`).
- Hypothèses : `ROOT_SOURCE_READY2` décrit la production finale ; mes tests ont été exécutés contre elle (124 PASS).
- Non vérifié : voir « Limites réelles ».
