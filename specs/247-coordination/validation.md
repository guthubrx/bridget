# Validation247

Source candidate :b784e71be3f95b31ac101383995e0cc4b9d6cc8b.0.8.13/247-coordination-r1, retour0.8.12/246-quota-accord-r2 ; aucun nouveau format de stockage.

## Faits et dispositions

- Revue245R1 : P2 déjà corrigé dans bc8b9d6d avec scénario panneSQLite → réponsepositive → redémarrage. Revue245R2 inconclusive, invalid_envelope sur lecture contrôlée ; journal historique indisponible, cause exacte inconnue. Aucun nouveau verdict indépendant inventé, aucune revue ancienne rejouée. Le fichier existait dans la vue ; le refus d’un port de matérialisation ne prouve pas une absence ou un refus de lecture locale de ce fichier. Les briefs suivants rappellent de lire directement les fichiers présents.
- Diagnostic246 et revue246R1 : reçus tardifs déjà lus/ACK ; correctionLineage chargée et capture après confirmée par François. Pas de nouvelle mission pour ces avis.
- Parixsteer a5ba5e7f : cross_project_reason_required puis envelope_mismatch réellement reçus. Changer le motif sous la même clé change l’enveloppe. Le projet logique d’un enfant natif ne doit pas être déduit de la seule différence Mac/Cartae. Le nouveau helper vérifie parent, deux instances vivantes, tâche active et révocations ; il exclut la remise exacte de mission. Préparation de message et guidage réutilisent ce fait. Le sommeil reste vérifié avant toute admission ordinaire. Aucun envoi a5ba rejoué.
- ParixShared89d2 : refus attendu pour le changement deHEAD du partagé ; P1 actif dans un worktree descendant. Les pistes maître ne prétendaient pas identifier le détenteur worker. Le guidage explique la protection et propose une vue isolée seulement si ce changement est autorisé et l’absence d’admission confirmée. Aucun déverrouillage, arrêt deP1 ou nouvelle recherche ; la recherche T3 distincte reste sur son parcours.
- Parix650c : cancelled/errornull, mais demande60f8 encoreopen observée. Un DeliveryRejected tardif passait au message générique « Échec ». Il est maintenant classéStaleIgnored sans changer la tâche. Le tick ferme le suivi ouvert seulement avec cancelled et correspondance mission/senderowner/targetchild, par Store.cancel_request. Les pannes SQL restent réessayables ; aucun ACK de livraison ou arrêt physique inventé.
- Revue148b28c : état durablecancelled/errornull observé,updated1794147462. Présencee418 connected reconstatée ; diagnostic envoyé. Cause historique deprimary_absent inconnue, pas de causalité commune avec le délai de message affirmée. Livraison4c2a/idmcp-60763-6af07d54-1 : phaseindeterminate,acked_atnull ; suivitimed_out,created1794145621,deadline1794145920,completed1794145922. Ces deux registres sont distincts ; aucune preuve de réception ou d’absence d’exécution déduite. Aucun replay ni remplaçant.

## Contrôles

Quatre contrôles247 PASS Mac et Linux : coordination native distante, anciennes instances/remise exacte/clôture/révocation, sommeil conservé, refus après annulation et clôture logique idempotente/sender étranger refusé, guidage de clé. Huit tests existantsnative174 PASS Linux, dont l’interdiction de réutiliser l’autorité d’une mission dans un envoi direct. Les répétitions ne sont pas additionnées.

Un premier test comparait l’objet avant normalisation de save à sa ligne relue : fixture corrigée en capturant la vraie ligne persistée. Ajout de gardes négatives : erreurs de types de fixture corrigées avant le succès final, sans modification fonctionnelle pour les faire passer. Aucun test général relancé.

Revue indépendantef577942d terminée, lue et ACK : aucun P1/P2 confirmé sur le candidat, lecture statique uniquement. Aucun test indépendant ni note V15 inventés.

## Activation réelle

Sept composants/SHA chargés sur les quatre hôtes : Mac302859af673738d3a5eefb362079de27deccb8275f5ab45e2bd4e6487df8a5df ; Linux55561e90795bb606ff30c09eaef11a753d66e20d50d42e2360885d07d4b551f0. Retour246-r2 qualifié et pointeurs rollback écrits atomiquement sous verrous natifs ; formats aller/retour/inverse compatibles, ancien245 refusé. Coupurecore1,86s. T3 principal82837 et serveur14674 ainsi que67156/74516 conservent PID et date de démarrage. Registre spécial Mac et sept payloads protégés identiques avant/après.

Le suivi réel60f8 est devenu cancelled à1794150474, cancel_reason=mission_native_cancelled, après le tick du correctif. Aucun envoi métier de qualification, aucune annulation supplémentaire ni modification des tâches étrangères. Attestation nativeV2 et annuaire same_project live sans avertissement après le passage. Les anciennes façades ouvertes restent préservées ; le correctif de coordination est côté serveur et n’ajoute aucun argument MCP.

APK8013 signé/certificat et lignée8012 vérifiés ; SHA88f3d027bef69506775e31f2f2e5731f2391300d03cff772b77f296d18493133. Aucun changement visuel d’interface, aucun diaporama technique imposé. Diagnostics et avis de reprise envoyés aux deux demandeurs ; reçus privés conservés sans lecture affirmée.

Publicationv0.8.13 : six actifs vérifiés par SHA/taille GitHub puis téléchargement anonyme. Quatre rapports natifs installed0.8.13/up_to_date/fresh. Code et bilan intégrés àmain et poussés ; alignement final quatre hôtes et retrait du checkout Mac dans le reçu de clôture privé.

Worktrees247 AndroidCartae et compilationSYS retirés après intégration, propreté, inventaire des ignorés et usages ;870 fichiersAndroidbuild avec le checkout. Quatre copies de transfert retirées :138168132 octets logiques. Le nettoyageCargo automatique a retiré les intermédiaires dépassant le budget ; caches partagés finauxMac2,03Gio/SYS5,51Gio sous10Gio. Conserver version active247 et retour246-r2 qualifié, anciennes images encore chargées, preuves, APK de référence, vues et travaux étrangers. Le dernier checkout/branche247 est retiré après le dernier commit ; pas de suppression de tâche ou de reçu pour faire le ménage.

Preuves et reçus privés : /Users/moi/.cache/bridget247-coordination/. Aucun redémarrageT3 autorisé ni effectué.
