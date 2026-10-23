# Diagnostics Git et préparation : preuves (spec 174)

## Portée

Trois défauts corrigés. Les plafonds de 128 worktrees ne changent pas.
Correction technique, non visible dans une interface : pas de diaporama.

## Défaut 1 : code 128 classé comme référence absente

- Avant : tout code 128 de Git donnait « référence de base inconnue » (durable).
- Après : `rev-parse --verify --quiet --end-of-options`. Code 1 donne une référence absente (refus durable). Code 128 reste technique.
- Test RED : `spec174_base_ref_absente_sans_depot_ou_git_technique_sont_separees` échoue sur un dossier hors Git.
- Test GREEN : le même test passe. Couvre référence absente, dépôt sans commit, dossier hors Git, référence valide et référence `-bad`.

## Défaut 2 : détail absent pour workspace_base_unavailable

- Avant : `failure()` renvoyait seulement `message`. Le client MCP lit `detail`.
- Après : `message` et `recovery` constants dans `detail`. Le message technique ne contient plus la sortie d'erreur Git.
- Test RED : `spec174_base_unavailable_detail_constant_sans_stderr` échoue.
- Test GREEN : le même test passe.
- Rendu client : `native157_quota_detail_visible_cote_mcp` passe avec une ligne `workspace_base_unavailable`. Ce test part d'un JSON littéral aligné sur `failure()`, car la fonction est privée au module.

## Défaut 3 : reçu bloquant après échec de lancement Git

- Avant : seul `Refused` déclenchait le nettoyage. Un `OSError` au lancement de Git laissait `ready=false`.
- Après : le nettoyage couvre aussi `OSError`. La garde reste inchangée : dossier absent et entrée absente de `git worktree list`.
- Test RED : `test_linked_checkout_launch_failure_is_cleaned_then_replays` échoue (reçu restant).
- Test GREEN : le même test passe. Le rejeu réussit et `HEAD` vaut le commit demandé.
- Garde : `test_linked_checkout_launch_failure_without_absence_proof_keeps_receipt` passe avant et après. Sans preuve d'absence, le reçu reste.
- Les tests existants de délai et de résidu passent.

## Différence avec T3

T3 renvoie une erreur Git explicite et n'écrit aucun reçu distant.
Bridget écrit un reçu `ready=false` avant `git worktree add`. Le reçu doit donc être nettoyé quand l'absence est prouvée, et conservé sinon.

## Limites

- `mkdir` et la vérification `no_links` restent avant l'appel Git (hors périmètre).
- En mode automatique, une erreur technique garde le préfixe `workspace_preflight_technical`. Son `detail` reste absent.
- Une erreur système renvoyée par la capture Git (lancement, délai) n'est pas filtrée.
- Les tests complets et le build release n'ont pas été lancés. Seul le lot ciblé a été exécuté.

## Commandes de vérification

- `python3 -m pytest tests/test_spec172_project_preparation.py -k linked_checkout -q` : 5 réussis.
- `cargo test -p bridget-daemon spec174_base` : 2 réussis.
- `cargo test -p bridget-daemon native157_quota_detail_visible_cote_mcp` : 1 réussi.
- `cargo test -p bridget-daemon native154_git_creation` : 1 réussi.
