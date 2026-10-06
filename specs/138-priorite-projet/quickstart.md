# Recette 138 — Validation isolée

Cette recette est prévue pour l'implémentation. Aucun résultat de test n'est
déclaré ici. Ne pas utiliser le binaire, la socket ou les données de production.

## Préparer les sorties

Dans le terminal de validation:

```bash
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet
umask 077
export BRIDGET_HOME="$(mktemp -d /tmp/b138.XXXXXX)"
export BRIDGET_SOCKET="$BRIDGET_HOME/bridget.sock"
export TMPDIR="$BRIDGET_HOME"
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR="$(mktemp -d /Volumes/8TB2/01-workflow/bridget-138-cargo.XXXXXX)"
```

Vérifier avant ces commandes que /Volumes/8TB2/01-workflow est monté et
inscriptible. Le home est privé et court. La socket est son enfant, jamais
sa sœur. TMPDIR utilise ce même home. Le target Cargo est propre à cet essai
sur le volume ; aucun dossier partagé avec une compilation en cours.

## Vérifier le comportement

```bash
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec138_ -- --nocapture
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test spec102_threads_test spec138_ -- --nocapture
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test spec138_project_test -- --nocapture
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test execution_store_test spec_138_control_warnings -- --nocapture
```

Ces filtres correspondent aux noms relus dans les preuves noyau, fils et store.
Le filtre test_138 ne sélectionne pas ces tests. Vérifier le nombre sélectionné.
Les helpers de performance ignorés ne sont pas des scénarios fournisseurs sautés.

Le harnais démarre son daemon et ses transports temporaires. Il ne lance aucun
fournisseur et ne dépend d'aucune API extérieure. Il vérifie les échanges
structurés CLI/MCP et l'état persistant, pas uniquement les mocks.

Scénarios attendus:

1. Agents A1/A2 du projet A, B1 du projet B et U inconnu. Défaut local: A1/A2
   uniquement ; --global volontaire: tous avec leur portée visible.
2. A vers B sans motif: aucun dépôt ni notification. Avec motif: avertissement
   résultat, un dépôt, aucun ajout au corps. Réponse corrélée possible.
3. Fil A/B avec notify=[]: choix interprojets requis malgré le silence. Lecture
   legacy: accessible selon les membres, zéro notification et aucun mandat créé.
4. Dépôt/worktree/symlink: même projet ; homonymes et fausses identités: distincts
   ou inconnus. Ancien client inconnu: envoi possible avec warning d'incertitude.
5. Ancien reçu et envoi accepté en cours de remise: résultat conservé, zéro
   doublon. Nouveau motif avec même clé: conflit. Historique136 reste silencieux.
6. Agent Loop sans local ou avec local busy: aucun recrutement extérieur. ROOT
   extérieur configuré sans mandat: décision visible. Avec mandat explicite:
   rappels worker/coordinator/ROOT conservés à plusieurs ticks.

## Recette Agent Loop, CLI et daemon réel

La recette est opt-in. Fournir les deux chemins isolés ; aucun défaut vers la
skill vivante ou le binaire de production. Le build ci-dessous produit le
binaire dans le target privé de la préparation.

```bash
/Users/moi/.cargo/bin/cargo build -p bridget-daemon --bin bridget
export BRIDGET_SPEC138_AGENT_LOOP_SCRIPT=/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py
export BRIDGET_SPEC138_BIN="$CARGO_TARGET_DIR/debug/bridget"
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec138_real_agent_loop_cli_daemon_scoped_replay -- --ignored --nocapture
```

Cette commande et ces variables correspondent au passage réel consigné dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades.md
et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-loop-real.log.
Le harnais injecte AGENT_LOOP_BRIDGET_BIN dans le script et retire l'identité
T3 héritée. Il contrôle le mandat attach-agent, le corps, les warnings et le
reçu rejoué sans quatrième remise. Aucun heartbeat ou LaunchAgent n'est lancé.

## Consolider

```bash
/Users/moi/.cargo/bin/cargo test --workspace
/Users/moi/.cargo/bin/cargo fmt --all -- --check
/Users/moi/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
/Users/moi/.cargo/bin/cargo build --release --workspace
python3 -m unittest discover -s /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/tests -p 'test*.py'
```

Pour la skill en cours de modification, unittest cible le worktree dotfiles isolé.
Consigner le chemin absolu réel et la commande.
Les tests modifient uniquement leurs répertoires temporaires et transports locaux.

Conserver les preuves utiles dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence.
Consigner commande, code de sortie et assertions observées. Aucun restart,
installation globale, commit ou appel fournisseur n'est inclus dans cette recette.
