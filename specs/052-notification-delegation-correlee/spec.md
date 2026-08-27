# Feature Specification: Corréler la notification de délégation

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 052-notification-delegation-correlee
Titre: Corréler la notification de délégation
Statut: In Progress
Priorité: P1
Tâches: 3/4 (75%)
Tests: 2/4 (50%)

Résumé:
- Contexte: La notification automatique ne porte que le but et précède le mandat complet de 27 à 2765 secondes.
- Objectif: Faire de la notification automatique un mandat autonome portant ses trois identifiants durables.
- Exécution: Projeter les identifiants au moment où l'outbox connaît les trois valeurs, dans les chemins immédiat et différé.
- Risque principal: Corriger seulement la sortie CLI ou le chemin immédiat et laisser un destinataire recevoir encore un corps incomplet.
- Mitigation: Deux oracles lisent le vrai `PublicMessage` sérialisé après création réussie.
- Validation: Les trois identifiants exacts figurent dans le corps et `PublicMessage.id` égale le `message_id` annoncé.
- Dépendances: SPEC-026 ; composition avec SPEC-047 sans rouvrir sa tête gelée.

Fichiers:
- spec.md: ✓ (specs/052-notification-delegation-correlee/spec.md)
- tasks.md: ✓ (specs/052-notification-delegation-correlee/tasks.md)
- plan.md: ✓ (specs/052-notification-delegation-correlee/plan.md)
- implementation.md: ✓ (specs/052-notification-delegation-correlee/implementation.md)
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-052-notification-delegation-correlee`
**Created**: 2026-08-27
**Status**: In Progress
**Priority**: P1
**Dependencies**: SPEC-026, composition avec SPEC-047

---

## Contexte et problème mesuré

La délégation Maicie crée déjà trois identifiants durables : `objective_id`,
`delegation_id` et `message_id`. Son résultat terminal les rend à l'appelant,
mais la notification remise au participant ne contient aujourd'hui que le but.

Sur six délégations consécutives, le mandat manuel complet est arrivé 27, 33,
33, 33, 39 et 2765 secondes après cette notification. Quatre agents ont refusé
de travailler sur l'annonce incomplète. Ce refus était correct, mais chaque
occurrence a consommé un tour et laissé l'agent sans corrélation exploitable.

L'identifiant affiché par l'enveloppe tmux n'est pas une solution portable :
les transports gérés injectent le corps du message. La surface commune à tous
les destinataires est le `body` du `PublicMessage` sérialisé.

## Décision

La notification automatique devient le mandat. Son corps conserve
l'instruction métier puis porte un bloc fermé :

```text
IDENTIFIANTS DU MANDAT
objective_id  : <uuid>
delegation_id : <uuid>
message_id    : <uuid>
```

L'alternative « préannonce, mandat à suivre » est rejetée : elle enlève
l'ambiguïté mais conserve le second message, le tour perdu, le délai mesuré et
l'impossibilité de corréler plusieurs annonces simultanées.

Cette projection ne crée aucun champ filaire ou schéma SQLite. Les anciennes
outboxes gardent leurs octets et sont rejouées sans reconstruction. Les deux
producteurs actuels — création immédiate et déblocage après prérequis —
consomment la même règle de formatage.

## Relation avec la session 047

La session 047 transporte une cible Git d'entrée typée vers la délégation. La
session 052 projette les identifiants de sortie créés par le greffe. Les deux
faits se complètent, mais ne partagent pas le contrat filaire : 052 ne modifie
pas `ServiceRequestPayload` et ne rouvre pas la tête 047 déjà gelée.

Une fois 047 intégrée, une revue reçoit dans le même corps le but, la cible Git
gelée puis les trois identifiants. Avant cette composition, 052 reste utile et
correcte pour toute délégation ordinaire.

## User Scenarios & Testing

### User Story 1 — Délégation immédiate exploitable (Priority: P1)

Un participant reçoit le premier message automatique et peut commencer sans
attendre un second mandat, car il possède le but et les trois identifiants.

**Independent Test**: créer une vraie délégation, relire son outbox, décoder
le `PublicMessage` et comparer les quatre occurrences aux valeurs créées.

**Acceptance Scenarios**:

1. **Given** une délégation sans prérequis, **When** elle est préparée, **Then**
   le corps sérialisé contient les trois identifiants exacts.
2. **Given** ce message, **When** son enveloppe est décodée, **Then** son champ
   `id` égale exactement le `message_id` écrit dans le corps.

### User Story 2 — Déblocage différé identique (Priority: P1)

Une délégation en attente ne possède pas encore de `message_id`. Lors du
dernier prérequis clos, l'outbox nouvellement créée doit produire le même mandat
complet que le chemin immédiat.

**Independent Test**: créer une dépendance réelle, clore le prérequis, puis
décoder le `PublicMessage` de l'outbox débloquée.

**Acceptance Scenarios**:

1. **Given** une délégation en attente, **When** le dernier prérequis se ferme,
   **Then** le corps sérialisé porte l'objectif, la délégation et le nouveau
   message exacts.
2. **Given** une ancienne outbox déjà persistée, **When** elle est reprise,
   **Then** ses octets ne sont jamais reconstruits pour lui ajouter le bloc.

## Requirements

### Functional Requirements

- **FR-5201**: Toute nouvelle outbox de délégation DOIT porter dans son corps
  les `objective_id`, `delegation_id` et `message_id` exacts.
- **FR-5202**: Le `PublicMessage.id` DOIT être identique au `message_id` rendu
  dans le corps.
- **FR-5203**: Les chemins immédiat et différé DOIVENT employer une règle de
  formatage unique.
- **FR-5204**: Le témoin DOIT lire le message sérialisé destiné au participant,
  jamais la seule sortie de la commande de délégation.
- **FR-5205**: Une outbox historique DOIT conserver et rejouer ses octets ;
  aucune migration ni réécriture opportuniste n'est admise.
- **FR-5206**: Le but et l'éventuelle cible de revue existants DOIVENT rester
  byte-identiques avant le bloc d'identifiants.
- **FR-5207**: Aucun second mandat manuel ni libellé « mandat à suivre » ne fait
  partie du contrat livré.

### Non-Functional Requirements

- **NFR-5201**: Le formatage est O(n), où n est la longueur de l'instruction,
  sans I/O ni nouvelle dépendance.
- **NFR-5202**: L'ajout reste sous la borne de trame existante et passe par la
  validation actuelle de l'outbox.

## Success Criteria

### Measurable Outcomes

- **SC-5201**: Le chemin immédiat rend un message sérialisé avec trois valeurs
  exactes et aucune valeur reconstruite depuis la sortie CLI.
- **SC-5202**: Le chemin différé rend le même contrat après création effective
  du `message_id`.
- **SC-5203**: Un mutant qui rétablit le corps brut tue chaque oracle sur son
  assertion finale après que la création a réussi.
- **SC-5204**: Les tests de reprise d'outbox existants restent verts, prouvant
  qu'aucun octet historique n'est reconstruit.

## Hors périmètre

- Modifier le protocole de requête du guichet ou les versions filaires.
- Rouvrir ou réécrire la tête de la session 047.
- Reconstruire les anciennes notifications.
- Ajouter un parseur de texte des identifiants côté destinataire.
- Automatiser l'attribution des numéros de session.
