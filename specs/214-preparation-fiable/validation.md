# Validation214

## Contrôles avant revue

- Reproduction fermée du parent réel213 dans la fixture distante : PASS. Parent natif provider_turn, plein accès, preset exact, seule approbation locale T3, callback attesté, aucun launch_context.
- 33 tests du module droits149/162/171/214 PASS, 7 recettes privées ignorées explicitement. Réduction discovery sans écriture/réseau, règles tierces et refus d’outils restent refusés.
- 2 tests214 PASS (recoupement du test de droits ci-dessus), dont guidage du refus public sans inventer de changement de droits.
- 36 tests de l’installateur PASS, dont sept214. Vrai installateur en pseudo-terminal isolé : y/n/n/n, quatre propositions, seul programme Claude de fixture installé, aucun fournisseur lancé.
- Les premiers échecs du banc d’essai venaient d’un Gemini installé sur l’hôte puis du gzip manquant dans le PATH isolé. Le PATH contient maintenant seulement les utilitaires nécessaires et les programmes simulés ; aucune correction produit n’a été justifiée par ces échecs.
- sh -n et git diff --check PASS ; guides32/522/13 PASS.
- Preuve du parcours terminal : /Users/moi/Nextcloud/10.Scripts/64.bridget/preuves-corrections/2026-10-29-preparation214/preuves.html. Deux étapes textuelles, aucun clic car parcours clavier. Aperçu et navigation suivant/précédent vérifiés. Installateurs et service simulés, pas de preuve d’authentification ni de mission réelle dans ce diaporama.
- Première compilation de test a dépassé le budget de cache et son entretien automatique a purgé les intermédiaires ; contrôle ciblé --lib avec debug=0 ensuite, cache observé2,21Gio. Pas de Cargo direct.

Aucune activation214 ni reprise Opus2D encore attestée à ce stade.

## Complément après revue

Revue native8c1b76e4-c879-4a26-9dbf-258eaf1c8b62 lue/ACK/clôturée. AucunP1 ; ordre installateur corrigé avant livraison. Le contrôle de support des programmes présents et prepare_service passent avant toute installation choisie ; après un programme neuf, même contrôle de support avant lien/profil/démarrage. Trois contre-épreuves ajoutées : fédération refusée sans installation fournisseur, client existant incompatible sans installation, client neuf incompatible sans bascule Bridget. 39 tests installateur PASS, dont10session214. Les tests initialement attendus « aucune commande service » ont été corrigés après lecture de la trace réelle (show-environment/show uniquement) : une sonde de lecture n’est pas un démarrage. Aucun service actif ni profil modifié dans ces contre-épreuves.

Le commentaire formalise l’authentification exigée de l’appelant et le callback plan_exit non applicable au protocole Codex ; aucune autorité nouvelle du callback n’est exportée. Sans origine attestée, le cas natif fermé retourne maintenant permission_attestation_unavailable, testé exactement. Le guidage candidat précise la portée et les conditions not_recorded/aucun appel concurrent/rejeu strict. Les autres états conservent tâche et clé. Deux tests214 relancés PASS après complément. Gemini suit le paquet officiel npm stable non épinglé, documenté ; aucune prétention de téléchargement préalable d’un script pour cette branche.

Le lien public cartae.app/bridget/install.sh pointe encore sur d09d39398ff2f48af3805b5701682b126b2f916a au contrôle initial. Sa redirection doit être publiée vers le commit final vérifié, sinon la nouvelle option ne serait pas utilisée par ce parcours. Aucun autre routage web ne doit être modifié.

## Contre-revue et paquet public

Contre-revue1a27b0e3-f052-49c2-acbf-a0626bc62762 : aucunP1. Le conseil générique d’upgrade en cas de registre inconnu a été corrigé : lire la cause affichée ; une mise à jour n’est évoquée que si le rapport atteste un client trop ancien. Test ajouté pour client_machine_scope_unavailable ;40 tests installateur PASS. Les points d’authentification appelante et de relecture Codex sont des contrats historiques, pas des défauts constatés par la contre-revue ; les gardes avant Release demeurent.

Le vrai paquet publié0.6.3 MacARM64 a été téléchargé et vérifié par son sidecarSHA2568d3f5bb859717505b158786ee94fffe89f9b8d300826d033e07cae91c4d9c2e7. Son binaire rend exactement bridget0.6.3 et clientscheck sort avec2 (sous-commande inconnue). HOME et namespace isolés, aucun daemon, compte ni fournisseur lancé. Le doute de compatibilité historique est levé pour cet artefact ; les fixtures restent limitées à leur portée.

Publication0.6.4 préparée : seuls versionworkspace, trois entréesworkspace du lock et changelogs changent pour le paquet. Les programmes et comptes fournisseurs ne sont pas modifiés. Les binaires2140.6.3 de qualification ne sont pas présentés comme des paquets0.6.4.
La compilation Cartae porte GLIBC2.39 ; la cibleSYS demande un build sur sa plateformeGLIBC2.36. Rust1.92 existe déjà sous /home/moi/.cargo/bin mais n’était pas dans le PATH SSH initial ; aucune absence de toolchain ni nouvelle installation n’est déduite. Source privée exacte sur /mnt/storage/bridget-build/source214 et cache existant source204/target réutilisé. Une fois construit et vérifié, son binaire compatible sera commun aux trois serveursLinux.

