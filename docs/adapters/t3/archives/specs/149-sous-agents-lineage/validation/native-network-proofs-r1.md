# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-network-proofs-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-network-proofs-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Daemon Bridget 149 réel (binaire debug `823e8a5f`), clients `bridget mcp` réels, CLI `bridget lineage` réelle.
- Hôte MCP de T3 réel en HTTP (registre de sessions, rotation, révocation, `bridget_session` v1/v2, faits de permissions).
- Serveur T3 complet réel (`bin.ts`, SQLite privée, RPC WebSocket, jetons scopés) qui lit le daemon réel via `T3CODE_BRIDGET_EXECUTABLE`.
- Enfants Codex fermés (aucun modèle). 28 lancements de PID réels comptés sur le dernier passage des trois recettes.
- Aucun mock CLI `bridget_fixture.mjs`, aucun faux Orchestrator HTTP.

<a id="extrait-2"></a>

## Extrait 2

Publication du fait de permissions (API publique réelle, mais la recette tient lieu d'adaptateur), projection de conversation en mémoire pour l'hôte MCP, wrapper de fil T3 côté daemon (trames réelles), fournisseur enfant.

<a id="extrait-3"></a>

## Extrait 3

| Id | Gravité | Sujet |
|---|---|---|
| F1 | moyenne | Mission en vol : relance d'un enfant + mission renvoyée, alors que la tâche est `failed: unreachable` (R9.2) |
| O1 | faible | Credential tombstoné : tous les outils MCP sont fermés, y compris le status d'une tâche admise ; le texte G-P-07(b) « lecture et rejeu inchangés » est plus large que le comportement |
| O2 | info | `project.create` de T3 ajoute un « New thread » ; sans lien avec Bridget |
| O3 | info | Pendant une panne T3, seule la CLI Lineage native lit/annule |
| O4 | info | L'arrêt du daemon ne signale pas les enfants |
| O5 | info | `delivery_generation` (u64, 19 chiffres) : à lire sans arrondi côté non Rust |

<a id="extrait-4"></a>

## Extrait 4

1. Décision du principal sur F1, puis rejeu de `recovery149.ts` et `server149.ts` sur le binaire qui contient `daemon.rs` / `native_delegation.rs` / `wrapper.rs` actuels (`NATIVE149_BIN=...`).
2. Preuve que le même processus T3 émet les credentials et sert Lineage (recette finale).
3. Les points T037/T038 (modèle réel, standalone Claude/GLM, observer PTY) restent à leurs propriétaires. Rien ici ne les remplace.
