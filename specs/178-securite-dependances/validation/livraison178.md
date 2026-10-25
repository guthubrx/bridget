# Livraison 178 — Correctifs de sécurité

Date : 25 octobre 2026. Les correctifs applicables à Bridget sont livrés. Le moteur WebView du Pixel reste ancien, car Google Play ne propose pas sa mise à jour. Bridget bloque son utilisation pour les aperçus HTML. La session reste In Progress pour ce suivi externe ; la mitigation est livrée.

## Versions livrées

- Natif : rusqlite 0.41.0, libsqlite3-sys 0.39.0, SQLite 3.54.0 embarqué. Les avis FTS5 identifiés sont corrigés. SQLite 3.54.1 corrige en plus un défaut propre au 32 bits ; les binaires Linux x86_64 et Mac aarch64 de ce lot sont en 64 bits. Android utilise son SQLite système.
- La fonction fallible_uint de rusqlite conserve les conversions vérifiées u64/usize existantes. Le SQL métier et les schémas ne changent pas.
- Android : Gradle 8.14.6, empreinte officielle vérifiée ; Kotlin et plugin Compose 2.4.21 ; AGP 8.9.2 avec R8 9.5.28 explicitement sélectionné, selon la méthode documentée par Android. Les options JVM 17 passent par compilerOptions.
- Plugins et outils de test : BouncyCastle 1.86, Commons Compress 1.28.0, JDOM 2.0.6.1, jose4j 0.9.6, Netty 4.1.139.Final, Protobuf 3.25.8, Tink 1.15.0 et play-services-basement 18.3.0. Le runtime protobuf-javalite ne change pas. Le verrou des plugins est séparé du verrou de l’application.
- Construction et signature Android : Temurin 17.0.21+9, actif officiel publié le 22 octobre, SHA-256 vérifié. scripts/android-build.sh sélectionne ce JDK privé, ou un JAVA_HOME explicitement fourni et vérifié. L’ancien JDK 17.0.20.1 est refusé avec le code 2. Les alternatives Java globales et les processus des autres projets ne changent pas.
- Rustup 1.29.2 installé et vérifié sur Mac et Cartae pour CVE-2026-111943. Compilateurs conservés : Mac 1.92.0, Cartae 1.98.0. La publication Rust 1.99.1 lue décrit des corrections de régressions ; aucun correctif de sécurité applicable à ces compilateurs n’a été établi dans ce lot.
- Caddy : paquet 2.11.7-1+bridget.security178, construit avec Go 1.27.3, x/net 0.60.0 et chi 5.4.0. Les sources standards et les verrous figurent dans deploy/caddy-security. Le paquet officiel 2.11.7 avait encore Go 1.26.8 et x/net 0.59.0 ; il a été remplacé après le scan complémentaire.

## Protection WebView

Le Pixel possède le niveau de correctifs système 2026-10-05 et WebView 155.0.8059.43. La fiche officielle, puis la recherche fraîche des mises à jour Google Play, ne proposent aucun nouveau WebView. La recherche propose seulement ChatGPT, laissé inchangé car hors périmètre. Aucun miroir APK, aucune désinstallation ni inscription à une bêta.

Bridget refuse de créer le WebView pour une version inconnue ou inférieure à 156.0.8078.48, baseline stable officielle relevée le 25 octobre. La garde est commune aux messages et fichiers. Elle relit le paquet à la reprise de l’écran. Le chemin autorisé conserve ses protections : JavaScript, réseau, accès fichiers et contenus désactivés, navigation bloquée, politique de contenu restrictive.

Le refus affiche le code HTML dans le composant texte existant, avec un guidage FR/EN et un bouton vers Google Play. Le moteur ancien ne sert pas à afficher ce code. Le moteur système lui-même reste NON MIS À JOUR ; le risque de rendu dans Bridget est mitigé. Son admission après mise à jour est testée en JVM, sans prétendre une recette physique sur une version indisponible pour ce Pixel.

## Contrôles

