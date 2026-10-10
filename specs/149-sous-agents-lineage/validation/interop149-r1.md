# T036 - Interop réseau native 149 (T3 réel <-> daemon Bridget 149 réel)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), ronde natif/réseau r1.
Aucun code de production modifié. Aucun commit. Aucune case cochée dans `tasks.md`.

## Verdict

**APPROVE pour ce qui a été exécuté, sous réserve du tableau des limites. T036 reste à cocher par le principal.**

- Partie 1 (hôte MCP T3 réel + daemon natif réel + clients `bridget mcp` réels + CLI `bridget lineage` réelle) : **43 PASS, 0 FAIL**.
- Partie 2 (serveur T3 complet réel `bin.ts` + base SQLite privée + RPC WebSocket + daemon natif réel) : **15 PASS, 0 FAIL**.
- Aucun modèle n'a été lancé. Les enfants sont un serveur app-server Codex fermé (`codex149.py`).
- Les sources de production Rust ont changé APRÈS la compilation du binaire testé (voir « Empreintes »). Il faut rejouer ces recettes sur le prochain binaire (`NATIVE149_BIN=...`).

## Couches : réel ou simulé

| Couche | État |
|---|---|
| Daemon Bridget 149, client MCP, CLI Lineage | RÉEL : binaire debug `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-823e8a5fab8a` |
| Serveur HTTP MCP de T3, registre de sessions (émission, rotation, révocation), toolkit, `bridget_session` v1/v2 | RÉEL : modules du worktree T3, port 14776 |
| Serveur T3 complet (auth bearer scopée, RPC WebSocket, `BridgetReader`, `BridgetLineage`, projection SQLite) | RÉEL : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bin.ts`, port 15736 |
| Magasin de faits de permissions (`publishMcpProviderPermissions`, tombstone, revision) | RÉEL (code T3) ; **le fait est publié par la recette** à la place de l'adaptateur Codex, car aucun fournisseur ne tourne : SIMULÉ |
| Projection de conversation de l'hôte MCP (run actif) | SIMULÉ en mémoire (gabarit de `BridgetRustInterop.test.ts`) |
| Wrapper de fil T3 côté daemon (Register + `T3ThreadBindingFact`) | SIMULÉ : trames réelles envoyées par la recette ; le credential vient du vrai `Registered`, jamais forgé |
| Enfant | FERMÉ, sans modèle : un lancement = un PID réel consigné |

Ni l'hôte MCP (14776) ni le serveur complet (15736) ne sont le même processus T3. En production, un seul processus T3 porte les deux. Ce point n'est pas prouvé ici.

## Commandes exactes

```zsh
umask 077
export PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:$PATH
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes
node interop149.ts   # partie 1
node server149.ts    # partie 2
python3 -I fingerprints.py
```

Résultats bruts : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/interop149.json`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/server149.json`, sorties `*.stdout.txt` du même dossier.
Fixtures : `/Users/moi/.cache/bridget149-native-interop.*` (0700, APFS, supprimées après la ronde) et projet git privé sous `/tmp/b149n.*`.

## Partie 1 - hôte MCP T3 réel et daemon natif réel

| Id | Résultat | Ce qui est prouvé | Valeur observée |
|---|---|---|---|
| S1.1 | PASS | deux credentials distincts, même endpoint et même instance fournisseur, sessions fournisseur distinctes | `{"sameEndpoint": true, "sameInstance": true}` |
| S1.2 | PASS | les deux agents Bridget dérivent de leur fil et diffèrent | `{"a": "74786b56", "b": "9744595c"}` |
| S1.3 | PASS | bridget_session sans fait : enveloppe v1 identité seule (5 champs camelCase, pas de permissions) | `{"keys": ["environmentId", "providerInstanceId", "providerSessionId", "threadId", "version"], "version": 1}` |
| S1.4 | PASS | bridget_who via T3 réel résout A et B sur leurs propres agents (identité148) | `{"aErr": false, "bErr": false}` |
| S1.5 | PASS | chaque attestation ouvre puis ferme sa session de transport HTTP (DELETE par session, statuts 2xx) | `{"posts": 9, "deletes": 3, "statuses": [200, 202, 204]}` |
| S2.1 | PASS | G-P-07(c) : admission inherit avec la seule enveloppe v1 -> permission_attestation_unavailable | `{"text": "permission_attestation_unavailable", "code": "native_delegation_refused"}` |
| S2.2 | PASS | v1 + posture development : même refus nommé, aucune ligne, aucun lancement, aucune remise, aucun grant | `{"text": "permission_attestation_unavailable", "before": {"rows": 0, "started": 0, "prompts": 0, "answered": 0, "deliveries": {"A": 0, "B": 0}}, "after": {"rows": 0, "started": 0, "prompts": 0, "answered": 0, "deliveries": {"A": 0…` |
| S2.3 | PASS | status/identité148 restent fonctionnels en v1 (tâche inconnue -> task_unavailable, pas de crash) | `{"payload": {"text": "task_unavailable"}}` |
| S3.1 | PASS | bridget_session avec fait publié : enveloppe v2 stricte (identité camelCase + permissions snake_case, revision 1) | `{"v2keys": ["environmentId", "permissions", "providerInstanceId", "providerSessionId", "threadId", "version"], "permKeys": ["cwd", "driver", "interaction_mode", "provider_instance_id", "provider_policy", "provider_session_id", "re…` |
| S3.2 | PASS | l'enveloppe v2 ne contient ni token, ni endpoint, ni en-tête d'autorisation (<= 64 KiB) | `{"bytes": 1455}` |
| S3.3 | PASS | le fait de A n'est jamais prêté à B : B (même processus fournisseur) reste en v1 | `{"version": 1}` |
| S3.4 | PASS | A (v2 full-access) : bridget_delegate sans posture admis, posture effective development, 1 ligne durable | `{"status": "queued", "posture": "development"}` |
| S3.5 | PASS | le moteur a lancé UN processus enfant réel et UN tour ; résultat corrélé disponible | `{"state": "result_available", "pid": 41657, "result": "fixture149-answer:a1"}` |
| S3.6 | PASS | la politique de l'enfant est celle du fait parent (dangerFullAccess + approval never), figée dans le snapshot | `{"sandbox": {"type": "dangerFullAccess"}, "approval": "never", "snapshotKeys": ["version", "source", "owner_agent_id", "owner_instance_id", "parent", "child_policy", "project_cwd"]}` |
| S3.7 | PASS | exactement une remise corrélée (in_reply_to = mission, from = enfant, to = A), ACK réel, rien chez B | `{"aDeliveries": 1, "matching": 1, "bDeliveries": 0, "acks": 1}` |
| S3.8 | PASS | B sans fait : refus nommé, A inchangé (aucun emprunt du fait de A) | `{"text": "permission_attestation_unavailable", "rows": 1}` |
| S3.9 | PASS | S149-02/25 : B lecteur (sandbox readOnly) demande development -> permission_not_inherited, aucune ligne, aucun lancement, aucune invitation au grant | `{"text": "permission_not_inherited", "rows": 1, "started": 1, "revB": 1}` |
| S3.10 | PASS | B (readOnly) sans posture : admis en lecture ; l'enfant de B reçoit un sandbox readOnly, celui de A est resté dangerFullAccess (S149-25) | `{"posture": "discovery", "sandboxB": {"type": "readOnly"}}` |
| S3.11 | PASS | B ne lit ni n'annule la tâche de A (task_unavailable, aucune fuite de champ) | `{"status": {"text": "task_unavailable"}, "cancel": {"text": "task_unavailable"}}` |
| S3.12 | PASS | B rejoue le request_id de A : jamais la tâche de A (soit refus, soit tâche distincte propre à B) | `{"isError": false, "sameTask": false, "rowsBefore": 2, "rowsAfter": 3}` |
| S4.1 | PASS | rotation : l'ancien credential est refusé par T3 (HTTP 401) et par Rust (t3_session_unavailable), sans repli | `{"http": 401, "code": "t3_session_unavailable"}` |
| S4.2 | PASS | le nouveau credential retrouve la même identité Bridget (stable) mais aucun fait hérité : v1 | `{"version": 1, "sameAgent": true}` |
| S4.3 | PASS | après rotation sans fait : nouvelle admission refusée (v1) mais lecture de la tâche admise intacte | `{"refusal": "permission_attestation_unavailable", "status": "result_available"}` |
| S4.4 | PASS | révocation de B : HTTP 401 + t3_session_unavailable ; le voisin A (nouveau credential) reste servi | `{"http": 401, "code": "t3_session_unavailable", "neighborOk": true}` |
| S4.5 | PASS | identités fausses : token forgé, endpoint hors runtime, credential valide sans binding Bridget -> t3_session_unavailable, jamais de repli PID | `{"forged": "t3_session_unavailable", "wrongPort": "t3_session_unavailable", "unboundC": "t3_session_unavailable", "delegateC": "t3_session_unavailable"}` |
| S5.1 | PASS | G-P-07(b) côté T3 : fait retiré (fin de run) -> refus nommé permission_attestation_unavailable, jamais l'enveloppe v1 | `{"rawIsError": true, "namedCode": "permission_attestation_unavailable", "hasStructured": false}` |
| S5.1b | PASS | côté Rust (montage MCP T3) : même credential -> admission refusée fermée, zéro ligne, zéro lancement | `{"delegate": "t3_session_unavailable", "status": "t3_session_unavailable"}` |
| S5.2 | PASS | fait lié à un autre run que le run actif : refus nommé ; la revision a augmenté à chaque publication du même credential | `{"rev3": 2, "stale": 3, "code": "permission_attestation_unavailable"}` |
| S5.3 | PASS | changement de mode (plan) : nouvelle revision pour les futures admissions ; le snapshot de la tâche déjà admise est inchangé (invariant 5) | `{"revision": 4, "snapshotUnchanged": true}` |
| S6.1 | PASS | S149-19 : A voit seulement sa tâche, B (même projet, même processus) seulement la sienne ; aucune ne voit l'autre | `{"a": 1, "b": 2}` |
| S6.2 | PASS | B ne peut ni show, ni journal, ni cancel la tâche de A via la CLI lecteur (task_unavailable, exit 2) | `{"show": "task_unavailable", "journal": "task_unavailable", "cancel": "task_unavailable"}` |
| S6.3 | PASS | mauvaise racine -> project_mismatch ; fil inconnu et fil sans binding -> binding_unavailable ; aucune donnée rendue | `{"wrongRoot": "project_mismatch", "unknown": "binding_unavailable", "unbound": "binding_unavailable"}` |
| S6.4 | PASS | show de la tâche terminale par son propre fil : résultat exact servi | `{"code": 0, "result": "fixture149-answer:a1"}` |
| S6.5 | PASS | 150 lectures Lineage : aucun lancement, aucune remise, aucun tour, base byte-stable (lecture quiète, aucun provider turn) | `{"quietBefore": {"rows": 3, "started": 3, "prompts": 3, "answered": 3, "deliveries": {"A": 1, "B": 2}, "db": "9ee81e018ead3604"}, "quietAfter": {"rows": 3, "started": 3, "prompts": 3, "answered": 3, "deliveries": {"A": 1, "B": 2},…` |
| S6.9 | PASS | pagination en snapshot : pages de 1 tâche, même génération et même seq, union = liste complète, aucun doublon | `{"pages": 4, "tasks": 4, "seq": 36}` |
| S6.10 | PASS | mutation entre deux pages : page 2 rend snapshot_changed (le staging est à abandonner), jamais un snapshot mélangé | `{"mutated": true, "page2": "snapshot_changed"}` |
| S6.11 | PASS | grammaire CLI fermée : formes ouvertes ou hors bornes -> exit 2, invalid_request, retryable false | `{"codes": [[2, "invalid_request"], [2, "invalid_request"], [2, "invalid_request"], [2, "invalid_request"], [2, "invalid_request"], [2, "invalid_request"]]}` |
| S6.6 | PASS | watch : ready seq 0 premier pour chaque racine | `{"a": {"generation": "fc0a1764-5321-4cfb-b497-c4cbbeaa89d7", "seq": 0, "status": "ready", "version": 1}, "b": {"generation": "fc0a1764-5321-4cfb-b497-c4cbbeaa89d7", "seq": 0, "status": "ready", "version": 1}}` |
| S6.7 | PASS | mutation de la racine de A (nouvelle tâche) : A est signalé (corps absent), B ne reçoit rien (aucun UUID de fil ni contenu) | `{"aLines": ["ready", "changed", "changed"], "bLines": 1}` |
| S6.8 | PASS | cancel CLI : reçu natif, rejeu identique = même reçu, même request_id sur une autre tâche = envelope_mismatch, tâche réellement cancelled | `{"first": {"status": "cancelling", "task_id": "26682637-1073-4b1a-8d8f-fea46f5a1447", "version": 1}, "replay": {"status": "cancelling", "task_id": "26682637-1073-4b1a-8d8f-fea46f5a1447", "version": 1}, "mismatch": "envelope_mismat…` |
| S7.1 | PASS | aucune autorité en trop : parent, permissions, sandbox, endpoint, credential, proof, agent_id forgés dans les arguments -> 7 refus (arguments fermés), zéro ligne, zéro lancement | `{"refused": 7, "of": 7, "rows": [8, 8], "messages": {"parent": "champ inconnu", "permissions": "champ inconnu", "sandbox": "champ inconnu", "endpoint": "champ inconnu", "credential": "champ inconnu", "proof": "champ inconnu", "age…` |
| S7.2 | PASS | projet attesté : cwd hors projet (/tmp), autre dépôt git, lien symbolique sortant, .. -> refus d'admission avant tout lancement | `{"tmp": "cwd_outside_parent_project", "otherRepo": "cwd_outside_parent_project", "symlink": "cwd_outside_parent_project", "dotdot": "cwd_outside_parent_project"}` |
| S7.3 | PASS | conversation D d'un AUTRE projet : ne peut pas lancer dans le projet de A (refus), lance dans le sien ; sa liste Lineage ne contient que sa tâche ; racines croisées -> project_mismatch ; tâche de A introuvable | `{"intoOtherProject": "cwd_outside_parent_project", "ownProject": "queued", "listD": 1, "dWrongRoot": "project_mismatch", "aWrongRoot": "project_mismatch", "dShowA": "task_unavailable"}` |

### Compteurs observés (dernier passage, partie 1)

- Tâches natives : 9 lignes. Lancements d'enfants (PID réels) : 9. Tours : 9. Réponses : 8.
- Remises idempotentes au parent A : 6 ; au parent B : 2.
- HTTP vers l'hôte MCP T3 : 224 POST, 73 DELETE ; statuts {'200': 146, '202': 73, '204': 73, '401': 5}. Les cinq 401 sont les refus attendus (credentials tournés, révoqués ou forgés).
- Processus résiduels après nettoyage : 0.
- Catalogue sous v1 : `inherit=false`, refus `permission_attestation_unavailable`.
- Catalogue sous v2 (parent Codex full-access) : cible `codex` et `fixture-codex-149` `inherit=true, development=true` ; cibles Claude `inherit=false`, refus `permission_source_unavailable` (aucun launcher Claude attesté dans cette fixture). Cohérent avec le contrat.

## Partie 2 - serveur T3 complet réel et daemon natif réel

| Id | Résultat | Ce qui est prouvé | Valeur observée |
|---|---|---|---|
| V0 | PASS | serveur T3 réel démarré sur base privée, projet + deux conversations créés par RPC | `{"results": [200, "ok", "ok"]}` |
| V1 | PASS | A : racine + enfant imbriqué (lien parent_task_id), modèle/protocole/posture du moteur ; la racine attend ses enfants (waiting_for_children), résultat retenu | `{"rootStatus": "waiting_for_children", "nestedStatus": "working", "tasks": 2}` |
| V2 | PASS | S149-19 via serveur T3 : B (même projet) ne voit que sa tâche, jamais celles de A | `{"b": ["465e7e88"]}` |
| V3 | PASS | B ne peut ni show, ni journal, ni cancel la tâche de A (task_unavailable) ; token read ne peut pas cancel | `{"show": "task_unavailable", "journal": "task_unavailable", "cancel": "task_unavailable"}` |
| V3b | PASS | scope : le jeton orchestration:read est refusé sur cancel (aucun effet natif) | `{"error": "EnvironmentAuthorizationError"}` |
| V4 | PASS | projection T3 : un fil virtuel marqué par tâche native (3, relation subagent) ; l'ensemble des conversations de premier niveau est identique avant/après lecture (aucune conversation créée par la lecture) | `{"virtual": 3, "topLevelBefore": 3, "topLevelAfter": 3, "topTitles": ["New thread", "Conversation A", "Conversation B"]}` |
| V5 | PASS | zéro effet fournisseur côté T3 : 0 run, 0 tentative, 0 tour, 0 session, 0 effet en attente, 0 message | `{"projection_runs": 0, "projection_run_attempts": 0, "projection_provider_turns": 0, "projection_provider_sessions": 0, "projection_provider_threads": 0, "projection_provider_session_bindings": 0, "projection_runtime_requests": 0,…` |
| V6 | PASS | tant que l'enfant imbriqué travaille, la racine n'expose aucun résultat (retenu) ; l'enfant lui-même n'a pas de résultat | `{"rootResult": null, "nestedResult": null}` |
| V7 | PASS | journal réel de l'enfant actif lisible via T3 (événements du daemon, séquence ordonnée) | `{"events": 5, "gap": null}` |
| V8 | PASS | bridget.lineage.watch via T3 : ready seq 0 premier, puis changed sans corps ni UUID après nouvelle tâche réelle | `{"statuses": ["ready", "changed"], "keys": ["version", "generation", "seq", "status"]}` |
| V9 | PASS | cancel via T3 : reçu natif, rejeu identique, request_id sur autre tâche = envelope_mismatch ; tâche cancelled et PID enfant réellement terminé | `{"receipt": {"version": 1, "task_id": "e45ef1c9-c942-4916-90f9-05b9a481af98", "status": "cancelling"}, "replaySame": true, "mismatch": "envelope_mismatch", "state": "cancelled", "childPid": 194, "alive": false}` |
| V10 | PASS | l'annulation depuis T3 ne crée aucun run/tour/session fournisseur côté T3 (tables toujours à zéro) | `{"projection_runs": 0, "projection_run_attempts": 0, "projection_provider_turns": 0, "projection_provider_sessions": 0, "projection_provider_threads": 0, "projection_provider_session_bindings": 0, "projection_runtime_requests": 0,…` |
| V11 | PASS | 110 lectures T3 -> daemon : aucun lancement, aucun tour, aucune mutation (états et updated_at identiques) | `{"before": {"rows": 4, "started": 4, "prompts": 4, "answered": 2, "deliveries": {}}, "after": {"rows": 4, "started": 4, "prompts": 4, "answered": 2, "deliveries": {}}}` |
| V12 | PASS | daemon coupé : T3 refuse explicitement (aucun contenu présenté comme courant) | `{"error": "binding_unavailable", "message": "Bridget lineage is unavailable (binding_unavailable)."}` |
| V13 | PASS | reprise sans relance : même génération native, mêmes tâches, aucun fil virtuel en double côté T3 | `{"sameGeneration": true, "tasksBefore": 2, "tasksAfter": 3, "virtualThreadsAfter": 4}` |

### Compteurs observés (dernier passage, partie 2)

- Base T3 privée : 0 run, 0 tentative, 0 tour, 0 session, 0 thread fournisseur, 0 binding, 0 requête runtime, 0 effet en attente, 0 workflow de lancement, 0 message. Fils virtuels : 4 (3 tâches natives + 1 tâche ajoutée ensuite).
- Daemon : 6 lancements d'enfants au total, dont 2 après la relance du daemon (voir le constat F1 de `recovery149.md`).

## Constats

1. **Aucun écart sur le contrat de permissions v1/v2** pour ce qui est exécuté : v1 sans fait, v2 stricte avec fait, jamais de repli v1 après retrait/stale, jamais de prêt de fait entre conversations, rotation et révocation par credential.
2. **Observation O1 (faible).** Après un tombstone (fin de run) la session MCP montée par `bridget mcp` refuse TOUS les outils (`t3_session_unavailable`), même `bridget_task_status` d'une tâche déjà admise (contrôle S5.1b). Le code nommé `permission_attestation_unavailable` n'apparaît que dans la réponse brute de `bridget_session` côté T3 (S5.1). Le contrat G-P-07(b) dit « lecture et rejeu inchangés ». Ce point est vrai pour la CLI Lineage native et pour un credential neuf sans fait (v1 : rejeu et status rendent l'état mémorisé, voir `recovery149.md` R8.3). Il est faux pour le credential tombstoné. Le comportement est fermé et sûr. Le texte du contrat devrait le dire.
3. **Observation O2.** Le projet T3 ajoute lui-même une conversation « New thread » à la création. Elle n'est pas produite par Bridget. L'ensemble des conversations de premier niveau est identique avant et après lecture Lineage (V4).
4. **Leçon de harnais, sans défaut produit.** `delivery_generation` est un u64 à 19 chiffres. Un client JavaScript qui le relit avec `JSON.parse` l'arrondit. Le daemon répond alors `Nack: remise idempotente invalide` et laisse la remise en phase `dispatching` ; elle est rejouée à chaque reconnexion. La recette conserve maintenant les chiffres exacts. Un consommateur non Rust de ce protocole doit faire pareil.

## Ce qui n'est PAS prouvé ici

| Point | Pourquoi | Propriétaire |
|---|---|---|
| Publication du fait par les vrais adaptateurs `CodexAdapterV2` / `ClaudeAdapterV2` en exécution | aucun fournisseur lancé ; couvert seulement par les tests T3 | tests T3 |
| Modèle réel, effet d'écriture, refus d'écriture hors politique | aucun modèle | T037 |
| Voie standalone Claude/GLM (launcher gclaude, `settings_revision_changed`, observer PTY G-P-02/G-P-08) | cible Claude refusée `permission_source_unavailable` avec cette fixture | T038 |
| Refus fournisseur `can_use_tool` corrélé (G-P-01) | fixture sans trame `can_use_tool` | tests Rust |
| Confinement OS | non applicable à une fixture | hors périmètre |
| Un seul processus T3 qui émet les credentials ET sert Lineage | deux instances T3 réelles dans cette recette | à confirmer en recette finale |
| Coque desktop, mobile, rendu navigateur | non lancés | recette UI (autre propriétaire) |
| Crash brutal du daemon (SIGKILL) | interdit par la règle de processus ; seul SIGTERM est testé | hors périmètre |

## Empreintes

Binaire : sha256 `823e8a5fab8abe38fcb439d8fda98d929622dae3751ff8f629d306744379328b` (égal au reçu `native149-debug-receipt.json` : True).

Sources Rust : empreinte actuelle `f54a66ee8bcdc305…`, empreinte du reçu `e6f45086b2397460…`, égales : **False**.
Sources de production plus récentes que le binaire : `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/daemon/native_delegation.rs`, `crates/bridget-daemon/src/wrapper.rs`.
Conséquence : les résultats valent pour le binaire `823e8a5f`, pas pour l'état actuel de ces trois fichiers.

T3 (worktree, HEAD `33f6d04e11`, arbre modifié) :

| Fichier | sha256 |
|---|---|
| `packages/provider-core/src/server/mcpSession.ts` | `2e95140365bef3f8…` |
| `apps/server/src/mcp/OrchestratorMcpService.ts` | `e4d6a2db680f9720…` |
| `apps/server/src/mcp/McpSessionRegistry.ts` | `6e1282d6fcde6846…` |
| `apps/server/src/mcp/McpHttpServer.ts` | `0bf639fbb86bb23f…` |
| `apps/server/src/bridget/BridgetReader.ts` | `99acc8a7c38c2477…` |
| `apps/server/src/bridget/BridgetLineage.ts` | `ddb6be37c21e1582…` |
| `apps/server/src/ws.ts` | `c9cce8ceef8fcb5a…` |
| `apps/server/src/server.ts` | `04bcd1aeae7c70f3…` |
| `packages/contracts/src/bridgetLineage.ts` | `cd216feb52261c28…` |
| `packages/contracts/src/bridgetPermissions.ts` | `df03fd3fb71dc3e2…` |

Outils de recette :

| Fichier | sha256 |
|---|---|
| `codex149.py` | `7de49a1b3edc6a73…` |
| `common149.ts` | `e273956be66a6e1d…` |
| `fingerprints.py` | `c5dec1c463b8f651…` |
| `fx.mjs` | `6f2e9f3c2467ce6a…` |
| `interop149.ts` | `f1f7cfaa59963617…` |
| `provider149.py` | `cdcefa2b742738fd…` |
| `recovery149.ts` | `12c3a16109eb5b49…` |
| `server149.ts` | `c120df6f1d291336…` |
| `t3host.ts` | `10b96a42ed8a9750…` |
| `t3server.ts` | `6c8ffc8b5ddc7d24…` |
