# Feature Specification: Environnement Docker partagé par projet

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 066-environnement-partage-projet
Titre: Environnement Docker partagé par projet
Statut: Draft
Priorité: P1
Tâches: 18/36 (50%)
Tests: 11/17 (65%)

Résumé:
- Contexte: les agents gérés s'exécutent actuellement sur l'hôte avec le cwd et un environnement filtré, mais sans frontière filesystem/process par projet.
- Objectif: permettre à un projet enregistré d'utiliser un environnement Docker unique et partagé par ses agents, tout en gardant Maicie et Bridget sur l'hôte.
- Exécution: ajouter un backend docker opt-in, un cycle de vie déterministe et un chemin de repli explicite vers le backend host.
- Risque principal: complexifier le lancement, masquer des pannes Docker ou casser le pilotage et l'arrêt des agents.
- Mitigation: activation projet par projet, image épinglée, contrôles fail-closed, aucun secret dans cette tranche et rollback host conservé.
- Validation: plusieurs agents du même projet partagent un conteneur, deux projets restent séparés et la destruction du conteneur ne perd ni code ni état durable.
- Dépendances: SPEC-063, SPEC-064, SPEC-065, SPEC-068, SPEC-075

Fichiers:
- spec.md: ✓ (specs/066-environnement-partage-projet/spec.md)
- tasks.md: ✓ (specs/066-environnement-partage-projet/tasks.md)
- plan.md: ✓ (specs/066-environnement-partage-projet/plan.md)
- implementation.md: ✓ (specs/066-environnement-partage-projet/implementation.md)
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-066-environnement-partage-projet`
**Created**: 2026-08-29
**Status**: Draft
**Priority**: P1
**Dependencies**: SPEC-063, SPEC-064, SPEC-065, SPEC-068, SPEC-075

## Contexte et problème

Bridget lance aujourd'hui chaque équipier géré directement sur l'hôte. Le
`cwd` est absolu, le processus est suivi par génération et l'environnement est
reconstruit à partir d'une baseline et d'une allowlist. Ces garanties sont
utiles, mais le processus voit toujours la machine selon les droits du compte
Unix.

L'objectif n'est pas d'introduire une plateforme d'orchestration. Le besoin
immédiat est une frontière simple par projet: les agents d'un projet partagent
les mêmes fichiers et outils, tandis que les projets distincts ne reçoivent pas
automatiquement les mêmes montages. Bridget et Maicie restent hors des
conteneurs afin de conserver une seule autorité de communication, une seule
flotte et une seule vérité de mission.

Cette tranche reste une préversion technique sans secrets projet. Elle prouve
le cycle de vie, le pilotage, les montages et le rollback. Les credentials et
extensions sont traités séparément par SPEC-067.

## Principes directeurs

- **Un conteneur par projet**: tous les agents d'un même projet partagent son
  environnement; aucun conteneur par agent par défaut.
- **Automates sur l'hôte**: Bridget et Maicie ne sont ni dupliqués ni enfermés
  par projet.
- **Conteneur jetable**: code, worktrees, registre et état durable restent sur
  l'hôte.
- **Activation explicite**: `backend=docker` est opt-in par projet; aucun
  projet existant n'est basculé automatiquement.
- **Repli explicite**: une panne Docker ne déclenche jamais un lancement host
  silencieux.
- **Sécurité honnête**: la tranche isole filesystem et processus entre projets,
  mais n'annonce ni isolation hostile intra-projet ni filtrage fin de l'egress.

## User Scenarios & Testing

### User Story 1 - Préparer un environnement Docker de projet (Priority: P1)

En tant qu'opérateur, je veux préparer et diagnostiquer un environnement Docker
pour un projet enregistré avant d'y lancer un agent.

**Why this priority**: la création doit être prouvée séparément du lancement
pour éviter qu'une panne d'image ou de montage soit interprétée comme une panne
du fournisseur.

**Independent Test**: activer le backend docker sur un projet fixture sans
agent actif, créer l'environnement depuis une image épinglée, vérifier les
montages et contraintes, le détruire puis le recréer sans perte.

**Acceptance Scenarios**:

1. **Given** un projet 065 actif et Docker disponible, **When** l'opérateur
   prépare l'environnement, **Then** un conteneur identifié par `project_id` et
   `binding_generation` atteint `ready`.
2. **Given** une image non épinglée ou absente, **When** la préparation est
   demandée, **Then** elle est refusée sans modifier le backend actif.
3. **Given** un conteneur supprimé manuellement, **When** le statut est consulté,
   **Then** la dérive est visible et la prochaine action propose une recréation.
4. **Given** un conteneur détruit, **When** il est recréé, **Then** le dépôt,
   les worktrees et l'état durable hôte sont intacts.

### User Story 2 - Faire travailler plusieurs agents dans le même projet (Priority: P1)

En tant que coordinateur, je veux lancer plusieurs agents dans l'environnement
du projet afin qu'ils partagent les fichiers et puissent communiquer via
Bridget sans obtenir l'accès global à l'hôte.

**Why this priority**: le partage par projet est la décision structurante qui
évite l'explosion de conteneurs et la complexité de worktrees artificiels.

**Independent Test**: lancer deux agents fixtures dans deux worktrees du même
projet, vérifier qu'ils utilisent le même conteneur et se parlent, puis lancer
un troisième agent dans un autre projet et vérifier l'absence de montage croisé.

**Acceptance Scenarios**:

1. **Given** un environnement `ready`, **When** deux agents du projet démarrent,
   **Then** ils s'exécutent dans le même conteneur avec deux générations et deux
   cycles de vie distincts.
2. **Given** deux worktrees valides du projet, **When** les agents sont lancés,
   **Then** chacun conserve son `cwd` sans créer un conteneur supplémentaire.
3. **Given** deux projets Docker, **When** un agent du projet A inspecte ses
   montages, **Then** aucune racine, worktree ou état privé du projet B n'est
   visible.
4. **Given** un message humain ou inter-agent, **When** il traverse Bridget,
   **Then** pilotage, interruption, accusés et corrélation SPEC-063/064 restent
   observables.

### User Story 3 - Exploiter, arrêter et revenir au host (Priority: P1)

En tant qu'opérateur, je veux arrêter proprement un environnement et revenir au
backend host afin que l'adoption Docker reste réversible.

**Why this priority**: sans rollback réel, l'option Docker devient une migration
irréversible et accroît le risque opérationnel.

**Independent Test**: arrêter tous les agents, arrêter puis supprimer le
conteneur, basculer le projet sur host, lancer un agent historique et vérifier
la conservation des identités et fichiers.

**Acceptance Scenarios**:

1. **Given** des agents actifs, **When** un arrêt non forcé est demandé, **Then**
   il est refusé ou attend leur terminalité sans tuer le conteneur.
2. **Given** aucun agent actif, **When** l'environnement est arrêté ou supprimé,
   **Then** aucun fichier durable n'est supprimé.
3. **Given** un backend docker indisponible, **When** un agent est demandé,
   **Then** le lancement échoue avec une raison Docker et ne bascule pas sur host.
4. **Given** un projet sans agent actif, **When** l'opérateur choisit host,
   **Then** le prochain agent suit le lancement historique et l'environnement
   Docker reste arrêté.

## Edge Cases

- Docker daemon indisponible, lent ou redémarré pendant un `docker exec`.
- Conteneur du bon nom mais labels, image ou génération divergents.
- Image supprimée localement entre préflight et création.
- Racine projet déplacée après préparation.
- Agent actif lorsque la liaison 065 est désactivée ou rebindée.
- Processus `docker exec` perdu alors que le processus agent vit encore.
- Conteneur OOM, limite PID atteinte ou disque du state root saturé.
- Daemon Bridget redémarré alors que le conteneur et les agents vivent.
- Socket Bridget recréée après redémarrage.
- Worktree hors racine principale mais rattaché au même dépôt Git.

## Requirements

### Functional Requirements

- **FR-001**: chaque projet Docker DOIT posséder au plus un environnement
  actif, partagé par tous ses agents.
- **FR-002**: Bridget et Maicie DOIVENT rester des processus hôte uniques et ne
  doivent pas être dupliqués dans l'environnement projet.
- **FR-003**: le backend DOIT être un choix explicite `host` ou `docker` porté
  par ProjectBinding.
- **FR-004**: `host` DOIT rester le backend par défaut des projets existants et
  non enregistrés.
- **FR-005**: un passage host vers docker ou docker vers host DOIT être refusé
  tant qu'un agent du projet est actif.
- **FR-006**: une indisponibilité Docker NE DOIT PAS provoquer un repli host
  silencieux.
- **FR-007**: l'image DOIT être déclarée par digest immuable avant création.
- **FR-008**: le conteneur DOIT porter des labels vérifiables pour project_id,
  binding_generation, image digest et version de contrat.
- **FR-009**: un conteneur existant dont un label ou le digest diverge DOIT être
  classé `recreate_required`, jamais réutilisé silencieusement.
- **FR-010**: la racine projet et ses worktrees DOIVENT rester sur l'hôte et
  être montés dans le conteneur sans copie automatique.
- **FR-011**: le chemin visible dans le conteneur DOIT rester identique au
  chemin canonique hôte afin de préserver les références et diagnostics.
- **FR-012**: le conteneur NE DOIT PAS recevoir un montage de `$HOME`, `/`, du
  socket Docker ou d'une autre racine projet.
- **FR-013**: un répertoire d'état propre au projet DOIT être le seul espace
  durable supplémentaire monté en écriture.
- **FR-014**: la racine système du conteneur DOIT être en lecture seule, avec
  espaces temporaires explicitement déclarés.
- **FR-015**: le processus conteneur DOIT s'exécuter sans privilège, sans
  capabilities Linux ajoutées et avec `no-new-privileges`.
- **FR-016**: aucune clé API, credential fournisseur ou secret projet NE DOIT
  être injecté par cette spec.
- **FR-017**: le conteneur NE DOIT exposer aucun port entrant ni utiliser le
  réseau hôte.
- **FR-018**: la connectivité sortante via réseau bridge DOIT être annoncée
  comme non filtrée finement dans cette version.
- **FR-019**: chaque agent DOIT conserver ses identifiants, génération, parent,
  exécution et cwd propres malgré le conteneur partagé.
- **FR-020**: chaque environnement DOIT joindre un socket Unix distinct,
  privé par `project_id` et `binding_generation`, monté uniquement dans son
  conteneur et sans exposer les fichiers privés du cache Bridget.
- **FR-021**: le redémarrage de Bridget DOIT permettre aux wrappers de se
  reconnecter sans recréer le conteneur ni changer leur identité.
- **FR-022**: lancement, interruption, steering, stop et récupération DOIVENT
  conserver les issues structurées de SPEC-063/064 ainsi que la persistance,
  le rejeu ordonné et l'accusé des incidents runtime délégués de SPEC-068, avec
  la même ProjectReference que l'exécution enfant.
- **FR-023**: la suppression du conteneur NE DOIT supprimer ni le dépôt, ni les
  worktrees, ni l'état durable Bridget/Maicie, ni le state root projet.
- **FR-024**: l'arrêt non forcé DOIT refuser de supprimer un conteneur avec des
  agents actifs.
- **FR-025**: un arrêt forcé éventuel DOIT être une commande locale distincte,
  afficher les agents concernés et rester hors de l'API distante.
- **FR-026**: les états DOIVENT distinguer `absent`, `creating`, `ready`,
  `running`, `stopping`, `stopped`, `degraded` et `recreate_required`.
- **FR-027**: chaque transition DOIT avoir une borne, une raison structurée et
  une preuve observable.
- **FR-028**: le système DOIT réconcilier au redémarrage le registre Bridget,
  les labels Docker et les agents enregistrés avant toute action destructive.
- **FR-029**: deux projets distincts NE DOIVENT partager aucun montage en
  écriture, state root ou conteneur.
- **FR-030**: plusieurs worktrees d'un même projet PEUVENT être utilisés dans
  le même conteneur sans changement de frontière.
- **FR-031**: le backend docker DOIT être testable avec un agent fixture sans
  fournisseur ni secret avant activation générale.
- **FR-032**: le statut DOIT exposer image, état, âge, nombre d'agents, limites
  et dernière raison sans requérir de commande Docker manuelle.
- **FR-033**: le premier handshake sur le runtime ingress DOIT attester
  `project_id`, `binding_generation`, `container_id`, `environment_epoch` et
  génération Bridget avant toute inscription, remise ou commande de pilotage.
- **FR-034**: admission de spawn et transitions stop/remove/switch DOIVENT être
  sérialisées dans Bridget; chaque réservation porte un `environment_epoch`
  revérifié immédiatement avant `docker exec`.
- **FR-035**: si un rebind change `binding_generation` pendant une exécution,
  l'environnement DOIT devenir `recreate_required`, refuser toute nouvelle
  admission et laisser l'exécution existante terminer sans arrêt implicite.
- **FR-036**: toute politique Docker DOIT être résolue par Bridget depuis un
  document hôte v1 fermé, fourni par chemin absolu explicite au démarrage et
  indexé par `policy_id` et `policy_version`; Maicie et une requête projet ne
  peuvent fournir ni chemin, ni option Docker libre, ni valeur de politique.
- **FR-037**: la politique DOIT fixer un UID et un GID numériques non nuls;
  le préflight DOIT prouver que cet utilisateur peut traverser la racine
  projet, écrire dans le state root privé, joindre l'ingress et lire une
  fixture privée `0600` possédée par ce même UID avant toute activation.
- **FR-038**: le conteneur DOIT recevoir `HOME=/var/lib/bridget-project/home`
  et des chemins XDG sous `/var/lib/bridget-project`, tous issus de l'unique
  state root projet; le wrapper DOIT recevoir le socket explicite
  `/run/bridget/runtime/bridget.sock` par sa configuration de lancement et NE
  DOIT déduire aucun socket ou état Bridget depuis `HOME`.
- **FR-039**: une image immuable DOIT être soit un digest de registre
  `repository@sha256:...`, soit un identifiant local Docker `sha256:...`
  résolu et attesté avant create; un tag, un nom seul ou un RepoDigest absent
  NE DOIT servir d'autorité. La fixture locale utilise l'identifiant produit
  par `docker build --iidfile`.
- **FR-040**: tout changement de backend, `policy_version`, `policy_digest`,
  image ou UID/GID DOIT incrémenter `environment_epoch`, rendre
  l'environnement `recreate_required` ou `stopped` selon le backend et publier
  une raison `runtime_policy_changed` consommable par SPEC-067 pour rendre un
  profil `stale` avant toute nouvelle admission.
- **FR-041**: une politique hôte absente, invalide, vide ou modifiable par un
  autre compte DOIT fermer uniquement prepare et les admissions Docker; elle
  NE DOIT modifier ni le backend host historique ni déclencher un fallback.
- **FR-042**: une admission Docker DOIT résoudre, depuis la politique hôte
  attestée, un lanceur Bridget interne et une commande fournisseur interne exacte
  pour le type agent. Les chemins du registre hôte, arguments libres, chemins
  relatifs, doublons ou une entrée absente DOIVENT refuser le lancement sans
  aucun repli host.

### Non-Functional Requirements

- **NFR-001 - Réversibilité**: retour au backend host en une opération après
  arrêt des agents, sans migration de données.
- **NFR-002 - Sécurité**: pas de privileged, host network, Docker socket, montage
  home global ou capabilities Linux.
- **NFR-003 - Durabilité**: tout état utile survit à la destruction du conteneur.
- **NFR-004 - Bornage**: création, inspection, exec, stop et suppression ont des
  timeouts absolus et des issues terminales.
- **NFR-005 - Ressources**: CPU, mémoire et PID sont bornés par environnement.
- **NFR-006 - Performance**: un second agent du projet réutilise l'environnement
  sans seconde création; le temps de lancement ajouté est mesuré.
- **NFR-007 - Observabilité**: logs, métriques et événements séparent panne
  Docker, conteneur, wrapper et fournisseur.
- **NFR-008 - Minimalisme**: Docker CLI est réutilisé; aucune API Docker Rust,
  Compose, Kubernetes ou dépendance externe n'est ajoutée par défaut.

### Key Entities

- **ProjectEnvironment**: état durable de l'environnement Docker du projet.
- **ContainerAttestation**: labels, image, id et contraintes observés.
- **ProjectRuntimePolicy**: image épinglée, limites et montages autorisés.
- **ContainerAgentExecution**: liaison entre génération Bridget et exec Docker.
- **RuntimeIngress**: socket dédiée par laquelle les wrappers conteneurisés
  rejoignent Bridget.

## Hors périmètre

- Un conteneur par agent par défaut.
- Kubernetes, Compose, Podman, gVisor, Firecracker ou orchestration distante.
- Secret, credential fournisseur, plugin, skill ou mémoire globale.
- Filtrage egress par destination ou proxy réseau.
- Isolation hostile entre agents du même projet.
- Migration automatique des agents host déjà actifs.
- Build dynamique d'une image depuis du code agentique non approuvé.

## Success Criteria

### Measurable Outcomes

- **SC-001**: deux agents du même projet utilisent le même container id et
  conservent deux générations Bridget distinctes.
- **SC-002**: deux projets distincts n'ont aucun montage en écriture, state root
  ou container id commun dans 100 % des scénarios.
- **SC-003**: aucune inspection de montage ne révèle `/`, `$HOME`, le socket
  Docker ou la racine d'un autre projet.
- **SC-004**: 100 créations/suppressions/recréations d'une fixture ne modifient
  aucun octet du dépôt et ne perdent aucun état durable attendu.
- **SC-005**: steering, interruption, arrêt et reconnexion passent les mêmes
  oracles que le backend host pour les trois transports stabilisés.
- **SC-006**: un opérateur identifie en moins de 30 secondes si une panne vient
  de Docker, du conteneur, du wrapper ou du fournisseur.
- **SC-007**: toute divergence de digest ou label est détectée avant le premier
  lancement et exige une recréation explicite.
- **SC-008**: aucune panne Docker testée ne déclenche un agent host.
- **SC-009**: le retour explicite à host permet un lancement historique sans
  changement d'identité projet ni de fichiers.
- **SC-010**: les limites CPU, mémoire et PID sont observées et un dépassement
  produit une issue structurée.
- **SC-011**: aucun secret ou credential host n'apparaît dans l'environnement,
  les montages, les logs ou l'inspection du conteneur fixture.
- **SC-012**: aucune nouvelle dépendance Rust ni service externe n'est ajouté.
- **SC-013**: sur la fixture cible, l'utilisateur numérique du conteneur écrit
  dans son HOME/XDG projet, joint uniquement son ingress et lit la fixture
  `0600` attendue, tandis qu'aucun chemin n'est dérivé du HOME hôte.

## Assumptions et dépendances

- SPEC-065 est implémentée, active et réversible.
- Le serveur Linux possède Docker Engine et le compte Bridget peut exécuter les
  seules commandes nécessaires.
- Le modèle de menace v1 sépare les projets mais considère les agents d'un même
  projet coopératifs.
- La connectivité sortante est nécessaire aux fournisseurs, mais n'est pas
  encore filtrée par destination.
- SPEC-067 fournira les profils, extensions et secrets après preuve de ce socle.
- SPEC-075 doit être intégrée avant toute implémentation de 066 : elle fait
  évoluer les mêmes entrées de cycle de vie, d'arrêt, de relance et de
  décommissionnement.
- Le moteur cible peut ne pas être rootless; le modèle de menace n'annonce pas
  Docker comme frontière hostile contre l'administrateur hôte.
