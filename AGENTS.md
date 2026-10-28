## Incidents : corriger et livrer le guidage

Pour chaque incident remonté, vérifier la cause réelle. Tout défaut confirmé de
Bridget, y compris un message ou un conseil insuffisant, doit être corrigé puis
livré dans le binaire actif, avec les contrôles ciblés utiles et les sources
poussées. Une explication ponctuelle au demandeur ne suffit pas : le prochain
agent doit pouvoir agir avec le message reçu.

Un refus doit rendre sa cause constatée, le contexte utile autorisé et une action
de résolution précise. Il indique quand et quelle décision demander à
l'utilisateur si l'agent ne peut pas résoudre la cause dans son périmètre.
Une cause inconnue reste explicitement inconnue : nommer la vérification
manquante, sans inventer un problème de réseau, de compte ou de droits.
Conserver le code technique, la tâche et la clé de requête ; ne pas conseiller
un remplacement ou un nouvel essai sans préciser les prérequis. Informer ensuite le demandeur du
correctif livré. Cette consigne permanente a été donnée le 24 octobre 2026.

## Builds : entretien automatique obligatoire

Pour toute compilation ou test Cargo de Bridget, utiliser `make build`,
`make release`, `make test` ou `python3 scripts/build.py cargo ...`, y compris
les tests filtrés. Ne pas lancer Cargo directement : cela contourne le nettoyage.
Le passage contrôle les caches avant et après Cargo, même en cas d'échec.
Réutiliser le cache de ce checkout ; ne pas copier un target par test ou par revue.
Si une isolation est nécessaire, utiliser un dossier `.cache/bridget-build-*`
du compte courant, via `CARGO_TARGET_DIR` ou `--target-dir`, pour son enregistrement.
Le budget commun des intermédiaires reconnus reste 10 Gio et 7 jours.
Après installation vérifiée hors staging, appeler la commande `installed` du
même script. Les exécutables installés et sauvegardes restent hors de ce budget.
Conserver la version active et une version de retour ; traiter les autres copies
identifiées à la livraison, sans supprimer de travaux non intégrés ni de preuves.

## Fin de livraison : Git et nettoyage obligatoires

À chaque livraison autorisée, sans nouvelle demande utilisateur :

- committer le travail validé avec l’identité Git existante, l’intégrer à `main`
  et pousser les sources ; vérifier que `main` est propre et aligné au distant
  sur les machines concernées ;
- retirer les worktrees et branches de travail devenus inutiles après vérification
  de l’intégration, de la clôture des tâches, des fichiers ignorés et des usages
  réels ; supprimer aussi les références distantes obsolètes vérifiées ;
- nettoyer les intermédiaires de compilation et les copies de binaires devenues
  inutiles ; conserver la version active et une version de retour vérifiée ;
- conserver tout travail non intégré, preuve, tâche active ou binaire encore
  utilisé ; annoncer le motif précis et reprendre son nettoyage à la livraison
  suivante ; ne pas terminer les processus d’un autre travail pour faire le ménage ;
- donner le bilan réel : commit poussé, éléments retirés et conservations.

Le nettoyage avant et après Cargo reste automatique avec le script existant.
Le contrôle de fin de livraison complète ce script : il ne transforme pas une
suppression de données ou de travaux en cours en entretien autorisé. Consigne
permanente confirmée par l’utilisateur le 24 octobre 2026.

## Autorité produit et adaptateurs

Bridget définit ses parcours, ses contrats et son expérience utilisateur.
Pour concevoir ou corriger un parcours, partir du besoin utilisateur et de
l'implémentation Bridget. Aucun produit tiers ne sert de référence imposée.
Les noms des clients externes sont réservés à la documentation de leur adaptateur.
Les guides généraux et les bilans présentent les fonctions natives de Bridget.
Ne pas décrire le produit par comparaison avec un client externe.

<!-- SPECKIT START -->

Session198 repriseStart EN COURS : défaut bd3dabf6 confirmé, ancien maître/worker183 inchangés. Plan de reprise de la même tâche après clôture durable avant naissance ; aucun remplacement métier. Première partie worker25a78321 intégrée dans la branche de correction :4tests198+quota et10contrôlescompatibilité rapportésPASS. Préparation du worktree conservé et raccord maître encore en cours ; aucune qualification finale, intégration main ou activation revendiquée. Revue de conception2d37aa0c (2rondes) a relevé les gardesdeadline/fence/tick/parent/ACK. Sourceetpreuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/198-reprise-start/validation.md.


