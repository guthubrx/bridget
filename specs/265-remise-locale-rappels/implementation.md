# Diagnostic et correction 265

## Cause constatée

Quatre missions locales ont une réponse finale conservée dans la boîte durable du wrapper. Leurs fins de tour sont completed, mais le maître refuse les enveloppes reply244 avant capture : authorize_wrapper_reply exige un worker pour toute mission, même sans remote_placement. Les réponses restent bloquées sur disque. Les demandes suivies restent ouvertes, puis les tâches expirent. Aucun défaut de transport n'est déduit de ce constat.

Le collecteur de rappels réarme aussi le même palier après chaque alternance occupé/repos. Un palier différé ne doit être armé que s'il dépasse le palier déjà envoyé. Les anciens textes conseillaient une réponse manuelle incompatible avec les missions natives et tronquaient leur identifiant.

## Périmètre

La réponse locale reste soumise à l'identité primaire, à la corrélation exacte, à l'autorité de l'enfant et à la conservation avant acquittement. Une mission distante exige toujours le dépôt attesté du worker. La capture locale clôt le suivi après sauvegarde durable ; le tick reprend une clôture SQL échouée sans nouveau tour. Les accusés de transport ne sont pas fabriqués.

Les quatre tâches historiques sont déjà en échec et leurs enfants arrêtés. Leurs états restent conservés. Les réponses récupérées sont copiées avec identifiants, dates et empreintes dans le stockage privé, puis remises au responsable par le parcours normal. Aucun résultat canonique n'est inventé, aucune mission n'est recréée ou rouverte.

Correction portable : aucun nom de machine, chemin d'installation ou identifiant d'incident dans le code. Temps de collecte inchangé ; seules les transitions de niveau et une clôture indexée par identifiant sont ajoutées.

## Qualification

Les deux régressions échouent avant patch : refus remote_result_requires_worker du résultat local et répétition d'un palier déjà envoyé. Le premier lancement de tests a rencontré un chemin temporaire macOS trop long ; les essais utilisent ensuite TMPDIR=/tmp.

Quatre régressions ciblées, trois scénarios voisins remote170 et vingt-huit scénarios spec244 réussissent sur Mac. Clippy strict des bibliothèques réussit. Les tests de stockage voisins ont d’abord refusé le lien /tmp avec SQLITE_OPEN_NOFOLLOW ; relancés sous TMPDIR=/private/tmp, ils réussissent. Aucune garde système n’est affaiblie.

La reconstruction265 reproduit les quatre fichiers attendus de la base exécutable264. Les variantes de noms de modules réservées au programme260 sont adaptées seulement dans le patch de construction. Aucun code260 supplémentaire n’est activé.

Activation et revue en cours. Ne pas annoncer la livraison avant activation vérifiée.

## Revue indépendante et corrections complémentaires

La revue de la tâche 473ab1da-d91c-41b5-b873-a6f402458d7d confirme la distinction des identités. Elle relève un acquittement sans conservation possible pour une finale tardive sur une tâche terminale, ainsi qu’un message trompeur après sauvegarde suivie d’un échec SQL de clôture. Ces deux défauts sont corrigés avant activation. Une réponse hors ronde ou après clôture reste explicitement refusée et conservée dans la boîte du wrapper ; aucun acquittement de capture n'est fabriqué. Le suivi SQL différé ne fait plus annoncer que le résultat durable manquerait. Le filtre des rappels exige aussi une demande ouverte exactement corrélée, et l'autorité de wrapper vérifie l'enfant et son instance contre la tâche locale.

Les six régressions réussissent sur main, dont trois états terminaux, conservation de la finale tardive, reprise SQL et retrait du rappel d'une demande déjà close. Les deux cas initiaux avaient échoué avant patch. Les réponses historiques récupérées représentent 10376 octets de texte au total ; aucune de leurs tâches n'est rouverte.

La première admission de revue a été refusée sans tâche à cause d'un SHA mal saisi par le parent. La demande corrigée a rencontré une coupure avant transmission pendant l'activation264 ; son rejeu strict après retour du lien a admis une seule tâche. Revue lue, accusée puis clôture propriétaire demandée ; sa vue reste conservée.

Les quarante scénarios native153 réussissent sur main après la vérification de compatibilité : les anciennes enveloppes tardives restent absorbées sans notification ni mutation de la nouvelle ronde. Pour une nouvelle enveloppe reply244 conservée, une ancienne ronde rend un refus explicite, afin de garder la matière dans sa boîte. L’analyse statique stricte finale de main réussit.

Le signalement d'attestation T3 vers 09 h UTC précède toutes les traces serveur encore présentes, dont les plus anciennes commencent à 10 h 48 UTC. Les archives de trace tournantes et le journal enfant ont été examinés. Le rejeu strict a ensuite admis la mission initiale ; aucun doublon n'est créé ici. Ces traces ne permettent pas d'attribuer rétrospectivement le délai à un réseau ou à des droits. unknown_recipient sur l'enfant déjà terminé est cohérent avec l'absence de route active, et ne démontre pas une perte du résultat.

## Clôture des suivis historiques après expiration

Le contrôle réel après chargement de 0.8.31 montre que les quatre tâches historiques restent en échec, mais leurs demandes suivies restent open. Le défaut est distinct de la capture corrigée : le tick clôt les annulations et les résultats, mais omet les échecs canoniques mission_reply_timeout après nettoyage. La révision r2 clôt ces seuls suivis avec mark_timed_out, sous la transaction existante. Elle exige open et le couple propriétaire/enfant exact. Elle ne change aucune tâche, aucun résultat et ne relance rien ; les autres erreurs ne sont pas reclassées.

Deux tests supplémentaires couvrent la tâche expirée déjà nettoyée, son état inchangé après deux ticks, les erreurs différentes et un suivi étranger. Le premier essai comparait la fixture avant la normalisation de save ; la comparaison porte désormais sur la tâche réellement relue après sauvegarde, comme les quatre cas constatés.

Revue indépendante ciblée de r2 : tâche a34a6017-d733-4b11-88f1-7a6d8ef6d485, lue, accusée et clôture propriétaire demandée. La logique de clôture est validée à la lecture. L'objection de compilation concerne quatre appels de test à l'ancien nom, déjà renommés dans le commit de correction 94f4d6c9 ; les huit scénarios compilent et passent. Les tests sont renforcés sur le suivi effectivement présent, une panne SQL de clôture puis sa reprise, le propriétaire étranger et l'enfant étranger. Les annulations gardent leur état ; leur clôture retire aussi le rappel en mémoire et réveille l'attente native une fois. Aucun nouvel avis de relance n'est envoyé par ce chemin.
