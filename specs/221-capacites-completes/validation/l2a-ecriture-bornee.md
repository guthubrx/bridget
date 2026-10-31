# Écriture bornée du catalogue — T182

État : développement. Le test Mac r36 du lot C19 est resté dans send. Son
processus possédé a reçu TERM après identification ; arrêt confirmé et entretien
du wrapper exécuté. La suite interrompue n'est pas un PASS.

Deux reproductions Python avec alarm(2), sans Bridget : send de65536octets
sur paireUnix bloquante, puis troisième send de4096octets, restent en attente
malgré MSG_DONTWAIT|MSG_NOSIGNAL. Une sonde avec SO_SNDTIMEO=1ms renvoie
8192octets en1,28ms puis EAGAIN en1,27ms. Ce sont des mesures locales,
pas une garantie de planification temps réel.

Le source Apple uipc_socket.c (sosendcheck) vérifie SS_NBIO ou MSG_NBIO
pour l'attente de tampon ; MSG_DONTWAIT ne couvre pas ce chemin.
Référence primaire : https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/uipc_socket.c
La version du noyau installé n'est pas rattachée au commit de cette référence.
Les reproductions locales établissent le comportement de la machine.

Correction : délai SO_SNDTIMEO de1ms pendant les seuls envois métadonnées
sur Mac, sous le verrou partagé des writers, puis restauration du délai.
O_NONBLOCK n'est pas modifié : les clones de lecture conservent leur mode.
Échéance250ms vérifiée aussi après succès partiel ou interruption. Si la
restauration échoue, le canal est fermé ; une ligne partielle n'est pas rejouée.
Linux conserve MSG_DONTWAIT. La fixture remplit sa paire en mode nonbloquant
puis rétablit le mode bloquant avant de tester la vraie fonction.

Le test vérifie le mutex occupé, le tampon plein, la ligne partielle,
la restauration du délai existant, l'absence de changement O_NONBLOCK
et une ligne nominale décodable. Tests et livraison encore à établir.

Ancien clientMCP214 : credential_mismatch après redémarrage du pont.
La revue native reste indisponible tant que l'application appelante n'est
pas complètement quittée puis rouverte sur le même fil. Aucun autre moteur
ou accès fournisseur ne remplace cette revue.

## Vérifications source finales
Linux : /home/moi/.cache/bridget221/socket-linux-manifest-r3.log
34daemon+2transport PASS,3recettesprivées ignorées ; manifeste explicite de
ce worktree confirmé dans le log. Premier Linux via le script principal sans
manifest explicite compilait main : résultat exclu de la preuve nouvelle.
Deux dépendances transport de sources différentes ont aussi été mélangées
dans l'ancien cache ; ce refus de compilation est conservé. Les essais de
ce lot utilisent le cache enregistré bridget-build-221-delivery, commun à
tous ses contrôles, distinct du protocoleC19 non intégré. Aucun target copié.

Mac : /Users/moi/.cache/bridget221-ecriture/tests-mac-short-tmp.log
30daemon+2transport PASS,2recettesprivées ignorées. La première passe globale
échouait sur les chemins temporaires trop longs ; TMPDIR=/tmp/b221-socket
rétablit le harnais sans modifier la protection de chemin produit.
Le premier correctif de délai révélait EINVAL après shutdown ; restauration
avant fermeture vérifiée après observation réelle du comportement du noyau.

Recherche de variantes : les autres MSG_DONTWAIT des crates ont été relus.
Un recv et une fixture déjà nonbloquante ne sont pas affectés ; la fixture205
de saturation a reçu le même remplissage nonbloquant, rétabli avant le vrai
test. Ses4tests passent surMac/Linux. Aucun autre writer de production fondé
sur ce seul drapeau n'a été trouvé dans ce périmètre.

AuditV15 source /home/moi/.cache/bridget221/audit-socket-r3 :
finalize pass/release_blocked=false et validate true ;12/12contrôles applicables.
Semgrep et Gitleaks exécutés sans candidat. Les notes par axe et leurs fractions
figurent dans axis-scoring.md ; cette preuve n'est pas un score produit global,
une revue indépendante ou une qualification d'activation.
Revue native empêchée par le clientMCP214 encore chargé, credential_mismatch ;
aucun remplacement par CLI/T3 ou sous-agent. Limite explicite, pas d'avis inventé.

Analyze et convergence ciblés : T182 reprend le mécanisme d'envoi existant,
aucune nouvelle autorité ni dépendance ; contrat, tests et comportement
cohérents. Aucune autre catégorie n'est close. Convergence source bornée,
livraison/activation/nettoyage encore requis avant clôture de T182.

