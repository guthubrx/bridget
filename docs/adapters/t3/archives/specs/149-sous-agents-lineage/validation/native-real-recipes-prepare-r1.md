# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-real-recipes-prepare-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-real-recipes-prepare-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

- Prod `/Users/moi/.cache/bridget-core` et `/Users/moi/.config/bridget` :
  jamais écrites, jamais relancées ; refus codé dans les scripts.
- Aucun secret lu, affiché ou copié ; seuls des digests sont stockés.
- Pas de redémarrage T3/Bridget prod, pas de seconde app T3, pas de config DB prod.
- Arrêts : SIGTERM seul, jamais `-9`, jamais Firefox ; pidfile + attente.
- Aucun processus durable ni scheduler : le daemon fixture ne vit que pendant
  la recette ; SIGTERM documenté en fin d'ordre.
- Pas de copie `node_modules`/cibles Cargo ; pas de nouvelle dépendance.
