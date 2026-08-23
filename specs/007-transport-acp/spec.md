# Feature Specification : Transport ACP pour agents équipiers

**Feature Branch**: `session-07-transport-acp` (à créer au démarrage de l'implémentation)
**Created**: 2026-08-22
**Status**: Clôturée — transport ACP livré, revu et mergé ; socle des sessions 008-013
**Input**: User description: "Remplacer l'artifice d'injection terminal (💬 + règles de prompt) par une livraison structurée des messages via l'Agent Client Protocol, pour les agents équipiers headless. Priorité Codex, puis Claude, puis Gemini. Ouvert à l'ajout d'autres agents a posteriori sans modification de code."

## Contexte et problème

Aujourd'hui, Bridget livre les messages entrants en tapant une ligne `💬` dans
le terminal de l'agent (send-keys tmux), et obtient les réponses grâce à un bloc
de « Règles ABSOLUES » injecté dans le prompt de session, qui ordonne à l'agent
de répondre via la commande `bridget send`. Ce montage fonctionne mais présente
quatre défauts structurels :

1. **Fiabilité dégradable** : les règles de prompt s'affaiblissent avec la
   longueur de session (compaction de contexte) ; un agent peut répondre dans
   son terminal sans que l'expéditeur ne voie rien.
2. **Fragilité d'échappement** : la réponse transite par une commande shell ;
   apostrophes, guillemets et sauts de ligne cassent ou tronquent les messages.
3. **Cécité d'état** : Bridget ne sait pas si l'agent travaille ou est inactif ;
   les relances des demandes suivies sont aveugles.
4. **Hors tmux, rien ne marche** : la livraison de repli (affichage stderr)
   n'atteint jamais l'agent.

L'Agent Client Protocol (ACP) fournit un canal officiel et structuré vers les
agents CLI : livraison d'un message = requête de prompt, réponse = événement de
fin de tour typé. Cette feature fait d'ACP le mode de livraison des agents
« équipiers » (agents headless pilotés par Bridget), tmux restant le repli pour
les agents interactifs.

## Dépendances

- **002-federation-ssh** : la fédération est inchangée ; le transport ACP est
  strictement local entre le wrapper et son agent.
- **003-cycle-vie-demandes** : la clôture automatique des demandes suivies
  s'appuie sur le cycle de vie existant (aucune nouvelle sémantique de demande).
- **005-domaines-dnd** : l'état occupé/disponible est alimenté par l'état de
  session ACP au lieu d'être déclaré.
- **006-identite-agent-robuste** (en cours) : l'identité persistante des agents
  doit fonctionner pour des agents sans terminal.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Équipier Codex en ACP, sans artifice (Priority: P1)

L'utilisateur lance un équipier Codex headless. Un autre agent lui envoie une
demande suivie. Le message est livré à l'équipier comme un tour de conversation
structuré (expéditeur et attente de réponse en métadonnées), l'équipier répond
naturellement, et sa réponse est automatiquement routée vers l'expéditeur, ce
qui clôt la demande suivie — sans qu'aucune règle de prompt Bridget n'ait été
injectée, et sans que l'équipier n'exécute la moindre commande `bridget`.

**Why this priority** : c'est la suppression du défaut de fond (artifice
d'injection + discipline de prompt), sur l'agent prioritaire désigné par
l'utilisateur (Codex). À elle seule, cette story constitue un MVP démontrable.

**Independent Test** : lancer un équipier Codex, lui envoyer `bridget send --to
<équipier> --reply "question"` depuis un terminal humain, constater la réponse
reçue et la demande close dans le ledger, puis vérifier qu'aucun bloc de règles
Bridget n'apparaît dans le contexte de l'équipier.

**Acceptance Scenarios**:

1. **Given** un équipier Codex ACP enregistré dans l'annuaire, **When** un agent
   lui envoie un message avec réponse attendue, **Then** la réponse de
   l'équipier parvient à l'émetteur et la demande suivie passe à l'état clos,
   sans action manuelle ni commande exécutée par l'équipier.
2. **Given** un message contenant apostrophes, guillemets, `$`, backticks et
   sauts de ligne, **When** il est livré à l'équipier et que celui-ci répond
   avec un contenu tout aussi riche, **Then** les deux contenus arrivent
   intacts, octet pour octet.
3. **Given** un équipier Codex ACP, **When** on inspecte son contexte de
   session, **Then** aucune « Règle ABSOLUE » Bridget n'y figure.
4. **Given** l'équipier est en plein tour de travail, **When** un second message
   arrive, **Then** il est mis en attente et livré à la fin du tour, sans
   interrompre ni corrompre le tour en cours.

---

### User Story 2 - Registre d'agents ouvert : Claude, puis un type inconnu du code (Priority: P2)

L'utilisateur déclare les types d'agents ACP dans une configuration (type →
commande de lancement). Claude est ajouté comme deuxième type supporté. Puis un
type **absent du code de production** est activé par une simple entrée de
configuration, sans modification de Bridget — c'est le test d'acceptation de
l'ouverture demandée. *(Note de compatibilité conditionnelle : Gemini, cible
initiale de ce troisième type, dépend d'un compte Google éligible — la voie
individuelle est fermée par Google depuis le constat T708 du 2026-08-22 ; son
entrée de registre est conservée, documentée non validée.)*

