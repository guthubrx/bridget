# Spécification - SPEC-077 Menu contextuel des agents

## Fiche synthèse

Spec: 077-menu-contextuel-agents
Titre: Menu contextuel unifié des agents
Statut: Implemented - validation automatisée complète
Priorité: P1
Branche: session-077-menu-contextuel-agents
Créée: 2026-08-30
Dépendances: SPEC-069, SPEC-071, SPEC-073, SPEC-075
Compatibilité future: SPEC-076

## Contexte

La colonne des agents possède déjà un bouton à trois points qui ouvre une grande fiche d'identité. Cette fiche expose des informations utiles et certaines actions de cycle de vie, mais son comportement ressemble à un panneau de détail plutôt qu'à un menu contextuel. Les utilisateurs habitués aux applications de bureau s'attendent également à pouvoir ouvrir les actions d'un élément avec un clic droit.

Le besoin est de disposer d'un seul menu cohérent, rapide et lisible, accessible par les trois points, par clic droit et par clavier. Les trois voies doivent ouvrir exactement les mêmes actions et ne jamais diverger.

## Objectifs

- Rendre les actions d'un agent immédiatement compréhensibles et accessibles.
- Conserver les trois points comme déclencheur visible et découvrable.
- Ajouter le clic droit comme accélérateur sans en faire l'unique accès.
- Présenter l'identité d'exécution de façon compacte dans le même menu.
- Permettre d'ouvrir, épingler, marquer comme lu, masquer, arrêter, relancer et décommissionner selon l'état réel de l'agent.
- Conserver un comportement stable pendant les actualisations automatiques de la flotte.

## Hors périmètre

- Dupliquer une définition d'agent.
- Modifier le profil technique, les secrets ou le fournisseur d'un agent.
- Affecter un agent à un projet ou à une section.
- Ajouter un nouvel endpoint de cycle de vie.
- Modifier la sémantique durable de stop, relaunch ou decommission.
- Supprimer l'historique d'un agent décommissionné.

## Acteurs

- Opérateur: consulte et pilote la flotte depuis l'interface Bridget.
- Bridget UI: présente un état dérivé des faits du daemon et des préférences locales.
- Daemon Bridget: reste l'autorité sur le cycle de vie des agents gérés.

## User Story 1 - Ouvrir un menu unique par trois voies - P1

Comme opérateur, je veux ouvrir le même menu depuis les trois points, le clic droit ou le clavier afin de choisir la voie la plus rapide sans apprendre plusieurs interfaces.

### Scénarios d'acceptation

1. Étant donné une ligne d'agent, quand je clique sur les trois points, alors le menu s'ouvre près du bouton et le premier élément actionnable reçoit le focus.
2. Étant donné la même ligne, quand je fais un clic droit, alors le menu natif du navigateur ne s'ouvre pas et le même menu apparaît près du pointeur.
3. Étant donné la même ligne au clavier, quand j'utilise la touche de menu contextuel ou Maj+F10, alors le même menu s'ouvre.
4. Quand le menu est ouvert, les flèches verticales, Début, Fin, Échap et Tab suivent les conventions d'un menu d'application.
5. Quand j'ouvre le menu d'un autre agent, le menu précédent se ferme et un seul menu reste présent.

## User Story 2 - Organiser et lire la flotte - P1

Comme opérateur, je veux ouvrir une conversation, épingler un agent, marquer ses messages comme lus et masquer une ligne afin de garder la colonne adaptée à mon travail courant.

### Scénarios d'acceptation

1. Ouvrir la conversation sélectionne l'agent sans rechargement de page.
2. Épingler place l'agent avant les agents non épinglés de son groupe et l'action devient Désépingler.
3. Marquer comme lu met le compteur à zéro sans supprimer de message.
4. Masquer retire l'agent des listes courantes mais le rend récupérable dans une section Agents masqués.
5. Depuis Agents masqués, Afficher dans la barre restaure l'agent dans son groupe actif ou arrêté.
6. Les préférences d'épinglage, de masquage et de lecture survivent à un rechargement de l'interface sur le même poste.

## User Story 3 - Piloter le cycle de vie sans ambiguïté - P1

Comme opérateur, je veux que les commandes arrêter, relancer et décommissionner reflètent l'état réel et la gestion Bridget afin d'éviter une action impossible ou trompeuse.

### Scénarios d'acceptation

1. Pour un agent géré actif, Arrêter est disponible et Relancer est indisponible.
2. Pour un agent géré arrêté, Relancer est disponible et Arrêter est indisponible.
3. Pour un agent non géré, les commandes de cycle de vie sont visibles mais indisponibles avec une raison lisible.
4. Décommissionner est visuellement destructif et conserve la confirmation existante.
5. Une action réussie actualise la flotte sans saut de page et conserve le retour utilisateur.
6. Une action refusée ou en délai affiche le verdict exact sans mutation optimiste mensongère.

## User Story 4 - Identifier l'exécution sans ouvrir un panneau séparé - P2

Comme opérateur, je veux voir dans l'en-tête du menu le nom, la présence, le fournisseur et le mode d'exécution afin de comprendre immédiatement l'agent concerné.

### Scénarios d'acceptation

1. Le menu montre le nom et la présence réelle de l'agent.
2. Le logo fournisseur provient du catalogue runtime existant et ne dépend jamais du nom de l'agent.
3. Le mode FLUX, TMUX ou inconnu reste visible sous forme compacte.
4. L'activité, le transport, le modèle et l'effort ne sont affichés que lorsqu'ils apportent une information attestée.

