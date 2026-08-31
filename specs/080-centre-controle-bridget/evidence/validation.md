# Preuves de validation - SPEC-080

Date : 2026-08-31

## Relais effectivement déployé

- Build : `cargo build --release -p bridget-daemon` réussi.
- Installation : `/home/moi/.local/bin/bridget` SHA-256 `addfbfd59131a5d05e5ffe9ea48db6de334ef9d5bc0478a4d956a8c26c50f083`.
- Services utilisateur : `bridget-daemon.service` et `bridget-ui.service` actifs depuis 10:10:18 UTC.
- Build et test réalisés dans le checkout Linux isolé `/tmp/bridget-human-sender.7CyaQE` du commit `a40f97dad7bf38778867c5dd4ad33ff2c4ec89dc`.
- Sauvegarde du binaire antérieur : `/home/moi/.local/bin/bridget.before-message-dedup-20260831-101018`.
- Le relais sert les marqueurs `project-presentation-overlay`, `project-onboarding-overlay` et `Navigation des projets` après redémarrage.
- Les routes authentifiées `GET /v1/projects/settings` et `GET /v1/projects` répondent `200`, avec une racine autorisée et un projet inscrit.

## Contrôles automatisés

- `node --check crates/bridget-daemon/assets/ui/app.js` : PASS.
- `node crates/bridget-daemon/assets/ui/app.js` : PASS, 100 tests, 0 échec.
- `cargo fmt --check` : PASS.
- `cargo test -p bridget-daemon spec_080 --lib` : PASS, 2 tests.
- `node --check apps/bridget-desktop/ui/app.js` : PASS.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml panels` : PASS, 1 test.
- `git diff --check` : PASS.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test desktop_commands` : PASS, 2 tests.
- `cargo test -p bridget-daemon --test ui_relay_test` sur le serveur : PASS, 25 tests, 0 échec.
- `cargo build --release -p bridget-daemon` sur le serveur : PASS, binaire installé puis services redémarrés.

## Régression de finition couverte

Le test UI vérifie que le centre contient l'overlay, la roue de bas de barre, la grille `SettingsRow` portée, le sélecteur de police de 11 rem, le sélecteur de taille de 5,5 rem et la bordure fine de contrôle. Cette preuve ne remplace pas l'évaluation visuelle humaine.

Le test UI couvre aussi la séparation de portées dans la navigation Projets : l'ancien pied global `Réglages / Retirer` est absent, l'overlay de présentation du projet est présent, les préférences d'initiales et de couleurs sont tolérantes aux valeurs invalides et restent locales, la colonne repliée conserve la liste d'icônes, et `Commande + virgule` ouvre le même centre de contrôle.

## Correctif des actions Ajouter et Importer

- Cause observée : l'interface attendait `coordinator_options` et une route `POST /v1/projects/confirm` absents du relais. Les clics ne pouvaient donc ni ouvrir un parcours utile ni inscrire un projet.
- Correctif : le dialogue local prévisualise puis confirme la création ou l'import. Le relais liste et inscrit les projets par la capacité `ProjectRegistryV1`, revalide les racines et expose aussi les actions de retrait, réactivation et reconnexion.
- `cargo test -p bridget-daemon spec_080_confirmation_ui_cree_puis_enregistre_le_projet_dans_le_registre --lib` : PASS. Le test simule le registre, exige la négociation de capacité, vérifie la racine canonique et atteste que le dossier est créé uniquement après la prévisualisation.
- `cargo fmt --check`, `cargo test -p bridget-daemon --test ui_relay_test` et `cargo build --release -p bridget-daemon` ont passé dans le clone propre du commit `cb327e2e9e3c84b9e44c44b84be085f0e32c0a7f`.

## Correctif des actions Desktop

- La délégation de clic du listing de profils couvre Connecter, Réessayer, Ouvrir, Déconnecter, Modifier et Retirer.
- La carte Desktop ne rend plus de lien textuel Réglages. La roue de la barre basse du panneau distant est l'unique entrée de réglages serveur.
- Le test Desktop statique atteste les cinq branches et l'absence de `data-action="settings"` ainsi que de `openServerSettings`.
- Sonde SSH des arguments de production : `moi@cartae.app:2222` réussit. Le profil Carte enregistré en `Moi` est refusé par SSH et doit devenir `moi` après fermeture de l'application.

## Correctif de duplication des panneaux et de l'icône