## Livraison effective —31octobre
Source exécutable0a0d3950e2002a78029fc7ad14ec8f87e10ad8db intégrée et poussée.
Build221-write-0a0d3950, version commerciale0.6.5 inchangée.
Linux40016792octets, SHA256c8986a783c1edd48ca5dbda6f6fa5e3a04bb2073f533a4a4b19eb8eac19b336a,
GLIBC maximale2.34 vérifiée sur la construction SYS.
MacARM64 SHA256b1e6eb586505ace3597d2ea61d45e4ba533d034503ff2b130ac88d7d93b286b2,
signature codesign stricte vérifiée.

Images chargées et hachées : maîtreCartae466954, workerCartae468097,
mobileCartae467170 ; workerSYS822239 ; workerKS1232171 après reconnexion.
WorkerMac82168 et pontMac82266 vérifiés par lsof ; updaterMac82723 et
updaterCartae467201 relancés par leurs unités natives. PID datés, pas des
identités durables. Les contrôles clients locaux et l'ancien validateur ont
accepté leurs inventaires :2Cartae/2SYS/1KS/4Mac avant bascule.

SauvegardesSQLite cohérentes et stores locaux, arrêts natifs, version de retour
et pointeur updater conservés. Coupure maître2,47s. WrapperCartae2500061,
naissance270585460 conservé ; aucun service métier, compte, droit ou ancienne
mission relancé. Les agents monuments/trajets/Politique/Roadie ont reçu
les avis avant/après dans leurs fils existants ; aucune réponse demandée.
Le canalBridget de ce fil refusant credential_mismatch, ces avis ont utilisé
les filsT3 natifs existants autorisés, sans délégation ni nouvelle mission.

Port natif CLI :15connexions lues, nouvelles observationsCartae/SYS revenues,
aucune contrainte d'arrêt de sonde. KS reste désactivé selon décision existante ;
les deux observationsCodexMac restent not_observed, sans cause inventée.
La première invocationCLI omettait --catalogue et a été refusée localement
MISSION obligatoire ; corrigée une fois selon l'aide, sans admission.
ClientMCP appelant58296 toujours214-clients-30581503 : composant ancien
explicitement conservé sous usage, T177ouverte. Réattester/reprendre un tour
ne remplace pas cet exécutable : quitter complètement T3Code avec Cmd+Q,
rouvrir puis reprendre ce même fil. Aucune boucle, copie de secret ni
reprise d'une ancienne mission pour forcer la connexion. Services actifs
ne signifie pas tous les clients externes rechargés.

## Nettoyage et état du programme
installed exécuté pour le cacheMac et la constructionSYS. Caches logiques
Mac6,41Gio/Cartae5,04Gio/SYS0,57Gio, sous10Gio ; budgets et gardes conservés.
Cartae : copie de transfert de40Mo et ancien paquet221-stop-guide retirés
après empreinte, inventaire complet, contrôle des liens et références courantes.
Le paquet actif221-write et le retour221-catalogue vérifiés sont conservés.
Mac/KS : copies et ancienne image gardées, observation de processus incomplète.
SYS : ancienne image gardée, process_access_unavailable. Aucun nouveau
privilège ajouté. Images historiques encore utilisées ou non entièrement
qualifiées ne sont pas supprimées.

Worktree local221-ecriture-bornee et sa branche retirés normalement, sans force,
après intégration, Git propre avec ignorés absents et inventaire complet frais.
Un premier contrôle incomplet avait refusé ce retrait ; l'observation suivante
complète a permis cette seule action. Les revues historiques, preuves,
archives et le worktreeC19 encore actif sont conservés.

T182close ; programme35/182InProgress,147tâches ouvertes. T177, C19 et les
autres catégories ne sont pas déclarés achevés. Les développementsC19 privés
ont24fichiers sauvegardés et empreintés ; Linux114daemon+4transport,
Mac110daemon+4transport PASS et22tests updater PASS, avec limites privées
ignorées indiquées dans c19-conception.md. Aucun de ces résultats ne qualifie
une admissionC19 publique, un fournisseur ou tous les formats conservés.

### Notes de l'audit source ciblé
| Axe | Note observée | Critères notés/applicables |
| --- | --- | --- |
| Sécurité |75/100|3/3|
| SécuritéIA |non applicable|0/0|
| Fiabilité |75/100|1/1|
| Tests |67/100|3/3|
| Ressources |69/100|4/4|
| Complexité |75/100|2/2|
| Qualité |75/100|2/2|
| Architecture |75/100|2/2|
| Responsabilité future |67/100|3/3|
| Minimalisme |75/100|3/3|
| Métier |75/100|3/3|
| Traçabilité |63/100|2/2|

Pas de moyenne produit. Les sous-contrôles et raisons de non-applicabilité
figurent dans /home/moi/.cache/bridget221/audit-socket-r3/axis-scoring.md.
La lecture est celle du responsable, pas une contre-revue indépendante.
Preuves d'interface : non requises, correction purement technique.
