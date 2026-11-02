# Validation du correctif226

Cause constatée : snapshot expiré ferme tous les liens, journaux Mac du2novembre05:30–05:43UTC. Comportement introduit par9356b0f8 le14septembre, antérieur à0.7.4.
Sources c9c103f1/c6160be5. Quatre tests ciblés R2 PASS, 2744 autres non relancés.
Premier essai de fixture lisait une trame initiale au lieu du heartbeat : corrigé par lecture des trames réelles. Le renforcement de fixture utilisait une chaîne au lieu de ObservationKind : erreur de compilation corrigée. Aucun de ces essais non passants ne vaut preuve.
Revue757976fd : aucunP1, P2heartbeat horsgrâce corrigé par événement dédié SnapshotTimeout. P3connexion/réponse distingué. Delta relu par le responsable, sans nouvelle campagne. Reçu lu/ACK/gardeclose ; vue conservée tant que les usages restent incomplets.
Bornes :90s depuis début du dernier succès, PID/port/naissance identiques. Aucune autorité ou observation conservée ; dispatch relit l’état. Pas de retry RPC nouveau ni de mission métier rejouée.
Assemblages Mac/Linux portables R2 et activation : à compléter.
Limite séparée : rôle mainteneur bridget non transversal, mandat vide. Décision humaine demandée sans changer les permissions.

## Remise lente et version finaleR3
Trace71a4016c : dispatch interrompu à15,319s après timeoutclient10s ; transaction de lecture13,505s, dont deux extractionsJSON5,439s et3,533s. Pas de reçuT3 pour les remises historiques contrôlées ; elles restent indéterminées, jamais recréées.
BaseT3 contrôlée25 766 764 544octets ; fil relancemonde73 462éléments/61 843nœuds ; payloads393 217 059/59 822 766caractères. Accès par index(thread_id) vérifié. Coût de matérialisation historique, pas indexmanquant démontré. Optimisation côté serveur reste ouverte ; aucun changement de sa base.
R3 porte uniquement message.dispatch à60s, autresRPC10s. Client cloné sous mutex, réseau de remise hors mutex, incarnation réévaluée. Sept tests ciblés finaux PASS0,03s ; test de non-blocage, absence de retrytimeout et résultat inconnu après changementruntime inclus.
Revue6ec2f7b6 : pasP1. Trois réserves résolues : renew(generation) renvoie déjàOk si un autreappelant a renouvelé ; liveness300s dépasseRPC60s ; RpcSocket fixe lecture/écriture au temps restant de la deadline et la remise réelle suivante en prouve le passage au-delà de10s. Aucun troisième correctif exigé par ces réserves conditionnelles. Concurrence interliens assumée, un dispatchparlien et files bornées conservées ; pas de mesure de charge globale annoncée.
Mac/Linux portables compilésR3. Sept images chargées : Macworker88030/pont88113 ; Cartaecore246859/mobile246899/worker248405 ; SYS1528907 ; KS1981892. Empreintes Mac4bd5ae6ed3c3c4474f099d4d746afb29be414acdd035573e4c3586da5d7f844b / Linux7794a477407bff17d966bbd07d88052e3a42dad1a9ab671117795d648b9e683d. Deux passagesR2/R3 distincts, R2n’était qu’intermédiaire. Coupuresmaître2,03s/1,94s. SuperviseurCartae2500061 même naissance conservée ; marqueurs et réservations sauvegardés.
Retour225-r4 compatible et rollbacknatifs écrits atomiquement sous verrou sur4hôtes, aucunretour exécuté. RegistreMac namespaceinchangé.
Remise technique8700e0e2, clébridget226-after-r3-20261102-01 : ACK1793601354, séquenceT34890009 après environ30s. Pendant l’attente, timeoutHTTP06:35:36 garde les liens, lectureauditeurs expirée06:35:46 sans effacementglobal. Nouvelle notification auditeurs25a0b3c7 conservée, réception à compléter. Cela n’atteste pas la reprise métier de la demandeV4.
APK0.7.5/code7005, certificat et lignée existants conservés, gardepublicationPASS. Androidsanschangementfonctionnel depuis225 ; pas de nouvelle recetteUI/FCM.
