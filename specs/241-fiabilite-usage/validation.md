# Validation 241 — In Progress

Début : 7 novembre 2026,10:25CET. ETA initiale212–425min ; après les tâches,
250–440min restantes à10:51CET (fin15:02–18:12). T3 non redémarré.

## Préparation et preuves de plan

Synchronisation SpecKit acquise, runtime1.2.5 conservé. Scripts/templates projet
absents ; skills specify/plan/audit-existing/tasks/analyze/implement lues et
protocoles appliqués directement, sans installer un runtime projet parallèle.
Feature héritée222 explicitement remplacée par241 dans le worktree, en accord
avec le chantier validé du fil. Main initial04ad641e propre, travail isolé dans
.worktrees/241-fiabilite-usage. Audit de réutilisation PASS, Analyze effectué
et relu : 13FR/7SC couverts, aucun CRITICAL persistant.

Contre-revue plan -01 refusée Mac pour saturation, not_recorded sans tâche.
-02 sans machine imposée, plan intégral au brief : Claude Haiku/SYS choisi,
tâchecf827b77 terminée, résultat lu, ACK natif acquis le7novembre. Les premières
lectures natives ont échoué sur le raccordement ; lecture SQL native en mode
ro pour ne pas perdre le résultat, puis lecture MCP et ACK après retour du lien.
Verdict et objections vérifiées dans adversarial-review-plan.md. Contrat de
révocation explicite dans contracts/admission.md ; pas d'autorisation fabriquée.

## Raccordement — cause globale toujours ouverte

Le runner installé valide config/reçu puis ouvre un SSH par connexion locale.
Validation seule :210–279ms sur3lectures ; six sondes publiques d'identité par
le relais :550–656ms. Aucun appel métier. Sources et runner diffèrent par la
commande resume, pas par ce chemin de flux ; aucune conclusion « ancien code ».

Deux nouvelles lectures du résultat de revue échouent sur10s/JSONL. Une capture
strace bornée14s du cœur est faite pendant un échec, sans contenu de chaîne
(-s0). Le traceur est interrompu/détaché ; cœur vivant. Les appels terminés
observés ne suffisent pas à localiser l'attente : les attentes inachevées doivent
être interprétées séparément. La première analyse des preuves avait employé
un UID constant incorrect ; les propriétaires des seuls fichiers de diagnostic
ont été remis au compte réel lu par pwd, sans modification du service.

Une seconde capture de piles, sans arguments, dure199ms pendant une lecture
réussie ; elle n'est pas une preuve de cause. Le lien puis l'ACK natif reviennent.
Un essai de transfert SSH Unix natif privé refuse l'ouverture de ses canaux ;
aucune permission SSH ou liaison de production modifiée. Le processus de test
est absent, la socket propre stale est retirée après contrôle inode/uid/refus.
Cet essai n'est pas présenté comme une solution de repli opérationnelle.

Instrumentation ajoutée dans DaemonConnection::transmit : phase fermée, code et
durée lors d'un échec, aucun corps/credential. Pas encore chargée en production.

## Premières régressions

