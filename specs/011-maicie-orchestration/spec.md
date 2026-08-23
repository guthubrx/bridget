# Feature Specification : Maicie v3, coordinatrice d'orchestration légère

**Feature Branch**: `session-11-maicie-orchestration` (à créer avant toute implémentation)  
**Created**: 2026-08-22  
**Status**: Planifiée — implémentation explicitement exclue de cette session  
**Input**: Construire, au-dessus de Bridget et sans reprendre Maicie historique,
une couche d'orchestration légère. Maicie doit pouvoir coordonner une délégation
complète tout en laissant l'utilisateur converser directement avec les agents.
Tous ses développements futurs appartiennent à `plugins/maicie/`.

## Contexte

Bridget est le transport local fiable : agents présents, messages, demandes
suivies et délais de réponse. Le dogfooding montre que cette communication
directe est précieuse parce qu'elle n'enferme pas l'utilisateur dans un
workflow. Maicie v3 ajoute donc une coordinatrice, pas un carcan : elle porte
un objectif, choisit qui solliciter et rend une synthèse, sans confisquer les
conversations ni devenir la vérité du transport.

L'ancien Maicie apporte des enseignements — état durable, distinction entre
échec et blocage, décisions rattachées au travail — mais son superviseur,
pipeline fixe, worktrees et ordonnanceur ne sont pas repris.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Déléguer sans perdre la conversation (Priority: P1)

L'utilisateur confie un objectif à Maicie, qui reste joignable, choisit un
agent disponible ou celui que l'utilisateur désigne, puis en suit la réponse.
Pendant toute cette période, l'utilisateur peut parler directement à Maicie ou
à n'importe quel agent. Ces échanges ne créent pas de tâche implicite et ne
réécrivent pas l'objectif sans commande explicite.

**Why this priority**: c'est le bénéfice immédiat recherché : déléguer quand
c'est utile sans perdre la fluidité actuelle de Bridget.

**Independent Test**: démarrer Maicie et deux agents Bridget ; déléguer un
objectif court ; constater la demande suivie, la vue Maicie et la synthèse ;
envoyer en parallèle un message direct à un agent et vérifier que l'état de
l'objectif ne change pas.

**Acceptance Scenarios**:

1. **Given** Maicie est connectée et un agent est disponible, **When**
   l'utilisateur lui délègue un objectif, **Then** elle crée un objectif
   coordonné, sollicite l'agent par Bridget et enregistre qui a reçu quoi.
2. **Given** un objectif coordonné actif, **When** l'utilisateur envoie un
   message direct à un agent, **Then** le message est livré normalement sans
   modifier l'objectif ni ses délégations.
3. **Given** un objectif coordonné actif, **When** l'utilisateur demande à
   Maicie de solliciter un agent nommé, **Then** cette nouvelle délégation est
   visible dans l'état Maicie avec son motif et son statut.
4. **Given** Maicie est redémarrée, **When** elle revient, **Then** elle
   retrouve ses objectifs ouverts et recompose leur état de livraison depuis
   Bridget sans jamais lire sa base SQLite.

---

### User Story 2 — Voir des faits, pas des suppositions (Priority: P2)

L'utilisateur peut savoir si un agent est présent, inactif, en cours de tour
ACP, a demandé une permission déjà traitée par le registre Bridget/ACP, ou a
produit une réponse. Maicie expose ces faits et ne prétend jamais déduire un
blocage, une compétence ou une demande de décision depuis du texte de terminal.

**Why this priority**: une coordinatrice qui invente le statut d'un agent est
plus nuisible qu'une coordinatrice qui avoue l'incertitude.

**Independent Test**: simuler des événements ACP de tour, une permission
auto-décidée et une interruption de flux ; vérifier que chaque information est
affichée avec source, fraîcheur et séquence, et que l'absence de signal reste
`inconnu`.

**Acceptance Scenarios**:

1. **Given** un équipier ACP exécute un tour, **When** Maicie observe son état,
   **Then** elle l'affiche comme activité runtime, distincte de la disponibilité
   Bridget et de l'état d'objectif.
2. **Given** une permission ACP est journalisée, **When** Maicie l'observe,
   **Then** elle l'affiche comme « permission demandée puis auto-décidée » avec
   son issue, jamais comme une décision humaine encore pendante.
