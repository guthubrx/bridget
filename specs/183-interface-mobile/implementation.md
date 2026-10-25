# Implémentation183

Source initiale214a33ecc3de1fafafc623f61477fc5b87c8cc8a, worktree serveur183, un responsable. Les commits seront créés après convergence/audit selon le pipeline autorisé.

## T001/T002

Cadrage et artefacts relus, AnalyzeP1/P2 corrigé, checklist8/8. SDK explicite requis : premier essai sans ANDROID_HOME refusé « SDK location not found », puis ANDROID_HOME=/home/moi/.local/share/bridget/android-sdk scripts/android-build.sh testDebugUnitTest : PASS21s,24tâches Gradle.

Tests de recherche, attention, corrélation compactage et enveloppe HTML préparés dans les classes existantes. Exécution rouge avant code : compilation échoue sur les nouvelles fonctions non encore implémentées, constat attendu ; aucun échec de référence préexistante. Log privé /home/moi/.cache/bridget183-tests-red.log. Le passage vert de ces nouveaux cas reste requis dans les tâches de comportement et T019.

Self-reviewXIX/XX : réutilisation des tests existants, cas sur erreurs et frontières réelles ; aucun nouveau framework. Le rendu HTML et la voix réelle ne sont pas encore qualifiés.

## T003

Signal structurel du tour actif ajouté au transport existant. Test positif et mauvais thread/tour/type/id ; oracle de la vraie session native de test jusqu’au journal, début et fin avec même item ;2testsPASS. La validation fermée du journal a conduit à réutiliser kind=tool/tool_type=context_compaction, au lieu d’ajouter un vocabulaire parallèle. Aucun contrôle d’écriture affaibli. Première compilation a révélé turn_id optionnel ; correction explicite avec as_deref, absence => aucun signal. Le test de bout en bout vérifie aussi l’absence de JournalFailed et le rejet de l’item étranger. Cache commun réutilisé, entretien avant/après exécuté.

Self-review : deux producteurs utilisent la même règle de corrélation ; helper justifié par cette frontière. Aucun résumé de contexte ni pourcentage publié. Les signaux d’un moteur réel en compactage ne sont pas encore observés sur téléphone ; recette183reste requise.

## T004

Projection de compactage structurelle dans l'adaptateur existant, fil racine et tour corrélés, début/fin distincts. Deux tests ciblés PASS59s. Le vocabulaire fermé du journal reste inchangé. Cache commun5,47Gio, entretien exécuté. Les détails fournisseur restent dans son guide d'adaptateur.

## T005

Réducteur pur corrélé session/tour/item ; fins de tour/items mémorisées, perte du suivi et saturation ferment l'indication. La reprise ne relit pas les anciens débuts pour afficher un travail en cours. Normalisation des identités existantes du journal. Helpers purs recherche/attention préparés ici pour exécuter la classe de tests complète, raccordements UI dans T006/T010. Enveloppe HTML pure préparée avant T016 pour compiler tous les tests ; aucune activation JavaScript à cette étape. JVM ConversationTextTest + RichContentTest PASS20s. L'isolation réelle reste à vérifier en T017.

## T006–T009

Repères de lecture conservés dans l'historique chiffré existant, ensemble d'attention par association dans les préférences. Aucun acquittement à l'ouverture du volet ; il faut le fil vivant et son dernier message visible. Transition d'activité seulement entre observations fraîches de même génération. Annuaire projeté avec reconnect_count, sans identifiant de connexion ;2tests de portée/paginationPASS13s. Rafraîchissement groupé visible15s ; pages non rafraîchies marquées anciennes. Volet réutilise AgentDirectory et ProjectAvatar ; Tous/Favoris/recherche, libellés. États de lecture Compose conservés par agent via SaveableStateHolder. JVM completPASS18s. Premier lien de ressources refusé car « ? » est réservé Android ; libellé corrigé sans toucher les règles. Recette UI réelle reste ouverteT019.

## T010/T011

Recherche bornée aux groupes chargés, nombre et navigation précédent/suivant avec mention de couverture. Dictée système réutilisée, texte éditable sans envoi, origine association/agent figée ; annulation et réponses étrangères ignorées. Test taille UTF8 et changement d'association ajouté, JVM completPASS18s. Microphone réel non encore qualifié ; recetteT019reste requise.

## T012–T016 (recette restante)

Préparation dans le stockage chiffré existant, flag durable avant exposition ; client générique refuse send/reconcile tant que préparé. Choix photo et fichier système, cartes retirables, envoi explicite ; reprise des clés et reçus du lot. Transaction locale atomique pour lier message/références, vider le brouillon et retirer les cartes. Instrumentation ajoutée sur refus sans réseau, octets chiffrés relus, retrait, stabilité de l'enveloppe et rollback de composition ; elle est compilée mais pas encore exécutée.

