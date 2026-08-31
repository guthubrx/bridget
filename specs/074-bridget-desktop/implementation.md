# Journal d'implémentation - SPEC-074 Bridget Desktop

## Métadonnées

- Spec : 074-bridget-desktop
- Branche : `session-074-bridget-desktop`
- Démarré : 2026-08-30
- Terminé : En cours

## Progression

### T001 - Grammaire endpoint UI

- Statut : Complété
- Fichiers modifiés : `crates/bridget-daemon/src/cli.rs`
- Vérification : `/home/moi/.cargo/bin/cargo test -p bridget-daemon cli::hook_tests::endpoint_ui_exige_un_contrat_json_ferme --lib -- --exact` - OK, 1 test.
- Self-review Article XIX/XX : nécessaire pour empêcher un client Desktop d'utiliser une grammaire UI ouverte; une seule fonction de parsing réutilise la commande `ui` existante plutôt qu'une commande racine. Hypothèse: le contrat Desktop impose JSON. Code évité: aucun nouveau binaire ou parser CLI externe. Non vérifié: appel réel via SSH, couvert par T019.

### T002 - Lecture CLI de l'endpoint existant

- Statut : Complété
- Fichiers modifiés : `crates/bridget-daemon/src/cli.rs`
- Vérification : test T001 et inspection du diff - la voie `endpoint` appelle seulement `load_ui_endpoint`; elle ne démarre ni ne redémarre `UiRelay`.
- Self-review Article XIX/XX : nécessaire pour exposer le contrat persistant sans faire lire son fichier interne au client. La sous-action `ui endpoint` est plus maintenable qu'un second binaire car elle reste près de `UiEndpoint`. Hypothèse: la session SSH authentifiée est la frontière d'accès au binaire. Non vérifié: compatibilité Desktop complète, couverte par T019. Complexité: O(1), lecture d'un seul petit fichier.

### T003 - Intégration du contrat endpoint

- Statut : Complété
- Fichiers modifiés : `crates/bridget-daemon/tests/ui_relay_test.rs`
- Vérification : `/home/moi/.cargo/bin/cargo test -p bridget-daemon --test ui_relay_test spec_074_cli_endpoint_lit_l_etat_sans_demarrer_de_relais_ni_divulguer_en_erreur -- --exact` - OK, 1 test.
- Self-review Article XIX/XX : nécessaire pour prouver le contrat réel et l'absence de fuite sur état absent ou invalide. Le test réemploie les helpers d'intégration UI; aucune couche de test nouvelle. Hypothèse: le jeton de fixture est non secret et ne reflète aucune donnée de production. Non vérifié: tunnel SSH réel, réservé au test d'intégration client T019.

### T004 - Squelette client isolé

- Statut : Complété
- Fichiers créés : `apps/bridget-desktop/.gitignore`, `apps/bridget-desktop/src-tauri/{Cargo.toml,build.rs,tauri.conf.json,capabilities/main.json,src/lib.rs,src/main.rs}`, `apps/bridget-desktop/ui/{index.html,app.js,theme.css}`.
- Vérification : `/home/moi/.cargo/bin/cargo fmt --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --check` - OK. La compilation Linux s'arrête avant le code Bridget Desktop parce que `pkg-config` et `dbus-1` sont absents sur cartae.app; cette preuve environnementale est conservée pour T005 et la validation cible macOS T037.
- Self-review Article XIX/XX : nécessaire car aucun client desktop n'existe. Le client est un workspace Cargo imbriqué, explicitement séparé du workspace serveur, ce qui évite d'imposer Tauri aux tests Bridget. La capability ne vise que `main`; aucun label `panel-*` n'est permis. Hypothèse: l'API multiwebview Tauri est disponible avec le feature `unstable`. Non vérifié: compilation et bundle macOS, réservés à T037. Code évité: aucun framework frontend, aucun plugin shell, aucun serveur local supplémentaire.

### T005 - Compilation isolée sur le serveur