Session197 compatibilitémacOS : source04214db5f89e4159a25a4061bb8b7dfb5969ce52 corrige la taille des constantes de mode dans le diagnostic188 ; buildsMacARM64/Linux2.39/Linux2.36PASS,10tests188Linux+7tests196MacPASS. Candidats197 construits et vérifiés surMac/Cartae/KS/SYS, NON actifs.14contrôles de maintenance et1contre-épreuve de préservation distantePASS. RevueBridget refusée delegations_halted, aucun avis197 inventé. Gelhumain génération7 posé ; gardesOpus6eea/b871 closes et tousreçusworkersterminaux, cgroupsdesunitésseuls. Prepare reste refusé pour annulationhistorique/admissions non soldées ; aucune modificationdustore ou restartforcé. Sauvegarde cohérente prête ; activationT004ouverte. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/197-compatibilite-macos/validation.md.


Session196 lecture d’attente : sourcea42d133cd3d21f31c07c146a8d8bf8bc0418c685,75contrôles ciblésPASS, guides vérifiés et revue5007923b favorable. Après échec de transport du seul task_wait, une relecture1s des mêmes tâches refait toutes les gardes ; aucune mutation ou refus métier rejoué, second échec guidé. Cause exacte du délai historique inconnue. CandidatLinux196-wait-a42d133cd3d2 construit/vérifié NON actif ; clientMac183-e2987fb7175f et maître183 inchangés, T004 ouverte. Correction commune disponible dans les sources poussées, pas dans une ancienne release déjà publiée. Réparation Git195 toujours opérationnelle, pas encore une prévention automatique. Revue close nativement ; référence SYS conservée sous usages partiellement inconnus. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/196-relecture-attente/validation.md.


Session195 réparation opérationnelleKS : index stat du worktree bef1 actualisé par Git, entrées/HEAD/chemin/inode conservés ; status1,700→0,081s, refresh/sum_lstat1774→0. Vérification native4157daa7 surKS a livré un témoin exact inconnu du brief, errornull, résultat lu/ACK et garde clôturée cleanup_done. La mission métier bef1 a enfin enregistré sa remise1793164798 puis result_available/errornull, même tâche/clé, sans rejeu ni annulation du mainteneur. Fichiers métier intacts, aucun restart/binaire/config modifié ; runtime183 reste actif, maintenance globale et activation194 toujours distinctes. Preuves privées KS conservées ; reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/195-reparation-ks-index/validation.md.


Session194 admission rejouable : source b38a08713c95c0858807cdaed7fcf78ce7fa7b3d,40contrôles ciblés PASS, contre-épreuves locales et protocole distant, revue finale790cce5d favorable après correction du P1 effort distant. Choix/effort/canon durcis avant revalidation ; aucun décideur neuf au rejeu chosen, gardes source/droits/cible conservées. Candidat194-admission-b38a08713c95 vérifié NON actif ; maître183 et ancienne annulation e595 non attestée inchangés. Clé propriétaire opus167-revue-fermeture-et-gardes-20261028-01 reste decision_started sans tâche, dernier refus corrélé workspace_diverged ; aucun rejeu ou mission métier. T005 chargement/usage ouverte. Sources intégrées/poussées ; revues clôturées, worktrees de revue et exécutables193/194fee retirés après gardes, référence légère SYS conservée sous ancien HEAD. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/194-admission-rejouable/validation.md.


Session193 rapport Git unique : source180c0aa4c74333616919d1709dcbc8249c1c264b,39contrôles ciblés PASS, contre-épreuve2→1status, revueacc0f775 favorable. Candidat193-report-180c0aa4c743 vérifié NON actif ; maître183 confirmé, maintenance native toujours bloquée. Aucun rejeu/purge/annulation métier. Huit demandes historiques non persistées ; refus Cartae final sans mesure conservée en183. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/193-rapport-unique/validation.md.


Session192 délai de preuve : source1b77b7390fa292e1933a1e69d4967387020ebf3b qualifiée,15contrôles ciblésPASS, contre-épreuve5s reproduite, guides vérifiés et revue7b317595 favorable. Le Reconcile de preuve attend30s au lieu de5 ; file8, droits, identités, permis et créneau unique inchangés. Candidat192-proof-1b77b7390fa2 vérifié, NON actif ; garde de maintenance183 maintenue. Aucune admission métier, annulation tierce ou urgence de caches rouverte. Composition historique de la file KS non reconstituée. Sources intégrées/poussées ;2worktrees et1ancien exécutable retirés, référence légère SYS/actif/retour/diagnostic188/candidat192 conservés. Retour Politique accepté in_flight ; lecture non déduite. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/192-delai-verification/validation.md.


