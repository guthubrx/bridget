# Interop réseau natif 149 - T036 - ronde r2

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production, aucun modèle.

## Verdict : APPROVE sur le périmètre exécuté (65/65 PASS)

| Recette | Contrôles | Résultat brut |
|---|---|---|
| `interop149.ts` : hôte MCP de T3 + daemon natif | 43/43 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/interop149.json` |
| `server149.ts` : serveur T3 complet (`bin.ts`) + daemon natif | 15/15 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/server149.json` |
| `unified149.ts` : UNE seule instance T3 qui émet le credential et sert Lineage | 7/7 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/unified149.json` |

Les 58 premiers contrôles rejouent le périmètre de la ronde r1 sur le nouveau binaire. Les 7 derniers sont nouveaux et lèvent la réserve « deux instances T3 » de la ronde r1.

## Binaire et sources testés

- Binaire IMMUABLE r5 : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-ec6b18b5d468`.
- SHA-256 recalculé : `ec6b18b5d468dd0031eb6282da10bdc27cb38b7193f07b4a8b9107cbe92d8b0f`. Il est égal à celui du reçu `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native149-debug-receipt-r5.json` (le reçu canonique est passé depuis à r6, binaire différent, non rejoué ici).
- Chaque recette refuse de démarrer si l'empreinte du binaire diffère de celle du reçu r5.
- Sources Rust : les empreintes de tous les fichiers listés au reçu r5 (fichiers clés de production + fichiers modifiés ou non suivis) sont identiques à celles d'aujourd'hui (`sourcesChangedSinceReceiptDigests` vide). `daemon.rs` `04d5aca3…`, `native_delegation.rs` `5da3619e…`, `wrapper.rs` `e838a821…`.
- Dérive FUTURE des manifestes, à ne pas confondre avec le manifeste historique du binaire : `Cargo.toml` (`sha2` avec la fonction `asm`) et `Cargo.lock` ont changé après la compilation du binaire r5, et un fichier de test (`crates/bridget-transport/tests/permission_source_revision_vectors149_test.rs`) a été ajouté. L'empreinte de production d'aujourd'hui (`6cc9a2be…`) est celle du reçu r6 : r6 = r5 + manifestes seulement. Le binaire r5 reste donc représentatif du code Rust actuel, sauf pour la dépendance `sha2`.
- Sources T3 (`/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`, HEAD `33f6d04e`) : empreinte des 32 fichiers de production recalculée à `26fba25527ad0247` (même valeur que le rapport T3 r5) ; `git diff HEAD` à `59a54267ce7c1d2a` (même valeur). Les recettes importent ces modules directement, donc elles exercent les sources T3 actuelles (formatter et HostPlatform compris).
- Résultats bruts d'empreintes : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/fingerprints.json`.

## Couches : réel ou simulé

| Couche | État |
|---|---|
| Daemon Bridget 149, clients `bridget mcp`, CLI `bridget lineage` | RÉEL (binaire r5) |
| Hôte MCP de T3 (HTTP Node, registre, `bridget_session` v1/v2) pour `interop149.ts` | RÉEL ; la projection de conversation (runs actifs) est EN MÉMOIRE (simulée) et le fait de permissions est publié par la recette via l'API publique (la recette tient lieu d'adaptateur) |
| Serveur T3 complet `bin.ts`, base SQLite privée, RPC WebSocket, jetons scopés, `BridgetReader` | RÉEL (`server149.ts`, `unified149.ts`) |
| Session fournisseur Codex de T3 (`ProviderSessionManager`, adaptateur Codex V2) | RÉEL dans `unified149.ts` : c'est lui qui émet le credential et publie le fait de permissions |
| Processus « codex » de `unified149.ts` | SIMULÉ : faux pair app-server fourni par T3 (`testFixtures/codexCollabMockPeer.mjs`, copié en lecture seule) derrière un espion qui consigne les trames. Il rejoue des réponses de session capturées. Aucun modèle. |
| Wrapper du fil T3 côté daemon (Register + `T3ThreadBindingFact`) | SIMULÉ (trames réelles émises par la recette ; le credential vient du vrai `Registered`) |
| Enfants des missions | SIMULÉS : faux serveur app-server Codex fermé, sans modèle (`codex149.py`) ; chaque lancement est un PID réel compté |