- Statut : Complété
- Fichiers modifiés : `apps/bridget-desktop/src-tauri/{Cargo.toml,build.rs,src/lib.rs}`.
- Vérification : `/home/moi/.cargo/bin/cargo check --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml` - OK sur cartae.app.
- Décision de portabilité : la dépendance `tauri` est ciblée macOS. Le coeur Rust des profils reste donc testable sur le serveur Linux, qui n'a volontairement ni `pkg-config` ni les en-têtes DBus requis par le moteur WebKit. `tauri-build` n'est invoqué que sur macOS. La compilation du client graphique et le paquet sont toujours exigés par T037 sur le Mac cible.
- Self-review Article XIX/XX : cette séparation est nécessaire pour prouver le coeur sans installer de dépendance GUI sur le serveur de production. Elle n'ajoute pas de mode d'exécution Linux ni de second client : `run()` y échoue explicitement. Complexité inchangée et workspace Bridget non modifié.

### T006-T007 - Modèle de profils non secrets

- Statut : Complété
- Fichier créé : `apps/bridget-desktop/src-tauri/src/profile.rs`.
- Vérification : `/home/moi/.cargo/bin/cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --lib` - OK, 6 tests dont validation SSH/local, rejet d'hôte ambigu et sentinelle de clé privée.
- Self-review Article XIX/XX : les deux variantes exactes `ssh` et `local` empêchent structurellement l'ajout de paramètres SSH à un profil local. Les seules références d'identité admises sont l'agent SSH local ou un chemin absolu : jamais une clé importée. Code évité : aucun coffre de secrets, parser URI ou abstraction de transport générique.

### T008-T009 - Stockage local atomique

- Statut : Complété
- Fichier créé : `apps/bridget-desktop/src-tauri/src/profile_store.rs`.
- Vérification : même suite de 6 tests - OK, couvrant fichier absent, migration v0, refus de version inconnue, permissions `0600` Unix, écriture atomique et suppression exacte.
- Self-review Article XIX/XX : un fichier JSON versionné dans le répertoire applicatif est le minimum nécessaire aux profils. Écriture temporaire, synchronisation et renommage évitent un fichier partiellement écrit. Aucun fichier SSH global n'est lu ou modifié, et aucune matière secrète n'est sérialisée.

### T010-T019 - Coque et premier serveur distant

- Statut : Complété.
- Fichiers : `apps/bridget-desktop/ui/{index.html,app.js,theme.css}`, `apps/bridget-desktop/src-tauri/src/{lib.rs,ssh.rs,host_identity.rs,connection.rs}` et `apps/bridget-desktop/src-tauri/tests/{desktop_commands.rs,remote_connection.rs}`.
- Fonctionnement : la coque crée, édite et retire explicitement des profils. Elle demande l'empreinte SSH, utilise uniquement l'agent SSH ou un chemin de clé existante, exécute la commande distante constante `bridget ui endpoint --json`, ouvre un forward loopback possédé et vérifie HTTP avant `connected`.
- Sécurité : aucune commande, URL de relais ou secret venant de la WebView n'est accepté. Le jeton reste dans `RelayEndpoint`, non sérialisable et redacted par `Debug`.
- Vérification : tests Desktop puis tunnel réel temporaire vers cartae.app avec HTTP 200, jeton redacted.

### T020-T023 - Etats et cycle de vie

- Statut : Complété.
- Fonctionnement : la perte de l'enfant SSH possédé passe une seule fois par `reconnecting`, puis `failed` avec diagnostic redacted et action utilisateur `Réessayer`. L'événement `connection-state` vise exclusivement la WebView `main`; aucun panneau externe ne reçoit de capability.
- Cycle de vie : fermeture d'un profil ferme son panneau et son tunnel. `Drop` sur le transport ferme également l'enfant restant lors de la fermeture de l'application.
- Vérification : tests de transitions, de perte de tunnel, de session fermée, et test d'arrêt d'un enfant possédé sans toucher un second enfant.

### T024-T031 - Panneaux isolés et profil local

- Statut : Complété.
- Fonctionnement : les panneaux `panel-*` sont des enfants Tauri externes, limités aux URL `http://127.0.0.1:<port>/?token=...` issues d'une session vérifiée. Deux panneaux au maximum sont redimensionnés côte à côte; leurs origines restent visibles dans l'en-tête de la coque. Un endpoint loopback déjà accessible lit le même contrat sans SSH supplémentaire.
- Sécurité : la capability ne cible que `main`; les panneaux ne reçoivent aucune commande Tauri.
- Vérification : tests de limite, d'URL loopback, de fermeture indépendante, de capability et de chemin local.

### T032-T037 - Frontière navigateur, qualité et paquet