Session191 stockage et EOF : source e882662490006e34cf54eda5137294a3d0769b88 qualifiée,46contrôles ciblésPASS, guides vérifiés, contre-revue afb72cfc favorable. Codes ENOSPC/EDQUOT fermés et guidage EOF avec référence de commande au propriétaire, inconnus conservés. SYS a récupéré8,7Gio par archivage de ses copies de revue effectué par leur propriétaire ; garde métier15Gio inchangée. Candidat191-storage-e88266249000 vérifié, NON actif ; chargement natif encore bloqué par la maintenance183, aucun forçage. Sources intégrées/poussées,3worktrees et2anciens exécutables retirés ;2références légères SYS conservées sous HEAD ancien. Bilans propriétaires acceptés in_flight ; lecture non déduite. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/191-guidage-stockage/validation.md.


Session190 diagnostic des remises : source0bb08e3a02585d498f165f4e9a0a46b001be6724 qualifiée,59contrôles ciblésPASS, guides vérifiés, contre-revue98c719f9 favorable. Cache maître volatil borné, statut propriétaire et trace des refus réels sans changer la remise. Candidat190-dispatch-0bb08e3a0258 vérifié, NON actif ; maître183 conservé. Holder durable du sharedKS : bef1a3bd ; motif maître exact encore inconnu. Aucun recheck local188 répété, rejeu métier ni annulation tierce. T004 ouverte, maintenance native bloquée sans preuve d’arrêt de l’ancienne tâche. Sources intégrées/poussées ;7worktrees et8anciens exécutables candidats retirés après gardes, actif/retour/diagnostic188/candidat190 conservés. T005190 soldée : bilan conservé accepté in_flight après retour du lien et rejeu exact ; lecture non déduite. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/190-diagnostic-remise/validation.md.


Session189 guidage capacité et précontrôle : source550b096f9b3242c00ba602d37ac6749b8da99ca6, 23 contrôles initiaux et14 du complément PASS, guides FR/EN vérifiés et contre-revues finales favorables. Preuve de capacité atomique, code durable inchangé et anciens refus sans preuve explicitement inconnus. Motif candidat resource_limit : file worker pleine à cet instant, projection commune initiale/relecture ; état actuel et cause amont inconnus. Précontrôle MCP explicite sans données de tâche avant attestation. Candidat Linux189-queue-550b096f9b32 vérifié, NON actif ; ancien candidat001 retiré après vérification, preuve conservée. Maître183 confirmé ; T004 chargement maître/client MCP reste ouverte, bloquée par l’annulation historique non attestée. Aucun service ni mission métier modifié. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/189-guidage-capacite/livraison.md.


Session188 diagnostic des remises : source7625123ca78fe0bf9d421922270152318182e306,110contrôles ciblésPASS, guidesFR/ENvérifiés, contre-revues finales favorables. Candidat séparé188-diagnostic-7625123ca78f livré et vérifié surCartae/KS, diagnostic_only=true, nonactivable. Diagnostic réel bef1a3bd : rechecklocalpassed/complete avec contexteoutilsworker ; cela ne prouve pas la remise métier ni la cause du maintien mission_pending dans maître183. Aucunservice, missionmétier oustoremodifié. Maintenance184–187 toujours distincte/bloquée ; runtime183 confirmé. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/188-diagnostic-remise/livraison.md.


Session187 guidage après huit rondes : code68093e21ea1f1cab3874e60e17c4587de90b4b9c,15tests ciblésPASS, guidesFR/EN vérifiés, contre-revue favorable. Candidat187-rounds-68093e21ea1f vérifié, NON actif. Aucune action sur production/services selon mandat ; plafond et continuable inchangés. Parcours existant : préserver, fermer la garde native, confirmer l'arrêt, puis nouvelle tâche de suite autorisée avec passation et base explicites. Aucun acte sur la tâche monuments4badfd6b. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/187-guidage-plafond-rondes/livraison.md.


Session186 guidage admission : sources qualifiées35123122672abdf49a019bb415d8fed34afe7e95,8contrôles ciblésPASS, contre-revue finale favorable. Candidat186-guidance-35123122672a vérifié ; guidage NON actif, runtime183 confirmé. Maintenance184/185 bloquée update_missions_active_or_unconfirmed ; aucune nouvelle préparation répétée, aucun restart. AdmissionKS opus2d-reprise-20261027-1005-ks-galliera conservée pending, motif retrouvé worker_unavailable en relecture finale ; cause plus précise inconnue. LectureSYS refusée sur attestation du parent, sans conclusion sur58459353. T005 ouverte. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/186-guidage-admission/livraison.md.


