# Rapport R2 — tests natifs Bridget 148

**Rapport complet : `/private/tmp/b148.oDC5Jq/glm-tests-natifs-r2.md`** (logs `01` à `10` + empreintes dans le même dossier).

## Verdict

| Étape | Résultat | RC |
|---|---|---|
| 1. e2e natif 148 | **PASS** — 1/1 (5,51 s) | 0 |
| 2. identité T3 | **PASS** — 1 passed, 1 ignored (helper volontaire) | 0 |
| 3. workspace `--lib` mono-thread | **PASS — 1496 / 0 / 15** sous ENV complet | 0 |
| 4a. `cargo fmt --check` | **FAIL** — 36 diffs | 101 |
| 4b. `clippy -D warnings` | **FAIL** — 4 erreurs | 1 |

## Points clés

- **Correctif R1 vérifié** : un seul `prompt`, portant `resume_card=true` ET `mission=true`. Réponse corrélée unique, 10 replays idempotents, `t3_present=false`.
- **Cancel/restart vraiment atteint** : mission bloquée → `cancelled` sans réponse, restart du daemon, résultat durable identique, `started == 2`. Le test est une fonction unique : son PASS prouve chaque assertion.
- **Incident de ma ronde, consigné** : j'avais omis `umask 077` sur le premier run de suite (umask 022). Deux tests `presence_tests` échouaient : le daemon enfant refusait `bridget.db` « non privé » (mode 0644). Diagnostic par données runtime, isolement sous umask 077 → PASS. Cause environnementale, pas produit. Le run de référence complet est celui sous ENV prescrit.
- **fmt** : 36 diffs, tous dans des fichiers WIP. **clippy** : 3 erreurs WIP (`native_delegation.rs` : `cmp_owned`, `collapsible_if` ×2) + 1 **vieux warning de base** (`store/threads.rs:728`, fichier identique à HEAD).
- **Empreintes début = fin** sur les 7 fichiers suivis. Arbre git inchangé. Aucun processus résiduel. Mes 4 répertoires `/tmp` d'échecs nettoyés ; le reste du `/tmp` (runs antérieurs) intact. Aucun modèle réel ; recette GLM opt-in non exécutée.

**Budget : ~20 min / 60 min.**
