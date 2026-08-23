# Feature Specification: Domaines d'agents et statut « ne pas déranger »

**Feature Branch**: `session-05-domaines-dnd`
**Created**: 2026-08-17
**Status**: Livrée — vérifiée sur agents réels (session 005)
**Input**: « je ne voyais pas les domaines comme un élément de sécurisation pour
l'instant, plutôt comme essayer de mettre un petit peu d'ordre, de clarté.
J'aime bien l'idée de dériver les domaines depuis le cwd, mais je voudrais qu'on
puisse aussi surcharger ça si on a envie […] et je voudrais aussi que les agents
aient un statut ne pas déranger, à toggle ou non. »

## Contexte

L'annuaire Bridget mélange aujourd'hui tous les agents d'une machine, quel que
soit le projet sur lequel ils travaillent. Avec sept agents simultanés répartis
sur trois dépôts, deux frictions apparaissent : l'humain doit reconstituer de
mémoire qui travaille sur quoi, et un agent qui doit choisir un destinataire n'a
aucun élément pour le faire.

Par ailleurs, un agent en pleine tâche longue subit les interruptions sans
pouvoir les refuser. Le disjoncteur protège contre les boucles, pas contre le
dérangement légitime mais inopportun.

**Ces deux besoins ne sont pas des besoins de sécurité.** Tous les agents
tournent sous le même compte, sur un socket local. Il s'agit d'ordre et de
respect du temps de travail, pas de confinement. Un cloisonnement étanche serait
même nuisible : la revue croisée entre agents de projets différents est un usage
établi de Bridget — la session 004 a été relue par un agent d'un autre dépôt, et
cette relecture a trouvé trois défauts réels.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Savoir qui travaille sur quoi (Priority: P1)

En tant qu'humain, je consulte l'annuaire et je vois immédiatement à quel projet
chaque agent est rattaché, sans avoir à m'en souvenir ni à le configurer.

**Why this priority**: C'est le besoin d'ordre exprimé. Il n'exige aucune autre
partie de la fonctionnalité pour délivrer de la valeur.

**Independent Test**: Lancer deux agents depuis deux dépôts différents et
constater deux domaines distincts dans l'annuaire, sans avoir rien déclaré.

**Acceptance Scenarios**:

1. **Given** un agent lancé dans un dépôt git,
   **When** l'utilisateur consulte l'annuaire,
   **Then** le domaine affiché est le nom du dépôt.
2. **Given** un agent lancé hors de tout dépôt git,
   **When** l'utilisateur consulte l'annuaire,
   **Then** le domaine affiché est le nom du répertoire de travail.
3. **Given** des agents de plusieurs domaines,
   **When** l'utilisateur consulte l'annuaire,
   **Then** **tous** les agents restent visibles ; aucun n'est masqué par défaut.
4. **Given** des agents de plusieurs domaines,
   **When** l'utilisateur demande explicitement un domaine,
   **Then** seuls les agents de ce domaine sont listés.

---

### User Story 2 - Changer le domaine d'un agent en cours de route (Priority: P1)

En tant qu'humain ou agent, je décide qu'un agent appartient désormais à un autre
domaine, et ce choix tient.

**Why this priority**: La dérivation automatique est une commodité, pas une
vérité. Un agent prêté à un autre projet, ou un agent lancé depuis le mauvais
répertoire, doit pouvoir être rangé correctement.

**Independent Test**: Surcharger le domaine d'un agent, consulter l'annuaire,
puis revenir au domaine dérivé.

**Acceptance Scenarios**:

1. **Given** un agent dont le domaine est dérivé,
   **When** il déclare un nouveau domaine,
   **Then** l'annuaire affiche ce domaine.
2. **Given** un agent au domaine surchargé,
   **When** il demande la réinitialisation,
   **Then** l'annuaire affiche à nouveau le domaine dérivé.
3. **Given** un agent au domaine surchargé,
   **When** sa connexion est coupée puis rétablie,
   **Then** le domaine surchargé est conservé.
4. **Given** une déclaration émise hors de tout agent Bridget,
   **When** la commande est exécutée,
   **Then** elle est refusée avec un message explicite.

---

### User Story 3 - Refuser d'être dérangé (Priority: P1)

En tant qu'agent en pleine tâche, je signale que je ne veux pas être dérangé.
Les messages qui m'étaient destinés sont refusés à leur émetteur, qui l'apprend
immédiatement et décide quoi faire.

**Why this priority**: C'est la seconde moitié de la demande, et elle est
indépendante de la première.

**Independent Test**: Mettre un agent en « ne pas déranger », lui envoyer un
message depuis un autre agent, et constater le refus motivé côté émetteur.

**Acceptance Scenarios**:

1. **Given** un agent qui a activé le statut,
   **When** un autre agent lui envoie un message,
   **Then** l'envoi est refusé et l'émetteur reçoit la raison ainsi que le temps
   restant.
2. **Given** un agent qui a activé le statut,
   **When** l'utilisateur consulte l'annuaire,
   **Then** son état indique qu'il ne veut pas être dérangé.
3. **Given** un agent qui a activé le statut,
   **When** il le désactive,
   **Then** il redevient joignable immédiatement.
4. **Given** un agent qui a activé le statut sans préciser de durée,
   **When** la durée de sécurité s'écoule,
   **Then** il redevient joignable sans intervention.
