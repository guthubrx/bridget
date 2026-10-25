# Livraison 178 — Correctifs de sécurité — 25 octobre 2026
Statut : In Progress ; activation SQLite et vérification WebView à terminer.

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