Session185 réparation native du workspace : sources qualifiées et poussées sur main, commit46021dda412abdf66dee977d31492b28256affb6. Revue finale favorable.15tests daemon185,8transport185,129droits(1ignoré),3collecte,7briefsPASS. Candidat185-repair-46021dda412a vérifié sur Cartae ; activation REFUSÉE update_missions_active_or_unconfirmed, runtime toujours183. La tâche58459353 n'est pas réparée ; ses fichiers sont conservés ; projection propriétaire reçue : result_available, continuable=false, report_unavailable, cause de fermeture non précisée. T005 reste ouverte. Aucun forçage ni restart. Worktrees/candidats conservés pour livraison runtime ouverte et usage non complètement observé. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/185-reparation-workspace/livraison.md.

Session184 confirmation et guidage des refus : corrections1caabb9e et7899d0bd intégrées/poussées ; cinq tests initiaux et six tests du complément PASS, contre-revue finale approuvée. Candidat Linux actualisé installé, build184-guidance-7899d0bdab01 ; activation refusée par update_missions_active_or_unconfirmed. Le maître actif reste183-admission-7478e9e7, services conservés. Remise métierKS confirmée sans duplication. RefusSYS expliqué par un déplacement du dossier alors que les processus gardent l'ancien inode supprimé ; aucune reprise forcée. Worktree/branche184, candidats et caches conservés : livraison runtime encore ouverte, garde d'usage incomplet. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/184-confirmation-demarrage/validation.md.


Session183 interface mobile :22/22, Implemented ; pipeline Livré le26octobre2026 après contrôle du runtime et du nettoyage ; activationLinux réalisée le26octobre2026 après maintenance explicitement autorisée : core/mobile/workerCartae0.6.3, build183-admission-7478e9e7 ; ancienne admission OVH refusée durablement sans tâche. La tâche historique e5959986 reste non réconciliée. Source529d455a intégrée/poussée, tag/release0.6.3 sur e2987fb7, CIverte. Mac, émulateur6003 et core/mobile/workerCartae actifs en0.6.3.30JVM/21instrumentés/31RustciblésPASS ; auditV15pass, aucuneFULL ni voix/Pixelréels qualifiés, contre-revues en échecapi_error sans verdict. Preuves : diaporama vérifié. Worktrees mobiles177à183 retirés, preuves utiles archivées ; un seul checkoutmainpar machine, copies chargées/retours et tâche historique conservés. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/183-interface-mobile/validation/livraison.md.

Session182 livrée le25octobre2026 :4/4, Implemented, correction documentaire du produit autonome. Source97c7806bed1ee646a336c440e2e6019c031d9292 intégrée/poussée ; guides, consignes et archives séparés des détails d’adaptateurs.32pages/522liens/13paires de langues vérifiés ;39exemples JSON conservés et test de paquet source PASS. Cinq notes de version publiques corrigées, diaporama partagé actualisé sans modifier ses images. Aucun moteur ni binaire changé. Worktree/branche182 et fixture conservés car observation lsof incomplète. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/182-documentation-autonome/livraison.md.

Session181 livrée le25octobre2026 :25/25, Implemented. Questions natives Android/terminal, brouillons locaux, résolution partagée. Source dd1410f43dc28eb4774f7858582c58cdd44e49bb ; clôture documentaire59406a727dde33e8c9fb039e98d3bb89bf855983 ; release0.6.2. Mac/Cartae actifs, émulateur Android15 installé ; Pixel inchangé. Tests ciblés et audit pass ; aucune FULL. Voix réelle non qualifiée ; revue indépendante sans verdict. Preuves : diaporama vérifié. Tâche historique e5959986 conservée ; prochaine activation automatique bloquée tant qu'elle n'est pas réconciliée. Gardes de nettoyage incomplètes : certains worktrees/caches restent conservés. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/181-questions-mobile/validation/livraison.md.
Les reçus datés des sessions précédentes restent dans specs/. Ils ne sont pas des instructions de conception ni une preuve du runtime courant.
<!-- SPECKIT END -->

<!-- SPECKIT-USER START -->
# Couche utilisateur SpecKit

- Source de verite utilisateur: `~/.speckit/`.
- Avant une feature/refactor structurel: executer ou appliquer `/speckit.sync`.
- Charger `.specify/memory/constitution.md` pour les gates projet.
- Charger `.specify/memory/standards.md` pour retrouver les refs et baselines research.
- Ne pas modifier directement `.agents/skills/speckit-*`, `.specify/templates/*`, `.specify/scripts/*` ou `.specify/workflows/*`.
- Les commandes utilisateur canoniques vivent dans `~/.speckit/commands/` et sont publiees vers Codex/Claude/Gemini par `~/.speckit/scripts/sync-agent-adapters.py`.
<!-- SPECKIT-USER END -->
