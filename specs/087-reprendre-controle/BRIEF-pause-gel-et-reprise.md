# Brief d'exécution — SPEC-087, lot « pause = gel immédiat, reprise à la levée » + 2 corrections

Écrit le 2026-09-03 à 05:40 CEST pour un agent qui reprend sans le contexte de la conversation. Tout ce qu'il faut savoir est ici ou dans les fichiers pointés. Lire ce brief en entier avant le premier geste. Les chemins sont absolus. Les numéros de ligne datent du commit `a931a8ac` de `main` ; si un fichier a bougé, chercher le nom de la fonction, pas la ligne.

## 0. Décision du référent (le POURQUOI, ne pas rouvrir)

Le lot 087 est fusionné dans `main` et déployé sur le serveur. Sa spec disait : « la pause n'interrompt pas les tours commencés ». Le référent a tranché le 2026-09-03 : **quand il met en pause, les agents s'arrêtent tout de suite, et quand il lève la pause, ils reprennent ce qu'ils faisaient.** Rien ne doit être perdu.

Concrètement, la pause doit désormais :

1. **À l'activation** : interrompre le tour en cours de chaque agent, avec le mécanisme d'interruption qui existe déjà (SPEC-063), et mémoriser quelles exécutions ont été interrompues par la pause.
2. **Pendant la pause** : aucune reprise, aucun tour nouveau. C'est déjà codé (garde `admit_autonomous_effect`), ne pas y toucher.
3. **À la levée** : redonner à chaque agent connecté et libre l'exécution que la pause lui avait retirée, sans attendre qu'il se reconnecte. Un agent absent la retrouvera à sa prochaine reconnexion.

Deux corrections indépendantes s'ajoutent, trouvées à la relecture du lot :

- **A.** Le fichier de test `plugins/maicie/tests/controle_referent_087.rs` ne compile plus (5 appels avec une signature périmée). La tâche T043 était cochée à tort.
- **B.** La base gelée d'un focus est lue sur `refs/remotes/origin/HEAD` sans jamais rapatrier le remote : si personne n'a fait `git fetch`, le focus part d'une base périmée.

## 1. Règles absolues (une seule violation = travail refusé)

- **Git sans aucune trace d'IA.** Messages de commit au format strict `type(scope): Description`, rien d'autre. Jamais de `Co-Authored-By`, jamais « Generated with », jamais d'emoji robot, jamais le mot Claude/GPT/IA/LLM. Ni dans les commits, ni dans les fichiers de spec, ni dans les commentaires de code. Ne jamais modifier `git config user.name` ou `user.email`.
- **Un seul écrivain sur le worktree serveur.** Tu édites en local, tu envoies par `rsync`. Personne d'autre ne travaille sur ce worktree pendant ton lot.
- **Jamais `kill -9`, jamais tuer plusieurs processus en une commande, jamais tuer Firefox.** Avant tout `kill <PID>` : `ps -p <PID> -o pid,cmd`, vérifier, tuer un seul PID, attendre 3 s, revérifier.
- **Jamais de suppression par motif (`rm *foo*`) sans un `ls` du motif juste avant.**
- **Ne jamais lire la base de production directement** (`~/.cache/bridget/*.db` sur le serveur). Si tu dois regarder, copie d'abord (`cp` puis `sqlite3` sur la copie).
- **Ne jamais redémarrer le daemon sans y être invité.** Ce brief t'y invite UNE fois, à l'étape 8, après fusion dans `main`.
- **Compter les tests, jamais lire un code retour.** Toute annonce de gate donne la ligne `test result: ok. N passed; M failed` copiée telle quelle. Un `0 passed` n'est pas un vert. Jamais de `$?` derrière un pipe (`| tail`, `| grep`) : le code retour est celui du dernier programme du pipe.
- **Règle des deux tentatives.** Si deux corrections successives échouent sur la même hypothèse, arrête d'écrire du code : ajoute un `eprintln!`/`log` qui affiche les données réelles (état SQLite, trame reçue), regarde, et seulement ensuite reprends.
- **Pas de scope en plus.** Ce brief est le périmètre. Une idée en plus se note dans le rapport final, ne se code pas.

## 2. Où est le code, comment on travaille

### Dépôts

| Rôle | Chemin |
|---|---|
| Édition locale (le seul endroit où tu édites) | `/Users/moi/Nextcloud/10.Scripts/bridget` (branche à créer, voir §3) |
| Serveur, checkout principal (compilation release, services) | `cartae.app:/home/moi/bridget-referent/bridget` (`main`) |
| Serveur, worktree du lot (compilation et tests) | `cartae.app:/home/moi/bridget-referent/.worktrees/087b-pause-gel` (à créer, §3) |
| Cache de compilation partagé sur le serveur | `CARGO_TARGET_DIR=/home/moi/bridget-referent/bridget/target` |

Le serveur est la machine de compilation. Ne compile pas en local le daemon Rust : c'est lent et le cache est sur le serveur. Les tests d'interface (Node) se lancent en local.

