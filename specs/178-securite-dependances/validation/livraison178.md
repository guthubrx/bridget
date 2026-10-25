# Livraison 178 — Correctifs de sécurité — 25 octobre 2026
Statut : correctifs et mitigation livrés ; mise à jour du moteur WebView bloquée par GooglePlay. Session In Progress pour ce seul suivi externe.

## Corrections et contrôles

- rusqlite0.41.0/libsqlite3-sys0.39.0, SQLite3.54.0 embarqué en64bits. Les avis SQLite FTS5 identifiés sont corrigés. Le correctif3.54.1 supplémentaire est propre au32bits ; ni la version Linuxx86_64 ni la version Macaarch64 ne sont concernées. fallible_uint maintient les conversions vérifiées u64/usize utilisées par les magasins existants.
- cargo check workspace PASS. Persistance/migrations/permissions :10PASS,3ignorés dans les4cibles explicites ; store::97PASS ; mobile17769PASS,1ignoré. Aucun succès global de suite déduit. Le premier essai rusqlite sansfallible_uint a été refusé à la compilation ; feature de compatibilité activée, puis contrôles PASS. Aucun changement de SQL métier ni schéma.
- Gradle8.14.6 et empreinte SHA256 officielle ; Temurin17.0.21+9 installé depuis les actifs officiels publiés le22octobre, SHA256 vérifié. scripts/android-build.sh sélectionne ce JDK et refuse l’ancien17.0.20.1 (code2 observé). Aucun remplacement de Java global ni arrêt de processus Java étranger.
- Graphes des tests Android corrigés : Netty4.1.139.Final, Protobuf3.25.8, play-services-basement18.3.0 y compris métadonnées. Aucun changement du runtime protobuf-javalite. Verrous complets régénérés. Android14JVM PASS, lintRelease/assembleRelease PASS avec JDKcorrigé. OSV2.7.0 :454dépendances Cargo/Maven contrôlées,0avis remonté, y compris le graphe des outils de test.
- Caddy2.11.7-1+bridget.security178 construit avec Go1.27.3, x/net0.60.0 et chi5.4.0. Dépôt deploy/caddy-security : source minimale standard et verrous Go. govulncheck binaire :0vulnérabilité dans les symboles/paquets importés. Une alerte module GO-2026-5932 concerne openpgp, paquet non importé par le binaire ; x/crypto reste nécessaire pour ses autres paquets. Aucune vulnérabilité ignorée par configuration.
- Les paquets Caddy amont2.11.7 ont d’abord été vérifiés et activés, puis le scan complémentaire de Go a confirmé que leurs composants Go1.26.8/xnet0.59 restaient anciens. Ils ont été remplacés par la construction corrigée ci-dessus. SHA512 de l’actif officiel vérifié avant extraction. La version finale a validé la configuration complète, puis les dix domaines partagés : mêmes étatsHTTP200/401/301/308 avant/après. L’absence de clé Bridget resteHTTP401. Paquet final SHA256 aeaf63fa4c9bbfafbb9c62db6db27a59c4823b47da471855bcd024f40b13f128 pour le binaire. PID1687157 réellement chargé depuis /usr/bin/caddy.

## Portée et limites

Référence T3 réellement lue : /home/moi/11.Repositories/t3code-local/apps/server/src/project/AgentSessionScanner.ts, importnode:sqlite. Aucun mécanisme rusqlite/Gradle/Caddy/WebView système équivalent. Bridget reste indépendant.
La délégation est bloquée par le routage de ce tour. Relecture directe du delta et des graphes ; aucune revue indépendante ni auditV15 revendiqués.
Le changement est technique, sans correction d’apparence ; pas de nouveau diaporama requis. Les parcours réels après mise à jour demeurent à vérifier, sans déduire leur succès des tests.
Les systèmes Mac/Ubuntu complets et les autres projets ne font pas partie de ce lot. Le JDK Ubuntu ancien reste présent pour leurs usages éventuels ; Bridget utilise le JDK privé corrigé.

## Sources primaires

- https://www.sqlite.org/cves.html
- https://www.sqlite.org/changes.html
- https://github.com/rusqlite/rusqlite/releases/tag/v0.41.0
- https://docs.gradle.org/8.14.6/release-notes.html
- https://github.com/gradle/gradle/security/advisories
- https://github.com/netty/netty/security/advisories
- https://github.com/adoptium/temurin17-binaries/releases/tag/jdk-17.0.21%2B9
- https://github.com/caddyserver/caddy/releases/tag/v2.11.7
- https://pkg.go.dev/vuln/GO-2026-6809
- https://pkg.go.dev/vuln/GO-2026-6603
- https://pkg.go.dev/vuln/GO-2026-5932
- https://github.com/go-chi/chi/security/advisories/GHSA-j6qf-3m9c-vwj3