5. **Given** une demande suivie en attente vers un agent qui active le statut,
   **When** l'échéance de rappel arrive,
   **Then** aucun rappel ne lui est délivré.

---

### Edge Cases

- **Deux worktrees d'un même dépôt** : même domaine, puisque la dérivation
  s'appuie sur la racine du dépôt. Cohérent avec la notion de projet ; la
  surcharge couvre le cas où l'on veut les distinguer.
- **Nom de domaine brut** : un dépôt rangé sous `12.mon-projet` donne le domaine
  `12.mon-projet`, préfixe de classement compris. Aucun embellissement
  automatique, pour que la règle reste devinable.
- **Agent injoignable** : conserve son domaine, comme il conserve son modèle.
- **Statut actif et agent qui se déconnecte** : le statut disparaît avec la
  présence ; un agent relancé est joignable.
- **Durée expirée pendant une coupure** : au retour, l'agent est joignable.
- **Auto-envoi** : déjà interdit, le statut ne change rien à ce cas.
- **Le daemon lui-même** : les notifications d'échec adressées à l'émetteur ne
  sont pas des dérangements du destinataire et ne sont pas concernées.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Chaque agent MUST porter un domaine, dérivé sans configuration du
  répertoire dans lequel il a été lancé.
- **FR-002**: La dérivation MUST utiliser le nom de la racine du dépôt git, et à
  défaut le nom du répertoire de travail.
- **FR-003**: L'annuaire MUST afficher le domaine de chaque agent et MUST
  continuer d'afficher tous les agents par défaut.
- **FR-004**: L'utilisateur MUST pouvoir restreindre l'annuaire à un domaine.
- **FR-005**: La sortie machine de l'annuaire MUST exposer le domaine.
- **FR-006**: Un agent MUST pouvoir remplacer son domaine, et revenir au domaine
  dérivé.
- **FR-007**: Un domaine remplacé MUST survivre à une reconnexion et à un
  redémarrage du daemon.
- **FR-008**: Une commande de domaine émise hors d'un agent Bridget MUST être
  refusée avec un message explicite.
- **FR-009**: Un agent MUST pouvoir activer et désactiver un statut « ne pas
  déranger ».
- **FR-010**: L'activation MUST accepter une durée, et MUST appliquer une durée
  de sécurité par défaut à défaut de précision.
- **FR-011**: Un message adressé à un agent ayant activé le statut MUST être
  refusé, et l'émetteur MUST en connaître la raison et le temps restant.
- **FR-012**: Les rappels d'escalade vers un agent ayant activé le statut MUST
  être suspendus.
- **FR-013**: L'annuaire MUST rendre le statut visible.
- **FR-014**: Le statut MUST cesser de s'appliquer dès sa désactivation ou dès
  l'expiration de sa durée, sans action supplémentaire.

### Key Entities

- **Domaine** : étiquette de regroupement d'un agent, dérivée de son répertoire
  ou déclarée. Sert à l'orientation humaine et au choix d'un destinataire ; ne
  confère ni n'ôte aucun droit.
- **Statut de disponibilité** : refus temporaire, borné dans le temps, d'être
  destinataire d'un message. Porté par la présence de l'agent.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Pour 100 % des agents lancés dans un dépôt git, le domaine affiché
  est le nom de ce dépôt, sans configuration préalable.
- **SC-002**: Un domaine remplacé est toujours affiché après une coupure de
  connexion et après un redémarrage du daemon.
- **SC-003**: Un message vers un agent ayant activé le statut est refusé en moins
  d'une seconde, avec une raison lisible mentionnant le temps restant.
- **SC-004**: Aucun rappel d'escalade n'est délivré à un agent ayant activé le
  statut, vérifié sur une demande suivie dont l'échéance survient pendant le
  statut.
- **SC-005**: L'annuaire reste lisible et aligné avec des domaines de longueurs
  très différentes et des agents sans domaine connu.
- **SC-006**: Aucune régression : les colonnes et champs existants conservent
  leur nom et leur sémantique, et la suite de tests reste verte.

## Assumptions

- Les domaines ne portent aucune sémantique de sécurité. Un agent qui déclare un
  domaine n'obtient aucun droit ; le point d'authentification de l'émetteur,
  identifié lors de la session 004, reste ouvert et sort de ce périmètre.
- La communication entre domaines différents reste autorisée. Le domaine
  informe, il ne cloisonne pas.
- La durée de sécurité par défaut du statut est d'une heure — assez longue pour
  couvrir une tâche de fond, assez courte pour qu'un oubli se répare seul.
- Le statut est déclaré par l'agent lui-même, depuis son propre contexte. Mettre
  un tiers en « ne pas déranger » à sa place n'est pas prévu.

## Dépendances avec les specs existantes

- **`001-renommer-agent`** : la surcharge de domaine réutilise le mécanisme
  d'identification de l'agent courant et le motif de persistance sur disque du
  nom.
- **`002-federation-ssh`** : le domaine est dérivé sur l'hôte de l'agent et
  remonte comme l'hôte, l'OS et le transport. La persistance du domaine
  surchargé suit la même exigence de survie aux coupures que le nom.
- **`003-cycle-vie-demandes`** : la suspension des rappels d'escalade (FR-012)
  porte sur le mécanisme introduit par cette spec.
- **`004-runtime-modele-agents`** : le domaine et le statut étendent la présence
  exactement comme le modèle et l'effort, et suivent la même règle d'affichage
  d'une valeur inconnue.
