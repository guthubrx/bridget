# Smoke réel T037/T038 sur la release r8 - ronde Sonnet r6

Date : 2026-10-10, 18:59 à 19:10. Testeur : Claude Sonnet 5.5. Modèles réels : GLM `glm-5.3-flash`, Codex `gpt-6.1-sol` effort `high`.
Aucune case de `tasks.md` cochée. Aucun Git, Cargo, build, ni fichier de production modifié. Aucun service, aucune configuration ni base de production touchés. T3 non lancé.

## 1. Verdict

| Scénario | Verdict | Résumé |
|---|---|---|
| A - parent GLM PTY vers enfant GLM `glm-5.3-flash` | **PASS** | `result_available`, fichier écrit, une seule ligne, 0 grant, 17 sources héritées |
| B - SIGTERM du daemon pendant qu'un enfant Codex travaille (correctif F2/O4 de r8) | **PASS** sur l'arrêt et sur l'état après relance | Tous les processus photographiés sont morts en 1,32 s au plus. Le fichier reste figé. Après relance : `failed/unreachable`, aucun doublon |
| B - relance du daemon avec le parent Codex TUI encore vivant | **BLOQUÉ par le produit, contourné et consigné** | Le daemon refuse de démarrer : `symlink d'état interdit` (section 5) |

Le correctif O4 de r8 est prouvé sur un vrai `codex app-server` : le SIGTERM du daemon arrête le wrapper, le fournisseur et la commande de l'enfant.

## 2. Entrées, fixture, règle « zéro grant »

