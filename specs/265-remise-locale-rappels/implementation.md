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
