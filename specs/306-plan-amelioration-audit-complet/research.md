# Recherche technique : Programme complet d'amélioration post-audit V15 (306)

**Date** : 2026-11-20
**Périmètre** : `/Users/moi/Nextcloud/10.Scripts/64.bridget`

## 1. Dépendances vulnérables (Go Caddy, Rust, Android)

- **Décision** : Mettre à jour `deploy/caddy-security/go.mod` vers la dernière version corrigée de Caddy v2 et de ses dépendances transitives (`go-jose/v3`, `go-jose/v4`, `klauspost/compress`, `opentelemetry-go`, `golang.org/x/crypto`, `golang.org/x/net`), remplacer `_ "github.com/caddyserver/caddy/v2/modules/standard"` par le sous-ensemble minimal de modules Caddy nécessaires au reverse proxy TLS/HTTP, mettre à jour `pulldown-cmark` dans `Cargo.lock` et ajouter une contrainte `resolutionStrategy` sur `com.google.guava:guava` dans `android/app/build.gradle.kts`.
- **Justification** : OSV-Scanner a confirmé 11 CVE dans `deploy/caddy-security/go.mod` (dont 2 HIGH sur `go-jose` RSA JWK DoS) et 4 avis non atteignables en runtime dans `Cargo.lock` et `android/*.lockfile`. `go 1.27.3` et `osv-scanner` sont installés localement.
- **Alternatives considérées** : Ignorer les CVE Go dans `.osv-scanner.toml` (refusé : le proxy Caddy est exposé en frontal de la passerelle mobile).
- **Impact mainteneur futur** : Réduit la surface Go embarquée et ajoute un scan CI bloquant pour éviter toute dérive silencieuse.

## 2. Signature détachée des livraisons (D8)

