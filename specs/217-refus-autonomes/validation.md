# Validation217 — refus expliqués au propriétaire

## Cause constatée

Les tâchesc5904cc7 surCartae et e7aa9c3e surKS ont reçu un refusHTTP400/model_not_found. Le protocole encapsule l’objet fournisseur dans error.message avec codexErrorInfoother. L’ancien transport ne conservait que failed ; la preuve nécessitait alors une recherche de journal par le mainteneur. L’utilisateur confirme avoir choisi un abonnement inadapté lors de sa connexion. Cette information ne doit pas devenir une supposition sur chaque futur refus.

Le catalogue montrait encore huit optionsKS après enabled=false dans les trois registres persistants. Les ports réutilisaient st.registry chargé lors de l’admission antérieure. Le chemin d’admission relisait déjà la politique et refusait les connexions désactivées ; la vue affichée était périmée.

## Correction et contrôles

Source fonctionnelle initialec3840d9e puis complémentaadd7b6da3c126171231d6d08b34825a1ebf1310, version0.6.5. Le transport reconnaît seulement failed + codexErrorInfoother + messageJSON limité à4096octets + typeerror + HTTP400 + invalid_request_error + codemodel_not_found. Aucune phrase libre, identité de compte ou valeur privée du fournisseur publiée. Les statuts succès/interruption, causes inconnues et anciennes erreurs restent inchangés. Aucun compte, modèle ou droit modifié.

Le statut expose une cause fermée et explique directement au propriétaire que cette connexion ne permet pas le modèle demandé. Il donne la vérification du compte et de l’abonnement à demander à l’utilisateur. Aucune réinstallation, reconnexion, substitution ou nouvelle mission automatique. Arrêt, nettoyage et rapport du dossier restent requis avant une nouvelle demande autorisée. Les détails de commande restent réservés au statut propriétaire complet ; les autres travaux suivent leur état.

Les trois ports de catalogue lisent le registre actuel. Toute erreur de lecture est réduite à routing_catalogue_unavailable avant projection. Les codes fermés de politique sont conservés. Aucune écriture de registre, mutation de tâche ou requête fournisseur dans cette lecture.

Dix contrôles ciblés distincts PASS : quatre217 (deuxdaemon, deuxtransport), trois204 et trois212. Deux217daemon relancés après complément final PASS, sans double comptage. Test de transport réellement exécuté avec fournisseur simulé et journal ; aucune fuite du marqueur privé. Contre-épreuve : registre activé en mémoire puis désactivé sur disque, options retirées ; registre invalide ensuite, vue indisponible et aucune ancienne option reprise. Le test appelle aussi le vrai port client négocié mêmeUID et contrôle que son erreur n’expose pas le chemin.

Guides32pages/522liens/13paires PASS ; git diff --check PASS. Pas de nouvelle mission d’essai fournisseur : la cause historique est connue et le contrat est testé avec une fixture de protocole. Correction technique des ports MCP, sans parcours visuel d’application à simuler.

## Revue indépendante native

Première revue fb286b21-1039-4c83-b38e-eb7cd6e38edd, sourcec3840d9e : aucunP1, unP2 de fermeture d’erreur sur le port client mêmeUID. Corrigé dans la fonction commune et couvert sur le vrai port. Deuxième revue59d1c70d-e9d9-4afa-af9c-6631ae239564, sourceaadd7b6d : favorable, aucunP1/P2 concret. Revues statiques Read/Grep sans shell ni test ; aucun PASS indépendant ni activation déduit. Résultats lus/ACK, clôtures natives demandées. Aucun axe non imposé forcé au moteur de délégation.

## Construction et activation

Les premiers builds c3840d9e ne sont pas livrés après le complément. Le candidat final doit porter217-guidance-aadd7b6d. Les compilations Mac etLinux final sont en cours à cette étape ; aucune activation revendiquée ici.

SurSYS, source privée exacte dans /mnt/storage/bridget-build/source217 ; cache existant /mnt/storage/bridget-build/source204/target réutilisé via le script de son checkout propriétaire, --manifest-path explicite. Empreintes des deux scripts build.py identiques78f21ac00c2beb6ce0c5a474e7f5bf19c1e07380510fdb6d19192804a2c42775. Aucun cache copié par test, aucune nouvelle toolchain installée. BuildLinux surglibc2.36 pour la flotte ; buildMacARM64 avec le cache de ce dépôt.

