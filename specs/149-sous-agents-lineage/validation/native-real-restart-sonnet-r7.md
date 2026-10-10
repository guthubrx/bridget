# Redémarrage réel du daemon avec un parent Codex vivant - ronde Sonnet r7

Date : 2026-10-10. Ronde exécutée de 22:04:27 à 22:06:26 (1 min 59 s). Testeur : Claude Sonnet 5.5.
Modèles réels : Codex `gpt-6.1-sol` effort `high`, un parent et un enfant. Aucun modèle GLM.
Aucune case de `tasks.md` cochée. Aucun Git, Cargo, build, ni fichier de production modifié.
Aucun service, aucune configuration ni base de production touchés. T3 non lancé. Aucun port TCP.

## 1. Verdict

| Contrôle | Verdict |
|---|---|
| Alias Codex hors `home` (`/private/tmp/bridget-codex-<hex>/s.sock`), zéro `c-*.sock` sous `home` | **PASS** |
| SIGTERM du seul daemon pendant que l'enfant écrit : arrêt complet, fichier figé | **PASS** |
| Relance du daemon avec le parent Codex vivant, sans retirer de lien à la main | **PASS** (le blocage de r6 a disparu) |
| Après relance : `failed` / `unreachable`, aucun doublon, aucun nouveau processus, aucune nouvelle écriture | **PASS** |
| Parent reconnecté : résultat vide, jamais de faux résultat | **PASS** |
| 0 grant avant et après (lu directement) | **PASS** |
| Alias et dossier temporaire disparus après la fin du parent | **PASS** |
| Fin « normale » du parent par `/exit` | **LIMITE** : `/exit` n'a pas été honoré en 20 s (section 5) |
| Observateur de processus du script r7 | **ÉCHEC DE L'OUTIL** : 0 évènement de processus enregistré (section 7) |

Oracles scriptés : `logs/r7-verify.out` dans la fixture. 29 contrôles OK, 0 ECHEC.

## 2. Entrées et fixture

