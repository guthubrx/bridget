# Reprise du projet de communication

Le 14 novembre 2026, le fil Revue garde une présence native active mais aucun projet. Le journal maître à 07:10:54 UTC porte project_resolution_empty. À 07:20 UTC, bridget_capabilities et bridget_specialists refusent origin_session_reattest malgré une preuve T3 valide. Aucune admission de ce propriétaire depuis 07:18 UTC.

Cause de persistance vérifiée : le connecteur mémorise les chemins dès l’envoi et ignore ProjectContextResult. human_authority_guard exige un projet attesté. Le premier échec de résolution reste de cause inconnue ; l’indisponibilité initiale du volume externe est une hypothèse, pas une preuve.

Correction : un résultat sans projet programme la réannonce des mêmes chemins et du rattachement après 30 secondes. Une réponse positive arrête la reprise. Un changement de chemins passe immédiatement. Aucun droit, état de tâche ou contenu de conversation ne change. Les contrôles du maître restent autoritaires. Coût constant par fil et annonce bornée à une par délai après refus.

Qualification sur une copie du runtime cumulatif, sans compilation de main. Le test spec291 réussit sur Mac et vérifie les trames réelles de projet puis de rattachement, la borne, l’arrêt après succès et le changement de contexte. Autres qualifications et activation en cours.

Preuves : /Users/moi/.cache/bridget291-projet.

La contre-revue indépendante 6817bd89-fcc7-4886-a0e6-320b420f63de ne trouve aucun défaut confirmé. Elle précise que le délai est testé par déplacement de l’échéance, sans attendre 30 secondes réelles, et que l’intégration maître reste à vérifier sur le fil. Résultat lu, ACK et garde close ; vue gardée sous review_required.

Quatre tests ciblés réussissent sur Mac : spec291, deux spec138 de projet et spec145 de rattachement. Clippy tous targets avec -D warnings réussit après simplification d’une condition booléenne. Le premier lancement de qualification n’avait sélectionné aucun test, car le patch n’était pas appliqué dans la copie sans Git ; ce passage n’est pas compté comme succès. Le patch a ensuite été appliqué et la présence du test vérifiée avant exécution.

Le binaire Mac 291-reprise-projet-r1 est compilé et installé hors staging : /Users/moi/.local/lib/bridget-communication/291-reprise-projet-r1/bin/bridget. SHA-256 713fe0d7037797cb94fc9eabc8d34a242540578432db935cd020c57bffbaded5. Les lecteurs de formats sont compatibles dans les deux sens avec l’image active 288, vérifiés par les commandes natives formats et leur validateur. Le lien actif n’a pas changé.

Activation préparée pour le connecteur Mac uniquement. Le maître et les workers n’ont pas de modification dans ce lot. Le parcours local exige T3 arrêté avec confirmation fraîche, puis un état récent waiting/threads=0 du connecteur avant son remplacement. Il ne libère ni ne réécrit les anciennes tâches ou admissions qui retiennent la maintenance globale. Aucun arrêt n’est encore exécuté.

## Activation différée par décision utilisateur

Le 14 novembre 2026, François choisit « Garder T3 ouvert pour le moment » à la demande de confirmation spécifique. La procédure préparée n’est pas exécutée. Le lien actif et les services restent inchangés ; aucune réparation réelle du fil n’est prétendue. Les sources du correctif sont poussées au commit f6fb14f7752724c8625d47b5ca7886a620f44a4c.

Le contrôle natif des versions des clients locaux réussit, verdict compatible. Le binaire installé et son retour sont compatibles avec les formats conservés. Le script build.py installed a retiré 1,37 Gio d’intermédiaires, sans compilation en cours. La copie de qualification (121 Mio), le binaire candidat (36 Mio) et les preuves sont conservés pour la future activation. La vue native de revue reste sous conservation Bridget.

Reprise : une nouvelle confirmation explicite est nécessaire avant tout arrêt de T3. Vérifier à nouveau le binaire actif, les formats, les processus et les passages de maintenance avant d’exécuter la procédure locale. Après chargement, vérifier le projet du fil dans l’annuaire puis un appel natif depuis ce fil. La livraison reste incomplète jusqu’à cette preuve.