- Cause confirmée : deux WebViews enfants étaient autorisées et redimensionnées à `largeur / nombre de panneaux`. Avec Loin et Carte ouverts, la fenêtre affichait donc deux interfaces reliées côte à côte.
- Le registre Desktop est limité à un panneau. Toute ouverture ferme le panneau visible avant de créer le suivant, sans fermer le tunnel SSH associé.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml panels` : PASS, 1 test.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test two_panels` : PASS, 1 test.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test desktop_commands` : PASS, 3 tests.
- Le bundle macOS reconstruit contient `CFBundleIconFile=icon.icns` et `Contents/Resources/icon.icns`; sa signature ad hoc passe `codesign --verify --deep --strict`.

## Finition de la liste d'agents

- La grille d'agents est ancrée en haut avec `align-content: start` et des lignes `min-content`, afin qu'un petit nombre d'agents ne soit jamais réparti sur toute la hauteur de la barre.
- La recherche a une marge basse de `1rem` avant la liste.

## Correctif de l'identité humaine

- Le test Node vérifie que la présence interne `humain` est exclue de la flotte, de sa projection latérale et de toute sélection restaurée.
- Le test Rust ciblé `garde_destinataire_refuse_humain_absent_et_arrete_sans_refuser_un_agent_vivant` passe et vérifie que le relais refuse `humain` tout en acceptant un agent vivant.
- La suite d'intégration complète `ui_relay_test` ne peut pas être conclue sur ce Mac : sa fixture lance le daemon dans un `HOME` temporaire qui n'est pas résolu comme attendu par macOS. Elle échoue avant les scénarios métier, sur `daemon non prêt`.
- Dans le checkout Linux de déploiement, `ui_relay_test` passe 25/25. Après redémarrage, `GET /v1/snapshot` et l'asset `app.js` répondent `200`, l'asset contient les gardes de flotte, et une requête valide adressée à `humain` renvoie `400 human_recipient` sans créer de message.

## Correctif de duplication des messages utilisateur

- Mesure runtime : le snapshot du fil `rc1` ne contient qu'une occurrence de chacun des messages visibles en double, tandis que le journal de l'agent contient un `turn_start` avec le même `message_id`.
- Reproduction exacte avant correctif : la projection de la bulle ledger et du `turn_start` portant `6f153302a7494` produisait 2 bulles.
- Après correctif : la même projection produit 1 bulle avec `delivery_id=6f153302a7494`. Le programme Node complet passe 100/100, `ui_relay_test` passe 25/25 sur Linux, et l'asset déployé contient `renderedUserMessageIds`.

## Paquet macOS attesté

- Bundle reconstruit avec un seul panneau distant et l'icône Bridget intégrée.
- Manifeste vérifié : `CFBundleDisplayName=Bridget`, `CFBundleName=Bridget`, `CFBundleIdentifier=app.cartae.bridget-desktop`.
- Intégrité locale : `codesign --verify --deep --strict` passe après signature ad hoc.

## Rectification de l'icône validée

- L'icône restaurée provient de l'ancien build qui a servi de référence visuelle, non du fichier `/Users/moi/Downloads/bridget.svg` qui avait perdu le fond noir et l'échelle de mascotte.
- Les fichiers source et macOS ont le même rendu attendu : fond noir légèrement dégradé, mascotte rose volontairement grande.

## Vérification manuelle restante

1. Ouvrir Bridget sur un profil déjà relié.
2. Ouvrir la roue en bas de la barre gauche, puis Typographie.
3. Vérifier les titres, sous-titres, deux sélecteurs alignés à droite et les aperçus sous les lignes.
4. Changer puis restaurer une préférence locale, et vérifier qu'aucune mutation serveur n'est déclenchée.
5. Vérifier la page Serveur sans appliquer de réglage, puis Usage et facturation sans supposer un coût absent.
6. Fermer Bridget, vérifier que Carte utilise `moi`, installer le bundle corrigé dans `/Applications/Bridget.app`, reconnecter Loin puis Carte et vérifier qu'un seul panneau pleine largeur reste visible et que l'icône Bridget est présente dans le Dock.
7. Vérifier sous les boutons macOS la colonne Projets : « Toute la flotte » doit expliciter « Tous projets confondus », un projet doit afficher une tuile avec initiales, et le pied de colonne ne doit afficher ni « Retirer » ni « Réglages ».
8. Ouvrir `…` ou faire un clic droit sur un projet, changer ses initiales puis sa couleur dans l'overlay, refermer puis vérifier la persistance locale. Replier la colonne et vérifier que les tuiles restent visibles.
9. Presser `Commande + virgule`, vérifier que le même overlay de centre de contrôle s'ouvre au-dessus de la conversation.

Aucune validation visuelle ambiguë, aucun clic non confirmé et aucune opération de serveur non demandée ne sont comptés comme preuve d'acceptation.

## Correctif overlay et permission native

- `node --test crates/bridget-daemon/assets/ui/app.js` : PASS, 100 tests. Le contrat vérifie l'absence de `tabindex="0"` sur le fil, l'héritage de police de la barre agents et des contrôles, la police monospace et l'ascenseur du centre de contrôle.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test capabilities` : PASS, 2 tests. Le test confirme que le panneau distant ne peut demander que l'autorisation native de notification sur `http://127.0.0.1:*`, sans permission de commande Tauri générale ni permission d'émettre une notification.
- Vérification manuelle à effectuer après installation du bundle : ouvrir puis fermer le centre de contrôle et cliquer sur son arrière-plan. Aucun cadre bleu ne doit persister autour du fil. Changer la police d'interface doit mettre à jour les deux barres latérales et les contrôles du centre. Cliquer « Activer les notifications » doit faire apparaître la demande macOS ou afficher un libellé de refus ou de nouvel essai explicite.
- Déploiement serveur : build isolé du commit `ab75bab`, SHA-256 `a6ca650772b0d9a40af1fa331413db85286473d464a4b95feb2f914cc09e4881`, installation dans `/home/moi/.local/bin/bridget`, puis redémarrage réussi de `bridget-daemon.service` et `bridget-ui.service`. Les assets effectivement servis contiennent les marqueurs de l'overlay, de l'ascenseur et du nouvel état de notification.