Preuves privées : /home/moi/.cache/bridget-security178 et /Users/moi/.cache/bridget-security178. Les identités, bases et configurations privées ne sont pas intégrées àGit.


## WebView : blocage externe vérifié

Pixel disponible explicitement. Play Store ouvert sur la fiche officielle, puis Gérer les applications et l’appareil, Téléchargements, Rechercher les mises à jour. La recherche fraîche ne propose que la mise à jour ChatGPT (non installée, hors périmètre). Aucun paquet WebView corrigé proposé ; la fiche officielle ne présente que Suppr. mises à jour. Version système observée155.0.8059.43, inchangée. Aucun APK provenant d’un miroir, aucune désinstallation, aucune inscription à une version bêta ni changement de fournisseur. Ce point reste NON CORRIGÉ ; le déploiement officiel Google est bloquant. La date de disponibilité pour ce compte/appareil n’est pas connue.

La rotation automatique du téléphone a été temporairement désactivée pour stabiliser la recette et sera rétablie à son état initial1 (rotation0). Le téléphone sera laissé dans Bridget à la fin.


## Complément : plugins et outils du compilateur

L’export réel du classpath des plugins a ajouté142composants au contrôle initial. Des défauts non couverts par le seul verrou de l’application ont été trouvés : BouncyCastle, CommonsCompress, JDOM, jose4j, Netty, Protobuf et le cache Kotlin. Le correctif Kotlin concerne KAPT (non utilisé par Bridget), mais le plugin a tout de même été actualisé en2.4.21, avec le plugin Compose correspondant. Les options de compilation JVM17 utilisent désormais compilerOptions. AGP8.9.2 et Gradle8.14.6 sont conservés ; R8 est explicitement remplacé par9.5.28 selon le mécanisme documenté par Android pour Kotlin2.4. L’essai9.1.29, seuil de compatibilité indiqué dans la documentation, a été refusé parce que cet actif précis n’est pas publié sur MavenGoogle ; aucun fallback silencieux.

Les bibliothèques des plugins sont sélectionnées avant résolution : BouncyCastle1.86, CommonsCompress1.28.0, JDOM2.0.6.1, jose4j0.9.6, Netty4.1.139.Final et Protobuf3.25.8. Tink1.15.0 remplace les classes protobuf anciennes dont le runtime signalait encore un code généré vulnérable ; aucun avertissement n’est masqué. Le scan OSV du classpath final exporté couvre143composants et ne remonte aucun avis. Le verrou du classpath de compilation est ajouté séparément du verrou de l’application.

Rustup1.29.2 installé et vérifié sur Mac et Cartae : correctifCVE-2026-111943 du gestionnaire de toolchain. Versions du compilateur conservées (Mac1.92.0, Cartae1.98.0) : la publicationRust1.99.1 lue décrit des corrections de régressions, pas un correctif de sécurité applicable établi pour ce lot.

Sources supplémentaires :
- https://developer.android.com/build/kotlin-support
- https://kotlinlang.org/docs/gradle-configure-project.html
- https://github.com/JetBrains/kotlin/commit/bf51df665b458fda7c3eaf436c4d88dc119d7ec6
- https://github.com/protocolbuffers/protobuf/security/advisories/GHSA-h4h5-3hr4-j3g2
- https://blog.rust-lang.org/2026/10/20/Rustup-1.29.2/

## Activation native et données

