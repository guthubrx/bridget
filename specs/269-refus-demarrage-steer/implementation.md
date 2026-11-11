# 269 : refus de démarrage, steer et cache de compilation

## Incidents vérifiés

- SYS : aucune compilation encore active au contrôle. Cache partagé à 22 Gio physiques, disque libre 1,1 Gio. Le cache existant est traversable et sudo lsof demande une authentification ; le repli limité au compte ne s’applique donc pas. Après saisie sécurisée du mot de passe et lecture administrateur, le nettoyeur natif inchangé, sous verrous Cargo, a retiré27658693770octets logiques. Espace libre :1217060864 ->23992938496octets. Aucun agent Parix arrêté. Le secret n’est ni dans Git, ni dans les arguments, ni dans les journaux.
- Deux refus de démarrage : c06bd9f9 et ca2dcf0b. Le reçu Fleet conserve quota_exceeded avec limite 24 ; le parcours natif a remplacé ce fait par remote_spawn_not_new et laissé queued sans échéance. Aucun Start ni mission remis. Les propriétaires ont depuis annulé les tâches ; aucune reprise ni nouvelle clé par la maintenance. Leurs worktrees restent conservés et sans rapport natif, car aucune instance/lien de commande n’a été créée.
- Deux steer Claude :8aa5f1bd et d2907ee3 restent dispatching, aucune injection journalisée. Le pilote refuse le steer strict, puis le wrapper ignore un DeliveryRejected non lié à une exécution ni à un message ordinaire actif. Une consigne queue attend sa fin de tour ; aucun contrôle intermédiaire sans interruption n’est pris en charge par ce pilote.
- Les quatre steer Codex comparés sont déjà acked. Les journaux portent prompt_dispatched aux quatre heures transmises au responsable. Le premier reçu in_flight est asynchrone ; la remise ne prouve pas que le modèle suit la consigne.

## Correction du cache

Un cache neuf était créé par Cargo avant son enregistrement, avec des droits publics par défaut. Le script crée maintenant le seul staging autorisé en 0700 et l’enregistre avant Cargo. Les droits des caches préexistants ne changent pas. Le test neuf échoue avant correction ; les 71 tests du nettoyeur passent ensuite, dont 3 ignorés pour les conditions de plateforme déjà prévues. Aucune garde de suppression n’est retirée. Les prochaines qualifications utilisent explicitement CARGO_INCREMENTAL=0 pour éviter l’accumulation mesurée de 20 Gio logiques d’incrémentaux sur SYS.

Preuves privées : /Users/moi/.cache/bridget269-incidents. Retours conservés sans réveil : mcp-86189-6af48954-1b, mcp-86189-6af48ac6-1c, mcp-86189-6af48ac8-1d, mcp-86189-6af48bda-1e et mcp-86189-6af48c46-1f. Une conservation ne vaut pas lecture.

## Refus et conservation

Les deux régressions initiales échouent sur le runtime 268 : la demande de
lancement reste queued et le rejet corrélé ne produit aucun reçu de livraison.
Les quatre nouveaux tests passent après correction. Le contrôle de conservation
est ensuite étendu à la vue de lecture : le rapport reste indisponible et la
clôture exige toujours son accusé worker. L’état des tâches annulées et leur erreur
historique ne sont pas réécrits. Leur lecture affiche le refus Fleet corrélé.

La revue initiale 8db01550 confirme les deux causes. Les objections sont traitées :
Failed seul, transport de la définition figée, aucune reprise de commande terminale,
rejet seulement pour la tête d’une livraison idempotente en attente, aucun faux ACK
ou événement de fin de tour. Résultat lu, ACK, garde close, vue conservée.
La contre-revue 6f134d15 reproduit le quota de flotte24 avant démarrage. Commande
native failed/quota_exceeded, sans Start ni mission ; demande annulée par son
propriétaire après lecture du reçu, aucune alternative ni modification du plafond.
Elle n’a donc produit aucun avis. Sa vue de lecture reste à conserver par le
worker, ce qui a révélé le second cas couvert par le correctif.

La qualification regroupe les filtres dans une seule commande Cargo et désactive
les fichiers incrémentaux. Les fixtures de CLI appellent directement la bibliothèque
ou créent leur socket, sans le point d’entrée qui pose normalement umask 077.
Un contrôle élargi lancé avec umask 022 a constaté des sockets 0755 et a échoué à
raison sur les gardes du namespace. La qualification utilise désormais umask077,
comme le point d’entrée natif, et construit d’abord l’exécutable debug requis.
La tentative de canonicaliser le chemin temporaire n’était pas la cause : elle
est retirée. Aucune garde ni permission produit n’est changée pour les tests.