- Cargo check workspace : PASS.
- Quatre cibles de persistance, migration et sécurité : 10 PASS, 3 ignorés.
- Stockage : 97 PASS. Mobiles : 69 PASS, 1 ignoré. Aucun succès global de la suite native n’est déduit de ces contrôles ciblés.
- Android : 16 tests JVM PASS, lintRelease et assembleRelease PASS. La compilation de livraison depuis le commit validé reprend ces sorties et régénère les métadonnées Git. Les avertissements R8 sur le parsing asynchrone du provider AGP restent sans échec. Aucun avertissement de code protobuf vulnérable dans le dernier build.
- OSV 2.7.0 : 175 entrées Cargo, 280 entrées du verrou Android et 147 entrées du verrou des plugins. Total de 602 entrées avant déduplication ; aucun avis remonté. L’export réel du classpath des plugins, 143 artefacts, a aussi été scanné sans avis. SQLite C embarqué a été vérifié séparément, car ce scan ne le couvre pas.
- Caddy : govulncheck binaire ne trouve aucune vulnérabilité dans les symboles et paquets importés. Le seul avis de module, GO-2026-5932, concerne openpgp, paquet non importé. x/crypto reste nécessaire pour ses autres paquets ; aucune exclusion d’avis n’a été ajoutée.
- Les dix domaines servis par Caddy conservent leurs états HTTP 200/401/301/308 après activation. Bridget répond toujours 401 sans clé. Le candidat a validé la configuration complète avant activation ; binaire, configuration et unité précédents sauvegardés.
- Les deux bases natives ont été sauvegardées par SQLite backup. Après activation : quick_check OK et aucune anomalie foreign_key_check. Le statut natif est en ligne, build 178-c49d2766.
- Gitleaks sur les deux commits de code réels : aucun secret. Le scan initial du diff brut avait pris cinq sommes Go pour des secrets, faute de contexte de fichier.

## Parcours réel et preuves

Sur le Pixel : conversation de ce fil, menu Fichiers, Bridget-apercu.html, Aperçu. Le refus et les balises du fichier sont visibles, sans nœud WebView dans l’arbre de l’interface. Le bouton ouvre Android System WebView, Google LLC, dans Google Play ; seule la suppression des mises à jour est proposée. Au retour, refus et source restent présents. Association conservée, sans désinstallation. La rotation automatique est rétablie à son état initial 1, rotation 0, et Bridget reste ouvert.

Preuves : diaporama vérifié. Trois étapes et deux touchers relevés. Curseur et cercle orange vus sur la première action. html_preview sans erreur console ; navigation suivante/précédente, dernière étape, lecture/pause et retour à la première étape vérifiés dans le navigateur T3. Originaux, manifestes et traces restent privés.

Diaporama : /home/moi/10.Scripts/64.bridget/preuves-corrections/2026-10-25-webview-securise/preuves.html

Référence T3 lue : /home/moi/11.Repositories/t3code-local/apps/web/src/components/files/BrowserDocumentFrame.tsx et HtmlRenderFrame.tsx. T3 isole les documents dans des iframes avec origine opaque. Il ne gère pas le moteur Android système. Bridget garde son rendu statique plus strict et ajoute la garde avant construction. Pour SQLite, AgentSessionScanner.ts utilise node:sqlite ; aucune liaison rusqlite équivalente. Bridget reste indépendant.

## Sources et installation

Source native : c49d2766b0b9eba61548a107679edb5bc10c30f1, intégrée et poussée dans 51ddcd1a. La livraison documentaire parallèle 0774dffc a été conservée par fusion. Aucun autre delta de code natif.

Source Android finale et preuves : 3edd5f698a722517d5f9255e34f14122a2144dc0, intégrée et poussée dans 3a440911. Aucun changement des crates/Cargo entre la source native et le lot Android final.

- Linux : /home/moi/.local/lib/bridget-communication/178-c49d2766/bin/bridget. SHA-256 170f7c0225bee0a7eebd4703a1d2a9f5ef53f74cd740019c807b58ca0a9d8839. Core 1712494, mobile 1713392, worker 1712497 : liens /proc/PID/exe vers le nouvel exécutable.
- Mac : /Users/moi/.local/lib/bridget-communication/178-c49d2766/bin/bridget. Release 4 min 44 s, signature ad hoc vérifiée. SHA-256 459919c1e706a8d595c03e097969a0c1476037f352f2a334369e009c1a4669da. Pont 460, worker 464 ; application T3 650 et serveur 975 inchangés. Client who et pont status conformes. Les anciennes sessions MCP restent chargées ; aucun autre travail n’a été interrompu pour les remplacer.
- Caddy : /usr/bin/caddy. PID 1687157. SHA-256 aeaf63fa4c9bbfafbb9c62db6db27a59c4823b47da471855bcd024f40b13f128.
- APK signé depuis 3edd5f69 : SHA-256 8d4a872d3b9c389af7ddee7a1e7bb73daa165077edfd1cfbc8c266b2fcec4cf3. Même certificat que la version précédente : 0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588. Mise à jour en place, paquet relu depuis Android et comparé byte-identique, association conservée.

