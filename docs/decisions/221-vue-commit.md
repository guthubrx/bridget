# 221 — Vue de lecture d'un commit publié
Date : 31 octobre 2026. Statut : contrat accepté, implémentation en cours ;
admission et activation non qualifiées.

## Contexte
Lire A ne doit pas déplacer le Shared B ni attendre un worktree descendant.
Un dépôt enfant avec alternates rendrait d'autres objets accessibles par SHA
et demanderait une référence anti-GC dans le dépôt partagé. La stratégie de
lecture doit rester distincte d'un worktree et de ses règles de rétention.

## Décision
Exporter les fichiers exacts du commit, sans .git, index, référence ni filtre.
Réutiliser la sélection pure compact207, le constructeur Git fermé et un seul
cat-file batch. Décrire les omissions et les liens dans une carte publique.
Conserver le manifeste complet et l'intention liée worker/commande/canon hors
de la vue, dans le budget existant de WorkerStore. Publier après audit et
synchronisation ; une preuve tronquée ou absente n'autorise pas une recréation.

Déplacer les écritures et l'export dans un helper du même binaire, sous le
bootstrap géré existant. Son marqueur précède RELEASE et conserve le groupe
Git si l'arrêt reste incertain. Vérifier Child, identité, naissance et groupe,
pas le seul PID déclaré. Aucun service ou ordonnanceur supplémentaire.

Sur macOS, chaque fichier et dossier reçoit fsync, puis une seule barrière
forte finale sur un fichier du même volume. Le mode est posé avant la
synchronisation. Ce groupe de synchronisations reste best effort face au
matériel ; il n'est pas une garantie absolue de coupure électrique.

## Conséquences
La vue n'est pas un dépôt Git. La lecture native d'une omission vise seulement
le blob exact lié au manifeste, sans écrasement. Les objets non ancrés peuvent
disparaître : rendre ce fait, sans substituer une base. Les droits readOnly ne
prouvent pas une invisibilité OS de toutes les autres lectures du même UID.
Le retour ancien exige un lecteur opérationnel complet des reçus et processus
conservés. Une simple désérialisation ne suffit pas. La capacité, le plancher
de formats et le parcours natif restent des gates avant première admission.
Le contrat complet et les preuves se trouvent dans les artefacts de la session.

## Retrait natif — contrat avant implémentation

Cette section fixe les préconditions du retrait ; elle ne constitue pas un
parcours déjà disponible. Ne pas ouvrir les admissions avant son raccordement
et ses contre-épreuves.

Le demandeur est le propriétaire parent de la Task, ou l'opérateur natif
autorisé. Réutiliser l'authentification de clôture existante. Un PID, un chemin,
un booléen « autorisé » ou la notice publique ne donnent aucun droit.
Le canon de retrait lie la préparation, son empreinte et la vue publiée ;
il n'ouvre pas une nouvelle mission et ne modifie aucun autre chantier.

Exiger la clôture native de toutes les commandes de cette Task et la disposition
finale des slots de résultat, ou leur preuve native de non-remise. Exiger aussi
l'arrêt confirmé des helpers d'export et de fichier. Conserver leurs preuves
de clôture tant que la vue n'est pas retirée. Une absence de reçu après collecte
n'est pas une preuve de clôture ; aucun fallback depuis un code de tâche inconnu.

Réutiliser le même ViewJob, le même fichier de verrou et le même marqueur.
Ajouter une opération fermée de retrait, distincte de la matérialisation :
aucune mission active n'est nécessaire ni autorisée par cette opération.
Le reçu existant porte une intention monotone de retrait avant RELEASE.
Il enregistre son marqueur puis sa clôture ; aucun second store de nettoyage.

Le helper compare l'intention et la publication privées, l'inode et le
périphérique de la vue. Il audite tout son contenu avant le premier retrait.
Il vérifie les lecteurs réels du compte worker, sans lire leurs secrets et sans
leur envoyer de signal. Inventaire indisponible, tronqué ou incomplet :
conserver et rendre la vérification manquante. Le périmètre de cette observation
n'est pas une isolation contre un administrateur de la machine.
Le mécanisme d'inventaire reste à qualifier sur Mac et Linux avant le code
destructif : l'index de copies de disk_hygiene n'est pas suffisant tel quel
(sortie/attente non bornées et sélection de dossiers à établir).

