# Passage de maintenance197 — décision humaine requise

Le gel humain génération7 est attesté. Tous les travaux natifs récents sont clôturés. Le maître183 refuse encore Prepare (update_missions_active_or_unconfirmed) en raison d’états historiques conservés. Le gel seul n’autorise pas à ignorer ce refus. Aucun passage exceptionnel exécuté.

## Périmètre concret proposé
Remplacer seulement les exécutables Bridget des services maître/mobile/workers Cartae, KS, SYS, et préparer le client/liaison Mac. Candidats197 séparés, mêmes configurations, identités, fichiers, commits et bases. Aucun service ou modèle Opus2D, ni programme T3, redémarré. Les connexions MCP vivantes ne sont pas présentées comme rechargées par copie d’un fichier.

Conserver e595 cancelling/inconnu, les deux admissions anciennes et les deux questions persistantes. Ne jamais modifier directement ces lignes, inventer Terminal ou rejouer une mission. Les correctifs chargés ne signifient pas que ces anciens états sont réconciliés. Cette limite doit être annoncée.

## Conditions vérifiées avant la décision
- Candidats MacARM64/Linuxglibc2.39/Linuxglibc2.36 construits, vérifiés par version, build_id intégré et SHA256 ; tests ciblés et signature locale Mac valides.
- Sauvegarde SQLite cohérente (quick_check OK), flotte, configuration et unités privées conservées avant tout arrêt. Une sauvegarde finale après arrêt est requise pour capturer les derniers événements.
- Cgroups des six services Linux concernés : un seul PID principal, aucun fournisseur enfant. Tous leurs reçus worker sont terminal/refused ; seules les données historiques inconnues du maître restent bloquantes.
- Aucun souhait local Running dans la flotte. Aucun child request natif persistant. Les fermetures6eea etb871 sont prouvées par cleanup_done/guard_released et cancel_stopped.
- SendSIGKILL=no sur core/mobile/updater/workers ; pas de forçage d’arrêt. Aucun processus Firefox concerné.
- Contre-épreuve de redémarrage : annulation distante expirée/flotte absente conservée, aucun fournisseur local préparé. Ce test ne prouve pas seul l’ensemble d’une activation réelle.

## Contrôles obligatoires pendant le passage
1. Relire halt, tâches, commandes, flotte et PID avant chaque arrêt ; tout nouveau travail ou changement d’empreinte arrête le passage.
2. Arrêter proprement le seul updater pour exclure son activation concurrente, vérifier sa sortie et prendre son verrou existant sans le supprimer ni le remplacer. S’il ne sort pas ou si le verrou reste pris : ne pas arrêter le maître.
3. Arrêter proprement le maître sous le contrôle de son unité, attendre la disparition du PID vérifié, sans escalade. S’il reste vivant : aucun second maître ni remplacement de socket.
4. Après arrêt confirmé, sauvegarder l’état final puis remplacer atomiquement le lien de l’exécutable et démarrer le maître. Vérifier le chemin /proc/PID/exe, son empreinte et le build annoncé ; vérifier que le gel et les états historiques restent conservés.
5. Mettre à jour les workers inactifs et la façade mobile un à un, avec mêmes gardes et vérification des PID/empreintes. Aucun modèle lancé pendant le gel.
6. Un échec de santé revient au seul binaire précédent compatible, après arrêt propre confirmé du nouveau. Ne pas restaurer automatiquement la base avant arrêt et ne pas supprimer les événements reçus depuis la sauvegarde.
7. Vérifier chaque binaire chargé, puis rétablir l’updater. La levée du gel reste une commande humaine native. L’essai fonctionnel de délégation vient après cette levée, avec une mission de qualification distincte, jamais une copie d’une mission métier.

## Limites explicites
La contre-revue197 a été refusée par le gel : aucun avis favorable indépendant de cette procédure n’est revendiqué. Les revues précédentes avaient refusé une procédure moins complète. Les nouvelles observations lèvent plusieurs objections mais ne constituent pas une autorisation humaine exceptionnelle. Les états inconnus ne disparaîtront pas du seul fait de remplacer le binaire. Le rechargement des clients MCP Mac vivants exige le parcours de reconnexion de leur hôte ; aucun fournisseur ni fil n’est redémarré à cette fin.
