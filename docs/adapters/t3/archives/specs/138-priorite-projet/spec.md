# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/spec.md](../../../../../../specs/138-priorite-projet/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

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
