# Feature Specification: Remonter les incidents des délégations

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 068-remontee-incidents-delegues
Titre: Remonter les incidents des délégations
Statut: Implemented
Priorité: P1
Tâches: 15/15 (100%)
Tests: 8 ciblés + 2 suites crate

Résumé:
- Contexte: un coordinateur peut déléguer une tâche puis rester ignorant d'un incident d'outil ou de l'échec terminal de son enfant.
- Objectif: rendre ces deux faits visibles, corrélés et durables pour le coordinateur, sans interrompre son travail ni inventer une transition Maicie.
- Risque principal: confondre une erreur d'outil récupérable avec une tâche échouée, ou émettre un message libre non corrélable.
- Mitigation: deux catégories fermées de faits, identifiants stables, référence redacted et accusé de remise au coordinateur.
- Validation: un incident récupérable est signalé comme avertissement, un terminal en échec comme échec, et aucun des deux ne modifie une mission Maicie.
- Dépendances: SPEC-052, SPEC-063 et SPEC-064.
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-068-remontee-incidents-delegues`
**Created**: 2026-08-30
**Status**: Implemented
**Priority**: P1
**Dependencies**: SPEC-052, SPEC-063, SPEC-064

## Contexte et problème

Un coordinateur peut lancer un agent enfant pour traiter une délégation. Le lien
parent-enfant est aujourd'hui connu, mais il ne porte que les changements de
cycle de vie. Lorsqu'un enfant rencontre un refus ou une erreur d'outil, le
coordinateur ne reçoit pas de fait corrélé. Il ne sait donc pas distinguer une
tâche encore en cours, une erreur récupérable et une exécution réellement
échouée.

Un refus d'outil peut laisser l'enfant capable de choisir un autre chemin. Il
ne prouve donc pas que la délégation a échoué. À l'inverse, une terminaison
réellement en erreur doit avertir le coordinateur que le résultat attendu n'a
pas été produit. Ces deux cas doivent rester séparés.

## User Scenarios & Testing

### User Story 1 - Avertissement exploitable d'un incident d'outil (Priority: P1)

Un enfant délégué rencontre une erreur d'outil pendant son travail. Son
coordinateur reçoit un avertissement qui identifie l'enfant, le type d'incident
et une référence de diagnostic, sans que son propre travail soit annulé.

**Independent Test**: provoquer un incident d'outil connu dans un enfant relié,
observer le fait durable et la notification du parent, puis vérifier que
l'exécution de l'enfant reste non terminale.

**Acceptance Scenarios**:

1. **Given** un enfant relié à un coordinateur et une erreur d'outil
   récupérable, **When** l'incident est observé, **Then** le coordinateur reçoit
   un avertissement corrélé à cet enfant.
2. **Given** cet avertissement, **When** le coordinateur le lit, **Then** il
   contient un code stable et une référence de diagnostic mais aucun détail
   brut, secret, argument d'outil ou contenu fournisseur.
3. **Given** l'avertissement, **When** l'enfant poursuit et termine son
   travail, **Then** le système n'a jamais annoncé que la tâche avait échoué.

### User Story 2 - Échec terminal visible du coordinateur (Priority: P1)

Un enfant délégué termine son exécution en erreur. Son coordinateur reçoit un
fait distinct indiquant que cette exécution n'a pas produit son résultat.

**Independent Test**: faire terminer en erreur une exécution enfant corrélée et
vérifier que le parent reçoit exactement un signal d'échec terminal avec la
corrélation de délégation.

**Acceptance Scenarios**:

1. **Given** un enfant relié dont l'exécution termine en erreur, **When** le
   terminal est reçu, **Then** le coordinateur reçoit un signal d'échec distinct
   d'un avertissement d'outil.
2. **Given** le même terminal rejoué, **When** la remise est reprise, **Then**
   le coordinateur ne traite pas deux fois le même incident.
3. **Given** un échec terminal, **When** le fait est enregistré, **Then**
   l'autorité de mission Maicie ne change pas automatiquement son état.

### User Story 3 - Reprise après indisponibilité du coordinateur (Priority: P2)

Un coordinateur est temporairement indisponible quand l'enfant signale un
incident. À sa prochaine connexion utilisable, il reçoit les faits non accusés
sans perdre leur ordre ni les confondre avec des messages ordinaires.

**Independent Test**: déconnecter le parent, produire un incident enfant,
reconnecter le parent et observer une seule remise du fait durable.

**Acceptance Scenarios**:

1. **Given** un coordinateur indisponible, **When** un enfant relié produit un
   incident, **Then** le fait est conservé jusqu'à une remise ultérieure.
2. **Given** une remise reprise, **When** le coordinateur accuse le fait,
   **Then** une reconnexion suivante ne le remet plus une seconde fois.

## Functional Requirements

- **FR-6801**: Le système DOIT enregistrer un fait durable et corrélé lorsqu'un
  enfant relié rencontre un incident d'outil exploitable par son coordinateur.
- **FR-6802**: Un incident d'outil non terminal DOIT être nommé
  `avertissement` et ne DOIT jamais être présenté comme l'échec de la tâche.
- **FR-6803**: Une exécution enfant qui termine en erreur DOIT produire un fait
  `échec terminal` distinct de tout avertissement précédent.
- **FR-6804**: Chaque fait DOIT identifier au minimum le lien de délégation,
  l'enfant, l'instant observé, une catégorie fermée, un code stable et une
  référence de diagnostic redacted.
- **FR-6805**: Le coordinateur DOIT recevoir les faits non accusés dans l'ordre
  de leur enregistrement, y compris après une indisponibilité temporaire.
- **FR-6806**: Une même occurrence ne DOIT pas être remise deux fois après son
  accusé par le coordinateur.
- **FR-6807**: La notification ne DOIT ni interrompre ni annuler le travail du
  coordinateur. Elle peut être traitée à sa prochaine frontière sûre.
- **FR-6808**: Aucune transition d'objectif, de délégation ou de mission Maicie
  ne DOIT être déduite de ces seuls faits d'exécution Bridget.
- **FR-6809**: Les détails bruts du fournisseur, arguments d'outil, secrets et
  contenu de la conversation ne DOIVENT jamais être projetés dans la
  notification du coordinateur.
- **FR-6810**: Un enfant sans lien de délégation ne DOIT produire aucune
  notification de coordination.

## Non-Functional Requirements

- **NFR-6801**: L'enregistrement et la recherche du prochain fait à remettre
  sont bornés et n'impliquent ni boucle sur la flotte ni service externe.
- **NFR-6802**: La persistance reste compatible avec les données de liens
  antérieures et ne reconstruit aucun message ou historique existant.
- **NFR-6803**: Toute remise vers un parent connecté se produit dans les cinq
  secondes suivant l'observation locale, hors indisponibilité du fournisseur
  du parent.

## Success Criteria

- **SC-6801**: Un incident d'outil récupérable produit un avertissement
  durable, corrélé et redacted, sans transition d'exécution vers `failed`.
- **SC-6802**: Un terminal enfant en erreur produit exactement un échec terminal
  durable et accusé au plus une fois par son coordinateur.
- **SC-6803**: Après une déconnexion puis reconnexion du coordinateur, chaque
  fait non accusé est remis dans son ordre d'origine et aucune occurrence déjà
  accusée n'est répétée.
- **SC-6804**: Les tests prouvent qu'aucun fait de cette feature ne modifie un
  état métier Maicie.
- **SC-6805**: Les diagnostics projetés ne contiennent ni corps fournisseur ni
  paramètres d'outil, tout en gardant un code et une référence permettant le
  débogage.

## Key Entities

- **Fait d'incident délégué**: constat d'exécution Bridget, associé à un lien
  parent-enfant et destiné au coordinateur.
- **Avertissement**: incident d'outil non terminal dont l'enfant peut se
  remettre.
- **Échec terminal**: constat qu'une exécution enfant s'est terminée en erreur.
- **Accusé de coordinateur**: preuve que le wrapper du coordinateur a accepté
  la remise du fait, distincte de toute réponse métier.

## Assumptions

- Le coordinateur et l'enfant sont reliés par une délégation Bridget déjà
  attestée.
- Une remise au coordinateur est une information de pilotage et n'attend pas
  de réponse métier.
- La première version couvre les incidents normalisés par les adaptateurs gérés
  et les terminaux d'exécution corrélés. Elle ne transforme pas les diagnostics
  textuels historiques en nouveaux faits.

## Hors périmètre

- Interrompre automatiquement le coordinateur quand son enfant échoue.
- Déduire ou modifier un état Maicie depuis le runtime Bridget.
- Ajouter un backend d'observabilité, une dépendance externe ou un nouveau
  framework multi-agent.
- Rejouer, reclassifier ou enrichir les incidents antérieurs à cette feature.
