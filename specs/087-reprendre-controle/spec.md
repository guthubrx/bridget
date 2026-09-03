# Feature Specification: Reprendre le contrôle

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 087-reprendre-controle
Titre: Reprendre le contrôle du coordinateur
Statut: Draft
Priorité: P1
Créée: 2026-09-02
Dépendances: ADR-014, SPEC-063, SPEC-064, SPEC-068, SPEC-070, SPEC-079, SPEC-080, SPEC-081-pilotage-rondes-projet-ui
Résumé:
- Contexte: le coordinateur sait s'ouvrir du travail (routines, rondes, réassignations) mais le référent n'a aucun verbe pour l'arrêter, le prioriser ou être prévenu ; 100 objectifs créés pour 26 clos sur une fenêtre mesurée, et un référent qui « n'était plus écouté ».
- Objectif: donner au référent quatre leviers durables et visibles : mettre en pause l'autonomie, imposer un objectif prioritaire d'origine humaine attestée, recevoir dans une boîte durable tout ce qui exige sa décision, et borner le travail auto-généré par un budget.
- Risque principal: un levier qui paraît appliqué sans l'être (pause qui laisse partir une ronde, focus contourné par une routine, item de boîte perdu), ou une provenance humaine forgeable par un agent.
- Mitigation: chaque levier est un état persistant lu à la source par tous les producteurs de travail autonome, avec preuve observable et refus explicites ; la voie humaine passe par l'attestation d'origine déjà conçue (ADR-014), jamais par une simple déclaration.
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `087-reprendre-controle`
**Created**: 2026-09-02
**Status**: Draft
**Input**: User description: "Reprendre le contrôle : verbes pause et focus, boîte de réception humaine, budget d'objectifs auto-générés, voie HumanRequest de l'ADR 014 branchée. Le coordinateur s'est emballé, a créé des tâches en chaîne et ne m'écoutait plus ; je n'avais aucun moyen de lui dire : arrête, travaille pour moi là-dessus, tu reprendras tes trucs plus tard."

## Contexte observé

Le produit a été conçu autour d'un coordinateur qui ne s'endort pas : rondes toutes les sept minutes, relève toutes les deux minutes, routines qui ouvrent des objectifs, réassignations automatiques quand un agent ne répond pas. Ces mécanismes fonctionnent. Ce qui manque est la moitié symétrique : rien ne permet au référent de suspendre cette autonomie, de la subordonner à sa propre demande, ou d'être prévenu quand le système attend sa décision.

Faits mesurés le 2026-09-02 :

- toutes les ouvertures d'objectifs sont aujourd'hui d'origine automatique ; la voie « demandé par l'humain » est représentable mais aucun appelant ne peut l'obtenir, faute d'attestation émise par le daemon (ADR-014, tranche 1) ;
- l'état « intervention humaine requise » et la décision associée existent, mais aucun canal ne les porte hors de la base : un agent a dû déposer un fichier dans le dépôt pour joindre le référent ;
- la ronde constate et ne décide rien par doctrine ; sans « second geste » humain, le tour de garde n'a pas lieu ;
- sur la fenêtre D des métriques d'efficience, 100 objectifs ont été créés pour 26 clos.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Mettre l'autonomie en pause (Priority: P1)

Le référent constate que le coordinateur enchaîne des ouvertures de travail qu'il n'a pas demandées. Il déclenche une pause en un seul geste, depuis l'interface ou la ligne de commande. À partir de cet instant, plus aucun travail autonome ne démarre : aucune routine n'ouvre d'objectif, aucune ronde ne réveille d'agent, aucune réassignation automatique ne crée de génération. La pause interrompt les tours d'agents en cours, mémorise leurs exécutions et les reprend à la levée pour les agents connectés et libres, ou à leur prochaine reconnexion. L'état « en pause » est visible partout où l'on regarde le système, survit à un redémarrage, et ne se lève que par un geste explicite du référent.

**Why this priority**: c'est le frein d'urgence. Sans lui, les trois autres leviers peuvent être contournés par le flux autonome avant même d'être appliqués.

