# Preuves natives r1 - T036 et T039 (récapitulatif)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production.

## Verdict global : PARTIAL

| Tâche | Contrôles | Verdict | Détail |
|---|---|---|---|
| T036 interop réseau | 43/43 + 15/15 PASS | APPROVE sur le périmètre exécuté | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md` |
| T039 scénarios dégradés | 23/24 PASS, 1 FAIL | CHANGES_REQUIRED (R9.2) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md` |

Pourquoi PARTIAL : un écart runtime réel (F1 : mission en vol + redémarrage du daemon), et les sources de production ont bougé après le binaire testé. Aucune des deux tâches ne doit être cochée avant la décision sur F1 et un rejeu sur le binaire suivant.

## Ce qui est réellement exécuté (et non déduit d'un prompt)

- Daemon Bridget 149 réel (binaire debug `823e8a5f`), clients `bridget mcp` réels, CLI `bridget lineage` réelle.
- Hôte MCP de T3 réel en HTTP (registre de sessions, rotation, révocation, `bridget_session` v1/v2, faits de permissions).
- Serveur T3 complet réel (`bin.ts`, SQLite privée, RPC WebSocket, jetons scopés) qui lit le daemon réel via `T3CODE_BRIDGET_EXECUTABLE`.
- Enfants Codex fermés (aucun modèle). 28 lancements de PID réels comptés sur le dernier passage des trois recettes.
- Aucun mock CLI `bridget_fixture.mjs`, aucun faux Orchestrator HTTP.

## Simulé, nommé

Publication du fait de permissions (API publique réelle, mais la recette tient lieu d'adaptateur), projection de conversation en mémoire pour l'hôte MCP, wrapper de fil T3 côté daemon (trames réelles), fournisseur enfant.

## Écarts et observations

| Id | Gravité | Sujet |
|---|---|---|
| F1 | moyenne | Mission en vol : relance d'un enfant + mission renvoyée, alors que la tâche est `failed: unreachable` (R9.2) |
| O1 | faible | Credential tombstoné : tous les outils MCP sont fermés, y compris le status d'une tâche admise ; le texte G-P-07(b) « lecture et rejeu inchangés » est plus large que le comportement |
| O2 | info | `project.create` de T3 ajoute un « New thread » ; sans lien avec Bridget |
| O3 | info | Pendant une panne T3, seule la CLI Lineage native lit/annule |
| O4 | info | L'arrêt du daemon ne signale pas les enfants |
| O5 | info | `delivery_generation` (u64, 19 chiffres) : à lire sans arrondi côté non Rust |

## Lacunes concrètes avant de cocher

1. Décision du principal sur F1, puis rejeu de `recovery149.ts` et `server149.ts` sur le binaire qui contient `daemon.rs` / `native_delegation.rs` / `wrapper.rs` actuels (`NATIVE149_BIN=...`).
2. Preuve que le même processus T3 émet les credentials et sert Lineage (recette finale).
3. Les points T037/T038 (modèle réel, standalone Claude/GLM, observer PTY) restent à leurs propriétaires. Rien ici ne les remplace.

## Fichiers

- Outils et résultats : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/` (voir `README.md`).
- Rapports : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md`.
- Empreintes : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/fingerprints.json`.
