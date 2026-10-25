# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/identite-durable-tests149.md](../../../../../../../specs/149-sous-agents-lineage/validation/identite-durable-tests149.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-correctif-identite/crates/bridget-daemon/tests/t3code_opaque_identifiers149_test.rs` (nouveau, 406 lignes, API publique `Client::for_base_url` suffisante, aucun module `#[cfg(test)]` ajouté)
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-correctif-identite/apps/server/src/provider/Drivers/CodexMcp.test.ts` (433 lignes)
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-correctif-identite/apps/server/src/provider/Drivers/ClaudeMcp.test.ts` (963 lignes)

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

Le `vp lint` du worktree échoue (plugin `@oxlint/plugins` introuvable depuis le chemin réel des dépendances liées). Je l'ai lancé depuis `/Users/moi/11.Repositories/t3code-local` avec des chemins du worktree. Une preuve négative (fichier avec `debugger` et variable inutilisée) confirme que ce lint détecte bien les alertes.

<a id="extrait-4"></a>

## Extrait 4

- Positif Codex et Claude : définition globale officielle (lien symbolique) → montage avec `command` égal au `realPath` absolu (jamais la chaîne de l'utilisateur), `args == ["mcp"]`, `env` strictement égal aux six clés canoniques, anciennes valeurs `BRIDGET_T3_MCP_*` écrasées, options sûres conservées (`startup_timeout_sec`, `tool_timeout_sec`).
- Sources Claude couvertes : `~/.claude.json`, `.mcp.json`, statut SDK avec `config`, `servers.bridget` fourni par l'hôte.
- Commande nue résolue par le `PATH` du processus, retour canonique.
- Rotation : deux sessions avec endpoint et autorisation distincts, aucune fuite de l'une vers l'autre, session révoquée sans preuve, fichiers et objets de configuration de l'utilisateur inchangés (comparaison avant/après), aucune clé de session dans l'environnement du processus.
- Refus qui laissent la définition de l'utilisateur intacte (Codex `undefined`, Claude code nommé) : autre binaire, lien symbolique vers un autre binaire, argument en plus, args vide ou absent, autre sous-commande, `enabled` faux ou texte, type http, URL, commande plus URL, exécutable absent, commande nue hors `PATH`, définition non objet ou nulle, env non texte, identité héritée (`BRIDGET_AGENT_ID_FILE`, `BRIDGET_AGENT_INSTANCE_ID`), `HOME`, `BRIDGET_HOME`, `BRIDGET_SOCKET`, `T3CODE_HOME` étrangers.
- Variables inconnues (`DYLD_INSERT_LIBRARIES`, `LD_PRELOAD`, `NODE_OPTIONS`, variable quelconque) : le test accepte un refus ou un montage canonique sans la variable. Il interdit le jeton à côté de la variable. La production actuelle refuse (`user_server`).
- Claude : une seule déclaration étrangère parmi plusieurs suffit à refuser ; opt-out `disabledMcpServers` et `disabledMcpjsonServers` prioritaires ; statut SDK `disabled` donne `disabled` ; `explicit_config`, identité héritée du processus et session révoquée gardent leur code.
- Claude, après montage : `source === "dynamic"` exigé. Une source `user`, `project`, `local`, `plugin`, `managed` ou absente, même connectée avec les outils, donne `catalogue_unavailable` et aucune règle de lecture appliquée. Les deux règles de lecture ne sont appliquées qu'après catalogue vérifié.
- Test existant modifié : les statuts SDK `disabled` d'un homonyme attendent désormais `disabled` au lieu de `user_server` (changement voulu par Sol). Les statuts simulés après montage portent `source: "dynamic"`.

<a id="extrait-5"></a>

## Extrait 5

- Aucun modèle, aucune session T3 réelle, aucun daemon, aucune base live n'ont été lancés. La reconnaissance de l'entrée réelle `source` du SDK repose sur la documentation du type `McpServerStatus` (`sdk.d.ts`), pas sur un CLI Claude réel.
- Le test natif réel du CLI Claude (existant) reste ignoré (1 skip, préexistant).
- Une variable inconnue dans l'`env` de l'utilisateur fait perdre le rafraîchissement : la session reste sans identité et le résultat est un refus visible, pas un montage dégradé. C'est le choix de Sol ; mes tests l'autorisent sans l'imposer.
- Les tests Rust utilisent un serveur de test minimal : ils prouvent le contrat de route et l'identifiant d'origine, pas un T3 réel.
- Les 20 puis 31 échecs RED sur HEAD T3 sont des échecs du comportement attendu (montage absent), pas des erreurs de compilation. Le détail par test n'est pas archivé.
- Le contrôle de format de `BridgetMcp.ts` et `ClaudeMcp.ts` après la correction de Sol n'a pas été relancé.
