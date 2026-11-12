# Refus de préparation SYSLE2

La tâche 97d5b971-e272-4e48-99dd-e048351f514a est annulée, remote_prepared=false, workspace=null. Le journal bridget_prepare_capacity corrélé donne cause=held_bootstrap, held=1, prepared=0, prepared_limit=128. La garde de démarrage refuse avant la création du worktree. Le stockage courant dispose de 36 Gio sur /home et 6,3 Tio sur /mnt/storage ; ces mesures actuelles ne sont pas une preuve historique de capacité.

La continuation 0345 reste released/native_ready au contrôle. Aucun arrêt, drainage, changement de limite ou rejeu de la tâche annulée.

Correction minimale : le refus indique l’événement exact, les champs et leur interprétation. Le fil protocolaire reste inchangé ; cause=unobserved est conservé tant que le reçu ne porte pas de preuve. Le test vérifie le parcours task_view, la conservation de la tâche et l’absence de projection pour une autre erreur.

Les anciennes façades décrivant search sans rôle restent celles non rechargées ; le correctif 287 existe déjà. Aucun second correctif ni redémarrage de T3 pour ce point.

Le journal corrélé est daté de 16:27:04.796800 Europe/Paris. Les preuves sont conservées dans /Users/moi/.cache/bridget288-preparation/incident97-capacity.json. La recette reconstruite correspond octet par octet aux quatre fichiers du runtime cumulatif. Aucun build du checkout principal.

Revue indépendante non relancée : le refus précédent de tous les profils de lecture seule du dépôt reste non résolu. Aucun profil ou droit changé pour le contourner. Relecture directe du delta et des appelants effectuée.

Deux tests distincts passent sur Mac et Linux : spec288 (projection et diagnostic fermé) et spec172_remote_admission_error_conserve_identifiant_erreur_et_rejeu (conservation des identifiants et du rejeu). Clippy strict Mac passe. L’APK 8052 est construit et sa signature comparée à 8051. Aucune validation graphique requise : changement de diagnostic technique.

Clippy strict et release Linux passent, ainsi que release Mac. Le contrôle ABI du binaire Linux passe sur la cible Debian 12. Les qualifications exécutent exclusivement scripts/build.py ; aucun échec de test.

## Activation et limite restante

Sources de correction : 81f3e8e6e1f8a7b889fb98e3ab5939d91a66cafb. Release publique v0.8.52 : six fichiers téléchargés anonymement et comparés au manifeste. Le statut natif confirme build_id=288-preparation-guidage-r1 sur Cartae. Core, mobile et worker Cartae, worker KSSTORE, worker Mac, connecteur et outil de mise à jour Mac, worker M1 sont chargés et leurs empreintes vérifiées. Les pointeurs de retour sont 287, avec compatibilité et empreinte contrôlées. M1 n’avait pas d’état privé de mise à jour ; aucune structure artificielle créée.

Le premier passage maître a été retenu par une admission en cours, avant arrêt. La relecture a constaté la fin de la transition ; la bascule a ensuite duré 3,58 s. Quatre superviseurs Cartae et trois superviseurs Mac ont conservé leur PID et leur naissance. T3 conserve le PID 77762, né le 11 novembre à 14:10:41. Le premier statut natif après reprise a rendu primary_absent ; la seconde lecture a confirmé le maître 288, sans recréer de mission. HTTP mobile : 200.

L’activation SYSLE2 reste incomplète : deux contrôles d’installation ont refusé la lecture de version Claude en délai. Une mesure séparée réussit avec version 2.1.310 après 6,69 s ; le contrôle natif impose 2 s. Aucun problème de compte ou de compatibilité n’est déduit, aucune garde modifiée. Le worker 109486 reste chargé en 287, retour 286, sans activation pendante. Le binaire candidat et les intermédiaires Linux sont conservés pour la reprise. La correction de guidage est portée par le maître 288 et fonctionne aussi avec le protocole inchangé du worker 287.

Ménage exécuté : 1,15 Gio logiques d’intermédiaires Mac retirés par la commande installed. Trois anciennes images inutilisées ont été retirées sur Mac, M1 et KSSTORE. Les images Mac 266/281/284/285 restent utilisées ; les anciennes images Cartae sont gardées faute d’observation privilégiée complète. Aucun travail métier, dossier de revue ou worktree actif retiré. Le checkout principal n’a aucune branche de travail ni worktree auxiliaire.

Preuves locales : /Users/moi/.cache/bridget288-preparation. Les compteurs de suppression sont des tailles logiques, pas une mesure d’espace physique récupéré sur APFS. La livraison complète à toutes les machines ne peut pas être déclarée tant que SYSLE2 n’a pas franchi son contrôle.

## Incident de maintenance découvert après activation

La tâche a716ff89-0c56-4eaa-9f21-b91dcc35b24c, admise le 12 novembre à 16:50:29 Paris, a échoué à 16:50:44 avec native_sender_primary_absent. Le connecteur a redémarré à 16:50:41 ; le parent Territoire a été réexposé à 16:51:00 seulement. Le maître lui-même n’a redémarré qu’à 16:51:19. Le processus enfant 97364 figurait parmi les processus conservés, mais la remise initiale a échoué pendant l’absence du parent. La vérification des PID n’était donc pas une preuve de conservation des missions.

Preuves : aucun tracked_request ni événement ledger pour la mission 2b1e7e20-a67f-43e4-bb07-f2d72ab7f287 ; delivery_acked=false ; cleanup_done=true ; enfant présent dans cancel_stopped. Worker terminal à 16:50:54, PID absent. Placement shared, aucun worktree propre à retirer. Le commit b0e7bfdc64dac056c963484a206796217e781ce7 est présent. Le lien parent est revenu et observé busy. Aucune nouvelle tâche n’a été lancée par la maintenance.

Le parcours ponctuel de redémarrage du connecteur est suspendu dans /Users/moi/.cache/bridget288-preparation/deploy-mac281.py. Une relecture ponctuelle des admissions ne ferme pas la course entre contrôle et arrêt. La protection des admissions pendant cette coupure reste à corriger avec le mécanisme natif de maintenance ; aucune relaxation de l’attestation du parent ni réparation directe de la base n’est acceptable. Ce défaut n’est pas déclaré corrigé par le lot 288. Preuve bornée : /Users/moi/.cache/bridget288-preparation/territoire-initial-dispatch-incident.json.