## Exigences fonctionnelles

- FR-7701: L'interface DOIT posséder un seul composant de menu contextuel d'agent, réutilisé par toutes les voies d'ouverture.
- FR-7702: Le bouton à trois points DOIT rester visible sur chaque ligne et exposer la sémantique d'un bouton de menu.
- FR-7703: Un clic droit sur une ligne DOIT empêcher le menu natif et ouvrir le menu Bridget pour l'agent ciblé.
- FR-7704: La touche Menu contextuel et Maj+F10 DOIVENT ouvrir le menu depuis une ligne focalisée.
- FR-7705: Le menu DOIT gérer le focus, les flèches verticales, Début, Fin, Tab et Échap de façon prévisible.
- FR-7706: L'en-tête DOIT réutiliser l'identité runtime, le logo fournisseur, la présence et le mode existants.
- FR-7707: L'action Ouvrir la conversation DOIT sélectionner l'agent sans rechargement de document.
- FR-7708: L'action Épingler ou Désépingler DOIT modifier un ordre local durable sans modifier l'ordre fourni par le daemon.
- FR-7709: L'action Marquer comme lu DOIT mémoriser localement la dernière activité lue et ne jamais supprimer un message.
- FR-7710: L'action Masquer de la barre DOIT déplacer l'agent dans une section récupérable Agents masqués.
- FR-7711: L'action Afficher dans la barre DOIT restaurer un agent masqué sans modifier son état de cycle de vie.
- FR-7712: Les préférences locales DOIVENT être versionnées, validées à la lecture et tolérer un stockage indisponible ou corrompu.
- FR-7713: Les actions Arrêter, Relancer et Décommissionner DOIVENT réutiliser les règles, confirmations, endpoints et verdicts SPEC-075 existants.
- FR-7714: Les actions de cycle de vie impossibles DOIVENT rester grisées et expliquer leur indisponibilité.
- FR-7715: Décommissionner DOIT rester distingué visuellement des actions réversibles.
- FR-7716: Une actualisation de flotte DOIT conserver l'épinglage, le masquage, la sélection, le défilement et un menu ouvert lorsque l'agent existe encore.
- FR-7717: Un agent disparu DOIT fermer son menu sans erreur et ses préférences locales ne DOIVENT pas provoquer de ligne fantôme.
- FR-7718: Le menu DOIT se fermer sur clic extérieur, Échap, redimensionnement ou défilement de la fenêtre.
- FR-7719: L'ouverture du menu DOIT rester locale et ne déclencher aucune requête réseau.
- FR-7720: Aucun endpoint, dépendance ou service supplémentaire ne DOIT être créé pour cette fonctionnalité.

## Cas limites

- Le stockage local est refusé par le navigateur.
- Le stockage local contient un JSON invalide ou une version inconnue.
- L'agent change d'état pendant que son menu est ouvert.
- L'agent disparaît pendant une actualisation.
- L'agent est masqué alors que sa conversation est ouverte.
- Un agent masqué devient arrêté ou redevient actif.
- Tous les agents sont masqués.
- Le menu est ouvert près des bords droit ou inférieur de la fenêtre.
- Une confirmation de cycle de vie est ouverte depuis le menu.
- Un agent ne possède ni logo, ni modèle, ni effort attesté.

## Données et confidentialité

Les préférences locales contiennent uniquement des noms d'agents et des horodatages de lecture. Elles ne contiennent ni message, ni jeton, ni secret, ni commande. Le daemon reste la source de vérité pour la flotte et le cycle de vie.

## Hypothèses

- Le navigateur fournit localStorage, avec une dégradation silencieuse si son accès échoue.
- Le clic principal sur une ligne continue d'ouvrir directement la conversation.
- Les agents masqués restent consultables dans une section repliée dédiée.
- Les trois points restent l'accès visible pour la découvrabilité et les usages tactiles.
- La SPEC-076 pourra plus tard ajouter une action de projet sans modifier le mécanisme d'ouverture du menu.

## Critères de succès

- SC-7701: Les trois voies d'ouverture affichent exactement les mêmes actions pour un même état d'agent.
- SC-7702: Cent pour cent des actions du menu sont utilisables au clavier sans piège de focus.
- SC-7703: L'ouverture et la fermeture du menu prennent moins de 150 ms et ne provoquent aucune requête réseau.
- SC-7704: Les préférences valides sont restaurées après rechargement et une préférence corrompue ne bloque jamais l'interface.
- SC-7705: Un agent masqué peut être restauré en trois interactions maximum.
- SC-7706: Les tests automatisés couvrent les états actif, arrêté, non géré, masqué, épinglé, lu et stockage corrompu.
- SC-7707: Les suites UI existantes restent entièrement passantes.
- SC-7708: Aucun rechargement complet ni saut de défilement n'est introduit.

## Tests

Tests: 146/146 ciblés (100%)

## Résultat d’implémentation

- Menu global unique ouvert par trois points, clic droit, touche Menu ou Maj+F10.
- Sept actions cohérentes: ouvrir, épingler, lire, masquer, arrêter, relancer et décommissionner.
- Préférences locales versionnées et section Agents masqués récupérable.
- Aucun endpoint, service ou paquet ajouté.
- Validation: 93/93 tests Node, 53/53 tests Rust UI et git diff --check.
- Audit final: A, aucun finding bloquant. Deux dettes MEDIUM sont consignées dans audit.md.
- Revue cross-provider: indisponible faute de capacité Claude active.
