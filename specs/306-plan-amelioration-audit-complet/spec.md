# Spécification : Programme complet d'amélioration post-audit V15 (306)

**Répertoire** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/306-plan-amelioration-audit-complet`
**Branche** : `main` (règle branche unique du 10 novembre 2026)
**Date** : 2026-11-20
**Statut** : Draft
**Source** : Audit V15 du 2026-11-20 (`/Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-11-20/session-2026-11-20-projet-complet-01/notes/propositions-ameliorations.md`)

## Demande initiale et cadrage Phase 0

- **Demande utilisateur brute** : `/my-specify-all on fait tout` (suite à la restitution de l'audit complet V15 proposant 24 chantiers en Sécurité, Performance, Qualité/CI, Maintenabilité et Fonctionnalités).
- **Demande reformulée** : Exécuter de bout en bout, en 6 vagues ordonnées et vérifiables sur la branche unique `main`, l'ensemble des 24 améliorations identifiées par l'audit V15 du 20 novembre 2026 afin de lever le blocage de sécurité (`release_blocked`), supprimer les goulets de latence et de complexité, rendre les contrôles automatiques fiables et bloquants, réduire la charge cognitive des monolithes Rust et Android, et livrer les capacités d'exploitation demandées (`doctor`, `backup`/`restore`, `reaper` Phase 1 opt-in, `--dry-run`, évolutions mobiles, complétion shell).
- **Exclusions et garanties** :
  - Aucune interruption ni redémarrage de T3 sans confirmation explicite préalable de François.
  - Aucune clé privée de production inventée ou commise dans Git : le mécanisme de signature détachée est entièrement implémenté et testé avec des clés jetables de test, prêt à recevoir une clé de production configurée par l'opérateur.
  - Aucune trace IA dans les messages de commit, tags ou code livré.
  - Aucune suppression de données utilisateur ni de travail étranger.

## Dépendances avec les spécifications existantes

- `specs/260-assainissement-code` : reprend et solde les portes encore ouvertes (D4 masquage IP publique, D6 plafond de connexions, D7 regroupement des requêtes périodiques, D8 mécanisme de signature détachée, D11 retrait d'`exec()`, D12 arbitrage du code réservé T806b, D13 typage d'erreurs, D15 découpage mobile, D16 tests bloquants, et SC09 taille de fonction maximale).
- `docs/decisions/260-stockage-versions-schema.md` : applique la convergence du versionnement SQLite (option C puis ouverture privée unifiée).

## Scénarios utilisateur et priorités

### US1 (P1) - Sécurité des dépendances, des communications et hygiène immédiate
En tant qu'opérateur Bridget, je veux éliminer toutes les vulnérabilités connues des dépendances, supprimer l'IP publique suivie dans Git, plafonner les connexions locales au daemon, renforcer la résistance aux injections inter-agents et disposer d'un mécanisme vérifié de signature détachée des livraisons.

**Scénarios d'acceptation** :
1. **Étant donné** le reverse proxy de sécurité, **quand** l'analyse des vulnérabilités est lancée, **alors** aucune vulnérabilité confirmée (CRITICAL, HIGH ou MEDIUM) ne subsiste dans les manifestes suivis.
2. **Étant donné** le dépôt documentaire, **quand** le contrôle de documentation s'exécute, **alors** aucune adresse IPv4 publique hors plages de documentation RFC 5737 n'est acceptée.
3. **Étant donné** un client local saturant la socket Unix du daemon, **quand** le plafond de connexions simultanées est atteint, **alors** les connexions excédentaires reçoivent un refus structuré sans saturer les fils système, tandis que les canaux réservés aux workers et à l'administration restent disponibles.
4. **Étant donné** un message inter-agent contenant une tentative de falsification de délimiteur, **quand** son enveloppe est construite, **alors** le délimiteur unique par message empêche toute confusion entre métadonnées et contenu.
5. **Étant donné** la sentinelle de fédération, **quand** elle relève les liens actifs, **alors** elle interroge une sous-commande native structurée sans exécuter de code Python injecté via variable d'environnement.
6. **Étant donné** un brief ou message contenant un motif de secret manifeste lorsque le filtrage préventif est activé, **quand** il est soumis à persistance, **alors** le secret est rédigé ou refusé selon la politique configurée.
7. **Étant donné** une archive de livraison signée avec une clé de test, **quand** l'installateur vérifie sa signature détachée, **alors** une signature valide passe et une signature absente, corrompue ou issue d'une clé inconnue est rejetée.

### US2 (P1) - Performance, complexité bornée et contention maîtrisée
En tant qu'utilisateur CLI, MCP et mobile, je veux des réponses locales immédiates sans latence artificielle de 50 ms, zéro relecture disque inutile, des gardes mémoire strictement bornés et des accès SQLite concurrents et groupés.

**Scénarios d'acceptation** :
1. **Étant donné** un daemon au repos, **quand** une commande locale arrive sur la socket Unix, **alors** la boucle d'acceptation se réveille immédiatement sur événement système sans attente périodique de 50 ms et consomme 0 réveil par seconde au repos.
2. **Étant donné** un fichier `agents.json` inchangé, **quand** plusieurs requêtes successives consultent le registre, **alors** le contenu est servi depuis le cache mémoire tant que la date de modification, la taille et l'inode ne changent pas.
3. **Étant donné** des milliers d'agents éphémères ou de messages dans les gardes du noyau, **quand** la fenêtre glissante expire ou la capacité maximale est atteinte, **alors** l'éviction s'exécute en temps constant amorti, la mémoire reste bornée, les clés vides sont purgées et deux cibles différentes pour un même contenu ne s'écrasent pas.
4. **Étant donné** une boucle périodique de délégation avec plusieurs missions et propriétaires connectés, **quand** un cycle s'exécute, **alors** les données nécessaires sont chargées en requêtes SQL groupées plutôt qu'en boucle requête par élément.
5. **Étant donné** la passerelle mobile en charge, **quand** plusieurs notifications ou flux d'événements sont actifs, **alors** la base mobile fonctionne en mode journal concurrent (`WAL`), chaque rafale réutilise une seule connexion, le suivi d'événements attend une invalidation mémoire plutôt qu'un sondage toutes les secondes, et l'application Android réutilise son magasin local partagé.
6. **Étant donné** des requêtes concurrentes de lecture d'annuaire/présences et d'écriture d'état, **quand** elles s'exécutent dans le daemon, **alors** les lectures majoritaires utilisent un verrou lecture/écriture dédié au lieu d'un verrou exclusif global unique.

### US3 (P1) - Qualité de code, sûreté des blocs non vérifiés et CI bloquante
En tant que mainteneur, je veux que chaque bloc non vérifié par le compilateur soit réellement audité et justifié, que tous les tests unitaires et scripts passent sans échec préexistant, et que la chaîne d'intégration continue bloque toute régression sur Rust, Python, Go et Android.

**Scénarios d'acceptation** :
1. **Étant donné** l'ensemble du code Rust, **quand** le contrôle de formatage et de style s'exécute, **alors** 0 fichier dérive du formatage et 0 bloc non vérifié ne conserve la phrase de justification de repli.
2. **Étant donné** les suites de tests Python (`test_build_cleanup.py`, `test_updates.py` et autres suites `scripts/tests/`) et les tests Rust ciblés sur la base d'échecs connue, **quand** ils sont exécutés localement, **alors** tous les échecs identifiés sont corrigés et vérifiés.
3. **Étant donné** la configuration d'intégration continue, **quand** une modification est soumise, **alors** les contrôles Rust, Python, audit de dépendances, Android et reverse proxy Go s'exécutent sans drapeau qui ignore les échecs (`continue-on-error`).

### US4 (P2) - Maintenabilité Rust, typage d'erreurs et minimalisme
En tant que mainteneur futur, je veux un code Rust organisé par sous-domaines lisibles sans monolithes géants ni fonctions dépassant 1 000 lignes, un routage HTTP déclaratif, une stratégie de versionnement SQLite unifiée, des erreurs typées sur les frontières critiques et moins d'exemptions de linter.

**Scénarios d'acceptation** :
1. **Étant donné** les 5 fichiers monolithiques identifiés (`daemon.rs`, `native_delegation.rs`, `remote_worker.rs`, `protocol.rs`, `wrapper.rs`), **quand** le découpage est appliqué, **alors** chaque sous-domaine vit dans un sous-module cohérent et `launch_session_with_status` passe sous 1 000 lignes à comportement identique.
2. **Étant donné** la passerelle HTTP mobile, **quand** une requête est routée, **alors** le routage s'appuie sur les routes déclaratives du framework déjà présent plutôt que sur un aiguillage manuel de 509 lignes.
3. **Étant donné** les bases SQLite du projet, **quand** une base est ouverte, **alors** elle passe par un socle commun d'ouverture privée et un test garantit que chaque magasin expose une version lisible.
4. **Étant donné** les erreurs métier internes et les énumérations du protocole, **quand** les grosses variantes sont allouées sur le tas et les erreurs comparées par texte typées, **alors** les règles de linter correspondantes sont réactivées et le code mort ou mal nommé est assaini.

### US5 (P2) - Modularisation et nouvelles capacités Android
En tant qu'utilisateur de l'application mobile Bridget, je veux une application modulaire et typée offrant la relance encadrée (opt-in) d'un agent arrêté, le lancement d'un profil spécialiste approuvé, la recherche dans l'historique et le filtrage des notifications par projet et urgence.

**Scénarios d'acceptation** :
1. **Étant donné** le code source Android, **quand** un mainteneur inspecte la logique d'état et les écrans, **alors** le modèle de vue monolithique est découpé par domaines avec des structures de données typées et les écrans sont séparés par fichier.
2. **Étant donné** un utilisateur mobile ayant activé la portée opt-in correspondante, **quand** il demande la relance d'un agent arrêté ou le lancement d'un profil spécialiste approuvé, **alors** l'action est validée par le serveur et refusée si la portée n'est pas autorisée.
3. **Étant donné** un historique mobile de conversations, **quand** l'utilisateur saisit une recherche ou filtre ses notifications par projet et urgence, **alors** les résultats et alertes affichés respectent strictement le filtre sélectionné.

### US6 (P2) - Nouvelles fonctionnalités d'exploitation et d'ergonomie
En tant qu'opérateur et utilisateur d'agents, je veux diagnostiquer l'état complet du système en une commande, sauvegarder et vérifier la restauration à chaud des données, simuler une délégation avant exécution, nettoyer de façon opt-in les sessions abandonnées et disposer de la complétion shell ainsi que de schémas MCP alignés.

**Scénarios d'acceptation** :
1. **Étant donné** un daemon actif, **quand** j'exécute `bridget status --metrics` ou `bridget doctor`, **alors** j'obtiens les compteurs d'exploitation, les latences de cycle (p50/p95/p99), l'état des files, des workers distants et de la passerelle mobile ; avec `BRIDGET_LOG_FORMAT=json`, les journaux sont émis en JSON structuré corrélé.
2. **Étant donné** un espace `BRIDGET_HOME` actif, **quand** j'exécute `bridget backup <archive>` puis `bridget restore --verify <archive>`, **alors** une copie cohérente à chaud de toutes les bases et configurations est créée et sa restauration est vérifiée en bac à sable sans altérer les données actives.
3. **Étant donné** des sessions gérées dont l'activité de fond est observable, **quand** le mode nettoyage opt-in du `reaper` est activé, **alors** seules les sessions réellement inactives au-delà du seuil sans tâche active sont nettoyées.
4. **Étant donné** une demande de délégation, **quand** j'ajoute `--dry-run`, **alors** la sélection de machine, modèle, quotas et permissions est calculée et affichée sans réserver ni lancer de session.
5. **Étant donné** un shell utilisateur (`bash`, `zsh`, `fish`), **quand** j'exécute `bridget completions <shell>`, **alors** un script de complétion valide est produit ; et les schémas d'outils MCP sont dérivés d'une source unique testée.

## Exigences fonctionnelles

- **FR-001** : Mettre à jour et alléger `deploy/caddy-security/go.mod` pour éliminer toutes les vulnérabilités OSV confirmées.
- **FR-002** : Mettre à jour `pulldown-cmark` dans `Cargo.lock` et contraindre `com.google.guava:guava` dans Android afin d'assainir tous les lockfiles.
- **FR-003** : Masquer l'adresse IPv4 publique dans `specs/233-placement-opus2d-mac/validation.md` et bloquer toute IPv4 publique hors RFC 5737 dans `scripts/check-docs.py`.
- **FR-004** : Plafonner les connexions simultanées sur la socket Unix du daemon avec une réserve dédiée aux workers et à l'administration.
- **FR-005** : Encadrer les corps de messages dans `bridget-core` par un délimiteur unique par message, remplacer `exec()` dans `scripts/sentinel.py` par une sous-commande CLI native (`bridget federation links --json`), et ajouter un filtre préventif configurable de secrets avant persistance.
- **FR-006** : Implémenter la vérification et la signature détachée des archives de livraison via `ssh-keygen -Y` avec clé publique/privée configurable et couverture de tests complète.
- **FR-007** : Remplacer le `sleep(50 ms)` de la boucle d'acceptation Unix du daemon par `poll(2)` couplé à un tube d'éveil d'arrêt.
- **FR-008** : Mettre en cache `AgentRegistry` par `(mtime, size, inode)` pour éviter les relectures disque répétées.
- **FR-009** : Corriger `EnvelopeGuard`, `Deduplicator` (clé `(content_key, target)`) et `CircuitBreaker` en éviction O(1) amorti, capacité bornée, purge des clés vides et `checked_sub`.
- **FR-010** : Regrouper les requêtes SQL N+1 de `native_delegation::tick` hors contention inutile.
- **FR-011** : Activer `WAL` sur `mobile-devices.db`, réutiliser la connexion SQLite par rafale dans `mobile/notifications.rs`, remplacer le sondage 1 s de `mobile/http.rs::follow()` par une invalidation en mémoire, et partager `LocalStore` dans Android `MobileUpdates.kt`.
- **FR-012** : Scinder l'état global du daemon pour placer les domaines majoritairement lus (annuaire, présences, catalogue) sous verrou lecture/écriture (`RwLock`).
- **FR-013** : Auditer les 174 blocs `unsafe`, encapsuler les appels POSIX récurrents dans `bridget-transport`, supprimer toute phrase `SAFETY` de repli et bloquer son retour en CI.
- **FR-014** : Formater les 18 fichiers Rust en dérive, corriger les 3 échecs de `test_build_cleanup.py`, les 16 échecs macOS de `test_updates.py` et les échecs Rust préexistants de la base connue, puis rendre bloquants les jobs CI et ajouter les jobs Android et Go Caddy.
- **FR-015** : Découper les 5 monolithes Rust (`daemon.rs`, `native_delegation.rs`, `remote_worker.rs`, `protocol.rs`, `wrapper.rs` dont `launch_session_with_status` < 1 000 lignes) et migrer `mobile/http.rs::execute` vers des routes Axum déclaratives.
- **FR-016** : Appliquer l'ADR `260-stockage-versions-schema.md` (test de version lisible par magasin + socle commun d'ouverture SQLite privée en `WAL`).
- **FR-017** : Typer les erreurs critiques comparées par texte, réduire les exemptions globales de `Cargo.toml` (notamment `Box` sur les grosses variantes du protocole), nettoyer ou documenter le code T806b et renommer les symboles internes suffixés par numéro de spec.
- **FR-018** : Découper `MobileViewModel.kt` et `MainActivity.kt` en modules par domaine/écran avec classes de données typées.
- **FR-019** : Ajouter côté serveur mobile et Android la relance opt-in d'agent arrêté, le lancement de profil spécialiste approuvé, la recherche plein texte mobile et le filtrage des notifications par projet/urgence.
- **FR-020** : Exposer les métriques dans `bridget status --metrics`, créer `bridget doctor` et ajouter le format de log JSON (`BRIDGET_LOG_FORMAT=json`).
- **FR-021** : Créer `bridget backup` et `bridget restore --verify` avec sauvegarde cohérente à chaud SQLite et vérification isolée.
- **FR-022** : Exposer le signal `background_liveness` et activer le mode opt-in Phase 1 du `reaper`.
- **FR-023** : Ajouter `--dry-run` sur `bridget delegate` et `bridget auto-delegate`.
- **FR-024** : Ajouter `bridget completions bash|zsh|fish` et unifier la génération/vérification des schémas MCP.

## Cas limites identifiés

- Horloge monotone inférieure à la fenêtre au démarrage de machine (`Instant::now().checked_sub(window)` retourne `None`).
- Connexions Unix atteignant exactement le plafond : un worker distant ou l'administrateur doit toujours pouvoir se connecter via la réserve.
- Sauvegarde à chaud pendant une écriture active SQLite : le fichier sauvegardé doit passer `PRAGMA integrity_check` sans verrouiller durablement le daemon.
- Clé de signature détachée absente ou révoquée : refus clair avant extraction de l'archive.
- Liens symboliques macOS (`/var` -> `/private/var`) dans les tests temporaires Python et Rust.
- Clients mobiles anciens ne connaissant pas les nouvelles portées opt-in : rétrocompatibilité complète des routes existantes.

## Entités clés

- **Enveloppe sécurisée** : métadonnées d'échange + délimiteur unique par identifiant de message + corps sanitisé.
- **Cache de registre** : instantané mémoire d'`AgentRegistry` indexé par `(chemin, mtime, size, inode)`.
- **Archive de sauvegarde** : bundle horodaté contenant copies SQLite cohérentes à chaud, configurations et manifeste d'empreintes SHA-256.
- **Diagnostic système (`DoctorReport`)** : structure JSON/texte regroupant santé daemon, métriques, latences de tick, sockets, bases SQLite, workers et passerelle mobile.
- **Plan de simulation (`DryRunPlan`)** : décision complète de routage flotte, scores, quotas et contrôles d'autorisation sans effet de bord.

## Critères de succès mesurables

- **SC-001** : `osv-scanner` remonte 0 vulnérabilité confirmée sur l'ensemble du dépôt (Go, Rust, Android).
- **SC-002** : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked` et `scripts/check-docs.py` passent tous à 0 erreur.
- **SC-003** : 0 occurrence de la phrase `SAFETY` de repli dans `crates/`.
- **SC-004** : 100 % des tests `bridget-core`, `scripts/tests/test_build_cleanup.py`, `scripts/tests/test_updates.py` et des suites Python existantes passent sans échec.
- **SC-005** : La boucle d'acceptation Unix du daemon ne contient plus de `thread::sleep` sur `WouldBlock` et répond sur événement `poll(2)`.
- **SC-006** : `launch_session_with_status` dans `wrapper.rs` compte moins de 1 000 lignes et les 5 monolithes sont éclatés en sous-modules thématiques.
- **SC-007** : Les 6 nouvelles capacités (`doctor`/`status --metrics`, `backup`/`restore --verify`, `reaper` Phase 1 opt-in, `delegate --dry-run`, `completions`, évolutions Android) disposent chacune d'au moins un test automatisé vérifié.
- **SC-008** : Les commits sont propres sur `main` sans aucune trace IA, poussés et vérifiés sur `github/main`, suivis du ménage complet.

## Hypothèses retenues

- Travail direct sur la branche unique `main` conformément à la décision utilisateur du 10 novembre 2026 (`AGENTS.md`).
- Signature détachée implémentée avec `ssh-keygen -Y` (présent nativement sur macOS et Linux, contrairement à `minisign`/`cosign` absents de ce poste).
- Aucune nouvelle dépendance externe ajoutée en Rust ou Android sans preuve qu'aucune dépendance existante ne suffit (Article XIX).
