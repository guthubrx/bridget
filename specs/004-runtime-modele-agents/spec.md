# Feature Specification: Visibilité du modèle et du niveau d'effort des agents

**Feature Branch**: `session-04-runtime-modele-agents`
**Created**: 2026-08-17
**Status**: Livrée — vérifiée sur agents réels (session 004)
**Input**: User description: "dans bridget who on connaît le type d'agent claude/codex mais pas le modèle utilisé, serait-il possible de faire en sorte qu'on tienne à jour une table des modèles et niveau d'effort et que si je change de modèle ou de niveau d'effort ça se mette à jour ?"

## Contexte

L'annuaire Bridget (`bridget who`) affiche aujourd'hui le nom, le type d'agent
(`claude`, `codex`, `gemini`), l'hôte, l'OS, le transport et l'état de présence.
Le type d'agent ne dit rien de la capacité réelle de l'agent : un agent `claude`
peut tourner sur un modèle rapide et peu coûteux ou sur un modèle de
raisonnement profond, avec un niveau d'effort faible ou élevé. Un humain qui
répartit du travail entre agents, comme un agent qui choisit un destinataire,
prennent donc une décision d'aiguillage sans l'information la plus déterminante.

Ces deux attributs changent en cours de session, à l'initiative de l'humain.
Une valeur capturée une seule fois au lancement serait donc fausse la plupart du
temps : l'annuaire doit refléter la valeur courante.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Voir le modèle et l'effort de chaque agent (Priority: P1)

En tant qu'humain pilotant plusieurs agents CLI, j'ouvre l'annuaire Bridget et
je vois, pour chaque agent connecté, quel modèle il utilise et à quel niveau
d'effort, en plus des informations déjà présentes.

**Why this priority**: C'est la demande elle-même. Sans cet affichage, aucune
autre partie de la fonctionnalité n'a de valeur observable.

**Independent Test**: Lancer un agent, exécuter la commande d'annuaire, et
constater que le modèle et l'effort de cet agent y figurent. Vérifiable seul,
sans aucun changement de modèle.

**Acceptance Scenarios**:

1. **Given** un agent connecté dont le modèle courant est connu du système,
   **When** l'utilisateur consulte l'annuaire,
   **Then** la ligne de cet agent affiche son modèle et son niveau d'effort.
2. **Given** un agent connecté dont le modèle n'a pas encore pu être déterminé,
   **When** l'utilisateur consulte l'annuaire,
   **Then** la ligne affiche une valeur d'inconnu explicite, et l'annuaire reste
   lisible et aligné.
3. **Given** plusieurs agents de types différents,
   **When** l'utilisateur consulte l'annuaire au format machine,
   **Then** le modèle et l'effort sont présents comme champs distincts pour
   chaque agent.

---

### User Story 2 - Suivre un changement de modèle en cours de session (Priority: P1)

En tant qu'humain, je change le modèle ou le niveau d'effort d'un agent pendant
qu'il travaille. L'annuaire reflète ce changement sans que j'aie à relancer
l'agent ni à saisir quoi que ce soit.

**Why this priority**: C'est la condition de confiance. Un annuaire qui affiche
une valeur périmée est plus nuisible qu'un annuaire vide, parce qu'il conduit à
aiguiller du travail vers un agent qui n'a plus la capacité supposée.

**Independent Test**: Changer le modèle d'un agent en session, puis consulter
l'annuaire après le délai de rafraîchissement annoncé et constater la nouvelle
valeur.

**Acceptance Scenarios**:

1. **Given** un agent affiché avec le modèle A,
   **When** l'humain bascule cet agent sur le modèle B et que l'agent produit un
   tour de travail,
   **Then** l'annuaire affiche le modèle B au plus tard 60 secondes après ce tour.
2. **Given** un agent affiché avec un niveau d'effort donné,
   **When** l'humain change uniquement le niveau d'effort,
   **Then** l'annuaire affiche le nouveau niveau, le modèle restant inchangé.
