# Validation206

## Faits de l’incident

Tâche eba4db89-6703-4dd2-b79d-8e30d76bbc39, clé opus2d-sol-compact-20261029-01. Lecture native de la tâche et du reçu worker en lecture seule : failed, erreur remote_start_failed:permission_changed, mission_issued_at=null, cleanup_done=true, guard_released=false. Worktree propre conservé à sa base2aa616d41573dc2b9a8312789c1134e172552818, aucun commit. Reçu de commande génération1347 terminal sans permis Release. Le refus est antérieur à la remise métier ; le bootstrap a été arrêté.

Le journal natif du parent montre le premier appel auto_delegate interrompu à04:29:30UTC. Un appel ultérieur a relu la même tâche à04:30:19 ; aucune seconde tâche créée. Le run488 est resté actif, run489 a été annulé. Ces faits ne démontrent pas le champ de permissions divergent. Le205 ne conservait ni l’étape précise ni les catégories : la cause détaillée historique est inconnue, sans preuve de panne de compte, de serveur ou de révocation.

## Correction

Les six comparaisons existantes restent strictes. Le diagnostic ajoute une étape et uniquement des noms de catégories fermés ; aucune ancienne/nouvelle valeur, credential ou texte fournisseur. Le refus technique public reste permission_changed. La chaîne interne bornée est conservée dans le champ d’erreur existant de Task ; aucun schéma de stockage ni protocole worker neuf. Les anciens refus et suffixes invalides rendent unobserved avec changes vide.

Le guidage distingue nettoyage/rapport en cours et fin confirmée. Une tâche failed ne reprend pas par Continue ou rejeu. Clôture propriétaire et attestation fraîche restent nécessaires avant une nouvelle demande autorisée. Aucun changement de droits ou remplacement silencieux.

## Contrôles et livraison

Tests ciblés et revue en cours. Aucun chargement206 encore effectué. L’envoi utile à monuments mcp-50239-6ae2cf59-6 est accepté in_flight, lecture non déduite. Il ne demande ni gel global ni attente des quatre chantiers indépendants.