**Independent Test**: activer la pause, laisser passer une occurrence de routine et une fenêtre de ronde, vérifier qu'aucun objectif ni message de réveil n'a été créé, redémarrer le service, vérifier que la pause est toujours affichée, lever la pause, vérifier que la prochaine occurrence repart.

**Acceptance Scenarios**:

1. **Given** une routine dont l'occurrence est due et une ronde active sur un projet, **When** le référent active la pause, **Then** l'occurrence est consignée comme différée avec le motif « pause » et la ronde n'émet aucun réveil, et les deux faits sont lisibles dans l'état du système.
2. **Given** un agent en plein tour, **When** la pause est activée, **Then** il reçoit une interruption et son exécution est mémorisée ; **When** la pause est levée, **Then** elle lui est remise à nouveau s'il est connecté et libre, sinon à sa reconnexion.
3. **Given** la pause active, **When** le service Bridget ou le coordinateur redémarre, **Then** la pause est toujours active et affichée après redémarrage.
4. **Given** la pause active, **When** le référent envoie lui-même un message ou ouvre un focus, **Then** son action est acceptée : la pause ne bride que l'autonomie, jamais le référent.
5. **Given** la pause active, **When** un agent tente de lever la pause, **Then** la demande est refusée avec un motif explicite et le refus est consigné.

---

### User Story 2 - Imposer un objectif prioritaire (Priority: P1)

Le référent écrit une phrase dans l'interface : « Travaille sur X dans le projet Y ». Le système ouvre un objectif dont l'origine est attestée comme humaine, le marque comme focus, choisit les agents disponibles du projet, gèle lui-même la base de travail et transmet lui-même aux agents tout ce qu'ils doivent connaître pour livrer. Tant que ce focus est ouvert, aucun objectif auto-généré non commencé n'est dispatché, et tout agent qui se libère est proposé au focus avant tout autre travail. Le référent n'a saisi ni identifiant, ni empreinte de commit, ni nom d'agent.

**Why this priority**: c'est la demande d'origine, « travaille pour moi là-dessus, tu reprendras tes trucs plus tard ». Elle rend au référent la position de client du système.

**Independent Test**: depuis l'interface, saisir une phrase et un projet, vérifier qu'un objectif d'origine humaine existe avec le statut focus, qu'au moins une délégation est partie vers un agent du projet avec sa base gelée, et qu'une occurrence de routine due pendant ce temps est différée avec le motif « focus ».

**Acceptance Scenarios**:

1. **Given** un projet actif avec deux agents disponibles, **When** le référent soumet « Travaille sur X » pour ce projet, **Then** un objectif est ouvert avec une origine humaine attestée liée au message du référent, marqué focus, et une délégation est remise à un agent du projet avec sa base gelée calculée par le système.
2. **Given** un focus ouvert, **When** une routine ou une ronde veut ouvrir ou réveiller du travail auto-généré, **Then** ce travail est différé avec le motif « focus », sans être perdu, et repart quand le focus se ferme.
3. **Given** un focus ouvert, **When** un second focus est demandé, **Then** le système demande une confirmation explicite : remplacer le focus courant, ou le mettre en file derrière. Un seul focus est actif à la fois.
4. **Given** un message qui prétend venir de l'humain mais dont l'émetteur observé n'est pas l'identité humaine adressable, **When** une ouverture avec origine humaine est demandée, **Then** elle est refusée avec le motif de refus de l'attestation, et aucun objectif n'est ouvert.
5. **Given** un focus livré et clos, **When** le référent consulte l'historique, **Then** l'objectif conserve son origine humaine et le lien vers le message d'origine.

---

### User Story 3 - Recevoir ce qui attend une décision (Priority: P1)

Chaque fois que le système a besoin du référent, l'événement arrive dans une boîte de réception durable et unique : intervention humaine requise, chaîne de réassignation épuisée, verdict de revue à arbitrer, approbation d'activation demandée, budget atteint, dette de réponse due. La boîte est visible dans l'interface avec le nombre d'items ouverts, et chaque item peut être poussé vers un canal externe personnel que seul le référent lit. Le référent traite un item en l'accusant ou en décidant, et sa décision revient au système qui l'a demandée.

