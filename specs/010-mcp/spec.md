# Feature Specification : Outils MCP Bridget — la voie montante native

**Feature Branch**: `session-10-mcp` (ordre d'implémentation inchangé : après 008 et 009 ; cette spec est anticipée pendant le développement 007)
**Created**: 2026-08-22
**Status**: Clôturée — implémentée, revue et mergée (8/8)
**Input**: User description: « pendant que l'ACP se développe, spécifier la voie MCP : que les agents disposent de bridget_send / bridget_who / bridget_ledger comme outils natifs, à la place des commandes shell et des règles de prompt. »

## Contexte et problème

La session 007 supprime l'artifice sur la voie **descendante** (livraison des
messages). Reste la voie **montante** : quand un agent veut, de sa propre
initiative, écrire à un autre agent ou consulter l'annuaire, il doit composer
une commande shell `bridget send …` — avec trois faiblesses structurelles :

1. **Dégradation en session longue** : la consigne d'utiliser la commande vit
   dans le prompt, qui se compacte ; l'outil, lui, est retransmis
   structurellement à chaque tour par le harness.
2. **Champ de mines de l'échappement shell** : apostrophes, guillemets, `$`,
   sauts de ligne — tout corps de message riche casse ou se tronque.
3. **Résultats à parser** : un refus (DND, disjoncteur, file pleine) revient en
   texte de stderr, que l'agent doit interpréter.

Le Model Context Protocol (MCP) est le standard par lequel tous les CLI
concernés (Claude Code, Codex, Gemini — et les adaptateurs ACP via le paramètre
`mcpServers` de `session/new`, constaté au spike 007-T701) consomment des
outils externes : paramètres typés validés par le harness, résultats
structurés, présence garantie dans le contexte. Cette session expose Bridget
comme serveur MCP local.

## Positionnement

- **MCP ne remplace pas la livraison** : les messages entrants continuent
  d'arriver par le transport (ACP ou tmux). MCP couvre l'initiative de l'agent.
- **Le binaire `bridget` reste** : voie humaine (terminal) et repli universel.
  Même daemon, mêmes garde-fous — un message envoyé par outil ou par binaire
  est indistinguable une fois dans le daemon.

## Dépendances

- **007-transport-acp** : registre d'agents (branchement déclaratif), variantes
  de protocole livrées, paramètre `mcpServers` d'ACP pour les équipiers.
- **003/005** : le ledger et les refus motivés (DND) sont les contenus que les
  outils exposent — inchangés.
- Session 008 : indépendante. **Session 009** : frontière explicite — la 010
  livre l'extension du schéma de registre et l'injection éphémère par session ;
  la 009 ne fera que **transmettre** cette configuration au spawn, sans aucune
  logique MCP propre.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Envoyer par outil, sans shell (Priority: P1)

Un agent en pleine tâche veut poser une question à un autre agent. Il appelle
l'outil `bridget_send` avec destinataire, corps et attente de réponse en
paramètres typés. Le corps traverse intact (multiligne, caractères spéciaux),
et le résultat de l'outil lui dit précisément ce qui s'est passé : accepté (id,
sauts), ou refusé avec le motif exact (DND et temps restant, disjoncteur,
doublon, budget de sauts, destinataire inconnu).

**Why this priority** : c'est la suppression du dernier artifice (échappement
shell + interprétation de stderr) sur le chemin le plus fréquent.

**Independent Test** : configurer le serveur MCP sur un agent Claude Code
interactif, lui faire envoyer un message contenant apostrophes/`$`/sauts de
ligne à un équipier, vérifier corps intact à l'arrivée et résultat d'outil
structuré au départ ; puis répéter vers un agent en DND et constater le motif
dans le résultat de l'outil.

**Acceptance Scenarios**:

1. **Given** un agent équipé de l'outil, **When** il appelle `bridget_send`
   avec un corps riche (multiligne, quotes, `$`, backticks), **Then** le corps
   arrive octet pour octet au destinataire et le résultat contient l'id du
   message.
2. **Given** un destinataire en DND, **When** l'agent appelle `bridget_send`,
   **Then** le résultat de l'outil porte le refus, le motif et le temps
   restant — sans texte à parser.
