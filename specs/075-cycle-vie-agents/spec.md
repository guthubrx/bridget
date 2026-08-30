# Feature Specification: cycle de vie complet des agents gérés

**Feature Branch**: `session-075-cycle-vie-agents`
**Created**: 2026-08-30
**Status**: Implémentée dans le worktree, non livrée
**Input**: « Pouvoir arrêter, relancer et décommissionner un agent depuis
l'interface, avec les fonctions associées, sans confondre ces opérations. »

## Contexte et correction de sémantique

SPEC-073 a nommé « Décommissionner » une opération qui ne faisait qu'appeler
`StopOrder`. Ce contrat est insuffisant: un arrêt doit être réversible et
l'agent doit rester visible, alors qu'un décommissionnement doit retirer
l'agent de la flotte visible. La présente spécification remplace explicitement
FR-7308, FR-7309, FR-7312, FR-7317 et l'hypothèse de décommissionnement de
SPEC-073. Les garanties d'ouverture volontaire de la fiche, d'identité runtime,
d'accessibilité, d'authentification locale et d'absence de succès optimiste de
SPEC-073 restent applicables.

La conservation de l'historique est indépendante du cycle de vie du processus.
Aucune des trois opérations ne supprime les messages, journaux, sessions,
objectifs ou artefacts.

## User Scenarios & Testing

### User Story 1 - Arrêter sans faire disparaître (Priority: P1)

Comme utilisateur, je veux arrêter un agent géré afin d'interrompre son
processus tout en conservant sa fiche, son identité et son historique pour une
relance ultérieure.

**Independent Test**: arrêter un agent jetable connecté, vérifier la disparition
de son groupe de processus, sa présence durable sous l'état `stopped`, puis
redémarrer Bridget et vérifier qu'il reste visible sans être relancé.

**Acceptance Scenarios**:

1. **Given** un agent géré actif, **When** l'utilisateur confirme « Arrêter »,
   **Then** Bridget arrête son groupe de processus par le superviseur existant
   et attend le verdict réel.
2. **Given** un arrêt confirmé, **When** la liste se rafraîchit, **Then** l'agent
   reste visible avec un état arrêté explicite et n'est plus joignable.
3. **Given** un agent arrêté, **When** le daemon redémarre, **Then** l'agent reste
   arrêté et visible, même s'il avait été créé avec la persistance automatique.
4. **Given** un agent occupé, **When** l'arrêt est demandé, **Then** la
   confirmation avertit que le travail en cours sera interrompu.
5. **Given** un arrêt qui dépasse le délai, **When** le verdict revient, **Then**
   l'interface n'annonce pas un succès et conserve un état réessayable.

### User Story 2 - Relancer le même agent (Priority: P1)

Comme utilisateur, je veux relancer un agent arrêté afin de reprendre le même
agent logique, avec le même nom, le même historique et la dernière définition
runtime attestée.

**Independent Test**: relancer l'agent arrêté de US1, vérifier une nouvelle
génération de processus sous le même nom, puis lui envoyer un message et lire
l'historique antérieur.

**Acceptance Scenarios**:

1. **Given** un agent géré arrêté dont la définition durable est complète,
   **When** l'utilisateur confirme « Relancer », **Then** une nouvelle génération
   est lancée sous le même nom avec le même type, projet, répertoire de travail,
   politique de persistance et définition runtime figée.
2. **Given** une relance en cours, **When** l'agent négocie sa connexion,
   **Then** l'interface affiche un état transitoire honnête et n'autorise pas une
   seconde relance.
3. **Given** une relance réussie, **When** la liste se rafraîchit, **Then** le
   même agent logique redevient connecté sans créer une seconde entrée.
4. **Given** une relance refusée ou échouée, **When** le verdict arrive,
   **Then** l'agent reste arrêté et relançable, avec un motif concis.
5. **Given** un nom redevenu actif entre le clic et le traitement, **When** la
   relance est traitée, **Then** elle est refusée sans lancer un doublon.

### User Story 3 - Décommissionner réellement (Priority: P1)

Comme utilisateur, je veux décommissionner un agent afin de le retirer de la
flotte et de l'interface sans effacer son historique.