## Activation et usage réel

Source fonctionnelle compilée30581503bf9eac47211bf154b43160b9803189f5 ; version0.6.4, build214-clients-30581503. Revue finale6c2adcf0-d31f-4b97-b1b6-74b249ef22c6 favorable, aucunP1/P2 concret. Résultat lu/ACK/clôturé comme les deux revues précédentes ; aucune revue statique n’est transformée en qualification runtime.

Binaire Linux construit surSYS/glibc2.36, symboles requis jusqu’à2.34 : SHA256ebb9ad5efbc534e7b446f1efec59f7be20034b97d7c82c105be7e5536bcfbd45.
Binaire MacARM64 : SHA25601cd1e490f9bc831fc5796491ce594fea87ae45061883b65ff447609d1def47e.
Installés sous /home/moi/.local/lib/bridget-communication/214-clients-30581503/bin/bridget et /Users/moi/.local/lib/bridget-communication/214-clients-30581503/bin/bridget.
Les quatre contrôles clients sont compatibles avec inventaires non vides (Cartae2,SYS2,KS1,Mac4), revalidés par le binaire installé précédent. CodexKS reste désactivé : son compte n’a pas été essayé ni réactivé.

MaîtreCartae3414383, workerCartae3416760, mobile3414449 ; workerSYS378844 etworkerKS694607 ; Macworker75024 etpont75068. Images réellement chargées et empreintes vérifiées, pas seulement liens installés. Updaters restaurés.
Sauvegardes cohérentes puis arrêts natifs : coupure maître2,16s ; trois wrappers Cartae et un wrapperKS observés préservés par identité de naissance. Aucun wrapperSYS actif au contrôle. Aucun service Opus2D/NE02/Roadie arrêté et aucun calcul métier relancé. Les reçus privés sont dans /home/moi/.cache/bridget214/{core-loaded.json,worker-loaded.json,candidate.json} et /Users/moi/.cache/bridget214/mac-loaded.json.
UNE bridget_session et UNE bridget_capabilities après activation : lien disponible, fleet declared45options, delegations_halted=false. Les MCP ouverts avant livraison restent à distinguer des services rechargés.

Campagne propriétaire unique214-reprise-terra-opus2d-20261029 : fil existant32e1b8f8, clientRequestIdbridget214-reprise-terra-opus2d-20261029-01, tour591. Après UNE réattestation, UN rejeu strict de chaque enveloppe originale213 a admis les mêmes demandes, sans nouvelle clé ni moteur de remplacement :
- qual213-opus2d-20261029-terra-cartae → edbeeaea-f753-4634-a12b-525f56b363ce → QUAL213-OPUS2D-TERRA-CARTAE-OK ;
- qual213-opus2d-20261029-terra-sysle2 → ca5c7ba3-d66e-44d9-a3de-07379f73ccdf → QUAL213-OPUS2D-TERRA-SYSLE2-OK.
Même base/base_ref d96b0b7ec88bce5b1ffdeb1a9ddea67ecb021de4, worktree/development. Résultats réels reçus par le propriétaire et relus au registre maître : result_available,errornull,guarded=false,cleanup_done=true. Aucun acquittement fait en son nom. Les autres modèles ne sont pas tous qualifiés par ces deux témoins.

Les trois propriétaires ont reçu des avis de maintenance puis de reprise. MessagesBridget de fin mcp-35474-6ae39e6a-11, mcp-35474-6ae39fa0-12 et mcp-35474-6ae39fa2-13 : reçus in_flight, pas de lecture déduite. Le fil natif monuments confirme la reprise effective.

## Paquet Android cohérent

Une release publique devient aussi la source de mise à jour Android. Ne pas réétiqueter l’ancienAPK0.6.3 en0.6.4 : le client contrôle versionName et versionCode. L’APK a été reconstruit avec le Cargo0.6.4, configuration Firebase publique existante et identité de signature existante, sans modification du codeAndroid (diff contrev0.6.3 vide).
assembleRelease, lintVital et30testsJVM release PASS. VersionName0.6.4,versionCode6004,packageapp.cartae.bridget.
CertificatSHA2560c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588 identique au paquet0.6.3 publié ; mêmes schémas de signature (v3 uniquement) et même lignée existante. Premier appel sign refusé faute de plus ancien signataire pourv1 ; lecture de l’ancienAPK et des options officielles a établi quev1/v2 étaient déjà désactivés. Les mêmes options ont été utilisées, sans changement de clé ni réinstallation d’application.
APK SHA2562ec44aa47d559abcee8620e428c673ec4d3e47435d824318be22b662e682e410.
Aucun téléphone n’a été désinstallé ni mis à jour de force. Cette recette n’est pas une nouvelle qualification exhaustive de l’interface mobile.

Sources : https://source.android.com/docs/security/features/apksigning/v3 et https://developer.android.com/tools/apksigner.

