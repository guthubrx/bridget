# 241 — Autorité de la mission acceptée et fichiers source

Décision du 7 novembre 2026, dans la refonte de fiabilité autorisée par François.
Objectif : faire aboutir le travail confié sans dépendre du maintien du tour qui
l’a demandé. T3 reste ouvert ; aucun rejeu de mission métier pour qualification.

Une admission fraîche vérifie le port, l’identité et les droits du demandeur.
Le champ interne optionnel Task.admission_authority_version=1 fige ensuite
l’autorité de cette seule mission. Aucun credential HTTP n’est stocké. La fin
d’un tour et le changement des futurs tours ne révoquent pas rétroactivement
cette mission. Une nouvelle demande ou une continuation exige une preuve fraîche.
Les tâches historiques sans marqueur gardent leur contrat ; elles ne sont pas
promues lors d’une lecture ou d’une reprise. Une version inconnue est conservée
et interdit l’exécution. L’annulation et les révocations SQLite du propriétaire,
de son instance initiale/courante et de la racine restent effectives. Définition,
placement, politique, sources de permissions et contrôle natif d’arrêt/pause sont
relus avant Release. La machine source vient du placement attesté ; la présence
du parent ne remplace pas cette preuve et son absence ne la supprime pas.

Le contrat aligne les missions provider_turn sur l’autorité de mission déjà
native pour les wrappers, avec son marqueur explicite et les mêmes refus de
révocation. Il ne transforme pas une ancienne attestation en pouvoir d’admettre
une autre tâche. Les tests couvrent le tour fini, le suivant, chaque révocation,
l’annulation, la réouverture et le parcours réel de sockets jusqu’à Release.

Les fichiers source suivent les droits déjà attestés. Les racines supplémentaires
explicites sont reconnues. Le cas Codex dangerFullAccess et le cas natif Claude
exact déjà qualifié sans règles privées couvrent aussi un dossier extérieur au
cwd initial. Le seul nom full-access ne suffit pas. Ce choix ne donne aucun droit
supplémentaire au parent ou à l’enfant, ni au rôle de maintenance sur un autre
projet. Le mandat humain métier reste celui de l’appelant. Les protections
physiques du worker, les scopes de communication et l’identité d’un project_id
explicitement imposé restent contrôlés. Un dossier local hors Git peut être son
propre placement sans copie, miroir ni dépôt fictif. Le registre reçoit seulement
un nouveau mapping natif quand nécessaire ; aucun mapping existant n’est réécrit.

Le retour opérationnel doit lire et appliquer ces contrats, pas seulement accepter
leur JSON. storage_compatibility ajoute delegation.admission_authority.v1,
workspace.commit_view.resource_bound.v3 et worker.workload_tools.v1. Les anciennes
images 239/240 ne sont donc pas annoncées comme retours de 241. Construire et
qualifier un retour compatible avant activation ; aucun retour réel pour tester.

## Revue indépendante R1 — précisions

Le répertoire supplémentaire local est déjà couvert par l'autorité native du
parent. Le mode observé est vérifié comme stable pendant la projection ; il ne
prétend pas valider la politique d'un miroir distant. Aucun mode privé nouveau
n'est exigé sur un dossier de travail déjà autorisé. Les protections de stores,
l'UID, les ancêtres, l'absence de liens et la canonicalisation restent vérifiés.

Au démarrage d'une tâche admise, le snapshot de cette tâche remplace un fait de
tour ultérieur fourni par l'appelant. Les comparaisons suivantes garantissent
que cette autorité n'a pas changé pendant les échanges ; elles ne revalident
pas l'existence de l'ancien tour. Les sources et révocations sont relues.

Les identifiants storage_compatibility désignent aussi des contrats de lecture
et d'action, pas seulement des clés JSON nouvelles. Task produit effectivement
admission_authority_version ; le manifeste de vue conserve son schéma après
retrait du quota, et la sonde outils possède une capacité négociée. Un ancien
lecteur qui ignore le marqueur ne constitue pas un retour compatible. Les
images239/240 sont exclues comme retour de241. Le pointeur doit viser une image
qui traite ces contrats et dont les contre-épreuves ont été vérifiées.