Rendu CommonMark en blocs racine sans doublon ; texte exact du code conservé pour Copier, langage et carte HTML. Listes/citations gardent leur AST et leur coloration ; le contrôle Copier s'applique aux blocs racine, les codes imbriqués restent sélectionnables. JVM complet et APK de test compilésPASS25s.

ADR183écrite. Trois modes aperçu : interactif isolé, statique et code. Une seule fabrique WebView porte les refus, aucune interface native ni session, iframe opaque échappée et CSP avant contenu. Le test hostile réel vient d'être ajouté, son résultat reste requis avant validationT017.

## Observation réelle de l'annuaire

La recette native a montré observed_age_secs=249 malgré les heartbeats. Lecture de daemon.rs5629confirme qu'il s'agit de capacity_seen, âge du dernier fait d'activité, pas âge de la connexion ou de la réponse HTTP. Le volet ne doit donc pas demander une actualisation impossible pour une activité simplement ancienne. Fraîcheur liée au rafraîchissement réussi et au cache ; âge du dernier signal affiché séparément au-delà de90s. La transition d'activité exige toujours deux observations de même génération. Ce diagnostic repose sur la réponse runtime réelle puis le code producteur.

## Recette de conversation : stabilité du défilement

Capture réelle après chargement du journal : les blocs colorés augmentaient leur hauteur après le défilement initial, et le dernier bouton d'aperçu restait sous la zone visible. Le suivi du bas observe maintenant la taille du dernier item réellement mesuré, sans réagir à ses seuls déplacements, et s'arrête pendant une traction manuelle. Le test d'usage et la position conservée doivent confirmer cette correction. Deux premiers essais automatiques ont également exposé des défauts du banc (attente de la première fenêtre et du volet, Subscribe du producteur contrôlé). Une lecture API réelle après correction du banc retourne39événements en0,1s pour chaque agent. Ces erreurs du banc ne sont pas des résultats de produit.

## Fil masqué par le volet

Lecture de l'appelant ModalNavigationDrawer : le fil restait composé pendant l'ouverture du volet. Il doit cesser son suivi et son acquittement tant qu'il est masqué, même si sa dernière ligne était auparavant visible. La visibilité transmise inclut maintenant l'état courant et la cible d'animation du volet. Le rafraîchissement groupé de l'annuaire reste actif. Le fil reprend depuis son curseur à la fermeture. Cela ferme aussi une indication de compactage masquée sans inventer un début ancien.

Le troisième parcours automatique a été arrêté, sans succès revendiqué : Android a produit un timeout du service cpuinfo après l'arrêt de l'application. Le diagnostic par debuggerd demande root sur cet émulateur GooglePlay ; aucun contournement. Redémarrage du système Android sans effacement avant nouvelle qualification.

## Géométrie de l'aperçu — cause observée

Le parcours dédié a mesuré un WebView1080x1904, affiché et JavaScript actif, mais iframe.height=0 alors que innerHeight=725CSSpx. La capture était blanche et l'arbre accessible gardait le bouton. Le dimensionnement en pourcentage dépendait de la hauteur du document dans cette fenêtre. Le cadre adopte maintenant une position fixe et100vw/100vh ; test de géométrie ajouté au parcours Dialog et au banc d'isolation. Le prochain passage doit vérifier hauteur réelle, peinture et toucher0->1. Aucun succès du rendu complet n'est revendiqué avant ce passage.

Bornes revues : rafraîchissement silencieux limité à1000agents comme la lecture manuelle, dépassement guidé ; qualificatifs de projet calculés une fois sur p<=256, puis accèsO(1) dans chaque ligne.

### Contraintes natives confirmées

Deuxième mesure : layout_height=-2 et layout_width=-2 (WRAP_CONTENT), parent ViewFactoryHolder. La géométrie CSS est0px en mode standard malgré un clientHeight725. Le banc qui fonctionnait utilisait explicitement MATCH_PARENT dans addContentView. La fabrique native doit donc déclarer MATCH_PARENT avant son chargement. Le changement CSS100vh infructueux est retiré, sans empiler une hauteur forcée ; seule la disposition bloc du cadre complète la politique initiale. Le parcours utilise maintenant de vrais événements tactiles sur les rectangles relevés, avec coordonnées et captures avant geste. Les premiers dix changements étaient déjà vérifiés via accessibilité ; leur nouveau passage couvre le circuit tactile.

### Résultat du dimensionnement natif

