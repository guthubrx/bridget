# 267 : charge indicative et inspections sans place

Statut : adopté, décision utilisateur du 11 novembre 2026.

## Contexte

Des demandes sont refusées alors que les machines gardent du CPU disponible. La charge moyenne ne mesure pas le pourcentage de calcul utilisé. Une inspection worker peut aussi être refusée par la borne de préparation alors qu’elle ne réserve aucun de ces objets.

## Décision

CPU et charge orientent le classement automatique. Ils n’interdisent plus une machine choisie explicitement, par demande ou politique. La protection mémoire reste active pour une mesure fraîche. Une inspection pure garde ses contrôles d’identité, droits, connexion, dépôt et base ; elle ne demande pas de place de création et n’est pas bloquée par les bootstraps retenus. Les créations et files entrantes gardent leurs bornes.

## Conséquences

Aucun paramètre personnel ni exception par hôte dans le code. Aucun nouveau champ de reçu : les faits historiques et les refus durables restent inchangés. Une modification de comportement n’autorise pas à recréer une mission d’issue inconnue. Les refus de préparation ont un guidage distinct des mesures matérielles et des événements de corrélation fermés. Le déblocage de configuration d’urgence sera remplacé par ce comportement commun, puis la télémétrie réactivée.