**Why this priority**: sans canal sortant, tout état d'attente humaine dort dans une base. C'est le trou fonctionnel qui rend le coordinateur sourd en pratique.

**Independent Test**: provoquer une réassignation jusqu'à épuisement de la chaîne, vérifier qu'un item apparaît dans la boîte en moins de deux minutes avec son contexte, le traiter, vérifier que la délégation concernée reflète la décision.

**Acceptance Scenarios**:

1. **Given** une délégation dont la chaîne de réassignation est épuisée, **When** le système l'enregistre, **Then** un item « intervention requise » apparaît dans la boîte avec l'objectif, la délégation, l'historique des tentatives et les décisions possibles.
2. **Given** un item ouvert, **When** le canal externe est configuré et joignable, **Then** le référent reçoit une notification qui contient l'essentiel de l'item et un moyen de revenir à la boîte.
3. **Given** un canal externe injoignable, **When** un item est créé, **Then** l'item existe dans la boîte, l'échec de notification est consigné, et rien n'est perdu.
4. **Given** un item traité par le référent, **When** un agent tente de traiter un item de la boîte, **Then** la tentative est refusée : seul le référent décide.
5. **Given** un même événement produit deux fois, **When** il est déposé, **Then** la boîte ne contient qu'un item, et la seconde occurrence lui est rattachée.
6. **Given** des items ouverts depuis plus d'un délai configurable, **When** la ronde passe, **Then** elle a le droit de rappeler ces items au référent par le canal externe, et c'est sa seule décision autorisée.

---

### User Story 4 - Borner le travail auto-généré (Priority: P2)

Le référent fixe un plafond d'objectifs auto-générés ouverts simultanément. Quand le plafond est atteint, le coordinateur n'ouvre plus d'objectif automatique : il consigne l'occurrence comme différée avec le motif « budget » et dépose une demande dans la boîte de réception pour que le référent relève le plafond ou clôture du travail. Le plafond, sa consommation courante et les occurrences différées sont visibles.

**Why this priority**: c'est la garde permanente contre l'emballement, celle qui agit même quand le référent ne regarde pas. Elle vient après la pause parce que la pause est le geste immédiat et le budget la règle de fond.

**Independent Test**: fixer le plafond à N, créer N objectifs automatiques, déclencher une routine, vérifier qu'aucun objectif supplémentaire n'est ouvert, que l'occurrence est différée avec le motif « budget » et qu'un item de boîte a été déposé une seule fois.

**Acceptance Scenarios**:

1. **Given** un plafond de N et N objectifs auto-générés ouverts, **When** une routine veut en ouvrir un autre, **Then** l'ouverture est refusée, l'occurrence différée avec le motif « budget », et un item « budget atteint » est déposé dans la boîte.
2. **Given** le plafond atteint, **When** un objectif auto-généré se ferme, **Then** la prochaine occurrence due peut ouvrir un objectif sans intervention.
3. **Given** le plafond atteint, **When** le référent ouvre un focus, **Then** le focus est accepté : le budget ne compte que l'auto-généré.
4. **Given** aucun plafond configuré, **When** le système démarre, **Then** un plafond par défaut raisonnable s'applique et est affiché comme tel.

---

### User Story 5 - Voir l'état de contrôle d'un coup d'œil (Priority: P2)

Où qu'il soit dans l'interface, le référent voit en permanence : pause active ou non, focus courant s'il existe, nombre d'items ouverts dans la boîte, consommation du budget. Les mêmes faits sont disponibles en ligne de commande dans l'annuaire des agents.

**Why this priority**: un levier invisible finit oublié ; une pause oubliée depuis trois jours est un incident.

**Independent Test**: activer la pause, ouvrir un focus, déposer un item, vérifier que les quatre faits sont affichés sur l'écran principal et rendus par la commande d'annuaire, puis vérifier qu'ils disparaissent quand chaque levier est levé.

