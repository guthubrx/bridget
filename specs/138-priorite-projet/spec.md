# Feature Specification: Priorité au projet dans les échanges Bridget

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 138-priorite-projet
Titre: Priorité au projet dans les échanges Bridget
Statut: Implemented
Priorité: P1
Tâches: 20/20 (100%)
Tests: 63/63 (100%)

Résumé:
- Contexte: un agent peut découvrir ou solliciter un agent d'un autre projet sans rendre ce changement de contexte explicite.
- Objectif: privilégier les agents du même projet et rendre volontaire chaque échange entre projets.
- Dépendances: SPEC-065, SPEC-084, SPEC-094, SPEC-102, SPEC-115, SPEC-133, SPEC-135, SPEC-136.

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-138-priorite-projet`
**Created**: 2026-10-06
**Status**: Implemented
**Priority**: P1
**Dependencies**: SPEC-065, SPEC-084, SPEC-094, SPEC-102, SPEC-115, SPEC-133, SPEC-135, SPEC-136

## Contexte et objectif

Bridget facilite la collaboration entre agents. Cette collaboration doit partir
du projet de l'émetteur. Un échange extérieur peut apporter du contexte inutile
ou déclencher un travail dans un autre projet. Le choix doit donc être visible.

Les échanges entre projets restent possibles. Une demande explicite suffit à
les autoriser dans son périmètre. Elle ne doit pas entraîner une seconde
confirmation humaine obligatoire. La règle réduit les sollicitations surprises.
Elle ne crée pas une frontière de permission infranchissable.

Le résultat attendu couvre l'annuaire, les suggestions, les messages directs,
les fils et les boucles de coordination. Les agents du même projet sont les
destinataires proposés par défaut. Une vue globale reste disponible sur choix
volontaire. Cette vue n'autorise pas à elle seule un envoi hors projet.

## Scénarios utilisateur et tests

### US1 — Trouver les agents de mon projet (P1)

Comme agent ou coordinateur, je veux voir d'abord les agents de mon projet.
Je peux ainsi recruter un collaborateur sans diffuser mon contexte ailleurs.
Je peux choisir une vue globale si je recherche volontairement un autre projet.

**Test indépendant**: préparer deux projets, deux agents locaux et deux agents
extérieurs. Comparer l'annuaire et les suggestions par défaut avec la vue globale.

**Scénarios d'acceptation**:

1. Étant donné un émetteur lié au projet A, quand il consulte l'annuaire sans
   option de portée, seuls les agents du projet A sont proposés.
2. Quand il choisit la vue globale, les agents de A et de B restent consultables.
   Chaque résultat indique sa portée. Ce choix ne vaut pas autorisation d'envoi.
3. Quand aucun agent local n'est disponible, le résultat explique cette absence.
   Il ne remplace pas les suggestions par des agents extérieurs.
4. Quand le projet de l'émetteur est inconnu, le résultat le signale. Il ne
   présente pas des agents comme appartenant au même projet sans preuve.

### US2 — Choisir un échange entre projets (P1)

Comme émetteur, je veux pouvoir demander volontairement un échange extérieur.
Je veux voir le projet concerné et le motif avant la notification. Une demande
déjà explicite doit aboutir sans nouvelle confirmation humaine bloquante.

**Test indépendant**: adresser le même message au projet B depuis A, d'abord
sans choix interprojets, puis avec ce choix et un motif. Vérifier l'avertissement,
l'absence de notification dans le premier cas et une notification dans le second.

**Scénarios d'acceptation**:

1. Un envoi de A vers B sans mode interprojets volontaire est refusé avant
   notification. Le refus donne la raison et l'action nécessaire.
2. Un choix interprojets avec un motif vide ou composé d'espaces est refusé.
3. Une demande explicite avec un motif non vide produit un avertissement
   observable par l'émetteur avant notification. Elle peut ensuite aboutir sans
   demander une nouvelle validation humaine.
4. Une réponse reste possible dans un échange extérieur explicitement autorisé.
   Elle conserve son mandat et son motif. Elle n'autorise pas un nouveau
   destinataire ou une nouvelle mission hors de ce mandat.
5. Un fil qui contient plusieurs projets ne transforme pas tous ses participants
   en agents locaux. Sa création et tout nouveau dépôt exigent le choix
   interprojets, même sans notification. Tous ses membres peuvent lire le corps.

### US3 — Garder les boucles dans leur mandat (P1)

Comme responsable d'une boucle, je veux que ses recrutements automatiques restent
dans mon projet. Je veux aussi conserver les relances déjà attribuées lorsque
j'ai volontairement défini un mandat entre projets.

**Test indépendant**: exécuter une boucle locale sans agent disponible, puis une
boucle avec un responsable, un coordinateur et un rôle ROOT extérieurs
explicitement mandatés. Vérifier les destinataires de chaque recrutement et relance.

**Scénarios d'acceptation**:

1. Une boucle locale sans candidat disponible signale le besoin ou le blocage.
   Elle ne recrute pas automatiquement un agent d'un autre projet.
2. Un agent local occupé ne provoque pas un remplacement automatique extérieur.
3. Un mandat interprojets explicite conserve les rappels vers le responsable,
   le coordinateur et le rôle ROOT déjà attribués à la mission.
4. Un rôle ROOT extérieur simplement configuré ne constitue pas un mandat.
   Sans autorisation explicite, la boucle rend visible le besoin de décision.

### US4 — Reconnaître le projet sans casser l'historique (P1)

Comme opérateur, je veux que les chemins équivalents d'un dépôt restent dans le
même projet. Je veux aussi préserver les fils et clients historiques sans
transformer une identité incertaine en appartenance prouvée.

**Test indépendant**: comparer un dépôt, son worktree, son lien symbolique et
un autre dépôt portant le même nom. Lire ensuite un fil historique, publier un
historique silencieux et rejouer un envoi autorisé.

**Scénarios d'acceptation**:

1. Une identité attestée commune au dépôt, à son worktree et à son lien
   symbolique les classe dans le même projet.
2. Un domaine d'affichage identique dans deux projets distincts ne les fusionne
   pas. Une identité fournie par le client ne remplace pas l'identité attestée.
3. Une identité inconnue est affichée comme inconnue. Elle n'est pas réputée
   locale par défaut. Un envoi historique reste possible avec un avertissement
   d'incertitude destiné à l'émetteur.
4. La lecture d'un fil existant reste possible selon les accès existants. Elle
   ne rejoue aucune notification et ne change aucune appartenance enregistrée.
5. Un historique silencieux reste silencieux. Le rejeu d'un envoi autorisé ne
   crée pas de nouvelle notification ni de nouvelle autorisation.

## Exigences fonctionnelles

- **FR-13801**: l'annuaire et les suggestions DOIVENT proposer par défaut les
  agents dont l'appartenance au projet de l'émetteur est attestée. Une absence
  de candidat ou un agent occupé NE DOIT PAS déclencher un repli hors projet.
- **FR-13802**: une vue globale volontaire DOIT rester disponible. Elle DOIT
  distinguer même projet, autre projet et identité inconnue. Son activation
  NE DOIT PAS autoriser implicitement un envoi.
- **FR-13803**: l'appartenance DOIT utiliser le rattachement projet attesté du
  transport T3 ou la racine commune Git attestée. Le worktree et le lien
  symbolique d'un même projet DOIVENT conserver cette appartenance. Un domaine
  d'affichage, un nom de dépôt ou une identité déclarée par le client NE DOIT
  PAS suffire à la prouver. Un registre retiré NE DOIT PAS être réactivé.
- **FR-13804**: une identité absente, contradictoire ou non vérifiable DOIT
  rester inconnue. Elle NE DOIT PAS être classée dans le même projet ou proposée
  automatiquement. Un envoi historique avec identité inconnue DOIT rester
  possible avec un avertissement d'incertitude pour l'émetteur.
- **FR-13805**: tout nouvel envoi, création de fil ou dépôt qui rend un contenu
  accessible à un destinataire attesté dans un autre projet DOIT exiger un mode
  interprojets volontaire et un motif non vide. Le refus DOIT précéder tout
  dépôt ou notification et indiquer la correction attendue.
- **FR-13806**: avant toute notification extérieure autorisée ou d'identité
  inconnue, un avertissement DOIT être établi pour l'émetteur. Il DOIT identifier
  les projets connus, l'incertitude éventuelle et le motif interprojets lorsqu'il
  est requis. Il DOIT figurer dans le résultat destiné à l'émetteur, distinct du
  corps envoyé. Une demande explicite valide NE DOIT PAS exiger une confirmation
  humaine bloquante supplémentaire.
- **FR-13807**: l'autorisation interprojets DOIT rester liée au destinataire,
  à l'échange ou à la mission explicitement visés. Les réponses et relances
  dans ce périmètre DOIVENT rester possibles en réutilisant le motif volontaire
  du fil ou du mandat configuré. Un autre destinataire ou une autre mission
  DOIT nécessiter une nouvelle autorisation explicite.
- **FR-13808**: un fil mixte ou un envoi à plusieurs destinataires DOIT appliquer
  la règle à chaque destinataire qui peut lire le contenu. Une liste de
  notification vide NE DOIT PAS être traitée comme un dépôt privé. La présence
  historique d'un agent extérieur NE DOIT PAS créer un mandat de nouveau dépôt.
- **FR-13809**: les boucles et coordinateurs DOIVENT recruter automatiquement
  dans leur projet uniquement. Une configuration générale, une vue globale
  ou un manque de capacité locale NE DOIT PAS créer de mandat extérieur.
- **FR-13810**: un mandat interprojets explicite DOIT préserver les relances
  vers le responsable, le coordinateur et le rôle ROOT déjà attribués. Un rôle
  extérieur sans ce mandat DOIT provoquer une demande de décision visible,
  sans notification extérieure ni réaffectation automatique de mission.
- **FR-13811**: la lecture des fils et historiques existants DOIT conserver les
  accès et contenus actuels. Elle NE DOIT PAS réécrire les identités, rejouer
  des messages ou notifier des participants. Les dépôts silencieux DOIVENT
  conserver leur absence de notification. Un nouveau dépôt silencieux dans un
  fil mixte DOIT respecter l'autorisation interprojets de ce fil.
- **FR-13812**: le rejeu d'une opération DOIT conserver l'idempotence existante.
  Il NE DOIT PAS multiplier les messages, avertissements persistants ou
  notifications. Il NE DOIT PAS élargir l'autorisation initiale.
- **FR-13813**: MCP, CLI, daemon, fils, Agent Loop et règle de la skill Bridget
  DOIVENT appliquer les mêmes distinctions et refus. Un ancien client DOIT
  conserver les lectures et envois locaux compatibles. Il NE DOIT PAS contourner
  le choix explicite requis pour un nouvel envoi extérieur.
- **FR-13814**: la fonction DOIT réutiliser les identités, fils et contrôles
  existants. Elle NE DOIT PAS ajouter de registre concurrent, migrer les
  missions ou redémarrer la production. Les essais DOIVENT utiliser un espace
  Bridget et une socket isolés.

## Entités clés

- **Appartenance projet**: identité attestée d'un agent et de sa source. Les
  chemins et domaines visibles aident au diagnostic sans remplacer cette preuve.
- **Portée d'annuaire**: choix entre le projet de l'émetteur et la vue globale.
- **Mandat interprojets**: choix volontaire, motif et périmètre d'un échange ou
  d'une mission. Il permet les suites prévues sans autorisation générale.
- **Destinataire effectif**: agent qui peut lire un nouveau contenu ou recevoir
  sa notification après résolution du message, du fil ou du rôle de mission.

## Cas limites

- Deux dépôts distincts ont le même nom ou le même domaine d'affichage.
- Un client forge une identité projet ou une identité d'émetteur.
- Le worktree et le lien symbolique pointent vers le même projet attesté.
- Une identité devient inconnue entre découverte et envoi.
- Un fil contient des participants de plusieurs projets ou sans identité connue.
- Un dépôt sans destinataire à notifier reste lisible par les membres du fil.
- Tous les candidats locaux sont occupés ou absents.
- Un rôle ROOT extérieur est configuré sans mandat, puis avec mandat explicite.
- Un ancien client envoie sans les éléments nécessaires au choix interprojets.
- Une vue globale est ouverte, puis un envoi est tenté sans choix interprojets.
- Un ancien historique est lu ou publié silencieusement après activation.
- Un message autorisé est rejoué après reprise ou perte de sa réponse.
- Une réponse suit un échange extérieur explicitement autorisé, puis tente
  d'ajouter un destinataire ou une mission qui ne figure pas dans ce mandat.

## Critères de succès

- **SC-13801**: dans la matrice de deux projets et d'une identité inconnue,
  100 % des résultats par défaut appartiennent au projet attesté de l'émetteur.
  La vue globale volontaire rend les autres agents consultables et identifiés.
- **SC-13802**: 100 % des nouveaux envois entre projets connus sans choix
  volontaire valide sont refusés. Ils produisent zéro notification et aucun
  message livré partiellement à leurs destinataires extérieurs.
- **SC-13803**: 100 % des envois extérieurs explicitement autorisés établissent
  leur avertissement avant notification et le rendent dans le résultat émetteur.
  Ils aboutissent sans confirmation humaine obligatoire supplémentaire, sous
  les accès existants. Les envois historiques d'identité inconnue restent
  possibles et rendent tous l'avertissement d'incertitude, hors corps du message.
- **SC-13804**: les essais d'absence locale, d'agent occupé et de ROOT extérieur
  non mandaté produisent zéro recrutement ou relance extérieure automatique.
  Les trois rôles explicitement mandatés conservent chacun leur relance prévue.
- **SC-13805**: dépôt, worktree et lien symbolique attestés sont reconnus comme
  un même projet. Deux dépôts homonymes et une identité forgée ne sont jamais
  reconnus comme un même projet sur ce seul motif.
- **SC-13806**: la matrice MCP, CLI, daemon, fils et Agent Loop rend les mêmes
  décisions pour les cas de portée. Les lectures historiques causent zéro
  notification. Un historique silencieux et un rejeu gardent zéro notification
  supplémentaire. Aucune mission ni aucun contenu historique n'est modifié.

## Hypothèses et dépendances

SPEC-065 et SPEC-084 apportent les règles d'identité et de distinction des sources.
Leur ancien registre retiré n'est pas une autorité disponible. Le rattachement
attesté du transport T3 et la racine commune Git fournissent l'appartenance de
communication sans nouveau registre. SPEC-115 fournit l'identité des émetteurs.
Une identité locale ne suffit pas à fusionner deux sources distinctes.
SPEC-094 porte la cohérence CLI, MCP et skill.
SPEC-102 fournit les fils. SPEC-133 conserve les liens de relais entre agents.
SPEC-135 fournit les missions et leurs rôles. SPEC-136 conserve la distinction
entre historique silencieux et sollicitation. Leur travail non fusionné doit
être préservé.

Le responsable d'un échange choisit son motif et son mandat. Bridget ne déduit
pas cette autorisation depuis le texte libre, un nom de projet ou l'absence de
candidats. Il ne cherche pas automatiquement un remplaçant dans un autre projet.

## Hors périmètre

- Refonte de l'interface ou du modèle de permissions.
- Interdiction absolue de communiquer entre projets.
- Nouveau registre d'identité ou nouvelle autorité globale.
- Migration, déplacement ou réaffectation des missions existantes.
- Suppression, reclassement ou réécriture des messages historiques.
- Redémarrage de la production ou essais sur sa socket et son espace de données.
- Choix des champs, formats de stockage et mécanismes internes. Ces détails
  appartiennent au plan d'implémentation.
