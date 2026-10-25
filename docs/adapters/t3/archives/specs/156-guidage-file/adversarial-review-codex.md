# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/adversarial-review-codex.md](../../../../../../specs/156-guidage-file/adversarial-review-codex.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Question** : faille de droits, contournements, intention demandée contre intention effective, preuve de l'effet, sûreté de l'édition de file, migration, pont T3, tests.
- **Verdict reçu** : BLOCKED (11 points)
- **Vérification** : chaque objection est une hypothèse, relue dans le checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`) et dans T3 `origin/main`.

<a id="extrait-2"></a>

## Extrait 2

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | `Interrupt` sans identité ni projet ; il faut toutefois connaître génération et révision | lu `daemon.rs:9002-9035`, `:9100-9170`, `:16174` ; la comparaison génération/révision (`:9161-9165`) suit la réservation et ne protège pas des droits ; `execution_id` est dérivable (`execution-<id du message>`) | oui, nuancée | la faille est réelle mais demande trois valeurs connues, pas seulement l'`execution_id` (la spec a été corrigée). Garde depuis la connexion attestée avant dispatch ; l'interruption de pause (`handle_execution_control(None, …)`, `daemon.rs:9667`) est une exception interne explicite, testée |
| 2 | Contournement par `Send` historique et par `origin=Human` | lu `daemon.rs:17521-17560` et `:8584-8640` : ni `Send` ni `SendIdempotent` ne retirent `origin` ni `intent` fournis par le client ; seul `thread_notice` est neutralisé. `codex_app_server.rs:865-866` interrompt tout message `origin=Human` si un tour est actif ; Claude (`:329`) et ACP (`:630`) font de même. Aucun producteur légitime de l'origine `Human` dans `crates/` hors le dépôt de service (`daemon.rs:15834`, ledger seul) | oui, et plus grave | un agent attesté interrompt déjà aujourd'hui par une trame brute. La garde couvre les trois chemins (FR-010). Le daemon fabrique l'origine : `Human` n'est gardée que pour une connexion attach ou un client nu humain, sinon elle est retirée (FR-009). T002 inventorie les producteurs réels (relais UI, attach) avant la neutralisation |
| 3 | Intention demandée et effective confondues ; canon réservé avant le routage | lu `daemon.rs:8654` (`canonical_send` puis `reserve`) avant `prepare_dispatch` ; le canon inclut l'intention | oui | le plan réécrivait l'intention selon l'état de la cible, donc après le canon. Décision : on canonise l'intention DEMANDÉE (`steer` → `SteerCurrent`, `interrupt` → `InterruptAndStart`, `queue` → aucune). Le daemon ne la réécrit jamais. Plus de table de décision R4 ni de lecture d'état de la cible à l'envoi. Test de rejeu actif puis inactif |
| 4 | Le résultat annoncé ne prouve pas l'effet | lu `execution_store.rs:527-531` (état = projection des transitions ; une exécution n'existe que pour `TriggerTurn`/`InterruptAndStart`, donc un tour démarré par un envoi ordinaire est invisible) ; `codex_app_server.rs:1700-1760` : un `turn/steer` refusé remet le message en file (repli), un accusé « accepté » n'est pas « remis » | oui | `steered`, `interrupted_restarted` et `started` supprimés. Le résultat d'envoi est l'admission, avec l'intention demandée et `effect: "unconfirmed"`. L'effet réel est celui du transport, observable par l'accusé de remise et le journal du transport. Aucune promesse synchrone qu'aucun code ne peut tenir |
| 5 | `cancel` peut interrompre un message déjà actif | lu `codex_app_server.rs:1081-1093` : `cancel_delivery` annule d'abord le tour actif si l'identifiant correspond | oui | `cancel` sort du périmètre (voir décision) |
| 6 | Accusé `QueueEdit` non durable : panne après retrait mémoire, la remise `dispatching` est réinjectée | lu `send_delivery.rs:379-449` (reprise relit toutes les remises `dispatching`) | oui | `cancel` et `reorder` sortent du périmètre. Prérequis noté pour la future session : réservation durable avant émission, réconciliation avant relivraison |
| 7 | `promote` sans perte non garanti : panne entre retrait et renvoi ; `outcome_unknown` masque un envoi réussi (doublon) | lu `daemon.rs:8931` (`OutcomeUnknown` quand `Prepared`/`Dispatched`) | oui | `promote` sort du périmètre. Prérequis : saga durable de promotion |
| 8 | Le journal n'est pas un reçu idempotent : clé sans cible ni paramètres, purge 24 h inférieure à l'horizon de 7 jours, refus mémorisés contredisant « clé libre » | lu `daemon.rs:466` (`CLIENT_IDEMPOTENCY_HORIZON_SECS` = 7 jours) et `reject_idempotent_send` (`:7694-7718`, un refus après réservation est mémorisé jusqu'à l'expiration) | oui | le reçu de rejeu est l'enregistrement d'idempotence existant (7 jours, canon complet). `guidance_events` devient un audit et un compteur de rythme, jamais une mémoire de rejeu (FR-009 d'origine supprimée). Les refus de la garde sont mémorisés par le mécanisme existant : pour réessayer, il faut une nouvelle clé. La phrase « la clé reste libre » est retirée du contrat |
| 9 | Pont T3 : seule la tête de file est examinée ; T3 peut transformer `steer` en interruption | lu `t3code.rs:1962-1975` (`queue.front()` seul) ; lu T3 `CommandPolicy.ts:238-262` : sans `forceRestart` et sans guidage actif, T3 choisit `interrupt_restart` si le fournisseur sait interrompre | oui | le pont (a) fait passer un message de guidage devant les messages bloqués, (b) n'envoie `steer` que vers un fournisseur à guidage natif prouvé (Claude, Cursor : `deliverable_while_busy`, session 129) et refuse sinon `steer_unavailable`, (c) ne laisse jamais T3 se replier en interruption, (d) laisse la remise Claude/Cursor groupable inchangée |
| 10 | Migration : le `CHECK` de `phase` n'accepte ni `cancelled` ni `rejected` ; `dispatching` contient des remises en cours d'injection ; le rang par instance doit survivre aux réaffectations | lu `idempotency.rs:534` : `phase IN ('dispatching','acked','indeterminate','orphaned')` ; `send_delivery.rs:230` : la réaffectation met à jour en place | oui | sans `cancel`, plus de nouvelle phase ni de colonne `queue_rank`. `list` trie par `rowid` (conservé par `UPDATE` en place) et étiquette les lignes « en attente ou en cours d'injection » |
| 11 | Tests anti-boucle et pause à couvrir ensemble | lu `tasks.md` d'origine (T014-T019 par chemin) et `daemon.rs:9670` | oui | tâche de test combinée ajoutée (T020) : quatre chemins, exception de pause, pause qui interrompt toujours après 3 interruptions utilisateur, compteurs qui ignorent l'interruption de pause |

<a id="extrait-3"></a>

## Extrait 3

- La version réduite de 156 garde ce qui apporte l'essentiel du gain et ferme des failles : `steer`/`interrupt` exposés, garde de droits complète sur tous les chemins, correction de l'origine forgeable, anti-boucle, pause, protection des missions, et `bridget_queue list` en lecture seule.
- `cancel` existe déjà pour les demandes suivies (`bridget_cancel`, `reply=true`) avec retrait avant démarrage sur les fils T3 et ACP (`daemon.rs:18125-18200`). Un `cancel` de plus, sûr, demande la saga ci-dessus.
- `reorder` ne marche que sur le pont T3 et change peu de choses.
- `promote` s'écrit en composant `cancel` et `steer` : il suit `cancel`.
- Conséquence sur les critères de la feuille de route : Guidage atteint 4 sur 5, pas 5. Le 5 attend la session d'édition de file. Messages 4 vers 5 reste visé (intention + lecture de file).
- Question utilisateur ajoutée : reporter `cancel`, `reorder`, `promote` à une session dédiée (défaut : oui).

<a id="extrait-4"></a>

## Extrait 4

Les 11 objections sont retenues. Les points 5 à 8 et 10 sont fermés par réduction de périmètre, pas par une saga. Points restant à vérifier : source du rapport de refus asynchrone d'un `steer` à l'émetteur, producteurs légitimes de l'origine `Human`, fournisseurs T3 à guidage natif, noms de champs de la tâche native (tous en T002). Pas de ronde 2 demandée : le périmètre réduit retire les objections d'édition de file.

<a id="extrait-5"></a>

## Extrait 5

| # | Point | Classe | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|---|
| R2-1 | Humain usurpable par reconnexion ou client nu | partie DANS le modèle, partie HORS | lu `daemon.rs:14213` (rôle déclaratif sans preuve), `:8604-8606` (client nu `human`), `cli.rs:3479` et `:4322` (la CLI signe `human` sans preuve et part comme client nu), `attach.rs:267`, `:405-415` (attach demande un terminal) | oui | DANS : `env -i bridget send` suffit, par l'outil. Correctif : humain attesté = rôle attach ; client nu refusé pour le contrôle (FR-008, T016, SC-013). HORS : trame brute `RoleHandshake{Attach}` et pseudo-terminal propre pilotant `bridget attach` : limites documentées avec leurs preuves ; `RoleHandshake` sur une connexion qui a déjà une identité d'agent reste à confirmer (T002(e)) et testé (reconnexion d'agent) |
| R2-2 | `ControlExecution` garde l'origine forgée | DANS le modèle | lu `daemon.rs:9015-9075` (le message est validé, jamais normalisé), `wrapper.rs:5629-5640` (`transport.deliver(message)`), `codex_app_server.rs:865-866`, `:915` (interrompt si `origin=Human` et tour actif) | oui | un parent autorisé obtient guidage ET interruption, hors `mission_active` et limites. Normalisation de l'origine du message de commande avant canon et garde (FR-010, T017, SC-010) |
| R2-3 | Guidage implicite T3 : message ordinaire groupable devient `steer` | DANS le modèle | lu `t3code.rs:2206`, `:3126` (aucun contrôle de l'expéditeur), `t3code_contract_v2.rs:269` | oui | un pair injecte du texte dans un tour actif sans garde, sans pause ni rythme. Le daemon calcule `mid_turn_ok` (humain attaché, parent, enfant) ; les pairs attendent la fin du tour (FR-022, T021, SC-011). Changement visible, d'où la question 3a. L'enfant vers parent reste immédiat pour ne pas casser la session 129 |
| R2-4 | `bridget_cancel` interrompt la cible ACP | DANS le modèle | lu `daemon.rs:18125-18175` (`CancelRequest` contrôle l'émetteur seulement, puis `CancelDelivery`), `acp.rs:518-531` (annule le tour actif si l'identifiant correspond), `t3code.rs:1684-1692` (le pont retire seulement) ; la description MCP de l'outil dit « n'arrête pas l'agent » | oui | retrait en attente et interruption active séparés : `interrupt_active` sur la trame, `cancel_queued` atomique sous le verrou ACP, décision par la garde commune (FR-023, T022, SC-012). Codex et Claude gérés reçoivent un texte, pas `CancelDelivery` : hors périmètre |

<a id="extrait-6"></a>

## Extrait 6

Les 4 points de la ronde 2 sont retenus et intégrés. Reste à vérifier en
T002 : (a) rejet asynchrone d'un `steer`, (b) producteurs de `origin=Human`, (c)
fournisseurs T3 à guidage natif, (d) champs de la tâche native, (e) `RoleHandshake`
sur une connexion avec identité d'agent et résultat de `env -i bridget send`,
(f) points de construction de `DeliverIdempotent` et acteurs qui reçoivent
`CancelDelivery`. Pas de ronde 3 demandée.