### Cycle édition → test (à répéter à chaque changement)

```bash
# 1. envoyer le local vers le worktree serveur
rsync -az --exclude target --exclude .git --exclude .worktrees --exclude .claude --exclude .gstack --exclude 'watch_*' --exclude node_modules \
  /Users/moi/Nextcloud/10.Scripts/bridget/ cartae.app:/home/moi/bridget-referent/.worktrees/087b-pause-gel/

# 2. sur le serveur : formater puis tester
ssh cartae.app 'export PATH=$HOME/.cargo/bin:$PATH CARGO_TARGET_DIR=/home/moi/bridget-referent/bridget/target; cd /home/moi/bridget-referent/.worktrees/087b-pause-gel && cargo fmt -p bridget-transport -p bridget-daemon -p maicie && <commande de test>'

# 3. rapatrier le formatage (sinon le prochain rsync l'écrase)
rsync -az cartae.app:/home/moi/bridget-referent/.worktrees/087b-pause-gel/crates/ /Users/moi/Nextcloud/10.Scripts/bridget/crates/
rsync -az cartae.app:/home/moi/bridget-referent/.worktrees/087b-pause-gel/plugins/ /Users/moi/Nextcloud/10.Scripts/bridget/plugins/
```

`cargo` n'est pas dans le PATH par défaut sur le serveur : l'`export PATH=$HOME/.cargo/bin:$PATH` est obligatoire dans chaque commande ssh.

### Pièges connus (ne pas les redécouvrir)

1. Si la compilation dit `unresolved import` ou `no field named …` sur un symbole qui existe bien dans le fichier : artefact de cache périmé. `cargo clean -p bridget-transport -p maicie -p bridget-daemon` puis recompiler.
2. `cargo test --workspace` nu est rouge par construction. Le daemon exige `--features test-support`. Deux tests d'intégration du daemon (`mcp_injection_smoke_test`, `idempotency_crash_test`) ne compilent plus depuis un lot antérieur (champ `name`) : c'est connu, ce n'est pas à toi de le corriger, ne les lance pas.
3. La suite `--lib` du daemon a environ 144 tests rouges qui préexistent au lot 087 (fixture `state_with_registered_agent` cassée depuis la session 081). La liste de leurs NOMS est sur le serveur dans `/tmp/baseu.txt` (146 noms ; si le fichier a disparu, refais la base sur `main` nu AVANT tes modifications : voir §7). Un gate est vert si l'ensemble des noms rouges après tes changements est inclus dans cette base. Compare les noms, jamais les totaux.
4. Le fichier `crates/bridget-daemon/src/daemon.rs` fait plus de 23 000 lignes. Ne le lis jamais en entier ; utilise `grep -n` puis `sed -n a,bp`.
5. Deux matrices de rôles exhaustives dans `handle_wrapper_message` (`daemon.rs`, vers les lignes 9354 à 9420) : si tu ajoutes une trame au protocole, elle doit être classée dans les deux, sinon rien ne compile. Ce lot n'ajoute aucune trame : si tu crois en avoir besoin, tu t'es trompé de chemin.
6. `ClientHello` et `ServiceHello` filtrent les capacités par liste en dur. Ce lot n'ajoute pas de capacité.
7. Le fichier de test 087 du daemon utilise la fixture `spec_087_state` (`daemon.rs:23218`), agent enregistré sous l'UUID `SPEC_087_AGENT_ID`. Utilise-la, pas `state_with_registered_agent`.
8. Le disque du serveur a déjà été plein. Avant de partir : `df -h /home` ; ne crée pas de second `target`.

## 3. Premier geste : branche et worktree

```bash
cd /Users/moi/Nextcloud/10.Scripts/bridget
git status --short          # doit ne montrer que .claude/ .gstack/ watch_* (non suivis) ; sinon STOP et le dire
git fetch origin && git checkout -b 087b-pause-gel origin/main
git push -u origin 087b-pause-gel

ssh cartae.app 'cd /home/moi/bridget-referent/bridget && git fetch origin && git worktree add /home/moi/bridget-referent/.worktrees/087b-pause-gel origin/087b-pause-gel && cd /home/moi/bridget-referent/.worktrees/087b-pause-gel && git checkout -B 087b-pause-gel origin/087b-pause-gel'
```

Ensuite, à chaque étape terminée : un commit local, un `git push`, et sur le serveur le worktree se remet à jour par le rsync (le `.git` du worktree n'a pas besoin d'être à jour pour compiler ; il servira seulement au merge final depuis le checkout principal).

## 4. Lot A — remettre le test Maicie en état (20 à 30 min)

**Symptôme** : `cargo test -p maicie --test controle_referent_087` ne compile pas, 5 erreurs, toutes « argument manquant ».

**Cause** : deux signatures ont gagné un paramètre `project_id: &str` après l'écriture du test.

