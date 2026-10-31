# Références de tâches après refus Shared

Statut : correction vérifiée, chargement en cours. Incident
opus201-rattacher1439-sites-nommes-20261101-01 ; refus not_recorded/sansTask,
workspace_busy sur11candidats Cartae. Aucun reçu durable de cette clé à la
lecture initiale, sans rejeu ni mission de remplacement.

## Lecture réelle sans mutation
Le store du worker Cartae porte une commande released/native_ready pour
5b00f328-14fa-4c05-9ce2-e33bec70ebb3 ; son exécution est sous
/home/moi/10.Scripts/69.opus2D/.worktrees/5b00f328.
Même tâche côté maître : propriétaireaf5ff0db-b846-45a2-8158-c4ab53a89b84,
working/ronde3, guarded=true, cleanup_done=false. Son propriétaire peut lire
bridget_task_status avec cet identifiant. Aucune garde libérée, aucun arrêt.
La lecture maître seule contient aussi d’anciens états qui ne constituent
pas une preuve de réservation effective du worker.

## Correction bornée
La projection existante refus/admission réatteste le fil, retrouve son
identité vivante et vérifie le canon de la clé avant tout détail. Pour le
seul refus workspace_busy de Shared, elle ajoute detail.workspace_tasks :
- owner et owner_instance exacts ; pas de récupération implicite d’une
  ancienne instance, pas de tâche d’autrui ;
- lecture SQL de129métadonnées au plus, sans result/brief/historique ;
- filtrage par machine imposée si présente, projet sélectionné dans le
  registre, et recoupement de racine identique dans les deux sens au worker ;
- huit références maximum, ordre task_id croissant, troncature explicite ;
- aucun chemin, brief, résultat ni champ fournisseur projeté.

scope=owned_current_instance_tasks, basis=coordinator_task_state et
blocker_identity_attested=false. Ce sont des pistes pour task_status, pas
une preuve de la garde exacte. Une liste vide ne prouve pas un dossier libre.
Préparations worker, autres instances et autres propriétaires restent masqués ;
le message demande une corrélation ciblée par le responsable si nécessaire.
Sans machine imposée, ces pistes couvrent les placements déclarés du projet,
pas une attribution à l’un des candidats refusés. L’agent lit machine_id et
l’avertissement avant toute interprétation. Aucune priorité métier déduite du tri.
Une observation indisponible ne modifie ni cause, ni canon, ni sort de l’admission.
Aucun nouveau protocole, outil MCP, magasin ou changement de droits.

## Vérifications utiles et limites
Deux contrôles daemon distincts PASS : vraie requête SQL avec exclusions
propriétaire/instance/projet/machine/terminal et sortie bornée sans champs
privés ; vraie projection du refus après identité et lecture canonique,
absence de détail sans connexion ou pour Worktree, aucune tâche ni commande
créée. Les fixtures clonées exigeaient des identifiants enfant ET mission
uniques et une insertion explicite ; ces erreurs de montage ont été corrigées
après lecture des contraintes SQL, sans changement des données réelles.

Revue01ee6048 en lecture seule du premier patch : pas de P1, P2 discutés.
Elle n’a pas relu le dernier delta SQL ; cette limite n’est pas masquée.
Le guidage peut expliquer le recoupement : shared_refresh_allowed ne produit
WorkspaceBusy qu’après shared_refresh_needed et recoupement réel d’une
préparation ou commande. Il ne déduit pas l’identifiant de ce seul code.
Le tri borné est déterministe, sans promesse de trouver la garde effective ;
les pistes coupées sont signalées. Le demandeur de l’incident est monuments,
propriétaire de5b00f328, et non le responsable de ce diagnostic.
Le recoupement dans les deux sens reproduit la garde worker et est voulu.
Le delta final remplace tasks() par une projection SQL bornée et plus stricte
sur l’instance. Audit direct : lecture seule, filtre d’identité avant sortie,
pas de chemin projeté, garde/cause/admission inchangées, aucun nouveau lancement.
Résultat de revue lu/ACK et clôture demandée une fois.
Changement technique de diagnostic : pas de diaporama d’interface requis.
Le programme global reste52/182, In Progress ; aucune catégorie close.