| Élément | Valeur | Début | Fin |
|---|---|---|---|
| Binaire release r9 | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-abc850858975` | `abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8` | identique |
| CLI `codex` 0.161.0 | `/opt/homebrew/Caskroom/codex/0.161.0/bin/codex` | `12ac11d2…e6de` | identique |
| Lanceur `codex-pro` | `/Users/moi/.local/bin/codex-pro` | `5caa99aa…92a8` | identique |
| `/Users/moi/.codex/config.toml` | non modifié par la ronde (pas d'entrée de confiance ajoutée) | `19f4b240…6d7a` | identique |
| Empreinte production (111 fichiers) | worktree 149 | `b2b87458cf3cec7989debeb91352b838c417c3eb48329d57099bec4c9a0d5f29` | identique |

- Fixture neuve : `/Users/moi/.cache/bridget149-recipe-r7.iZGJUu` (0700, `umask 077`, plus aucun fichier en `go+rwx` après correction de 2 fichiers, section 8).
- Environnement des processus : minimal (`PATH`, `HOME`, `TERM`, `LANG`, `TMPDIR`, `BRIDGET_HOME`, `BRIDGET_SOCKET`). Aucune variable `BRIDGET_T3_*`. Aucune copie d'authentification.
- Registre fixture : `codex` = clone de `codex-pro`, `gpt-6.1-sol`, effort `high` déclaré. Aucun secret.
- **Grants, lus directement dans la base** : avant le parent : `grants 0`, `revocations 0`, `cancel_receipts 0`, `native_delegations 0`. Après la ronde : `0`, `0`, `0`, `1`. Aucune aide de grant, aucune `posture` dans le prompt.

## 3. Faits actuels du parent (pas autodéclarés)

Lus dans la ligne `native_delegations` (`permission_snapshot`) :

- Requête : `codex`, `gpt-6.1-sol`, effort `high`, `posture` vide. L'en-tête du TUI affiche `GPT-6.1-Sol high`.
- Fait du parent : `source native_wrapper`, `driver codex_app_server`, `revision 1`, `runtime_mode auto`.
- Politique : `approval_policy never`, `sandbox_policy workspaceWrite`, `networkAccess false`, `writableRoots []`.
- `child_policy` égale à la politique du parent. Aucun droit en plus.
- ACK de hook : **non applicable** au parent Codex. `grep codex` dans `native_permission_observer.rs` ne donne aucun résultat. Le fait Codex vient de l'app-server du wrapper. Je n'ai donc pas d'ACK de hook à citer. Le cas « fait forgé » reste couvert par les tests unitaires, pas par cette ronde.

## 4. Chronologie mesurée

Heures locales. Le daemon fixture 1 a le PID 59775 (début 22:04:27). Le parent est lancé à 22:04:28,257.

| Heure | Évènement | Source |
|---|---|---|
| 22:04:28,35 | alias `/private/tmp/bridget-codex-b210f556…` créé, mode 0700, `s.sock` absent | observateur |
| 22:04:28,85 | `s.sock` devient un lien vers `/private/tmp/codex-daemon-501/4b6e2c41…` | observateur |
| 22:05:03 / 22:05:04 | tâche `starting`, puis `working` | observateur (base) |
| 22:05:09,2 / :12,3 / :14,8 | battements 1, 2, 3 dans `allowed/heartbeat.txt` | observateur |
| 22:05:14,998 | **SIGTERM au seul daemon 59775** (PID revérifié : commande `daemon`, `BRIDGET_HOME` de la fixture, pas Firefox) | `r6_lifecycle.py` |
| +1,15 s | les 14 descendants photographiés et le daemon sont morts. Dernier : 63370 (`node`, npx) à +1,34 s | `r6_lifecycle.py` |
| 22:05:16 à 22:05:31 | quiet 15 s : 3 lignes au SIGTERM, 3 à la dernière mort, 3 quinze secondes plus tard | `r6_lifecycle.py` |
| 22:05:31 | **nouveau daemon 65196**, code 0, socket absent avant relance | `r6_lifecycle.py` |
| 22:05:31,76 | première lecture après relance : `failed` / `unreachable` (+0,0 s) | observateur |
| 22:05:31 à 22:06:11 | 8 points d'observation : 0 descendant du nouveau daemon, 3 battements | `r6_lifecycle.py` |
| 22:05:58 | message « Délégation native … échouée : unreachable » écrit dans le ledger | base |
| 22:06:19,80 | alias et dossier supprimés | observateur |
| 22:06:25,8 | runner PTY terminé, statut 0 (durée de session 117,6 s) | `r7_run.py` |

Écritures prouvées : 3, avec le premier battement 40,65 s après le départ de `r6_lifecycle.py`. L'enfant lance `python3` qui écrit une ligne toutes les 3 s. La boucle aurait duré 6 minutes.

Photographie avant SIGTERM : 14 descendants avec PID et date de début (`ps lstart`) : fournisseur `codex app-server`, wrapper de l'enfant, commande `python3 … heartbeat.txt`, `bridget mcp`, `codex-code-mode-host`, serveurs MCP (uv, node, chrome-devtools, browser-use). Survivants : **0**. L'identité compare PID **et** date de début.

## 5. Parent vivant, reconnexion et fin

**Parent vivant pendant le SIGTERM et la relance.** Je le prouve de deux façons.

1. Le parent n'a reçu aucun signal avant la fin de la ronde. `ps` à 22:06:11 montre encore : wrapper `bridget codex` 59802, `codex app-server` 59819, `bridget mcp` 59989, TUI `codex resume … --remote unix:///private/tmp/bridget-codex-b210…/s.sock` 60226. Date de début : 22:04:28 à 22:04:31. Le nouveau daemon date de 22:05:31, soit 40 s plus tôt. C'est une relève manuelle, ponctuelle.
2. Le parent a lu l'état après la relance (voir ci-dessous).

**Ce que le parent a vu** (journal PTY nettoyé, sans horodatage) :

- Pendant que le daemon est arrêté : `daemon Bridget injoignable : No such file or directory (os error 2)`. Refus nommé.
- Juste après la relance : `auxiliary_identity_unproven : preuve absente, invalide ou révoquée`. Refus nommé, plusieurs appels. Le MCP du parent n'est pas encore reconnu par le nouveau daemon.
- Puis `bridget_task_status` a rendu `status failed`, `error unreachable`, `result null`. Le modèle écrit : « État terminal failed reçu à la 4e vérification ».
- Le message de défaillance est arrivé une seule fois dans le TUI.
- **Aucun faux résultat** : `result` vide, `result_sent` faux, `failure_sent` vrai, `cleanup_done` vrai.

