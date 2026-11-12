# Placement partagé et carte de reprise

Le placement et la première commande de la tâche 8bd1d8b8 étaient conformes
au dossier et à la base demandés. La carte ajoutait pourtant une interdiction
globale du checkout principal et exigeait un worktree. La seconde commande de
l’enfant sélectionne explicitement un ancien worktree. Le contrôle métier
refuse ensuite la publication : 270 entrées observées, 282 attendues.

Le correctif remplace la consigne globale par un constat Git sans autorité.
Il renvoie aux consignes de la mission et du dépôt, interdit de déduire un
changement de placement de cette carte et demande un arbitrage sur un conflit.
Il ne modifie aucune permission, aucun placement ni contrôle de base.

La revue indépendante a été refusée avant admission :
command_execution_unavailable et definition_unavailable, aucune tâche créée.
Aucun changement de profil, permission ou route n’a été tenté.

Preuves privées : /Users/moi/.cache/bridget286-placement.
Sept tests ciblés passent sur Mac et Linux, ainsi que Clippy strict workspace/all-targets.
L’APK 8050 est construit et sa signature est vérifiée contre la version publiée précédente.
Les builds release Mac et Linux passent. Le Linux demande glibc 2.34,
compatible avec Debian 12 (maximum 2.36). Activation vérifiée sur les services concernés.

La copie cumulée de qualification diffère de main : application du patch de
main refusée à sec sur les tests indentés. Un patch propre au runtime a été
produit puis sa recette reconstruite et comparée octet par octet. Aucun
fichier de production ni tâche n’a été modifié par cet échec.

Le formatage global du runtime signale des écarts préexistants, notamment
l’indentation de son module de tests. Aucun succès global de formatage n’est
revendiqué. Le nouveau test isolé passe rustfmt et le diff passe git diff --check.

Le premier essai Android local échoue avant compilation : JDK dédié absent
sur le Mac. L’environnement Android existant sur Cartae est réutilisé ;
aucune installation ni configuration d’outillage n’est modifiée.

## Livraison vérifiée

Code poussé : c0736a1d0fb0782d6b4a3aa6d3e9267c04454f8a.
Version publique : https://github.com/guthubrx/bridget/releases/tag/v0.8.50.
Les six fichiers publics sont téléchargés anonymement et vérifiés par empreinte.

Coordinateur et mobile Cartae, workers Cartae/SYSLE2/KS/Mac/M1, connecteur et
outil de mise à jour Mac sont chargés. Images de retour 285 et pointeurs privés
vérifiés ; M1 n’avait pas d’état privé de mise à jour, aucun n’a été inventé.

- Mac : 16a7943e06b4c13b6c565c05eaa1ab32d2d5bdb0319b1a5c2cf10d0b3a3e5be4.
- Linux : 4f8aa1c19f37f9387efd6053dce23534a27a2c46a1c2c8f00983869bfc8f1e50.
- APK : ac522722aa8ad47b20807be1ba35f32a550146f5432636260dfc2ec73926b1c3.

La garde a refusé la première bascule avant tout arrêt, une admission ayant
commencé entre deux contrôles. Une lecture suivante sans transition permet
la bascule. Coupure du coordinateur : 3,79 s. Les processus métier contrôlés
restent identiques ; le contrôle n’est pas un verrou atomique sur les admissions.

Le lien natif répond avec 286-placement-carte-r1 et le mobile rend HTTP 200.
T3 garde PID 77762, né le 11 novembre à 14:10:41. Les anciennes cartes sont
historiques ; les wrappers déjà vivants restent conservés et peuvent garder
l’ancien texte. Les nouvelles cartes chargent la correction. Aucun agent métier
n’est redémarré et aucune issue inconnue n’est remplacée.

Les sept propriétaires de missions concernées reçoivent un avis silencieux
avant et après ; le demandeur reçoit le diagnostic. Les reçus stored ne
prouvent pas la lecture. La reprise corrective métier reste intacte.

## Ménage gardé

Caches retirés : 1,15 Gio Mac et 0,70 Gio debug SYSLE2, tailles logiques arrondies.
Le release SYSLE2 reste conservé : descripteur 9 du worker PID 88903 non
qualifié par le lecteur natif. Aucun arrêt d’autrui pour nettoyer.
Anciennes images retirées après vérification : Mac 74967904 octets,
M1 74973984, SYSLE2 90729216, KS 90719160. Cartae conserve les images dont
l’observation d’usage reste incomplète ; Mac conserve les images encore utilisées.
Les versions active et de retour sont conservées.
Les copies temporaires locales et la reconstruction de recette retirées totalisent
201718725 octets. Le rapport de ménage ne trouve aucun worktree ou branche
éligible. Aucun worktree nouveau n’a été créé pour ce lot.
