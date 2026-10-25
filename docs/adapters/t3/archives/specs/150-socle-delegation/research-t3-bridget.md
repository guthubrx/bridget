# Archives techniques de l’adaptateur T3

Document d’origine : [specs/150-socle-delegation/research-t3-bridget.md](../../../../../../specs/150-socle-delegation/research-t3-bridget.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11. Source : analyse en lecture seule de T3 Code `origin/main`
(`/Users/moi/11.Repositories/t3code`, commit 563645cf, lu par `git show`) et de
Bridget (`/Users/moi/Nextcloud/10.Scripts/64.bridget`, base `1f2c661d`).
Document de référence pour les sessions 151 à 158. Feuille de route :
`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`.

<a id="extrait-2"></a>

## Extrait 2

Abréviations T3 (chemins relatifs au dépôt T3) : `MCP` =
`apps/server/src/mcp/OrchestratorMcpService.ts`, `C` =
`packages/contracts/src/orchestratorMcp.ts`, `CV2` =
`packages/contracts/src/orchestrationV2.ts`.

<a id="extrait-3"></a>

## Extrait 3

| Critère | T3 | Bridget | Session cible |
|---|:-:|:-:|---|
| Attente d'un résultat dans le même tour | 5 | 2 | 151 |
| Garanties de livraison | 5 | 4 | 151 |
| Suivi et annulation | 5 | 4 | 151, 152 |
| Résilience : panne, nouvel essai, quota | 5 | 2 | 152, 157 |
| Contexte transmis, rondes de revue | 4 | 2 | 153 |
| Isolation de l'espace de travail de l'enfant | 3 | 1 | 154 |
| Demandes de l'enfant (approbation, question) | 3 | 1 | 155 |
| Guidage, interruption, file d'attente | 5 | 2 | 156 |
| Envoi de messages et suivi des réponses | 4 | 4 | 151, 156 |
| Lancement, choix et disponibilité du modèle | 5 | 4 | 157 |
| Réveil d'un agent sur événement | 5 | 3 | 158 |
| Contrôle humain, vue d'ensemble | 5 | 3 | 158 |
| Sécurité de l'appelant | 5 | 4 | 158 |

<a id="extrait-4"></a>

## Extrait 4

**A8 Limites.** T3 n'a ni profondeur max, ni nombre max d'enfants, ni budget.

<a id="extrait-5"></a>

## Extrait 5

- **Attente** : seulement asynchrone ; aucun outil MCP n'attend un résultat ;
  `bridget_send` rend la main à l'admission.
- **Délégation** : délai fixe 3600 s (corrigé en 150) ; après un crash du
  daemon, mission `failed/unreachable`, aucun nouvel essai.
- **Pause** : fait échouer les missions en cours (deux chemins :
  `DeliveryRejected` → `native_delegation.rs:820`, état `interrupted` →
  `:1193-1203`) ; la reprise ADR 027 exclut les missions natives
  (`daemon.rs:4251-4258`). Transmis à 152.
- **Contexte** : l'enfant reçoit `task` (64 Kio max) et une carte de reprise ;
  pas d'historique, pas de fork. `bridget_handoff` existe, en message séparé.
- **Rondes** : l'enfant est arrêté dès son résultat remis (`cancel_tree` au
  nettoyage) ; pas de réutilisation.
- **Isolation** : aucun worktree ; `cwd` sous la racine du parent ; seul
  garde-fou : événement `file_collision`.
- **Demandes de l'enfant** : approbations Codex répondues automatiquement selon
  `permissions` (`codex_app_server.rs`, `approval_response`) ; élicitations MCP
  refusées ; la boîte humaine (`human_inbox.rs`, neuf types) ne couvre pas les
  délégations.
- **Guidage** : `MessageIntent::{SteerCurrent, InterruptAndStart}` et
  `ExecutionControlV1` existent, utilisés seulement par la pause ; aucun
  paramètre MCP ou CLI. File de remise ni éditable ni réordonnable.
- **Catalogue** : pas de disponibilité réelle ni de quota ; `rate_limits`
  observés mais inutilisés. Incohérence : `discovery: true` annoncé, refus
  `permission_source_unavailable` depuis un fil T3 Claude.
- **Réveils** : abonnements `bridget_events` (`turn_ended`,
  `permission_required`, `file_written`, `file_collision`), sans rejeu des faits
  manqués ; aucune planification par agent ; aucun webhook.
- **Contrôle** : `bridget lineage` exige `--t3-thread` ; budget du référent
  limité aux objectifs auto-générés ; pas d'arrêt global des délégations.
- **Sécurité** : identité par UID du pair et preuve privée ; fédération SSH
  seulement ; pas de portées déclarées par outil MCP.
