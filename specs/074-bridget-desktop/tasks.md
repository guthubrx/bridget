# Tâches - SPEC-074 Bridget Desktop

**Entrées** : `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/desktop-commands.md`, `quickstart.md`, `reuse-audit.md`.

**Principe d'exécution** : une tâche n'est cochée qu'après la preuve indiquée. Les tests précèdent le comportement qu'ils verrouillent. Aucune clé privée, jeton UI ou URL signée ne doit apparaître dans les fixtures, les diagnostics, le stockage de profils ou Git.

## Dépendances

`Fondations` -> `US1 distant` -> `US2 états` -> `US3 deux panneaux` -> `US4 tunnels possédés` -> `Qualité et paquet macOS`.

US5 ne crée pas de navigateur: elle prouve que la frontière future reste documentée et non simulée par une abstraction prématurée.

## Phase 1 - Contrat serveur et initialisation

- [x] T001 Ajouter dans `crates/bridget-daemon/src/cli.rs` les tests unitaires de syntaxe pour `bridget ui endpoint --json`, incluant le rejet de toute option inconnue et l'absence du jeton dans les erreurs.
- [x] T002 Étendre `crates/bridget-daemon/src/cli.rs` avec la sous-action fermée `ui endpoint --json`, qui appelle `crates/bridget-daemon/src/ui.rs` et écrit exclusivement le contrat versionné documenté dans `specs/074-bridget-desktop/contracts/desktop-commands.md`.
- [x] T003 Ajouter dans `crates/bridget-daemon/tests/ui_relay_test.rs` une preuve d'intégration: l'endpoint existant est lu sans lancer de relais, le JSON est exploitable et les cas état absent ou invalide ne divulguent pas de jeton.
- [x] T004 Créer `apps/bridget-desktop/src-tauri/Cargo.toml`, `apps/bridget-desktop/src-tauri/build.rs`, `apps/bridget-desktop/src-tauri/tauri.conf.json`, `apps/bridget-desktop/src-tauri/capabilities/main.json` et `apps/bridget-desktop/ui/` comme application Tauri macOS séparée, avec la capability limitée à la coque locale et le feature multiwebview `unstable` justifié dans le manifeste.
- [x] T005 Vérifier avec `cargo check --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml` que le nouveau client est compilable comme projet isolé et que le workspace Bridget existant n'acquiert aucune dépendance Tauri.

## Phase 2 - Fondations bloquantes

- [x] T006 Écrire dans `apps/bridget-desktop/src-tauri/src/profile.rs` les tests de sérialisation et validation des profils `ssh`: aucun mot de passe, contenu de clé ou jeton n'est admis.
- [x] T007 Implémenter dans `apps/bridget-desktop/src-tauri/src/profile.rs` les modèles versionnés de profil, session et état définis dans `specs/074-bridget-desktop/data-model.md`, avec une validation explicite des ports, hôtes, comptes et références d'identité.
- [x] T008 Écrire dans `apps/bridget-desktop/src-tauri/src/profile_store.rs` les tests de migration depuis un fichier absent ou ancien, de persistance atomique en droits restrictifs et de refus d'une donnée secrète sentinelle.
- [x] T009 Implémenter dans `apps/bridget-desktop/src-tauri/src/profile_store.rs` le stockage local non secret, versionné et atomique des profils, sous le répertoire de données applicatif, sans modifier les fichiers SSH globaux.
- [x] T010 Créer dans `apps/bridget-desktop/src-tauri/src/lib.rs`, `apps/bridget-desktop/ui/index.html`, `apps/bridget-desktop/ui/app.js` et `apps/bridget-desktop/ui/theme.css` la coque locale accessible: liste de profils vide, focus visible, ajout, édition, retrait après confirmation explicite et sélection au clavier, sans panneau distant encore chargé.

## Phase 3 - User Story 1 - Ajouter et ouvrir un serveur distant (P1)

