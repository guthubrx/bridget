# SPEC 032 — GUI : poste de travail conversationnel

**Statut** : proposée · **Bloc** : B (Piste GUI) · **Créée** : 2026-08-25 21h00

## 1. Le besoin, tel qu'il a été exprimé

> « Être obligé de jongler quand je te parle avec tous les autres agents qui
> t'envoient des trucs, j'en peux plus. »

**Fait mesuré le 2026-08-25** : les messages d'agents s'intercalent dans le fil
de conversation **et partent avec la phrase de l'utilisateur**. Il valide malgré
lui. Sur cette seule journée, plus de trente rapports d'agents ont traversé la
conversation, dont plusieurs de trente lignes.

**Cause structurelle** : le référent est l'**unique point de passage**. Tout ce
qu'un agent produit remonte par le même canal que la conversation humaine.

## 2. Propriété visée

> **La zone de saisie appartient à l'utilisateur.** Les messages d'agents
> arrivent dans le fil ; **rien ne part tant qu'il n'appuie pas**.

C'est la propriété fondatrice. Toutes les autres sont subordonnées.

## 3. Ce qui existe déjà — mesuré

| Brique | État | Mesure |
|---|---|---|
| Flux temps réel | **livré** | `/v1/watch` (SSE, abonnement avant snapshot) |
| Agrégateur | **livré** | `/v1/snapshot` — annuaire + ledger + projection Maicie |
| Journal | **livré** | `/v1/journal` |
| Relais UI | **livré** | `crates/bridget-daemon/src/ui.rs`, 780 lignes |
| Page locale | **livré** | 3 zones, tunnel **lecture seule** |

**Ce qui manque : la liste d'agents, le fil en bulles, et l'envoi.**
Le relais est en lecture seule (39 occurrences de la contrainte).

## 4. Périmètre de cette tranche

### 4.1 Liste des agents (colonne gauche)

- Une ligne par agent : **nom · heure du dernier message · extrait · pastille
  de non-lu**.
- Ordre : dernier message en premier.
- Agents éteints en section **masqués**, avec compteur.
- Sections optionnelles pour grouper par machine.

### 4.2 Fil de conversation (panneau droit)

- **Un agent à la fois.**
- Bulles : agent à gauche, utilisateur à droite.
- Séparateurs de date centrés.
- **Événements système en gris, centrés, dans le fil** : mission déléguée,
  verdict rendu, refus de contrainte, agent arrêté.

### 4.3 Trace inter-agents — mécanisme central

Entre deux bulles, une **ligne centrée en gris avec l'avatar de l'autre agent**.
Trois formes, qui distinguent **la direction** :

| forme | sens |
|---|---|
| `Message de <agent>` | reçu |
| `<N> messages avec <agent>` | échange, **avec le nombre** |
| `Message à <agent>` | envoyé |

**Ni le contenu, ni rien** : qui, dans quel sens, combien.
Le marqueur apparaît **à sa place chronologique** — donc on voit qu'un agent a
consulté quelqu'un **avant** de répondre.

**Interaction — deux gestes distincts sur la même ligne :**

- **clic sur la ligne** → déplie sur place si `N ≤ SEUIL_DEPLI`, sinon ouvre le
  panneau latéral. **Geste unique, décision mécanique sur la taille.**
- **clic sur l'avatar ou le nom** → va à la conversation de cet agent,
  positionnée au bon endroit de son fil.

`SEUIL_DEPLI = 3`, **paramétrable**. Valeur provisoire, à recalibrer sur mesure
réelle.

### 4.4 Trois niveaux de détail

Hiérarchisés **par nature, pas par volume** :

| niveau | contenu | état |
|---|---|---|
| **0** | **le résultat** seul, précédé de la durée (`a travaillé 47 min`) | replié par défaut |
| **1** | **les actes** — journal d'exécution | dépliable |
| **2** | **la délibération** — raisonnement de l'agent | imbriqué dans le 1 |

**Niveau 1 — deux typographies entrelacées dans le même flux :**
- **blanc** : l'intention annoncée, en phrases courtes ;
- **gris** : l'acte (`cargo test -p maicie → 313 passés`, `2 messages avec rc1`).