- **Décision** : Utiliser `ssh-keygen -Y sign` et `ssh-keygen -Y verify` (natif OpenSSH présent sur macOS et Linux) avec un fichier `allowed_signers` configurable (variable d'environnement ou chemin explicite, sans clé privée de production commise dans Git), intégré à `scripts/install.sh`, `scripts/update.py` et un utilitaire de signature réutilisable testé avec clés éphémères Ed25519.
- **Justification** : `minisign` et `cosign` ne sont pas installés sur le poste ; `ssh-keygen` est natif sur macOS et Debian 12 et ne rajoute aucune dépendance externe.
- **Alternatives considérées** : Installer `minisign` ou ajouter une dépendance Rust tierce (refusé par l'Article XIX et la portabilité de `scripts/install.sh`).
- **Impact mainteneur futur** : Zéro dépendance binaire supplémentaire ; l'activation en production ne demande qu'une clé publique de confiance.

## 3. Boucle d'acceptation Unix, cache `AgentRegistry` et `DaemonState`

- **Décision** :
  - Remplacer `thread::sleep(50 ms)` dans `daemon.rs:7226` par `libc::poll` sur le descripteur du `UnixListener` et un tube non bloquant (`pipe`) réveillé lors du shutdown.
  - Plafonner les connexions simultanées par un compteur atomique borné (256 connexions générales + réserve de 32 pour workers/admin) avec garde RAII décrémentant à la sortie du thread.
  - Mettre en cache `AgentRegistry::load*` par `(PathBuf, mtime, len, inode)` dans un `RwLock` statique/partagé afin de servir les >20 appels chauds sans relecture disque lorsque le fichier n'a pas changé.
  - Découper les lectures majoritaires de `DaemonState` (annuaire/présences/catalogue) derrière des verrous `RwLock` dédiés pour diminuer la contention sur le verrou exclusif global.
- **Justification** : Supprime jusqu'à 50 ms de latence par commande locale, 20 réveils CPU/s au repos, et la contention de lecture/disque sous verrou global.
- **Alternatives considérées** : Migrer tout le daemon synchrone vers Tokio async (refusé : refonte trop risquée et coûteuse alors que `poll(2)` + `RwLock` ciblés résolvent la cause directe sans changer le modèle d'exécution).
- **Impact mainteneur futur** : Code court, isolé et testable sans modifier le protocole Unix.

## 4. `bridget-core` (`EnvelopeGuard`, `Deduplicator`, `CircuitBreaker`)

- **Décision** : Passer `EnvelopeGuard` et `Deduplicator` sur une file chronologique (`VecDeque`) avec capacité bornée, éviction O(1) amorti en tête et `checked_sub` ; indexer `Deduplicator` par `(content_key, target)` pour éliminer la collision de cibles ; purger les clés vides dans `CircuitBreaker::history` avec `checked_sub` ; encadrer les enveloppes inter-agents (`envelope.rs`) avec un délimiteur unique lié à l'identifiant du message.
- **Justification** : Élimine les scans O(n), les fuites mémoire sur agents éphémères, le bug de déduplication multi-cibles et la confusion données/instructions inter-agents.
- **Alternatives considérées** : Dépendance `lru` externe (refusé : `VecDeque` + `HashMap`/`HashSet` standard de `std` suffit en zéro dépendance).
- **Impact mainteneur futur** : Invariants O(1) et bornes explicites vérifiés par tests unitaires dans `bridget-core`.

## 5. SQLite unifié, passerelle mobile Axum et `follow()`

- **Décision** :
  - Activer `PRAGMA journal_mode=WAL` dans `mobile/store.rs`, réutiliser un unique `Store` par rafale dans `mobile/notifications.rs`, remplacer le thread OS + polling 1 s de `mobile/http.rs::follow()` par un canal `tokio::sync::watch` notifié à la révocation/rotation, et partager `LocalStore` dans Android `MobileUpdates.kt`.
  - Regrouper en requêtes SQL uniques (`IN (...)` / `JOIN`) les boucles N+1 de `native_delegation::tick`.
  - Appliquer l'ADR `260-stockage-versions-schema.md` : helper commun d'ouverture privée `WAL` (`private_wal_connection`) et test garantissant que chaque magasin expose sa version.
  - Migrer `mobile/http.rs::execute()` (509 lignes) vers un routeur déclaratif `axum::Router` déjà présent dans `Cargo.toml` (`axum = "=0.8.9"`).
- **Justification** : Réutilise `axum` et `tokio` déjà verrouillés ; supprime un bloc manuel répétitif de 509 lignes et les ouvertures SQLite redondantes.
- **Alternatives considérées** : Conserver le routeur manuel (refusé : coût de maintenance élevé et duplication avec les nouvelles routes mobiles US5).
- **Impact mainteneur futur** : Chaque route mobile devient lisible, typée et testable isolément.

## 6. Audit des 174 blocs `unsafe` et CI bloquante

- **Décision** : Encapsuler les appels POSIX récurrents (`fcntl`, `tcgetattr`/`tcsetattr`, `poll`, `flock`, `pre_exec` CLOEXEC) dans des helpers documentés dans `bridget-transport` / `bridget-daemon`, auditer chaque bloc `unsafe` restant (production et tests) pour remplacer les 174 phrases génériques par l'invariant réel vérifié, et interdire la phrase de repli dans un contrôle CI. Corriger `test_build_cleanup.py`, `test_updates.py` (canonisation `realpath` `/private/var`), et les échecs Rust préexistants, puis retirer `continue-on-error: true` et ajouter les jobs CI Android et Go Caddy.
- **Justification** : La phrase de repli masquait l'absence réelle de revue de sûreté ; les jobs en `continue-on-error` masquaient les régressions.
- **Alternatives considérées** : Supprimer `unsafe` via des crates tierces (`nix`/`rustix`) (refusé pour limiter la surface de dépendances, sauf si déjà présent ; des wrappers locaux minimalistes suffisent).
- **Impact mainteneur futur** : Chaque `unsafe` porte un contrat vérifiable et la CI garantit la non-régression sur les 4 écosystèmes (Rust, Python, Go, Android).

## 7. Découpage des monolithes, Android et fonctionnalités d'exploitation

- **Décision** :
  - Éclater les 5 monolithes Rust (`daemon.rs`, `native_delegation.rs`, `remote_worker.rs`, `protocol.rs`, `wrapper.rs`) en sous-modules internes sans casser les exports publics, en abaissant `launch_session_with_status` sous 1 000 lignes.
  - Allouer sur le tas (`Box`) les variantes volumineuses du protocole, typer les erreurs métier comparées par chaîne, et réactiver les lints `clippy` correspondants dans `Cargo.toml`.
  - Découper Android `MobileViewModel.kt` (contrôleurs + `data class`) et `MainActivity.kt` (écrans), et ajouter relance opt-in agent/spécialiste, recherche plein texte et filtres notifications.
  - Ajouter les sous-commandes CLI natives `bridget federation links --json`, `bridget status --metrics`, `bridget doctor`, `bridget backup` / `bridget restore --verify`, `bridget completions <shell>`, l'option `--dry-run` sur `delegate`/`auto-delegate`, `BRIDGET_LOG_FORMAT=json`, le signal `background_liveness` + `reaper` Phase 1 opt-in, et une source unique testée pour les schémas MCP.
- **Justification** : Aucune dépendance externe nouvelle nécessaire (`rusqlite` bundled supporte `VACUUM INTO` ; `main.rs` gère le parsing CLI nativement ; Android utilise déjà Kotlin + Compose).
- **Alternatives considérées** : Ajouter `clap` + `clap_complete` ou `schemars` ou `thiserror` (refusé : augmente le temps de compilation et le graphe de dépendances alors que Bridget repose sur un parsing natif propre et zéro dépendance superflue).
- **Impact mainteneur futur** : Réduit la taille moyenne des fichiers critiques et unifie le diagnostic, la sauvegarde et le schéma MCP.
