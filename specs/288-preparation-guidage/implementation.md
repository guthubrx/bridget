# Refus de préparation SYSLE2

La tâche 97d5b971-e272-4e48-99dd-e048351f514a est annulée, remote_prepared=false, workspace=null. Le journal bridget_prepare_capacity corrélé donne cause=held_bootstrap, held=1, prepared=0, prepared_limit=128. La garde de démarrage refuse avant la création du worktree. Le stockage courant dispose de 36 Gio sur /home et 6,3 Tio sur /mnt/storage ; ces mesures actuelles ne sont pas une preuve historique de capacité.

La continuation 0345 reste released/native_ready au contrôle. Aucun arrêt, drainage, changement de limite ou rejeu de la tâche annulée.

Correction minimale : le refus indique l’événement exact, les champs et leur interprétation. Le fil protocolaire reste inchangé ; cause=unobserved est conservé tant que le reçu ne porte pas de preuve. Le test vérifie le parcours task_view, la conservation de la tâche et l’absence de projection pour une autre erreur.

Les anciennes façades décrivant search sans rôle restent celles non rechargées ; le correctif 287 existe déjà. Aucun second correctif ni redémarrage de T3 pour ce point.

Le journal corrélé est daté de 16:27:04.796800 Europe/Paris. Les preuves sont conservées dans /Users/moi/.cache/bridget288-preparation/incident97-capacity.json. La recette reconstruite correspond octet par octet aux quatre fichiers du runtime cumulatif. Aucun build du checkout principal.

Revue indépendante non relancée : le refus précédent de tous les profils de lecture seule du dépôt reste non résolu. Aucun profil ou droit changé pour le contourner. Relecture directe du delta et des appelants effectuée.

Deux tests distincts passent sur Mac et Linux : spec288 (projection et diagnostic fermé) et spec172_remote_admission_error_conserve_identifiant_erreur_et_rejeu (conservation des identifiants et du rejeu). Clippy strict Mac passe. L’APK 8052 est construit et sa signature comparée à 8051. Aucune validation graphique requise : changement de diagnostic technique.

Clippy strict et release Linux passent, ainsi que release Mac. Le contrôle ABI du binaire Linux passe sur la cible Debian 12. Les qualifications exécutent exclusivement scripts/build.py ; aucun échec de test.