**La trace inter-agents appartient au niveau 1** : c'est un acte, au même titre
qu'un test lancé.

**Sources disponibles — relevées le 2026-08-25 dans une implémentation de
référence** (`~/11.Repositories/t3code`, paquets `effect-acp` et
`effect-codex-app-server`). **Toutes traversent déjà nos connexions.**

**Codex** — `item/*` :

| événement | alimente |
|---|---|
| `item/reasoning/summaryTextDelta` · `summaryPartAdded` · `textDelta` | **niveau 2** |
| `item/agentMessage/delta` | le message — **seul lu aujourd'hui** |
| `item/commandExecution/outputDelta` | **niveau 1** — sortie de commande |
| `item/fileChange/patchUpdated` · `outputDelta` | **niveau 1** — fichiers touchés |
| `item/plan/delta` | **niveau 1** — plan annoncé |
| `item/mcpToolCall/progress` | **niveau 1** — outils |
| `item/*/requestApproval` | **demandes d'approbation** — voir §4.7 |
| `item/started` · `item/completed` | bornes des actes |

**Cursor (ACP)** — variantes de `SessionUpdate` :

| variante | alimente |
|---|---|
| `agent_thought_chunk` | **niveau 2** — écarté par `acp.rs:1500` |
| `agent_message_chunk` | le message — **seul lu aujourd'hui** |
| `tool_call` · `tool_call_update` | **niveau 1** |
| `usage_update` | **la jauge de contexte** (§7) |
| `current_mode_update` · `session_info_update` | état |

**Claude** — blocs `thinking` / `thinking_delta` : **défaut amont connu**, non
émis. Rien à brancher aujourd'hui.

> **Conséquence** : le niveau 1 n'est pas à inventer. Les actes sont déjà
> nommés par les protocoles — commande exécutée, fichier modifié, outil appelé,
> plan annoncé.

> **Règle imposée par les faits : traiter la pensée comme OPTIONNELLE.**
> Quand elle n'arrive pas, afficher **« raisonnement non fourni »** — jamais un
> vide qu'on lirait comme « il n'a pas réfléchi ».

Cas documenté : chez Gemini, le chunk n'est **jamais** émis, le modèle ne
marquant pas ses parties comme pensée en flux.

### 4.5 Zone de saisie

**Une seule zone**, arrondie, multi-lignes, **tous les boutons à l'intérieur,
alignés en bas** :

- **à gauche** — ce qui qualifie l'envoi : **machine** (locale / distante, pour
  une mission ponctuelle) et **attente de réponse** (`reply`) ;
- **à droite** — indicateur d'activité et bouton d'envoi rond.

**Sous la zone**, une ligne de contexte discrète : dépôt, branche.
**Au-dessus**, un bandeau d'état du fil quand l'agent est éteint :
*« cet agent est arrêté ; envoyer un message le relance »*.

> **Ce qui n'est PAS dans la zone de saisie, et pourquoi :**
> le **modèle**, l'**effort** et les **permissions** sont scellés dans la
> définition figée de l'agent, approuvée par **frappe humaine** (ADR 011).
> Une liste déroulante contournerait exactement ce qui les protège.

### 4.6 Panneau latéral

Pour ce qui ne tient pas dans une bulle : rapport de relecture, registre du dû,
tableau d'avancement, échange inter-agents au-delà du seuil.

### 4.7 Interaction — écrire et être répondu

**C'est le cœur de la tranche.** Le relais est aujourd'hui en lecture seule :
on voit tout, on n'envoie rien.

**Ce qui doit exister :**

- **Envoyer** un message à l'agent du fil courant, avec ou sans attente de
  réponse (`reply`).
- **Sa réponse apparaît dans le fil**, poussée par le flux temps réel — sans
  rechargement, sans que l'utilisateur redemande.
- **Accuser** une demande suivie depuis l'interface.

**Ce qui reste interdit depuis l'interface**, et ce n'est pas négociable :

| interdit | raison |
|---|---|
| approuver un profil ou une routine | **frappe humaine obligatoire** (ADR 011) — jamais exposé à un canal programmatique |
| changer modèle, effort, permissions | scellés dans la définition figée de l'agent |
| changer la branche d'un agent | ferait **expirer un verdict** en cours |