| Élément | Valeur | Début | Fin |
|---|---|---|---|
| Binaire release r8 | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb` | `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6` | identique |
| Lanceur `gclaude` | `/Users/moi/.local/bin/gclaude` | `dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e` | identique |
| CLI claude 2.1.296 résolu | `/Users/moi/.local/share/claude/versions/2.1.296` | `c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937` | identique |
| Empreinte production (111 fichiers) | worktree 149 | `3943009ca82db59d850d913b55b18c7a0a17c91c3143ffaeb39ee6cdd600526d` (reçu r8, 18:52) | identique, recalculée à 19:09 |

- Source immobile pendant la ronde : `find` ne trouve aucun `.rs`, `.toml` ni `Cargo.lock` plus récent que 18:52. Empreinte de toute l'arborescence : `1b8ed026…e4f9`, 200 fichiers, identique au reçu r8.
- Fixture neuve, indépendante des rondes r3 à r5 : `/Users/moi/.cache/bridget149-recipe-r6.bsNt3u` (0700, `umask 077`). Sous-dossiers privés : `home`, `logs`, `project`, `state`, `tmp`, `outside`.
- Aucun port TCP : le daemon n'écoute que sur sa socket Unix `home/bridget.sock`. `lsof` : 0 écoute TCP. Les ports 14776 et 15736 ne sont donc pas concernés.
- Environnement des processus : minimal et explicite (`PATH`, `HOME`, `TERM`, `LANG`, `TMPDIR`, `BRIDGET_HOME`, `BRIDGET_SOCKET`). Le shell du testeur porte le `BRIDGET_HOME` de production et des jetons `CLAUDE_CODE_*`. Aucun n'est transmis. Aucune variable `BRIDGET_T3_*`. Aucune copie d'authentification.
- Registre fixture : `glm` = `glm-5.3-flash` exact ; `codex` = clone de `codex-pro`, `gpt-6.1-sol`, effort `high` déclaré. Aucun secret.
- **Grants (AFTER)** : `native_delegation_grants` = 0, `native_delegation_revocations` = 0, `native_delegation_cancel_receipts` = 0, après A et après B. Aucune aide de grant dans les scripts, aucun `posture` dans les prompts, `request.posture` vide dans les deux tâches.
  - Le BEFORE n'a pas été échantillonné en direct. Il se déduit : la base est neuve (créée à 18:59) et, dans le code de production, la table n'est qu'alimentée par `INSERT … ON CONFLICT` (`delegation.rs:218,220`). Le seul `DELETE` est dans un test (`native_delegation.rs:1510`). Zéro ligne en fin de ronde implique donc zéro ligne avant.

## 3. Scénario A - parent GLM vers enfant GLM (`r6-glm-smoke-01`)

Parent : `bridget gclaude` réel dans un PTY, hors T3, profil `/Users/moi/.claude-glm`. Durée de la session : 167 s.

| Contrôle | Observé |
|---|---|
| Ligne en base | 1 ligne, task `68e3a990-96d7-4e0d-b971-3153564b2ffe` |
| État | `result_available`, `error` vide, `result_sent` vrai, `cleanup_done` vrai, `failure_sent` faux |
| Requête | `agent_type glm`, `model glm-5.3-flash`, `posture` vide, posture effective `development` (déduite par le produit) |
| Fait du parent | `source native_wrapper`, `runtime_mode auto`, `interaction_mode default`, `driver claude_stream_json`, `revision 1` |
| Lanceur et CLI figés | `cli_path` gclaude + `cli_revision dd8dee56…`, `resolved_cli_revision c9b53416…`, `config_dir /Users/moi/.claude-glm` |
| Sources de permission | **17** : user 10, managed 5, project 1, local 1 |
| Héritage | `child_policy` identique à `parent.provider_policy` (égalité exacte) |
| Écriture autorisée | `allowed/child-smoke.md` = `smoke r6 child ok`, 17 octets |
| Hors politique | `forbidden/` et `outside/` vides |
| Modèle réel (transcrits Claude) | parent `glm-5.3-flash` x8 ; enfant `glm-5.3-flash` x4 ; outil de l'enfant : `Write` ; outils du parent : `bridget_delegate`, `bridget_task_status` |
| Durée de l'enfant | `started_at` 1791651710, `completed_at` 1791651730 : 20 s |

Fait actuel, pas autodéclaré. Le parent n'a transmis aucune posture. Le mode `auto` vient du wrapper. Le code `native_permission_observer.rs:54-109` arme un hook `PreToolUse` sur `bridget_delegate`, avec `onFailure block` et un délai de 5 s. Il attend l'accusé du daemon avant de laisser passer l'appel. La délégation a été admise, donc l'accusé est venu. Je n'ai pas de trace directe de l'accusé : c'est une déduction du code et de la base (voir limites).

Refus hors politique : non rejoué. La preuve r5 reste valide (R1 : `Edit(/forbidden/**)` refusé, tâche `failed` + `provider_permission_denied`). Les fichiers de permission, de hook et d'assemblage sont les mêmes qu'en r5. Le reçu r8 ne liste que 5 fichiers de production modifiés depuis r6 : `native_delegation.rs`, `daemon.rs`, `wrapper.rs`, `codex_app_server.rs`, `managed_session.rs`.

Remarque de journal : le stderr du CLI enfant contient `[claude-code:unrecognized_model] {"model":"glm-5.3-flash","query_source":"sdk"}`. Le modèle a répondu normalement. Ce message est informatif.

## 4. Scénario B - cycle de vie du daemon (`r6-codex-life-01`)

Parent : `bridget codex -a never -s workspace-write` réel dans un PTY (TUI Codex 0.161.0). Il délègue à un enfant Codex `gpt-6.1-sol` `high`. L'enfant lance `python3` qui ajoute une ligne à `allowed/heartbeat.txt` toutes les 3 s (borné à 120 itérations). Le parent attend avec `sleep 10` et `bridget_task_status`. Il n'appelle jamais `task_cancel`.

### 4.1 Politique et identité

- Une ligne en base pour le `request_id`.
- Requête : `codex`, `gpt-6.1-sol`, effort `high`, `posture` vide.
- Parent : `source native_wrapper`, `driver codex_app_server`. Politique : `approval_policy never`, `workspaceWrite`, `networkAccess false`.
- `child_policy` égale à la politique du parent. Aucun droit en plus.
- Je n'ai pas de transcrit Codex indépendant pour le modèle. Le modèle s'appuie sur la requête enregistrée, le registre fixture et l'argument `model="gpt-6.1-sol"` de la ligne de lancement.

### 4.2 Travail prouvé, puis SIGTERM du daemon seul

- Attente de preuve : `heartbeat.txt` a atteint **3 lignes** (orchestrateur : premier battement vu 47,18 s après son départ).
- À 19:04:40,8 : photographie de **14 descendants** du daemon, avec PID et date de début. Puis `SIGTERM` au **seul** daemon, PID 25414, après vérification (commande `daemon`, `BRIDGET_HOME` du fixture, pas Firefox).
- Ni le parent PTY ni le wrapper n'ont reçu de signal avant.

| Processus photographié | Rôle | Mort après SIGTERM |
|---|---|---|
| 25414 | daemon fixture | 1,15 s |
| 42717 | `managed-wrapper codex` (wrapper de l'enfant) | 1,32 s |
| 42734 | `codex … app-server` (fournisseur réel de l'enfant) | 1,32 s |
| 43190 | `python3 … heartbeat.txt` (commande de l'enfant) | 1,15 s |
| 42821 | `bridget mcp` (petit-fils du fournisseur) | 1,15 s |
| 10 autres | serveurs MCP lancés par le fournisseur (uv, node, chrome-devtools, browser-use…) | 1,15 à 1,32 s |

- Survivants à 90 s : **0**. L'identité est comparée par PID **et** par date de début, donc pas de confusion par réutilisation de PID. Résolution de la mesure : environ 0,2 s.
- Le stderr de l'enfant montre le démontage du fournisseur au même instant (`thread … not found`, `Unknown process id`), 17:04:41 UTC.
- **Fichier de battement** : 3 lignes au SIGTERM, 3 lignes à la dernière mort, 3 lignes 15 s plus tard. Croissance : 0. Un battement toutes les 3 s : 5 battements manqués. La boucle `python3` aurait duré environ 6 minutes. Elle est donc bien arrêtée par le produit.
- Après l'arrêt, seuls 3 processus mentionnaient encore le fixture : le runner PTY, l'`app-server` du **parent** et le TUI du parent. Ce sont les processus que le brief interdit de tuer avant.
- Base pendant que le daemon est arrêté : l'état reste `working` (personne ne peut le changer).
- Je n'ai pas capturé le `turnID`. La preuve d'arrêt repose sur les PID, le fichier et le journal.

### 4.3 Relance privée

1. **Première relance, 19:04:57 : refusée** (code 1). Cause : `daemon error: symlink d'état interdit : …/home/c-98e50566-55c.sock` (section 5). L'état de la tâche est resté `working` pendant les 90 s suivantes. Aucun processus, aucune écriture.
2. **Seconde relance, 19:07:13** : j'ai retiré uniquement ce lien symbolique (le lien, pas sa cible `/private/tmp/codex-daemon-501/…`). Le daemon démarre (PID 50663). Observation de 45 s :
   - la tâche est **`failed` / `unreachable`** dès la première lecture (+0,0 s) ;
   - `result` vide, `result_sent` faux, `cleanup_done` vrai, `failure_sent` vrai ;
   - 1 ligne pour le `request_id`, 2 lignes au total (A et B) : pas de doublon ;
   - **0 descendant** du nouveau daemon pendant toute l'observation : aucun nouveau processus, aucun nouveau tour, aucune relance de prompt ;
   - `heartbeat.txt` toujours à 3 lignes ;
   - le modèle n'a pas été relancé.
3. **Reconnexion du parent** : le parent Codex a reçu, par Bridget, le message `Délégation native 282247a0-… échouée : unreachable`. Son `bridget_task_status` a rendu `"state": "failed"`, `"result": null`. Aucun faux résultat. La ligne native est devenue terminale, puis le runner a saisi `/exit` 25 s après. Le parent s'est terminé avec le code 0 (le chef PTY a mis plus de 3 s à partir après SIGTERM, sans `-9`, puis il est parti).
4. `native_delegation_cancel_receipts` = 0 : aucune annulation n'a été émise. L'état `failed` vient de la reprise, pas d'un `task_cancel`.

## 5. Constat produit : la relance du daemon est refusée si un parent Codex TUI est vivant

- Cause concrète : `daemon.rs:4483` appelle `environment::validate_existing_tree(state_root)`. Cette fonction (`environment.rs:240-275`) refuse tout lien symbolique dans le namespace. Le TUI Codex du parent crée `home/c-<instance>.sock` comme lien vers `/private/tmp/codex-daemon-501/<hash>`. Le daemon relancé refuse alors de démarrer.
- Ancienneté : `environment.rs` date de la session 089. Le delta r8 ne le touche pas. Ce n'est donc pas une régression F2/O4. C'est la première fois que le cas est observé, car r5 n'avait relancé le daemon qu'avec un parent GLM.
- Effet : tant que le TUI Codex du parent reste ouvert, le daemon ne redémarre pas. Une relance automatique (type launchd) boucle jusqu'à la fin du TUI. **Non testé en production.** Je n'ai pas inspecté d'autre namespace que ce fixture.
- Je n'ai rien corrigé. Contournement du testeur : suppression du seul lien du fixture. Le TUI déjà connecté n'en avait plus besoin.
- Décision du principal : corriger (ignorer ou nettoyer ce lien), documenter, ou accepter.

## 6. Limites vraies

1. Le BEFORE des grants n'est pas échantillonné en direct (déduction en section 2).
2. L'accusé du hook n'est pas lu dans un journal. Il est déduit du code (hook bloquant), du fait enregistré (`source native_wrapper`, `revision 1`) et du succès de la délégation. Le cas inverse (hook sans accusé) a été prouvé en r5 (R4b).
3. Le refus hors politique de l'enfant n'est pas rejoué ici (r5 R1, R2c valides).
4. Pas de transcrit Codex pour le modèle de l'enfant B (4.1).
5. Une seule exécution par scénario. Pas de répétition statistique.
6. La seconde relance n'a pas eu lieu dans les conditions normales (lien retiré). Le comportement « parent reconnecté » est donc observé après un contournement.
7. Le `turnID` n'est pas capturé. L'arrêt du tour est prouvé par les processus et le fichier, pas par l'événement `turn/interrupt` lui-même.
8. Le correctif F2 « petit-enfant imbriqué » n'a pas été rejoué ici (réseau imbriqué hors périmètre, conforme au brief).
9. Résolution des temps de mort : environ 0,2 s (cadence de `ps`).

## 7. Nettoyage et effets de bord

- Le daemon fixture s'est arrêté par `SIGTERM` (PID 50663, 0,10 s). Les processus du parent PTY sont partis seuls. `ps` : 0 processus lié au fixture ou au binaire r8. 0 socket, 0 lien, 0 artefact `np-*.sock` ou `no-*.json` dans `home/`.
- Aucun `kill -9`, `pkill` ni arrêt de groupe. Les seuls `SIGTERM` du testeur visent le daemon fixture : un dans `r6_lifecycle.py` (PID 25414), un à la fin (`r6_daemon.py stop`, PID 50663). Le runner a envoyé un `SIGTERM` individuel au chef PTY du parent B.
- Métadonnées de confiance `/Users/moi/.claude-glm/.claude.json` : sauvegarde 0600 dans `…/state/claude-glm.dot-claude.json.before`, jamais imprimée. Le CLI a ajouté la confiance du dossier du fixture. **Retirée** à 19:10, cette seule entrée. L'entrée étrangère `/Users/moi/Nextcloud/10.Scripts/69.opus2D` est conservée à l'identique. Les compteurs `numStartups`, `tipsHistory`, `promptQueueUseCount` et `tipLifetimeShownCounts` ont bougé par l'usage normal du CLI : je ne les ai pas restaurés, pour ne pas écraser une écriture d'un autre agent. Mode final 0600.
- `settings.json` du profil : `f38457036b1861c851a07bab2e8f05270579540adc3eb8810c18b277f3e4f279`, inchangé.
- Restes volontaires : transcrits du CLI dans `/Users/moi/.claude-glm/projects/-Users-moi--cache-bridget149-recipe-r6-bsNt3u-project-parent-proj/` et lignes d'historique du profil. Ils servent de preuve. Je ne les ai pas supprimés.
- La fixture reste en place : `/Users/moi/.cache/bridget149-recipe-r6.bsNt3u` (0700). Elle contient la base, les journaux PTY et les journaux JSON de cycle de vie (`state/lifecycle-r6-codex-life-01.json`, `state/restart-r6-codex-life-01.json`, `logs/r6-verify.out`).
- Aucune écriture dans `/Users/moi/.cache/bridget-core` (lecture seule du registre, pour le clone) ni dans `/Users/moi/.config/bridget`.

## 8. Comptes de modèles réels

| Rôle | Nombre |
|---|---|
| Parents GLM | 1 |
| Enfants GLM | 1 |
| Parents Codex | 1 |
| Enfants Codex | 1 |
| Essais de harnais sans modèle | 0 |

Aucun repli de modèle ni de fournisseur. Aucune erreur d'API fournisseur.

## 9. Scripts de la ronde (propriété : `recipes/`)

Tous dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recipes/`.

| Fichier | Rôle |
|---|---|
| `r6_daemon.py` | démarre le daemon fixture (environnement minimal, session propre) ou l'arrête par `SIGTERM` vérifié |
| `r6_lifecycle.py` | attend 3 écritures, photographie l'arbre, `SIGTERM` au daemon, mesure morts, fichier et restes |
| `r6_restart.py` | relance et observe (états, arbre, battements, base). Option `--remove-parent-socket-link` |
| `r6_verify.py` | oracles A et B, empreintes, grants |
| `prompts/r6_parent_glm_smoke.md`, `prompts/r6_parent_codex_lifecycle.md` | prompts des deux parents |

Réutilisés sans changement : `recipe_env.sh`, `make_fixture_registry.py`, `run_parent_pty.py`. Les rapports canoniques r5 (`provider-write149.md`, `standalone149.md`) restent historiques. `quickstart.md` et `impl` ne sont pas modifiés.

Rejeu :

```zsh
FX=$(mktemp -d /Users/moi/.cache/bridget149-recipe-r6.XXXXXX)
env -i PATH="$PATH" HOME="$HOME" BRIDGET_149_FIXTURE_ROOT="$FX" BRIDGET_149_BIN=<binaire r8> zsh recipe_env.sh
python3 -I r6_daemon.py start --fixture-root "$FX" --bin149 <binaire r8>
python3 -I run_parent_pty.py --bin149 <bin> --fixture-root "$FX" --prompt-file prompts/r6_parent_glm_smoke.md \
  --request-id r6-glm-smoke-01 --parent claude --cwd "$FX/project/parent-proj" --type-prompt --exit-when-terminal
# B : run_parent_pty.py (codex, workspace-write) + r6_lifecycle.py en parallèle, puis r6_restart.py
python3 -I r6_verify.py --fixture-root "$FX" --bin149 <bin>
```

## 10. Reste à décider par le principal

- Le constat de la section 5 (relance refusée avec un parent Codex TUI vivant) : corriger, documenter ou accepter.
- La clôture de T037/T038 pour le delta r8 : les preuves A et B ci-dessus, avec les limites de la section 6.
