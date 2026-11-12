# Faits et correction 276

Deux demandes bridget_auto_delegate ont eu leur réponse perdue. Les reçus natifs
du propriétaire attestent les tâches f94d5042 et fc06d175, toutes deux admises.
Leurs identifiants complets et enfants ont été rendus au responsable par le
parcours normal, sans rejouer d’admission. Le premier appel était déjà admis ;
le second a été admis par le rejeu strict du propriétaire. Ce n’est pas une preuve
que toute recherche par rejeu serait sans effet.

GuichetLookup consulte seulement le registre historique du guichet. Une clé
absente y rend le même code idempotency_expired qu’une protection expirée. Le MCP
en déduisait applied:false sans résultat métier. Cela décrit à tort une absence
d’application et masque la séparation avec les admissions natives.

La projection sans payload métier rend maintenant applied:null, scope explicite
guichet_request et native_admission_checked:false. Cela couvre aussi queued,
outcome_unknown et une coupure après dépôt. Les résultats terminaux qui doivent
avoir un payload continuent de refuser son absence. Les résultats métier prouvés
gardent leur booléen. Aucun format wire, stockage, droit ou accès de registre ne
change. Aucun rapprochement natif n’utilise un issuer_scope de guichet déclaratif.

Le texte précise le parcours : task_status avec un task_id connu ; sinon rejeu
natif strict uniquement si le mandat reste valide, car un appel jamais reçu peut
alors être admis. Pour un contrôle sans effet ou un brief périmé, faire identifier
le reçu par un responsable autorisé sans rejouer. Ne pas changer de clé après
une issue inconnue. Le catalogue et la référence native expliquent cette portée.

La remise feea6010 a une entrée dispatching côté maître, la même instance
récipiendaire que la tâche fc06d175 et une ligne de spool SQLite côté wrapper.
Elle est conservée et différée pendant le tour courant. Le journal montre le
même message de mission encore actif. Aucun ACK ni lecture n’en est déduit.
Le code ne perd pas ce corps : il attend de pouvoir l’injecter. Aucune mission
ni remise métier n’est rejouée pour ce contrôle.

Les deux tests276 échouent sur le runtime274, avec applied:false au lieu de null,
puis passent après correction. La table couvre sept états sans payload et garde
les quatre refus de terminal sans résultat. Le test Unix exerce le maître ancien
avec idempotency_expired, le scope exact et l’absence de toute commande native
ou mutation supplémentaire. Les tests voisins sont réutilisés et exécutés.

Limite d’activation : les anciennes façades MCP266 déjà montées dans T3 gardent
leur code de projection. Le choix explicite de garder T3 ouvert est préservé.
Le correctif bénéficie aux façades chargées depuis la nouvelle image ; l’ancien
format encore chargé n’est pas présenté comme corrigé. Aucune preuve de session
n’est copiée pour essayer de monter une autre façade depuis un shell.

La revue R1 demande quatre précisions de guidage. La coupure après dépôt
réutilise désormais le constructeur commun de réponse ; le détail transport
reste séparé. Chaque code connu a une prochaine action spécifique, puis les
conditions où demander la décision du propriétaire. Le conseil natif conditionnel
reste réservé à la clé absente/expirée et exige d’abord d’identifier l’outil
d’origine. Les états terminaux techniques sont présentés sans preuve d’effet
métier ; une issue future non reconnue n’atteste pas non plus la fin. La valeur
future_issue du test est synthétique. Les tests de mutation vérifient scope,
absence de lecture native et guidage, sur file d’attente et perte après dépôt.

Dans la fenêtre 20:27–20:37, cinq remises de ce groupe sont retrouvées accusées,
dont une émission à 20:31:43. Aucun corps perdu n’est établi pour cette fenêtre.
La référence du cas vers 22:40 manque : les faits fournis ne permettent pas de
l’identifier avec certitude ; ne pas lui attribuer la cause de feea6010.

Qualification après R1 : les trois tests276 (dont les booléens positifs/négatifs
attestés), les deux tests de mutation modifiés et Clippy strict workspace/all-targets
passent sur Mac et Debian 12. Les trois autres tests voisins avaient passé avant
ces ajustements et leurs chemins ne changent pas. Huit tests distincts sont donc
couverts sur chaque plateforme. Les deux builds release réussissent ; Linux
exige glibc 2.34, sous le plafond 2.36. La recette cumulative est vérifiée sans
fuzz ; ses cinq fichiers du delta correspondent à l’export compilé. La revue
R2 porte sur les conditions de guidage R1 et conserve son identifiant natif.

R2 confirme les corrections des points 1, 2 et 4. Sa dernière remarque sur la
description guichet est traitée en retirant son conseil de rejeu natif ; seul
le périmètre de lecture et l’absence de preuve restent dans cette description.
Le candidat non activé r1 est remplacé par r2, sans réutiliser son répertoire
exécutable immuable. Le reste de la logique et les garanties wire ne changent pas.


## Publication et activation du 12 novembre 2026

