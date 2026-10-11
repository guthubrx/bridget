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
