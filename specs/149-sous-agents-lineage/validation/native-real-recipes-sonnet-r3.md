# Recette native réelle T037/T038 - ronde Sonnet r3 (checkpoint)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Statut : **CHECKPOINT_READY**.

## Constaté

- Aucun modèle lancé. Aucun daemon, aucun parent PTY, aucune fixture `bridget149-recipe-*` créée.
- Receipt frais absent : `validation/native149-debug-receipt.json` n'existe pas.
- Binaire debug absent : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target/debug/bridget` n'existe pas.
- Compilation en cours (PID 95028 et 95105, `rustc bridget_daemon`, build lib et build test). Cargo appartient au compilateur r3 ; je n'y touche pas.
- Aucun processus fixture r2 hérité (`recipe149`, `bridget149-recipe`) : pas de duplication possible.

## État des scripts `recipes/` (relus, non modifiés)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r3.md#extrait-1).

## Obstacle exact

Le binaire 149 n'existe pas encore et aucun receipt ne prouve qu'il est postérieur à la dernière source `wrapper.rs` (publication initiale avant `NativeTui::start`). Exécuter un modèle maintenant serait invalide.

## Reprise (ronde d'exécution)

1. Vérifier `native149-debug-receipt.json` : digest du binaire et empreinte des sources postérieurs à la dernière édition de `wrapper.rs`.
2. `BRIDGET_149_BIN=<binaire debug> BRIDGET_149_FIXTURE_ROOT=/Users/moi/.cache/bridget149-recipe-r2.XXXX ./recipe_env.sh`, puis `run_fixture_daemon.sh`, parents PTY (Codex `-a never -s workspace-write`, puis `-s danger-full-access` pour fullCodex→GLM), puis `verify_oracles.py`.
3. Écrire `provider-write149.md` et `standalone149.md` seulement si de vrais modèles s'exécutent.

## Non vérifié

Tout le comportement runtime : aucune preuve fonctionnelle produite dans cette ronde.