Déplacer atomiquement la seule vue attendue dans un contenant privé exclusif,
sur le même volume. Ne jamais remplacer un chemin existant. Refaire le contrôle
d'identité, l'audit et l'inventaire des usages après ce déplacement. Un contrôle
échoué conserve le contenant et rend son emplacement réel ; aucune restauration
automatique qui pourrait remplacer un dossier recréé par un opérateur.
Le reçu permet de retrouver ce contenant après une coupure.

Le retrait travaille par descripteurs, sans suivre de lien et sans commande
récursive aveugle. Au rejeu d'un retrait déjà autorisé, les entrées restantes
doivent être un sous-ensemble exact du manifeste privé ; les suppressions
déjà effectuées sont admises seulement dans cette phase. Un intrus, un lien
substitué ou des octets divergents conservent les données restantes.
Les bornes de nombre, volume, durée et possession restent celles du helper.

Une petite preuve privée de fin lie l'intention et l'identité publiée aux
emplacements retirés, puis est synchronisée. La sortie0 seule ne suffit pas.
Après arrêt confirmé du helper, le store durcit un tombstone compact dans le
reçu avant de retirer le manifeste et ses compagnons connus. Une coupure entre
ces étapes conserve des compagnons comptés, jamais une réussite de nettoyage
inventée. Une corruption privée est conservée pour examen.

Ce tombstone interdit la recréation sous l'ancien tuple, même sans manifeste.
Il garde les identifiants et la conservation finale, sans faits Git fictifs.
Le quota peut alors libérer la réservation du grand manifeste ; les octets
encore présents restent comptés. Ne pas augmenter le quota pour masquer un
retrait incomplet.

Ne pas collecter automatiquement le tombstone après sept jours : WorkerPrepare
ne porte pas d'expiration. La preuve de non-recréation ne doit pas disparaître
par supposition sur une durée inexistante. Le compact reste dans les bornes
existantes du store. Le lecteur de retour et le contrôle de bascule doivent
comprendre aussi ce compact et le retrait en cours, avant les premiers reçus.

Contre-épreuves requises : Task active ; résultat non acquitté ; helper vivant
sans sidecar ; lecteur réel avec fichier ou cwd ouvert ; inventaire incomplet ;
dossier divergent ; remplacement de chemin ; coupure avant/après déplacement,
pendant retrait et pendant compactage ; rejeu exact ; source Git indisponible ;
quota plein ; ancien lecteur. Aucun de ces cas n'autorise un nouvel essai de
mission métier, un retrait forcé ou une modification des droits.

## Choix utilisateur — conservation sans nouveau privilège

Le choix a été confirmé le 31 octobre : conserver les fichiers si l'inventaire
est incomplet ; aucun contrôle administratif ajouté au worker.
Les prototypes administratifs privés ne deviennent pas du code produit.

La conservation ne doit pas immobiliser un grand manifeste dupliqué dans le
petit store. Dès la première exportation, la carte de données contient aussi
le manifeste complet du commit. Elle reste une donnée non fiable : seule son
recalcul dans l'intention et la comparaison à l'empreinte privée font autorité.

Après annulation/clôture autorisée par le parent et vérification native de
toutes les preuves, figer la vue dans le reçu existant. Aucun nouvel export,
matérialisation ou démarrage ne peut reprendre cette vue figée.
Réduire ensuite l'intention privée à une référence fermée : même liaison,
source, racine de données et empreinte SHA256 de l'intention complète.
La publication privée lie encore périphérique/inode et cette empreinte.
Le lecteur reconstruit l'intention depuis cette référence et le manifeste
non fiable de la carte ; il exige le même hash et les mêmes validations.

Aucun fichier de la vue n'est modifié ou déplacé pour cette réduction.
Une carte absente, tronquée ou altérée reste un refus avec conservation.
Le rapport fonctionne toujours sans le dépôt Git source si les preuves
et la carte intactes sont disponibles. La carte ne peut choisir ni ses
racines privées ni une autre commande.

L'ordre est : figer le reçu en gardant la réservation pleine ; remplacer
atomiquement l'intention privée ; réduire la réservation. Une coupure
conserve donc soit la preuve pleine, soit la référence, sans sous-réserver.
Les temporaires restent dans la réserve de transition existante.
Le compact conserve un tombstone de non-recréation. Les preuves de commandes
et de slots peuvent ensuite suivre leur collecte ordinaire ; le tombstone
de vue demeure. Aucun second store, service, nouvel accès ou quota augmenté.

Avant un futur retrait destructif, réserver puis durcir à nouveau un plan
complet de retrait privé ; ne pas détruire la seule carte nécessaire à sa
reprise. Ce cas, le lecteur de retour et tous les états intermédiaires
doivent être qualifiés avant ouverture des admissions.