Durée de la reconnexion : **déduite**, pas lue. Le message de défaillance est écrit dans le ledger à 22:05:58, soit 27 s après le nouveau daemon. Je le prends comme date de reprise du parent. Je n'ai pas de trace directe de l'enregistrement du wrapper.

**Fin du parent.** La ligne devient terminale à 22:05:31,8. Le runner saisit `/exit` 25 s plus tard (22:05:57, calculé). Le TUI était alors en train de traiter le message de défaillance (« Working »). Le runner envoie un SIGTERM individuel au wrapper 59802 20 s après `/exit` (22:06:17, calculé). Il note : « toujours vivant après 3 s - pas de -9 ». Le wrapper part avec le statut 0. L'alias disparaît à 22:06:19,8. Ce n'est donc pas une fin par `/exit`. C'est la fin standard du runner : un SIGTERM sur un seul PID. Les processus qui restent : 0.

## 6. Alias et namespace

- `home` ne contient aucun lien symbolique pendant toute la ronde (observateur, échantillon toutes les 0,4 s). Contenu final de `home` : `agent-domains`, `agent-names`, `agent-pids`, `agents.json`, `bridget.db(-shm/-wal)`, `bridget.fleet.json`, `bridget.pid`, `bridget.sock`, `managed`, `managed-stderr`, `named-roster.json`, `sessions`, `state`, `tmp`. Aucun `c-*.sock`.
- Le second daemon n'a affiché aucun refus de namespace. Journal du daemon : 4 lignes (PID file écrit, SIGTERM reçu, deux fois).
- Je n'ai supprimé aucun lien, ni modifié aucun namespace ni aucune configuration source.
- Le refus d'un lien étranger n'est pas rejoué ici (test unitaire r9 : 6 négatifs, 2 UT + 4 Codex/HTTP fake).

## 7. Limites vraies

1. **Observateur de processus défaillant.** `r7_run.py`, tel qu'exécuté (SHA256 `be0a1781da4c97016f0367c182972c9000c79c2d50f49a74b17723191c934c04`), n'a enregistré aucun évènement `proc+` ni `proc-` pendant toute la ronde. Cause établie en partie : `emit(self, kind, …)` recevait aussi `kind=` dans ses arguments (`TypeError`). Ce défaut est corrigé (`emit(self, ev, …)`, SHA256 actuel `833f4b94…`). Le correctif est validé sans modèle : sur un vrai daemon fixture jetable, l'observateur voit `proc+` puis `proc-`. **Cause non établie** pour la ronde réelle : le `TypeError` aurait dû tuer le thread au premier processus vu, et il ne l'a pas fait. Le fil a continué à écrire d'autres évènements. Donc l'observateur n'a jamais vu de processus de la fixture. Je n'ai pas relancé de série (consigne). Le cycle de vie repose sur `r6_lifecycle.py` (PID et dates de début), qui, lui, a fonctionné.
2. La liveness du parent s'appuie sur une relève manuelle de `ps` et sur les réponses du TUI. Pas de trace automatique.
3. Durée de reconnexion déduite du ledger (27 s), pas mesurée.
4. `turnID` non capturé. L'arrêt est prouvé par les PID, le fichier et le quiet, pas par l'évènement `turn/interrupt`.
5. Les délais « fin par `/exit` » de la section 5 sont calculés à partir des constantes du runner (25 s, 20 s). Le journal PTY n'a pas d'horodatage.
6. Une seule exécution. Pas de répétition statistique. Charge machine : load average 14 à 45 pendant la ronde.
7. `r6_lifecycle.py` observe 15 s puis 40 s après la relance, conformément au script réutilisé. L'observation de 90 s de r6 n'est pas refaite (consigne).
8. Le registre de la fixture lance le fournisseur de l'enfant avec `--dangerously-bypass-approvals-and-sandbox` (arguments hérités de `codex-pro`, `definition.args`). La politique de l'enfant est `workspaceWrite` côté Bridget. L'application effective de cette politique par le système n'est pas rejouée ici (négatif OS prouvé en r5).
9. Le fichier `4b6e2c41…b08.lock` (0 octet) reste dans `/private/tmp/codex-daemon-501/`. Il appartient au fournisseur Codex. Le socket physique est supprimé. Je n'y touche pas.
10. L'état `auxiliary_identity_unproven` côté MCP du parent dure jusqu'à la reprise du wrapper (environ 27 s déduites). Aucun résultat erroné n'en sort. À garder en tête pour une relance automatique rapide.

