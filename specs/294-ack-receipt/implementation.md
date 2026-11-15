# Acquittement explicite des pages de fil

Signalement Chef Parix du 15 novembre 2026, mcp-81881-6af92f5a-9. Le schéma commun expose receipt et ack_receipt sans préciser leur action. Le transport accepte receipt pour Ack et ack_receipt pour Post. Le parent a acquitté sa page avec receipt ; aucune réparation de son fil nécessaire.

Correction : descriptions des deux champs et exemple verbal dans la description de bridget_thread. Pour action=ack avec ack_receipt, refus explicite avant tout raccordement : fournir thread_id et receipt, retirer ack_receipt réservé à post. Le contrat de transport, les droits et les états restent inchangés.

Le test spec294 vérifie la documentation, le décodage de receipt et le refus local des deux formes erronées, y compris quand les deux champs sont présents. Qualification dans /Users/moi/.cache/bridget291-projet/runtime-source : spec294 passe (un test). Les régressions spec147 passent : 57 tests, trois interopérations explicitement ignorées, aucun échec. Clippy strict passe sur la bibliothèque et les tests (-D warnings). Le nettoyage automatique a retiré 10,27 Gio de cache après ce contrôle. Les journaux de qualification sont dans /Users/moi/.cache/bridget294-ack.

Activation non effectuée : maître actif 288-preparation-guidage-r1 ; contrôle du 15 novembre, timestamp 1794720724 : trois admissions chosen et quatre tâches non terminales (une queued, trois starting). Ces faits satisfont le garde update_missions_active_or_unconfirmed. Aucun état n’est forcé, aucun arrêt de T3 demandé ou effectué. Les façades déjà chargées restent anciennes. La livraison est incomplète tant que le correctif n’est pas chargé.

Incident de qualification : premier Cargo lancé dans main puis arrêté (seul sous-processus propre). Après reprise, git apply dans la copie sans dépôt a ignoré le patch avec code 0 ; le premier filtre a donc exécuté zéro test et ne constitue pas une validation. Patch transféré explicitement et test ajouté vérifié sur disque avant le second passage. Les écarts rustfmt préexistants de main sont conservés ; le code ajouté est formaté.
