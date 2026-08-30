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

- Ajout, édition et retrait confirmé de profils SSH ou locaux dans la coque native.
- Première empreinte SSH affichée puis acceptée explicitement avant la connexion.
- Tunnel SSH loopback, contrôle HTTP du relais et panneau externe limité à `127.0.0.1`.
- Deux panneaux maximum, isolés par profil, et fermeture du premier sans suppression du second.
- Profil local sans tunnel SSH, avec erreur relais distincte.

## Limite restante de validation humaine

Le paquet macOS est compilé et signé ad hoc, mais l'ouverture graphique manuelle reste à effectuer par l'opérateur : créer un profil, confirmer l'empreinte connue de cartae.app et observer le panneau. Aucun faux verdict visuel n'est consigné à la place de cette interaction.

## Smoke macOS

L'application installée a été ouverte en arrière-plan sur le Mac, son binaire `bridget-desktop` a été observé vivant, puis cette instance de test a été fermée proprement. Ce smoke prouve le lancement du paquet mais ne remplace pas le parcours visuel opérateur ci-dessus.