- Statut : Complété, sauf validation graphique humaine conservée dans T036.
- Frontière navigateur : ADR-018 et les artefacts SPEC documentent une future session associée à un profil et à une exécution, par un second tunnel local isolé. Aucun navigateur, VNC, noVNC ou tunnel générique n'est créé.
- Qualité : `cargo test -p bridget-daemon` a terminé avec 676 tests passants et 7 ignorés. La suite Desktop termine avec 30 tests passants.
- Paquet : Bridget Desktop 0.1.0 a été compilé sur macOS, signé ad hoc et installé dans `/Users/moi/Applications/Bridget Desktop 0.1.0.app`. Les détails et la limite Gatekeeper sont dans `evidence/macos-package.md`.

### T039 - Correctif d'acceptation du client macOS

- Statut : Complété.
- Cause : la coque HTML lit volontairement `window.__TAURI__.core.invoke`, mais la configuration Tauri n'activait pas `app.withGlobalTauri`. Dans l'application native, elle se croyait donc ouverte dans un navigateur.
- Correctif : `withGlobalTauri` est activé explicitement. L'accès visible « Endpoint déjà accessible depuis ce Mac » accepte `127.0.0.1:port` ou `localhost:port`, y compris tout port loopback tel que `17893` issu d'un tunnel déjà existant. Son jeton est demandé à la connexion, reste en mémoire dans la session et n'entre jamais dans le profil persistant.
- Présentation : le client réutilise désormais la feuille de style fondatrice de l'UI Bridget, complétée par une coque Desktop compacte plutôt qu'une identité visuelle indépendante.
- Vérification : 30 tests Desktop passent, le build macOS et la vérification de signature ad hoc passent avant installation.

### T040 - Clarification endpoint et tunnel SSH géré

- Statut : Complété.
- Modèle : Bridget Desktop ne distingue plus deux sortes de serveur. Il rejoint soit un endpoint déjà accessible depuis le Mac, sans tenter d'identifier son trajet, soit crée lui-même un tunnel SSH temporaire.
- Présentation : le formulaire commence par ce choix de trajet. Un endpoint accepte `127.0.0.1` ou `localhost` avec son port effectif. Le trajet SSH demande seulement le serveur SSH, son port, le compte et l'identité déjà présente sur le Mac.
- Vérification : syntaxe JavaScript, validation des profils loopback et suite Desktop complète avant reconstruction macOS.

## T041 - Tunnel SSH possédé et jeton transparent

- Statut : Complété.
- Modèle : un profil ne mémorise désormais que la destination SSH, le compte et une référence d'identité déjà présente sur le Mac. Bridget Desktop ouvre son propre forward loopback au début de sa session et le ferme avec elle. Aucun endpoint direct, port de relais ou jeton n'est demandé à l'opérateur.
- Migration : le stockage passe à la version 2. Les anciens profils SSH sont conservés. Les anciens profils d'endpoint direct sont écartés de façon atomique car ils ne peuvent pas devenir un tunnel possédé sans coordonnées SSH.
- Secret : la commande distante constante prépare uniquement `~/.local/bin` dans le `PATH`, puis exécute `bridget ui endpoint --json`. Elle est contrôlée par le client natif, ne reçoit aucune donnée de la WebView et le jeton reste seulement dans la session mémoire.
- Vérification : 30 tests Desktop, contrôle de format Rust, contrôle de syntaxe JavaScript, contrôle de diff, contrat réel du daemon actif sur cartae.app et paquet macOS signé ad hoc ont tous réussi.

### Correctif T041 - Approbation visible de l'identité SSH

- Cause : la première approbation utilisait `window.confirm` après une invocation Tauri asynchrone. Sur la WebView macOS, cette boîte ne constitue pas une interface fiable pour une décision de sécurité et pouvait être résolue comme un refus sans être visible.
- Correctif : un dialogue HTML natif de Bridget Desktop affiche le serveur et l'empreinte, avec deux choix explicites : `Annuler` par défaut ou `Approuver l'identité`. Le ticket mémoire déjà vérifié reste inchangé.
- Vérification : `node --check apps/bridget-desktop/ui/app.js`, `git diff --check` et le paquet macOS reconstruit et vérifié par `codesign --verify --deep --strict`.

## Etat final

L'implémentation est terminée et les preuves automatisées sont présentes. Il reste seulement la validation graphique manuelle d'acceptation listée dans `quickstart.md` avant de déclarer la SPEC entièrement acceptée par l'opérateur.