**Independent Test**: décommissionner un agent arrêté, puis un agent actif,
vérifier qu'ils disparaissent de la liste après verdict, ne reviennent pas au
redémarrage du daemon et que leurs journaux restent lisibles.

**Acceptance Scenarios**:

1. **Given** un agent géré arrêté, **When** l'utilisateur confirme
   « Décommissionner », **Then** Bridget le fait passer dans un état durable
   caché, retire sa présence de flotte, puis l'agent disparaît de la barre
   latérale.
2. **Given** un agent géré actif, **When** l'utilisateur confirme
   « Décommissionner », **Then** Bridget l'arrête d'abord proprement puis le
   retire de la flotte visible seulement après confirmation de l'arrêt.
3. **Given** l'arrêt préalable au décommissionnement échoue, **When** le délai
   expire, **Then** la définition durable n'est pas supprimée et l'interface
   n'annonce pas le décommissionnement.
4. **Given** un agent décommissionné, **When** le daemon redémarre, **Then** il
   ne réapparaît pas et n'est pas relancé.
5. **Given** un agent décommissionné, **When** son historique est recherché,
   **Then** les données antérieures restent disponibles.

### User Story 4 - Comprendre les actions disponibles (Priority: P1)

Comme utilisateur, je veux que la fiche n'affiche que les actions cohérentes
avec l'état attesté afin de ne pas confondre arrêt, relance et retrait.

**Independent Test**: ouvrir la fiche pour les états actif, occupé, en reprise,
arrêté, externe et disparu, puis vérifier la matrice des boutons, textes et
confirmations au clavier et à la souris.

**Acceptance Scenarios**:

1. **Given** un agent géré actif, **When** sa fiche s'ouvre, **Then** « Arrêter »
   et « Décommissionner » sont disponibles, tandis que « Relancer » ne l'est
   pas.
2. **Given** un agent géré arrêté, **When** sa fiche s'ouvre, **Then**
   « Relancer » et « Décommissionner » sont disponibles, tandis que « Arrêter »
   ne l'est pas.
3. **Given** un agent externe ou TMUX non géré, **When** sa fiche s'ouvre,
   **Then** aucune mutation de cycle de vie n'est proposée et le motif est
   explicite.
4. **Given** une action en cours, **When** la fiche est rerendue par le
   rafraîchissement dynamique, **Then** le bouton reste verrouillé et le verdict
   est rattaché au bon agent.
5. **Given** une panne du daemon, **When** une action échoue, **Then** la fiche
   reste ouverte et l'état local n'est pas modifié optimistement.

### User Story 5 - Conserver une flotte cohérente après incident (Priority: P2)

Comme exploitant, je veux que la distinction entre actif, arrêté et retiré soit
durable afin qu'un redémarrage ou une course entre commandes ne ressuscite pas
un agent ou ne le fasse pas disparaître par erreur.

**Independent Test**: injecter des arrêts du daemon aux frontières avant et
après l'écriture de l'état, puis vérifier la reprise déterministe.

**Acceptance Scenarios**:

1. **Given** un fichier de flotte au schéma antérieur, **When** le nouveau
   daemon démarre, **Then** les entrées existantes sont lues comme agents actifs
   persistants sans réécriture destructive.
2. **Given** un agent non persistant actif au moment d'un redémarrage, **When**
   le daemon revient, **Then** son processus n'est pas repris mais son agent
   logique reste visible comme arrêté.
3. **Given** un arrêt durable confirmé avant un crash, **When** le daemon
   revient, **Then** aucune reprise automatique de cet agent n'est tentée.
4. **Given** une relance échouée, **When** le daemon revient, **Then** la dernière
   définition arrêtée demeure disponible et aucune génération partielle n'est
   annoncée active.

## Functional Requirements

- **FR-7501**: Bridget DOIT distinguer trois commandes métier: `stop`,
  `relaunch` et `decommission`. Aucun libellé ne peut masquer une autre
  opération.
- **FR-7502**: `stop` DOIT arrêter le groupe de processus supervisé par le
  chemin existant, conserver l'identité logique et l'historique, et persister
  l'état `stopped` avant tout verdict de succès.
- **FR-7503**: Un agent arrêté DOIT rester dans `who` et dans l'interface après
  un redémarrage du daemon, avec un fait explicite attestant qu'il est géré.