MATCH_PARENT donne une iframe725,33px pour un viewport725px ; capture réelle du titre, du texte et du bouton bleu, alors que la même fenêtre était blanche avant. Le contrôle géométrique passe. Le geste synthétique construit manuellement n'a pas déclenché le bouton WebView. Le contrôleur utilise désormais la commande Android input tap, sur les rectangles réellement relevés, comme le pilotage manuel ADB ; aucune modification de la page pour simuler son succès. Les suites stockage/isolation étendues restent à relire avant validation finale.

## T017 — passages positifs

Après correction native, parcours tactile complet d'aperçuPASS23,85s : géométrie visible725,33px, compteur0->1 par Android input tap, statique, code, retour au brouillon exact. Stockage et isolation étendusPASS4tests/10,84s : refus parent, stockage local/session/cookies, pont natif, réseau/file/content/WebSocket, worker, popup, permissions et dialogues ; source et fermeture native en cas d'erreur de script. Le document n'obtient aucun handler privilégié.

Parcours tactile combinéPASS49,98s :10changements entre deux agents, brouillons distincts, recherche précédente/suivante, résultat de dictée contrôlé éditable sans commande ajoutée, compteur et retour. La voix réelle reste une qualification distincte, non revendiquée. Les traces et captures originales restent privées sur l'émulateur et le Mac avant diaporama.

Priorité revue sur favoris anciens : les demandes effectivement relues possèdent un ensemble d'identifiants frais distinct de la fraîcheur de l'annuaire. Une question native fraîche garde son marqueur même sur un favori chargé depuis le cache. Une lecture échouée n'invente pas une question active. Aucun appel réseau nouveau ; la relecture bornée existante est réutilisée.

## Régression Android élargie

Première passe :18/20PASS. Deux fixtures antérieures étaient obsolètes : un test exigeait le refus du MIMEHTML alors que les aperçus/fichiersHTML sont déjà pris en charge ; un exemple de publication omettait le digest devenu obligatoire. Le premier vise désormais un exécutable non autorisé, avec un cas positif HTML préparé dans QueueStoreTest. Le second fournit un digest synthétique valide et vérifie séparément le refus si digest absent. Aucune protection de production relâchée. Nouveau passage requis.

Frugalité : les entrées autorisées du fil sont calculées une fois et partagées au rendu du message, à la file et aux cartes ; les contrôles d'origine restent réexécutés sur les effets. Une variable inutilisée dans la préparation a été retirée. Les diagnostics de test conservent géométrie et contraintes natives ; la copie superflue du document HTML complet est retirée.

## Lecture réellement visible

La même règle de visibilité couvre maintenant le volet, l'aperçuHTML, la question agrandie, l'éditeur de file et les détails. Les surfaces enfants signalent seulement leur ouverture/fermeture ; le fil reste l'unique responsable du suivi et de la lecture. Un ensemble de clés évite qu'une surface ferme le masque d'une autre. Aucune lecture n'est acquittée sous une surface modale. Les boutons de la file passent à48dp réels.

La reprise du même fil relit une page de journal autorisée avant le suivi long. Cela conserve les messages et la position, sans laisser Connexion pendant25s à la fermeture d'un aperçu. Les débuts de compactage de rattrapage ne sont pas réactivés ; seuls les signaux reçus après retour à un suivi vivant sont présentés en cours. Une reprise de50événements reste en synchronisation jusqu'au rattrapage.

## Nouvelle compilation et contrôles de banc

Les deux paquets signés ont été réinstallés après vérification du signataire. Régression Android élargie20/20PASS23,91s. Le parcours long final échoue sur un repère UiAutomation : les contrôles natifs de l'aperçu apparaissent, mais le contenu de l'iframe n'est pas exposé dans son arbre. Capture réelle puis geste Android sur le bouton visible : compteur0->1 observé ; statique/code/retour fonctionnent. Le test de parcours conserve la géométrie réelle et le retour, sans prétendre lire une valeur absente de cet arbre. Le compteur reste qualifié par l'instrumentation réelle d'isolation et le circuit tactile illustré. Les tentatives échouées restent conservées privées.

Une erreur de préparation précédente avait permis la poursuite après refus de signer le testAPK ; ce passage est invalidé. Le garde de fraîcheur distingue maintenant les sources de l'application des sources de son testAPK. Les phases signature/installation sont vérifiées séquentiellement.

Actualisation : une panne de questions ne doit pas court-circuiter l'observation de l'annuaire. Les deux contrôles restent indépendants dans la boucle existante. Version de livraison0.6.3, aucune dépendance modifiée. Compilation JVM/debug/tests/releasePASS49s ; version finale et signatures restent à contrôler.

