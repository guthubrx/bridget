# Recherche148

Sources primaires : code local T3 commit1535fc0844, Bridget6e167cb0 et données
runtime relues le10octobre. Baseline utilisateur agentic AI consultée : architecture
orientée responsabilité, coût de contexte et réutilisation avant framework.

Décision : utiliser le credential MCP T3 par session. Justification : sa portée
contient environnement/fil/fournisseur et sa révocation est déjà centralisée.
Alternative rejetée : choisir le premier rollout d'un PID partagé, qui usurperait
une identité. Alternative rejetée : nouveau serveur de jetons Bridget concurrent.

Décision corrigée par instruction utilisateur : moteur Bridget entièrement
autonome. Réutiliser sa flotte gérée, son registre, ses remises et exécutions.
Adapter les principes T3 (appel unique, tâche parente, résultat, annulation),
jamais son moteur. Le connecteur T3 peut évoluer séparément pour l'identité.
Un simple alias de delegate_task est explicitement exclu.

Les hooks/templates/scripts SpecKit ne sont pas présents dans le dépôt/worktree.
La couche utilisateur a été synchronisée ; les artefacts sont rédigés manuellement
selon les skills canoniques. Aucun fichier officiel SpecKit n'est modifié.