- `plugins/maicie/src/store.rs:8897` : `pub fn focus_enqueue(&mut self, objective_id: Uuid, project_id: &str, replace: bool, now: i64)`
- `plugins/maicie/src/app.rs:913` : `pub fn open_focus_waiting_for_agent(store, request, human_message_id, project_id: &str, replace)`

**Appels à corriger** dans `plugins/maicie/tests/controle_referent_087.rs` :

| Ligne | Aujourd'hui | Attendu |
|---|---|---|
| 265 | `store.focus_enqueue(objective_id, true, 1_000)` | `store.focus_enqueue(objective_id, "<project_id du test>", true, 1_000)` |
| 285 | `store.focus_enqueue(a, true, 1_000)` | idem, avec le projet |
| 286 | `store.focus_enqueue(b, false, 1_001)` | idem |
| 293 | `store.focus_enqueue(c, true, 1_002)` | idem |
| 330 | `open_focus_waiting_for_agent(&mut store, &request, "hmo-focus-waiting", true)` | ajouter `"<project_id>"` avant `true` |

Le `project_id` à passer est celui que le test utilise déjà pour son projet (cherche dans le même fichier la constante ou la chaîne de projet des objectifs créés plus haut ; s'il n'y en a pas, prends celui de la requête `request` du test 330, ou une chaîne stable comme `"projet-test-087"` utilisée partout dans le fichier). Lis d'abord ce que `focus_enqueue` fait du `project_id` (store.rs à partir de 8897) pour choisir une valeur cohérente avec les assertions du test.

**Preuve** :

```bash
cargo test -p maicie --test controle_referent_087 2>&1 | tail -30
```

Attendu : compile, ligne `test result: ok. N passed; 0 failed` avec N > 0. Note N dans le rapport. Si des tests échouent pour une autre raison qu'une signature, lis l'assertion et corrige LE TEST seulement si l'assertion est devenue fausse par le changement de signature ; sinon, c'est un vrai défaut, note-le et STOP sur ce lot.

Commit : `fix(087): Realigner le test controle_referent_087`.

## 5. Lot B — rapatrier `origin` avant de geler la base d'un focus (30 à 45 min)

**Fichier** : `plugins/maicie/src/review_git.rs`, fonction `freeze_origin_default_review_target` (ligne 312). Son commentaire dit « ne contacte jamais le réseau : la référence doit avoir été rapatriée avant la demande de focus ». Personne ne la rapatrie. Résultat : un focus peut geler une base vieille de plusieurs jours.

**Changement** : avant la lecture de `refs/remotes/origin/HEAD`, exécuter un `git fetch` borné :

```text
git fetch --quiet --no-tags --prune origin
```

en réutilisant `git_capture` (même fichier), qui est déjà le seul moyen d'appeler git ici. Regarde sa signature et si elle permet un délai maximum ; si elle n'en a pas, ajoute la variable d'environnement `GIT_HTTP_LOW_SPEED_LIMIT=1000` / `GIT_HTTP_LOW_SPEED_TIME=30` via le mécanisme d'environnement de `git_capture` s'il existe, sinon laisse tel quel et note-le dans le rapport (un fetch qui pend est rare, mais tu dois le dire).

**Politique en cas d'échec du fetch** : refuser le gel avec une erreur explicite. Ajoute une variante `ReviewGitError::OriginUnreachable` (regarde comment les variantes existantes sont affichées et mappées vers le refus du focus ; fais pareil). Une base périmée fait travailler l'agent sur le mauvais commit ; un refus clair vaut mieux.

**Mettre à jour le commentaire** de la fonction : il ment aujourd'hui.

**Test** (sans réseau) : dans `plugins/maicie/tests/` il existe déjà des tests git qui créent des dépôts temporaires (`rg -n "git init\|init_bare\|--bare" plugins/maicie/tests plugins/maicie/src/review_git.rs` pour trouver le helper). Écris un test qui :