- **FR-7504**: Un agent `stopped` NE DOIT jamais être repris automatiquement au
  démarrage, quelle que soit sa politique de persistance antérieure.
- **FR-7505**: `relaunch` DOIT être admis seulement pour une entrée gérée
  `stopped`, absente du routeur et de toute génération supervisée active.
- **FR-7506**: `relaunch` DOIT créer une nouvelle génération idempotente sous le
  même nom à partir de la définition runtime figée, du type, du `cwd`, du
  projet et de la politique de persistance conservés. Un parent, mandat ou
  objectif clos NE DOIT pas être réactivé automatiquement.
- **FR-7507**: Un échec de relance DOIT conserver l'entrée `stopped` précédente
  et ne jamais la supprimer par une compensation visant la nouvelle
  génération.
- **FR-7508**: `decommission` DOIT faire passer l'entrée dans l'état durable
  `decommissioned` et retirer sa présence de flotte seulement après l'arrêt
  confirmé d'une génération active, ou immédiatement si l'agent est déjà
  arrêté. Une entrée `decommissioned` est cachée de l'annuaire et ne peut pas
  être relancée.
- **FR-7509**: Un échec ou délai d'arrêt pendant `decommission` DOIT laisser une
  entrée gérée réessayable et ne DOIT pas afficher de succès.
- **FR-7510**: Aucune action de cette feature NE DOIT supprimer l'historique,
  les journaux, les messages, les objectifs, les remises ou les artefacts.
- **FR-7511**: La source durable de cycle de vie DOIT être versionnée, écrite
  atomiquement par le daemon seul et compatible en lecture avec les schémas
  `fleet.json` 1 à 3.
- **FR-7512**: Le nouveau schéma DOIT conserver au minimum le nom, le type, le
  `cwd`, la génération, la définition runtime, le projet, le lien parent, la
  politique de reprise automatique et l'état
  `running|stopped|decommissioned`.
- **FR-7513**: Au démarrage, seules les entrées `running` et persistantes
  DOIVENT devenir candidates à la reprise. Une entrée `running` non persistante
  DOIT être réconciliée durablement en `stopped`.
- **FR-7514**: Une compensation d'échec NE DOIT retirer une définition durable
  que si son `command_id` et sa génération correspondent à la génération
  compensée.
- **FR-7515**: Les agents arrêtés DOIVENT être projetés depuis la source durable
  même si aucune présence mémoire ne survit au redémarrage.
- **FR-7516**: L'éligibilité aux actions DOIT provenir de faits gérés durables,
  jamais du nom, de l'avatar, du fournisseur ou du transport.
- **FR-7517**: Les ordres et résultats `relaunch` et `decommission` DOIVENT être
  typés, corrélés par `command_id`, bornés par délai et distincts de
  `StopResult`.
- **FR-7518**: La CLI et le relais HTTP local DOIVENT utiliser les mêmes ordres
  daemon que l'interface, sans signal système direct ni seconde implémentation
  de terminaison.
- **FR-7519**: Les routes HTTP DOIVENT rester protégées par la présence humaine
  locale et le secret de session existants, valider la version, le nom et le
  `command_id`, et ne pas réessayer automatiquement.
- **FR-7520**: L'interface DOIT appliquer une matrice d'actions fermée: actif =
  arrêter et décommissionner; arrêté = relancer et décommissionner; reprise ou
  mutation en cours = aucune commande concurrente; non géré = aucune mutation.
- **FR-7521**: Chaque action destructive DOIT avoir une confirmation nommant
  l'agent et ses conséquences. L'arrêt d'un agent occupé DOIT signaler
  l'interruption du travail.
- **FR-7522**: Le rendu ne DOIT jamais muter optimistement l'état. La liste
  dynamique existante devient l'autorité après un verdict terminal.
- **FR-7523**: Une action répétée ou devenue incohérente entre l'ouverture de la
  fiche et son traitement DOIT retourner une issue fermée telle que
  `already_stopped`, `already_running`, `not_managed`, `not_found`,
  `conflict`, `timeout` ou `failed`.
- **FR-7524**: La mise à niveau DOIT prévoir une adoption explicite et
  vérifiable des agents gérés déjà arrêtés avant le schéma 4, sans classer les
  anciens agents externes comme gérés et sans recréer toute l'histoire dans la
  barre latérale.
