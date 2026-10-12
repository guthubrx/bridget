# Contre-revue adverse - Session158

## Ronde 1 (spec, plan, tâches)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Question** : réalisabilité de l'arrêt global, du budget de tours, des portées MCP et de la vue globale ; cohérence avec ADR 027.
- **Verdict reçu** : BLOCKED (10 points)
- **Points jugés cohérents** : séparation arrêt/pause, découpage 158a/158b, rejet du webhook, couverture par nom des 24 outils actuels.
- **Vérification** : lecture du checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`) et des specs 151 à 157 du worktree.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | L'arrêt ajoute une règle hors de la garde unique de l'ADR 027 | `referent_control.rs:105-133` : `admit_autonomous_effect` ne lit que `ControlStateFrame.paused` ; l'arrêt vit dans une autre table | oui | admission commune `admit_native_delegation(control, halt)` dans `referent_control.rs`, table de vérité à 4 combinaisons, rejeu avant admission conservé (R2) |
| 2 | `last_command_id` : A, B, rejeu de A remet l'arrêt | lecture du plan de données : une seule valeur mémorisée ; `control_events.command_id UNIQUE` (`referent_control.rs:107`) est le modèle déjà utilisé pour le contrôle | oui | table de reçus bornée (256), distincte de l'état ; test A, B, rejeu A, aussi après redémarrage (R3) |
| 3 | `ControlStateV1` se négocie sans attestation humaine : la vue globale s'ouvre aux agents | `daemon.rs:16168-16180` : la capacité est accordée à tout client qui la demande ; la boîte humaine ajoute `control_client_actor` (`daemon.rs:9772`, `9802`) | oui | la lecture globale exige aussi `control_client_actor` ; test d'un client agent négocié (R1) |
| 4 | Le plafond doit suivre `effective_posture`, pas la demande, et durer toute la vie de l'enfant | `delegation.rs:40` (`effective_posture`), `native_delegation.rs:83` (`task.effective_posture.or(posture)`), spec 153 FR-013/014 (enfant inactif conservé entre rondes) | oui | plafond calculé sur `effective_posture.or(request.posture)` tant qu'une tâche de cet enfant n'a pas `cleanup_done` (R7) |
| 5 | `ack` classé écriture de fil : l'enfant `discovery` relit la même page | `store/threads.rs:1549-1565` : un reçu actif rend la même plage jusqu'à l'ACK | oui | `ack` et `read` = état propre ; `post`, `create`, `add_members`, `close` restent refusés, rupture voulue et documentée (R6) |
| 6 | `turns` compte des échantillons non dédupliqués ; seul Claude en émet | `execution_store.rs:793` : `ON CONFLICT ... turns = turns + 1` à chaque échantillon ; `claude_stream_json.rs:1379` : un échantillon par `result` ; `wrapper.rs:6372-6378` : ACP et Codex ignorés (« source non autorisée pour L4 ») | oui | pas de tour attesté pour Codex : budget de tours retiré de 158a (voir décision) |
| 7 | Faux échec au dernier tour (`turns >= max_turns` même si le tour N réussit) | l'ordre usage, réponse, `tick` n'est pas fixé ; `native_delegation.rs:800-822` | oui | sans objet après retrait ; règle notée pour la future session : réussite au tour N acceptée, ouverture du tour N+1 refusée |
| 8 | Budget renouvelé à chaque ronde | spec 153 FR-016/017 : une mission distincte par ronde ; `execution_id = execution-<mission>` (`native_delegation.rs:901`) | oui | sans objet après retrait ; une future borne agrège toutes les exécutions de la tâche |
| 9 | 158b : réveiller un autre agent ou un fil n'est pas `writes_own_state` | cohérent avec la règle d'`bridget_send` (droits de projet) et avec les fils (membres) | oui | portée dépendante de la cible : propre = `writes_own_state`, autre agent ou fil = `sends` ; droits vérifiés avant admission (FR-104) |
| 10 | Coût O(T) faux : arbres de continuation non bornés, requêtes récursives | `execution_store.rs:818` : CTE récursive sur `execution_continuations`, sans borne de profondeur | oui | sans objet pour 158a (plus d'appel) ; coût réel et limite vérifiable consignés en R4 pour la future borne |

## Décision sur le budget de tours

Retiré de 158a. Un tour attesté, dédupliqué et agrégé sur toute la tâche pour
Claude ET Codex n'existe pas aujourd'hui :

- le compteur est un incrément par échantillon, sans identifiant de tour ;
- Codex et ACP sont écartés à la source (`wrapper.rs:6372-6378`) ;
- l'identifiant d'exécution change à chaque ronde (spec 153), donc le cumul par tâche n'existe pas ;
- les arbres de continuation sont non bornés.

En définir un demande un identifiant de tour provider-neutre, un adaptateur
Codex et une agrégation par tâche. C'est une session propre. Livrer une borne
qui ne s'applique qu'à un fournisseur et qui se remet à zéro à chaque ronde
tromperait l'humain (Art. XIX, XX). Le budget de temps reste couvert par
`reply_timeout` (150). Le budget de tours devient la question 3 de la spec.

**Conséquence sur les critères visés** : Contrôle humain passe de 3 à 4
(vue globale, arrêt global, temps par tâche). Le 5 attend le budget de tours.
Sécurité de l'appelant vise 5 avec les portées corrigées (points 4 et 5).

## Corrections faites

Artefacts modifiés : `spec.md`, `plan.md`, `tasks.md` (26 vers 22 tâches),
`data-model.md`, `contracts/control158.md`, `research.md`, `reuse-audit.md`,
`quickstart.md`, `checklists/requirements.md`, `analyze-report.md`.

## Points restant ouverts

- Les outils de 151 à 157 (`bridget_task_continue`, attente, file, etc.) ne sont pas classés dans la table : leur liste dépend de l'implémentation. Le test exhaustif de T016 les impose.
- Le moyen de retrouver la tâche d'un appelant par nom d'enfant reste à mesurer (T017).
- `issuer_scope` est déclaré par le client : l'attestation humaine reste celle de l'ADR 011 (borne connue, non refermée).

## Ronde 2

- **Date** : 2026-10-11. Même agent, lecture seule.
- **Bilan** : objections 1, 3, 4, 5, 9 fermées ; 2 fermée dans la fenêtre de 256 commandes ; 6, 7, 8, 10 sans objet. Verdict : 3 points restants.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| R2-1 | Rejeu de `--cancel-active` : A (arrêt + annulation), B (reprise), C (nouvelle tâche), rejeu de A pourrait annuler C | cohérent avec le plan de données de la ronde 1 : le reçu ne portait pas de réponse et l'annulation n'était pas liée à la première application | oui | reçu avec `response_json` dans la même transaction que l'état ; annulation après commit, une seule fois, jamais au rejeu ; tests A/B/C/rejeu de A, avant et après redémarrage |
| R2-2 | Le partage de journal et la publication d'artefacts deviennent interdits à `discovery` alors qu'ils existent ; élargissement laissé à l'implémenteur | lu `mcp.rs:544-557` (journal avec `to` appelle `execute_send`), `wrapper.rs:5045` (`bridget_publish_artifact` dans la liste d'outils), `wrapper.rs:6184` (réponse finale automatique hors MCP) | oui | politique FR-029 décidée maintenant : réponse automatique et lecture d'artefact permises ; `post`, partage de journal, publication, envoi refusés ; aucun élargissement en cours d'implémentation ; tests ajoutés |
| R2-3 | `bridget_journal` : portée selon `to` mais `readOnlyHint` statique | lu `mcp.rs:544-557` | oui | base `sends`, abaissement à `reads` sans `to`, `readOnlyHint` dérivé de la base seule donc faux ; test `mcp158_journal_read_only_hint_faux` |

**Limite assumée** : l'annulation en cascade est au plus une fois. Un crash entre
l'enregistrement du reçu et la fin de l'annulation laisse des tâches actives ;
l'humain relance `bridget control delegations cancel`.


## Contre-revue d’implémentation158A — Haiku R1, 2026-10-12

Les rondes Codex ci-dessus restent historiques. La revue d’implémentation R1
est distincte : verdict **APPROVE_WITH_CHANGES**, lecture seule.
Question : cohérence des réponses, de l’idempotence, du plafond MCP et des
preuves du diff158A. Rapport brut :
/Users/moi/.cache/bridget-review158A-20261012-r1/rapport.md
SHA256 fcf93c6b503a00d26afa98e3bcc803e30ab9cc8ac103a600760044ceb6dbe736.

| Finding | Vérification et décision ROOT | État actuel et limite |
|---|---|---|
| F1 MEDIUM : refus après arrêt committé | `handle_delegation_halt_set` peut rendre StoreUnavailable après les human_cancel postcommit. Le même code couvre aussi un échec précommit. Dans cmd_control, HaltSet+StoreUnavailable affiche que l’arrêt ou les annulations ont pu être appliqués, puis recommande control delegations et delegations cancel. Exit1 conservé. Aucun champ wire ajouté. | Correctif appliqué et relu. Ne confirme ni commit ni arrêt physique. Le replay de reçu n’émet aucune annulation. Aucun nouveau scénario de panne forcée ajouté. |
| F2 LOW à vérifier : ancêtre purgé | settled_trees ne retient que les arbres entièrement soldés ; purge_trees puis delete_tree suppriment l’arbre dans une transaction. parent_lineage refuse une profondeur d’au moins8. Aucun chemin normal démontré où la purge enlève seulement un ancêtre encore nécessaire. | Objection non retenue comme défaut de production. Une DB corrompue/lignée incohérente ferme cancel_active avant commit ; halt sans annulation reste possible si sa propre lecture/écriture est disponible. Limite ADR documentée, pas test de corruption nouveau. |
| F3 LOW : status partiel peut sortir1 | Le bras status imprime contrôle087 puis lit l’arrêt ; échec de cette lecture fait exit1. FR011, contrat et ADR explicitent que l’affichage partiel n’est pas un succès. | Clarification appliquée. Comportement existant conservé. |
| F4 LOW : suspended ajusté lors du replay Continue | Le replay rend current_task_view avant la garde, sans nouvelle ronde. suspended représente la pause courante. L’oracle à quatre combinaisons confirme la tâche inchangée et cette projection dynamique. | Contrat clarifié : Delegate/Continue rendent une vue courante ; seuls les reçus HaltSet sont historiques. Pas une réponse de mission rendue immuable. |
| TEST-001 MEDIUM, audit cycle1 | La première adaptation158 avait affaibli l’oracle ChildProof en acceptant un RPC préalable. La garde queue préconnexion est restaurée au puits, ainsi que la seule section historique ChildProof avant let queue = tools(). La section positive garde une ligne reply_caller_scope ; la fonction entière n’est pas byte-identique. | Corrigé avec preuve ciblée : lot T021-review-fixes158-lib,22PASS,RC0. L’oracle garantit delegated_tool_forbidden et zéro accept, sans credential factice. Le JSON d’audit original conserve historiquement son état open. |
| Régression preview103 trouvée dans FULL | Reads est autorisé pour toutes les postures. Seul handoff actionpreview réemploie parse_request strict puis preview_result avant le RPC ; aucun ID, DB ou transport. Les paramètres d’envoi, même nuls, sont refusés. | Replay1031PASS. Aucun fallback sur panne RPC et aucune autre lecture de socket exemptée. |

Les pointeurs de code actuels sont :
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/cli.rs:6313 — if command_id.is_some().
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/mcp.rs:631 — if name == "bridget_queue" && identity.delegated_origin.is_some().
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/mcp.rs:639 — if name == "bridget_handoff" && scope == crate::mcp_scopes::Scope::Reads.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/mcp.rs:2690 — fn guidance156_mcp_queue_schema().
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/delegation_lineage.rs:176 — fn settled_trees.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/delegation_lineage.rs:200 — pub fn purge_trees.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle/crates/bridget-daemon/src/delegation_lineage.rs:151 — pub fn parent_lineage.

Preuves : /Users/moi/.cache/bridget-plan158-20261012-r1/T021-review-fixes158-lib.json
(22PASS,16,63s,RC0,218sources stables, logSHAcbdb3bc3ba4a4801a8540edada180445cd3664d96ef9caa10aaff82f85c5e630)
et /Users/moi/.cache/bridget-plan158-20261012-r1/T021-preview103-replay.json
(1PASS,9,24s,RC0, logSHA04aad2ef396a1c5805f165b8caab5f24c2dceb679aef80a1e5cabaacf828e303).
La revue R1 n’avait pas lu le diff entier et n’avait exécuté aucun test.
Les vérifications ciblées et les replays ROOT ci-dessus complètent cette
preuve ; ils ne deviennent pas une revue entière ni un GREENglobal.
T021/T022 et la clôture Converge restent en attente du contrôle final ROOT.

Le dernier oracle152 `native152_rouge_panne_mission_working` est désormais
PASS : /Users/moi/.cache/bridget-plan158-20261012-r1/T021-native152-replay.json
(1PASS,9,29s,RC0,218sources stables), logSHA5a45fa4e827c9d28e0752808384f3a6ca8f653424fd79131dffc73137904f3e7.
Les quatre nouveaux échecs de FULL sont ainsi résolus par replays distincts ;
les huit échecs historiques et le RC101 de la FULL restent conservés.
T021/T022 et Converge demeurent soumis au contrôle final ROOT.

## Contre-revue d’implémentation158A — Haiku R2, 2026-10-12

Verdict : **APPROVE_WITH_CHANGES**, sans défaut bloquant restant. Rapport :
/Users/moi/.cache/bridget-review158A-20261012-r2/rapport.md
SHA256 83b9d943af4268591ab529a2b8b4ffbead0de9450ef58d93cde9fa3e0dda725c.
Les checkpoints précédents restent historiques. T021/T022 sont désormais
prouvées et cochées par ROOT :22/22 ; aucune nouvelle tâche n’est ajoutée.

| Objection | Vérification et décision ROOT | Preuve et limite |
|---|---|---|
| C1 LOW : message trop large pour resume | Un seul littéral CLI devient neutre : l’état du contrôle des délégations a pu changer ; consulter la vue puis annuler séparément si nécessaire. Le bras et exit1 restent identiques. | Source CLI cedf1abb594c5720e37bc94d94b15b6bbe39eca1f5128de7263a99ca413dc543 relue. Ce texte ne confirme pas un commit. |
| C2 LOW : fonction156 présentée entière identique | L’extraction de fn guidance156_mcp_queue_schema jusqu’avant let queue = tools() donne1710octets, SHAc107269454f8b8d3923eaf36a546972137760630a74a86fd6f61f3a010010d69. Seule la section ChildProof est byte-identique. Le reste diffère par la ligne positive reply_caller_scope. | Formulations de tests et de ce rapport corrigées. Aucun code ou oracle changé pour cette clarification. |
| C3 LOW : branche du nouveau message sans test dédié | Le dernier changement est un littéral de diagnostic. Le chemin reste relu, mais aucune panne forcée de human_cancel n’est testée. La compilation de livraison reste une preuve distincte à venir. | Limite acceptée. Aucun scénario supplémentaire : l’utilisateur impose les tests prévus. Aucun PASS de ce message ni build final anticipé. |
| Limite R2 : contenu compilé incertain à cause des dates | Les reçus lib22PASS et quatre replays portent avant/après le même mcp0c525f7c… et cli3c428621…,218sources strictement stables. Le lancement à la même seconde que l’écriture ne remplace pas cette corrélation par empreintes. | Correction CallerScope/preview/ChildProof testée sur les octets annoncés. Le littéral C1 ultérieur cedf1abb… est séparé, sans fausse attribution aux runs antérieurs. |
| Proposition R2 : refaire une FULL | La FULL unique historique reste2440P/12F/74I, RC101. Les quatre nouveaux échecs sont résolus par replays ciblés ;8échecs historiques restent déclarés. | Non retenue : la stratégie retenue limite les répétitions et respecte les tests prévus demandés par l’utilisateur. Pas de GREENglobal déclaré. |

Les limites de lecture R1/R2 demeurent : ces contre-revues n’ont ni lu tous les
hunks ni exécuté les tests. La relecture complète T022 est distincte et attribuée
à ROOT :152/152hunks,126corps identiques réutilisés et26relus après correctifs.
La capture de livraison27fichiers/151hunks suit le seul littéral C1 et les deux
cases T021/T022 cochées ; aucune nouvelle règle métier n’est attribuée à ce delta.