Les **demandes d'approbation** émises par les agents
(`item/*/requestApproval`) sont **affichées** — l'utilisateur voit qu'un agent
attend — mais la validation reste hors interface.

### 4.8 Comportements critiques

Trois comportements sans lesquels l'interface reproduirait le problème qu'elle
corrige.

**Le défilement ne saute jamais.** Si l'utilisateur lit plus haut et qu'un
message arrive, **la position ne bouge pas**. Un bouton « nouveaux messages »
apparaît ; **c'est lui qui descend, pas l'interface**.
*Motif : être déplacé sans l'avoir demandé est exactement le défaut d'origine.*

**La coupure du serveur est visible et réparée seule.** Le daemon a été
redémarré **deux fois le 25/08**. À la coupure du flux : reconnexion
automatique, **et un bandeau qui le dit**. Sans ce bandeau, l'utilisateur
regarde une interface muette en croyant que personne ne parle.

**Le lancement est une commande unique.** `bridget ui`, qui ouvre l'adresse
locale. Aucune étape de compilation, aucune installation.

## 5. Charte graphique — règles applicables

Écrites en règles pour ne pas dépendre du goût de l'implémenteur.

- **Fond noir, pas gris** — proche de `#0a0a0a`.
- **Aucune bordure.** Les zones se distinguent par une variation de fond de 2 à
  3 %, jamais par un trait.
- **Deux niveaux de texte seulement** : blanc cassé pour ce qui compte, gris
  moyen pour tout le reste — activité, métadonnées, horaires. **C'est ce gris
  qui permet de survoler sans lire.**
- **Un seul accent coloré**, bleu, **réservé** aux liens et au bouton d'envoi.
  Pas de vert « succès », pas de rouge « erreur » : **l'état passe par le
  texte**.
- **Coins très arrondis**, typographie système large, **interligne généreux**.
  C'est l'air vertical qui distingue l'interface d'un terminal.
- **Aucune ombre, aucun dégradé, aucune icône pleine.** Icônes fines,
  monochromes, discrètes.
- **Code en ligne** : fond légèrement teinté, couleur chaude (saumon). Seul
  écart chromatique toléré.
- **Les actions n'apparaissent qu'au survol** — réagir, répondre, options.
  C'est ce qui garde le fil propre.

## 6. Hors périmètre de cette tranche

| Écarté | Raison |
|---|---|
| Routage par description, conversations de groupe | **existent déjà côté machine** — deux agents se sont coordonnés seuls le 25/08 ; les refaire ici dupliquerait |
| Modes (Plan / Debug / Ask) | notre équivalent est le **régime de revue**, en cours de mécanisation — deux endroits pour la même décision |
| Choix du modèle | **règle de sécurité** — voir §4.5 |
| Serveurs d'outils (MCP) | plomberie, pas décision d'usage |
| Sélection de branche | **dangereux** : nos agents gèlent leur tête pendant une relecture ; un changement ferait **expirer un verdict** (constaté le 25/08) |
| Vue par projet, conversations classées | nos branches et sessions le font déjà |

## 7. Reporté aux tranches suivantes

- **Jauge de contexte par agent** — trois agents sont morts d'épuisement le
  25/08 **sans qu'aucun signal ne l'annonce**.
- **Alerte « demande lente »** avec heure de début — un agent a répondu
  « imminent » sans avoir commencé ; un autre paraissait mort alors qu'il
  compilait.
- **Coût par agent** — mesuré le 25/08 : **89 entrées de coût, 89 « inconnu »**.
  Aucune visibilité sur la consommation.
- Joindre un fichier · sections · masqués · branche affichée · actions
  contextuelles · fiche de réglages par agent.

## 8. Critères d'acceptation

- [ ] **AC1** — un message d'agent arrive pendant la saisie : le texte en cours
      **n'est pas modifié** et **rien n'est envoyé**.
- [ ] **AC2** — la liste affiche un agent par ligne, triée par dernier message,
      avec pastille de non-lu qui s'éteint à la lecture.
- [ ] **AC3** — un échange entre deux agents produit **une ligne de trace** à sa
      place chronologique, avec la bonne **direction** et le bon **nombre**.
