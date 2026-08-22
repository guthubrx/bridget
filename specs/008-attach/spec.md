# Feature Specification : Voir et piloter un équipier — `bridget attach`

**Feature Branch**: `session-08-attach` (à créer depuis `session-07-transport-acp` — pas de merge dans `main` sans validation utilisateur)
**Created**: 2026-08-22
**Status**: Draft — en attente de contre-revue adverse
**Input**: User description: « un équipier headless doit rester observable et pilotable : je veux voir ce que racontent les agents et leur parler individuellement, comme dans un pane tmux, sans réintroduire le terminal simulé. »

## Contexte et problème

La session 007 rend les équipiers headless : leurs échanges passent en ACP et
sont journalisés (JSONL par agent, `data-model.md` 007), mais plus rien ne les
*montre*. Aujourd'hui, la seule fenêtre sur un agent est son pane tmux — qui
cumule trois rôles (héberger le processus, journaliser, afficher). La 007 a
séparé les deux premiers ; cette session fournit le troisième sous forme de
**client** : une vue détachable sur le flux d'un équipier, depuis laquelle
l'humain peut aussi lui écrire. Le flux de données est la source de vérité, la
fenêtre n'en est qu'une projection — le modèle validé chez les harness
commerciaux (équipiers persistants + app cliente), transposé en local.

## Dépendances

- **007-transport-acp** : le journal de session (événements de tour) et l'état
  de l'équipier dans l'annuaire sont les sources de cette vue. Session 008
  démarre après le checkpoint MVP de la 007 (T706 : journal livré).
- **002-federation-ssh** : l'attach distant réutilise la fédération existante.
- **005-domaines-dnd** : écrire à un équipier via attach reste un message
  ordinaire, soumis aux mêmes règles (DND, garde-fous).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Regarder un équipier travailler (Priority: P1)

L'utilisateur tape `bridget attach <nom>` dans un terminal. La vue s'ouvre sur
l'historique récent de l'équipier (relecture du journal du jour) puis affiche
en continu ce qui se passe : débuts et fins de tours avec leur expéditeur, le
texte produit, les outils appelés, les erreurs — lisiblement, horodaté, sans
JSON brut.

**Why this priority** : c'est la contrepartie promise du passage en headless ;
sans elle, les équipiers sont des boîtes noires et la confiance disparaît.

**Independent Test** : lancer un équipier Codex (007), lui envoyer une demande
depuis un autre terminal, ouvrir `bridget attach` pendant le tour : le tour en
cours et sa réponse s'affichent ; fermer et rouvrir la vue : l'historique
réapparaît sans doublon.

**Acceptance Scenarios**:

1. **Given** un équipier en plein tour, **When** l'utilisateur s'attache,
   **Then** l'historique du jour s'affiche puis les événements du tour en cours
   arrivent en continu, chacun horodaté et attribué.
2. **Given** une vue attachée, **When** l'utilisateur se détache (Ctrl-C ou
   commande), **Then** l'équipier n'est ni interrompu ni ralenti, le terminal
   est restauré, et un nouvel `attach` rejoue volontairement sa fenêtre
   d'historique puis reprend le suivi — la garantie « ni perte ni doublon »
   porte sur la **jonction rejeu→suivi au sein d'une même invocation**,
   assurée par le curseur `seq` du journal (schéma v1, 007).
3. **Given** un équipier arrêté, **When** l'utilisateur s'attache, **Then** la
   relecture du journal fonctionne, avec un bandeau d'état « arrêté » et la
   date du dernier événement.

---

### User Story 2 - Parler à un équipier depuis la vue (Priority: P2)

Dans la vue attachée, l'utilisateur tape un texte et l'envoie : c'est un
message Bridget ordinaire, marqué comme venant de l'humain, qui suit le circuit
normal (file, tour, garde-fous). La réponse de l'équipier apparaît dans le même
flux.

**Why this priority** : ferme la boucle « voir + parler » qui faisait la valeur
du pane tmux, sans réintroduire de terminal simulé.

