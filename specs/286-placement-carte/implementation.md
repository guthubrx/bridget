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
compatible avec Debian 12 (maximum 2.36). Activation en cours.

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
