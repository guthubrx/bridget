# Préparations concurrentes du projet
Statut : correction active sur les sept services ; schéma de délégation inchangé.

## Incident et limites de preuve
Message opus201-origin-proof-busy-parallele-20261031-01, clé
opus201-nommes27-lot5-20261031-01. Après le rejeu strict du propriétaire :
admitted, tâche46b3a813-c8ca-45a5-b46d-252eb5c49544, même propriétaire.
Première lecture : starting, error=null, remote_prepared=true ; aucune mission
émise à cette lecture. Aucun rejeu ou changement métier fait par ce diagnostic.

Le journal du worker source montre capacity limit=1. Il s'agit du créneau
séparé de préparation de projet, distinct des 32 contrôles de lecture.
Le journal historique ne contient pas sa corrélation : l'attribution de CE refus
à cette clé reste inconnue. Le code a cependant un défaut démontrable :
deux préparations concurrentes sont refusées avant d'atteindre le verrou
exclusif du registre. Leur ResourceLimit est présenté comme une saturation
de contrôle de droits, avec la mauvaise étape origin_session_reattest.

## Correction par réutilisation
Le worker reçoit au plus 16 préparations actives ou en attente. La source
réutilise son verrou OS existant : une seule mutation de registre à la fois,
attente au plus30s. La destination garde son verrou non bloquant, pour qu'une
préparation en sens inverse ne provoque pas une attente croisée des deux
verrous. Le délai original couvre cette attente : aucun délai de requête
ou de mission n'est augmenté. Les préparations locales utilisent la même
attente source ; les bornes d'admission existantes restent applicables.

Aucun nouveau magasin, file persistante ou cache de preuves. Chaque appel
garde son canon, sa base et ses vérifications avant/après. Aucune mission n'est
démarrée avant les contrôles d'admission et de remise existants. Une préparation
interrompue conserve ses états selon le contrat existant, sans retrait forcé.

Causes distinctes :
- origin_proof_busy : capacité des contrôles, ancienne classification conservée
  pour une saturation globale du maître ou une lecture worker.
- origin_project_preparation_busy : capacité de préparations reçues du worker,
  étape origin_project_preparation.
- project_preparation_source_busy : verrou source encore occupé après30s.
- project_preparation_busy : verrou distant occupé lors du contrôle immédiat.

Les messages conservent la clé et exigent la vérification de l'admission durable
avant tout rejeu strict. Aucun remplacement d'issue inconnue, aucune suppression
de verrou, aucun changement de droits ou de compte conseillé. Le journal worker
ajoute opération et corrélation, sans données d'authentification ni corps libre.

## Vérification ciblée
Trois processus réels attendent un verrou déjà détenu, puis entrent un à un.
Un autre cas vérifie expiration source et refus distant immédiat, sans retrait
du fichier de verrou. Le dispatcher vérifie les bornes32/16 et le refus d'une
préparation expirée avant toute E/S ; la projection vérifie causes et étapes.
Les contrôles ne constituent pas une mesure de débit ni une garantie de16
admissions simultanées. L'attente de verrou n'est pas une file FIFO garantie.


## Revue et chemin MCP réel
68fbe3e2-cfdf-4cd6-971d-b0fef3880d56 : pas de défaut bloquant du verrou ou
de la capacité. La réserve P2 sur les deux codes helper est réfutée par le
chemin réel : CallerScope ne fait aucune préparation (native_delegation),
puis delegation_mcp::execute conserve intégralement le detail d'un refus de
la délégation. Il ne passe pas par caller_scope_refusal_message. Le test
de transport MCP existant est étendu aux deux codes et vérifie code, cause,
étape, message et récupération sans perte. Aucune extension de la projection
avant preuve vers des erreurs métier n'est donc nécessaire.
Les deux écritures de message écrasées signalées par la revue sont retirées.

Le plafond global32 du maître reste commun aux opérations d'origine.
Les compteurs32lectures/16préparations du worker sont séparés ; leur somme
n'est pas une garantie de48 opérations relayées. CPU et débit ne sont pas
mesurés par ces tests. Aucun nombre de missions simultanées n'est garanti.


## Vérifications finales et chargement
Sept contrôles distincts PASS : quatre Rust de dispatcher/projection, deux
Python de verrou avec de vrais processus, et le transport MCP existant étendu
aux deux codes helper. Les passages intermédiaires recouvrent ces mêmes tests ;
ils ne sont pas additionnés. Aucune campagne générale ni mission métier d'essai.

Source exécutable238223b518d02598cfcc035dca624c98dcaf4bb3, build
221-preparation-queue-r1. Les 291fichiers Rust et le helper Python ont été
comparés entre source et snapshots de compilation. Cache de compilation réutilisé.
Linux commun construit sur SYS, GLIBC2.34 :
SHA256ae2105f99aec8675663e19fe7d82adb3cb4ae81f35093a8fa7d0f0a98941a378.
MacARM64 : SHA256cc35d1a9cf73abd687a699c0f55ba919278f1422b6b9e7fe33ae254df537e77e.

Chargement réellement observé et haché : maîtreCartae3217377, mobile3217486,
workerCartae3217852, workerSYS1130851, workerKS1494304 ;
workerMac9216, pont9297. Coupure maître2,81s. Processus Cartae2500061
(birth270585460) et3149928 (birth300090634) observés conservés ;
aucun processus métier actif observé sur SYS au passage. Aucun service métier
modifié et aucune admission inconnue rejouée.

Retour : précédente image active221-origin-parallel-r2, compatible avec les
formats conservés dans les deux sens. Pointeurs rollback Cartae/Mac relus
après écriture atomique native sous verrou ; activation=false, updaters repris.
Authentification native réussie après le passage, puis lecture de la revue
68fbe3e2 réussie/result_available sans erreur. Aucun restart supplémentaire.

Le lot5 garde une seule tâche46b3a813-c8ca-45a5-b46d-252eb5c49544 ; lecture
postérieure au passage : result_available, error=null. Le propriétaire garde
la lecture et l'acquittement de son résultat métier. Aucun doublon créé.

Les processus MCP déjà ouverts ne sont pas déclarés rechargés. La correction
de préparation s'exécute dans les services et son refus de délégation utilise
le transport de détail historique vérifié ; aucun nouveau schéma n'est requis.
La limite de rechargement des clients signalée dans origin-parallel.md reste
distincte. Les verdicts de droits ne sont ni partagés ni assouplis.

## Notifications et rangement
Résultat adressé au propriétaire par
bridget221-preparation-parallele-resultat-20261031-01,
deliveryfa6b2e70-ac33-41bf-8523-512a0788d6df, in_flight sans preuve de lecture.
Les deux autres destinataires prévenus ont reçu l'avis de reprise.

Copies de transfert retirées : Mac44512215octets, Cartae44499280,
SYS3218055, KS41307095. Actif, retour, preuves, snapshots de compilation
réutilisés et worktree de revue conservés. installedMac refuse ce staging
non enregistré pour la copie du dépôt ; installedSYS garde les caches faute
d'observation complète des processus. Aucune purge ou réécriture forcée.
Revue lue/ACK, clôture demandée une fois ; son usage global n'est pas établi.
Programme52/182 InProgress, aucune catégorie supplémentaire clôturée.
