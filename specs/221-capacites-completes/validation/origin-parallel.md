# Lectures simultanées : attestation d'origine
Statut : correction vérifiée sur source ; activation à constater.

## Signalement et cause
Message monuments opus201-incident-attestation-statuts-paralleles-20261031-01 :
quatre task_status simultanés, deux succès et deux refus
permission_attestation_unavailable / origin_session_reattest.

Le dispatcher worker refusait immédiatement au-delà de deux contrôles
d'origine, alors que le maître conservait jusqu'à 32 attentes. Le refus
ResourceLimit du worker était aplati en origin_proof_unavailable, puis en
permission_attestation_unavailable. Ce défaut reproduit la forme du signalement.
Les journaux antérieurs n'ont pas conservé la cause de chacun des deux appels :
leur attribution historique exacte reste inconnue. Les avertissements du pont
sur le volume des fils constituent un fait séparé, pas la preuve de cette cause.

## Correction
Une borne commune de 32 contrôles pour le maître et les lectures worker.
La préparation de projet garde son créneau séparé de un. Les compteurs worker
restent en dehors de la boucle de reconnexion. Il n'existe aucune file ou cache
d'attestations nouveau. Chaque appel exécute son contrôle indépendant avec ses
identifiants, son époque, sa preuve et son échéance ; les gardes de droits et
de rattachement avant/après réponse sont conservées.

Une saturation explicitement reçue ResourceLimit garde le code public
permission_attestation_unavailable et expose la cause fermée origin_proof_busy.
Le message demande d'attendre les contrôles en cours puis de relire les mêmes
task_id. Une opération mutante exige de vérifier son admission durable avant
tout rejeu strict. Aucun changement de compte, droits ou rattachement conseillé.
La portée reste cet appel ; les autres travaux suivent leur propre état.
Une cause différente ou libre n'est jamais projetée comme une saturation.

## Vérification ciblée
Quatre tests PASS : vrai dispatcher worker et socketpair avec deux contrôles
déjà retenus puis quatre lectures indépendantes ; plafond 32 et libération ;
classification du refus ResourceLimit ; portée et non-fuite daemon/MCP.
Aucune mission fournisseur, aucune tâche métier, aucune écriture dans leurs
reçus ou résultats. La fixture de lecture utilise une racine absente privée.
La première compilation a détecté un import et un type de lecteur manquants ;
ils ont été corrigés avant le passage réussi. Pas de campagne générale.
Log : /home/moi/.cache/bridget221-provider-lot/origin-targeted-r2.log.

32 est une borne de contrôles d'origine, pas un nombre garanti de missions.
Un programme ou un lien indisponible peut encore refuser son attestation.
Aucune réutilisation d'une preuve réussie pour autoriser un appel refusé.


## Revue indépendante
534fce05-a1ed-4165-a827-dcb85f44cf76 : pas d'objection bloquante.
Le chemin de reprise de démarrage effaçait aussi le motif busy. Ce complément
préserve uniquement ce code fermé et réutilise exactement le même refus guidé ;
toute autre cause garde le comportement antérieur. Le test vérifie l'égalité
de cette projection avec le refus direct. Aucun démarrage ni rejeu ajouté.

La réserve sur la fidélité du test est précisée : deux contrôles sont simulés
par leur compteur retenu, puis les quatre suivants passent par le vrai
dispatcher et le socket. Aucun fournisseur n'est simulé comme ayant attesté
des droits. L'affirmation de famine de préparation par les lectures est réfutée :
run_worker utilise deux compteurs distincts, origin_slots et
origin_preparation_slots, sélectionnés avant dispatch. Les bornes CPU/mémoire
de 32 lectures n'ont pas été mesurées. L'échéance est contrôlée par execute
avant l'opération puis après sa réponse, sans modification dans ce correctif.
