# Validation207 — en cours

Source initiale c79bbc60 : 5 tests ciblés et26 tests Git PASS. Source3ff8be65 :6 tests207 PASS. Revue7d2f4890 : trois P2 corrigés dans3ff8be65 (full-tree, manifeste, codes E/S). Revuea9757f7d confirme les trois corrections. Son P2 sur1Mio est infirmé par le puits capture : stdout de ls-tree et ls-files dispose déjà de8Mio, autres commandes1Mio ; une vraie extraction native a passé sur le dépôt Opus2D de près de20k fichiers. Le plafond reste borné, aucune sortie tronquée acceptée.

La suite élargie sur3ff8be65 a trouvé une régression : lire un marqueur absent avec config --worktree échoue sur un ancien dépôt sans extension worktreeConfig. La lecture utilise maintenant config --no-includes --get, qui consulte aussi la configuration du worktree lorsqu'elle est activée et accepte l'absence sur un ancien dépôt. Les réglages ne sont pas modifiés. Le contrôle de drapeaux passe avant l'absence de manifeste ; un ancien assume-unchanged peut désormais produire rapport indisponible plutôt qu'un faux propre. Aucun fichier de ces dossiers n'est converti.

Recette manuelle native sur un clone privé Opus2D sous le cache : SHA ab9e77f0c57d428aa31029cce58c09cbced1c01d,350243993octets sélectionnés sur9923953231 ;387444Kio alloués,5,87s. Aucun Task, modèle ou service métier. Les deux premiers essais privés ont été refusés avant création car le clone sans checkout n'avait pas la règle d'exclusion locale .worktrees ; la recette a ensuite fourni cette règle privée, sans changer le projet original.

Aucun chargement207 à ce stade. Binairesc79/3ff sont candidats, pas une preuve de runtime. La mesure finale, les hachages des binaires chargés, la configuration et le nettoyage seront ajoutés après vérification.

## Livraison réelle — 29 octobre 2026

Source fonctionnelle d7ba7fa5f5df480c406e0b25cbfa4b606448eebb intégrée et poussée sur main, Mac et Cartae propres. Avis91e9fd50 ronde2 favorable limité au complément de guidage ; les trois objections initiales sont levées. Six tests207 PASS,27 contrôlesGit PASS avec recoupements, guides32pages/522liens/13paires PASS. Aucune FULL revendiquée.

Build207-compact-d7ba7fa5, exécutables exacts installés et chargés :
- Cartae : maître1373915, worker1374645, mobile1374130 ; SHA256 a064e7475990d81be3ddd13ce3b0b405c14e09e619f1a3744a72d8bdd4ad6c0d.
- SYS : worker341100 ; SHA256 7345dbc38bf9908dbf4683ed89dcef60b5827e8d2cf3854a1d32803c07e99c4d.
- KS : worker454409 ; même empreinte Linux2.36 que SYS.
- Mac : worker29637 et pont29640 ; SHA256 071dc164a7a78e5d70943fd196ec57038c78fca0f073711338d37a22cdaa2fda. Le lanceur worker utilise maintenant le lien installé stable, au lieu de rester fixé à183.

Les superviseurs de mise à jour restent actifs ; leur processus Python ne porte pas l'identité d'un exécutable Bridget. Les anciens clients MCP déjà ouverts peuvent conserver leur version ; le binaire207 est utilisé à leur prochaine reconnexion. La nouvelle durée effective est déjà rendue par le maître. Des MCP207 nouvellement lancés ont été observés. Aucun arrêt de modèle ni de service Opus2D demandé ou effectué.

Coupure maître2,51s, sauvegardes avant/après arrêt propre :
/home/moi/.local/share/bridget-maintenance/207-compact-production-20261029T071650Z.
Les deux wrappers Cartae1162524/birth277006331 et2500061/birth270585460 sont restés vivants, identités inchangées lors du remplacement de leur worker. SYS et KS n'avaient pas de reçu actif à leur chargement. Aucun permis ou Terminal fabriqué. La première copieKS a été tronquée ; aucun worker n'a chargé ces octets. Le fichier a été remplacé atomiquement après comparaison avec le SHA SYS complet, avant activation.