## 8. Nettoyage et effets de bord

- Daemon 65196 arrêté par `SIGTERM` (0,11 s). Le parent est parti par le SIGTERM individuel du runner. Aucun `kill -9`, `pkill` ni arrêt de groupe. Seuls SIGTERM : daemon 59775 (`r6_lifecycle.py`), daemon 65196 (`r6_daemon.py stop`), wrapper 59802 (runner, `terminate_process`).
- `ps` : 0 processus lié à la fixture, à `heartbeat.txt` ou à `recipe149-parent-codex-r7`. Aucun `/private/tmp/bridget-codex-<32 hex>` ne subsiste. Les dossiers `bridget-codex-native-*` d'autres rondes sont étrangers et intacts.
- `/Users/moi/.codex/config.toml` : hash inchangé. Aucune entrée de confiance à retirer. Aucune entrée étrangère touchée.
- `/Users/moi/.claude-glm` : non utilisé, non touché.
- Fixture conservée comme preuve : `/Users/moi/.cache/bridget149-recipe-r7.iZGJUu`. Elle contient `state/r7-run.json`, `state/r7-timeline.jsonl`, `state/lifecycle-r7-codex-restart-01.json`, `logs/parent-pty-r7-codex-restart-01.log`, `logs/r7-verify.out`, la base et le journal du daemon.
- Deux fichiers étaient lisibles par le groupe (`logs/r7-verify.out`, un verrou `uv` du fournisseur). Ils sont passés en 0600.
- Un daemon jetable a servi à valider l'observateur corrigé. Il s'est arrêté par SIGTERM. Son dossier `/tmp/b149t-r7obs` est supprimé. Autres daemons vus dans `ps` (PID 67525 et suivants) : ceux d'une autre ronde, non touchés.

## 9. Comptes de modèles réels

| Rôle | Nombre |
|---|---|
| Parents Codex | 1 |
| Enfants Codex | 1 |
| GLM | 0 |
| Essais de harnais avec modèle | 0 |

Aucun repli de modèle. Aucune erreur d'API fournisseur.

## 10. Scripts de la ronde (propriété : `recipes/`)

Dossier : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recipes/`.

| Fichier | Rôle |
|---|---|
| `r7_run.py` | crée daemon, parent PTY, `r6_lifecycle.py` ; observateur ; grants avant/après ; hashes début/fin |
| `r7_verify.py` | oracles en lecture seule (29 contrôles) |

Réutilisés sans changement : `recipe_env.sh`, `make_fixture_registry.py`, `run_parent_pty.py`, `r6_daemon.py`, `r6_lifecycle.py`, `prompts/r6_parent_codex_lifecycle.md`. `quickstart.md`, l'implémentation, `provider-write149.md` et `standalone149.md` ne sont pas modifiés.

Rejeu :

```zsh
FX=$(mktemp -d /Users/moi/.cache/bridget149-recipe-r7.XXXXXX)
env -i PATH="$PATH" HOME="$HOME" BRIDGET_149_FIXTURE_ROOT="$FX" BRIDGET_149_BIN=<binaire r9> zsh recipe_env.sh
env -i PATH="$PATH" HOME="$HOME" LANG=fr_FR.UTF-8 python3 -I r7_run.py --fixture-root "$FX" --bin149 <binaire r9> --request-id r7-codex-restart-01
env -i PATH="$PATH" HOME="$HOME" python3 -I r7_verify.py --fixture-root "$FX"
```

## 11. Reste à décider par le principal

- Le défaut de l'observateur (limite 1) n'affecte pas les verdicts. Faut-il une relance courte pour obtenir la trace automatique du parent vivant ? Elle coûterait 1 parent et 1 enfant Codex.
- Le délai de reprise du parent après relance (environ 27 s déduites, avec refus nommés entre-temps) : à accepter ou à mesurer proprement.