**Independent Test** : s'attacher, envoyer « quel est ton domaine ? », voir la
réponse arriver dans le flux ; vérifier dans le ledger que l'échange est un
message normal tracé.

**Acceptance Scenarios**:

1. **Given** une vue attachée à un équipier inactif, **When** l'utilisateur
   envoie un texte, **Then** un tour démarre, la réponse s'affiche dans le
   flux, et le message est attribué à l'humain dans le journal.
2. **Given** l'équipier est en plein tour, **When** l'utilisateur envoie un
   texte, **Then** le message est mis en file (règles 007 inchangées) et la vue
   l'indique (« en attente, tour en cours »).
3. **Given** deux vues attachées au même équipier, **When** l'une envoie un
   message, **Then** l'autre voit le message et la réponse dans son flux.
4. **Given** l'équipier est en « ne pas déranger », arrêté, injoignable, ou sa
   file est pleine, **When** l'utilisateur envoie un texte, **Then** le refus
   ou l'échec de livraison s'affiche dans la vue avec son motif exact (celui du
   `Nack`/`DeliveryRejected` du daemon) — jamais un envoi silencieusement
   perdu.
5. **Given** un envoi accepté puis un échec de livraison ultérieur (mort du
   processus, purge), **When** l'échec est remonté par le daemon, **Then** il
   apparaît dans la vue, attribué au message concerné.

---

### User Story 3 - S'attacher à un équipier d'une autre machine (Priority: P3)

`bridget attach <nom>` fonctionne aussi quand l'équipier tourne sur une machine
fédérée : même vue, même interaction, à travers le tunnel existant.

**Why this priority** : la fédération est l'argument différenciant de Bridget ;
un équipier distant invisible n'aurait pas de sens. Priorité 3 car la valeur
locale existe sans elle.

**Independent Test** : équipier sur machine distante (ou boucle SSH locale du
gate T712), `bridget attach` depuis le poste local : relecture + flux + envoi
fonctionnent.

**Acceptance Scenarios**:

1. **Given** un équipier distant `connected`, **When** l'utilisateur s'attache
   localement, **Then** le flux s'affiche avec une latence acceptable et
   l'envoi de texte fonctionne.
2. **Given** la liaison tombe pendant une vue distante, **When** la fédération
   se reconnecte (mécanisme 002), **Then** la vue se resynchronise et signale
   la coupure plutôt que d'afficher un flux figé sans explication.

---

### Edge Cases

- **Attach sur un agent tmux (non-ACP)** : refus motivé — « agent interactif,
  ouvre son pane tmux » — avec le nom du pane si connu.
