# Comparaison T3 Code / Bridget - communication et délégation entre agents

Date : 2026-10-11. Source : analyse en lecture seule de T3 Code `origin/main`
(`/Users/moi/11.Repositories/t3code`, commit 563645cf, lu par `git show`) et de
Bridget (`/Users/moi/Nextcloud/10.Scripts/64.bridget`, base `1f2c661d`).
Document de référence pour les sessions 151 à 158. Feuille de route :
`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`.

Abréviations T3 (chemins relatifs au dépôt T3) : `MCP` =
`apps/server/src/mcp/OrchestratorMcpService.ts`, `C` =
`packages/contracts/src/orchestratorMcp.ts`, `CV2` =
`packages/contracts/src/orchestrationV2.ts`.

## Notes actuelles (sur 5)

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

Bridget est déjà devant sur l'annuaire, les fils partagés, les agents hors
outil et les garde-fous (profondeur 8, 16 enfants, droits hérités). Ne pas
régresser sur ces points.

## T3 : mécanismes à reprendre

**A1 Attente.** `delegate_task` `mode: "async"` (défaut) ou `"wait"`,
`timeoutMs` 10 min par défaut, 60 min max. Expiration : l'enfant n'est pas
annulé, `waitTimedOut: true`, la politique de réveil du parent passe à
`always`. Attente sans polling : abonnement aux événements stockés depuis un
curseur de séquence pris avant la première lecture. Résultat structuré :
`taskId`, `status`, `workState` (`working` / `waiting_for_children` /
`result_available`), `summary` stable, `latestTerminal*` pour les tours
suivants.

**A3 Annulation.** Vérifie d'abord récursivement que toute la descendance peut
être arrêtée, puis arrête l'enfant et la descendance, puis abandonne la remise
au parent. Un nouvel essai avec la même clé ne rejoue que les arrêts échoués.

**A4 Idempotence et reprise.** Identifiants déterministes dérivés de
`clientRequestId`, espace de clés propre à l'appelant. Reçus de commande qui
rejouent aussi les refus. Au redémarrage, rejeu des événements persistés,
reprise du tour racine interrompu de l'enfant ; un enfant non repris « doit
encore un résultat ». Outbox : 5 essais, bail 30 s.

**A5 Contexte.** L'enfant reçoit seulement le prompt. Historique complet par
fork (natif ou transfert portable budgété à 16k tokens, avec références vers
l'historique omis).

**A6 Isolation.** Par `t3_thread_launch` `workspaceStrategy` : `root`,
`existing_worktree{worktreePath}`, `worktree{baseRef, branch?,
startFromOrigin?}`. Un enfant `delegate_task` partage le worktree du parent.

**A7 Droits.** Rangs de modes ; refus de toute élévation ; contrôle refait sous
le verrou de la cible.

**A8 Limites.** T3 n'a ni profondeur max, ni nombre max d'enfants, ni budget.

**A9 Rondes.** Doctrine : une nouvelle tâche par ronde avec tout le contexte
(brief, constats, réponses, objections), clé distincte par ronde, jamais de
relance par envoi dans le fil enfant. `t3_thread_merge_back` : transfert du
delta de contexte.

**A10 Quotas.** Reprise sur limite d'usage à l'heure de réinitialisation, même
fournisseur, identité `fil:run:reset` contre les boucles. Pas de bascule
automatique de fournisseur.

**A11 Demandes de l'enfant.** `t3_pending_request_*` ne répond qu'aux
questions `user_input`, pas aux approbations. Les approbations restent à
l'humain (alerte push).

**B1 Envoyer et attendre.** `t3_thread_wait` : réveil sur fin de run, rend
`timedOut` sans interrompre.

**B2 Steer et file.** `t3_thread_send` modes `auto` / `queue` / `steer` /
`restart` ; fournisseur sans steer natif → interruption puis relance.
`t3_queue_list/read/edit/cancel/reorder/promote_to_steer`.
`t3_thread_interrupt`.

**B3 Réveils.** Boîte aux lettres durable de fin d'enfant, injectée en steer si
le parent est actif, sinon mise en file, résultats groupés. Tâches planifiées
`interval` / `fixed_time` / `webhook` liées au fil appelant ; exécution manquée
ignorée et reprogrammée. Webhooks HMAC-SHA256, 60 requêtes/min, dédup par
identifiant de livraison. Surveillance de PR avec limite de réveils.

**B4 Livraison.** Événements, projection et reçu dans une seule transaction.
Outbox à identifiants déterministes. Remise au parent au moins une fois,
états `pending → claimed → delivered → acknowledged → disposed` ; « remis »
distinct de « lu ».

**B7 Secrets.** `request_secret` : la valeur va au magasin, l'agent ne reçoit
qu'un `secretRef` à usage unique, limité au projet, 24 h.

**C Contrôle.** Panneau d'agents, journal agrégé, arrêt d'un fil qui arrête sa
descendance. Pas d'arrêt global.

**D Sécurité.** Jeton par environnement, fil, fournisseur et session, 24 h,
révoqué à la fin de session. Agents extérieurs : OAuth, plafond lecture seule
par défaut. Chaque outil MCP déclare ses portées (`reads`, `writesThreads`…),
vérifiées à la compilation.

## Bridget : écarts constatés (base 1f2c661d)

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
