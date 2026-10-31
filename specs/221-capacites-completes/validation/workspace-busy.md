# Références de tâches après refus Shared

Statut : correction intégrée, poussée et chargée sur les sept services. Incident
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


## Chargement, retour et nettoyage
Source exécutablee1e5e857f752826d994213a92627ee27e0ebe965 ; build
221-workspace-busy-r1/version0.6.5. Comparaison des267sources Rust :
code de production identique entre main et snapshot de compilation ; seul
le corps cfg(test) de la fixture d’identifiants diffère après sa correction.
Cela n’exige pas de nouvelle compilation release. Cette différence est
explicitement conservée dans source-proof.json ; aucune identité octet pour
octet de tout le fichier de test n’est prétendue.
Mac et SYS ont reçu les mêmes deux fichiers de production avant compilation.

Image Linux commune construite sur SYS, GLIBCmax2.34, 41334544octets :
SHA2562515a855e045d1d33982a2f2d0203991d3806895a04a108eac49e781dc0e595f.
MacARM64, 34140240octets :
SHA256b1a9c324015dcf136d230b90ec8793957f145e72997f7c0e1293c9727b072d4b.
Processus réellement chargés et hachés : maîtreCartae3340078,
mobile3340164, workerCartae3341763, workerSYS1140488, workerKS1506085,
workerMac6349 et pontMac6781. Coupure maître2,28s.
Wrappers Cartae3149928/birth300090634 et2500061/birth270585460 préservés ;
wrapper SYS1137782/birth65945242 également observé conservé. Aucun service
métier modifié, aucune garde libérée ou mission rejouée.

Retour221-claude-terminal-r1 compatible avec les formats conservés dans
les deux sens ; pointeurs natifs rollback Cartae/Mac mis à jour par écrivain
atomique sous verrou, activation=false, updaters repris. Session native
révision10 réussie puis task_status de la même revue réussi. Aucun nouveau
schéma MCP requis ; clients ouverts et sessions fournisseur existantes
conservés, pas de rechargement ni de nouveau comportement prétendu sur ces
anciennes images en cours de travail.
Pas de nouvelle admission, de rejeu du refus ou de mission d’essai : les
nouveaux champs sont éprouvés par les tests et leur code est chargé ; aucun
nouveau refus réel n’a été provoqué pour les observer.

Copies binaires de transfert vérifiées retirées sur Mac/Cartae/KS :
124003632octets. installed appelé sur Mac/SYS : staging non enregistré côté
Mac et observation processus incomplète SYS ; caches conservés sans forcer.
Revue01ee6048 arrêtée, lue/ACK, clôture demandée et Git propre y compris
ignorés ; worktree/branche conservés faute de preuve d’absence globale
d’usage des processus. Workspace221 actif, actif/retour, anciens binaires
encore utilisés et preuves conservés. main intégré/poussé ; refs main des
miroirs SYS/KS synchronisées en gardant leurs HEAD détachés.
Preuves privées dans les caches bridget221-workspace-busy des quatre hôtes.
