# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/contracts/communication.md](../../../../../../../specs/138-priorite-projet/contracts/communication.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Nouvelle variante DirectoryScoped avec scope: same_project | global.
Réponse CommunicationDirectory séparée: ScopedAgentInfo avec AgentInfo aplati,
communication_project optionnel et project_relation same/other/unknown.
AgentInfo et ListAgents de diagnostic restent inchangés.
CommunicationProjectFact est une annonce séparée après Register. Seul le
transport propriétaire vivant peut l'écrire ; les auxiliaires en héritent.
Un client CLI de fond négocié peut établir son propre contexte depuis la racine
explicite du run, validée avec son hôte par le daemon. Aucun Register d'agent ni
identité T3 empruntée. Ce contexte n'écrase jamais celui d'un auxiliaire.
Le projet appelant vient de la connexion attestée. La requête ne reçoit pas
un project_id choisi librement comme autorité. Défaut de communication:
same_project. ListAgents reste un diagnostic global compatible.