Publication publique, téléchargement et nettoyage : clôture ci-dessous.


## Clôture de publication et de livraison

La version0.6.4 est publiée et définie comme dernière release : https://github.com/guthubrx/bridget/releases/tag/v0.6.4. Le tag désigne c77a2a4048b8bdfde0fb54b48f700f0666d0cf00. La source exécutable reste30581503bf9eac47211bf154b43160b9803189f5 ; le complément est documentaire. Le diff exécutable jusqu’au tag est vide.

Les six fichiers publics ont été téléchargés ; leurs empreintes concordent avec les fichiers préparés. Les exécutables extraits concordent avec les images chargées. Preuve privée : /Users/moi/.cache/bridget214/public-verification.json. Les archives et le paquet Android portent réellement0.6.4 ; aucun téléphone n’a été mis à jour de force.

Le lien https://cartae.app/bridget/install.sh sert maintenant le script exact30581503, SHA2568f0b5743f595e7f098d49c8f31c20c45d03336aaf8d6b6444f431b969d07fbe5. Une seule redirection Caddy a changé, après sauvegarde et validation. Rechargement sans redémarrage du processus Caddy ; autres routes conservées.

### Limite des anciens clients MCP

Après UNE reprise native, UNE attestation et UNE lecture du catalogue réussissent : flotte déclarée,45options,gel=false. Les appels de ce fil restent cependant servis par le MCP211/PID35474 : reçus mcp-35474-6ae3a6cd-16 et mcp-35474-6ae3a6d0-17. Deux autres processus214 visibles ne prouvent pas le remplacement de cette connexion.

Le marqueur non autoritaire essayé dans la configuration Codex a été retiré. Configuration restaurée à l’octet près ; commande,HOME,socket et namespace inchangés. Aucune seconde reprise, aucun arrêt du serveur fournisseur partagé, aucune mission métier relancée. Preuve : /Users/moi/.cache/bridget214/mcp-reload-attempt.json.

Les services sont chargés214. Les anciens MCP211 de ce fil,209 de monuments et les autres clients ouverts ne sont pas déclarés rechargés. Les deux admissions Terra réussies prouvent le fonctionnement corrigé avec ces anciens clients. Pour renouveler les connexions conservées par l’application : terminer le tour courant, quitter complètement T3 Code avec Cmd+Q, rouvrir T3 Code, puis reprendre les fils existants. Une simple reprise de tour n’a pas suffi. Ce geste reste à faire ; aucun redémarrage global implicite n’a été lancé. Aucun travail ni résultat à supprimer ; aucune mission d’issue inconnue à recréer.

### Nettoyage et conservation

Trois revues214 terminées, guarded=false/cleanup_done=true : dossiers propres, sans fichiers ignorés, sans processus ni ouverture lsof. Retrait normal git worktree remove, sans force :
- /home/moi/.local/share/bridget-projects/bridget/repository/.worktrees/8c1b76e4
- /home/moi/.local/share/bridget-projects/bridget/repository/.worktrees/1a27b0e3
- /home/moi/.local/share/bridget-projects/bridget/repository/.worktrees/6c2adcf0

Leurs branches locales ont été retirées après preuve d’intégration à origin/main et absence de worktree. Branche de session214 localeCartae et distanteGitHub retirée après intégration. CandidatMac56c706c7 jamais actif, sans usage, retiré. Copie de transfertSYS retirée après empreinte et usage vérifiés ; copiesCartae/KS conservées car usage non entièrement exclu. Actif214, retour212, anciens exécutables209/211/198 encore utilisés et worktrees sous observation incomplète conservés.

Entretien installed réussi sur Mac et SYS. Sur SYS, le premier appel depuis source214 a refusé le cache dont le propriétaire est source204 ; l’appel depuis source204 a ensuite réussi. Aucun registre de cache modifié, aucune compilation refaite. Main et tag poussés aux miroirsSYS/KS ; les anciens checkouts sous usage ne sont pas réinitialisés. La clôture documentaire peut avancer main après le tag sans changer les exécutables publiés.

Les trois propriétaires ont été informés de la reprise. Messages finaux : monuments mcp-35474-6ae3a66a-15, Politique sol mcp-35474-6ae3a6cd-16, trajets bus mcp-35474-6ae3a6d0-17. Reçus in_flight, pas de lecture déduite. Le résultat réel propriétaire des deux demandesTerra prouve la reprise indépendamment de ces messages.

CodexKS reste désactivé après401 attesté. Aucun essai ni authentification supplémentaire. Action utilisateur ciblée : ssh -t ksstore '/home/moi/.local/bin/codex login --device-auth'. Une connexion réussie doit précéder qualification distincte et réactivation. Aucun autre modèle non testé n’est présenté comme qualifié.

Contrôles acquis, sans nouvelle exécution à la clôture :34testsRust distincts (7recettes privées ignorées),40installateur,30JVMAndroid, guides32/522/13 et revue finale favorable. Preuves : diaporama vérifié pour le terminal isolé ; il ne prouve pas l’authentification. Les deux admissionsTerra réelles complètent cette preuve technique.