3. **Given** aucun fait public récent ou un `Gap` d'abonnement, **When** Maicie
   construit sa vue, **Then** elle affiche `inconnu` ou `flux incomplet`, jamais
   `bloqué` comme un fait.

---

### User Story 3 — Délai réaliste et relance proportionnée (Priority: P2)

L'utilisateur choisit une classe de durée courte, normale ou longue pour une
délégation. Cette durée est affichée par Maicie et devient le délai de demande
Bridget correspondant ; elle n'est pas une promesse prise au mot par un modèle.
Bridget est la seule horloge active : Maicie ne relance, ne clôture ni ne crée
de décision de façon autonome à sa propre échéance.

**Why this priority**: les estimations verbales de modèles sont souvent trop
longues ; un délai fixe de deux minutes est tout aussi faux pour les tâches
longues.

**Independent Test**: créer trois délégations de classes distinctes ; vérifier
les délais transmis à Bridget, puis observer que seul l'événement terminal ou
une consultation utilisateur change l'affichage de coordination.

**Acceptance Scenarios**:

1. **Given** une délégation `courte`, `normale` ou `longue`, **When** elle est
   envoyée, **Then** Bridget reçoit le timeout associé et Maicie conserve la
   classe et l'échéance sémantique.
2. **Given** un fait ACP récent indique un tour actif, **When** l'utilisateur
   consulte Maicie, **Then** elle l'affiche comme observation fraîche au lieu
   de qualifier la demande de bloquée.
3. **Given** Bridget émet l'issue terminale d'une demande, **When** Maicie la
   réconcilie, **Then** elle la présente à l'utilisateur sans relance locale.

---

### User Story 4 — Solliciter une personnalité compétente, avec consentement (Priority: P3)

L'utilisateur maintient des profils nommés — par exemple Prospective ou
Sentry — qui déclarent personnalité, capacités et référence de lancement.
Maicie peut proposer un profil absent dont les tags correspondent explicitement
à la demande ; son réveil passe exclusivement par le contrat public
`SpawnOrder` de la session 009 après approbation explicite et consommable une
seule fois de l'utilisateur.

**Why this priority**: les profils rendent une équipe lisible ; l'activation
automatique est utile plus tard mais trop risquée au démarrage.

**Independent Test**: déclarer un profil sécurité inactif, demander un objectif
de sécurité, puis vérifier que Maicie propose ce profil et n'exécute aucun
lancement avant validation humaine.

**Acceptance Scenarios**:

1. **Given** un objectif requiert une capacité déclarée, **When** aucun agent
   présent ne la propose, **Then** Maicie suggère un profil compatible et
   indique qu'il est inactif.
2. **Given** le profil est inactif, **When** l'utilisateur ne valide pas son
   activation, **Then** aucun `SpawnOrder`, processus ni contexte n'est émis.
3. **Given** l'utilisateur valide une approbation non expirée, **When** le
   profil se connecte via `SpawnOrder`, **Then** Maicie lui délègue le contexte
   minimal dont le hash a été approuvé et trace la consommation unique.

### Edge Cases

- Bridget est arrêté ou inaccessible : Maicie reste joignable mais affiche que
  la livraison ne peut pas être vérifiée ; elle ne simule pas l'envoi.
- Maicie tombe entre l'envoi et l'accusé : une outbox durable contient déjà
  l'identifiant métier ; au redémarrage Maicie réconcilie d'abord cet identifiant
  avec Bridget, puis réessaie exactement le même envoi si l'issue reste inconnue.
- Un même agent est sollicité par deux objectifs : chaque délégation garde sa
  corrélation propre ; aucun état global `occupé` ne tranche seul la priorité.
- L'agent répond hors sujet, répond sans contenu ou disparaît : Bridget porte
  l'issue de livraison ; Maicie marque l'objectif comme `à évaluer`, pas comme
  terminé automatiquement.
- Un profil annoncé a des capacités trop larges : Maicie n'applique que
  l'égalité de tags déclarés ; elle ne déduit pas une compétence de son nom ou
  du texte de l'objectif.
- Les interfaces GUI/TUI futures arrivent après une commande durable : elles ne
  peuvent pas posséder d'état caché distinct.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Maicie MUST être un exécutable compagnon permanent placé sous
  `plugins/maicie/`, et MUST NOT être chargé dans le démon Bridget. Elle MUST
  s'enregistrer sous une identité de coordinatrice stable afin de rester
  joignable par Bridget après son démarrage.