3. **Given** des paramètres invalides (destinataire vide, corps manquant),
   **Then** l'erreur de validation est retournée à l'agent par le mécanisme
   d'outils de son harness, qui peut corriger et rappeler.
4. **Given** le daemon est arrêté, **Then** l'appel d'outil échoue avec un
   message explicite (« daemon Bridget injoignable ») — jamais de blocage.

---

### User Story 2 - Consulter l'annuaire et le ledger par outil (Priority: P2)

L'agent appelle `bridget_who` (liste structurée des agents : nom, type, hôte,
domaine, état, modèle) et `bridget_ledger` avec une vue explicite —
`requests` (demandes suivies : état, échéances, qui attend quoi) ou
`messages` (messages récents) ou `both` — reflétant la distinction que le
binaire fait déjà entre `bridget requests` et `bridget ledger`. Il peut ainsi
choisir un destinataire ou vérifier qu'une réponse est attendue de lui, sans
commande shell ni parsing de tableau.

**Why this priority** : complète la boucle décisionnelle de l'agent ;
s'appuie sur la même infrastructure que US1.

**Independent Test** : appel `bridget_who` → la liste correspond à `bridget
who` ; appel `bridget_ledger` → les demandes ouvertes concernant l'agent, avec
leurs échéances.

**Acceptance Scenarios**:

1. **Given** plusieurs agents connectés, **When** l'agent appelle
   `bridget_who`, **Then** il reçoit la liste structurée, identique en contenu
   à la sortie du binaire.
2. **Given** une demande suivie ouverte vers l'agent, **When** il appelle
   `bridget_ledger` avec `view=requests`, **Then** elle apparaît avec émetteur,
   échéance et état — et `view=messages` retourne les messages récents, chacun
   dans son schéma propre.

---

### User Story 3 - Branchement déclaratif et prompt allégé (Priority: P3)

Le wrapper branche le serveur MCP automatiquement au lancement, selon le type
d'agent : configuration MCP du CLI pour les agents interactifs, paramètre
`mcpServers` de la session ACP pour les équipiers. L'identité de l'agent
appelant est établie par le serveur sans que l'agent ne la déclare. Pour les
agents ainsi équipés, le bloc d'instructions Bridget du prompt se réduit à
l'essentiel (reconnaître un message entrant), les règles d'envoi devenant
inutiles.

**Why this priority** : c'est l'industrialisation — zéro configuration
manuelle, identité fiable, prompt minimal.

**Independent Test** : lancer un agent de chaque type (interactif tmux,
équipier ACP) ; vérifier l'outil présent sans action manuelle, l'expéditeur
correct dans le ledger, et le prompt de session réduit pour les agents
équipés.

**Acceptance Scenarios**:

1. **Given** un équipier ACP lancé par le wrapper, **When** il appelle
   `bridget_send`, **Then** l'expéditeur enregistré est le nom d'annuaire de
   l'équipier — sans variable déclarée par l'agent lui-même.
2. **Given** un agent interactif lancé par le wrapper avec branchement MCP,
   **Then** son prompt de session ne contient plus les règles de syntaxe
   d'envoi (le bloc se réduit à la sémantique des messages entrants).
3. **Given** un agent renommé en cours de session (`bridget rename`), **When**
   il envoie ensuite par outil, **Then** l'expéditeur est le nom courant.

---

### Edge Cases

- **Identité introuvable** (serveur MCP lancé hors d'un agent Bridget) : les
  outils répondent par une erreur explicite (« aucun agent Bridget associé à ce
  processus »), jamais par une identité devinée.
- **Pid recyclé après crash** : l'entrée `agent-pids/` typée (naissance du
  processus + `instance_id`, FR-004) rend l'usurpation impossible — un pid
  réutilisé par l'OS échoue à la validation et l'entrée orpheline est ignorée.
- **Deux agents sur la même machine** (y compris même binaire) : chaque
  instance du serveur MCP est liée à son agent (une instance par agent, lancée
  par son harness) — pas de confusion d'expéditeur, la filiation retenant le
  premier ancêtre **validé**.
