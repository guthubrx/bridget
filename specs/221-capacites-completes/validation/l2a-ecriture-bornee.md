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