- **FR-002**: Maicie MUST posséder une persistance privée de coordination et
  MUST NOT lire, écrire ou partager `bridget.db`.
- **FR-003**: Bridget MUST rester la source de vérité pour présence, livraison,
  demande suivie, réponse, annulation et timeout.
- **FR-004**: Maicie MUST conserver pour chaque objectif son but, son mode
  (`collaboratif` ou `délégué`), ses délégations, corrélations Bridget, attente
  de synthèse et état de décision.
- **FR-005**: Une conversation directe MUST NOT créer ni modifier implicitement
  un objectif ou une délégation Maicie.
- **FR-006**: L'utilisateur MUST pouvoir demander explicitement à Maicie de
  déléguer un objectif, d'ajouter ou retirer un participant, de demander une
  synthèse et de reprendre la main.
- **FR-007**: Maicie MUST interroger uniquement une interface Bridget publique,
  versionnée et locale ; aucun import de crate interne ni accès socket non
  documenté n'est autorisé.
- **FR-008**: Maicie MUST exposer séparément disponibilité Bridget, faits ACP,
  snapshot de transport, décision de coordination et fraîcheur du flux ; un
  statut absent ou interrompu est `inconnu` ou `flux incomplet`.
- **FR-009**: Maicie MUST NOT inférer `bloqué`, `terminé` ou `besoin de
  décision` depuis une sortie terminal ou l'absence d'événements ACP.
- **FR-010**: Une délégation MUST porter une classe de durée et l'échéance
  Bridget correspondante ; les valeurs initiales sont configurées par
  l'utilisateur et non apprises automatiquement.
- **FR-011**: Maicie MUST traduire la durée choisie vers le timeout Bridget
  existant, sans introduire de scheduler, politique de relance ou état de tâche
  dans Bridget au premier incrément.
- **FR-012**: Maicie MUST corréler une demande Bridget à sa délégation et
  réconcilier son état après redémarrage sans réémettre une demande déjà ouverte.
- **FR-013**: Les profils nommés MUST déclarer nom, tags de capacités,
  personnalité, outils et référence `SpawnOrder` ; ils ne sont pas confondus
  avec une connexion agent éphémère.
- **FR-014**: L'activation d'un profil inactif MUST exiger une approbation
  locale, mono-usage et atomiquement consommée, liée à `objective_id`,
  `profile_id`, hash/version du profil, hash/périmètre de contexte, paramètres,
  acteur `local_human` et expiration. Cet acteur désigne une commande locale
  structurée dans le modèle de confiance mono-utilisateur, non une preuve
  cryptographique. L'approbation MUST NOT être exposée via Bridget ou MCP.
  Maicie MUST utiliser seulement le contrat
  public `SpawnOrder` de session 009 et MUST NOT créer de processus OS. Cette
  activation MUST utiliser une `ActivationOutbox` avec `command_id` idempotent,
  lookup/replay après crash et consommation seulement après issue durable.
- **FR-015**: Les commandes et vues de Maicie MUST être utilisables sans GUI ou
  TUI, et leurs sorties structurées doivent pouvoir alimenter plus tard ces deux
  canaux sans logique métier dupliquée.
- **FR-016**: Maicie MUST journaliser les transitions, corrélations et décisions
  avec un identifiant d'objectif ; les contenus sensibles sont minimisés dans
  les logs.
- **FR-017**: Le premier incrément MUST réutiliser la capacité actuelle de
  timeout de Bridget ; toute API Bridget supplémentaire doit être justifiée par
  un besoin démontré dans les tâches, avec contrat et tests de compatibilité.
- **FR-018**: Un message libre adressé à Maicie MUST être enregistré et affiché
  sans mutation d'objectif, de délégation ni de décision. Maicie répond par une
  aide structurée invitant aux commandes CLI ; seul `maicie delegate` ou une
  confirmation CLI explicite peut créer un objectif.
- **FR-019**: Quand aucune cible n'est imposée, Maicie MUST appliquer une
  politique de sélection visible et déterministe : égalité de tags de capacité,
  puis délégation à l'unique agent compatible disponible, ou proposition de
  candidats si plusieurs restent. Une compréhension sémantique ou sélection
  opaque est interdite.
