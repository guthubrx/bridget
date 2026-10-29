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

Session204 diagnostic de fin fournisseur : sources6192c4aa401549c3488b359417e07207045fa46f qualifiées/poussées ;3tests204PASS,38régressions initialesPASS, dernier contrôle37PASS+fixtureTextfilebusy puis témoin isoléPASS ; revue1788cd8d favorable après correction du stop_reason inconnu. Cause réelle4fb1a401 : server_overloaded côté fournisseur, mission exécutée460s, fichierspréservés, tâcheannuléeparpropriétaire. Le29oct03:29UTC maître612880/mobile612959 etworkersSYS324448/KS330946 chargent204-terminal-6192c4aa ; SHA207254c583786386da4248d6752eece6c4065a320c251916cbc41bab95d4b0af surCartae/KS etded1ce405522fbe8c61f97e62a2e45e4e0b7fd3c40f6018ac2f1fa904970dc9f surSYSglibc2.36. Sauvegardes, arrêtspropres et8,09sdecoupure ; updateractif/aucungel. PontT3réinscritPolitique03:30:06 etroot03:30:08, catalogueMCPvérifié ; near1minapplicative. WorkerCartae2013750537 conservé(reçu1ccReleased), clientsMac201/listener183inchangés : aucuneactualisationuniverselle. c471/fd2913admitted sansremise ; prérequisderejeustrict envoyésaupropriétaire, aucunrejeumainteneur. Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/204-cause-terminale/validation.md.


Session203 placement SYS corrigé par configuration native : toutes gardes/commandesSYS fermées avant publication ; mapping SYS/opus2d révision2 vers /mnt/storage/bridget-projects/opus2d/repository et identitéGit /mnt/storage/bridget-projects/opus2d/mirror.git. Trois autorités maître/worker/origine cohérentes, autres champs et additional_root inchangés. Préparation nativeGit desSHA99a7ed783687138f57cf14d353bb94a5b48df058 et49d905b81f4bd3128b6103abc307244650add44cPASS ; HEAD8691b2189513541247cd40a4c6ec534a75a45ceb conservé, dépendances présentes. Aucun Task/modèle/restart ou nouveau binaire ; maître202/workerSYS183. d653/e2dd déjàcancelled/cleanup_done, aucune reprise. Propriétaires notifiés in_flight ; ancien dépôt/worktrees et Cartae préservés. Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/203-placement-sys/validation.md.

Session202 reprise après choix d’effort : source11c305f5e148619e20cbae98ea52d5d554a2a44a qualifiée/poussée ;33tests198/201/202PASS, guides32/522/13PASS, revue25641d2f favorable. Profil initiallow reconstruit depuis définitionhigh admise, deux empreintes vérifiées avant plan/fence, aucun nouveau choix ni droit. Diagnosticferméstart_recovery. MaîtrePID3896872 etmobile3896961 chargent202-effort-11c305f5e148/SHA9ff74dc3aa5f131b34a24e57fd5b5b61b71e7a87dc09d74c04d16b15e3d8f2ec depuis21:34:45UTC ; workerCartae201PID3750537 etSYS/KS préservés, coupure2,47s avec sauvegardes et arrêt propre. Tâche92965 déjàcancelled/cleanup_done depuis21:23:29, aucune remise ni reprise ; propriétaire informé par message acceptéin_flight. Revue close etworktreeSYS retiré, référence légère conservée carmainSYSancien. Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/202-reprise-effort/validation.md.

Session201 livrée en production le28octobre2026 à20:41UTC : source fonctionnelle56e0bfd59a6f8077f4d594fc136b82849d0feb1d qualifiée/poussée ; tests201/198/lifecycle155/updater et revue4adfc814 favorables. Recette supplémentaire sur exécutables exacts183→201 PASS, worker/fournisseur distants et résultat pendant coupure conservés. MaîtreCartae3750457, workerCartae3750537, mobile3751910 chargent201-cancel-56e0bfd59a6f/SHA8e2bec786938e906ac26e39fb526bccfed9a171d68189ac33e48eb7d3cbe4307 ; updater rétabli, aucun gel. Sauvegardes cohérentes et arrêts propres confirmés, coupure4,34s. Annulations6231 et e595 désormais cancelled/errornull/cleanup_done, par fence natif exact, guard_released restefalse. R51 reprend198 sur même tâche après message propriétaire ; bd3 à reprendre strictement par son propriétaire. SYS/KS et leurs fournisseurs183 maintenus vivants ; aucun serviceOpus touché. Ancien wrapper1982500061 et binaires sous usage conservés, aucun cleanup inventé. T005 soldée pour chargement/clôture201. Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/201-annulation-avant-recu/validation.md.


Session200 — Champ interdit en admission automatique : source5f320738b96ed96eb7cf926497d8836f7b824b18 qualifiée et poussée. Le schéma automatique omet agent_type, le client le refuse avant connexion et le maître nomme le champ avec code public auto_admission_invalid. Conseils de correction avant reçu cohérents ; canon existant préservé et parcours explicite inchangé. Deux tests200, quatre états d’admission174, un précontrôle174 et un protocole168 PASS ; guides32/522/13 et CI hygiène PASS. Revue d7f01d39 ronde2 favorable. Candidat Linux200-field-5f320738b96e construit/vérifié NON actif ; chargement soumis à la maintenance encore bloquée. Omettre agent_type et utiliser engine fonctionne déjà avec183. Revue close, worktreeSYS retiré après gardes ; caches et worktree d’intégration conservés sous observation incomplète/livraison runtime ouverte. T005 ouverte. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/200-champ-admission-auto/validation.md.


Session199 — Refus de cible explicites : source2918b957cfbeabeb286ff4ac5173cf3a026bb590 qualifiée et poussée. Cinq tests199, trois rejeux183, deux tests194 et un test168 PASS ; guides32/522/13 et CI hygiène PASS. Revue3ad2f1b9 ronde2 favorable après correction de la catégorie machine. Comparaisons inchangées, diagnostic fermé borné à128octets conservé avec le refus, code public stable et anciens refus inconnus. Candidat Linux199-routing-2918b957cfbe construit et vérifié NON actif ; Prepare094a0da5 refuse update_missions_active_or_unconfirmed, maître183/PID351206 confirmé. Aucun gel, restart ni acte métier. Revue close et worktreeSYS retiré après gardes ; cache commun conservé sous observation incomplète. T005 chargement ouverte. Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/199-refus-cible-explicites/validation.md.


Session198 repriseStart : sources6617aee8ab4a6f898e360af1f28acaba3539d730 intégrées/poussées ;24testsmaître/46daemonremote162/59worker/32droits(7ignorés)/11transport ciblésPASS (recoupements conservés), guides32/522/13 etCIhygiènePASS, revuefinaleddccc585 favorablelimité après corrections. MêmeTask/clé/enfantlogique/mission/ronde/worktree, clôture prénaissance, bail/routage/ACK/reconnexion/Cancel/quota locaux gardés ; format183lisible, retour183 opérationnel limité. CandidatsMac/Linux construits/vérifiés198-start-6617aee8ab4a NONactifs. Prepare53ba0c37 refuséupdate_missions_active_or_unconfirmed ; maître183/PID351206 enligne, aucun nouveaugel, actebd3 ou serviceOpus2D. T006activation/usageréelouverte ; décisionhumaineexceptionnelle etpréconditions précisées enmaintenance.md.4candidatsrejetés retirésMac/SYS ;3copiesCartae/KS etworktreesgardés conservés sousobservationincomplète ;1worktreederevueSYS retiré aprèsgardescomplètes. Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/198-reprise-start/validation.md.


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
