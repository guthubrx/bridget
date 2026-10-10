# Preuve réelle - écriture autorisée et refus hors politique (ronde r5)

Ronde Sonnet r5, 2026-10-10. Binaire RELEASE `bridget-abfb346e23cc` (SHA256 `abfb346e23ccf51dad41b90658d475e5cbab321c9865dfd9e778a24d37c8c138`). Fixture `/Users/moi/.cache/bridget149-recipe-r2.r5`. Aucun grant Bridget : table `native_delegation_grants` à 0 ligne avant et après. Preuves r4 archivées dans `archive-r4-recettes/`.

## 1. GLM → GLM (`recipe149-t037-glm-r5-04`, task `2a5c6c2f-238e-449f-92b2-ea74523fd0b9`)

- Parent : `bridget gclaude` réel dans un PTY, hors T3, mode réel observé par l'ACK du hook : `auto` / `default`.
- Règles préexistantes (settings projet fixture) : allow `Edit(/allowed/**)`, `Read`, `Glob`, `Grep` ; deny `Bash`, `Edit(/forbidden/**)`.
- Écriture autorisée : outil `Write` sur `allowed/write-ok.md` → « File created successfully ». Contenu `recette149 ok` (13 octets).
- Écriture hors politique : outil `Write` sur `forbidden/write-denied.md` → `File is in a directory that is denied by your permission settings.` Fichier **absent** ; `forbidden/` vide.
- Modèle exact dans les traces : `glm-5.3-flash` (parent x13, enfant x8). Aucun repli `glm-5.3`.
- Résultat de tâche : `failed` + `provider_permission_denied` (G-P-01 corrélé par `request_id`). Pas de faux `result_available`.

## 2. Codex (full-access) → GLM (`recipe149-t037-fullcodex-r5-01`, task `c820c3c7-abac-495c-8df6-0894d936be81`)

- Parent Codex réel `-a never -s danger-full-access` (fait attesté : `approval never`, `dangerFullAccess`, mode `full-access`).
- Enfant `glm-5.3-flash` (transcrit : `glm-5.3-flash` x5). Politique enfant : `bypassPermissions`, `native_wrapper`.
- Écriture `allowed/write-glm-from-codex.md` réelle (`recette149 codex-parent ok`). État `result_available`.

## 3. Codex (workspace-write) → Codex (`recipe149-t037-codexchild-r5-01`, task `744b3d2a-0404-4a83-b61d-58bc2dacbe4a`)

- Enfant `gpt-6.1-sol` effort `high`. Politique enfant stockée = politique du parent : `approval never`, `workspaceWrite`, `networkAccess false`.
- Écriture autorisée : `allowed/write-codex-ok.md`, code 0, contenu `recette149 codex-child ok`.
- Écriture hors workspace : `…/outside/forbidden-codex.txt` → `zsh:1: operation not permitted`, code 1. `outside/` vide.
- État `result_available` (le bac à sable refuse la commande, le tour se termine normalement).

## 4. Codex (workspace-write) → GLM (`recipe149-t037-confinement-r5-01`)

Refus nommé `provider_confinement_unavailable` visible dans le journal du parent. **0 ligne** en base, aucun enfant lancé.

## Limites

- Le refus de l'enfant GLM est un refus d'OUTIL du CLI (règle settings), pas un confinement par le système d'exploitation.
- Une exécution par scénario. Pas de test du parent GLM en mode `plan`.