## Qualification et activation

98 contrôles ciblés passent sur Mac et Debian 12 : refus 269, guidage 156, wrappers
162, refus locaux et issue incertaine198/227, liaison de conservation221.
Les deux analyses statiques strictes et les builds debug/release réussissent.
Le binaire Linux exige glibc 2.34, sous le plafond 2.36. L’APK 8036 garde le certificat
de production. La recette cumulative reproduit exactement dix fichiers ; elle
conserve les lots précédents et le programme réservé reste hors activation.

Sources exécutables : f2fd4d0307fb0a15749fb0615c687ad6c7de29a5, poussées et
alignées sur quatre checkouts propres. Version 0.8.36, build 269-refus-guides-r1.
Publication : https://github.com/guthubrx/bridget/releases/tag/v0.8.36, six fichiers
retéléchargés sans authentification et vérifiés par taille et SHA.

Sept services chargent l’image vérifiée : core 3577520, mobile 3578075 et worker
Cartae 3582246, worker SYS 3670391, worker KS 1362235, worker Mac 22889 et connecteur
Bridget de T3 23613. Les quatre pointeurs de retour désignent 268, qualifié par
les contrôles de formats dans les deux sens et écrit atomiquement sous le verrou
natif. Aucun retour exécuté. Le watcher Mac 23874 est rechargé. Empreinte Mac :
5cdb9a61cf12a85dac6faf234ad8248d1ce2023e8a213dbc856f9ff84d27228c.
Empreinte Linux : 8666eea902ae3d35067c5bea2c6b2d14e285a486102df35646ed54c66cb67106.

La coupure core dure 4,68 secondes. Dix-sept superviseurs métier conservent leur
PID et leur naissance pendant leur passage : huit Cartae, un SYS, quatre KS et
quatre Mac. Cela ne les déclare pas tous encore actifs après leur fin naturelle.
T3 garde le PID 77762 et sa naissance du 11 novembre à 14 h 10 min 41 s : aucun arrêt,
remplacement ou redémarrage. Les anciennes images des sessions métier restent
chargées pour préserver ces travaux ; le refus précoce du maître s’applique aux
nouveaux contrôles natifs concernés. Le raccord MCP réel répond avec build 269.

Le circuit réel confirme c06bd9f9 et ca2dcf0b cancelled/cleanup_done=true, dossiers
présents et kept, aucun started_at/Start/instance/mission inventé, données Git
inconnues et erreur historique conservée. La lecture native de notre tâche 6f134d15
rend maintenant quota_exceeded, limite 24 et start_issued=false. Sa vue publiée
reste protégée, cleanup_done=false : la clôture worker existante exige un reçu de
commande, absent avant lease. Aucun accusé, fence ou reçu artificiel n’a été créé.
La tentative de rapport est terminée, mais ce fait ne remplace pas l’accusé de
conservation exigé. Cette preuve utile reste conservée, sans nouvelle mission.

## Retours et ménage

Avis avant passage aux six propriétaires affectés, conservés sans réveil.
Retours après vérification : mcp-86189-6af49f69-26 à mcp-86189-6af49f7d-2c,
y compris les causes, les gestes possibles et la fin de l’incident disque.
Les reçus stored ne prouvent pas une lecture. Aucun bulletin aux 986 présences.

Le cache privé SYS est purgé par installed après installation vérifiée : 2,88 Gio
logiques. Disque disponible final :31 772 246 016 octets, environ 29,6 Gio. Les exports
propres Mac/SYS et les images 266/267 inutilisées sur KS retirent 711 495 765 octets
supplémentaires. Version active 269 et retour 268 restent conservés sur quatre hôtes.

Ménage partiel gardé : le cache Mac reste conservé à cause du cwd non observable
du PID 33346. Les observations natives incomplètes empêchent aussi le retrait de
copies anciennes Mac/SYS/Cartae et de copies temporaires Cartae/KS. Aucun autre
processus n’est arrêté pour nettoyer. Les vues de revue, résultats, sauvegardes et
preuves restent protégés. Aucun worktree ni branche créé. Preuves privées et
bilans : /Users/moi/.cache/bridget269-incidents.