**Why this priority** : l'ouverture « ajouter d'autres agents a posteriori »
est une exigence explicite de l'utilisateur ; elle se vérifie au moment où le
deuxième type passe et où un type inconnu du code traverse tout le cycle.

**Independent Test** : après le support Claude, ajouter dans une fixture de
registre l'entrée d'un type pointant vers l'adaptateur de test stdio et
dérouler le cycle complet de la User Story 1 avec ce type, en vérifiant
mécaniquement qu'aucune ligne hors test/fixture n'a changé.

**Acceptance Scenarios**:

1. **Given** la configuration contient une entrée Claude, **When** l'utilisateur
   lance un équipier Claude, **Then** le scénario de la User Story 1 passe à
   l'identique.
2. **Given** le support Codex et Claude fonctionne, **When** un type d'agent
   supplémentaire — inconnu du code — est déclaré par configuration seule,
   **Then** un équipier de ce type passe le scénario de la User Story 1 **sans
   aucune modification de code de production**. *(Révision du 2026-08-22,
   gate de support : ce scénario visait Gemini, dont la voie individuelle a
   été fermée par Google — « migrate to the Antigravity suite », constaté sur
   CLI 0.46.0 et 0.56.0, consigné dans research.md. La preuve d'ouverture se
   fait par un type s'appuyant sur l'adaptateur de test stdio ; l'entrée
   Gemini reste au registre par défaut, documentée comme indisponible pour les
   comptes individuels — elle resterait fonctionnelle pour un compte
   éligible.)*
3. **Given** une entrée de configuration pointe vers une commande absente ou
   invalide, **When** l'utilisateur lance un équipier de ce type, **Then** le
   lancement échoue immédiatement avec un message qui nomme la commande fautive
   et le fichier de configuration.

---

### User Story 3 - Relances informées et facturation protégée (Priority: P3)

Les relances des demandes suivies distinguent un équipier encore en train de
travailler (pas de relance : inutile) d'un équipier inactif qui n'a pas répondu
(relance justifiée). Par ailleurs, un équipier ACP refuse de démarrer si une
clé API du fournisseur traîne dans l'environnement, afin de garantir que la
consommation passe par l'abonnement (login CLI) et jamais par une facturation
au token.

**Why this priority** : ces deux garanties transforment des promesses
implicites (relances utiles, coûts maîtrisés) en comportements vérifiés. Elles
s'appuient sur les stories précédentes sans les conditionner.

**Independent Test** : (a) envoyer une demande suivie à un équipier occupé par
une tâche longue et vérifier qu'aucune relance n'est émise tant que le tour est
en cours ; (b) exporter une clé API dans l'environnement, tenter de lancer un
équipier, constater le refus motivé.

**Acceptance Scenarios**:

1. **Given** un équipier en plein tour de travail sur une demande, **When**
   l'échéance de relance intermédiaire arrive, **Then** aucune relance ne lui
   est envoyée et l'événement « relance différée : agent au travail » est
   consigné.
2. **Given** un équipier inactif n'ayant pas répondu, **When** l'échéance de
   relance arrive, **Then** la relance est livrée comme un message ordinaire.