- **Daemon redémarré pendant la session** : l'appel suivant se reconnecte ou
  échoue explicitement ; jamais de file d'attente silencieuse côté serveur MCP.
- **Corps volumineux** : même limite que le binaire (celle du daemon) ; refus
  motivé au-delà, pas de troncature silencieuse.
- **Appels concurrents** (agent qui envoie pendant qu'il consulte) : sûrs — le
  serveur est sans état partagé mutable entre appels.
- **Agent hors registre / type inconnu** : le branchement échoue au lancement
  avec le message du registre (007), pas à l'appel d'outil.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** : Bridget DOIT fournir un serveur MCP local (sous-commande
  dédiée, transport stdio) exposant exactement trois outils : `bridget_send`,
  `bridget_who`, `bridget_ledger` — schémas de paramètres typés et documentés
  en français.
- **FR-002** : `bridget_send(to, body, reply, reply_timeout?)` DOIT transmettre
  le corps **octet pour octet** au daemon et retourner un résultat structuré :
  accepté (id, hops) ou refusé (catégorie de refus + motif exact du daemon +
  données utiles, ex. minutes restantes de DND). **Sémantique temporelle
  explicite** : l'appel d'outil retourne **dès l'accusé ou le refus** — il
  n'attend jamais la réponse du destinataire ; `reply=true` crée une demande
  suivie dont `reply_timeout` appartient au cycle de vie du **daemon** (il ne
  borne pas l'appel MCP) ; `reply_timeout` fourni avec `reply=false` est une
  erreur `invalid_params` (inscrite au schéma). L'appel lui-même a un timeout
  court dédié connexion + accusé, **valeur concrète au registre** (défaut
  10 s) — la gate de support (FR-012) exige que chaque harness pinné ait un
  timeout d'outil strictement supérieur, ce qui rend la contrainte testable.
  **Idempotence** : l'id métier du message est généré **avant** la connexion
  au daemon ; un échec avant écriture est distingué d'un `outcome_unknown`
  après écriture (Ack perdu) ; une annulation MCP ne révoque **jamais** un
  envoi déjà transmis ; un retry explicite avec le même id est possible et la
  déduplication du daemon s'applique. Tests : Ack perdu après acceptation,
  `cancelled` pendant l'attente, retry même id vs nouvel id.
- **FR-003** : les appels d'outils DOIVENT emprunter le même protocole
  daemon que le binaire (même socket, mêmes messages) : tous les garde-fous
  (disjoncteur, déduplication, budget de sauts, DND, cycle de vie des
  demandes) s'appliquent inchangés, et le ledger trace indistinctement les
  deux voies.
- **FR-004** : l'identité de l'expéditeur DOIT être établie par le serveur MCP
  lui-même, **résolue à chaque appel d'outil** (jamais figée au lancement),
  dans cet ordre : (1) **localisateur dynamique** = le fichier de nom courant
  (`BRIDGET_AGENT_NAME_FILE`, déjà posé par `wrapper.rs:552`), s'il est
  présent et valide — un `instance_id` seul n'est un localisateur que si une
  requête daemon typée le résout en nom courant (à définir au plan si retenu ;
  sinon le fichier de nom est la seule forme) ; (2)
  sinon **filiation processus → agent** via `agent-pids/`, en remontant les
  ancêtres jusqu'à pid 1 avec une borne explicite, le **premier ancêtre
  valide** l'emportant ; (3) sinon erreur explicite. Un nom d'environnement
  figé (`BRIDGET_AGENT_NAME`) n'est **jamais** utilisé comme source d'identité.
  L'entrée `agent-pids/<pid>` DOIT être **typée** : pid, **identité de
  naissance du processus** (horodatage de démarrage, disponible sur macOS et
  Linux), `instance_id`, chemin du fichier de nom courant — et n'est valide
  que si naissance et instance concordent — la naissance étant comparée à la
  valeur **capturée au moment du dépôt du marqueur** par le wrapper (un pid
  recyclé par l'OS ne peut pas usurper une identité). Le renommage en cours de session est suivi (le
  fichier de nom est relu à chaque appel). Tests imposés : `rename` entre deux
  appels du même serveur, deux agents du même binaire, chaîne `npx` à 3
  intermédiaires, agents imbriqués, entrée orpheline puis pid réutilisé.
- **FR-005** : le wrapper DOIT brancher le serveur MCP automatiquement selon le
  type d'agent : configuration MCP du CLI pour les agents interactifs (chaque
  CLI ayant sa forme), paramètre `mcpServers` de `session/new` pour les
  équipiers ACP — le tout déclaré dans le registre d'agents (007), sans code
  spécifique par agent hors registre.
- **FR-006** : pour un agent branché MCP, le bloc d'instructions Bridget
  injecté au prompt DOIT être réduit à la sémantique des messages entrants
  (plus aucune règle de syntaxe d'envoi) ; pour un agent non branché, le
  comportement actuel est inchangé.
- **FR-007** : `bridget_who(domain?)` DOIT retourner l'annuaire structuré
  (mêmes champs que le binaire). `bridget_ledger(view, limit?)` DOIT distinguer
  explicitement les deux contenus que le binaire sépare aujourd'hui
  (`bridget ledger` = messages récents lus en base ; `bridget requests` =
  demandes suivies via `ListRequests`) : `view` ∈ `messages` \| `requests` \|
  `both`, avec un schéma de résultat propre à chaque vue. Les requêtes daemon
  typées nécessaires sont ajoutées, et le **binaire réutilise la même couche de
  lecture** sans changement de sortie — équivalence testée champ par champ.
- **FR-008** : le serveur MCP est un processus stdio concurrent au modèle
  explicite : **lecteur stdin unique** avec corrélation des ids JSON-RPC
  (chaîne et nombre acceptés), **writer stdout sérialisé**, `notifications/
  cancelled` annulant uniquement l'attente locale de l'appel visé, EOF →
  terminaison propre. **Chaque appel d'outil ouvre sa propre connexion
  daemon** (pas de connexion réutilisée : elle peut mourir entre vérification
  et écriture), avec délais bornés de connexion/lecture/écriture et **zéro
  retry** — daemon injoignable = échec explicite immédiat, aucune file locale.
  Tests : appels concurrents, daemon redémarré entre connexion et réponse,
  harness qui abandonne l'appel en cours.
- **FR-009** : le serveur MCP NE DOIT introduire aucune dépendance nouvelle :
  le sous-ensemble MCP nécessaire est implémenté avec les primitives du
  workspace, **avec une matrice de conformité obligatoire** (leçon D-201/R-004
  de la 007) couverte par fixtures officielles avant implémentation :
  `initialize` + négociation `protocolVersion`/capacités (version pinnée),
  `notifications/initialized`, `tools/list` rappelée plusieurs fois
  (idempotente), `tools/call`, `ping`, méthode inconnue, paramètres invalides,
  JSON malformé, ids mixtes et hors ordre, notifications intercalées, EOF,
  **`notifications/cancelled` avant et après l'écriture daemon** (l'annulation
  ne révoque pas un envoi transmis), **réponse produite après annulation**
  (ignorée proprement), **appels concurrents se terminant hors ordre**, et
  **`tools/call` reçu avant `initialized`** (refusé conformément au
  protocole) — et **pureté absolue de stdout** (tout log part sur stderr).
  Chaque harness est testé contre les mêmes fixtures.
- **FR-010** : le binaire `bridget send`/`who`/`ledger`/`requests` reste
  fonctionnel et inchangé en sortie (voie humaine et repli) ; aucune
  dépréciation dans cette session.
- **FR-011** : frontière des erreurs, fermée et extensible : paramètres
  invalides ou méthode inconnue = **erreur JSON-RPC** ; daemon injoignable ou
  timeout interne = résultat `tools/call` avec `isError` structuré ; accusé ou
  refus Bridget = **résultat métier** structuré à catégorie fermée
  (`accepted`, `dnd`, `circuit_breaker`, `duplicate`, `hops_exhausted`,
  `unknown_recipient`, `queue_full`, …) + `reason` texte — un client DOIT
  pouvoir ignorer une catégorie inconnue future sans casser.
- **FR-012** : le branchement est **strictement éphémère et par session** :
  extension du schéma du registre 007 + injection au lancement (`mcpServers`
  de la session ACP, ou forme d'injection au lancement du CLI) ; **aucune
  modification persistante d'une configuration utilisateur** (fichiers
  `~/.claude`, `~/.codex`, `~/.gemini`…) n'est autorisée ; si un CLI ne permet
  pas l'injection par session, son type est déclaré `unsupported` pour MCP
  avec repli binaire — jamais d'écriture globale. **Gate de support
  versionnée** : le spike de branchement teste les quatre voies (Claude,
  Codex, Gemini, équipier ACP) et consigne le résultat par version pinnée ;
  soit les quatre passent et SC-004 s'applique tel quel, soit un type échoue
  et la spec est **explicitement révisée avant implémentation** — un
  `unsupported` ne permet jamais de déclarer SC-004 atteint.

### Key Entities

- **Serveur MCP Bridget** : processus stdio lancé par le harness de l'agent,
  lié à un agent unique, traducteur outils ↔ protocole daemon existant.
- **Outil** : `bridget_send` | `bridget_who` | `bridget_ledger`, chacun avec
  schéma d'entrée typé et résultat structuré, documentés en français.
- **Identité d'appelant** : nom d'annuaire résolu par le serveur **à chaque
  appel**, dans l'ordre exact de FR-004 (fichier de nom courant → filiation
  `agent-pids/` validée par naissance+instance → erreur explicite), jamais
  fourni par l'agent.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** : le corpus de corps riches de la 007 (SC-002) transite à 100 %
  octet pour octet via `bridget_send` (aujourd'hui : échec dès la première
  apostrophe non échappée par la voie shell).