Source native c49d2766b0b9eba61548a107679edb5bc10c30f1 intégrée àmain et poussée dans51ddcd1a. Une livraison documentaire parallèle0774dffc a été conservée par fusion, sans modification de son contenu. Aucun autre delta de code n’a été intégré.
Linux : build178-c49d2766 installé, SHA256170f7c0225bee0a7eebd4703a1d2a9f5ef53f74cd740019c807b58ca0a9d8839. Core1712494, mobile1713392 et worker1712497 : liens/proc/PID/exe vers le nouveau binaire. Les deux bases sauvegardées par SQLitebackup puis vérifiées ; quick_checkok et foreign_key_check0 après activation. HTTP public sans clé401, statut natif en ligne avecbuild178-c49d2766.
Le premier contrôle d’installation a supposé à tort que --version incluait le build-id : assertion refusée, puis attestation sur les octets embarqués et le statut natif réel. Les unités ont d’abord été relancées sans changement de chemin, avant correction et activation réelle ci-dessus. Aucune base n’a été effacée.
Mac : release4min44s, signature adhoc vérifiée, SHA256459919c1e706a8d595c03e097969a0c1476037f352f2a334369e009c1a4669da. Pont460 et worker464 actifs ; appT3650/serveur975 inchangés. Clientwho et pontT3status conformes. Les anciens processus MCP de sessions déjà ouvertes restent chargés ; aucun autre travail n’est interrompu pour les remplacer. Le prochain montage utilise le nouvel exécutable.
Installed exécuté sur les deux machines. Mac :1,24Gio de caches restants. Linux :0,56Gio supprimé dans le staging174, puis observation incomplète protège le reste. Le scanGitleaks sur le commit source réel ne trouve aucun secret ; le scan préliminaire du diff brut avait pris5sommes Go pour des secrets, faute de contexte de fichier.


## Protection WebView livrée

Le moteur système reste155.0.8059.43, GooglePlay ne proposant pas de mise à jour pour cet appareil. Bridget refuse maintenant de créer le WebView pour toute version inférieure à156.0.8078.48 ou inconnue. La vérification est commune aux messages et aux fichiers. À la reprise de l’écran, la version du paquet est relue ; aucune autorisation ancienne n’est mémorisée. JavaScript/réseau/fichiers restent désactivés dans le chemin autorisé.

Le refus affiche le code source HTML avec le composant texte existant, un guidageFR/EN et le bouton vers la fiche GooglePlay officielle. Aucun moteur ancien n’est utilisé pour afficher ce code. Deux nouveaux tests JVM contrôlent les versions anciennes/inconnues/malformées, la frontière exacte, les correctifs et branches futures :16tests JVM PASS au total. lintRelease et assembleRelease PASS. Deux avertissements R8 signalent seulement l’absence de parsing asynchrone du providerAGP ; compilation terminée normalement, aucun avertissement de codeprotobuf vulnérable dans le dernier build.

Parcours réel Pixel : conversation de ce fil, menuFichiers, Bridget-apercu.html, Aperçu ; le refus et les balises du fichier sont visibles. Aucun nœudWebView dans l’arbreUI. Le bouton ouvre AndroidSystemWebView/GoogleLLC dans GooglePlay, qui n’offre que Suppr.misesàjour. Au retour, le refus et le code restent présents. Association conservée, aucune désinstallation. La version corrigée du moteur n’a pas été simulée : son admission est testée en JVM, pas déclarée testée sur le Pixel.
Preuves : diaporama vérifié, trois étapes et deux touchers relevés, curseur/cercle visibles. Fichier : /home/moi/10.Scripts/64.bridget/preuves-corrections/2026-10-25-webview-securise/preuves.html. Originaux, manifestes et traces restent privés. L’ancienne recette de rendu HTML de177 reste historique ; elle ne vaut pas validation de la garde178.

## Scans et entretien finaux

OSV final :175entréesCargo,280entrées du verrouAndroid et147entrées du verrou des plugins, soit602entrées avant déduplication ;0avis remonté. L’export du classpath des plugins (143artefacts réellement résolus) a également0avis. Tous les composants C embarqués ne sont pas couverts par OSV ; SQLite a été traité séparément. govulncheckCaddy conserve0paquet importé vulnérable et le seul avis openpgp non importé décrit plus haut.

Cinq anciens répertoires de binaires serveur retirés après observation complète linux_system_lsof, absence de fichiers ouverts et de références dans les unités/configurations actives :177-r10a-opt,177-r10b-opt,177-r10c-opt,177-r10d-opt,177-mobile-r8-opt.186447648octets logiques. Version178 active et retour177-r10e conservés. Le cacheGo privé de cette construction a été retiré après garde complète :884360496octets logiques. Le compilateur Go, le JDK corrigé et les verrous nécessaires à une reconstruction sont conservés.
Mac : observation incomplète pour les anciens binaires/source-mac, donc aucune suppression forcée ;178 actif et177-r10e de retour. Les copies177-r10c/r8 sont conservées jusqu’à observation complète. Les worktrees177 serveur restent conservés : la session177 comporte encore des tâches, et le statut de la revuecaffee7b n’est pas réattestable (task_not_found, tâche hors de ce parent). Aucun nouvel agent lancé. Le worktree178 reste utile pour le suiviWebView et ses cachesAndroid ignorés. Aucune référence distante de travail n’a été créée pour178.
