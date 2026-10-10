# Recettes natives réelles T037/T038 — préparation r1 (session 149)

Auteur : sous-agent TEST 149. Périmètre : préparation seule. Cette ronde n'a
lancé ni modèle, ni daemon, ni compilation. Les scripts sont vérifiés par
`python3 -m py_compile` et par un passage passif de `recipe_env.sh`
(binaire factice `/bin/echo` uniquement pour ce passage ; fixture de test
supprimé ensuite). Aucune case de `tasks.md` n'est cochée. Aucun PASS.

## 1. Outils créés (fichiers nouveaux)

Sous `specs/149-sous-agents-lineage/validation/recipes/` :

| Fichier | Rôle |
|---|---|
| `recipe_env.sh` | Vérifie les prérequis, crée le fixture 0700 sous `/Users/moi/.cache/bridget149-recipe`, génère le registre fixture, fige `state/expected.json` (digests) et `state/sources-snapshot.json`. Refuse tout `--settings` projet avec règles `permissions`. |
| `make_fixture_registry.py` | Écrit `$BRIDGET_HOME_fixture/agents.json` (0600) : clone du modèle glm prod, deux écarts seulement — modèle `glm-5.3-flash` exact (aucun repli) et capabilities réduites. Aucun secret. Refuse la cible prod et `/Users/moi/.config/bridget`. |
| `run_fixture_daemon.sh` | Daemon fixture (`bridget daemon`, binaire 149). `BRIDGET_HOME`/`BRIDGET_SOCKET` privés 0700 ; socket directement dans `BRIDGET_HOME` (exigence `Namespace`). Aucun vecteur T3 exporté. PID dans `state/daemon.pid`. Refuse un socket déjà présent. |
| `run_parent_pty.py` | Parent externe réel sur PTY (`pty.fork` ; `check_terminal` exige un terminal). `CLAUDE_CONFIG_DIR=/Users/moi/.claude-glm` exporté → composition réelle lanceur+profil capturée par l'observer via l'environnement source. `--disallowedTools Bash` au lancement → entrée réelle revisionnée. Journal PTY complet dans `logs/`. Branche négative `--stop-daemon-pidfile` (SIGTERM daemon → observer sans ACK). Arrêt SIGTERM au groupe, jamais `-9`. |
| `verify_oracles.py` | Post-run : digests lanceur/CLI figés, sources non dérivées, 1 ligne `native_delegations` par request_id (0 en branche négative), états `failed`+`provider_permission_denied` ou `cancelled`, artefacts observer `np-*.sock`/`no-*.json` absents, aucun processus résiduel, aucun vecteur T3. SQLite en lecture seule. |
| `prompts/parent_delegate_glm.md` | Prompt parent T037 V1 (délégation exacte, statut, sans Bash). |
| `prompts/parent_delegate_cancel.md` | Prompt parent T038c (mission longue, annulation, statut). |
| `README.md` | Ordre d'exécution, commandes exactes, table de correspondance des oracles. |

Aucune dépendance ajoutée. Shell zsh + Python 3 stdlib seulement (macOS).

## 2. Prérequis binaires vérifiables

1. `BRIDGET_149_BIN` : binaire 149 compilé, exécutable. Le même binaire sert
   daemon, wrapper interactif et superviseur managé (`current_exe`).
2. `shasum -a 256 /Users/moi/.local/bin/gclaude` =
   `dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e`
   (constante `TRANSPARENT_GCLAUDE`). Vérifié réel à la préparation.
3. CLI claude résolu (symlink canonisé) : `/Users/moi/.local/share/claude/versions/2.1.296`
   (sha256 figé dans `expected.json` au moment de la préparation).
4. Profil `~/.claude-glm` observable : settings utilisateur sans règles
   bloquantes ; `managed-settings.json` de `/Library/Application Support/ClaudeCode/`
   présent sans règles ; pas de `managed-settings.d` ni de plist MDM ; plugins
   sans hooks/settings/MCP. Vérifié réel à la préparation.

`recipe_env.sh` échoue nommément tant que 1 ou 2 manque. C'est le signal
d'attendre le binaire de l'owner GLM (G-P).

## 3. Commandes exactes (exécution future)

Voir `recipes/README.md` — ordre : `recipe_env.sh` → `run_fixture_daemon.sh` →
`run_parent_pty.py` (scénario) → `verify_oracles.py` (scénario) → SIGTERM du
daemon via pidfile. Un request_id unique par run (`recipe149-t037-glm-01`,
`recipe149-t038-cancel-01`, `recipe149-t038-neg-01`).

## 4. Correspondance oracles ↔ observations