**Objectif** : l'opérateur ajoute `cartae.app`, vérifie son identité SSH, puis ouvre l'UI Bridget distante sans que le relais serveur soit public.

**Critère indépendant** : avec un faux SSH et un faux relais, l'état devient `connected` seulement après réponse HTTP à travers un forward `127.0.0.1`; un serveur inconnu requiert une approbation humaine et un changement de clé bloque.

- [x] T011 [US1] Ajouter dans `apps/bridget-desktop/src-tauri/src/ssh.rs` les tests de construction d'arguments SSH sûrs: pas de shell, `BatchMode=yes`, `ControlMaster=no`, `ExitOnForwardFailure=yes`, keepalives, loopback local explicite et refus des valeurs qui injecteraient une option.
- [x] T012 [US1] Implémenter dans `apps/bridget-desktop/src-tauri/src/ssh.rs` l'exécuteur SSH typé et la supervision d'enfant, avec arguments construits en `Command`, un port local réservé et un arrêt limité au processus possédé par la session.
- [x] T013 [US1] Ajouter dans `apps/bridget-desktop/src-tauri/src/host_identity.rs` les tests avec exécuteur factice: hôte déjà approuvé, hôte inconnu avec ticket éphémère, refus, et changement d'empreinte bloquant.
- [x] T014 [US1] Implémenter dans `apps/bridget-desktop/src-tauri/src/host_identity.rs` la découverte contrôlée par `ssh-keyscan`, le calcul d'empreinte, l'approbation explicite et le `known_hosts` propre à l'application, sans import automatique de clé privée.
- [x] T015 [US1] Ajouter dans `apps/bridget-desktop/src-tauri/src/connection.rs` les tests d'un endpoint `bridget ui endpoint --json` factice: contrat valide, version inconnue, port invalide et erreur redacted sans jeton.
- [x] T016 [US1] Implémenter dans `apps/bridget-desktop/src-tauri/src/connection.rs` la découverte SSH de l'endpoint, l'ouverture du forward local et la vérification HTTP du relais avant toute transition vers `connected`; le jeton reste en mémoire.
- [x] T017 [US1] Exposer dans `apps/bridget-desktop/src-tauri/src/lib.rs` uniquement les commandes locales fermées de `contracts/desktop-commands.md`, avec tests de désérialisation qui refusent une commande libre, un secret ou une URL fournie par le frontend.
- [x] T018 [US1] Compléter `apps/bridget-desktop/ui/index.html`, `apps/bridget-desktop/ui/app.js` et `apps/bridget-desktop/ui/theme.css` avec le parcours d'onboarding distant: formulaire, empreinte à confirmer, progression honnête SSH -> tunnel -> relais, origine visible avant le panneau, erreur lisible et diagnostic redacted.
- [x] T019 [US1] Ajouter dans `apps/bridget-desktop/src-tauri/tests/remote_connection.rs` un test d'intégration avec SSH et relais factices qui prouve le trajet découverte -> tunnel -> contrôle HTTP et l'absence de secret dans les sorties de test.

## Phase 4 - User Story 2 - Lire des états de connexion honnêtes (P1)

**Objectif** : comprendre si l'application attend SSH, l'approbation, le tunnel, le relais, une reconnexion ou un échec, sans confondre cela avec le travail des agents.

**Critère indépendant** : couper le processus de tunnel factice fait passer uniquement le profil concerné à `reconnecting` ou `failed`; il ne devient jamais artificiellement connecté.

- [x] T020 [US2] Ajouter dans `apps/bridget-desktop/src-tauri/src/connection.rs` les tests de machine d'états pour toutes les transitions de `data-model.md`, dont l'interdiction `ssh vivant -> connected` sans réponse relais.
- [x] T021 [US2] Implémenter dans `apps/bridget-desktop/src-tauri/src/connection.rs` les transitions, la reconnexion bornée, les catégories SSH/identité/tunnel/relais/version et l'émission ciblée de `connection-state` vers la seule coque locale.
- [x] T022 [US2] Ajouter dans `apps/bridget-desktop/ui/app.js` et `apps/bridget-desktop/ui/theme.css` le rendu accessible des états de connexion, l'action explicite de reconnexion et les diagnostics sans secret, explicitement séparés des états et messages affichés par l'UI distante.
- [x] T023 [US2] Ajouter dans `apps/bridget-desktop/src-tauri/tests/connection_lifecycle.rs` une preuve d'arrêt de fenêtre: l'enfant SSH possédé disparaît, aucune session fermée n'émet encore un état et une autre session reste intacte.

