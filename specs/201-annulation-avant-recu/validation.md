# Validation201

Cause6231 confirmée en lecture opérateur : Task cancelling, commande/instance/génération et bail durables exacts ; aucun reçu commande worker, reçu workspace de préparation présent, mission_issued_at null. Le Cancel183 refuse UnknownExecution ; cette absence ne prouve pas un arrêt.

Correction : après ce refus seulement, vérifier la tâche actuelle cancelling sans mission, remote_prepared, tuple et placement depuis l’autorité durable, négocier StartRecoveryV1, puis FenceUnstarted sur la même commande. Attendre StartFenced et relire l’état avant le parcours de clôture Fleet existant. Aucun Start/Prepare/Release ni remplacement. Causes fermées dans le statut ; identité absente ou pair ancien conserve cancelling.

5tests201 PASS : handler Cancel→UnknownExecution→Fence→cleanup/guard release sans spawn ; gardes état/mission/identité/capacité ; refus capacité/réponse indisponible ; changement d’état pendant E/S ; guidage fermé et cause historique inconnue.26tests198 PASS, dont worker réel du fence durable/Start tardif/quota/redémarrage et annulation après fence/reprise Fleet. Tests via scripts/build.py, cache commun Cartae, jobs2/nice10. Avertissements préexistants conservés ; entretien différé car observation des processus incomplète, aucune purge forcée.

Ces tests ne chargent pas de binaire actif et n’attestent pas la clôture6231. Maître183/PID351206 et workerCartae/PID35887 restent en ligne. Deux missions métier réelles SYS/KS encore working au relevé19:18UTC ; demande propriétaire unique de clôture au prochain point sûr, sans interruption ni nouveau gel. Revue/candidat/chargement encore ouverts.

La revue4adfc814 relève un P2 réel : ResourceLimit n’atteste pas une incapacité de stockage. Complément : préserver les causes locales constatées master_pending/master_request_frame, sinon resource_limit_unobserved avec code technique et vérification manquante. Aucun changement du fence ou des décisions d’arrêt. Test ajouté avec table maître pleine : zéro trame worker envoyée.

Complément final :6tests201 PASS après guidage corrigé ;13tests de l’updater PASS, dont refus sans restart pendant mission active et exclusion des activations concurrentes. Aucun nouvel effet d’arrêt ajouté. Revue complémentaire et reconstruction à poursuivre sur cette tête exacte.
