# Feature Specification: Afficher l'identité runtime des agents

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 071-identite-runtime-agent
Titre: Afficher l'identité runtime des agents
Statut: Implemented
Priorité: P1
Tâches: 12/12 (100%)
Tests: 145/145 ciblés (100%)

Résumé:
- Contexte: la liste latérale permet de reconnaître un agent mais pas de savoir clairement quel produit agentique, éditeur et mode d'exécution se trouvent derrière lui.
- Objectif: présenter ces informations dans une fiche compacte, lisible et exacte, sans surcharger chaque ligne d'agent.
- Risque principal: déduire le runtime ou le fournisseur depuis le nom de l'agent et afficher une identité fausse.
- Mitigation: utiliser uniquement des métadonnées explicites, distinguer les concepts et afficher un état inconnu lorsque la donnée manque.
- Validation: une matrice Codex, Claude Code, Cursor, Gemini CLI, TMUX, FLUX et données incomplètes produit une fiche exacte, accessible et non intrusive.
- Dépendances: SPEC-024, SPEC-046 et SPEC-070. La feature reste indépendante de la livraison de SPEC-064.
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-071-identite-runtime-agent`
**Created**: 2026-08-30
**Status**: Implemented
**Priority**: P1
**Dependencies**: SPEC-024, SPEC-046, SPEC-070

## Contexte et problème

La colonne de gauche donne aujourd'hui le nom, l'avatar, la présence et un
aperçu du dernier message de chaque agent. Au survol, un grand panneau reprend
surtout le contenu du message. Il ressemble à une bulle de conversation,
masque une partie de la liste et ne répond pas à la question essentielle :
quel environnement exécute réellement cet agent ?

Quatre informations différentes sont actuellement faciles à confondre :

- le produit agentique utilisé, par exemple Codex, Claude Code, Cursor ou
  Gemini CLI ;
- l'éditeur du produit agentique, par exemple OpenAI, Anthropic, Anysphere ou
  Google ;
- le mode d'exécution, soit TMUX, soit FLUX ;
- le transport technique, qui explique le chemin d'exécution mais ne doit pas
  devenir l'identité principale affichée.

La fiche doit rendre ces distinctions immédiatement lisibles, sans remplacer
l'avatar propre de l'agent et sans deviner une information depuis son nom.

## User Scenarios & Testing

### User Story 1 - Identifier le runtime réel d'un agent (Priority: P1)

Comme utilisateur qui supervise plusieurs agents, je veux voir dans une fiche
compacte le produit agentique, son éditeur et le mode d'exécution,
afin de comprendre immédiatement ce qui travaille derrière chaque identité.

**Independent Test**: présenter des agents de test couvrant Codex, Claude Code,
Cursor et Gemini CLI, avec les modes TMUX et FLUX, puis vérifier que chaque
fiche restitue exactement les métadonnées fournies et le catalogue local de
produits audité.

**Acceptance Scenarios**:

1. **Given** un agent portant des métadonnées complètes, **When** sa fiche est
   ouverte, **Then** elle affiche séparément son produit agentique, son
   éditeur et son mode d'exécution.
2. **Given** un agent Cursor utilisant un modèle dont le fournisseur n'est pas
   attesté, **When** sa fiche est ouverte, **Then** Cursor et Anysphere restent
   l'identité du produit, tandis que le modèle est affiché seulement par son
   libellé attesté.
3. **Given** un agent dont une métadonnée manque, **When** sa fiche est
   ouverte, **Then** la valeur concernée est explicitement inconnue et n'est
   jamais déduite du nom, d'un suffixe ou de l'avatar.

### User Story 2 - Reconnaître les fournisseurs sans surcharger la liste (Priority: P1)

Comme utilisateur, je veux reconnaître visuellement les produits agentiques par
leurs marques officielles dans la fiche, tout en conservant l'avatar de l'agent
comme identité principale dans la liste.

**Independent Test**: ouvrir successivement les fiches des quatre produits
supportés et vérifier le logo officiel, le libellé textuel associé, l'absence
de conteneur décoratif autour du logo et l'absence de requête vers un domaine
tiers.

**Acceptance Scenarios**:

1. **Given** un produit agentique connu, **When** sa fiche est affichée,
   **Then** son logo officiel non déformé apparaît avec un libellé textuel.
2. **Given** un produit agentique inconnu, **When** sa fiche est affichée,
   **Then** un symbole neutre et le libellé « Inconnu » remplacent le logo sans
   usurper une marque.
3. **Given** l'indisponibilité du réseau public, **When** la fiche est ouverte,
   **Then** tous les éléments d'identité restent disponibles.

### User Story 3 - Lire la fiche sans gêner la navigation (Priority: P1)

Comme utilisateur qui parcourt rapidement la flotte, je veux une fiche plus
petite et mieux hiérarchisée que le panneau actuel, afin de lire l'identité de
l'agent sans perdre le contexte de la liste.

**Independent Test**: ouvrir la fiche au pointeur puis au clavier dans une
fenêtre de 1280 par 720 pixels, passer du déclencheur à la fiche, la fermer avec
Échap et vérifier que le nom de l'agent, les lignes voisines et le contenu
principal restent utilisables.

**Acceptance Scenarios**:

1. **Given** une ligne d'agent visible, **When** elle est survolée ou reçoit le
   focus clavier, **Then** une fiche d'identité apparaît sans transformer la
   ligne elle-même en tableau de badges.
2. **Given** une fiche ouverte, **When** le pointeur passe de la ligne à la
   fiche, **Then** la fiche reste stable assez longtemps pour être lue.
3. **Given** une fiche ouverte au clavier, **When** l'utilisateur appuie sur
   Échap, **Then** elle se ferme et le focus reste dans la navigation des
   agents.
4. **Given** une fiche ouverte, **When** le dernier message est long, **Then**
   son aperçu reste secondaire et borné à deux lignes visibles.

## Functional Requirements

- **FR-7101**: La liste DOIT conserver l'avatar et le nom comme identité
  principale de l'agent, sans ajouter en permanence les logos fournisseur et
  les modes d'exécution sur chaque ligne.
- **FR-7102**: La fiche DOIT distinguer quatre champs : produit agentique,
  éditeur du produit, mode d'exécution et transport technique.
- **FR-7103**: Le produit agentique DOIT être identifié par une valeur
  explicite parmi Codex, Claude Code, Cursor, Gemini CLI ou Inconnu.
- **FR-7104**: L'éditeur DOIT provenir du catalogue local audité qui associe
  chaque produit supporté à OpenAI, Anthropic, Anysphere ou Google. Un produit
  absent du catalogue DOIT afficher un éditeur inconnu.
- **FR-7105**: Le mode d'exécution DOIT être affiché dans une puce textuelle
  `TMUX` ou `FLUX`. Toute autre valeur doit être présentée comme inconnue.
- **FR-7106**: Le runtime et le mode ne DOIVENT jamais être
  déduits du nom, d'un suffixe, de l'avatar ou du dernier message de l'agent.
- **FR-7107**: Chaque produit agentique connu DOIT utiliser son logo officiel,
  accompagné de son nom en texte et sans bordure, pastille ou fond décoratif
  propre au logo.
- **FR-7108**: Les logos DOIVENT rester disponibles sans requête vers un
  service tiers et leur provenance officielle DOIT être documentée.
- **FR-7109**: La fiche DOIT présenter au minimum le nom de l'agent, son état
  de présence, son produit agentique, son éditeur, son mode
  d'exécution et l'âge de sa dernière activité.
- **FR-7110**: Le modèle, le niveau d'effort et le transport technique DOIVENT
  apparaître seulement lorsqu'ils sont attestés ; une valeur absente ne doit
  pas être inventée.
- **FR-7111**: L'aperçu du dernier message DOIT rester secondaire, être borné
  à deux lignes et ne pas donner à la fiche l'apparence d'une bulle de message.
- **FR-7112**: La fiche DOIT pouvoir être ouverte au pointeur et au clavier,
  rester ouverte pendant son survol et se fermer avec Échap.
- **FR-7113**: L'ouverture et la fermeture de la fiche ne DOIVENT modifier ni
  la sélection courante, ni l'état d'activité, ni le contenu du fil.
- **FR-7114**: Les libellés d'identité et de mode DOIVENT rester
  compréhensibles sans dépendre de la couleur ou de la reconnaissance d'un
  logo.

## Non-Functional Requirements

- **NFR-7101**: La fiche visible DOIT avoir une largeur maximale de 360 pixels
  dans une fenêtre d'au moins 1280 par 720 pixels.
- **NFR-7102**: La fiche DOIT apparaître au plus 300 millisecondes après le
  survol stable ou le focus, sans requête réseau supplémentaire.
- **NFR-7103**: Les logos DOIVENT conserver leur ratio, rester nets aux tailles
  d'affichage prévues et respecter les règles publiques de leur marque.
- **NFR-7104**: Le comportement DOIT rester utilisable avec le thème sombre
  actuel, un zoom navigateur de 200 % et la navigation clavier.
- **NFR-7105**: La feature ne DOIT ajouter aucune dépendance d'exécution ni
  modifier le cycle de vie, le routage ou l'autorité des agents.

## Success Criteria

- **SC-7101**: Dans la matrice de validation, 100 % des agents portant des
  métadonnées complètes affichent le produit, l'éditeur et le mode
  attendus.
- **SC-7102**: Dans les cas de données incomplètes, zéro identité n'est déduite
  du nom de l'agent et chaque valeur absente est explicitement indiquée comme
  inconnue.
- **SC-7103**: Les quatre produits connus disposent d'un logo officiel, d'un
  libellé textuel et d'une provenance vérifiable, sans aucun chargement tiers
  à l'ouverture de la fiche.
- **SC-7104**: Dans une fenêtre de 1280 par 720 pixels, la fiche reste sous 360
  pixels de largeur, le dernier message ne dépasse pas deux lignes et les
  lignes voisines restent identifiables.
- **SC-7105**: Le parcours clavier permet d'ouvrir, lire et fermer la fiche
  sans changer l'agent sélectionné ni perdre le focus de navigation.
- **SC-7106**: Les tests de non-régression prouvent que présence, activité,
  sélection et fil de conversation sont identiques avant et après l'ajout de
  la fiche.

## Key Entities

- **Identité d'agent**: nom et avatar propres à l'agent, indépendants du
  produit agentique qui l'exécute.
- **Produit agentique**: environnement utilisateur qui porte l'agent, par
  exemple Codex, Claude Code, Cursor ou Gemini CLI.
- **Éditeur du produit**: organisation qui publie le produit agentique. Cette
  information ne prétend pas identifier le fournisseur du modèle exécuté.
- **Mode d'exécution**: frontière opérationnelle explicite, TMUX pour une
  session interactive ou FLUX pour une session gérée.
- **Transport technique**: mécanisme attesté de communication avec le runtime,
  affiché comme détail et non comme identité.
- **Fiche d'identité**: vue temporaire et accessible regroupant les faits
  utiles sans modifier l'agent ni son fil.

## Edge Cases

- Un runtime connu exécute un modèle dont le fournisseur réel n'est pas
  attesté ; la fiche ne doit pas inventer ce fournisseur.
- Le mode est absent ou ne correspond ni à TMUX ni à FLUX.
- Le modèle ou le niveau d'effort n'est pas fourni.
- Un nom d'agent se termine par `-flux` alors que son mode explicite ne vaut
  pas FLUX.
- La fiche doit se repositionner près du bord droit ou inférieur de la fenêtre.
- Un logo monochrome doit rester lisible sur le thème sombre sans être
  recoloré en dehors des variantes autorisées.

## Assumptions

- Les métadonnées disponibles dans le registre ou le flux de présence restent
  la seule source de vérité opérationnelle de la fiche ; le catalogue local
  ne fournit que le nom, l'éditeur et le logo d'un runtime explicite.
- Le terme FLUX désigne dans cette interface une exécution gérée, distincte
  d'une session interactive TMUX.
- La langue de l'interface reste celle déjà servie par Bridget ; les noms de
  produits et de marques ne sont pas traduits.
- L'interface peut afficher une information inconnue plutôt que tenter de la
  reconstruire.

## Hors périmètre

- Remplacer les avatars des agents par les logos des fournisseurs.
- Permettre de choisir ou changer le runtime, le modèle, l'éditeur ou le
  mode depuis la fiche.
- Déduire des métadonnées à partir du nom de l'agent.
- Modifier le lancement, le transport, la reprise, les permissions ou
  l'orchestration Bridget et Maicie.
- Ajouter un annuaire de marques distant, une bibliothèque d'icônes ou une
  requête réseau au rendu.
- Épingler durablement la fiche ou refondre toute la colonne latérale.

## Preuves d'implémentation

### Matrice fonctionnelle

| Exigences | Preuve |
|---|---|
| FR-7101, FR-7111 | la ligne conserve avatar, nom et extrait ; l'ancien tooltip imbriqué est supprimé ; le nouvel extrait est limité à deux lignes |
| FR-7102 à FR-7106 | projection Rust de `transport`, `mode`, `model`, `effort` et tests Node du catalogue fermé sans inférence depuis le nom |
| FR-7107, FR-7108 | quatre SVG officiels embarqués, routes locales testées et provenance avec SHA-256 dans `providers/NOTICE.md` |
| FR-7109, FR-7110, FR-7114 | test de `identityCardData` et rendu textuel du nom, de la présence, du produit, de l'éditeur, du mode et des seuls détails attestés |
| FR-7112, FR-7113 | câblage productif testé pour survol, focus, maintien dans la fiche, Échap et nettoyage au rendu, au scroll et au redimensionnement |
| SC-7101 à SC-7103 | matrice Node Codex, Claude Code, Cursor, Gemini CLI, TMUX, FLUX et inconnus ; routes d'assets Rust exactes |
| SC-7104 | test CSS et prévisualisation Chromium à 1280 par 720 : largeur mesurée 344 pixels |
| SC-7105, SC-7106 | invariants d'accessibilité, de fermeture et suites UI de non-régression |

### Commandes exécutées

- `node crates/bridget-daemon/assets/ui/app.js` : 81 passés, 0 échec.
- `cargo test -p bridget-daemon ui::` : 43 passés, 0 échec.
- `cargo test -p bridget-daemon --test ui_relay_test` : 21 passés, 0 échec.
- `cargo fmt --all -- --check` : succès.
- `git diff --check` : succès.
- contrôle de sûreté des SVG : aucune surface active détectée.

### Limites de validation

Le parcours live complet reste à faire après compilation et mise en service
explicites. Il n'a pas été forcé pendant cette session afin de ne pas arrêter
le relais UI de production. La prévisualisation Chromium isolée valide le
rendu statique à 1280 par 720.

La suite complète `cargo test -p bridget-daemon` s'arrête sur trois tests de
`managed_parity_test`. Les trois échecs sont reproduits sur `origin/main` au
commit `a63cf97`, sans les modifications de SPEC-071. De même, Clippy strict
échoue sur deux diagnostics préexistants de `bridget-transport`, reproduits
sur la même base. Ces écarts ne sont pas corrigés dans cette feature.