## Phase 5 - User Story 3 - Gérer plusieurs serveurs et deux panneaux (P2)

**Objectif** : sélectionner un serveur ou visualiser exactement deux origines en parallèle, sans mélanger agents, messages ou notifications.

**Critère indépendant** : deux relais factices affichent des marqueurs d'origine différents dans deux webviews `panel-*`, sans que ces webviews puissent invoquer les commandes Tauri de la coque.

- [x] T024 [US3] Ajouter dans `apps/bridget-desktop/src-tauri/src/panels.rs` les tests de labels uniques, limite stricte à deux panneaux, mapping `panel -> profile_id` et rejet d'une URL qui n'est pas la boucle locale de la session.
- [x] T025 [US3] Implémenter dans `apps/bridget-desktop/src-tauri/src/panels.rs` les webviews enfants Tauri externes, leur redimensionnement, leur création et destruction, la navigation restreinte au relais `127.0.0.1` de la session et l'absence de capability pour `panel-*`.
- [x] T026 [US3] Mettre à jour `apps/bridget-desktop/src-tauri/capabilities/main.json` avec un test de configuration dans `apps/bridget-desktop/src-tauri/tests/capabilities.rs`: seuls les labels de coque ont accès aux commandes Bridget Desktop, aucun domaine distant n'obtient de permission.
- [x] T027 [US3] Ajouter dans `apps/bridget-desktop/ui/index.html`, `apps/bridget-desktop/ui/app.js` et `apps/bridget-desktop/ui/theme.css` les onglets de profils et le mode deux panneaux, avec un nom d'origine toujours visible et une séparation de notifications par profil.
- [x] T028 [US3] Ajouter dans `apps/bridget-desktop/src-tauri/tests/two_panels.rs` un test d'intégration de deux sessions factices: fermeture ou reconnexion du premier panneau sans impact sur le second.

## Phase 6 - User Story 4 - Posséder le tunnel de session (P2)

**Objectif** : retirer le parcours d'endpoint manuel, écarter les anciens profils directs et faire posséder chaque tunnel par Bridget Desktop pendant sa session.

**Critère indépendant** : un profil approuvé redémarre dans un tunnel SSH créé par Bridget Desktop, le jeton est récupéré de façon transparente et aucun formulaire ne demande un port de relais ou un jeton.

- [x] T029 [US4] Conserver l'historique des premiers tests de relais direct, remplacés par le mode SSH géré.
- [x] T030 [US4] Conserver l'historique du chemin de relais direct, retiré du code produit par T041.
- [x] T031 [US4] Conserver l'historique de son formulaire, retiré du parcours opérateur par T041.

## Phase 7 - User Story 5 - Préserver la future capacité navigateur (P3 documentaire)

**Objectif** : ne pas enfermer l'application dans une architecture incompatible avec un futur navigateur lancé sur le serveur puis visualisé localement.

**Critère indépendant** : aucun code SPEC-074 ne crée de navigateur, VNC ou tunnel générique; les données et la documentation associent explicitement une future session navigateur à un profil et une exécution isolés.

- [x] T032 [US5] Vérifier et compléter si nécessaire `specs/074-bridget-desktop/data-model.md`, `specs/074-bridget-desktop/plan.md`, `specs/074-bridget-desktop/research.md` et `docs/decisions/018-bridget-desktop-tunnels-client.md` pour documenter la frontière navigateur sans ajouter de dépendance ou de code hors scope.

