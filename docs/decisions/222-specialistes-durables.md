# Décision222 — spécialistes durables
Statut : implémentation en cours, activation non attestée.

Profil, identité et processus sont distincts. AgentProfileStore porte les
versions, rôles et connaissances. ExecutionStore conserve la seule autorité de
soumission et d'exécution. L'attribution222 et prepared sont réservés ensemble.
Le réveil relit le grant et utilise persistent=false ; RELEASE relit les droits
sous verrou SQL. Aucun redémarrage général ne crée une nouvelle mission.

Le contact humain avec réponse réutilise tracked_requests et le journal, sans
faux agent humain. Le reçu de remise fait autorité, pas le mot accepted de la
soumission. Un plan d'équipe fige seulement canon, version et clés ; il ne copie
aucun état de mission. Les budgets sont facultatifs et fondés sur les faits
natifs observés, sans promesse d'arrêt instantané d'un fournisseur.

Le rappel lexical ne prétend pas fournir une recherche vectorielle. Les sources
et tombstones restent durables ; le contexte est ciblé sans purge de corpus.
MCP/CLI/Android partagent les mêmes données et décisions humaines authentifiées.
Aucun client web, service d'orchestration ou magasin de tâches concurrent.


## Frontière du contrôle humain local
Le canal de contrôle local reprend le modèle administratif natif : UID de
l'installation, connexion Client nue, capacité et acteur de contrôle connus.
La CLI exige un vrai terminal et une confirmation de la commande exacte.
Le hash de requête est une corrélation de commande, pas une signature humaine.
Le produit ne prétend pas résister à un administrateur OS hostile de même UID,
qui peut déjà écrire la base ou remplacer les exécutables. Les agents doivent
utiliser leur port attesté et ne peuvent pas envoyer une approbation dans le JSON.
Mobile dispose de sa propre authentification d'appareil ; user_input conserve
une réponse humaine corrélée. Aucun de ces canaux ne donne de privilège Unix neuf.

Cette borne reprend explicitement l'ADR027 (« Borne de la garde ») et l'ADR011,
pas un nouveau choix de sécurité222. Les profils ne créent aucun type exécutable
dans le registre. La portée d'une décision par question vient de son acteur :
contrôle local natif, ou appareil actif doté de SpecialistsManage. La portée de
l'agent qui pose la question ne devient pas une autorité humaine.