1. crée un dépôt bare `origin` et un clone ;
2. dans un troisième clone, ajoute un commit et le pousse sur `origin` ;
3. appelle `freeze_origin_default_review_target` sur le premier clone (qui n'a PAS fait de fetch) ;
4. attend `expected_head` = le nouveau commit. Sans ton changement, ce test échoue (c'est ton mutant : vérifie-le une fois en commentant le fetch, puis remets-le).

Un second test : origin dont l'URL pointe vers un chemin inexistant → `OriginUnreachable`.

**Preuve** :

```bash
cargo test -p maicie review_git 2>&1 | tail -20
cargo test -p maicie --lib 2>&1 | tail -5     # attendu : 90 passed, 1 failed (rouge préexistant, nom à relever et comparer)
```

Commit : `fix(087): Rapatrier origin avant de geler la base du focus`.

## 6. Lot C — pause = gel immédiat, reprise à la levée (2 h 30 à 4 h)

### 6.1 Ce qui existe déjà (à réutiliser, pas à réécrire)

| Brique | Où | Ce qu'elle fait |
|---|---|---|
| Mutation de l'état de contrôle | `crates/bridget-daemon/src/referent_control.rs:222` `set(conn, ControlMutation)` | Écrit pause/plafond, journalise `control_events`, idempotent par `command_id`. Renvoie `Ok(Ok(state))` en cas de succès. |
| Bras daemon de la trame | `crates/bridget-daemon/src/daemon.rs:8796` `handle_control_state_set(...)` | Vérifie le principal humain, appelle `set`. **C'est ici que tu branches l'interruption (pause) et la relance (levée).** Note : `let st = state.lock()` n'est pas `mut` ; tu devras l'y passer. |
| Interruption d'un tour | `daemon.rs:8313` `handle_execution_control(issuer_scope, ExecutionControlCommand, &mut DaemonState)` | Vérifie génération/révision, refuse si l'exécution est terminale, réserve la commande (idempotente par `issuer_scope + command_id`), pousse `DaemonToWrapper::ControlExecutionDispatch` au wrapper de l'agent. Le wrapper (`wrapper.rs:4934`) annule la remise, publie `interrupting`, le fournisseur termine, l'état devient `interrupted`. |
| Structure de la commande | `crates/bridget-transport/src/protocol.rs:175` `ExecutionControlCommand { version: 1, command_id, execution_id, generation, revision, operation: ExecutionControlOperation::Interrupt, message: None }` | |
| Cible d'une exécution | `crates/bridget-daemon/src/execution_store.rs:1044` `execution_control_target(execution_id)` | Renvoie `target_agent` et `snapshot { state, generation, revision }`. |
| Liste des exécutions actives | `execution_store.rs` `recoverable_execution_ids()` (vers 1466) | États `queued, starting, running, waiting_approval, waiting_user_input, interrupting`. |
| Reprise d'un agent au réenregistrement | `daemon.rs:4104` `schedule_execution_recovery(state, conn_id, instance_id, agent_name, turn_in_progress)` | Cherche l'exécution active de l'agent (`recoverable_execution_ids_for_agent`, 1443), réserve une continuation gouvernée (`reserve_governed_continuation`, refusée `Budget(Paused)` en pause), reconstruit un enfant (`reconstruct_active_for_agent`, 1287), grave une remise idempotente et la met en file dans `state.pending_post_response_controls[conn_id]`. Appelée à `daemon.rs:6663` à chaque `Register`. |
| Réservation de continuation | `crates/bridget-daemon/src/managed_supervisor.rs:64` `reserve_governed_continuation` | La source `RecoveryAfterIdleWrapper` accepte un parent actif OU inactif (`execution_store.rs:943-960`) : un parent `interrupted` passe. |
| Présence des agents | `state.presences: HashMap<instance_id, …>` avec `name` (agent_id) et `busy_since: Option<Instant>` (`daemon.rs` vers 6627) ; `state.router.get_agent(agent_id)` donne `connection_id` (`crates/bridget-core/src/router.rs:156`) ; `state.router.list_agents()`. | |
| Identité du superviseur pour émettre des commandes internes | `state.idempotency.supervisor_scope()` (`idempotency.rs:483`), déjà utilisé par `schedule_execution_recovery` (ligne ~4262). | |
| Test existant de la garde en pause | `daemon.rs` test `spec_087_pause_differe_la_continuation_de_reprise` (vers 23140) | Montre comment : créer un état, une exécution `admit_starting_message`, poser la pause avec `referent_control::set`, appeler `schedule_execution_recovery`, lire `pending_post_response_controls`. Copie ce modèle. |

### 6.2 Le problème central à comprendre avant de coder

Quand la pause interrompt une exécution, elle finit dans l'état `interrupted`, qui est **terminal**. Or les deux requêtes de reprise (`recoverable_execution_ids_for_agent` et `reconstruct_active_for_agent`) ne regardent que les états actifs. Donc, sans rien d'autre, une exécution interrompue par la pause ne serait jamais reprise. Il faut :

1. **mémoriser** ce que la pause a interrompu (nouvelle table) ;
2. **élargir** la reconstruction à ces exécutions-là, et seulement à elles ;
3. **déclencher** la reprise à la levée pour les agents connectés et libres.

### 6.3 Étapes, dans l'ordre

**C1. Table `control_pause_interruptions`** dans `execution_store.rs` (même base que `executions`, pour que la reconstruction et le marquage « repris » soient dans une seule transaction). Migration suivante après la 9 (ligne 1570) : version 10.

```sql
CREATE TABLE IF NOT EXISTS control_pause_interruptions (
  execution_id       TEXT PRIMARY KEY REFERENCES executions(execution_id),
  target_agent       TEXT NOT NULL,
  control_generation INTEGER NOT NULL,   -- génération de control_state qui a posé la pause
  requested_at       INTEGER NOT NULL,
  resumed_at         INTEGER,            -- NULL tant que non repris
  resume_execution_id TEXT               -- l'enfant créé à la reprise
);
```

Fonctions à ajouter dans `impl ExecutionStore` (une par besoin, pas plus) :

- `record_pause_interruption(execution_id, target_agent, control_generation, now)` : `INSERT OR IGNORE`.
- `pending_pause_interruptions() -> Vec<(execution_id, target_agent)>` : `WHERE resumed_at IS NULL`.
- `ensure_schema` (ou l'équivalent existant) doit créer la table.

**C2. À la pause : interrompre.** Dans `handle_control_state_set`, après `Ok(Ok(state))`, si `paused == Some(true)` et que la pause vient d'être posée (compare avec l'état avant : `set` refuse `NothingToChange` si rien ne change, donc si tu es dans `Ok(Ok)` avec `paused: Some(true)`, la pause vient d'être posée ou la commande est rejouée ; pour le rejeu, l'idempotence de `handle_execution_control` par `command_id` te protège, mais lis `control_events` si tu veux être strict), appelle une nouvelle fonction dans `daemon.rs` :

```text
fn interrupt_executions_for_pause(st: &mut DaemonState, control_generation: u64)
```

qui, pour chaque id de `recoverable_execution_ids()` :

1. lit `execution_control_target(id)` ; ne garde que les états `running`, `waiting_approval`, `waiting_user_input` (un `queued`/`starting` n'a pas de tour à interrompre ; il reste actif et la reprise existante le retrouvera, gouvernée par la pause) ;
2. `record_pause_interruption(id, target_agent, control_generation, now)` AVANT d'envoyer l'interruption (si le daemon tombe entre les deux, il vaut mieux une ligne de trop qu'une exécution oubliée) ;
3. construit `ExecutionControlCommand { version: 1, command_id: format!("control-pause-{control_generation}-{id}"), execution_id: id, generation, revision, operation: Interrupt, message: None }` ;
4. appelle `handle_execution_control(&supervisor_scope, command, st)` et journalise le résultat avec `info!`/`warn!` (un `TargetUnavailable` n'est pas une erreur : l'agent est déconnecté, la ligne suffit).

Le `supervisor_scope` s'obtient par `st.idempotency.supervisor_scope()`, comme dans `schedule_execution_recovery`.

**C3. Élargir la reprise aux exécutions interrompues par la pause.** Dans `execution_store.rs` :

- `recoverable_execution_ids_for_agent` (1443) : ajouter au `WHERE` : `OR execution.execution_id IN (SELECT execution_id FROM control_pause_interruptions WHERE resumed_at IS NULL)` (en gardant `submission.target_agent = ?1`).
- `reconstruct_active_for_agent` (1287) : même élargissement du `SELECT` des candidats ; et l'`UPDATE` qui ferme le parent (`SET state = 'unreachable', reason = 'daemon_restart' … AND state IN (actifs)`) doit accepter aussi un parent `interrupted` ou `interrupting` présent dans la table, sinon `closed != 1` → erreur. Le plus simple : un `reason` différent selon le cas (`'control_resume'` au lieu de `'daemon_restart'`), et dans la même transaction `UPDATE control_pause_interruptions SET resumed_at = ?, resume_execution_id = ? WHERE execution_id = ?`. Pour un parent déjà `interrupted`, ne le passe pas en `unreachable` : laisse-le `interrupted`, mets seulement la ligne de pause à jour. Écris la ligne `execution_continuations` avec `reason = 'control_resume'`.
- NE PAS toucher `recoverable_execution_ids()` (sans agent) : elle sert au redémarrage du daemon pour réconcilier avec les fournisseurs, pas à rejouer du travail.

**C4. À la levée : relancer.** Dans `handle_control_state_set`, après `Ok(Ok(state))`, si `paused == Some(false)`, appeler :

```text
fn resume_executions_after_pause(st: &mut DaemonState)
```

qui, pour chaque `(execution_id, target_agent)` de `pending_pause_interruptions()` :

1. `st.router.get_agent(&target_agent)` → si absent : rien (la ligne reste, la reconnexion la reprendra par C3), `info!` ;
2. trouve la présence de cet agent dans `st.presences` (clé = instance_id, valeur avec `name == target_agent`) ; si `busy_since.is_some()` : rien, la ligne reste ;
3. sinon `schedule_execution_recovery(st, &connection_id, &instance_id, &target_agent, false)`.

`schedule_execution_recovery` relit l'état de contrôle : comme la pause vient d'être levée dans la même transaction `set`, elle est admise. Elle trouvera l'exécution grâce à C3.

Attention à un point : `schedule_execution_recovery` refuse si l'agent a plus d'une exécution candidate (`candidates.len() > 1`). Avec C3, un agent peut avoir une exécution interrompue par la pause ET une nouvelle exécution active (par exemple un message humain reçu pendant la pause). Dans ce cas la reprise est refusée avec `warn!` et la ligne reste. C'est acceptable pour ce lot ; note-le dans le rapport et dans `implementation.md`. Ne tente pas de le résoudre.

**C5. Remise en file après la réponse.** `schedule_execution_recovery` dépose la remise dans `pending_post_response_controls[conn_id]`, qui est vidée après la réponse à la trame en cours SUR CETTE CONNEXION. Or ici la trame en cours est celle du référent (connexion UI ou CLI), pas celle de l'agent. Vérifie comment `pending_post_response_controls` est vidée (`rg -n "pending_post_response_controls" crates/bridget-daemon/src/daemon.rs`) : si elle n'est vidée qu'à la fin du traitement de la connexion `conn_id` elle-même, la remise attendra la prochaine trame de l'agent (heartbeat), ce qui peut prendre des secondes. Si c'est le cas, après avoir appelé `schedule_execution_recovery`, pousse toi-même les contrôles en attente de `conn_id` avec le même mécanisme que celui qui les envoie (cherche l'endroit qui itère `pending_post_response_controls` et appelle `push_control_message`). Mesure avant de conclure : écris le test C6.b et regarde ce qu'il reçoit.

**C6. Tests d'effet (dans `daemon.rs`, module de tests 087, à côté de `spec_087_pause_differe_la_continuation_de_reprise`).** Pour chaque test, note dans le rapport le nom et « passed ».

- **a. `spec_087_pause_interrompt_le_tour_en_cours`** : agent enregistré (`spec_087_state`), socket de contrôle (`control_socket`) dans `state.connections["conn-1"]`, exécution admise puis passée en `running` avec `transition_if_current` (`execution_store.rs:1216`). Appeler `handle_control_state_set` par le chemin normal (`handle_wrapper_message` avec une connexion cliente déclarée principal humain ; regarde comment le test vers `daemon.rs:23630` fabrique une connexion `cap-client` avec `ControlStateV1`, puis comment le principal humain est reconnu : `control_client_actor`, et copie le test qui pose une pause avec succès s'il existe, sinon utilise le périmètre `bridget-ui-control` comme `ui.rs` le fait). Attendu : une trame `ControlExecutionDispatch { command: { operation: Interrupt, execution_id: … } }` lue sur `_reader` du socket, ET une ligne dans `control_pause_interruptions` avec `resumed_at IS NULL`.
- **b. `spec_087_levee_de_pause_relance_l_execution_interrompue`** : ligne de pause posée (via `record_pause_interruption`), parent en état `interrupted` (par `transition_if_current`), agent connecté, présence sans `busy_since`. Lever la pause par le même chemin. Attendu : `pending_post_response_controls["conn-1"]` contient un `DeliverIdempotent { execution: Some(e) }` avec `e.generation == parent + 1`, et la ligne de pause a `resumed_at` non nul. Si C5 a montré qu'il faut pousser directement, l'attendu est la trame lue sur le socket.
- **c. `spec_087_levee_de_pause_agent_occupe_ne_relance_pas`** : même chose avec `busy_since: Some(Instant::now())`. Attendu : rien en file, ligne intacte.
- **d. `spec_087_levee_de_pause_agent_absent_reprend_a_la_reconnexion`** : ligne de pause, agent NON enregistré. Lever la pause : rien. Puis simuler le `Register` de l'agent (ou appeler `schedule_execution_recovery` directement, comme le test existant) : attendu, remise en file et ligne marquée reprise.
- **e. Mutants, à jouer une fois puis à retirer** : (1) commenter l'appel `interrupt_executions_for_pause` → a doit échouer ; (2) commenter l'appel `resume_executions_after_pause` → b doit échouer ; (3) retirer l'élargissement `OR … IN (SELECT …)` de `recoverable_execution_ids_for_agent` → b et d doivent échouer. Écris dans `implementation.md` quel test chaque mutant a fait tomber. Un mutant qui ne fait rien tomber = un test qui ne garde rien : corrige le test.

**C7. Adapter le test existant** `spec_087_pause_differe_la_continuation_de_reprise` : il reste valable (sous pause, pas de reprise). Vérifie seulement qu'il passe encore.

### 6.4 Ce que ce lot ne fait PAS

- Il n'interrompt pas les exécutions `queued`/`starting` (rien à interrompre).
- Il ne résout pas le cas « deux candidats pour un agent » (C4, noté).
- Il n'ajoute ni trame, ni capacité, ni commande CLI.
- Il ne touche pas à Maicie (le versant Maicie de la pause est déjà correct : les routines et réassignations sont différées).

### 6.5 Incertitude à mesurer avant C2

L'interruption est déclarée par les trois fournisseurs (`crates/bridget-transport/tests/provider_contract_test.rs:40-43`), mais sur le serveur seul Codex a été vu interrompu en vrai. Avant de coder C2, sur le serveur, regarde `bridget who` (agents connectés et leur type) ; à l'étape 8 tu vérifieras l'interruption réelle sur un agent de chaque type présent. Si un type n'interrompt pas, la pause le laissera finir son tour et tu l'écriras noir sur blanc dans le rapport et dans `quickstart.md`.

## 7. Gates (à jouer tous, dans cet ordre, compter les tests)

```bash
export PATH=$HOME/.cargo/bin:$PATH CARGO_TARGET_DIR=/home/moi/bridget-referent/bridget/target
cd /home/moi/bridget-referent/.worktrees/087b-pause-gel

cargo fmt --all -- --check
cargo test -p bridget-transport 2>&1 | grep "test result"                       # attendu 242 passed (ou plus), 0 failed
cargo test -p maicie --lib 2>&1 | grep "test result"                             # attendu 90 passed, 1 failed (préexistant ; relever le NOM)
cargo test -p maicie --test controle_referent_087 2>&1 | grep "test result"      # attendu N passed, 0 failed
cargo test -p maicie --test human_origin_attestation 2>&1 | grep "test result"   # attendu 10 passed
cargo test -p bridget-daemon --features test-support --test execution_budget_test 2>&1 | grep "test result"
cargo test -p bridget-daemon --lib --features test-support 2>&1 | tee /tmp/lot087b.txt | grep "test result"
grep -E "^test .* FAILED$" /tmp/lot087b.txt | awk '{print $2}' | sort > /tmp/lot087b-rouges.txt
comm -13 /tmp/baseu.txt /tmp/lot087b-rouges.txt                                  # attendu : VIDE (aucun rouge nouveau)
cargo clippy -p bridget-daemon -p maicie --features test-support --all-targets 2>&1 | grep -E "^(warning|error)" | sort | uniq -c
cargo test --workspace --features test-support --no-run 2>&1 | grep -E "^error" # attendu : seulement les 2 tests d'intégration connus (champ name)
```

Si `/tmp/baseu.txt` n'existe plus : sur le checkout principal `main` (PAS ton worktree), joue `cargo test -p bridget-daemon --lib --features test-support` et produis la liste des noms rouges de la même façon dans `/tmp/baseu.txt`, AVANT de comparer.

Interface (en local) : `cd /Users/moi/Nextcloud/10.Scripts/bridget/crates/bridget-daemon/assets/ui && node --test app.js` → attendu 135 passés (ce lot ne touche pas l'interface ; c'est un contrôle de non-régression).

Le rapport donne la ligne `test result` brute de chaque commande.

## 8. Documentation à mettre à jour (le lot n'est pas fini sans ça)

Tous ces textes disent aujourd'hui que la pause n'interrompt pas les tours. Remplace par la nouvelle règle : « la pause interrompt le tour en cours de chaque agent ; les exécutions interrompues par la pause sont reprises à la levée pour les agents connectés et libres, ou à leur prochaine reconnexion ».

| Fichier | Endroit | Quoi |
|---|---|---|
| `specs/087-reprendre-controle/spec.md` | ligne 39 (« Les tours d'agents déjà commencés se terminent normalement ») ; ligne 48 (scénario 2 de la story pause) ; ligne 126 (cas limite remise en cours) ; ligne 201 (Assumptions) | Réécrire. Ajouter une exigence FR-002b : « la pause DOIT interrompre le tour en cours … et la levée DOIT reprendre … ». Ajouter un critère SC mesurable : « après levée, tout agent connecté et libre reçoit son exécution interrompue avant sa prochaine trame » (ou « dans la même seconde »). |
| `tests/features/087-reprendre-controle.feature` | scénario « La pause n'interrompt pas un tour commencé » (lignes 9-13) | Remplacer par deux scénarios : « La pause interrompt le tour en cours » et « La levée de la pause reprend le tour interrompu ». |
| `specs/087-reprendre-controle/research.md` | R8, ligne 72 (« Admis pendant la pause… la reprise… rejoue un travail accepté avant la pause, et la spec dit que les tours commencés se terminent ») et ajouter une ligne au tableau | Décision révisée le 2026-09-03 par le référent, avec la conception C1-C5 en trois phrases. |
| `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` | point 2, ligne 14 | Amender : la pause interrompt les tours ; nouvelle section « Amendement 2026-09-03 » avec contexte, décision, conséquences (une table de plus, une reprise de plus, le cas « deux candidats » non résolu). |
| `specs/087-reprendre-controle/contracts/control-state-v1.md` | effets de `ControlStateSet` | Ajouter : pause ⇒ commandes `Interrupt` internes `control-pause-<gen>-<execution>` ; levée ⇒ reprise. |
| `specs/087-reprendre-controle/quickstart.md` | §1 « Mettre en pause » | Étape : « un agent en plein tour est interrompu (visible dans `bridget who`, colonne état) ; à la levée il reçoit à nouveau son message ». Et la limite par type de fournisseur si mesurée (§6.5). |
| `docs/regles-chantier.md` | paragraphe SPEC-087 (vers ligne 433) | Une phrase sur le gel. |
| `specs/087-reprendre-controle/tasks.md` | fin, sous `## Convergence` | Ajouter T057 (lot A), T058 (lot B), T059 à T063 (C1 à C6) et T064 (doc), cochées quand faites, chacune avec fichier et preuve. T045 reste ouverte (validation opérateur par le référent). |
| `specs/087-reprendre-controle/implementation.md` | fin | Une section par lot : commit, fichiers, gates joués (lignes `test result`), mutants et test tombé, ce qui n'a PAS été vérifié. |

Commit doc : `docs(087): Pause = gel immediat et reprise a la levee`.

## 9. Commits, fusion, mise en service

Un commit par lot (A, B, C, doc), messages :

```text
fix(087): Realigner le test controle_referent_087
fix(087): Rapatrier origin avant de geler la base du focus
feat(087): Interrompre les tours a la pause et les reprendre a la levee
docs(087): Pause = gel immediat et reprise a la levee
```

Puis, seulement quand TOUS les gates du §7 sont verts :

```bash
# local
cd /Users/moi/Nextcloud/10.Scripts/bridget && git push
# serveur, checkout principal
ssh cartae.app 'cd /home/moi/bridget-referent/bridget && git fetch origin && git checkout main && git pull --ff-only && git merge --no-ff origin/087b-pause-gel -m "merge(087): Pause = gel immediat et reprise a la levee" && git push origin main'
```

Mise en service (procédure T046, déjà jouée une fois le 2026-09-03 à 03:12 UTC) :

1. `bridget agents --json > /tmp/agents-avant.json` (liste des agents gérés, tu en auras besoin pour les relancer).
2. Compilation release depuis le checkout principal : `cargo build --release -p bridget-daemon` avec le même `CARGO_TARGET_DIR`.
3. Binaire versionné : copier vers `~/.local/lib/bridget/bridget-<sha court>-<build id>` (regarde comment les précédents sont nommés : `ls -la ~/.local/lib/bridget/`), puis basculer le lien symbolique que `systemctl --user cat bridget-daemon` désigne. `ls -la` du lien avant et après.
4. `systemctl --user restart bridget-ui.service` puis `systemctl --user restart bridget-daemon.service`. C'est le SEUL redémarrage autorisé par ce brief.
5. Les agents gérés meurent avec le daemon (même cgroup). Relance chacun : `bridget relaunch <uuid>` pour chaque uuid de `/tmp/agents-avant.json`.
6. Vérifier : `bridget who` (ligne `Contrôle :` en pied, aucun avertissement `identity_version`), `journalctl --user -u bridget-daemon -n 50`.
7. **Essai réel** : avec un agent en plein tour (envoie-lui une tâche courte mais de plusieurs minutes), `bridget control pause --reason "essai gel"` ; attendu : l'agent passe en `interrupting` puis `interrupted` dans `bridget who`. Puis `bridget control resume` ; attendu : l'agent repart sur le même message (journal du daemon : `reprise` / `control_resume`). Faire l'essai sur un agent de chaque type présent (§6.5). Écrire le résultat par type dans le rapport et dans `quickstart.md`.

Ensuite supprimer le worktree serveur : `ssh cartae.app 'cd /home/moi/bridget-referent/bridget && lsof +D /home/moi/bridget-referent/.worktrees/087b-pause-gel | head ; git worktree remove /home/moi/bridget-referent/.worktrees/087b-pause-gel'` (si `lsof` montre un processus, ne supprime pas, dis-le).

## 10. Rapport final attendu (format)

```text
Statut : Implemented | In Progress | Blocked
Branche : 087b-pause-gel, fusionnée dans main à <sha> | non fusionnée
Lots : A <fait/pas fait>, B <…>, C <…>, doc <…>
Gates (lignes brutes) :
  transport : test result: …
  maicie lib : … (rouge préexistant : <nom>)
  controle_referent_087 : …
  daemon lib : … ; rouges nouveaux vs base : <liste ou "aucun">
  clippy : …
  no-run workspace : …
Mutants : (1) → <test tombé>, (2) → …, (3) → …
Essai réel : codex <interrompu/repris : oui/non>, claude <…>, autres <…>
Non vérifié : <liste honnête>
Cas connus laissés ouverts : deux candidats pour un agent (C4) ; <autres>
Prochaine tâche ouverte : T045 (validation opérateur avec le référent)
```

Un rapport sans les lignes `test result` brutes n'est pas recevable. Si un lot est bloqué, dis-le et donne la prochaine action exacte ; ne dis jamais « ça devrait marcher ».

## 11. Estimation

| Lot | Temps |
|---|---|
| §3 branche et worktree | 10 min |
| A | 20 à 30 min |
| B | 30 à 45 min |
| C (C1 à C7, mutants compris) | 2 h 30 à 4 h |
| Gates §7 complets | 20 à 30 min |
| Doc §8 | 25 à 40 min |
| Fusion, mise en service, essai réel §9 | 30 à 45 min |
| **Total** | **4 h 30 à 7 h** |

Hors estimation : T045 (le référent), et tout défaut préexistant découvert en chemin (à noter, pas à corriger).