Pas de `bridget_fixture.mjs`, pas de faux Orchestrator HTTP.

## Harnais : précision des ACK (obligatoire, conservée)

`delivery_generation` est un u64 de 19 chiffres. `JSON.parse` l'arrondirait. Le harnais (`fx.mjs`, classe `JsonLines` et `Peer`) relit les chiffres exacts dans le texte de la ligne et écrit l'ACK à la main. Preuve : R9.1b (aucune remise déjà accusée n'est rejouée après relance) passe, et aucune remise ne se répète dans les 3 recettes.

## Incident de recette à connaître (premier essai de `unified149.ts`)

Mon premier essai plaçait un faux `codex` dans le PATH du serveur T3. Le serveur a résolu le `codex` RÉEL de l'utilisateur par le shell de connexion, a ouvert une vraie session `app-server` dans un HOME isolé et a envoyé la requête de recette (« bonjour recette ») à `api.openai.com`. Réponse : 401 « Missing bearer or basic authentication ». Aucun jeton, aucune configuration utilisateur copiée, aucun modèle appelé (HOME et CODEX_HOME isolés dans le cache de la fixture ; le 401 « Missing bearer » prouve qu'aucune authentification n'était présente). Le texte envoyé n'avait aucun secret. Je l'ai constaté dans le journal fournisseur de la fixture, puis arrêté cette voie.

Correctif de la recette : `settings.json` de la fixture impose le chemin ABSOLU du faux pair, et les autres fournisseurs sont désactivés (`claudeAgent`, `cursor`, `grok`, `opencode`). Je n'ai vu aucun processus `codex` résiduel de la fixture (le seul `codex app-server` vivant est enfant du T3 de production, PID parent 58468, lancé à 16:22:46, donc pas le mien).

Note sur les recettes `server149.ts` : au démarrage, le serveur T3 exécute deux commandes d'installation Codex (`CodexInstallation.runCommand`, vues dans la trace) sur le vrai binaire. Je n'ai vu aucune session `app-server` ni aucun journal fournisseur dans ces fixtures. Ce comportement vient de T3, il existait en r1.

## Résultats détaillés

### T036 partie 1 - hôte MCP T3 + daemon natif (43)

| Id | Résultat | Couche | Contrôle | Observé |
|---|---|---|---|---|
| S1.1 | PASS | REEL | deux credentials distincts, même endpoint et même instance fournisseur, sessions fournisseur distinctes | `{"sameEndpoint": true, "sameInstance": true}` |
| S1.2 | PASS | REEL | les deux agents Bridget dérivent de leur fil et diffèrent | `{"a": "74786b56", "b": "9744595c"}` |
| S1.3 | PASS | REEL | bridget_session sans fait : enveloppe v1 identité seule (5 champs camelCase, pas de permissions) | `{"keys": ["environmentId", "providerInstanceId", "providerSessionId", "threadId", "version"], "version": 1}` |
| S1.4 | PASS | REEL | bridget_who via T3 réel résout A et B sur leurs propres agents (identité148) | `{"aErr": false, "bErr": false}` |
| S1.5 | PASS | REEL | chaque attestation ouvre puis ferme sa session de transport HTTP (DELETE par session, statuts 2xx) | `{"posts": 9, "deletes": 3, "statuses": [200, 202, 204]}` |
| S2.1 | PASS | REEL | G-P-07(c) : admission inherit avec la seule enveloppe v1 -> permission_attestation_unavailable | `{"text": "permission_attestation_unavailable", "code": "native_delegation_refused"}` |
| S2.2 | PASS | REEL | v1 + posture development : même refus nommé, aucune ligne, aucun lancement, aucune remise, aucun grant | `{"text": "permission_attestation_unavailable", "before": {"rows": 0, "started": 0, "prompts": 0, "answered": 0, "deliveries": {"A": 0, "B": 0}}, "after": {"rows": 0, "started": 0, "prompts": 0, "answered": 0, "deliveries": {"A": 0, "B": 0}}}` |
| S2.3 | PASS | REEL | status/identité148 restent fonctionnels en v1 (tâche inconnue -> task_unavailable, pas de crash) | `{"payload": {"text": "task_unavailable"}}` |
| S3.1 | PASS | REEL | bridget_session avec fait publié : enveloppe v2 stricte (identité camelCase + permissions snake_case, revision 1) | `{"v2keys": ["environmentId", "permissions", "providerInstanceId", "providerSessionId", "threadId", "version"], "permKeys": ["cwd", "driver", "interaction_mode", "provider_instance_id", "provider_policy", "provider_session_id", "revision", "run_id", "runtime_mode", "source", "version"], "revision": 1` |
| S3.2 | PASS | REEL | l'enveloppe v2 ne contient ni token, ni endpoint, ni en-tête d'autorisation (<= 64 KiB) | `{"bytes": 1455}` |
| S3.3 | PASS | REEL | le fait de A n'est jamais prêté à B : B (même processus fournisseur) reste en v1 | `{"version": 1}` |
| S3.4 | PASS | REEL | A (v2 full-access) : bridget_delegate sans posture admis, posture effective development, 1 ligne durable | `{"status": "queued", "posture": "development"}` |
| S3.5 | PASS | REEL | le moteur a lancé UN processus enfant réel et UN tour ; résultat corrélé disponible | `{"state": "result_available", "pid": 75063, "result": "fixture149-answer:a1"}` |
| S3.6 | PASS | REEL | la politique de l'enfant est celle du fait parent (dangerFullAccess + approval never), figée dans le snapshot | `{"sandbox": {"type": "dangerFullAccess"}, "approval": "never", "snapshotKeys": ["version", "source", "owner_agent_id", "owner_instance_id", "parent", "child_policy", "project_cwd"]}` |
| S3.7 | PASS | REEL | exactement une remise corrélée (in_reply_to = mission, from = enfant, to = A), ACK réel, rien chez B | `{"aDeliveries": 1, "matching": 1, "bDeliveries": 0, "acks": 1}` |
| S3.8 | PASS | REEL | B sans fait : refus nommé, A inchangé (aucun emprunt du fait de A) | `{"text": "permission_attestation_unavailable", "rows": 1}` |
| S3.9 | PASS | REEL | S149-02/25 : B lecteur (sandbox readOnly) demande development -> permission_not_inherited, aucune ligne, aucun lancement, aucune invitation au grant | `{"text": "permission_not_inherited", "rows": 1, "started": 1, "revB": 1}` |
| S3.10 | PASS | REEL | B (readOnly) sans posture : admis en lecture ; l'enfant de B reçoit un sandbox readOnly, celui de A est resté dangerFullAccess (S149-25) | `{"posture": "discovery", "sandboxB": {"type": "readOnly"}}` |
| S3.11 | PASS | REEL | B ne lit ni n'annule la tâche de A (task_unavailable, aucune fuite de champ) | `{"status": {"text": "task_unavailable"}, "cancel": {"text": "task_unavailable"}}` |
| S3.12 | PASS | REEL | B rejoue le request_id de A : jamais la tâche de A (soit refus, soit tâche distincte propre à B) | `{"isError": false, "sameTask": false, "rowsBefore": 2, "rowsAfter": 3}` |
| S4.1 | PASS | REEL | rotation : l'ancien credential est refusé par T3 (HTTP 401) et par Rust (t3_session_unavailable), sans repli | `{"http": 401, "code": "t3_session_unavailable"}` |
| S4.2 | PASS | REEL | le nouveau credential retrouve la même identité Bridget (stable) mais aucun fait hérité : v1 | `{"version": 1, "sameAgent": true}` |
| S4.3 | PASS | REEL | après rotation sans fait : nouvelle admission refusée (v1) mais lecture de la tâche admise intacte | `{"refusal": "permission_attestation_unavailable", "status": "result_available"}` |
| S4.4 | PASS | REEL | révocation de B : HTTP 401 + t3_session_unavailable ; le voisin A (nouveau credential) reste servi | `{"http": 401, "code": "t3_session_unavailable", "neighborOk": true}` |
| S4.5 | PASS | REEL | identités fausses : token forgé, endpoint hors runtime, credential valide sans binding Bridget -> t3_session_unavailable, jamais de repli PID | `{"forged": "t3_session_unavailable", "wrongPort": "t3_session_unavailable", "unboundC": "t3_session_unavailable", "delegateC": "t3_session_unavailable"}` |
| S5.1 | PASS | REEL | G-P-07(b) côté T3 : fait retiré (fin de run) -> refus nommé permission_attestation_unavailable, jamais l'enveloppe v1 | `{"rawIsError": true, "namedCode": "permission_attestation_unavailable", "hasStructured": false}` |
| S5.1b | PASS | REEL | côté Rust (montage MCP T3) : même credential -> admission refusée fermée, zéro ligne, zéro lancement | `{"delegate": "t3_session_unavailable", "status": "t3_session_unavailable"}` |
| S5.2 | PASS | REEL | fait lié à un autre run que le run actif : refus nommé ; la revision a augmenté à chaque publication du même credential | `{"rev3": 2, "stale": 3, "code": "permission_attestation_unavailable"}` |
| S5.3 | PASS | REEL | changement de mode (plan) : nouvelle revision pour les futures admissions ; le snapshot de la tâche déjà admise est inchangé (invariant 5) | `{"revision": 4, "snapshotUnchanged": true}` |
| S6.1 | PASS | REEL | S149-19 : A voit seulement sa tâche, B (même projet, même processus) seulement la sienne ; aucune ne voit l'autre | `{"a": 1, "b": 2}` |
| S6.2 | PASS | REEL | B ne peut ni show, ni journal, ni cancel la tâche de A via la CLI lecteur (task_unavailable, exit 2) | `{"show": "task_unavailable", "journal": "task_unavailable", "cancel": "task_unavailable"}` |
| S6.3 | PASS | REEL | mauvaise racine -> project_mismatch ; fil inconnu et fil sans binding -> binding_unavailable ; aucune donnée rendue | `{"wrongRoot": "project_mismatch", "unknown": "binding_unavailable", "unbound": "binding_unavailable"}` |
| S6.4 | PASS | REEL | show de la tâche terminale par son propre fil : résultat exact servi | `{"code": 0, "result": "fixture149-answer:a1"}` |
| S6.5 | PASS | REEL | 150 lectures Lineage : aucun lancement, aucune remise, aucun tour, base byte-stable (lecture quiète, aucun provider turn) | `{"quietBefore": {"rows": 3, "started": 3, "prompts": 3, "answered": 3, "deliveries": {"A": 1, "B": 2}, "db": "1559ac2145d08190"}, "quietAfter": {"rows": 3, "started": 3, "prompts": 3, "answered": 3, "deliveries": {"A": 1, "B": 2}, "db": "1559ac2145d08190"}}` |
| S6.9 | PASS | REEL | pagination en snapshot : pages de 1 tâche, même génération et même seq, union = liste complète, aucun doublon | `{"pages": 4, "tasks": 4, "seq": 36}` |
| S6.10 | PASS | REEL | mutation entre deux pages : page 2 rend snapshot_changed (le staging est à abandonner), jamais un snapshot mélangé | `{"mutated": true, "page2": "snapshot_changed"}` |
| S6.11 | PASS | REEL | grammaire CLI fermée : formes ouvertes ou hors bornes -> exit 2, invalid_request, retryable false | `{"codes": [[2, "invalid_request"], [2, "invalid_request"], [2, "invalid_request"], [2, "invalid_request"], [2, "invalid_request"], [2, "invalid_request"]]}` |
| S6.6 | PASS | REEL | watch : ready seq 0 premier pour chaque racine | `{"a": {"generation": "e06acf48-b87b-4c11-b85c-6500ac7f8a0b", "seq": 0, "status": "ready", "version": 1}, "b": {"generation": "e06acf48-b87b-4c11-b85c-6500ac7f8a0b", "seq": 0, "status": "ready", "version": 1}}` |
| S6.7 | PASS | REEL | mutation de la racine de A (nouvelle tâche) : A est signalé (corps absent), B ne reçoit rien (aucun UUID de fil ni contenu) | `{"aLines": ["ready", "changed", "changed"], "bLines": 1}` |
| S6.8 | PASS | REEL | cancel CLI : reçu natif, rejeu identique = même reçu, même request_id sur une autre tâche = envelope_mismatch, tâche réellement cancelled | `{"first": {"status": "cancelling", "task_id": "cbb6caad-9407-4139-a426-81341d2bef8f", "version": 1}, "replay": {"status": "cancelling", "task_id": "cbb6caad-9407-4139-a426-81341d2bef8f", "version": 1}, "mismatch": "envelope_mismatch", "state": "cancelled"}` |
| S7.1 | PASS | REEL | aucune autorité en trop : parent, permissions, sandbox, endpoint, credential, proof, agent_id forgés dans les arguments -> 7 refus (arguments fermés), zéro ligne, zéro lancement | `{"refused": 7, "of": 7, "rows": [8, 8], "messages": {"parent": "champ inconnu", "permissions": "champ inconnu", "sandbox": "champ inconnu", "endpoint": "champ inconnu", "credential": "champ inconnu", "proof": "champ inconnu", "agent_id": "champ inconnu"}}` |
| S7.2 | PASS | REEL | projet attesté : cwd hors projet (/tmp), autre dépôt git, lien symbolique sortant, .. -> refus d'admission avant tout lancement | `{"tmp": "cwd_outside_parent_project", "otherRepo": "cwd_outside_parent_project", "symlink": "cwd_outside_parent_project", "dotdot": "cwd_outside_parent_project"}` |
| S7.3 | PASS | REEL | conversation D d'un AUTRE projet : ne peut pas lancer dans le projet de A (refus), lance dans le sien ; sa liste Lineage ne contient que sa tâche ; racines croisées -> project_mismatch ; tâche de A introuvable | `{"intoOtherProject": "cwd_outside_parent_project", "ownProject": "queued", "listD": 1, "dWrongRoot": "project_mismatch", "aWrongRoot": "project_mismatch", "dShowA": "task_unavailable"}` |

### T036 partie 2 - serveur T3 complet + daemon natif (15)

| Id | Résultat | Couche | Contrôle | Observé |
|---|---|---|---|---|
| V0 | PASS | REEL | serveur T3 réel démarré sur base privée, projet + deux conversations créés par RPC | `{"results": [200, "ok", "ok"]}` |
| V1 | PASS | REEL | A : racine + enfant imbriqué (lien parent_task_id), modèle/protocole/posture du moteur ; la racine attend ses enfants (waiting_for_children), résultat retenu | `{"rootStatus": "waiting_for_children", "nestedStatus": "working", "tasks": 2}` |
| V2 | PASS | REEL | S149-19 via serveur T3 : B (même projet) ne voit que sa tâche, jamais celles de A | `{"b": ["b630e8be"]}` |
| V3 | PASS | REEL | B ne peut ni show, ni journal, ni cancel la tâche de A (task_unavailable) ; token read ne peut pas cancel | `{"show": "task_unavailable", "journal": "task_unavailable", "cancel": "task_unavailable"}` |
| V3b | PASS | REEL | scope : le jeton orchestration:read est refusé sur cancel (aucun effet natif) | `{"error": "EnvironmentAuthorizationError"}` |
| V4 | PASS | REEL | projection T3 : un fil virtuel marqué par tâche native (3, relation subagent) ; l'ensemble des conversations de premier niveau est identique avant/après lecture (aucune conversation créée par la lecture) | `{"virtual": 3, "topLevelBefore": 3, "topLevelAfter": 3, "topTitles": ["New thread", "Conversation A", "Conversation B"]}` |
| V5 | PASS | REEL | zéro effet fournisseur côté T3 : 0 run, 0 tentative, 0 tour, 0 session, 0 effet en attente, 0 message | `{"projection_runs": 0, "projection_run_attempts": 0, "projection_provider_turns": 0, "projection_provider_sessions": 0, "projection_provider_threads": 0, "projection_provider_session_bindings": 0, "projection_runtime_requests": 0, "effect_outbox": 0, "thread_launch_workflows": 0, "projection_thread_` |
| V6 | PASS | REEL | tant que l'enfant imbriqué travaille, la racine n'expose aucun résultat (retenu) ; l'enfant lui-même n'a pas de résultat | `{"rootResult": null, "nestedResult": null}` |
| V7 | PASS | REEL | journal réel de l'enfant actif lisible via T3 (événements du daemon, séquence ordonnée) | `{"events": 5, "gap": null}` |
| V8 | PASS | REEL | bridget.lineage.watch via T3 : ready seq 0 premier, puis changed sans corps ni UUID après nouvelle tâche réelle | `{"statuses": ["ready", "changed"], "keys": ["version", "generation", "seq", "status"]}` |
| V9 | PASS | REEL | cancel via T3 : reçu natif, rejeu identique, request_id sur autre tâche = envelope_mismatch ; tâche cancelled et PID enfant réellement terminé | `{"receipt": {"version": 1, "task_id": "7bdbcd86-9b0a-42d0-b743-975dbce4a50c", "status": "cancelling"}, "replaySame": true, "mismatch": "envelope_mismatch", "state": "cancelled", "childPid": 81443, "alive": false}` |
| V10 | PASS | REEL | l'annulation depuis T3 ne crée aucun run/tour/session fournisseur côté T3 (tables toujours à zéro) | `{"projection_runs": 0, "projection_run_attempts": 0, "projection_provider_turns": 0, "projection_provider_sessions": 0, "projection_provider_threads": 0, "projection_provider_session_bindings": 0, "projection_runtime_requests": 0, "effect_outbox": 0, "thread_launch_workflows": 0, "projection_thread_` |
| V11 | PASS | REEL | 110 lectures T3 -> daemon : aucun lancement, aucun tour, aucune mutation (états et updated_at identiques) | `{"before": {"rows": 4, "started": 4, "prompts": 4, "answered": 2, "deliveries": {}}, "after": {"rows": 4, "started": 4, "prompts": 4, "answered": 2, "deliveries": {}}}` |
| V12 | PASS | REEL | daemon coupé : T3 refuse explicitement (aucun contenu présenté comme courant) | `{"error": "binding_unavailable", "message": "Bridget lineage is unavailable (binding_unavailable)."}` |
| V13 | PASS | REEL | reprise sans relance : même génération native, mêmes tâches, aucun fil virtuel en double côté T3 | `{"sameGeneration": true, "tasksBefore": 2, "tasksAfter": 3, "virtualThreadsAfter": 4}` |

### T036 partie 3 - instance T3 unique (7)

| Id | Résultat | Couche | Contrôle | Observé |
|---|---|---|---|---|
| U0 | PASS | REEL | une seule instance T3 (bin.ts) : la session Codex réelle de T3 (ProviderSessionManager + adaptateur V2) reçoit un credential MCP émis PAR CE SERVEUR - l'URL MCP de thread/start est l'origine même du serveur qui sert Lineage - et le tour démarre (turn/start) | `{"dispatch": "ok", "endpoint": "http://127.0.0.1:15736/mcp", "sameOriginAsLineage": true, "methods": ["initialize", "initialized", "account/read", "skills/list", "model/list", "account/rateLimits/read", "initialize", "initialized", "config/read", "thread/start", "turn/start"], "spawnedPeers": 2}` |
| U1 | PASS | REEL | bridget_session v2 sur le serveur réel : le fait de permissions est celui publié par l'adaptateur Codex RÉEL (driver codex_app_server, cwd = racine du projet, mode full-access du fil) - la recette n'a rien forgé | `{"version": 2, "keys": ["environmentId", "permissions", "providerInstanceId", "providerSessionId", "threadId", "version"], "permissions": {"keys": ["cwd", "driver", "interaction_mode", "provider_instance_id", "provider_policy", "provider_session_id", "revision", "run_id", "runtime_mode", "source", "` |
| U2 | PASS | REEL | le credential émis par T3 pilote le daemon natif : identité résolue (bridget_who), admission héritée en full-access sans refus, enfant lancé (PID compté), résultat remis une fois au parent | `{"whoErr": false, "d1": "queued", "d2": "queued", "state": "result_available", "deliveries": 1}` |
| U3 | PASS | REEL | Lineage servi par la MÊME instance T3 : la racine (en attente), son enfant imbriqué et la tâche terminée sont listés avec modèle, protocole et posture issus du vrai fait ; rien d'un autre fil | `{"tasks": 3, "root": "waiting_for_children", "nested": "working", "posture": "development"}` |
| U4 | PASS | REEL | projection T3 : le parent a UN vrai run (créé par T3) ; les fils enfants Bridget sont des fils virtuels sans aucun run ; une seule session fournisseur (celle du parent) | `{"runs": [["89000000", "running"]], "virtual": 3, "providerSessions": 1}` |
| U5 | PASS | REEL | cancel Lineage depuis la même instance T3 : reçu natif, racine + enfant imbriqué cancelled, PID enfant réellement terminé ; le run du parent côté T3 n'a pas changé | `{"receipt": {"version": 1, "task_id": "a82bdb4d-53f0-412b-bbc3-83a96747e98e", "status": "cancelling"}, "rootState": "cancelled", "nestedPid": 83899, "nestedAlive": false, "runsUnchanged": true}` |
| U6 | PASS | REEL | interruption du run par T3 (vrai arrêt de session) : le credential émis ne donne plus d'identité ni d'admission (refus nommé), aucun enfant lancé pour la requête suivante | `{"interrupt": "ok", "sessionAfter": "200:parent_not_active", "delegateAfter": "t3_session_unavailable", "u3Prompts": 0, "runsStatusAfter": [{"status": "interrupted"}]}` |

## Ce que prouve l'instance unique (levée de la réserve r1)

- Un seul processus `bin.ts` (port 15736) émet le credential MCP au démarrage de la session Codex (`thread/start` porte l'URL `http://127.0.0.1:15736/mcp`) ET sert `bridget.lineage.read` / `bridget.lineage.cancel` (U0, U3, U5).
- Le fait de permissions vu par `bridget_session` v2 vient de l'adaptateur Codex réel (driver `codex_app_server`, mode `full-access` du fil, sandbox `dangerFullAccess`, cwd = racine du projet) : la recette ne publie rien (U1).
- L'enveloppe v2 a exactement 6 champs de premier niveau (`environmentId`, `permissions`, `providerInstanceId`, `providerSessionId`, `threadId`, `version`) et 11 champs de permissions. Ce sont les champs du contrat.
- Le parent a UN vrai run côté T3. Les trois fils enfants Bridget sont virtuels, sans run, et il n'y a qu'une session fournisseur (U4).
- Le cancel Lineage depuis T3 arrête les PID des enfants et ne change pas le run du parent (U5).
- Après `run.interrupt` (avec `holdQueue`), le vrai arrêt de session par T3 ferme le credential : `bridget_session` rend `parent_not_active`, l'admission suivante rend `t3_session_unavailable`, aucun enfant n'est lancé (U6). Attention : la première tentative a conclu trop vite (3 s) ; avec une attente bornée de 20 s sur l'état du run, le résultat est stable.

## Limites et non prouvé

- Aucun modèle réel (propriétaires T037/T038). Le faux pair Codex ne prouve ni la politique de sandbox du vrai Codex ni `can_use_tool` (G-P-01).
- Le wrapper du fil T3 côté daemon reste simulé : le binding Bridget d'un fil T3 vient normalement de la CLI ou du connecteur du fil.
- Les adaptateurs Claude de T3 ne sont pas exercés ici ; seul l'adaptateur Codex V2 l'est.
- Le rendu navigateur et la coque desktop ne sont pas dans ce périmètre.
- Le binaire r6 (reçu canonique actuel) n'a pas été rejoué : consigne de tester le binaire r5 immuable.
- Les chemins de ports : 14776 (hôte MCP simulé) et 15736 (serveur T3) ; 14777 et 15737 (UI r5) n'ont pas été touchés.