Inscription : la vraie socket perd le premier ACK avant toute commande. Ancien
comportement :1PASS/1FAIL. Correctif :2PASS (ACK récupéré puis une seule mutation,
refus d'identité non repris),0,04s. Reprise limitée à une coupure précoce dans
le délai du caller ; un délai épuisé n'est pas prolongé. Ce correctif ne clôt
pas à lui seul la cause globale des dix secondes.

Répertoire local supplémentaire : ancien comportement FAIL avec
permission_mapping_unavailable ; nouvelle branche PASS,0,01s. Le booléen
origine=cible provient du WorkerReadView authentifié, pas d'un nom de chemin.
Les répertoires figurent déjà dans les droits du parent ; canonicalisation,
uid/mode, existence, ancêtres et protections restent vérifiés. Aucune écriture
du registre et aucune autorité de répertoire transférée à une autre machine.

Un test supplémentaire de délai avait libéré son pair immédiatement dans sa
première fixture : ce n'était pas une attente réelle. Le pair est maintenant
conservé jusqu'au retour client ; vérification finale en cours. La fixture de
projection est complétée avec la forme native Claude→Codex du signalement.

Commandes exécutées : python3 scripts/build.py cargo test -p bridget-daemon
--lib spec241_registration -- --nocapture ; même commande avec spec241_same_host
puis spec241_. CARGO_BUILD_JOBS=2 pour les passages suivants, cache de ce dépôt
réutilisé. Premier inventaire --list a construit les exécutables de test, sans
les exécuter : ce n'est pas compté en tests passés. Cache après contrôles~6Gio,
sous budget10Gio. Un import de fixture privée a été corrigé vers le type public
du transport après E0603 ; aucun contrôle de production affaibli.

## Non livré

Autorité durable, hors-Git, inventaire/suppression de plafonds, qualification
d'outils, convergence, audit final, version et activation restent à réaliser.
Les tâches ouvertes sont conservées. Aucun commit ou push de241 effectué.

## Résultats intermédiaires à 11 h 40 CET

Sept puis huit puis dix tests 241 passent (0,79–0,96 s d’exécution après build).
La préparation des tests d’état a rencontré deux chemins de socket trop longs :
la fixture ignore son label et utilise TMPDIR macOS. Donnée réelle relue avant
correction du harnais : TMPDIR=/tmp. Ce n’est pas un correctif de production.

Vue lente : avant correctif, la réponse publique d’identité attend le rédacteur
d’une autre vue (timeout de test 150 ms, libération contrôlée). Après séparation,
le test passe. Le collecteur existant fonctionne dans un seul fil dédié avec
référence faible, hors des appels et des échéances métier. Ce couplage est
reproduit ; il n’explique pas à lui seul toutes les erreurs JSONL historiques.
La relecture des futex inachevés du strace de l’incident ne montre pas de join
corrélé démontré. Conserver cette limite causale.

Autorité de mission : avant changement de gather, fin de tour refusée avec
permission_attestation_unavailable. Après : fin de tour et tour suivant admis,
révocations natives propriétaire/instance/racine refusées, annulation sans
reprise et ancienne tâche non promue. Marqueur Task version 1 optionnel ; test
de réouverture de SQLite, propriétaire étranger et contrat inconnu refusés.
Le parcours complet Start/Release et le lecteur de retour restent à qualifier.

Volume : ancien lecteur refuse le manifeste de 33 Gio (maximum 32 Gio). Nouveau
lecteur accepte, overflow u64 refuse. L’export physique de 640 Mio de 232 reste
une preuve distincte ; ne pas compter le manifeste synthétique comme export.
L’inventaire et les décisions par borne figurent dans limits.md.

Outillage : tests du parseur de commandes (Docker/Compose/Cargo, mentions
facultatives/négatives), puis vraies commandes de fixture : absent, sortie 23
inutilisable, sortie 0 disponible, expiration inconnue avec SIGTERM du seul
Child créé. Aucun modèle ni service métier lancé. La négociation complète et
le placement avec outils connus restent à vérifier avant livraison.

## Parcours acquis à 12 h 10 CET

Reprise native complète sur vraie paire Unix : Fence → Prepare → Start → slot
→ Release, avec fait courant du parent retiré avant reprise et connexion retirée
après Start. Une seule Task, même mission/canon, ancien tuple conservé ; aucun
processus fournisseur. Test spec241_admitted_recovery… PASS.

Autre recette sur listener Unix et vrais handlers/SQLite : premier résultat
d’admission perdu, deux rejeux simultanés au même canon, une seule Task et un
seul ordre de lancement vers le superviseur simulé. État puis annulation sur
connexion auxiliaire réelle, état durable cancelled. PASS 0,24 s. La première
fixture avait collisionné avec conn-1 déjà inscrit ; compteur de fixture ajusté
comme les autres recettes de sockets, sans changer la production.

Préparation Python : 51 tests PASS 25,77 s avec TMPDIR=/private/tmp canonique.
Le premier passage (49 PASS / 2 erreurs) rencontrait les fixtures de verrous
qui ne résolvent pas leur TMPDIR macOS avant le refus de symlink ; la donnée
réelle a été relue. Aucune garde de production n’a été retirée pour ces tests.
La nouvelle fixture Rust hors Git était sous le .git sentinelle de sa fixture
de permissions : refus Git correct. Son dossier est déplacé dans un frère privé
réellement hors Git pour vérifier le contrat annoncé.

Aucune version 241 active à ce stade. La cause globale des anciennes pertes
JSONL reste ouverte ; séparation des vues et instrumentation ne valent pas
preuve rétroactive de tous les incidents.

## Qualification du candidat avant livraison — 7 novembre, vers 13 h CET

22 tests241 passent sur Mac (19,23s) et Linux (48,38s). Les scénarios comprennent
la vraie admission qui écrit le marqueur seulement pour provider_turn, les
sockets perdues/concurrentes, statut et annulation au milieu de4097 tâches,
la reprise jusqu’à Release après disparition du tour et de la connexion, le
nouveau fait de tour présenté au démarrage, et la sélection de cible avec outil
absent puis disponible. Aucun fournisseur ni mission métier lancé par ces tests.

La passe élargie exécute160 cas utiles :157 passent et3 attendus sont à corriger.
Les4 contrôles affectés repassent ensuite en3,09s. Deux attendus étaient
antérieurs à l’API courante : mission_reply_timeout_secs=3600 manquait et
ls-files était absent de la liste des lectures Git permises. Le témoin négatif
outside possédait une autorité dangerFullAccess : il porte maintenant une
restriction de dossier réelle. Les6 cas ignorés historiques restent ignorés.
26 régressions de reprise passent séparément ;51 du registre et36 de permissions
passent, avec leurs ignorés historiques déclarés dans les logs. Ces suites se
recouvrent : leurs totaux ne sont pas additionnés en un nombre de cas uniques.

Préparation Python :53 tests passent en33,34s, y compris alias local, source
hors Git, refus des racines de contrôle AVANT publication et rejeu sans mutation.
Les2 premiers échecs de fixture liés au TMPDIR macOS ont été corrigés en utilisant
/private/tmp ; aucune protection nofollow de production retirée.

Transport :403PASS/3FAIL/1ignoré au premier passage complet. La donnée réelle
montre que /usr/bin/python3 est le lanceur Xcode : CPATH/SDKROOT et d’autres
variables sont ajoutées, et le premier démarrage mesuré dépasse200ms. La fixture
utilise désormais sys.executable résolu, en argument séparé. La seule variable
d’exécution système macOS __CF_USER_TEXT_ENCODING est reconnue, comme LC_CTYPE.
Aucune règle d’environnement de production assouplie. Passe finale :406PASS,
0FAIL,1ignoré,41,90s, un seul thread de test.

Retrait du quota de volume : la carte publique est contrôlée AVANT le contenu.
L’ancien ordre lisait un fichier sparse de33Gio pour un manifeste déjà divergent,
observé à100%CPU pendant plus de deux minutes. Le seul processus de test détenu
a été identifié puis arrêté parSIGTERM ; son arrêt est prouvé. Le test corrigé
refuse sous une seconde. L’export réel640Mio passe ; le manifeste33Gio est
accepté et le dépassementu64 reste refusé/classé. Pas de prétendue copie33Gio.

Les erreurs compile rencontrées portent sur les raccords de signature, puis
sur deux fixtures cfgLinux oubliées ; elles sont corrigées. Les deux premières
recettes Linux prenaient Git2.39 du shellSSH. Le service possède déjà Git qualifié
dans .local/lib/bridget-tools/git-current/bin. La recette finale utilise ce même
chemin ; aucun programme ni réglage de service installé pour faire passer un test.

Revue R1 c962ab66 et R2 4c2b507d terminées, lues et ACK. Les corrections et
qualifications figurent dans les contrats et la décision241 ; une dernière
revue du delta final reste à faire. Les paquets privés sont immuables.

La cause de TOUS les anciens délaisJSONL n’est pas prouvée. Le couplage aux vues
est reproduit et corrigé ; les phases et durées de panne sont instrumentées.
Le bilan doit conserver cette limite et mesurer le parcours chargé après activation.
Aucun241 n’est encore activé. Le retour, Converge, audit, publication, Git et
nettoyage restent des portes ouvertes ; T3 n’a pas été redémarré.

## Mesures de parcours — recettes reproductibles, pas taux de fiabilité global

| Recette | Appels / issue observée | Identité conservée / intervention humaine |
|---|---|---|
| ACK d’inscription perdu avant commande | 1 coupure,2inscriptions,1mutation | Reprise dans le même délai ;0question |
| Refus d’identité explicite | 1inscription,0reprise,0mutation | Refus conservé ;0question |
| Inscription sans ACK | délai80ms ; pas de nouvelle connexion après expiration | Délai non renouvelé ;0question |
| Admission reçue mais réponse perdue | 2relectures concurrentes,1Task,1lancement simulé | Même clé/Task ;0question |
| Historique4097tâches retenues | 1lecture d’état,1annulation de la tâche cible | Historique gardé ;0question |
| Outil de commande | 1placement refusé outil absent,1qualifié outil disponible | Machine imposée gardée ;0admission et0question |
| Tour fini / tour suivant | 2cas admis ; révocations/annulation/historique distinctement refusés | Autorité de mission, pas nouveau tour ;0question |
| Vue divergente33Gio | refus avant lecture du contenu, moins1s | Vue conservée pendant preuve ; aucun export33Gio prétendu |
| Vue réelle640Mio | export/audit passent dans la recette existante | Source conservée ; aucun fournisseur |

Les délais de suite ne sont pas des latences de production. Les scénarios de
réponse tardive conservent les preuves240 : code du cache inchangé et huit cas
utiles acquis ; aucune nouvelle réponse métier tardive n’est fabriquée.

## Delta de revue R3 et contrôles finaux

R3 75f164a2 terminée/lue/ACK, APPROVE_WITH_CHANGES : deux P2 réellement
vérifiés sur les exigences de commandes. Correction des réserves liées à un autre
outil, des commandes enchaînées et des mutations manquantes. shlex2.0.1 déjà dans
le lock est réutilisé, aucun shell exécuté. Deux tests nouveaux avec tableaux
contre-exemples et vérification du routage. Premier passage 23/24 : l'ancien
attendu acceptait docker compose build en lecture seule. Le test vérifie
maintenant compilation refusée, puis compose ps accepté. Passage final24/24,
19,73s Mac. Les droits de production n'ont pas été assouplis.
Converge passage2 CONVERGED ; tasks.md conservé octet pour octet.
APK0.8.8/8008 construit en57s, signé avec l'association existante et filiation
vérifiée contre8007 ; SHA1ff61c007e38748acd74f779d1c40dfe7aa9a5f5d4131c38b112b0357b16ef19.
Ce fichier n'est pas encore publié ni annoncé installé sur un téléphone.

Dépendances : contre-revue44471d90 terminée/lue/ACK. Deux avis pulldown exigent
WIKILINKS absent de l'unique Parser configuré Options::empty ; deux avis Guava
portent sur les classpaths de build/tests, absents du runtime APK. Huit entrées Go
concernent le Caddy distinct inchangé et non inclus dans les sept composants241.
Elles restent ouvertes pour ce Caddy ; aucune déclaration de correction de ce
service. Le diff de deploy/caddy-security contre04ad est vide. Aucun risque humain
accepté inventé. Le rapport d'audit distingue cette exclusion de toute réfutation
du Caddy en production.

## Traitement de la revue R4

R4 d5ad10bc terminée/lue/ACK, APPROVE_WITH_CHANGES ; aucun P1. Les P2 proposés
sur les programmes non analysés sont écartés après lecture du contrat et du
consommateur : requires_workspace_write indique une écriture identifiée, pas
une certification d'innocuité. required_tools vide n'atteste aucun outil. Faire
de chaque script Python/Node ou wrapper inconnu une écriture obligatoire créerait
un nouveau refus pour les audits de lecture, contraire au périmètre retenu.
Le sandbox reste strictement inchangé ; l'analyse limitée ne garantit pas
l'aboutissement d'un programme inconnu. Sémantique rendue explicite dans le type,
le contrat et un test de non-élargissement. Aucun analyseur shell complet ajouté.
P3 utile traité : arête shlex redondante dans build-dependencies retirée ; le
build.rs existe et utilise sha2, sa présence n'était pas dans le petit paquet.
Le delta contre le paquet R3 était complet pour Cargo.toml absent de ce paquet ;
le vrai git diff contre04ad ne contient que l'arête runtime shlex. Preuves CLI
R3 conservées avec leur SHA ; R4 et le retour ont déjà passé la recette native
C19 sur les deux OS. Requalification CLI finale requise sur le dernier SHA.
Aucun nouvel agent de revue : les objections sont disposées avec preuve et limite,
sans boucle de réécriture du contrat pour satisfaire chaque hypothèse du relecteur.

## Qualification du candidat final241-r5

25tests241 verts Mac20,82s/Linux147,12s avec compilation Linux comprise dans
la durée de commande ; les durées de test seules sont dans les journaux.
Le25e test précise le sens du besoin d'écriture sans modifier le sandbox.
Production identique àR4 hors retrait d'une dépendance de build inutile ; type
annoté, aucun nouvel état réseau. BuildMac223,68s, SHA
3f1b7d5b1bbccbe2091e64d72fad134b005c8c9803b2685cd82bfd566132f3cc.
Recette binaire réelle C19 Mac : candidat15,89s, retour12,02s, aucune Task/modèle.
Compatibilité CLI du dernier Mac : forward/retour/inverse/clients compatibles ;
239et240 refusés comme retours. Les images n'ont pas encore été activées à ce point.
Audit ciblé :14contrôles/14 calculés, notes par axe et limites dans audit.md ;
les résultats bruts n'ont pas été assimilés à une qualité100%. Converge3 final
CONVERGED, aucune tâche de code ajoutée ; portes opérationnelles restent ouvertes.

## Activation, publication et nettoyage constatés

Source exécutabledb43c14254c2222c8ed82230c9a9f33392f37043 intégrée àmain et
poussée/vérifiée. Actif0.8.8/241-reliability-r5 : MacSHA
3f1b7d5b1bbccbe2091e64d72fad134b005c8c9803b2685cd82bfd566132f3cc ; LinuxSHA
019a82a61e1e639694f855d40ef3009a383429d88b4e9587db9c4fc3157f0fdd.
Sept composants/PID/images vérifiés : Cartae core2197363/mobile2197463/worker2200031 ;
SYS2605890 ; KS3913518 ; Macworker5363/pont5805. Coupurecore2,03s.
Retours241-reader-r1 compatibles sur lesquatre hôtes, pointeurs atomiques natifs
sous verrou et sauvegardes ; les anciens239/240 sont interdits comme retours241.
Cache de qualification des clients rafraîchi par le dernier binaire, sans modèle
ni authentification fournisseur revendiquée. Quatrerapports0.8.8/up_to_date/fresh.
Septpayloads protégés strictement identiques avant/après ; superviseurCartae
2500061/birth270585460 conservé, registreMacSHAcacf9ba0 inchangé, septprocessus
d’applicationT3 strictement inchangés. Aucune missionmétier rejouée.

Releasepubliquev0.8.8/sixactifs téléchargés sansauth et SHA identiques. Android
8008/certificat/filiation8007 vérifiés avant publication ; pas d’installation
ni recette graphique sur téléphone revendiquée. Caddydistinct non modifié.
Cinqavis avant et cinqavis après/diagnostics ont des reçus in_flight ; la lecture
par les destinataires n’est pas déduite de ces reçus.
DixlecturesMCPsame_project réussies après passage :1553–2207ms, aucun refus/avertissement.
DixlecturesCLIglobal réussies :685–1007ms ; elles gardent l’avertissement
project_unknown du client CLI sans identité de session. Ce n’est pas une preuve
de qualification de projet ; le lien natif de ce fil rend bienbridget/sanswarning.
Les essaisCLIwho--json puisupdatesstatus--json étaient des erreurs d’arguments
de qualification ; usages corrigés, aucune mutation. La première tentative de
rafraîchissement du cacheclientMac a rencontré le verrou avant fin du watcher ;
aucun état écrit, arrêt natif confirmé puis writer exécuté unefois, autres champs
strictement conservés. La cause de tous les anciens délaisJSONL reste ouverte.

Nettoyage :54copies/binaires identifiés,1745159157octetslogiques retirés ; ce
compte ne prétend pas mesurer des octets physiques libérés sur un volume à snapshots.
Worktree/branche241Mac et source-build241SYS retirés, code intégré et usages ciblés
vérifiés. Audit, anciennesrévisions, résultatsbruts, paquetsderevue et archive
source sont conservés dans le cacheprivé241. Aucun fichier temporaire métier
ni vue native sous garde n’a été retiré.
Les caches de compilation restent conservés : Mactarget13060168Kio, SYS227cache
22436132Kio. installed a été appelé via build.py ; le nettoyeur refuse car son
observation globale est incomplète. MesureMac :25enregistrements non qualifiés,
descripteursrévoqués et cwd/rtd indisponibles de processus encore vivants. Pas
de conclusion erronée « processus terminés », pas de kill d’autretravail ni de
contournement du nettoyeur. Les caches/repos/vues historiquesSYS et leWTmodifié
Cartae1cc44f4d restent conservés pour leurs états/usages non clôturés/qualifiés.
Le miroirKS gardait un origin pointant un ancienbundle/tmp absent : fetch
explicite depuis leGitHub vérifié puis avance rapide, sans réécrire ses réglages.
SYSrepo propre a rejointmain après vérification de l’absence de commande native
active ; les anciensworktrees distincts restent intacts.

## Client encore ouvert — T025

Le lien natif est revenu : bridget_sessionV2/permissions provider_turn et
who same_project réussis, projetbridget et aucun warning. La façade95234 de ce
fil charge encore240, confirmée par ps/lsof ; d’autresfaçades237/240 sont utilisées
et leurs images gardées. Aucun remplacement de l’appT3. Le code de son adaptateur
prépare le MCP lors d’une reprise native de fil ; aucune commande de reload MCP
n’est exposée par ses outils. Une continuation native de ce même fil, sans
redémarrage d’app, est la dernière vérification prévue. Son effet n’est pas
présenté comme acquis. Statut globalInProgress tant que cette porte reste ouverte.