L’APK0.6.5 a été reconstruit : versionCode6005, signaturev3 et lignée existantes conservées, certificatSHA2560c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588. EmpreinteAPK b162fc5337ca87439b7363ba24a48968ff309c135055d8778162f80792ed1542. CodeAndroid inchangé contrev0.6.4 ; aucune installation sur appareil ni nouvelle qualificationUI revendiquée. assembleRelease et lintVitalPASS ; numéro et signature vérifiés par les outilsAndroid existants.

Préavis envoyé aux trois propriétaires, sans demander d’arrêt métier : mcp-58296-6ae42a36-4, mcp-58296-6ae42a38-5, mcp-58296-6ae42a3a-6. Reçusin_flight, pas de lecture déduite. Aucun serviceOpus2D/NE02/Roadie à modifier ; aucun ancien chantier à rejouer.


## Activation vérifiée et publication

Candidat final0.6.5/build217-guidance-aadd7b6d. Linuxx86_64 SHA2566c406322eca2c725546a19d7aea216a2e163140a5d507193a1cdce67896ed530, dépendanceGLIBC maximale2.34. MacARM64 SHA25673cb6433132a9d804d6934ce36d3fe2f0798a86bec68887f63c2784728e6b1c2. Images installées sous /home/moi/.local/lib/bridget-communication/217-guidance-aadd7b6d/bin/bridget et /Users/moi/.local/lib/bridget-communication/217-guidance-aadd7b6d/bin/bridget. Les quatre contrôles clients non vides passent et l’ancien validateur accepte les rapports (Cartae2,SYS2,KS1,Mac4). Ce contrôle ne teste pas l’accès aux modèles.

Sauvegardes cohérentes et arrêt/reprise des seuls composants Bridget. MaîtreCartae784425, workerCartae785983, mobile784491 ; workerSYS446947, workerKS852008 ; workerMac89527, pontMac89573. Images réellement chargées et empreintes vérifiées. Coupure maître2,76s. Updaters restaurés. Un wrapper Cartae2500061 et un wrapperSYS379832 observés préservés par PID et naissance ; aucun wrapperKS actif. Les processus sous usage gardent leur ancien exécutable jusqu’à leur cycle natif de reprise : ne pas prétendre que leurs anciens tours sont réinterprétés. Aucun service Opus2D/NE02/Roadie ni calcul métier redémarré.

Après activation, attestation native réussie et catalogue de flotte déclaré :45options,aucune option CodexKS,gel=false. Une première opération de consignation de la réponse a échoué côté outil ; la relecture de catalogue sans mutation a confirmé ces faits. Aucun nouveau modèle lancé pour ce contrôle. L’ancienne vue de huit optionsKS a disparu. La configurationKS reste disabled après la qualification216 échouée ; aucune correction d’abonnement ni réactivation de modèle n’est déduite de217.

Releasepublique https://github.com/guthubrx/bridget/releases/tag/v0.6.5, id428686347, tag6fcce32b. Six fichiers publiés, empreintes vérifiées par l’API GitHub puis téléchargement public sans authentification. Les exécutables extraits correspondent aux images chargées. L’API publique latest désigne0.6.5. L’installateurpublic est inchangé et vérifié identique au script source ; aucune nouvelle modification Caddy nécessaire. Preuve privée : /Users/moi/.cache/bridget217/public-verification.json.

Entretien installed PASS sur Mac, Cartae et SYS. Caches conservés sous budget :4,83Gio Mac,6,36Gio Cartae,0,97Gio SYS. Deux worktrees de revueSYS retirés par Git sans force après clôture native, absence de fichiers ignorés et vérification d’usage : /home/moi/.local/share/bridget-projects/bridget/repository/.worktrees/fb286b21 et /home/moi/.local/share/bridget-projects/bridget/repository/.worktrees/59d1c70d. Copie de transfert Linux Mac etSYS retirées après contrôle. Les archives, sauvegardes, preuves, actif217, retour214 et anciens exécutables encore utilisés restent conservés. Le témoin216KS reste conservé sous inspection d’usage incomplète ; aucun retrait forcé.

Limite : les clients MCP déjà ouverts peuvent garder leur exécutable214. Leurs appels reçoivent les données et messages du maître217 sans changement de schéma. Les nouveaux lancements serveur emploient217 ; les wrappers existants ne changent pas de parseur à chaud. Aucun ancien résultat ni cause historique non conservée n’est fabriqué.
