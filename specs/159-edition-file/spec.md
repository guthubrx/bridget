# Session159 - File de remise tenue par le daemon et édition (159a, 159b)

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 159-edition-file
Titre: File de remise tenue par le daemon (159a) et édition de cette file (159b)
Statut: Livrée — A14/B14, 28/28 ; source 928631df00ac, activation et nettoyage attestés le 12 octobre 2026
Priorité: P1
Tâches: 28 (159a : 14, 159b : 14 ; cochage exclusivement par ROOT)
Tests: FULL unique RC101 puis ciblés PASS distincts ; résultats et arbitrage dans validation/tests-159.md, aucun GREEN global

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/spec.md#extrait-1).

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch utilisée pour B**: `session-159B-edition-file`
**Created**: 2026-10-11
**Status**: Livrée ; reçu canonique validation/livraison159B.md
**Priority**: P1
**Base livrée pour159B** : 159A source 53863bd09a4ff364dc2b412519b343c77e163685, métadonnées b0a165edfc6fc5476b2330193c7822e0faecef33 ; A14 livrées. Les références 158B/candidat2124 sont historiques.
**Dependencies**: SPEC-156 (livrée), SPEC-150 ; base opérationnelle159A livrée

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/spec.md#extrait-2).

## Besoin et périmètre

Un parent voit un message en file qui n'a plus de sens, ou qui doit passer avant
les autres, ou qui doit corriger le tour en cours. Il ne peut rien faire.

### Pourquoi la file doit vivre dans le daemon

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/spec.md#extrait-3).

Le daemon possède déjà tout ce qu'il faut pour tenir la file :
- une base durable unique, avec la saga de remise `send_deliveries` (`idempotency.rs:527`) ;
- un signal de fin de tour des porteurs gérés : `TurnState` met la présence en `busy` ou `connected` (`daemon.rs:3473-3491`, `:17812`) ;
- une reprise des remises non accusées (`daemon.rs:4175-4230`).

Ce qui manque : une phase « retenue » et son déclencheur de libération.

### Exigences d'entrée (issues de 156, objections 5 à 8 et 10) et leur sort

| # | Exigence | Satisfaite par |
|---|---|---|
| E1 | Retirer seulement un message en file, jamais un message actif | Seuls les messages `held` (retenus par le daemon) sont éditables. Un message libéré est engagé : `not_held` |
| E2 | Accusé durable, état final `cancelled` | Une transaction écrit l'état `cancelled` ; rien à accuser chez le porteur |
| E3 | Réservation avant émission, réconciliation avant relivraison | Une édition n'émet rien. La seule émission est la libération, déjà couverte par la reprise des remises `dispatching` |
| E4 | Migration valide du `CHECK` de phase | Migration v7 (phases `held` et `cancelled`, `queue_rank`), précontrôle relevé à 7 |
| E5 | Attente contre engagé, rangs atomiques | `held` contre `dispatching`; une seule copie de l'ordre ; rang global attribué dans la transaction d'écriture |
| E6 | Reçu complet, ni perte ni doublon | Reçu `queue_edits` écrit dans la même transaction que l'effet ; résultat toujours final |

### Découpage

- **159a - File retenue par le daemon.** Retenir, libérer, balayer, lire. Réglage par type d'agent, **fermé par défaut** : sans le réglage, aucun changement de comportement. Aucune édition.
- **159b - Édition.** `cancel`, `reorder`, `promote` sur les messages retenus, plus `bridget_queue` et la CLI. Dépend de 159a.
- Chaque lot se livre et se mesure seul. 159b n'a d'effet que sur les types d'agents où 159a est ouvert.

### Cadre de menace

Identique à 156 (ADR 011, ADR 027) : la garde protège contre un agent qui agit par
ses outils Bridget. Un processus hostile du même compte Unix qui forge des trames
brutes reste hors modèle ; on ne le promet pas. Le daemon reste le seul point
d'application. Les droits passent par la garde commune de 156 (`guidance.rs`).

### Hors périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/spec.md#extrait-4).

## Scénarios utilisateur

### US1 - Retenir les messages d'un agent occupé (P1, lot A)

Un type d'agent est ouvert à la file du daemon. Un pair envoie un message ordinaire
à un agent de ce type qui est en plein tour. Le daemon enregistre l'envoi (accepté
comme aujourd'hui) mais ne le pousse pas. Quand le tour finit, le daemon libère le
plus ancien message retenu. Les messages arrivent dans l'ordre, un par tour. Sans
le réglage, tout se passe comme avant.

### US2 - Rien n'est perdu, rien n'est doublé (P1, lot A)

Le daemon ou le porteur redémarre pendant la rétention ou la libération. Aucun
message encore retenu n'est perdu ni livré deux fois. Un message déjà libéré suit
les garanties de toute remise : s'il était entre l'écriture du reçu `Seen` et
l'injection, il devient `indeterminate`, état explicite, jamais silencieux. Un message retenu qui
dépasse son échéance, ou dont la demande est close, est retiré avec un état
final et l'auteur est prévenu.

### US3 - Voir la file (P1, lot A)

`bridget_queue list` rend les messages `held` (éditables, dans l'ordre de
libération) et `released` (déjà libérés, non éditables), lus dans la base.

### US4 - Retirer un message retenu (P1, lot B)

Un parent retire un message retenu. L'état passe à `cancelled` dans la même
transaction que la prévenance de l'auteur et la clôture du suivi de réponse. Le
message n'est jamais livré. Un message déjà libéré donne `not_held`.

### US5 - Réordonner (P2, lot B)

