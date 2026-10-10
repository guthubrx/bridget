# Recette native réelle T037/T038 - ronde Sonnet r5 (exécution)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Modèles cibles du produit : Codex `gpt-6.1-sol` high, GLM `glm-5.3-flash`.
Statut global : **PASS sur les voies exécutées, avec limites nommées plus bas**. Aucune case de `tasks.md` cochée. Aucun Cargo, aucun Git, aucun fichier source de production modifié.

## Binaire, fixture, règle « zéro grant »

- Binaire RELEASE immuable : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-abfb346e23cc`, SHA256 `abfb346e23ccf51dad41b90658d475e5cbab321c9865dfd9e778a24d37c8c138` (recalculé au début ET à la fin : identique). `bridget 0.1.3`. Receipt : `native149-release-receipt-r6.json` (production `6cc9a2be…367f`, 111 fichiers). Ancien binaire debug non utilisé.
- Lanceur `gclaude` `dd8dee56…872a8e` (constante du contrat, recalculé en début et fin). CLI claude 2.1.296 `c9b53416…419937` (240 664 432 octets, identique à la mesure r6).
- Fixture privée : `/Users/moi/.cache/bridget149-recipe-r2.r5` (0700, `umask 077`, socket `home/bridget.sock`). Registre : `glm` = `glm-5.3-flash` exact, `codex` = clone `codex-pro` avec `gpt-6.1-sol` effort `high`. Aucun secret dans le registre. Aucune variable `BRIDGET_T3_*`.
- **Correction impérative appliquée** : aucun `--grant-posture`, aucun `delegate-grant`. L'aide au grant est SUPPRIMÉE de `run_parent_pty.py`. Les prompts ne passent plus `posture`. Table `native_delegation_grants` : **0 ligne avant, 0 ligne après chaque scénario** (oracle `verify_oracles.py` : `check_no_grant`). L'héritage vient du fait attesté du wrapper (voie 149, `native_delegation.rs` 421-428).
- Aucun fait forgé : le mode réel vient de l'ACK du hook primaire (parent GLM : `runtime_mode=auto`, `interaction_mode=default` ; parent Codex : `full-access` ou `workspaceWrite`).

## Latence du hook (défaut r4) : corrigée

Hook PreToolUse du vrai CLI + ACK daemon réel passent. Le parent GLM a délégué dès le premier essai utile. Plus aucun `permission_attestation_unavailable` dû à la latence, plus de `NegotiationFailed … os error 35` sur l'enfant GLM. Le seul `permission_attestation_unavailable` observé en r5 est la branche négative voulue (voir R4).

## Résultats (comptes exacts)

| # | Scénario | request_id | Résultat |
|---|---|---|---|
| R1 | Parent GLM PTY (gclaude, premier parent, sans T3) → enfant GLM `glm-5.3-flash` | `recipe149-t037-glm-r5-04` | **PASS** : écriture `allowed/write-ok.md` réelle ; `Write forbidden/write-denied.md` refusé par la règle `Edit(/forbidden/**)` du settings projet préexistant ; fichier absent ; tâche `failed` + `provider_permission_denied` (G-P-01) |
| R2a | Parent Codex `danger-full-access -a never` → enfant GLM | `recipe149-t037-fullcodex-r5-01` | **PASS** `result_available`, `allowed/write-glm-from-codex.md` écrit |
| R2b | Parent Codex `workspace-write -a never` → enfant GLM | `recipe149-t037-confinement-r5-01` | **PASS (refus nommé)** `provider_confinement_unavailable`, 0 ligne en base |
| R2c | Parent Codex `workspace-write` → enfant Codex `gpt-6.1-sol` high | `recipe149-t037-codexchild-r5-01` | **PASS** `result_available` ; écriture OK, refus bac à sable hors workspace |
| R3 | Même famille Claude→Claude, règles préexistantes révisionnées | (inclus dans R1) | **PASS** : voir ci-dessous |
| R4a | Branche négative : `--settings` utilisateur explicite sur le parent | `recipe149-t038-neg-settings-r5-01` | **PASS (refus nommé)** `permission_source_unavailable`, 0 ligne |
| R4b | Branche négative : daemon arrêté avant l'appel | `recipe149-t038-neg-daemon-r5-02` | **PASS (refus nommé)** `permission_attestation_unavailable` par le hook, 0 ligne |
| R5 | Annulation d'un enfant Codex déjà au travail | `recipe149-t038-cancel-r5-01` | **PASS** `cancelled` ; voir limites |

### R1 - détails (preuves brutes)

- Une seule ligne : task_id `2a5c6c2f-238e-449f-92b2-ea74523fd0b9`, owner `709f2968-…`, enfant `607a409d-…`, mission `e3e5564b-…`. Corrélation unique par `request_id`.
- Modèle exact tracé (transcrits Claude du profil GLM, champ `model`) : parent `glm-5.3-flash` x13, enfant `glm-5.3-flash` x8.
- Trace de l'enfant : `Write allowed/write-ok.md` → « File created successfully » ; `Write forbidden/write-denied.md` → `<tool_use_error>File is in a directory that is denied by your permission settings.</tool_use_error>`.
- Aucun grant humain. Aucun faux succès : la mission demandait deux écritures dont une refusée, la tâche est `failed` + `provider_permission_denied`.

### R3 - sources de permission conservées

Le snapshot de la tâche R1 porte `launch_context` : `cli_path` gclaude + `cli_revision` `dd8dee56…`, `resolved_cli_path` `…/versions/2.1.296` + `resolved_cli_revision` `c9b53416…`, `config_dir` `/Users/moi/.claude-glm`, et **17 sources révisionnées** : user `settings.json` (`f3845703…`), managed (`managed-settings.json` `eef11945…`, autres absentes), **project `…/parent-proj/.claude/settings.json` (`df145aa2…`)**, local absent, plugins. Le cwd du parent (`project_cwd`) égale le cwd de délégation. L'overlay hooks-only `--settings home/no-*.json` est le seul `--settings` du parent (vu dans la ligne de lancement). Résultat : les règles `allow`/`deny` préexistantes sont appliquées par l'enfant, pas seulement `--disallowedTools`.

### R5 - annulation

Enfant Codex, commande `python3` qui ajoute une ligne à `allowed/compte.txt` toutes les 2 s. Le parent a attendu (`wc -l`) de voir au moins 3 lignes (il en a vu 6), puis a émis `bridget_task_cancel`. Observé : démarrage 1791644491, annulé 1791644511 (20 s) ; `compte.txt` = 7 lignes, **inchangé 6 s plus tard** ; aucun processus `python3`/enfant restant ; `cleanup_done` vrai ; `result` nul (pas de succès).

## Limites vraies (rien n'est caché)

1. **Nombre exact d'appels `task_cancel`** non dénombrable côté daemon (opération idempotente, aucun compteur). Le prompt exige un appel ; la transcription TUI est redessinée en boucle et ne permet pas de compter. L'état final est unique (`cancelled`).
2. **PID de la commande lente** non capturé : elle est déjà arrêtée quand j'observe. La preuve est le fichier qui cesse de grossir (7 → 7) et l'absence de processus.
3. **Annulation d'un enfant GLM en travail** : non exécutée. Un enfant GLM ne peut pas lancer de Bash (règle `deny Bash`), donc pas de commande lente équivalente. Seul l'enfant Codex est prouvé.
4. **Parent GLM en mode `plan`** (refus `permission_not_inherited` pour une posture `development`) : non exécuté. Le correctif Sol est couvert par les tests unitaires, pas par cette recette.
5. **R4a** : le CLI du parent a démarré avec le `--settings` utilisateur et le refus est tombé à l'appel `bridget_delegate` (`permission_source_unavailable`), pas au lancement. Le refus est nommé, avant effet, sans ligne en base.
6. **R4b** est un refus du hook (pas de ACK possible, daemon absent). Il ne couvre pas les cas fins nonce/PID/session/cwd/contradiction/timeout de l'observer : ceux-là restent couverts par les tests unitaires seulement.
7. **Refus Edit(forbidden)** est un refus d'OUTIL du CLI (règle settings), pas un confinement par le système d'exploitation. Il n'empêche pas d'écrire hors cwd par un autre moyen ; l'enfant GLM n'a pas Bash.
8. **Mode des tâches** : une seule exécution de chaque scénario positif ; pas de répétition statistique.

## Incident de harnais pendant la ronde (non produit)

- Essais `glm-r5-01` à `glm-r5-03` : harnais seulement. (1) Le wrapper refuse un prompt en argv avec caractères shell (`Argument non autorisé contient des caractères shell dangereux`) : le prompt est saisi dans le TUI (`--type-prompt`). (2) Les touches du dialogue de confiance puis du dialogue d'imports externes partaient trop tôt (le TUI ignorait la touche) : elles sont différées (2 s + 3,5 s ; 3 s). Aucun modèle n'a travaillé sur ces essais.
- Un wrapper bloqué sur un dialogue a été arrêté par `kill` d'**un seul PID** (44621, `bridget`, propriétaire fixture, vérifié, SIGTERM, 3 s). Aucun `pkill`, aucun `kill -9`, aucun groupe.
- `neg-daemon-r5-01` : délai d'arrêt du daemon trop long (35 s). La délégation a été admise AVANT l'arrêt (task `d16d0200-…`, enfant GLM démarré). Ce n'est donc PAS une branche négative. Constats utiles : à la mort du daemon l'enfant GLM s'arrête (aucun processus orphelin) ; au redémarrage la tâche passe `failed` + `unreachable`. Rejoué en `neg-daemon-r5-02` avec 6 s.
- Oracle `t037-codex-child` : la chaîne attendue `Operation not permitted` était sensible à la casse ; zsh écrit `operation not permitted`. L'oracle compare maintenant en minuscules. Le refus réel existait.
- Les pids de chef PTY `bridget` restent parfois plus de 3 s après SIGTERM (drainage du TUI) : consigné, pas de `-9`, ils sont tous partis ensuite.

## Comptes de modèles réels

- Parents GLM : sessions qui ont atteint le modèle = 4 (`glm-r5-04`, `neg-settings`, `neg-daemon-r5-01`, `neg-daemon-r5-02`). 3 sessions arrêtées sur dialogue avant modèle.
- Parents Codex : 4 (full-access, confinement, codexchild, cancel).
- Enfants GLM démarrés : 3 (`glm-r5-04` failed/denied, `fullcodex-r5-01` result_available, `neg-daemon-r5-01` interrompu par arrêt du daemon puis `failed`/`unreachable`).
- Enfants Codex démarrés : 2 (`codexchild-r5-01` result_available, `cancel-r5-01` cancelled).
- Délégations en base : 5 (4 attendues + 1 accidentelle). Refus avant effet : 3 (confinement, neg-settings, neg-daemon-r5-02), 0 ligne chacun.

## Effets de bord et nettoyage

- `/Users/moi/.claude-glm/.claude.json` : mon fichier de départ sauvegardé dans `…/state/claude-glm.dot-claude.json.before` (r5, 0600, jamais imprimé). Le CLI a ajouté la confiance du dossier fixture r5. **Retiré à la fin** (entrées `projects` des fixtures r4 et r5 uniquement ; projet `69.opus2D` et tout le reste conservés). `settings.json` du profil **inchangé** (`f3845703…`). Aucun secret copié dans les rapports.
- Tous les processus du fixture arrêtés (`ps` : aucun). Daemon fixture arrêté par SIGTERM (PID 18584, puis 94254). Aucun artefact `np-*.sock` / `no-*.json` restant. Fixture r4 intacte. Aucune écriture dans `/Users/moi/.cache/bridget-core`, `/Users/moi/.config/bridget`, ni config/DB/daemon de production.
- Archive r4 (avant mise à jour) : `validation/archive-r4-recettes/` (rapports, échantillons, scripts r4 d'origine).

## Mapping G-P / S149 (preuves mesurées)

| Exigence | Preuve r5 |
|---|---|
| G-P-01 refus fournisseur corrélé, pas de faux succès | R1 : `failed` + `provider_permission_denied`, task unique |
| Héritage sans grant | table grants = 0 partout ; R1, R2a, R2c |
| Même famille, règles préexistantes révisionnées | R3 (17 sources, project settings `df145aa2…`) |
| Confinement non représentable → refus nommé, pas de repli | R2b `provider_confinement_unavailable` |
| Politique Codex héritée exactement | R2c `workspaceWrite` / `never` ; enfant `gpt-6.1-sol` high |
| Observer : branche négative | R4a `permission_source_unavailable`, R4b `permission_attestation_unavailable` |
| Annulation native, descendants stoppés | R5 |
| Hors T3 | aucune variable T3, aucun processus T3 |

## Outils modifiés (propriété : `recipes/`)

`run_parent_pty.py` (aide au grant retirée, `--type-prompt`, `--extra-argv`, touches de dialogue différées), `verify_oracles.py` (`check_no_grant`, casse du refus bac à sable), `prompts/*.md` (ligne `posture` retirée, prompt d'annulation avec attente de travail réel). Compilation Python OK.

## Reste à faire (hors de ce testeur)

Aucun bloc produit. Cases `tasks.md` T037/T038 : décision du principal après lecture de ce rapport et de `provider-write149.md` / `standalone149.md`.