- **FR-020**: Avant tout envoi suivi, Maicie MUST persister une outbox avec un
  identifiant métier généré localement. L'envoi Bridget MUST accepter cet
  identifiant de façon idempotente ; les états `prepared`, `outcome_unknown`,
  `accepted` et `rejected` MUST permettre de reprendre après crash sans doublon.
  `InvalidIssuedAt` est un refus terminal `rejected`, jamais une incertitude
  rejouable. L'outbox
  MUST contenir l'enveloppe immuable exacte (cible, bytes du corps, reply,
  timeout/échéance et métadonnées), et tout état non terminal MUST effectuer
  lookup puis replay exact du même id.
- **FR-021**: Les faits ACP MUST être lus exclusivement par l'abonnement public
  de session 008 et porter `subscription_id`, séquence, fraîcheur et éventuel
  `Gap`/`End`. Le snapshot transport est dérivé et ne peut jamais déclencher une
  transition de coordination seul.
- **FR-022**: Le MVP MUST NOT interpréter des messages libres ni invoquer un
  modèle pour la sélection, la clarification ou la synthèse. `summarize` MUST
  être une agrégation factuelle des réponses corrélées, modifiable ou validable
  explicitement par l'utilisateur.
- **FR-023**: Bridget et SpawnOrder MUST conserver leur déduplication/tombstone
  au moins jusqu'à l'horizon de retry respectif de Maicie. Au-delà, Maicie MUST
  produire `idempotency_expired` et MUST NOT réémettre l'ordre ou le message.

### Key Entities

- **Objectif coordonné**: intention explicitement confiée à Maicie, distincte
  d'une conversation et de la livraison.
- **Délégation**: demande adressée à un participant, liée à un objectif et à
  une corrélation Bridget, avec durée et état sémantique limité.
- **Observation runtime**: fait horodaté provenant d'ACP ou de Bridget, jamais
  confondu avec un jugement métier.
- **Profil d'équipe**: personnalité durable et capacités déclarées, indépendante
  de l'instance actuellement connectée.
- **Décision**: choix humain ou de coordination explicitement demandé, avec
  motif et effet sur l'objectif.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Après crash entre l'acceptation Bridget et sa persistance de retour,
  100 % des délégations d'une fixture sont réconciliées ou réessayées avec le
  même identifiant métier, sans doublon Bridget ni perte de corrélation.
- **SC-002**: Une conversation directe effectuée pendant un objectif actif ne
  produit aucune modification de cet objectif dans les journaux Maicie.
- **SC-003**: Chaque snapshot affiché dans une fixture comporte une source parmi
  `bridget`, `acp_subscription`, `inconnu` ou `flux_incomplet`, avec fraîcheur
  et séquence lorsque la source est ACP.
- **SC-004**: Les trois classes de durée transmettent des timeouts Bridget
  distincts, configurés et vérifiés de bout en bout.
- **SC-005**: Un profil inactif n'est jamais lancé dans les tests sans une
  décision humaine explicite enregistrée.
- **SC-006**: Les sorties structurées de commande permettent de reproduire les
  scénarios P1 sans GUI ni TUI ; aucune dépendance DSH ou T3 Code n'est ajoutée.
- **SC-007**: Les tests unitaires, d'intégration et de contrat du plugin sont
  verts, et `cargo clippy --all-targets -- -D warnings` est vert avant livraison.
- **SC-008**: Le benchmark de consultation sur 100 objectifs satisfait p95 <
  250 ms, avec commande reproductible et résultat consigné.
- **SC-009**: Les barrières de crash avant écriture socket, après écriture avant
  Ack et après Ack avant commit local ne produisent ni double message ni double
  SpawnOrder ; les payloads rejoués sont identiques aux bytes persistés.

## Assumptions

- La session 008 fournit l'abonnement public fiable requis avant toute lecture
  d'événement ACP par Maicie ; la session 007 seule est insuffisante.
- La session 009 fournit le contrat `SpawnOrder` requis avant toute activation
  de profil ; Maicie ne gère pas un cycle de processus.
- Les commandes publiques Bridget nécessaires à la présence, aux demandes et
  aux envois peuvent être stabilisées derrière un contrat local avant le code
  Maicie.
- GUI, TUI, auto-réveil, auto-sélection opaque, compréhension LLM,
  historique prédictif de durée, DAG et ordonnanceur restent explicitement hors
  du premier incrément.