Le code final poussé est a6fad403faeaf9f005670ae4bd8f9927aa7c5f40.
La version 0.8.41 porte le build 276-guichet-sort-inconnu-r2. Après R2,
les trois tests276, Clippy strict et les builds release ont repassé sur Mac
et Debian 12. Huit tests distincts par plateforme couvrent le lot et ses voisins.
Les deux revues natives sont dd185dd6-ad16-4d75-ad5d-62519198ed37 et
b9851cd7-fb0a-4a9f-a4d0-647e1c934c0d ; leurs remarques sont traitées.
La seconde a fait retirer le conseil natif de la description générale du guichet.

Les six fichiers publiés de v0.8.41 ont été téléchargés sans authentification,
puis comparés aux empreintes attendues. L’archive Linux exige glibc 2.34.
La signature Android reste compatible avec 0.8.40. Aucun lancement métier
n’a servi de test de publication.

Les services activables ont été chargés sur Mac, Cartae, SYS, KS et M1.
Les processus, fichiers exécutables et empreintes sont contrôlés après le ménage.
Cartae : core 1268597, mobile 1269367, worker 1269396. SYS : worker 3969576.
KS : worker 1621547. Mac : worker 94726, connecteur 94921, mise à jour 95388.
M1 : worker 43793. Le service de mise à jour Mac exécute le Python embarqué.
La coupure core a duré 3,25 secondes. Les superviseurs vivants ont été conservés
avec leur identité de processus ; aucune mission n’a été recréée.
Le point de retour 274 est qualifié et inscrit avec l’écrivain natif sous verrou.
Sur M1, l’état natif absent avant le passage n’a pas été inventé.

Empreinte Mac/M1 : 49846d96a9957d16ea290a9c139b0411951f4755a3cc64435e44219ba3ce2d48.
Empreinte Linux : 5455d8bcde70c7f52228535e4f04ff442d27219fb56adcca2200761fe3545cd5.
Empreinte du delta : ab182cc964beaf505205ed9b154ac98691c698ef212e8ed71566b5e41fcea98b.

Le lien natif du mainteneur répond après le passage et l’API mobile répond 200
avec une enveloppe version 1. Le catalogue d’une nouvelle façade est vérifié
sans appel métier ni copie d’une preuve de session. Les cinq propriétaires
réellement affectés ont été prévenus par information conservée, avant et après.
Le responsable du signalement dispose des identifiants des deux tâches retrouvées.
Une information conservée n’atteste pas sa lecture.

Limite maintenue : T3 PID 77762, démarré le 11 novembre à 14:10:41,
est resté ouvert. Sa façade MCP PID 86189 charge encore 266. Sa projection
n’a donc pas été activée par ce passage. Les nouveaux montages utilisent le
correctif ; les sessions déjà ouvertes gardent leur ancien code. Cette livraison
ne prétend pas avoir corrigé leur réponse en fonctionnement.

## Ménage et conservations

Le nettoyeur natif a retiré 6,12 Gio logiques de caches Mac. Les clones APFS
empêchent d’en déduire un gain physique identique. Au dernier relevé, la racine
Mac avait environ 61,4 Gio disponibles. Le ménage gardé du dépôt trouve une
seule branche main, un seul checkout et aucun retrait de branche ou worktree
éligible. Aucun retrait n’est forcé.

Six anciennes copies exécutables ont été retirées : trois sur Mac, une sur SYS,
une sur KS et une sur M1, soit 240 450 192 octets. La version active et le retour
274 restent conservés. Les images 266, 268, 271 et 273 encore utilisées sur Mac,
269 sur SYS et 268 sur KS restent présentes pour leurs processus vivants.
Les propres exports temporaires retirés représentent 579 413 060 octets répartis
sur les cinq machines. Les archives publiées et les reçus privés restent disponibles.

Deux inventaires lsof sur Cartae ont rendu un descripteur de type unknown
(PID 1268597/fd37 puis 1269396/fd8). Le contrôle direct fd37 a lu une socket.
Sur SYS, les deux inventaires du cache ont refusé PID 3969576/fd7 ; la lecture
ultérieure du descripteur le trouvait déjà absent. Ces observations ne suffisent
pas à qualifier tout l’inventaire : ni suppression forcée ni arrêt de processus.
Les anciennes copies Cartae et le cache de compilation SYS hors disque système
sont conservés. Aucun diagnostic de fichier occupé n’est inventé à partir de
cette seule observation incomplète. SYS a environ 37,9 Gio libres sur sa racine ;
Cartae environ 131,3 Gio et KS environ 78,7 Gio au relevé de clôture.

Les quatre scripts temporaires Cartae sont conservés : la comparaison au fichier
de référence distant n’a pas établi l’identité attendue. Les vues de revue natives,
les preuves, les sauvegardes et le runtime cumulatif réutilisable restent conservés.
Les preuves détaillées sont hors Git dans le dossier privé de maintenance 276.
Aucune capture d’interface n’est requise pour cette correction de protocole.