Les premiers essais sans fallible_uint et avec un numéro R8 non publié ont été refusés avant livraison. Le contrôle --version natif ne contenant pas le build-id, l’attestation a été reprise sur les octets et le statut réel. Les activations finales ci-dessus remplacent ces états intermédiaires.

## Entretien et conservations

Installed exécuté sur Mac et Cartae. Cache Mac : 1,24 Gio logique. Sur Cartae, 0,56 Gio d’intermédiaires du staging 174 ont été retirés, puis l’observation incomplète a protégé le reste.

Une observation complète ultérieure a permis de retirer cinq anciennes copies serveur, sans fichier ouvert ni référence dans les unités et configurations actives : 177-r10a-opt, 177-r10b-opt, 177-r10c-opt, 177-r10d-opt et 177-mobile-r8-opt. Volume logique : 186447648 octets. Le cache Go privé a aussi été retiré après garde complète : 884360496 octets. Aucun gain physique n’est déduit de ces tailles logiques.

Actif 178, retour 177-r10e, bases, clés et preuves sont conservés. Le compilateur Go et le JDK corrigé restent disponibles. Sur Mac, observation incomplète pour les anciennes copies et source-mac : aucune suppression forcée. Les copies 177-r10c/r8 sont conservées jusqu’à observation complète.

Les worktrees 177 serveur restent conservés : tâches de session encore ouvertes et revue caffee7b non réattestable depuis ce parent (task_not_found). Le worktree 178 et sa branche restent utiles au suivi WebView ; les caches Android ignorés sont protégés. Aucune branche distante de travail n’a été créée pour 178. Les anciennes sessions MCP et leurs exécutables chargés sont préservés jusqu’à reconnexion.

## Limites et suivi

Seule la mise à jour officielle du moteur WebView reste à faire ; son rendu ancien est bloqué dans Bridget. La session 177, sa publication globale 0.5.0 et ses recettes restantes ne sont pas déclarées closes. Aucun audit global des autres projets, systèmes complets ou images de conteneurs n’est revendiqué. Le routage de délégation était bloqué ; relecture directe, sans revue indépendante ni audit V15.

Preuves privées : /home/moi/.cache/bridget-security178 et /Users/moi/.cache/bridget-security178.

## Sources primaires

- https://www.sqlite.org/cves.html
- https://www.sqlite.org/changes.html
- https://github.com/rusqlite/rusqlite/releases/tag/v0.41.0
- https://docs.gradle.org/8.14.6/release-notes.html
- https://github.com/gradle/gradle/security/advisories
- https://developer.android.com/build/kotlin-support
- https://kotlinlang.org/docs/gradle-configure-project.html
- https://github.com/JetBrains/kotlin/commit/bf51df665b458fda7c3eaf436c4d88dc119d7ec6
- https://github.com/protocolbuffers/protobuf/security/advisories/GHSA-h4h5-3hr4-j3g2
- https://github.com/netty/netty/security/advisories
- https://github.com/adoptium/temurin17-binaries/releases/tag/jdk-17.0.21%2B9
- https://blog.rust-lang.org/2026/10/20/Rustup-1.29.2/
- https://github.com/caddyserver/caddy/releases/tag/v2.11.7
- https://pkg.go.dev/vuln/GO-2026-6809
- https://pkg.go.dev/vuln/GO-2026-6603
- https://pkg.go.dev/vuln/GO-2026-5932
- https://github.com/go-chi/chi/security/advisories/GHSA-j6qf-3m9c-vwj3
- https://chromiumdash.appspot.com/fetch_releases?channel=Stable&platform=Webview&num=3