3. **Given** un agent dont ni le modèle ni l'effort n'ont changé,
   **When** le temps passe,
   **Then** aucune mise à jour n'est émise vers le daemon.

---

### User Story 3 - Déclarer manuellement le modèle d'un agent non instrumenté (Priority: P2)

En tant qu'agent d'un type non couvert par une détection automatique, ou en tant
qu'humain diagnostiquant un cas particulier, je peux déclarer explicitement le
modèle et l'effort de l'agent courant.

**Why this priority**: Garantit que la fonctionnalité dégrade proprement au lieu
de laisser un trou permanent pour les types d'agents non instrumentés
(`gemini`, agents personnalisés). Utile aussi comme point de vérification lors
du diagnostic.

**Independent Test**: Depuis un agent connecté, déclarer un modèle arbitraire,
puis constater la valeur dans l'annuaire.

**Acceptance Scenarios**:

1. **Given** un agent connecté au daemon,
   **When** il déclare un modèle et un effort,
   **Then** l'annuaire affiche ces valeurs pour cet agent.
2. **Given** une déclaration émise hors de tout agent Bridget,
   **When** la commande est exécutée,
   **Then** elle est refusée avec un message explicite, sans modifier l'annuaire.
3. **Given** un agent qui déclare un modèle,
   **When** une détection automatique produit ensuite une valeur différente,
   **Then** c'est la valeur observée la plus récente qui est affichée.

---

### Edge Cases

- **Agent injoignable** : un agent devenu `unreachable` conserve le dernier
  modèle connu, affiché tel quel, plutôt que de retomber à inconnu. L'annuaire
  distingue déjà l'état de présence ; le modèle décrit la dernière capacité
  observée.
- **Redémarrage du daemon** : les valeurs sont perdues et reconstruites au
  premier signal de chaque agent. L'annuaire affiche l'inconnu explicite dans
  l'intervalle, sans erreur.
- **Agent démarré mais sans aucun tour de travail** : le modèle peut rester
  inconnu tant que l'agent n'a rien produit. C'est un état légitime, pas une
  panne.
- **Nom de modèle long** : l'annuaire reste aligné et lisible ; les colonnes
  s'adaptent à la valeur la plus longue, comme les colonnes existantes.
- **Renommage d'un agent** : le modèle suit l'agent renommé, sans perte.
- **Agent fédéré via SSH** : la détection s'effectue là où tourne l'agent ;
  l'annuaire du daemon maître affiche la valeur remontée.
- **Valeur d'effort absente** : certains agents n'exposent pas de niveau
  d'effort. Le modèle seul est affiché, l'effort porte l'inconnu explicite.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: L'annuaire des agents MUST exposer, pour chaque agent, un modèle
  courant et un niveau d'effort courant, en plus des attributs existants.
- **FR-002**: L'affichage humain de l'annuaire MUST présenter ces deux attributs
  dans des colonnes dédiées, alignées avec les colonnes existantes.
- **FR-003**: La sortie machine de l'annuaire MUST exposer ces deux attributs
  comme champs distincts.
- **FR-004**: Le système MUST afficher une valeur d'inconnu explicite quand un
  attribut n'a pas encore été observé, et ne jamais afficher une valeur
  inventée ou déduite d'un défaut de configuration.
- **FR-005**: Le système MUST mettre à jour le modèle et l'effort d'un agent
  sans relance de cet agent, après un changement décidé par l'humain en cours de
  session.
- **FR-006**: Le délai entre un changement effectif et sa visibilité dans
  l'annuaire MUST être au plus de 60 secondes d'activité de l'agent.
- **FR-007**: Le système MUST n'émettre une mise à jour que lorsque la valeur
  observée diffère de la dernière valeur transmise.
- **FR-008**: Un agent connecté MUST pouvoir déclarer explicitement son modèle
  et son effort ; cette déclaration suit la même règle de récence que les
  valeurs détectées.
