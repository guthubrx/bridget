# Feature Specification: Piloter un agent depuis sa fiche

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 073-actions-agent-ui
Titre: Piloter un agent depuis sa fiche
Statut: Implémenté
Priorité: P1
Tâches: 24/24
Tests: 179 réussis, validation visuelle manuelle non requise par décision utilisateur

Résumé:
- Contexte: la fiche d'identité SPEC-071 apparaît au survol mais ne permet aucune action et son déclenchement gêne parfois la lecture de la liste.
- Objectif: ouvrir cette fiche volontairement depuis un bouton à trois points et permettre d'y décommissionner proprement un agent géré.
- Risque principal: arrêter un agent par erreur, confondre arrêt opérationnel et suppression d'historique, ou contourner le cycle d'arrêt existant.
- Mitigation: déclenchement séparé, confirmation explicite, éligibilité attestée, résultat non optimiste et réutilisation exclusive du mécanisme d'arrêt géré.
- Validation: parcours souris et clavier, agents gérés ou externes, agent actif ou déjà arrêté, résultats d'arrêt et erreurs reproductibles.
- Dépendances: SPEC-071, projection de présence existante et contrat actuel `bridget stop`.
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-073-actions-agent-ui`
**Created**: 2026-08-30
**Status**: Implemented
**Priority**: P1
**Dependencies**: SPEC-071, contrat d'arrêt géré existant

## Contexte et problème

La liste latérale affiche l'heure de dernière activité et la fiche d'identité
SPEC-071. Cette fiche s'ouvre actuellement quand le pointeur survole toute la
ligne ou quand la ligne reçoit le focus. Ce déclenchement involontaire masque
une partie de l'interface et ne permet pas de distinguer la sélection d'un agent
de l'ouverture de ses informations.

L'utilisateur veut également pouvoir retirer proprement un agent de la flotte
active. Bridget possède déjà une opération d'arrêt géré. L'interface ne doit ni
la dupliquer, ni tuer directement un processus, ni présenter cet arrêt comme
une suppression de l'historique.

## User Scenarios & Testing

### User Story 1 - Ouvrir volontairement la fiche d'un agent (Priority: P1)

Comme utilisateur qui supervise la flotte, je veux ouvrir la fiche d'identité
avec un bouton à trois points situé près de la dernière activité, afin de
consulter les informations d'un agent sans déclenchement involontaire.

**Independent Test**: afficher plusieurs agents, survoler et sélectionner leur
ligne, puis ouvrir une fiche avec le bouton à trois points à la souris et au
clavier.

**Acceptance Scenarios**:

1. **Given** une ligne d'agent, **When** elle est seulement survolée ou reçoit
   le focus de sélection, **Then** aucune fiche ne s'ouvre automatiquement.
2. **Given** une ligne d'agent, **When** l'utilisateur active le bouton à trois
   points, **Then** une seule fiche interactive s'ouvre pour cet agent.
3. **Given** une fiche ouverte, **When** l'utilisateur appuie sur Échap, clique
   hors de la fiche ou ouvre celle d'un autre agent, **Then** la fiche se ferme
   et le focus revient au déclencheur pertinent.
4. **Given** une fiche ouverte, **When** l'utilisateur consulte ses
   informations, **Then** la sélection et le fil de conversation ne changent
   pas.

### User Story 2 - Conserver une identité lisible et complète (Priority: P1)

Comme utilisateur, je veux retrouver dans la fiche volontaire toutes les
informations fiables de SPEC-071, afin que l'ajout d'actions ne dégrade pas la
compréhension du runtime.

**Independent Test**: ouvrir les fiches d'agents Codex, Claude Code, Cursor,
Gemini CLI et d'un runtime inconnu, en modes TMUX et FLUX, puis comparer les
faits affichés aux métadonnées reçues.

**Acceptance Scenarios**:

1. **Given** un runtime connu, **When** sa fiche est ouverte, **Then** son logo,
   son produit, son éditeur et son mode restent présents et lisibles.
2. **Given** une métadonnée absente, **When** la fiche est ouverte, **Then** la
   valeur reste inconnue et n'est pas reconstruite depuis le nom de l'agent.
3. **Given** une fenêtre étroite ou un zoom élevé, **When** la fiche s'ouvre,
   **Then** elle reste entièrement accessible sans masquer son action de
   fermeture.

### User Story 3 - Décommissionner un agent géré sans erreur (Priority: P1)

Comme utilisateur autorisé, je veux décommissionner un agent géré depuis sa
fiche, afin de l'arrêter proprement et de le retirer de la flotte active sans
utiliser la ligne de commande.

**Independent Test**: utiliser un agent jetable géré, ouvrir sa fiche, annuler
une première confirmation, confirmer la seconde tentative, puis vérifier le
résultat opérationnel et la conservation de l'historique.

**Acceptance Scenarios**:

1. **Given** un agent géré et actif, **When** l'utilisateur choisit
   « Décommissionner », **Then** une confirmation nomme exactement l'agent,
   explique l'arrêt du processus et précise que l'historique sera conservé.
2. **Given** la confirmation ouverte, **When** l'utilisateur annule, **Then**
   aucun ordre d'arrêt n'est envoyé et l'agent reste inchangé.
3. **Given** la confirmation ouverte, **When** l'utilisateur confirme, **Then**
   l'interface transmet un seul ordre d'arrêt géré et attend son verdict réel.
4. **Given** un arrêt réussi, **When** le verdict arrive, **Then** l'agent quitte
   la flotte active, apparaît comme arrêté et son historique reste consultable.
5. **Given** un agent externe, TMUX ou autrement non géré, **When** sa fiche est
   ouverte, **Then** le décommissionnement est indisponible avec une explication
   concise.
6. **Given** un agent déjà arrêté, **When** sa fiche est ouverte, **Then** aucun
   second ordre d'arrêt ne peut être émis.
7. **Given** un agent qui exécute un tour, **When** la confirmation est ouverte,
   **Then** elle avertit explicitement que le travail en cours sera interrompu.

### User Story 4 - Comprendre le résultat sans état trompeur (Priority: P1)

Comme utilisateur, je veux voir l'issue réelle du décommissionnement, afin de
ne jamais croire qu'un agent est arrêté lorsque le daemon ne l'a pas confirmé.

**Independent Test**: simuler les issues arrêt propre, arrêt forcé, agent non
géré, agent absent, délai dépassé et daemon indisponible.

**Acceptance Scenarios**:

1. **Given** un ordre en cours, **When** aucun verdict terminal n'est encore
   reçu, **Then** la fiche indique l'attente et interdit un doublon.
2. **Given** un arrêt propre ou forcé confirmé, **When** le verdict est reçu,
   **Then** le message distingue les deux issues sans masquer un arrêt forcé.
3. **Given** un refus, un délai dépassé ou une panne du relais, **When** le
   résultat arrive, **Then** la fiche reste ouverte, explique l'échec et ne
   modifie pas optimistement l'état de l'agent.
4. **Given** deux activations pendant la même demande en cours, **When**
   l'interface traite la seconde, **Then** elle n'émet pas un deuxième ordre et
   ne déclenche aucun retry automatique.

## Functional Requirements

- **FR-7301**: Chaque ligne d'agent DOIT afficher un bouton à trois points à
  droite de l'heure de dernière activité, sans masquer le nom ni l'état.
- **FR-7302**: La fiche ne DOIT plus s'ouvrir au simple survol ou au focus de la
  ligne ; seul le déclencheur à trois points DOIT l'ouvrir.
- **FR-7303**: Le déclencheur, la fiche et ses actions ne DOIVENT sélectionner
  l'agent, changer le fil, modifier un brouillon ou acquitter un message.
- **FR-7304**: Une seule fiche interactive DOIT être ouverte à la fois et être
  associée explicitement au déclencheur qui l'a ouverte.
- **FR-7305**: La fiche DOIT conserver toutes les informations et logos requis
  par SPEC-071, sans inférence depuis le nom de l'agent.
- **FR-7306**: La fiche DOIT être utilisable à la souris et au clavier, se
  fermer avec Échap et au clic extérieur, puis restituer un focus cohérent.
- **FR-7307**: La possibilité de décommissionner DOIT dépendre d'un fait
  explicite indiquant que l'agent est géré par Bridget ; elle ne DOIT pas être
  déduite du nom, du runtime, du transport ou de l'avatar.
- **FR-7308**: Un agent externe, non géré ou déjà arrêté NE DOIT pas pouvoir
  recevoir un ordre de décommissionnement depuis la fiche.
- **FR-7309**: L'action DOIT employer le libellé « Décommissionner » et
  expliquer qu'elle arrête l'agent et le retire de la flotte active sans
  supprimer son historique.
- **FR-7310**: Une confirmation explicite DOIT afficher le nom exact de l'agent
  et les conséquences avant tout envoi.
- **FR-7311**: Si une activité est attestée en cours, la confirmation DOIT
  signaler que ce travail sera interrompu.
- **FR-7312**: L'interface DOIT réutiliser exclusivement l'opération d'arrêt
  géré existante ; elle NE DOIT ni envoyer directement un signal système, ni
  créer un second mécanisme de terminaison.
- **FR-7313**: Chaque confirmation DOIT produire un identifiant de corrélation
  unique, le conserver pendant toute la demande et ne déclencher aucun retry
  automatique. Cet identifiant ne constitue pas une promesse d'idempotence du
  daemon.
- **FR-7314**: Pendant un arrêt en cours, les nouvelles confirmations pour le
  même agent DOIVENT être bloquées jusqu'à un verdict terminal.
- **FR-7315**: L'interface DOIT distinguer au minimum : arrêté proprement,
  arrêté avec terminaison forcée, non géré, introuvable, délai dépassé et daemon
  indisponible.
- **FR-7316**: Aucun succès ne DOIT être affiché avant le verdict réel du
  daemon ; une erreur ne DOIT pas déplacer l'agent vers les agents arrêtés.
- **FR-7317**: Après un succès, l'historique et la fiche de l'agent DOIVENT
  rester consultables dans la section des agents arrêtés.
- **FR-7318**: L'opération depuis l'interface DOIT rester soumise à la même
  présence humaine locale et au même secret de session que les autres actions
  du relais.
- **FR-7319**: La ligne d'un agent sans heure de dernière activité DOIT malgré
  tout conserver un emplacement stable et accessible pour le bouton à trois
  points.

## Non-Functional Requirements

- **NFR-7301**: L'ouverture ou la fermeture de la fiche DOIT être perceptible
  en moins de 150 millisecondes sans requête vers un service tiers.
- **NFR-7302**: La fiche DOIT rester entièrement utilisable dans une fenêtre de
  1280 par 720 pixels et avec un zoom navigateur de 200 %.
- **NFR-7303**: Le déclencheur et la confirmation DOIVENT avoir un nom
  accessible, un ordre de tabulation logique et une indication indépendante de
  la couleur.
- **NFR-7304**: La feature NE DOIT ajouter aucune dépendance d'exécution, aucun
  service, aucune table et aucun mécanisme de suppression d'historique.
- **NFR-7305**: Les listes d'agents DOIVENT conserver une complexité linéaire
  par rapport au nombre d'agents et ne déclencher aucun appel réseau par ligne.
- **NFR-7306**: Les erreurs du daemon et du relais DOIVENT être affichées sous
  une forme concise sans révéler le secret de session ni des données internes
  inutiles.

## Success Criteria

- **SC-7301**: Sur 100 ouvertures testées, 100 % proviennent du bouton à trois
  points et zéro du simple survol ou focus de la ligne.
- **SC-7302**: Les parcours souris et clavier ouvrent, parcourent et ferment la
  fiche sans changer la sélection courante dans 100 % des scénarios testés.
- **SC-7303**: La matrice SPEC-071 conserve 100 % des produits, éditeurs, logos,
  modes et valeurs inconnues attendus.
- **SC-7304**: Une annulation de confirmation émet zéro ordre d'arrêt et une
  confirmation valide en émet exactement un.
- **SC-7305**: Les six issues d'arrêt définies sont chacune affichées avec un
  état exact et aucune ne produit de succès optimiste.
- **SC-7306**: Un agent arrêté avec succès disparaît de la flotte active mais
  conserve 100 % de son historique consultable.
- **SC-7307**: Zéro agent non géré ou déjà arrêté peut être décommissionné par
  l'interface dans la matrice de validation.
- **SC-7308**: Tous les tests existants de SPEC-071 et du mécanisme d'arrêt
  continuent de réussir.

## Key Entities

- **Déclencheur d'actions**: bouton à trois points lié à un agent précis et
  indépendant du contrôle qui sélectionne la conversation.
- **Fiche interactive**: panneau compact contenant l'identité attestée et les
  actions autorisées pour l'agent.
- **Éligibilité de gestion**: fait explicite indiquant que le cycle de vie de
  l'agent est piloté par Bridget.
- **Demande de décommissionnement**: intention confirmée portant le nom de
  l'agent et un identifiant stable.
- **Verdict d'arrêt**: résultat terminal du daemon, seul fait autorisant une
  présentation de succès.
- **Agent arrêté**: agent retiré de la flotte active dont l'historique reste
  disponible.

## Edge Cases

- L'agent disparaît entre l'ouverture de la fiche et la confirmation.
- La fiche ouverte est rerendue pendant l'arrivée d'un événement de présence.
- L'agent passe à l'état arrêté pendant que la confirmation est visible.
- Deux clics rapides tentent d'envoyer la même demande.
- Le daemon confirme un arrêt forcé après un délai de grâce.
- Le relais redémarre pendant l'attente du verdict.
- Le nom de l'agent contient des caractères affichables particuliers.
- La ligne ne possède aucune heure de dernière activité.
- La première ou dernière ligne est proche d'un bord de la fenêtre.

## Assumptions

- « Décommissionner » signifie arrêter le processus géré, retirer sa présence
  active et conserver son historique. La suppression définitive des données
  n'est pas incluse.
- La présence humaine locale et le secret de session actuels constituent
  l'autorité existante de l'interface ; cette feature ne crée pas de système de
  rôles supplémentaire.
- Un fait de gestion explicite déjà dérivable de la présence peut être projeté
  vers l'interface. Une absence de fait interdit l'action.
- L'utilisateur peut intentionnellement arrêter un agent occupé après avoir lu
  l'avertissement ; l'activité en cours ne constitue pas un refus automatique.

## Hors périmètre

- Supprimer les messages, journaux, remises, objectifs ou artefacts d'un agent.
- Décommissionner en masse plusieurs agents.
- Arrêter directement un processus système ou un groupe de processus depuis le
  navigateur.
- Ajouter une action de redémarrage, de clonage ou de changement de runtime.
- Modifier la politique Maicie, la ronde de vigilance ou le coordinateur.
- Déduire qu'un agent est géré à partir de son nom ou de son transport.
- Ajouter une authentification distante ou exposer l'interface hors de son
  périmètre local actuel.

## État de validation

- Implémentation complète des quatre user stories et du contrat HTTP versionné.
- 87 tests JavaScript, 49 tests Rust UI, 24 tests d'intégration du relais et
  19 tests unitaires filtrés sur le mécanisme d'arrêt réussissent après
  intégration de `origin/main`.
- `git diff --check` est propre et aucune dépendance d'exécution n'a été ajoutée.
- Le parcours navigateur manuel prévu initialement a été retiré de la livraison
  à la demande explicite de l'utilisateur. Sa couverture fonctionnelle est
  assurée par les tests DOM, clavier, protocole HTTP et socket daemon isolée.