Profil compact-v1 activé dans la configuration Git locale de :
/home/moi/10.Scripts/69.opus2D (Cartae) ;
/mnt/storage/bridget-projects/opus2d/repository (SYS) ;
/home/moi/.local/share/bridget-projects/opus2d/repository (KS).
extensions.worktreeConfig était déjà true. Seule la clé administrative bridget.worktreeProfile est ajoutée. HEAD/index de chaque dépôt sont restés identiques au contrôle. Aucun worktree existant converti ni déplacé. Les checkouts Bridget préparés SYS/KS restent à leurs HEAD détachés historiques : aucun reset pour les aligner artificiellement.

Le catalogue actif rend mission_reply_default_secs3600, mission_reply_max_secs86400 et delegations_haltedfalse. La durée n'est pas allongée rétroactivement. Le propriétaire choisit reply_timeout avant une nouvelle admission. Aucun rejeu métier effectué par le mainteneur.

Monuments informé par envoi mcp-50239-6ae2f664-1a, accepté in_flight, sans preuve de lecture. Les preuves privées de builds, empreintes, processus, sauvegardes et configuration sont sous /home/moi/.cache/bridget207 sur chaque serveur et /Users/moi/.cache/bridget207 sur le Mac.

## Refus Politique sol pendant le chargement du pont

Les traces T3 du fil5d7b4e02 établissent deux auto_delegate refusés à07:21:58 et07:22:22, puis task_status refusé primary_not_live à07:22:55. Le journal du pont établit la réinscription du primaire à07:23:04. L'annuaire natif montre ensuite conn-443/busy/lien frais. Cela explique le décalage temporel ; une attestationT3 ne prouve pas la disponibilité du lien Bridget. Aucun droit changé ni tâche relancée.

Contrôle ciblé des métadonnées durables :856c1e90 est result_available/errornull/cleanup_done, résultat3017octets ; aucun corps lu ou publié. La clé opus167-passation-fermeture-ne02-20261029-01 n'a aucun reçu d'admission au contrôle. Propriétaire informé par mcp-50239-6ae2f77f-1c ; la lecture depuis son propre parent reste à confirmer. Aucun succès de son auxiliaire inféré du seul annuaire.

## Nettoyage de livraison

Commande installed exécutée sur les caches correspondants au dépôt propriétaire : Cartae4,24Gio, Mac4,48Gio et SYS0,57Gio logiques reconnus, tous sous le budget10Gio. Les appels depuis les paquets source isolés ont été refusés sans suppression ; le passage a ensuite été fait avec le même nettoyeur du dépôt propriétaire du cache partagé. Aucun target copié.

Retirés : candidat207-c79bbc60 Cartae jamais chargé, copie de transfert Linux2.36 sur Mac et copie de transfert KS après comparaison avec les exécutables installés complets. Les exécutables207 actifs et les retours restent installés. Les versions201/183 encore utilisées par des wrappers/MCP restent conservées ; aucun processus d'un autre travail arrêté pour les retirer.

Le clone privé de mesure (environ378Mio), ses fichiers Git et les journaux sont conservés comme preuve technique. Les checkouts de revue SYS restent conservés tant que leur absence d'usage n'est pas attestée. Leur tâche a été lue, acquittée puis clôturée nativement. Les branches et le worktree d'intégration207 sont examinés séparément avant retrait. Les références de base historiques d'autres sessions restent intactes.

## Clôture du rangement

Observation Cartae complète (lecteur natif linux_system_lsof,10,799s, aucune lacune). Le worktree d'intégration207 n'avait que les matériaux SpecKit générés ignorés ; ils ont été archivés, avec comparaison des empreintes, dans /home/moi/.cache/bridget207/specify-memory207. Le worktree /home/moi/10.Scripts/64.bridget/.worktrees/207-preparation-econome et sa branche locale ont ensuite été retirés sans force. La branche distante session-207-preparation-econome, intégrée à main, a été supprimée. Ses commits restent sur main. Reçu privé : /home/moi/.cache/bridget207/worktree-removed.json.

Le nouvel inventaire SYS rend encore process_access_unavailable : les checkouts de revue observés restent conservés pour cette raison, sans arrêt de processus pour les retirer. Aucun nettoyage d'une tâche Opus2D ou d'un binaire encore utilisé.

La réponse au signalement Politique a aussi été remise dans le même canal T3 que sa demande, message client bridget207-politique-retour-lien-20261029-01, remise steered sur son run251 existant. Aucune nouvelle conversation ou tâche métier. Cela atteste la remise du message, pas la réussite de son prochain outil. Monuments a reçu la confirmation de livraison par le même mécanisme dans son fil existant, clé bridget207-monuments-livraison-20261029-01.
