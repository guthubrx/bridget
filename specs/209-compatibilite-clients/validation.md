#209 — Validation et livraison du29octobre2026

## Source et périmètre

Source fonctionnelle finale113772d362898ecb820714e8de6c7f262cb06df8 intégrée et poussée surmain. Le build de production209-clients-113772d3 conserve la version commerciale0.6.3. Aucun tag/releaseGitHub supplémentaire publié ; le paquet public historique0.6.3 reste inchangé. Les binaires installés et l’installation depuis les sources courantes contiennent209.

Contrôle local des seuls programmes des connexions actives, par versions natives : aucune requête fournisseur, authentification, sélection de modèle ou mise à jour client implicite. Minima de support de cette release : Claude2.1.280 (API400 attestée dans208), Codex0.162.1 (schémaADR155). Aucun minimum universel aux modèles futurs n’est affirmé.

Le candidat fournit son rapport. AvantPrepare, le validateur installé recalcule l’inventaire local exact et vérifie machine, portée, politique versionnée/provenance, versions et états. Les minima futurs peuvent augmenter ; aucune baisse sous le plancher connu. Le champ de rapport reste optionnel pour les anciens messages d’update.

## Contrôles utiles exécutés

- 6testsRustspec209 PASS : connexions locales/déduplication/consentement, versions numériques/inconnues, ancien rapport/données incohérentes, sonde réelle/sorties privées/temps, TERMignoré et PID explicite, identitéworker distincte du nom système.
- 46testsPythoninstallateur/updater PASS, incluant refus avantPrepare/arrêt/bascule, bornes des deux flux, absence de répétition d’une sonde non arrêtée, installationhistorique et candidatfutur refusé.
- Guides32pages/522liens/13pairesdelangues PASS ; git diff --check PASS.
- Contrôle natifCartae et validateur réel acceptent Claude2.1.312/Codex0.163.0. FixtureCLI2.1.251 : client_update_required et demande d’accord utilisateur ; aucun service arrêté.
- Faux rapports d’inventaire vidé et de minima abaissés refusés par la combinaison scriptPython/validateur natif réel.
- Les compilations utilisent scripts/build.py, deux jobs, priorité réduite, caches partagés réutilisés. Profils de tests sans symboles pour maintenir le budget de cache ; aucun target copié.

## Revues et corrections

2ae2759f a trouvé des objections sur validation/provenance/processus/flux. Complément71a0ad34 : validation installée avant réservation, politique versionnée, sorties bornées, étatprobe_stop_unconfirmed+PID et absence de répétition automatique. Aucun SIGKILL, conformément à l’instruction utilisateur.

4f8cf840 n’a confirmé aucun P1/runtimeP2 ; seulP2documentaire sur des caractères littéraux corrigé47b58826. Dernière revue528c9429 favorable limitée à la lecture des sources113772d3, sans P1/P2 ; aucun build/test/runtime indépendant réattesté. Les trois résultats ont été lus, acquittés et leurs missions clôturées nativement.

Réserves : un contrôle de version ne prouve pas l’arrêt physique de tout descendant détaché. Identifiant initialPID/groupe et inspection humaine sont conservés lorsque l’arrêt est inconnu ; ne pas signaler un PID réutilisable. L’exception du paquet officiel historiquev0.6.3 conserve uniquement son installation antérieure, avec avertissementnonqualifié ; aucun refus d’un contrôle disponible ni candidat futur n’est contourné. Le code2 historique est couvert par une fixture, pas par réexécution de l’archive publique dans cette livraison.

## Cas réel de machine corrigé

Le premier contrôle SYS utilisait son hostOSbackup et renvoyait à tortclients=[] malgré la connexion provisionnéesysle2. Donnée réelle relue dans le registre worker privé. Correction : séparerhostetmachine_id, utiliser l’unique identité provisionnée du registre worker dédié, refuser les autres périmètres inconnus. Aucune équivalence DNS devinée. Contrôle final : hostbackup/machine_idsysle2, Codex0.164.0 effectivement lu. KS : hostns31107979/machine_idksstore, Claude2.1.310 effectivement lu.

## Chargement et empreintes exactes

| Machine | Processus réellement chargés | SHA256 |
|---|---|---|
| Cartae | maître1972790, worker1975931, mobile1972903 | ded33d44ac62e210f5c048837763f883ccaf1513312b85925247a97b5862529a |
| SYS | worker350814 | 468864fff96e9607b2d586e4d543e9b8b88f77eb602d213e29b05440ee9e56c7 |
| KS | worker535345 | 468864fff96e9607b2d586e4d543e9b8b88f77eb602d213e29b05440ee9e56c7 |
| Mac | worker39088, pontT339090 | 36e8e33c719a0752232ea952601d894ec87ac49774daafbc9c30a6eb36595984 |

UpdaterCartae1972909/Mac39093 actifs enPython, comportement attendu. Chemin d’exécutable et empreinte vérifiés pour chaque processus natif. Un ancienMCP ouvert peut conserver son ancien binaire jusqu’à reconnexion ; il n’est pas déclaré209.

Coupuremaître2,00s, sauvegarde cohérente /home/moi/.local/share/bridget-maintenance/209-clients-production-20261029T100734Z. WrappersCartae1162524/birth277006331 et2500061/birth270585460 inchangés. Aucun reçu worker actifSYS/KS/Mac au remplacement. Aucun service ou calculOpus2D/NE02 touché, aucune mission métier relancée ni dupliquée, aucun compte/modèle modifié.

Contrôles installés après chargement : inventairesnonvides et compatibles sur les quatre machines ; validateur local vrai. Rapport de compatibilitéCartae publié avec succès par le portnatupdatesreport. Gel relu false.

## Information et entretien

MessagesT3 natifs dans les fils existants : bridget209-fin-monuments-20261029-01 (run539 démarré), bridget209-fin-politique-20261029-01 (run251 steered), bridget209-fin-trajets-20261029-01 (run1067 démarré). La remise ne vaut pas une preuve de reprise métier. Les messages expliquent poursuite selon états natifs, réattestation/relecture ciblée après coupure, aucun remplacement d’une issue inconnue.

installedPASS surCartae/Mac/SYS ; caches3,50/4,49/0,57Gio. Trois copies du candidat209-71 jamaisactivé retirées après contrôle d’empreinte et provenance ; copies de transfertLinux retiréesMac/KS. Actif209 et retour208 conservés, ainsi que les anciens exécutables encore utilisés par wrappers/clients. Worktreesd’intégration/revues et branches conservés sous observation incomplète des processus/fichiers ouverts ; aucun retrait aveugle ni fichier métier supprimé. Preuves locales détaillées dans /home/moi/.cache/bridget209 et /Users/moi/.cache/bridget209.

Correction technique : tests de commande/transport ; aucune interface graphique modifiée. Aucun diaporama d’interface requis.
