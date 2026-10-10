# Recettes natives réelles T037/T038 — session 149

**Mise à jour du 2026-10-10 :** les recettes réelles ont été exécutées. Preuves : `validation/native-real-recipes-sonnet-r5.md` (GLM et Codex réels), `validation/native-real-smoke-sonnet-r6.md` (smoke), `validation/native-real-restart-sonnet-r7.md` (redémarrage avec parent Codex vivant). Les sections ci-dessous, dont « préparé, non exécuté » et « BLOQUÉ », décrivent l'état de préparation d'origine. Elles sont conservées comme historique. Synthèse : `validation/prelivraison149.md`.

État actualisé : les recettes réelles GLM et Codex sans nouveau grant ont été exécutées. Le reçu est `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r5.md`. Le smoke r6 est décrit dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-smoke-sonnet-r6.md`. Le redémarrage avec parent vivant r7 est encore en cours. Les instructions de préparation ci-dessous sont historiques ; elles ne constituent pas un refus actuel de Codex ni une exigence d'approbation humaine supplémentaire.

Outils de préparation et d'exécution. État : **préparé, non exécuté**. Aucune
case de `tasks.md` n'est cochée par ces outils ; aucun PASS n'est prononcé tant
qu'un binaire 149 compilé n'a pas exécuté les scénarios avec de vrais modèles.

Sécurité (rappel contractuel) : la prod (`/Users/moi/.cache/bridget-core`) et
`/Users/moi/.config/bridget` ne sont jamais lues en écriture ni relancées. Les
scripts refusent ces chemins. Fixture : `/Users/moi/.cache/bridget149-recipe`
(0700, APFS POSIX, socket Unix réel, aucun port TCP requis).

## Prérequis vérifiables

1. Binaire 149 compilé : exporter `BRIDGET_149_BIN=/chemin/bridget` (release,
   même binaire pour daemon et wrappers : le superviseur managé relance
   `current_exe`).
2. Lanceur attesté : `shasum -a 256 /Users/moi/.local/bin/gclaude` doit valoir
   `dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e`
   (constante `TRANSPARENT_GCLAUDE` du contrat). Vérifié à la préparation.
3. Profil `~/.claude-glm` observable (settings sans règles bloquantes, plugins
   sans hooks/settings/MCP). Vérifié à la préparation.

## Ordre d'exécution — T037 variante V1 (parent GLM PTY → enfant GLM flash)

```zsh
export BRIDGET_149_BIN=/chemin/binaire149
zsh recipe_env.sh                     # prérequis + fixture 0700 + registre 0600
zsh run_fixture_daemon.sh             # daemon fixture (socket privé)
python3 run_parent_pty.py --bin149 "$BRIDGET_149_BIN" \
  --fixture-root /Users/moi/.cache/bridget149-recipe \
  --prompt-file prompts/parent_delegate_glm.md \
  --request-id recipe149-t037-glm-01 --timeout 900
python3 verify_oracles.py --fixture-root /Users/moi/.cache/bridget149-recipe \
  --request-id recipe149-t037-glm-01 --scenario t037-glm
kill "$(cat /Users/moi/.cache/bridget149-recipe/state/daemon.pid)"   # SIGTERM, jamais -9
```

Effets attendus (prononcés après coup par l'humain sur les observations) :
l'enfant `glm-5.3-flash` écrit `allowed/write-ok.md` avec Write ; sa tentative
Bash vers `forbidden.txt` est refusée par le CLI (`--disallowedTools Bash`
hérité du parent) ; `result.permission_denials` corrèle le request_id ; la tâche
passe `failed` avec `provider_permission_denied` et jamais `result_available`.

## Ordre — T038 (parent externe hors T3)

a. **Parent GLM PTY** : mêmes commandes que T037 V1 (session parent externe
   gclaude + profil, voie `NativeDelegation`, aucun `BRIDGET_T3_MCP_*`).
b. **Annulation** : `prompts/parent_delegate_cancel.md` + scenario
   `t038-cancel` (état `cancelled`, aucun processus enfant résiduel).
c. **Preuve observer (G-P-08d)** : branche négative — relancer le daemon, puis
   `run_parent_pty.py ... --prompt-file prompts/parent_delegate_glm.md
   --request-id recipe149-t038-neg-01 --stop-daemon-pidfile
   .../state/daemon.pid --stop-delay 20` : le SIGTERM du daemon prive l'observer
   d'ACK ; l'appel `bridget_delegate` suivant est bloqué nommément ;
   `verify_oracles.py --scenario t038-negative` exige 0 ligne en DB.
d. **Parent Codex natif (T037 V2 / T038 parent Codex)** : **BLOQUÉ** — voir
   obstacle 1 dans `../native-real-recipes-prepare-r1.md`.

## Ce que chaque oracle couvre

| Oracle | Où il s'observe |
|---|---|
| Politique figée + révisions | `recipe_env.sh` (expected.json, sources-snapshot.json) ; le daemon revalide au spawn |
| Corrélation unique request_id | `verify_oracles.py` (1 ligne par request_id) |
| Refus fournisseur G-P-01 | payload `failed` + `provider_permission_denied` |
| Observer PTY G-P-08 | branche (c) : blocage nommé sans ACK ; artefacts `np-*.sock`/`no-*.json` supprimés en sortie |
| Composition lanceur+profil | hash gclaude == constante + `CLAUDE_CONFIG_DIR` exporté (capturé de l'environnement source) |
| Cleanup lifecycle | aucun processus lié au fixture (`ps`), état `cancelled` persistant |
| Absence T3 | variables `BRIDGET_T3_MCP_*` absentes, voie `NativeDelegation` seule |
