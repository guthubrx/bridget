# Preuves de validation - SPEC-074

Date : 2026-08-30.

## Contrat serveur et client

- `/home/moi/.cargo/bin/cargo test -p bridget-daemon` dans le worktree de session a terminé avec 676 tests passants et 7 ignorés.
- `/home/moi/.cargo/bin/cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml` a terminé avec 30 tests passants : profils, stockage atomique, identité SSH, arguments SSH, découverte, relais HTTP, capacités, deux panneaux, arrêt d'enfant possédé, cycle de vie, perte de tunnel et diagnostics sans secrets.
- Le contrat réel `bridget ui endpoint --json` a été relu sur cartae.app. Sa forme était `version=1`, port non nul, jeton présent mais jamais affiché.

## Trajet réel cartae.app

Depuis le Mac, un tunnel de vérification possédé par la commande a relié un port loopback temporaire au relais `127.0.0.1:17888` de cartae.app. La requête HTTP authentifiée à travers ce tunnel a renvoyé `200`.

- Port local temporaire : `39174`.
- Port de relais distant : `17888`.
- Après la vérification, le seul processus SSH enfant a été identifié comme tel, arrêté, puis son absence a été contrôlée. Aucun navigateur ni daemon Bridget n'a été arrêté.

## Parcours utilisateur couvert

- Ajout, édition et retrait confirmé de profils avec endpoint loopback déjà accessible ou tunnel SSH géré dans la coque native.
- Première empreinte SSH affichée puis acceptée explicitement avant la connexion.
- Tunnel SSH loopback, contrôle HTTP du relais et panneau externe limité à `127.0.0.1`.
- Deux panneaux maximum, isolés par profil, et fermeture du premier sans suppression du second.
- Endpoint Bridget loopback sans tunnel SSH créé par l'application, avec jeton conservé en mémoire seulement.

## Correctif d'acceptation macOS

- Le bundle initial ne rendait pas l'API globale Tauri à la coque statique. La cause est corrigée par `app.withGlobalTauri: true`, validée par la construction macOS.
- L'accès est présenté comme un « endpoint déjà accessible depuis ce Mac » : il accepte `127.0.0.1:port` ou `localhost:port`, y compris un port comme `17893` issu d'un tunnel SSH déjà ouvert, sans créer un second modèle « ce Mac » ni tenter de classifier ce tunnel.
- Le thème racine embarqué est celui de `crates/bridget-daemon/assets/ui/theme.css`; `desktop.css` ne contient que la mise en page compacte propre au client.
- Le bundle final est vérifié avec `codesign --verify --deep --strict` après signature ad hoc.

## Limite restante de validation humaine

Le paquet macOS est compilé et signé ad hoc, mais l'ouverture graphique manuelle reste à effectuer par l'opérateur : créer un profil, confirmer l'empreinte connue de cartae.app et observer le panneau. Aucun faux verdict visuel n'est consigné à la place de cette interaction.

## Smoke macOS

L'application installée a été ouverte en arrière-plan sur le Mac, son binaire `bridget-desktop` a été observé vivant, puis cette instance de test a été fermée proprement. Ce smoke prouve le lancement du paquet mais ne remplace pas le parcours visuel opérateur ci-dessus.

## T041 - Tunnel SSH possédé

- `/home/moi/.cargo/bin/cargo fmt --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml -- --check` : succès.
- `node --check apps/bridget-desktop/ui/app.js` : succès.
- `git diff --check` : succès.
- `/home/moi/.cargo/bin/cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml` : succès, 30 tests Desktop.
- `/home/moi/.cargo/bin/cargo test -p bridget-daemon endpoint_ui_exige_un_contrat_json_ferme --lib` : succès, contrat serveur vérifié.
- Vérification réelle depuis le Mac : la commande SSH constante `PATH="$HOME/.local/bin:$PATH"; exec bridget ui endpoint --json` résout Bridget et le JSON de l'endpoint est valide sans journaliser ni afficher le jeton. Le daemon est actif, PID `122945`, démarré le `2026-08-30 15:50:08 UTC`.
- `/Users/moi/.cargo/bin/cargo tauri build --bundles app` : succès. Paquet produit : `/tmp/bridget-desktop-managed-ssh-package.khBcgR/source/src-tauri/target/release/bundle/macos/Bridget Desktop.app`.

Le parcours graphique final reste une acceptation opérateur : à la première connexion, confirmer l'empreinte SSH de cartae.app. Après cette confirmation, le profil se reconnecte automatiquement à chaque lancement de Bridget Desktop et le jeton reste transparent.

## Correctif T041 - Dialogue d'empreinte SSH

- Observation réelle : le profil `Loin` avait `host_fingerprint: null` et l'interface affichait « L'identité SSH n'a pas été approuvée » sans dialogue visible.
- Cause corrigée : `window.confirm` est remplacé par un élément `<dialog>` de la coque Desktop. L'empreinte et le serveur sont visibles et le bouton par défaut est `Annuler`.
- `node --check apps/bridget-desktop/ui/app.js` et `git diff --check` : succès.
- Paquet macOS reconstruit, signé ad hoc et vérifié avec `codesign --verify --deep --strict` : succès.