- **SC-002** : sur les quatre familles de refus (DND, disjoncteur, doublon,
  destinataire inconnu), le résultat d'outil porte la catégorie et le motif
  exact dans 100 % des cas — zéro parsing de texte côté agent.
- **SC-003** : l'expéditeur enregistré au ledger est correct dans 100 % des
  scénarios d'identité : env var présente, env filtré (filiation pid seule),
  agent renommé en cours de session, serveur lancé hors agent (erreur
  explicite).
- **SC-004** : les trois types d'agents (Claude, Codex, Gemini) plus un
  équipier ACP obtiennent l'outil par le seul branchement du wrapper — zéro
  configuration manuelle, zéro écriture persistante de config utilisateur
  (vérifié par diff des répertoires de config avant/après) — prouvé par
  fixtures automatisées **plus un smoke test réel par type**, versions des
  harness et adaptateurs pinnées et consignées.
- **SC-005** : le bloc de prompt exact avant/après est **versionné dans le
  dépôt** (fixtures) ; réduction ≥ 60 % mesurée en caractères Unicode
  (méthode de comptage fixée) ; l'absence de perte de comportement est établie
  par une **matrice comportementale définie** : les scénarios quickstart §1-§4
  de la 007 rejoués à l'identique avec le prompt réduit.
- **SC-006** : un message envoyé par outil et le même envoyé par binaire
  produisent des entrées de ledger identiques (hors id/horodatage) — preuve
  que les garde-fous et la traçabilité sont communs.

## Assumptions

- Le sous-ensemble MCP requis (initialisation, `tools/list`, `tools/call` en
  JSON-RPC sur stdio) est stable et documenté ; la conformité est vérifiée par
  fixtures contre le schéma officiel, versions des harness pinnées lors des
  tests (même approche que la matrice R-004 de la 007).
- Le mécanisme de filiation `agent-pids/` (posé en session 006/007 précisément
  pour ce cas : « un serveur MCP lancé par l'agent reçoit un environnement
  filtré ») fonctionne pour les trois CLI ; le spike de branchement le
  vérifiera avant l'implémentation.
- Le modèle de confiance est inchangé (agents coopératifs, même compte, socket
  local) : l'identité par filiation n'est pas une authentification — c'est une
  attribution fiable dans un environnement de confiance, et cette limite est
  documentée.
- Ordre des sessions : la 010 est spécifiée par anticipation ; son
  implémentation suit la 009, sauf redécision explicite de l'utilisateur.