**Acceptance Scenarios**:

1. **Given** la pause active depuis plus de 24 heures, **When** le référent ouvre l'interface, **Then** l'état de pause est affiché avec sa durée, sans qu'il ait à chercher.
2. **Given** un focus ouvert, **When** le référent regarde la liste des objectifs, **Then** le focus est distingué visuellement et placé en tête.

---

### Edge Cases

- Pause activée pendant qu'une délégation est en cours de remise : la remise déjà réservée se termine, aucune nouvelle remise autonome ne part.
- Pause activée pendant un tour fournisseur : le parent interrompu reste la seule exécution reconstruisible par la reprise de pause ; un agent occupé ou absent conserve cette reprise jusqu'à redevenir libre ou se reconnecter.
- Pause activée puis levée entre deux occurrences de routine : les occurrences différées pendant la pause ne sont pas rejouées en rafale ; seule la prochaine occurrence due repart.
- Focus ouvert alors que tous les agents du projet sont occupés : le focus attend, il est proposé au premier agent qui se libère, et un item « focus en attente d'agent » est déposé dans la boîte après un délai configurable.
- Focus demandé sur un projet sans agent missionnable : refus explicite avec la liste de ce qui manque, aucun objectif ouvert.
- Message humain rejoué : l'attestation est à usage unique, la seconde ouverture est refusée.
- Item de boîte dont l'objet a disparu entre le dépôt et le traitement, par exemple une délégation annulée : l'item se ferme seul avec le motif « objet disparu » et le référent le voit dans l'historique.
- Canal externe qui répond mais tronque : le contenu poussé est un résumé borné, jamais le corps intégral ; la boîte reste la référence.
- Budget abaissé sous la consommation courante : rien n'est fermé de force, aucune nouvelle ouverture jusqu'au retour sous le plafond.
- Deux référents ou deux interfaces ouvertes : les leviers sont des états uniques ; le dernier geste gagne et l'autre interface le reflète.
- Le coordinateur plante entre la lecture d'une décision du référent et son application : la décision n'est jamais perdue ; elle est relue à la relève suivante jusqu'à ce que son application soit confirmée.
- Une connexion s'enregistre sous l'identité humaine alors que l'interface est encore connectée : le référent est prévenu, et l'événement est consigné.

## Requirements *(mandatory)*

### Functional Requirements

Pause

- **FR-001**: Le référent DOIT pouvoir activer et lever une pause de l'autonomie depuis l'interface et depuis la ligne de commande, en une action, sans paramètre obligatoire.
- **FR-002**: Pendant la pause, le système NE DOIT PAS ouvrir d'objectif auto-généré, émettre de réveil de ronde, ni créer de génération de réassignation ; chaque occurrence empêchée DOIT être consignée avec le motif « pause ».
- **FR-002b**: La pause DOIT interrompre le tour en cours et sa levée DOIT reprendre l'exécution interrompue pour chaque agent connecté et libre.
- **FR-003**: La pause DOIT survivre au redémarrage de chaque composant et DOIT rester active jusqu'à un geste explicite du référent.
- **FR-004**: La pause NE DOIT PAS bloquer les actions du référent : messages, focus, clôtures, approbations restent possibles.
- **FR-005**: Seul le référent DOIT pouvoir lever la pause ; toute tentative d'un agent DOIT être refusée avec un motif consigné.

Focus