3. **Given** une variable d'environnement de clé API du fournisseur est
   définie, **When** l'utilisateur lance un équipier de ce fournisseur, **Then**
   le lancement est refusé avec un message expliquant la protection de
   facturation et comment passer outre explicitement si c'est voulu.

---

### Edge Cases

- **Processus adaptateur qui meurt en plein tour** : la demande suivie en cours
  ne doit pas rester pendante en silence — l'émetteur est prévenu de l'échec, et
  l'annuaire reflète l'état de l'équipier (arrêté), distinct d'`unreachable`.
- **Fin de tour sans contenu de réponse** (tour vide, refus, erreur du modèle) :
  l'émetteur d'une demande suivie reçoit un échec motivé, pas un silence.
- **Demande de permission émise par l'agent pendant un tour** (exécuter une
  commande, écrire un fichier) : un équipier headless n'a personne devant lui ;
  le comportement par défaut doit être défini, sûr et consigné (voir plan).
- **Messages multiples en rafale pendant un tour** : file d'attente ordonnée,
  déduplication existante applicable, pas de perte ni de réordonnancement.
- **Notification (`reply=no`) vs demande (`reply=yes`)** : une notification
  livrée en tour de conversation ne doit pas générer de réponse routée inutile.
- **Protocole incompatible** : la négociation d'ouverture de session échoue →
  échec de démarrage explicite nommant les versions de protocole en présence,
  jamais de dégradation silencieuse. (La version des adaptateurs est garantie
  par leur épinglage dans la configuration, pas par une sonde au lancement.)
- **Arrêt volontaire de l'équipier** (`bridget stop` ou signal) : fin de session
  propre, désenregistrement, demandes pendantes notifiées.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** : Bridget DOIT pouvoir lancer un agent équipier en mode headless
  via le protocole ACP, l'enregistrer dans l'annuaire avec les mêmes attributs
  que les agents existants (nom, type, hôte, OS, domaine, modèle, effort), et
  le désenregistrer proprement à l'arrêt.
- **FR-002** : Un message entrant destiné à un équipier ACP DOIT être livré
  comme un tour de conversation structuré comportant l'expéditeur, l'attente de
  réponse (`reply`) et le corps du message, sans passer par une simulation de
  saisie terminal.
- **FR-003** : La réponse d'un équipier à une demande suivie DOIT être capturée
  à la fin du tour et routée automatiquement vers l'émetteur, en clôturant la
  demande dans le ledger — sans que l'équipier n'exécute de commande.
- **FR-004** : Aucun bloc d'instructions Bridget (« Règles ABSOLUES » ou
  équivalent) NE DOIT être injecté dans le contexte d'un équipier ACP.
- **FR-005** : Le corps des messages DOIT être transmis dans les deux sens sans
  altération (contenu multiligne et caractères spéciaux intacts).
- **FR-006** : Les types d'agents ACP DOIVENT être définis dans une
  configuration déclarative (type → commande et arguments de lancement, la
  version étant épinglée dans la commande elle-même) ; l'ajout d'un nouveau
  type NE DOIT nécessiter aucune modification de code source.
- **FR-007** : Les messages arrivant pendant un tour en cours DOIVENT être mis
  en file **bornée** et livrés dans l'ordre à la fin du tour ; un dépassement de
  capacité produit un échec motivé (asynchrone) remonté à l'émetteur ; un message dont l'échéance
  est dépassée ou dont la demande est annulée est retiré de la file et jamais
  livré en retard ; les garde-fous existants (disjoncteur, déduplication,
  budget de sauts) s'appliquent inchangés.
- **FR-008** : L'état d'activité de l'équipier (tour en cours / inactif) DOIT
  alimenter l'annuaire et le mécanisme de relance : pas de relance pendant un
  tour en cours, relance normale sinon.
- **FR-009** : La mort du processus agent ou une fin de tour en erreur DOIT
  produire, pour toute demande suivie en cours, une notification d'échec motivée
  à l'émetteur — jamais un silence.
- **FR-010** : Le flux d'événements de chaque session ACP (tours, réponses,
  erreurs) DOIT être journalisé et consultable après coup.
- **FR-011** : Le lancement d'un équipier DOIT être refusé si une clé API du
  fournisseur concerné est présente dans l'environnement, avec un message
  explicite ; un contournement volontaire et explicite reste possible.
