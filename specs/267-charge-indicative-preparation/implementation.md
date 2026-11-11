# Diagnostic 267

## Déblocage immédiat

L’utilisateur demande de supprimer les refus CPU et charge qui cassent son travail. Le contrôle était déclenché sur une machine imposée même lorsque le CPU n’était pas plein. La charge moyenne était divisée par le nombre de cœurs puis plafonnée à 100 %, ce qui rendait toute charge supérieure au nombre de cœurs bloquante.

La télémétrie est désactivée à chaud pour Mac et Cartae dans les registres existants, sous verrou et avec sauvegardes privées. Les autres définitions, protections, comptes et tâches sont conservés. Cette mesure coupe aussi le seuil mémoire du routeur ; il sera rétabli après chargement du correctif générique. Aucun redémarrage de T3, aucun rejeu métier. Une vraie admission Mac, revue ff34af25-8fa6-4829-854d-6d52d2a6881e, a rendu son résultat.

Le premier écrivain choisi refusait le registre worker de plus de 64 Kio après mise à jour du premier fichier local ; la lecture a identifié sa borne. Le passage est repris avec l’écrivain atomique natif des registres, contrôle de la valeur précédente, sans restaurer une ancienne copie. Le maître Cartae est en révision 15 après déblocage des deux cibles ; les reçus privés conservent chaque opération.

## Correctif portable

CPU et charge restent utiles au classement automatique. La cible imposée par la demande ou par la politique n’est plus refusée ni retirée de la liste pour ces deux mesures. La mémoire fraîche sous seuil reste bloquante. Aucune structure de reçu ni valeur historique n’est changée ; aucun champ sérialisé ajouté. Le code est commun à toutes les machines.

## Préparation Cartae

Reçu parix-devantures-couverture-20261111-r1-worktree : choix Cartae durci avec CPU 29 %, mémoire 74 %, charge 22 %, puis remote_prepare:ResourceLimit. L’admission demeure chosen/pending sans tâche. Aucun lancement ni remplacement effectué ici. Le code refusait aussi une inspection pure si 128 préparations étaient présentes ou si un bootstrap était retenu, alors que cette inspection n’écrit aucun reçu, ne réserve rien et ne lance rien. La régression avec 128 préparations échoue avant patch sur ResourceLimit.

Ces deux bornes sont réservées aux vraies créations. Les contrôles de connexion, identité, droits, dépôt et base restent appliqués à l’inspection. Les limites des files entrantes sont conservées. Le guidage remote_prepare:ResourceLimit distingue la borne interne du CPU/RAM, garde la cause historique inconnue lorsque non attestée et demande le parcours de reprise de la même admission. Des événements fermés bridget_prepare_capacity corrèlent les prochains refus avec held_bootstrap ou prepared_slots, sans corps ni secret. Ce correctif couvre un défaut reproduit ; il ne prétend pas reconstruire rétrospectivement le sous-motif du reçu historique.

## Vérifications et état

La revue initiale Mac confirme le couplage des gardes et relève le filtrage ultérieur de la machine imposée. Le correctif couvre les deux endroits, sans ajout de champ sérialisé. La fixture du quinzième appel168 passe maintenant en mémoire 5/CPU 20 : supprimer cet appel faisait attendre son faux worker et empoisonnait le verrou des tests suivants. Les trois régressions CPU/charge/mémoire passent. La revue finale et les contrôles complets ciblés sont en cours.


La revue finale 6d13003d-d8d2-4b9e-a40d-ea955d23934f ne trouve aucun défaut bloquant. Ses remarques sont traitées : portée this_request et version du refus précisées ; absence de journal corrélé ne désigne pas arbitrairement une borne ; causes des deux logs remplacées par leurs littéraux. Les vues de revue sont conservées après lecture, ACK et clôture propriétaire. Le message d’un refus mémoire nomme la mémoire comme cause, même si les autres mesures franchissent aussi un seuil de conseil. Aucun changement de schéma durable.

Les régressions échouent avant patch et réussissent ensuite. Les familles 267, 168 et 189 et l’analyse statique stricte passent sur Mac et Debian 12 ; les compilations finales sont vérifiées avant installation. La reconstruction cumulative reproduit exactement six fichiers. L’APK 8034 garde le certificat de production. Les binaires Linux exigent glibc 2.34, sous le plafond 2.36. Les anciens fichiers de test intégraient une attente de quinzième sonde : la sonde mémoire reste effectuée pour garder le scénario de bout en bout cohérent.
