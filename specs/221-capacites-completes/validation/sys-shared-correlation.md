# Corrélation Shared SYS — 1 novembre2026, 09:46–09:47UTC

Demande : corréler deux refus workspace_busy sans libérer de garde, interrompre
autrui, rejouer une demande ni changer ses axes. Investigation en lecture seule,
code actif221-sender-guide-r2 ; aucun défaut d’exécution nouveau établi.

## Faits corrélés

1. opus201-photos-reports10-20261101-01 : reçu maître chosen, candidatec001,
   task_id null ; créé/mis à jour1793526369. Même owner monuments.
   Observation initiale du candidat1793526355 ; lot reçu1793526369.
   Recontrôle refusé à09:46:32UTC :
   admissiona-680c6addb0170d6b0083c82cb3f454a2,
   corrélation5b0b93a4-cd5b-4b46-ae18-3371960390f7, workspace_busy.
   Aucun task_id ni commande durable du projet corrélée à l’intervalle
   1793526340–1793526392 dans les reçus courants du worker.
   Le bloqueur exact de ce premier refus reste non attesté.

2. opus201-prochaine-vague48-20261101-01 : aucune ligne de reçu auto trouvée
   dans le registre courant, aucun task_id. Refus des12candidats SYS à
   09:47:57UTC, admissiona-f0a30443ceade71eafe56382b6867c78,
   corrélationc217e183-cd70-415e-90ab-107dbbba7b3f.

3. Esquirol5a642a40-2f99-407c-863f-65c5c034f257 : tâche maître créée1793526437,
   donc APRES le premier refus. Commande worker créée1793526465, actualisée
   Released/native_ready1793526469 ; démarrage métier maître1793526498.
   À1793526596 : unique commande non terminale du store worker dont le dossier
   recoupe la racine Opus2D ; cwd dans .worktrees/5a642a40.
   Marqueur PID1318184/naissance69628328, naissance OS identique.
   Ce marqueur et cette phase corrèlent le SECOND refus à un travail vivant.
   Ils ne permettent pas d’attribuer le premier refus à cette tâche.

## Pourquoi Shared est concerné

Au relevé1793526596, racine SYS :
/mnt/storage/bridget-projects/opus2d/repository
HEAD2f1ae8a831d50658f5bc5885f46c007a97e8452d, Git propre.
HEAD est ancêtre strict de la base675d10d20b91dcf0b5d307847aadeeedccae17d2.
Shared doit donc déplacer HEAD avant la remise du travail demandé. L’exclusion
existante vérifie les chemins recoupés, y compris un worktree descendant.
Le lieu des sorties métier hors Git ne change pas ce contrat de préparation.

Le code shared_refresh_allowed et ProbeReadContext joignent les préparations
en mémoire aux commandes durables non terminales. Le refus ne conserve pas
l’identité de la préparation qui a éventuellement tenu la garde. Le premier
événement est antérieur au seul bloqueur aujourd’hui attesté : ses causes
passées ne peuvent pas être reconstruites depuis les seules tâches courantes.
Une liste workspace_tasks vide du maître ne prouve pas une racine libre.
Elle ne répertorie pas toutes les préparations du worker.

## Action et limites

Aucune garde libérée, aucune annulation, aucun fichier Git, profil, compte,
service ou tâche modifié. Aucun essai de mission, build ou test lancé.
Esquirol continue selon son propre état. Les autres travaux autorisés aussi.

Conserver les deux enveloppes et leurs bases exactes. Photos reste chosen/pending,
pas admise ; ne pas créer une clé de remplacement. Après disparition attestée
de l’occupation et fin des appels concurrents, seul le propriétaire peut décider
un rejeu strict dans son objectif autorisé, avec source et base675 inchangées.
Un changement de HEAD source doit être traité comme une autre enveloppe, jamais
injecté sous la même clé. Aucun rejeu n’est exécuté dans ce diagnostic.

La seconde demande n’a pas de reçu persistant trouvé. Cette observation n’est
pas une autorisation automatique de relance ni une preuve générale d’arrêt.
Le geste immédiat est de laisser finir Esquirol puis relire ses conditions
natives de clôture ; aucune posture ou stratégie n’est changée pour contourner
la garde. Le premier bloqueur reste inconnu faute de trace historique conservée.


## Complément : verrou source du Mac, refus à09:57:44UTC

Request opus201-patrimoine-communes-suite07-8-20261101-01,
project_preparation_source_busy/origin_project_preparation, observed1793527064.
Au registre courant : aucune tâche ni reçu auto pour cette clé, cohérent avec
le refus not_recorded transmis par le parent. Aucun rejeu exécuté.

À09:58:54UTC, lsof a observé Python PID52891, UIDmoi, descripteur3 ouvert sur
/Users/moi/.cache/bridget-core/project-preparation.lock, inode599398446.
Le processus a terminé entre ce relevé et la lecture ciblée ps/lsof suivante.
Il n’a donc pas été possible de relever sa commande, son parent et sa naissance.
Ce descripteur ouvert ne prouve pas à lui seul qu’il détenait déjà le flock à
09:57:44. La durée réelle de détention historique reste inconnue.
Les relevés suivants ne montrent ni ce PID ni un autre processus ouvrant ce
fichier. Aucun signal, effacement de verrou ni tentative de prise du flock.

Le script source utilise un verrou par répertoire du registre, commun aux
préparations de ce namespace, pas un verrou Git propre à Esquirol. Il le garde
pendant inspection SSH, éventuel export/import et publication de mapping.
L’attente d’un concurrent est limitée à30s. Chaque Git/SSH a un délai300s ;
le parcours natif plafonne la préparation à595s, réduit par son échéance restante.
Ce ne sont pas des mesures de la durée du processus52891.

Le dossier temporaire restant bridget-prepare-92d65ec6… est ancien
(mtime1793503603), destinationKS/commit68442d3f…, réponseok=true :
il ne correspond pas à la demande SYS/base675 et n’est pas attribué au bloqueur.
Il est conservé, aucun dossier supprimé.

Conclusion bornée : ce refus source est distinct du Shared occupé surSYS.
Aucune réservation exécutante pour cette clé n’est attestée par le registre
courant. Le détenteur exact au moment du refus ne peut pas être reconstitué
depuis les traces conservées ; aucun diagnostic de verrou orphelin n’est établi.
Le fichier vide restant ne prouve pas un verrou encore pris. Conserver clé,
base et enveloppe ; seul le parent peut décider d’une reprise strictement
identique après fin de ses appels concurrents et vérification native actuelle.
Esquirol et les autres travaux suivent leur propre état. Aucun code exécutable
modifié, aucun build, test, service, compte ou droit touché dans ce complément.