## Phase 8 - Qualité, sécurité et paquet macOS

- [x] T033 Ajouter dans `apps/bridget-desktop/src-tauri/tests/secrets_and_diagnostics.rs` des sentinelles qui prouvent que profils, diagnostics, erreurs et traces de test ne contiennent ni clé privée, ni jeton UI, ni URL signée.
- [x] T034 Exécuter les suites Rust pertinentes: `cargo test -p bridget-daemon` et `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml`, puis corriger tout échec attribuable à SPEC-074 avant de cocher.
- [x] T035 Vérifier les surfaces créées avec `rg` dans `apps/bridget-desktop` et mettre à jour `specs/074-bridget-desktop/reuse-audit.md` pour tout nouveau module, dépendance ou arbitrage créé pendant l'implémentation.
- [ ] T036 Exécuter les scénarios opérateur de `specs/074-bridget-desktop/quickstart.md` avec cartae.app et un faux serveur, mesurer un ajout distant sans secret en moins de trois minutes, vérifier tout le parcours clavier, puis consigner commandes, résultats et éléments macOS vérifiés dans `specs/074-bridget-desktop/evidence/validation.md`.
- [x] T037 Construire sur macOS avec `cargo build --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --release` puis `cargo tauri build --config apps/bridget-desktop/src-tauri/tauri.conf.json`, consigner le chemin du paquet ou l'obstacle environnemental vérifiable dans `specs/074-bridget-desktop/evidence/macos-package.md`.
- [x] T038 Mettre à jour `specs/074-bridget-desktop/spec.md`, `specs/074-bridget-desktop/quickstart.md` et `specs/074-bridget-desktop/implementation.md` avec les preuves réelles, les limites résiduelles et le statut exact de la SPEC.
- [x] T039 Correctif d'acceptation : activer explicitement l'API globale Tauri requise par la coque statique, renommer le profil local en relais Bridget direct avec hôte loopback et port, demander son jeton uniquement pour la connexion en mémoire, et réaligner l'habillage sur `crates/bridget-daemon/assets/ui/theme.css`.
- [x] T040 Correctif d'acceptation : présenter l'accès comme un endpoint déjà accessible ou un tunnel SSH géré, accepter tout port loopback valide, ne jamais tenter de classifier le tunnel préexistant et expliquer le trajet dans le formulaire et les états.
- [x] T041 Correctif d'industrialisation : ne conserver que les profils SSH gérés, migrer en écartant les anciens endpoints manuels, retirer toute demande de jeton, restaurer automatiquement les profils approuvés au lancement et mettre à jour la documentation opérateur.

## Ordre d'implémentation

1. T001-T010 établissent le contrat, le client séparé et les profils.
2. T011-T019 livrent l'incrément distant utilisable, sans panneau double ni local.
3. T020-T023 rendent ce premier incrément observable et fiable.
4. T024-T028 ajoutent la double vue sécurisée.
5. T029-T031 ajoutent le local sans élargir SSH.
6. T032 confirme la frontière navigateur, puis T033-T038 imposent les preuves de sécurité et le paquet macOS.

## Opportunités de parallelisme

- Après T005, T006-T010 sont séparables par fichier, sous réserve de garder T007 avant les commandes qui consomment le modèle.
- Après T016, les tests T020 et le squelette T024 peuvent être préparés en parallèle, mais l'intégration des panneaux attend T021 et T025.
- T033 peut être préparée dès que les formats de profil et diagnostic existent; elle n'est cochée qu'après les implémentations correspondantes.

## Vérification Article XX

- La réutilisation du relais évite une seconde UI, API et synchronisation métier.
- La commande endpoint, les profils, SSH et les panneaux portent chacun une règle de sécurité ou de cycle de vie observable; ce ne sont pas des wrappers génériques.
- Le seul coût exceptionnel est le feature Tauri `unstable`, justifié par l'exigence exacte de deux panneaux isolés; T032 et T037 en conservent la trace et la vérification.