- **FR-010**: Le référent DOIT pouvoir ouvrir un objectif prioritaire à partir d'une phrase et d'un projet, sans saisir d'identifiant, d'empreinte de commit ni de nom d'agent.
- **FR-011**: Un objectif ouvert par le référent DOIT porter une origine humaine attestée, liée au message d'origine, produite selon les cinq vérifications déjà définies par l'ADR-014 ; une simple déclaration d'origine humaine DOIT être refusée. Cette garantie vaut dans le modèle coopératif du produit : elle arrête un agent qui agit par ses outils déclarés, pas un processus hostile du même compte (ADR-003, ADR-011). Toute reprise de l'identité humaine par une nouvelle connexion alors que la précédente est encore vivante DOIT être signalée au référent par la boîte de réception et son canal externe.
- **FR-012**: Le système DOIT choisir les agents du focus parmi les agents missionnables du projet, geler lui-même la base de travail et transmettre lui-même aux agents les identifiants nécessaires à la livraison.
- **FR-013**: Tant qu'un focus est ouvert, le travail auto-généré non commencé DOIT être différé avec le motif « focus » et repartir à la fermeture du focus.
- **FR-014**: Un seul focus DOIT être actif à la fois ; une seconde demande DOIT exiger une confirmation explicite entre remplacer et mettre en file.
- **FR-015**: Un agent qui se libère DOIT être proposé au focus avant tout autre travail.

Boîte de réception humaine

- **FR-020**: Le système DOIT tenir une boîte de réception durable et unique des événements qui exigent le référent, couvrant au minimum : intervention humaine requise, chaîne de réassignation épuisée, verdict de revue à arbitrer, approbation d'activation demandée, budget atteint, dette de réponse due, focus en attente d'agent.
- **FR-021**: Chaque item DOIT porter son contexte, ses décisions possibles, son horodatage et le lien vers l'objet concerné ; un même événement NE DOIT PAS produire deux items.
- **FR-022**: La boîte DOIT être visible dans l'interface avec le nombre d'items ouverts, et lisible en ligne de commande.
- **FR-023**: Le référent DOIT pouvoir configurer un canal externe personnel vers lequel chaque item est poussé sous forme de résumé borné ; l'indisponibilité du canal NE DOIT jamais empêcher la création de l'item et DOIT être consignée.
- **FR-024**: Seul le référent DOIT pouvoir traiter un item ; sa décision DOIT être transmise au composant qui a déposé l'item et DOIT être consignée.
- **FR-025**: La ronde DOIT pouvoir rappeler au référent les items ouverts depuis plus d'un délai configurable, et cette décision DOIT être la seule qu'elle prend.

Budget

- **FR-030**: Le référent DOIT pouvoir fixer un plafond d'objectifs auto-générés ouverts simultanément ; un plafond par défaut DOIT s'appliquer en l'absence de réglage et être affiché comme tel.
- **FR-031**: Au plafond, aucune ouverture auto-générée NE DOIT avoir lieu ; l'occurrence DOIT être différée avec le motif « budget » et un item « budget atteint » DOIT être déposé une seule fois par épisode de saturation.
- **FR-032**: Le budget NE DOIT compter que les objectifs d'origine automatique ; les objectifs d'origine humaine n'y entrent pas.

Visibilité et traçabilité

- **FR-040**: L'état des quatre leviers, pause, focus courant, items ouverts, consommation du budget, DOIT être affiché en permanence dans l'interface et rendu par la commande d'annuaire.
- **FR-041**: Chaque changement d'état d'un levier DOIT être consigné avec son auteur, son horodatage et son motif, et DOIT être relisible dans l'historique.
- **FR-042**: Tout refus lié à ces leviers DOIT être explicite, avec un motif fermé et lisible par un humain.

### Key Entities *(include if data involves data)*

