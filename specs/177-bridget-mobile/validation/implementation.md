# Implémentation 177 — état au 24 octobre 2026

Statut : In Progress. Les étapes ci-dessous sont historiques. Le bilan courant
figure dans delivery.md : r10b actif côté serveur, dernière recette physique
Android ouverte ; aucune publication complète 0.5.0.

## Contrôles ciblés et distribution

Le lot mobile177 passe 57 tests, avec une recette physique isolée ignorée par défaut. La régression des vues humaines passe 33 tests, avec un cas ignoré. La première invocation de cette régression avait six échecs de précondition car le binaire CLI de test n'était pas construit ; après sa construction via le wrapper natif, le même filtre passe. Aucun GREEN global ni suite complète déduit.

Le binaire optimisé r2 est installé séparément sur Mac et Cartae. Le Mac est signé et vérifié. La commande installed a été appelée sur les deux caches ; l'observation Linux incomplète conserve les fichiers protégés. Aucun nettoyage forcé.

## Retour de la recette de migration

La copie hors ligne a conservé exactement 353 Tasks et 424 reçus d'admission, vérifiés par empreintes de lignes avant démarrage. Aucun maître simultané. Les fichiers de machine, les journaux, les clés et les travaux sont restés sur leur origine. Les droits du dossier d'artefacts ont été rétablis à 0700 après extraction.

Le maître Cartae, le backend loopback et HTTPS ont démarré. Le contrôle HTTPS sans autorisation a reçu 401 avec certificat valide. Le premier relais Mac avait un défaut duplex : cat rencontrait EAGAIN après le premier échange. Le runner observe maintenant le premier octet avec MSG_PEEK sans le consommer, puis laisse SSH posséder son flux. Le test natif sous macOS couvre deux échanges espacés et la sonde vide ; 23 tests scripts095 passent.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/177-bridget-mobile/validation/implementation.md#extrait-1).

La correction en cours résout le projet sur le worker d'origine authentifié, via le lecteur existant et le canal corrélé à l'époque. La publication revérifie l'identité et l'hôte ; absence de worker invalide le fait, sans repli local. La garde humaine garde son binding strict et accepte l'hôte du projet vérifié. Une contre-revue indépendante est en cours. Aucune nouvelle autorité de lancement n'est accordée au téléphone.

## Android et notifications

Le build release 0.5.0 et le lint fatal passent après remplacement de l'ancienne dépendance Fragment par la version épinglée 1.8.9. Le SDK de notification est configuré, sans SDK Analytics. Le compte serveur est privé, rôle FCM seul, projet Spark sans facturation activée.

Le Pixel a validé 14 tests de stockage/gardes sur 0.5.0 debug. L'APK release signé est installé par mise à jour, sans suppression. La preuve de lecture des données chiffrées après rotation du certificat passe sur le Pixel réel : résultat preserved_after_certificate_rotation par instrumentation Android indépendante des classes applicatives. Les versions debug et release ont été installées par mise à jour sans effacement. Limite de méthode initiale : les tests JUnit compilés contre debug ne sont pas utilisables tels quels contre les classes réduites/renommées par R8 ; l’instrumentation de vérification utilisant les seules API Android est donc employée. Aucune notification réelle n’est encore revendiquée.

## Revue d’autorité de projet — décision de traitement

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/177-bridget-mobile/validation/implementation.md#extrait-2).

L’objection selon laquelle franchir une garde distante serait en soi un gain de droits n’est pas retenue comme défaut : le déplacement du coordinateur sans perte du droit de déléguer des parents existants est le parcours autorisé. La garde de projet n’accorde pas une posture ; les permissions HTTP fraîches sur le worker, les identités, les révisions, les sources, la politique enfant et la remise native restent obligatoires. Un second avis indépendant est demandé sur ce raisonnement et les renforcements.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/177-bridget-mobile/validation/implementation.md#extrait-3).

La rotation Android a dû conserver la capacité permission de l’ancien certificat privé pour la permission signature DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION générée par AndroidX. Sans cette capacité, Android refusait la mise à jour DUPLICATE_PERMISSION. L’ancien keystore reste privé 0600, hors dépôt ; aucun nouveau droit serveur ou permission d’agent n’est donné. Les capacités shared-uid, rollback et auth de cet ancien certificat restent désactivées. L’APK public ne contient aucune clé privée ni certificat de signature debug additionnel en tant que signataire courant : sa signature courante est celle de production, avec historique de rotation.

## Contre-revue R2 et corrections finales d’origine

Le second avis (Task fa7dcd1d-7991-4d9a-ae46-4394bd28617e) lève les objections de garde corrigées et ne démontre aucune élévation. Il confirme un défaut fonctionnel : une nouvelle annonce pendant une panne effaçait le projet connu. Le scope connu est désormais conservé, mais le binding humain frais est retiré. Le test exige encore le refus interprojet après cette panne. HumanThreadView et HumanThreadWatch utilisent maintenant le même lecteur d’origine. Les lectures de données persistées (artefacts, tâches) ne sont pas une autorisation de lancement ; l’absence du worker n’efface pas ces données.

Le catalogue natif est évalué sur le worker d’origine avec le même évaluateur que le catalogue local. Le coordinateur conserve ses quotas, pauses et arrêts ; le worker relit les sources de droits avant et après le calcul. La réponse est bornée, corrélée et liée au digest parent. Aucun Task, session fournisseur ou worktree n’est créé pour cette lecture. Régressions r6 : 13 tests catalogue PASS, un ignoré ; lecture d’origine réelle Git/catalogue 1 PASS ; vues humaines 33 PASS, un ignoré. Mobile177 r6 avant ajout de la vérification catalogue : 57 PASS, un ignoré. Aucun succès global déduit.

Les deux avis ont été lus et acquittés ; leurs continuations sont closes. Les observations restantes sont des limites du modèle de confiance du même compte SSH, documentées ci-dessus, et non une preuve de gain de droits. Les binaires r6 sont en construction avant la seconde bascule ; les r3/r4 n’ont jamais remplacé la production r2.

## 24 October — physical reassociation and recent-history feedback

The Pixel was updated in place to signed r8c (SHA-256 e5177cb0b7585b460213b56b68b2e7e5b29b7b01b5fd3e9f2fe6472343654255). The saved rejected attempt remained present. Reading the same receipt displayed the dedicated expired/consumed QR guidance, with no generic unconfirmed response. The user scanned a newly issued QR and confirmed association; the server recorded the second device binding. The private QR page was replaced by an association-complete message. It had an automatic five-minute expiry. No QR is included in the public proof.

The user’s long message was read through the official Bridget ledger: it asks for short project labels, recent-first pagination and keyboard-aware bottom following. Its queue entry is released; this is not proof that the busy provider has processed it. The UI now distinguishes a saved instruction from a provider response. Recent history uses the native Attach tail and bounded forward pages; older pages do not advance the live cursor. Native mobile tests after this change: 58 passed, 1 ignored. Android r9 first build/lint/JVM run passed; final UI adjustment is being checked.

One complete workspace test invocation was performed on the integrated r8 source: daemon library 2231 passed, 18 failed, 23 ignored, 117.26 seconds. Cargo stopped at that failing library, so no global-green or all-workspace completion is claimed. The log is private. These failures remain to be classified; no repeated full run is planned.

The core was independently updated by bridge1016 to native-project d5af84f8. That build used the r8 source archive before the empty-snapshot amendment. The common candidate must contain both changes. The user explicitly requested commit/merge/push and cleanup to eliminate this source divergence.