| Oracle contrat | Observation prévue |
|---|---|
| Politique figée + révisions sources | `recipe_env.sh` fige digests ; le daemon revalide les sources avant spawn (`recheck_permission_context_sources`) ; `verify_oracles.py` prouve l'absence de dérive entre préparation et exécution. |
| Enfant réel `glm-5.3-flash` exact | Registre fixture pinné ; `apply_child_arguments`/`for_inherited_delegation` fixent `--model glm-5.3-flash` ; le journal PTY et le registre font foi. |
| Écriture autorisée puis refus hors politique (T037) | `allowed/write-ok.md` écrit avec Write ; tentative Bash hors répertoire refusée par le CLI (`--disallowedTools Bash` hérité) → `result.permission_denials` → tâche `failed` `provider_permission_denied`, jamais `result_available` (G-P-01). |
| Corrélation unique | `verify_oracles.py` : exactement 1 ligne `native_delegations` par request_id ; payload consigné dans le rapport d'exécution. |
| Observer PTY (G-P-08d) | Branche négative : daemon SIGTERM-é en session → hook sans ACK dans les 2 s → appel `bridget_delegate` bloqué nommément, 0 ligne en DB ; en positif : délégation acceptée ; artefacts `np-*.sock`/`no-*.json` supprimés en sortie. |
| Composition lanceur+profil attestée | hash gclaude == constante (pas un « nouveau lanceur » : c'est le lanceur connu, composition réelle par `CLAUDE_CONFIG_DIR` exporté) ; capture observer de l'environnement source. |
| Sélection settings sans secrets | `make_fixture_registry.py` ne porte aucune valeur de profil ; `sources-snapshot.json` ne stocke que des digests. |
| Cleanup lifecycle | état `cancelled` persistant (T038c) ; `ps` sans processus lié au fixture ; socket fixture libéré après SIGTERM daemon. |
| Absence T3 | `BRIDGET_T3_MCP_ENDPOINT`/`BRIDGET_T3_MCP_AUTHORIZATION` absents de l'environnement daemon/wrapper ; voie `NativeDelegation` seule (sans preuve `private_proof`). |
| Annexe détachée (artefacts gros build) | Sans objet ici : la recette ne compile rien ; `/Volumes/8TB2/.../bridget149-target` reste le choix de l'owner build, jamais monté par ces scripts. |

## 5. Obstacles précis (attendu / réel / suite)

### Obstacle 1 — parent Codex natif sans fait publié (bloque T037 V2 et le parent Codex de T038)

- Attendu (contrat `lineage.md`/T038) : un parent Codex externe publie son fait
  de permissions issu de l'observation app-server (`config/read` :
  `approval_policy`, `sandbox_mode`) avant toute délégation.
- Réel (lecture du worktree 149) : `codex_interactive.rs` et
  `claude_interactive.rs` ne référencent aucun `NativePermissionFact` ni
  `observed_fact`. La seule publication de faits hors observer Claude vient de
  la voie managée (`launch_session_with_status`, boucle `observed_fact`).
  L'observer PTY n'est monté que pour `protocol == "claude_stream_json"`
  (`wrapper.rs`, branche observer). Sans fait, `parent_fact` refuse le Delegate
  hors posture discovery ; la découverte ne donne pas de posture development.
- Conséquence : le scénario « vrai parent Codex → enfant Codex
  `workspaceWrite`, refus sandbox nommé » n'est pas exécutable aujourd'hui.
  Le refus `provider_confinement_unavailable` reste testable côté mapping
  (unitaires, fixtures synthétiques — distincts de cette recette), pas en réel.
- Fix Sol minimal : T014 — monter l'observer pour le parent Codex externe
  (publication d'un `NativePermissionFact` dérivé de `config/read`, même
  mécanique d'ACK). Sans cela, la recette reste V1 (GLM→GLM) + branches
  annulation et observer négatif, toutes exécutables.

### Obstacle 2 — refus par règle settings incompatible avec le mapping

- Attendu : un refus d'enfant par `permissions.deny` dans des settings projet.
- Réel (`child_policy`) : le chemin claude→codex exige
  `claude_inputs_without_rules` (aucune règle dans les sources, aucun
  `settings_overrides`, `disallowed_tools` vide). Une règle deny casserait la
  voie et dégraderait l'attestation.
- Résolution retenue (aucun relâchement) : refus réel par entrée de lancement
  `--disallowedTools Bash` du parent, capturée par
  `claude_launch_policy_options`, héritée claude→claude, refusée par le CLI
  enfant, corrélée en `permission_denials`. `recipe_env.sh` refuse tout
  settings fixture portant des règles. Pour le futur enfant Codex (post-T014),
  le refus viendra du `sandboxPolicy` du parent réel, pas d'une règle.

## 6. Restrictions de sécurité rappelées (inchangées)

- Prod `/Users/moi/.cache/bridget-core` et `/Users/moi/.config/bridget` :
  jamais écrites, jamais relancées ; refus codé dans les scripts.
- Aucun secret lu, affiché ou copié ; seuls des digests sont stockés.
- Pas de redémarrage T3/Bridget prod, pas de seconde app T3, pas de config DB prod.
- Arrêts : SIGTERM seul, jamais `-9`, jamais Firefox ; pidfile + attente.
- Aucun processus durable ni scheduler : le daemon fixture ne vit que pendant
  la recette ; SIGTERM documenté en fin d'ordre.
- Pas de copie `node_modules`/cibles Cargo ; pas de nouvelle dépendance.

## 7. Limites

- `run_parent_pty.py` n'a jamais piloté un vrai modèle (mission : ne rien
  lancer). Seule la mécanique Python est compilée ; la boucle PTY et la branche
  négative se valideront à l'exécution.
- Les faits `ProviderPermissions` vivent en mémoire daemon : la corrélation se
  prouve par les effets en DB (délégation unique) et par le journal PTY, pas
  par une table de faits.
- Le parent réel peut s'écarter du prompt : le journal PTY fait foi, l'humain
  juge l'observation.
- Aucune durée ni probabilité de succès n'est annoncée.