- **FR-7525**: Les événements et journaux DOIVENT permettre d'identifier
  l'action, le nom, le `command_id`, l'ancienne et la nouvelle génération et le
  verdict, sans exposer de secret ni de contenu de conversation.
- **FR-7526**: Le nom d'un agent `decommissioned` DOIT rester réservé tant que
  son historique est conservé, afin qu'un nouveau spawn ne soit pas confondu
  avec l'ancienne identité logique. La libération du nom relève d'une future
  purge explicite hors périmètre.

## Non-Functional Requirements

- **NFR-7501**: Aucune dépendance d'exécution, base de données ou service
  supplémentaire ne doit être ajouté.
- **NFR-7502**: Les transitions durables doivent conserver les permissions
  privées et les garanties `temp + fsync + rename + fsync parent` existantes.
- **NFR-7503**: La construction de la liste reste O(n) pour n agents et aucune
  requête réseau par ligne n'est ajoutée.
- **NFR-7504**: Les boutons et dialogues restent utilisables au clavier, au zoom
  200 % et dans une fenêtre de 1280 par 720 pixels.
- **NFR-7505**: Toutes les nouvelles variantes de protocole acceptées doivent
  être couvertes par sérialisation aller-retour et par un cas d'intégration
  daemon.
- **NFR-7506**: Les suites SPEC-009, SPEC-071, SPEC-073 et le rafraîchissement
  dynamique de la barre latérale doivent rester vertes.

## Success Criteria

- **SC-7501**: Sur la matrice actif, occupé, arrêté, en reprise, externe et
  disparu, 100 % des boutons correspondent à FR-7520.
- **SC-7502**: Après 3 cycles arrêt explicite puis redémarrage daemon, l'agent
  reste arrêté et visible 3 fois sur 3, sans processus résiduel.
- **SC-7503**: Après 3 relances, le même nom et le même historique sont
  conservés 3 fois sur 3, avec une génération strictement croissante et un seul
  processus actif.
- **SC-7504**: Après décommissionnement d'un agent actif et d'un agent arrêté,
  ils disparaissent de la flotte et ne reviennent pas après 3 redémarrages,
  tandis que leurs journaux restent lisibles.
- **SC-7505**: Les crash-tests avant et après chaque écriture durable ne
  produisent ni résurrection après stop, ni disparition après échec de relance,
  ni doublon de nom.
- **SC-7506**: Les issues fermées de FR-7523 sont toutes testées et aucun test
  ne constate de succès optimiste.
- **SC-7507**: Tous les tests ciblés Rust, intégration HTTP et JavaScript ainsi
  que `cargo fmt --check` et `git diff --check` réussissent.

## Key Entities

- **Agent géré**: identité logique dont Bridget possède le cycle de vie et la
  définition durable.
- **Génération**: incarnation supervisée unique d'un agent géré.
- **État de cycle de vie**: `running`, `stopped` ou `decommissioned`, indépendant
  de l'état de transport observé.
- **Politique de persistance**: booléen déterminant la reprise automatique d'un
  agent `running` au redémarrage du daemon.
- **Décommissionnement**: retrait de la flotte visible après arrêt confirmé,
  avec tombstone durable cachée et sans suppression d'historique.
- **Adoption héritée**: migration explicite d'un agent déjà arrêté dont une
  génération gérée peut être prouvée dans l'état existant.

## Hors périmètre

- Purge définitive de l'historique ou droit à l'effacement.
- Pause d'un tour sans arrêter le processus, drain différé ou mise en sommeil.
- Renommage, clonage, changement de fournisseur ou de modèle lors d'une
  relance.
- Actions en masse sur plusieurs agents.
- Gestion du cycle de vie des agents TMUX ou lancés hors Bridget.
- Modification des politiques Maicie, des rondes ou des mandats.
- Déploiement automatique, fusion ou commit automatique.

## Hypothèses

- L'historique est adressé par le nom logique et n'est pas supprimé lorsque
  l'agent quitte la flotte visible.
- La définition runtime figée existante est suffisante pour relancer le même
  agent tant que ses préconditions locales restent valides.
- Un nom décommissionné reste réservé tant que son historique existe. Sa
  libération nécessite une opération de purge distincte, hors de cette session.