- **FR-012** : Le transport tmux existant DOIT rester fonctionnel et inchangé
  pour les agents non-ACP ; le choix du transport est déterminé au lancement.
- **FR-013** : La livraison de repli par affichage stderr (inopérante) DOIT être
  supprimée ; tout chemin hérité rendu obsolète par cette feature DOIT être
  supprimé ou inscrit au registre des dépréciations du projet.
- **FR-014** : Les agents fédérés distants (via SSH) DOIVENT pouvoir être des
  équipiers ACP sans changement du mécanisme de fédération.

### Key Entities

- **Équipier ACP** : agent enregistré dans l'annuaire dont le transport est une
  session ACP au lieu d'un terminal ; possède les mêmes attributs d'identité
  que les agents existants, plus un état d'activité (inactif / tour en cours).
- **Entrée de registre d'agents** : déclaration de configuration d'un type
  d'agent ACP — nom du type, commande de lancement et arguments (version
  épinglée dans la commande).
- **Tour de livraison** : unité d'échange avec un équipier — un message entrant
  livré, un flux d'événements, une fin de tour (réponse, vide ou erreur).
- **Journal de session** : trace persistée et consultable des tours d'un
  équipier.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** : sur un échange complet (demande suivie → réponse → clôture) avec
  un équipier Codex, **zéro** commande exécutée par l'équipier et **zéro**
  instruction Bridget dans son contexte.
- **SC-002** : 100 % des messages d'un corpus de test contenant caractères
  spéciaux et multilignes transitent intacts dans les deux sens (aujourd'hui :
  échec dès la première apostrophe non échappée).
- **SC-003** : l'ajout d'un **type d'agent inconnu du code** après Codex et
  Claude se fait avec **zéro ligne de code de production modifiée** (diff
  limité à la configuration et au code de test) — prouvé par un type
  s'appuyant sur l'adaptateur de test stdio, via une fixture de registre
  versionnée. *(Révisé le 2026-08-22 : Gemini, cible initiale, est
  indisponible pour les comptes individuels — constat versionné.)*
- **SC-004** : **zéro** relance émise vers un équipier dont le tour est en
  cours, sur un scénario de tâche longue dépassant l'échéance de relance.
- **SC-005** : un lancement avec clé API dans l'environnement échoue avec
  message explicite dans 100 % des cas.
- **SC-006** : les agents tmux existants passent la suite de tests actuelle sans
  modification (non-régression du transport historique).
- **SC-007** : après la feature, le dépôt ne contient aucun chemin de livraison
  hérité non étiqueté : supprimé, ou marqué déprécié et inscrit au registre des
  dépréciations.

## Assumptions

- Les adaptateurs ACP tiers **Codex et Claude** sont installables et
  fonctionnels sur les machines cibles ; leurs versions sont pinnées dans la
  configuration. Gemini dépend d'un **compte Google éligible** — non
  vérifiable sur la voie individuelle depuis le constat T708 (2026-08-22) ;
  son entrée reste déclarative, statut « conditionnel/non validé ». Un spike
  manuel de validation précède l'implémentation (hors périmètre de cette
  spec).
- La consommation passe par les logins CLI des abonnements existants ; ACP ne
  change pas le canal de facturation (validé en recherche préalable).
- Le périmètre « équipier » couvre les agents headless pilotés par Bridget ;
  l'interaction humaine directe avec un équipier (`bridget attach`), le
  lancement par le daemon lui-même et le serveur MCP sont des features
  ultérieures (sessions 08, 09, 10) et hors périmètre.
- Le modèle de confiance est inchangé : agents coopératifs, même compte
  utilisateur, socket Unix local (cf. README « Positionnement et modèle de
  confiance »).
- Recherche préalable (Article IX) : baseline `04-architectures-patterns.md` ;
  validation live 2026-08-22 — adoption d'ACP par les principaux harness
  (grok-build, DeepSeek Harness dsh, Zed, Gemini CLI, adaptateurs Zed pour
  Codex et Claude), adaptateur Codex `codex-acp` maintenu par Zed et utilisable
  hors Zed. Sources consignées dans `research.md` (phase Plan).