Un parent place un message retenu avant un autre (ou à la fin). L'ordre de
libération change dans une transaction. Il tient face à une arrivée concurrente.

### US6 - Promouvoir en guidage (P1, lot B)

Un parent promeut un message retenu : le daemon le libère tout de suite en
guidage dans le tour en cours. Si le transport ne peut pas guider, il l'enfile
en file ordinaire. Ce repli reste soumis au rejet explicite si la file est pleine
et aux ambiguïtés après libération de FR-016, SC-009 et R17. Un porteur qui n'a pas
déclaré cette garantie reçoit un refus `steer_unavailable` et le message reste
retenu.

### US7 - Aucun changement pour l'existant (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/spec.md#extrait-5).

## Exigences fonctionnelles

**Lot A - File retenue**

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/spec.md#extrait-6).

**Lot B - Édition**

- **FR-012** : `bridget_queue` accepte `action: "cancel" | "reorder" | "promote"` avec `target`, `message_id`, `request_id` (obligatoire), et pour `reorder` `before_message_id` (texte ou `null` pour la fin).
- **FR-013** : Seuls les messages `held` sont éditables. Un message libéré, accusé, orphelin, indéterminé ou déjà retiré donne `not_held`.
- **FR-014** : `cancel` : une transaction unique écrit la phase `cancelled`, le résultat terminal de l'envoi d'origine `Rejected(cancelled_in_queue)`, la notice d'auteur, la clôture du suivi de réponse et le reçu ; après le commit, le suivi mémoire est retiré (FR-006). Le message n'est plus jamais livré. Le résultat est final (`applied`) : jamais `outcome_unknown`.
- **FR-015** : `reorder` : une transaction re-numérote les seuls messages retenus du destinataire. Les valeurs de rang du groupe sont conservées, leur attribution change. Les deux messages doivent être retenus, du même destinataire. Une arrivée concurrente donne l'un de deux ordres valides selon l'ordre de validation ; le résultat persistant est celui que `list` montre.
- **FR-016** : `promote` : refusé hors tour actif (`no_active_turn`), refusé si le porteur n'a pas déclaré `queue_fallback` (`steer_unavailable`). La transaction recontrôle l’identité attestée, les droits et gardes de `Steer` de 156, la phase `held`, le tour actif, l’absence de réservation de tour, la saga non expirée et, si `reply=true`, la demande ouverte et son échéance. Le tour actif remplace la condition de repos de la libération ordinaire ; il ne la cumule pas. Pendant la pause, un agent est refusé ; l’humain attaché attesté conserve l’exception de guidage de 156. La priorité de reprise de FR-026 continue à différer les libérations automatiques, sans supprimer cette commande humaine explicite. La transaction libère le message et pose la réservation de tour. La copie de la trame de remise porte l’intention de guidage et le marqueur `steer-or-queue` ; l’enveloppe stockée reste inchangée. Un transport qui ne peut pas guider enfile le message en file ordinaire au lieu de le rejeter. **Garanties réduites, dites telles quelles** : après la libération, le message suit les garanties de toute remise. Si la file ordinaire du porteur est pleine, la remise est rejetée explicitement (indéterminée, sort connu de l'expéditeur par les voies existantes). Si Codex reçoit une erreur ambiguë alors que le guidage a été appliqué, il remet le message en file (`codex_app_server.rs:1724`) et un doublon est possible. Ces deux cas sont testés et documentés ; ils ne sont pas supprimés.
- **FR-017** : Reçu `queue_edits` complet, écrit dans la transaction de l'effet, conservé 7 jours. Même clé et mêmes paramètres : même résultat. Autres paramètres : `envelope_mismatch`. Les refus sont mémorisés ; pour réessayer il faut une nouvelle clé.
- **FR-018** : Droits : `cancel` : auteur du message, parent de la cible, humain attaché. `reorder` : parent, humain attaché. `promote` : droits de `steer` de 156 (garde obligatoire). Le destinataire et les pairs n'éditent pas (question 1). Pendant la pause, `promote` est refusé pour un agent ; l’humain attaché attesté garde l’exception de `Steer` de 156. Son identité vient de la connexion validée, jamais de `from=human` ou d’une référence du message. Cette exception ne supprime aucune autre garde de FR-016. `cancel` et `reorder` sont admis. Preuve enfant et client nu : refusés, comme en 156.
- **FR-019** : Un porteur déclare `queue_fallback` à l'enregistrement (champ optionnel, défaut faux). Un porteur ancien reste valable pour `cancel` et `reorder` mais pas pour `promote`, et n'ouvre pas la file retenue sans `turn_attest`.
- **FR-020** : Capacité client `QueueEditV1` pour les écritures de `bridget_queue`. Sans elle : `CapabilityNotNegotiated`.
- **FR-021** : `docs/reference-communication.md`, la skill Bridget et une ADR décrivent la nouveauté, le cadre de menace, le réglage `held_queue`, les garanties réduites, et la procédure de sauvegarde et de restauration avant la migration.

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/spec.md#extrait-7).

## Entités et hypothèses

**Entités**

- **Message retenu (`held`)** : remise acceptée mais pas encore poussée au porteur, en base du daemon, avec un rang.
- **Message libéré** : remise `dispatching` ou après : engagée, non éditable.
- **Réglage `held_queue`** : booléen par type d'agent dans le registre.
- **Édition** : `cancel`, `reorder`, `promote`, avec reçu `queue_edits`.

**Diagnostic historique avant159A (vérifié en T002 ; corrections A livrées)**

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/spec.md#extrait-8).

## Questions pour l'utilisateur

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/spec.md#extrait-9).