La recette de photo a confirmé le stockage séparé puis la remise des références au bon agent. Elle a aussi exposé une présentation trop technique dans la bulle envoyée : UUID et empreinte étaient affichés. Le rendu reconnaît maintenant uniquement la ligne canonique complète et présente le nom de pièce jointe. L'original reste inchangé dans le message, les reçus et la vue technique ; les lignes inconnues restent visibles intégralement. Tests positif, auteur assistant et format inconnu ajoutés. La recherche vide de l'annuaire reçoit aussi son libellé propre, sans conseil trompeur d'actualisation.

## Garde de version avant livraison

Le badgingAPK a refusé le candidat :6002/0.6.2 au lieu de6003/0.6.3. La modification de version avait visé le checkout serveur principal à cause d'un calcul de parent incorrect. Diff relu : seuls Cargo.toml et les trois packages internes du lock étaient concernés. Le checkout principal a été restauré octet pour octet depuis HEAD après comparaison avec cette seule modification. Les mêmes changements sont appliqués au worktree183 explicitement nommé. Aucune livraison du candidat refusé. Nouvelle compilation0.6.3 et contrôles du badging/binaire requis.

Retour Android dans le volet : deux gestes réels ont laissé le volet ouvert. La fermeture par sa croix a révélé l'annuaire derrière : le BackHandler de l'appelant désélectionnait l'agent avant de fermer le volet. Il traite désormais explicitement le volet ouvert/en animation avant la sortie du fil. Refaire le geste Retour sur la nouvelle compilation ; ne pas déclarer cette correction validée avant ce passage.

## Recette consolidée

Compilation finale6003/0.6.3 vérifiée par badging. SourceLinux candidat0.6.3. JVM30/30 ; instrumentation de parcours1/1PASS51,587s, copie exacte du presse-papiers comprise ; régressionAndroid20/20PASS10,809s. Le geste Retour ferme maintenant le volet et garde Atelier/brouillon. Le message de photo présente le nom, sans métadonnées internes ; le destinataire contrôlé conserve les références canoniques. Recherche d'agent vide : libellé spécifique observé.

Le producteur de recette privé diffuse désormais ses fragments vivants après sa capture initiale ; son ancien mode ne faisait que rejouer le journal et retardait artificiellement les observations. Aucun changement du protocole de production pour corriger ce banc. Cycle natif en direct : début/fin de compactage visibles ; nonlu conservé sous volet et après changement de police ; suppression seulement après accès aux derniers messages. Position de lecture identique après allerretour entre agents. Réglagefont1,0 rétabli.

Diaporama30étapes/28images : captures et gestesAndroid réels, tests de navigation/lecture/pause/zoom/dernière étape ;28images décodées ; aperçus360/1000px. Captures originales/manifeste horsGit. Les légendes distinguent les gestesAndroid des événements souris.

Le premier miroir de compilationMac n'avait pas reçu le diff : Git retrouvait le dépôt du compte parent et ignorait les chemins hors de son préfixe. Le candidat de socle est rejeté et noninstallé. Patch appliqué par l'outil de fichiers puis six empreintes critiques comparées aux sources serveur ; candidat0.6.3 recompilé. La version doit encore être vérifiée à sa sortie. Les exécutables définitifs seront reconstruits depuis le commit intégré, jamais depuis ce miroir sans métadonnéesGit de projet.

T021 : candidatMacbridget0.6.3 vérifié après compilation2m23s et comparaison de sources ; entretien depuis le checkout durable a contrôlé le cache commun, sous10Gio. Les vérifications préalables de version, signatures et procédures sont terminées ; installation et identité du commit final restent des gates distinctes de livraison. Aucun candidat de socle ou d'identitéGit impropre n'a été installé.

Correction d'auditM1 : la recherche des appels confirme qu'htmlCodeBlocks n'était plus utilisé en production après la découpe CommonMark. Son unique appel restait dans un test hérité. Suppression de l'extracteur et de deux imports ; le test vise désormais markdownRenderBlocks, distingue balise mentionnée et bloc réel et vérifie dix blocs sans troncature. Avant et recherche des appels conservés dans la session d'audit. Aucun changement visible attendu, car le rendu n'appelait pas ce code. T022 ajoutée, puis vérification et convergence supplémentaires requises.

T022 vérifiée :13lignes de production retirées, aucun appel résiduel à htmlCodeBlocks ; test reporté sur l'AST réellement affiché ; JVM30/30 et assemblagedebug/tests/releasePASS39s. Le code retiré n'était appelé par aucune interface : les circuits visibles et leur diaporama restent pertinents. Le changement reste une correction technique à comportement constant.