- **FR-009**: Une déclaration émise hors du contexte d'un agent Bridget MUST
  être refusée avec un message explicite.
- **FR-010**: Le système MUST conserver la dernière valeur connue d'un agent
  passé à l'état injoignable, pendant toute la durée de rétention de sa présence.
- **FR-011**: L'absence de source de détection pour un type d'agent MUST laisser
  la fonctionnalité opérationnelle pour les autres types, sans erreur ni
  dégradation de l'annuaire.
- **FR-012**: L'activation de la détection automatique MUST être une action
  explicite et réversible de l'utilisateur lorsqu'elle modifie une configuration
  hors du périmètre de Bridget, et MUST sauvegarder l'état antérieur.
- **FR-013**: Le surcoût de la détection MUST rester imperceptible pour l'agent
  observé : aucune interruption de son travail, aucune écriture dans ses
  fichiers de session.

### Key Entities

- **Runtime d'agent** : capacité courante d'un agent connecté, composée d'un
  identifiant de modèle et d'un niveau d'effort, chacun pouvant être inconnu.
  Rattaché à un agent de l'annuaire, il vit aussi longtemps que la présence de
  cet agent.
- **Source d'observation** : origine de la dernière valeur connue — observation
  automatique ou déclaration explicite. Permet de comprendre pourquoi une valeur
  est absente ou périmée lors d'un diagnostic.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Pour 100 % des agents `claude` et `codex` connectés et ayant
  produit au moins un tour de travail, l'annuaire affiche un modèle non inconnu.
- **SC-002**: Un changement de modèle décidé par l'humain est visible dans
  l'annuaire en moins de 60 secondes d'activité de l'agent, mesuré sur trois
  essais consécutifs.
- **SC-003**: Un agent inactif dont rien n'a changé ne génère aucune mise à jour
  vers le daemon, vérifié sur une fenêtre de 5 minutes.
- **SC-004**: L'annuaire reste lisible et aligné avec un modèle de 40 caractères
  et avec des agents dont le modèle est inconnu, dans la même sortie.
- **SC-005**: L'ensemble des tests automatisés du projet reste vert, et la
  fonctionnalité ajoute au moins un test par source d'observation.
- **SC-006**: Aucune régression de comportement sur les commandes d'annuaire
  existantes : les colonnes et champs actuels conservent leur nom et leur
  sémantique.

## Assumptions

- Les changements de modèle et d'effort sont décidés par l'humain dans
  l'interface de l'agent, jamais par Bridget. Bridget observe, il ne pilote pas.
- La valeur observée est celle réellement utilisée par l'agent lors de son
  dernier tour de travail, et non un réglage de configuration qui pourrait ne
  pas être appliqué à la session en cours.
- Deux types d'agents sont instrumentés dans ce périmètre : `claude` et `codex`.
  `gemini` et les agents personnalisés relèvent de la déclaration explicite
  (User Story 3).
- L'historique des changements de modèle est hors périmètre : seule la valeur
  courante est conservée, comme pour l'hôte, l'OS et le transport aujourd'hui.
  Arbitrage utilisateur du 2026-08-17.
- La fonctionnalité s'appuie sur l'annuaire, la présence et le mécanisme
  d'identification de l'agent courant déjà livrés par les sessions 002 et 003.

## Dépendances avec les specs existantes

- **`001-renommer-agent`** : le renommage doit préserver le runtime de l'agent
  (Edge case « renommage »). Le mécanisme d'identification de l'agent courant
  introduit par cette spec est réutilisé par la déclaration explicite (FR-008).
- **`002-federation-ssh`** : la détection tourne sur l'hôte de l'agent ; les
  agents fédérés doivent remonter leur runtime comme ils remontent déjà hôte,
  OS et transport.
- **`003-cycle-vie-demandes`** : la rétention de présence introduite par cette
  spec porte la conservation de la dernière valeur connue (FR-010).