- **Attach sur un nom inconnu** : refus listant les équipiers attachables.
- **Réalités du suivi de fichier JSONL** (scénarios d'acceptation exigés) :
  - *dernière ligne partielle* (écriture en cours) : le lecteur attend le
    newline, ne traite jamais un fragment, ne panique pas ;
  - *ligne invalide* (JSON corrompu) : son `seq` est par définition
    inconnaissable — elle est signalée dans la vue par **numéro de ligne et
    offset dans le fichier** (« ligne 412, offset 83214 : illisible, seq
    inconnu ») puis le suivi continue — jamais d'arrêt du flux ;
  - *fichier pas encore créé* (équipier lancé sans tour) : la vue s'ouvre,
    affiche « en attente du premier événement », et enchaîne dès création ;
  - *rotation à minuit* : bascule vers le fichier du jour suivant **sans trou
    ni doublon**, prouvée par la continuité de `seq` ;
  - *troncature ou renommage pendant le suivi* : détectés (taille < offset,
    inode changé), signalés dans la vue, reprise propre depuis le nouvel état.
- **Équipier très bavard** : l'**intégrité du journal** et la **fidélité du
  rendu** sont deux garanties distinctes — le journal ne perd jamais rien (007)
  ; la vue lit via un tampon borné, sans jamais exercer de contre-pression sur
  l'écrivain ; si le rendu prend du retard ou tronque l'affichage, il le
  **signale** (« +N événements, rejouables ») et le rejeu reste possible.
- **Plusieurs vues + détachements en rafale** : aucun état résiduel côté
  équipier ; une vue est strictement un lecteur.
- **Contenu hostile** : le texte produit par un agent est une donnée non fiable
  affichée dans un terminal — voir FR-010.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** : `bridget attach <nom>` DOIT ouvrir une vue sur un équipier ACP
  désigné par son nom d'annuaire ; noms invalides et agents non-ACP sont
  refusés avec un message utile.
- **FR-002** : la vue DOIT afficher d'abord l'historique choisi (rejeu du
  journal **schéma v1** de la 007 : `seq`, `session_id`, payloads typés avec
  expéditeur) puis les événements en continu — tours (début/fin, expéditeur,
  `stop_reason`), texte, outils appelés, permissions, erreurs — horodatés et
  mis en forme lisiblement (pas de JSON brut).
- **FR-003** : sur le **plan de l'observation**, la vue DOIT être un lecteur
  pur : aucune écriture, verrou ou troncature du journal, aucune commande ACP
  émise, aucune contre-pression sur l'écrivain du journal ; s'attacher, se
  détacher ou multiplier les vues ne crée aucun état résiduel côté équipier.
  (L'envoi de messages — FR-005 — est un canal séparé qui n'appartient pas au
  plan d'observation.)
- **FR-004** : au sein d'une même invocation, la jonction rejeu→suivi DOIT être
  sans perte ni doublon, garantie par le curseur `seq` du journal. Une nouvelle
  invocation rejoue volontairement sa fenêtre d'historique. (La reprise
  inter-invocations par point de contrôle persistant est explicitement hors
  périmètre.)
- **FR-005** : la saisie envoyée depuis la vue DOIT emprunter le chemin d'envoi
  existant (message `Send` du daemon), attribuée à l'humain, soumise à tous les
  mécanismes existants (file 007, DND, garde-fous, ledger) — aucun canal
  privilégié — et la vue DOIT afficher les issues non heureuses : refus DND
  avec temps restant, cible arrêtée ou injoignable, file pleine, échec de
  livraison ultérieur. **Périmètre 008** (arbitré en revue des tâches) : les
  envois depuis la vue sont `reply=false` — la réponse de l'équipier est
  **observée dans le flux** (événements de tour), sans créer de demande suivie
  ni de cycle de relances.
- **FR-006** : les réponses et messages tiers concernant l'équipier DOIVENT
  apparaître dans toutes les vues attachées.
- **FR-007** : l'attach DOIT fonctionner vers un équipier d'une machine fédérée
  avec les mêmes capacités **tant que son wrapper est connecté**, par une
  **extension applicative sur la socket fédérée existante** (aucun nouveau
  canal réseau, aucun accès au système de fichiers distant) : le wrapper
  distant est le lecteur de son propre journal, le daemon est le multiplexeur
  vers les vues attachées, et le protocole d'abonnement couvre l'instantané
  initial, le curseur de reprise (`seq`), le suivi continu et la terminaison
  propre de l'abonnement (déconnexion du client comprise). Quand le wrapper
  distant a disparu (équipier distant arrêté), l'attach affiche un message
  d'indisponibilité explicite — « journal distant inaccessible : l'équipier et
  son wrapper sont arrêtés sur <hôte> » — sans prétendre à la relecture ; un
  relais distant persistant est explicitement hors périmètre (session 09+).
- **FR-008** : une coupure (équipier arrêté, fédération tombée) DOIT être
  signalée dans la vue avec son motif, et la resynchronisation après
  reconnexion DOIT repartir du curseur ; la relecture du journal reste
  disponible pour un équipier arrêté local.
- **FR-009** : le rendu DOIT rester du terminal simple (sortie texte ANSI) —
  pas de dépendance TUI lourde ; la vue est utilisable dans un pane tmux comme
  dans un terminal nu.
- **FR-010** : le contenu issu des données (texte d'agents, corps de messages)
  est **non fiable** : les séquences de contrôle et d'échappement (ANSI/OSC)
  qu'il contient DOIVENT être neutralisées à l'affichage — seules les séquences
  générées par le rendu lui-même sont admises ; la ligne de saisie DOIT rester
  intacte pendant l'arrivée d'événements ; le terminal DOIT être restauré dans
  tous les chemins de sortie (détachement, Ctrl-C, EOF, erreur). Des fixtures
  hostiles (journal contenant des séquences d'injection) font partie des tests
  d'acceptation.

### Key Entities

- **Vue attachée** : lecteur nommé du flux d'un équipier — position de lecture
  dans le journal, filtres d'affichage éventuels ; n'existe que côté client.
- **Événement rendu** : projection lisible d'une ligne de journal 007
  (`turn_start`/`update`/`permission`/`turn_end`/`error`) enrichie de
  l'expéditeur et de l'horodatage.
- **Message humain** : message Bridget ordinaire dont l'émetteur est marqué
  humain **au ledger** ; le client attach lui-même n'apparaît **jamais dans
  l'annuaire** (connexion à rôle dédié, hors routage — corrigé en revue des
  tâches).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** : en local, mesuré de la **fin de l'append complet** (newline
  écrit) jusqu'au rendu, sous une cadence fixée de 10 événements/s pendant
  60 s : latence p95 < 1 s, maximum < 3 s. Le distant a son propre budget :
  p95 < 3 s dans les mêmes conditions.
- **SC-002** : ouverture de vue = fenêtre d'historique affichée, puis **zéro
  perte et zéro doublon à la jonction rejeu→suivi** (vérifiée par la continuité
  de `seq`) pendant un tour actif, y compris à travers une rotation de minuit
  simulée.
- **SC-003** : un échange complet humain → équipier → réponse **observée dans
  le flux** se fait depuis la vue sans aucune commande annexe, le message étant
  tracé au ledger comme message ordinaire (`reply=false`, pas de demande
  suivie) ; et chacun des quatre cas non heureux (DND, arrêté, file pleine,
  échec de livraison différé) affiche son motif dans la vue.
- **SC-004** : deux vues simultanées affichent le même flux ; la fermeture des
  deux ne laisse aucun processus ni fichier résiduel, et le terminal est
  restauré dans chaque chemin de sortie.
- **SC-005** : budget d'observation mesuré **sur un faux adaptateur
  déterministe** (pas de modèle réel : la durée d'un tour LLM est trop bruitée
  pour un p95 à petit échantillon) — sur N ≥ 200 tours simulés identiques, la
  dégradation p95 de la **latence d'append au journal** (métrique instrumentée)
  avec 2 vues attachées est **< 5 %** par rapport à 0 vue. Tout retard de rendu
  est signalé dans la vue, jamais silencieux.
- **SC-006** : l'attach distant passe les scénarios des US1 et US2 dans un
  environnement où les répertoires (`HOME`, caches) du poste local et de la
  machine de l'équipier sont **réellement distincts** — une boucle SSH locale
  partageant le même système de fichiers ne suffit pas à valider ce critère.
- **SC-007** : les fixtures hostiles (séquences ANSI/OSC dans le journal)
  s'affichent neutralisées ; aucune altération du terminal ni de la ligne de
  saisie.

## Assumptions

- Le journal JSONL **schéma v1** de la 007 (`seq` continu, `session_id`,
  payloads typés avec expéditeur, fixtures de compatibilité lecteur) est le
  contrat de lecture — cette spec dépend de sa livraison en 007-T706. L'attach
  ne s'insère jamais dans la session ACP (FR-003) ; le mécanisme précis de
  lecture continue locale et le protocole d'abonnement distant (FR-007) sont
  détaillés au plan.
- L'identification « humain » de l'émetteur réutilise ce qui existe (les
  messages CLI sont déjà distingués des agents) ; pas de gestion d'identité
  nouvelle.
- Le périmètre exclut : le lancement d'équipiers depuis la vue (session 09),
  la reprise inter-invocations par point de contrôle persistant, toute
  persistance de préférences d'affichage, et tout rendu autre que terminal
  texte.