- [ ] **AC4** — clic sur une trace de `N ≤ 3` : dépli sur place. `N > 3` :
      panneau latéral. **Même geste.**
- [ ] **AC5** — clic sur l'avatar : bascule vers la conversation de cet agent.
- [ ] **AC6** — un message envoyé depuis la zone atteint l'agent et **sa réponse
      apparaît dans le fil**.
- [ ] **AC7** — niveau 0 par défaut ; le dépli révèle les actes ; le raisonnement
      est **imbriqué** dans les actes.
- [ ] **AC8** — un agent sans raisonnement disponible affiche **« raisonnement
      non fourni »**, jamais un vide.
- [ ] **AC9** — contrôle positif de la charte : **aucune bordure**, **un seul
      accent coloré**, **deux niveaux de gris de texte**.
- [ ] **AC10** — un message arrive pendant que l'utilisateur lit plus haut :
      **la position ne bouge pas** ; un bouton « nouveaux messages » apparaît.
- [ ] **AC11** — le daemon redémarre : le flux se **reconnecte seul** et un
      bandeau signale la coupure puis le rétablissement.
- [ ] **AC12** — `bridget ui` suffit à ouvrir l'interface. Aucune compilation,
      aucune installation.
- [ ] **AC13** — aucune approbation de profil ou de routine n'est possible
      depuis l'interface ; une demande d'approbation est **affichée** sans être
      **validable**.

## 8ter. Référence de lecture

`~/11.Repositories/t3code` — monorepo TypeScript, paquets **`effect-acp`** et
**`effect-codex-app-server`** : implémentation de référence des deux protocoles,
avec la liste exhaustive des événements (§4.4).

> **On lit, on ne copie pas.** La **forme des protocoles** est publique et
> documentée ; s'en inspirer ne pose aucun problème. **Reprendre du code en
> poserait un** — la licence du dépôt n'est pas vérifiée.

**Leçon de méthode, contre le référent** : il avait cherché sur le web et
mesuré notre propre code **sans jamais regarder les dépôts voisins**. Avant
d'instruire un protocole tiers par la mesure, **vérifier s'il existe une
implémentation de référence sur la machine**.

## 8bis. Architecture — où vit l'interface

**Écarté : Maicie.** Coordinatrice déterministe sans LLM ; la règle posée le
25/08 la protège (*elle lit, elle ne produit pas*), et elle n'a ni l'annuaire
ni le flux temps réel.

**Le juste : un processus séparé.** Argument mesuré — le daemon a été
**redémarré deux fois le 25/08** et a tourné **douze heures avec 41 versions de
retard**. L'interface serait tombée deux fois et figée douze heures.

**Retenu pour la première nuit, assumé et borné : le relais reste dans le
daemon**, parce qu'il existe (`ui.rs`, 780 lignes) avec ses trois routes qui
fonctionnent. Repartir dans un processus séparé cette nuit reviendrait à
**refaire le socle avant de commencer l'interface**.

> **Frontière qui rend l'extraction gratuite plus tard — une seule règle :
> LE RELAIS NE CALCULE RIEN.**
> Il sert les fichiers et transmet des **faits** (« message de `rc1`, 20h12,
> texte »). **Toute** la mise en forme est dans la page.

**Pourquoi cette règle, concrètement** : si le relais fabriquait l'affichage,
changer une couleur exigerait de **recompiler et redémarrer le daemon** — donc
**couper tous les agents**. Avec la séparation, on recharge la page.

**Socle du front pour cette nuit : HTML/JS pur, embarqué.** Aucune chaîne de
compilation, aucune dépendance nouvelle. État mesuré du front actuel :
**153 lignes écrites en dur dans le Rust**, servies par un serveur HTTP
artisanal ; aucun fichier web dans le dépôt.

## 9. Dépendances

- `/v1/watch`, `/v1/snapshot`, `/v1/journal` — **livrés**.
- **Le relais UI doit cesser d'être en lecture seule** pour AC6. C'est le seul
  changement de fond côté daemon.
- Branches d'aiguillage à ajouter pour le niveau 2 : `codex_app_server.rs`
  (événements `item/reasoning/*`) et `acp.rs:1500` (`agent_thought_chunk`).
  **Aucun changement de protocole** — les données traversent déjà nos
  connexions.