- **État de contrôle**: l'ensemble des leviers du référent : pause (active, depuis quand, par qui), focus courant, plafond de budget et sa consommation. Un seul par installation, persistant, lu par tout producteur de travail autonome.
- **Focus**: un objectif d'origine humaine attestée désigné comme prioritaire, avec son projet, son message d'origine, ses délégations et son état (en attente d'agent, en cours, livré, clos).
- **Item de boîte de réception**: un événement qui exige le référent, avec son type fermé, son contexte, ses décisions possibles, son état (ouvert, traité, fermé seul), la décision prise, et l'historique de ses notifications externes.
- **Occurrence différée**: le fait qu'un travail autonome n'a pas démarré, avec un motif fermé parmi pause, focus, budget, et l'horodatage ; elle explique un silence au lieu de le laisser muet.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Entre le geste de pause et l'arrêt effectif de toute ouverture autonome, il s'écoule moins d'une minute, et zéro objectif auto-généré n'est ouvert pendant une pause de 24 heures avec routines et rondes actives.
- **SC-002**: Le référent ouvre un focus depuis l'interface en moins de deux minutes, sans saisir aucun identifiant, et la première délégation part vers un agent du projet dans les cinq minutes quand un agent est disponible.
- **SC-003**: 100 % des événements « intervention humaine requise » et « chaîne épuisée » produits sur une semaine d'usage apparaissent dans la boîte de réception en moins de deux minutes, et aucun n'est découvert par un autre moyen.
- **SC-004**: Sur une fenêtre de mesure quotidienne, le rapport entre objectifs auto-générés créés et clos passe sous 2 pour 1, contre 100 pour 26 mesuré sur la fenêtre D.
- **SC-005**: Zéro ouverture d'objectif à origine humaine obtenue par un agent agissant par ses outils déclarés (outil MCP, ligne de commande documentée) lors d'un essai adverse où il tente de forger la provenance ; et toute reprise de l'identité humaine par une connexion concurrente produit un signalement au référent en moins d'une minute.
- **SC-006**: Un référent qui n'a jamais utilisé la ligne de commande peut activer la pause, ouvrir un focus et traiter un item de boîte en s'appuyant uniquement sur l'interface, sans documentation, lors d'un essai réel consigné.
- **SC-007**: Après la levée de pause, chaque agent connecté et libre reçoit son exécution interrompue avant toute nouvelle trame de travail qui lui serait destinée ; les exécutions d'agents occupés ou absents restent attestées jusqu'à disponibilité ou reconnexion.

## Assumptions

- Le référent est unique par installation ; le multi-référent est hors périmètre.
- La pause agit au niveau de la coordination et du tour fournisseur : elle interrompt les tours via SPEC-063 et ne reprend que les exécutions gelées par elle.
- Un seul focus actif à la fois ; la mise en file d'un second focus est une file d'attente simple, sans priorité relative.
- Le canal externe personnel réutilise le canal Telegram déjà en place hors dépôt ; il est optionnel et n'est pas une condition de fonctionnement de la boîte.
- Le second facteur d'approbation de l'ADR-011 n'est pas livré par cette spec ; la boîte de réception en est le prérequis et doit être conçue pour l'accueillir.
- Le plafond par défaut d'objectifs auto-générés est de 5 ; il est réglable.
- Le gel de la base de travail par le système réutilise le mécanisme de cible de revue existant ; il ne crée pas de second mécanisme.
- Les leviers sont lus à la source par chaque producteur de travail autonome ; un producteur qui ne les lit pas est un défaut de cette spec, pas une limite acceptée.

## Hors périmètre

- Un modèle de permissions par agent (shell, réseau, écriture) : sujet distinct.
- Des budgets exprimés en tokens ou en coût : le budget de cette spec compte des objectifs.
- La consolidation technique du daemon et des bases : sujet distinct.
- La fusion, le push ou le déploiement automatiques après un focus livré : le rituel de clôture reste humain.

## Dépendances avec l'existant

- ADR-014 : provenance des objectifs et attestation d'origine humaine ; cette spec livre la voie « demandé par l'humain » attendue par ses tranches 2 et 3.
- ADR-011 : second facteur différé ; la boîte de réception en est le canal futur.
- SPEC-063 : interruption et pilotage d'un tour humain ; la pause s'y appuie sans la dupliquer.
- SPEC-064 : plan de contrôle ; les opérations de contrôle d'exécution existantes sont réutilisées pour ne pas créer de tour autonome pendant la pause.
- SPEC-068 et SPEC-070 : incidents délégués et preuves d'activité ; les faits qu'elles produisent alimentent la boîte.
- SPEC-079 et SPEC-081-pilotage-rondes-projet-ui : rondes par projet ; la pause et le rappel d'items s'y greffent.
- SPEC-080 : centre de contrôle ; les réglages de budget et de canal externe y prennent place.
