# Tâches - SPEC-080 Centre de contrôle Bridget

## Rectification de suivi - 2026-08-31

## État correctif - 2026-08-31

Les tâches T010 à T013 sont réalisées dans le relais : structure de l'overlay, navigation locale, roue basse, styles accessibles, tests Node et portage de finition T3. Leur résultat est décrit dans `implementation.md` et les preuves dans `evidence/validation.md`.

T041 reste volontairement ouverte : aucune validation visuelle humaine ne peut être remplacée par une capture ambiguë. Le paquet macOS est maintenant construit et signé localement, mais la copie ouverte dans /Applications n'est pas écrasée sans choix explicite.

Les coches de la première tranche ne valent pas acceptation produit. La SPEC reste ouverte jusqu'à la vérification visuelle du parcours complet sur le relais réellement servi.

## Correctif navigation Projets - 2026-08-31

- [x] T042 [US1] Déplacer les actions de retrait et de présentation du projet dans un menu contextuel par projet, supprimer le pied de colonne ambigu et conserver les réglages globaux dans la roue unique.
- [x] T043 [US1] Ajouter la tuile locale initiales/couleur, le repli de la colonne sans perte d'icônes et la réserve pour les boutons macOS dans `crates/bridget-daemon/assets/ui/app.js`, `index.html` et `theme.css`.
- [x] T044 [US1] Ajouter le raccourci `Commande + virgule`, les preuves unitaires et le parcours de validation manuelle dans les artefacts de la SPEC-080.
- [x] T045 [US1] Raccorder les boutons `+`, `Importer`, retrait, réactivation et reconnexion au registre de projets via le relais, avec dialogue de prévisualisation et confirmation explicite.

## Dépendances

`Fondations -> US1 -> US3 -> US4 -> US5 -> US6`
`Fondations -> US2` peut avancer après la persistance locale.
US1 reste livrable avec les catégories en lecture seule; US4 active la première écriture serveur; US5 ne dépend pas d'un tarif prérempli.

## Phase 1 - Préparation

- [x] T001 Créer `tests/features/080-centre-controle-bridget.feature` avec les scénarios métier engin, portées, aperçu, conflit, usage inconnu et maintenance informative.
- [x] T002 Mettre à jour `specs/080-centre-controle-bridget/implementation.md` avec le journal de tâches, les commandes de preuve et l'interdiction de déployer ou installer automatiquement.

## Phase 2 - Fondations communes

- [ ] T003 [P] Ajouter les types fermés de contrôle, les erreurs et les tests de validation dans `crates/bridget-daemon/src/control_settings.rs`; prouver qu'aucune clé libre, commande, secret ou valeur hors schéma n'est acceptée.
- [ ] T004 Ajouter et tester les migrations reçus, tarifs et dimensions d'usage dans `crates/bridget-daemon/src/store.rs`; vérifier index de période et absence de contenu utilisateur.
- [ ] T005 Ajouter la projection et les routes relay v1 dans `crates/bridget-daemon/src/ui.rs`; vérifier token, version, limites de corps et réponses de refus bornées.
- [ ] T006 Adapter `crates/bridget-daemon/src/project_policy.rs` et `crates/bridget-daemon/src/ui.rs` pour que la clé de racines réutilise exactement validation, génération et écriture atomique existantes, tout en conservant les routes `/v1/projects/*` compatibles.
- [ ] T007 Créer les tests de contrat et d'idempotence de contrôle dans `crates/bridget-daemon/src/ui.rs` et `crates/bridget-daemon/src/control_settings.rs`; couvrir absence de capacité, lecture seule, corps inconnu et même `command_id`.
- [x] T008 Ajouter les helpers purs de requêtes control et leurs tests Node dans `crates/bridget-daemon/assets/ui/app.js`; vérifier que les URL réutilisent le jeton déjà présent et ne construisent aucune commande.

## Phase 3 - User Story 1: ouvrir et organiser le centre de contrôle

But: l'opérateur ouvre l'engrenage dans la barre gauche et navigue sans perdre sa conversation.

Test indépendant: ouvrir puis fermer la vue conserve agent sélectionné, brouillon, défilement et panneau redimensionnable; la navigation est utilisable au clavier.

- [ ] T009 [P] [US1] Ajouter les tests Node de navigation, focus et conservation d'état dans `crates/bridget-daemon/assets/ui/app.js` avant le rendu.
- [x] T010 [US1] Ajouter l'engrenage, les régions accessibles et les conteneurs de centre de contrôle dans `crates/bridget-daemon/assets/ui/index.html`.
- [x] T011 [US1] Étendre l'état et le montage de `crates/bridget-daemon/assets/ui/app.js` avec l'ouverture, fermeture, recherche locale et retour conversation sans requête inutile.
- [x] T012 [US1] Ajouter les styles de barre basse, panneaux et focus dans `crates/bridget-daemon/assets/ui/theme.css`, en respectant les tailles de barre existantes et la réduction de mouvement.
- [x] T013 [US1] Exécuter le programme de tests Node de `crates/bridget-daemon/assets/ui/app.js` et consigner le résultat dans `specs/080-centre-controle-bridget/implementation.md`.

## Phase 4 - User Story 2: préférences du Mac

But: le nom opérateur, thème, fuseau et lisibilité sont locaux au Mac et persistent sans toucher un serveur.

Test indépendant: le document de préférence corrompu retombe sur des valeurs sûres, et aucune commande réseau n'est produite par une modification locale.

- [ ] T014 [P] [US2] Ajouter les tests de sérialisation, validation IANA, valeurs par défaut et remplacement atomique dans `apps/bridget-desktop/src-tauri/src/preferences.rs`.
- [ ] T015 [US2] Implémenter `DesktopPreferencesV1` et son store atomique dans `apps/bridget-desktop/src-tauri/src/preferences.rs`, en reprenant le style de `profile_store.rs` sans y mélanger les profils SSH.
- [ ] T016 [US2] Enregistrer les commandes Tauri de lecture et mise à jour de préférences dans `apps/bridget-desktop/src-tauri/src/lib.rs`; prouver qu'elles ne retournent ni profil secret ni jeton relay.
- [ ] T017 [US2] Ajouter l'écran et les contrôles de préférences locales dans `apps/bridget-desktop/ui/index.html`, `apps/bridget-desktop/ui/app.js` et `apps/bridget-desktop/ui/desktop.css`.
- [ ] T018 [US2] Étendre `apps/bridget-desktop/src-tauri/tests/desktop_commands.rs` et `apps/bridget-desktop/src-tauri/tests/secrets_and_diagnostics.rs` pour valider persistance, refus des données hors schéma, absence de fuite et absence de requête tunnel lors d'un changement local.

## Phase 5 - User Story 3: consulter un serveur enregistré

But: une fiche serveur montre identité, capacités et portées réelles, même lorsque le serveur est déconnecté ou ne supporte aucune écriture.

Test indépendant: un relais sans capacité répond par catégories lecture seule expliquées, sans valeur inventée ni écran d'erreur trompeur.

- [ ] T019 [P] [US3] Ajouter les tests Rust de projection `ServerControlSnapshotV1` dans `crates/bridget-daemon/src/control_settings.rs` pour états connecté, indisponible, capacité absente et valeur historique invalide.
- [ ] T020 [US3] Implémenter l'identité bornée, catégories et descripteurs de portée dans `crates/bridget-daemon/src/control_settings.rs` et les exposer par `crates/bridget-daemon/src/ui.rs`.
- [ ] T021 [US3] Ajouter les requêtes de lecture, rendu de serveur, badges de portée et états déconnectés dans `crates/bridget-daemon/assets/ui/app.js`.
- [ ] T022 [US3] Compléter le rendu et les textes accessibles de la vue serveur dans `crates/bridget-daemon/assets/ui/index.html` et `crates/bridget-daemon/assets/ui/theme.css`.
- [ ] T023 [US3] Ajouter les tests de routes relay et UI pour une capacité retirée pendant la consultation dans `crates/bridget-daemon/src/ui.rs` et `crates/bridget-daemon/assets/ui/app.js`.

## Phase 6 - User Story 4: modifier un réglage serveur de façon contrôlée

But: l'opérateur prévisualise la modification de racines, confirme localement puis reçoit le résultat durable ou un refus exact.

Test indépendant: une génération concurrente ou un réglage interdit ne modifie aucune valeur; une demande répétée retourne le même reçu.

- [ ] T024 [P] [US4] Écrire les tests de prévisualisation et application atomique dans `crates/bridget-daemon/src/control_settings.rs` avant la mutation.
- [ ] T025 [US4] Implémenter prévisualisation, application et reçu idempotent dans `crates/bridget-daemon/src/control_settings.rs` et `crates/bridget-daemon/src/store.rs`.
- [ ] T026 [US4] Brancher les routes `POST /v1/control/settings/preview` et `POST /v1/control/settings/apply` dans `crates/bridget-daemon/src/ui.rs`, avec conflit de génération et réponse de refus explicite.
- [ ] T027 [US4] Ajouter le parcours UI delta, confirmation locale, erreur et reçu dans `crates/bridget-daemon/assets/ui/app.js`; ne jamais annoncer un succès avant la réponse apply.
- [ ] T028 [US4] Ajouter les contrôles accessibles de confirmation et les styles de delta destructif ou sûr dans `crates/bridget-daemon/assets/ui/index.html` et `crates/bridget-daemon/assets/ui/theme.css`.
- [ ] T029 [US4] Ajouter les tests Node et relais pour confirmation locale obligatoire, aperçu, conflit, rollback et rejeu identique dans `crates/bridget-daemon/assets/ui/app.js` et `crates/bridget-daemon/src/ui.rs`.

## Phase 7 - User Story 5: usage et estimation API

But: l'opérateur comprend les jetons observés et le coût seulement lorsque les données permettent une estimation honnête.

Test indépendant: une fenêtre sans échantillon, un modèle inconnu ou un tarif absent affiche une absence explicitement étiquetée et jamais zéro.

- [ ] T030 [P] [US5] Écrire les tests de migration, agrégat, filtre, jour IANA, dimension inconnue et tarif absent dans `crates/bridget-daemon/src/store.rs`.
- [ ] T031 [US5] Étendre les échantillons d'usage attestés et leurs index dans `crates/bridget-daemon/src/store.rs` et `crates/bridget-daemon/src/daemon.rs`, sans déduire rétrospectivement fournisseur ou modèle.
- [ ] T032 [US5] Implémenter `UsageRate` daté, correspondance exacte et calcul d'estimation dans `crates/bridget-daemon/src/control_settings.rs` et `crates/bridget-daemon/src/store.rs`.
- [ ] T033 [US5] Exposer `GET /v1/control/usage` avec couverture, provenance, groupes et résultat non tarifé dans `crates/bridget-daemon/src/ui.rs`.
- [ ] T034 [US5] Ajouter la page Usage, filtres, total fournisseur, courbe journalière et libellés Estimation API ou Non tarifé dans `crates/bridget-daemon/assets/ui/app.js`, `index.html` et `theme.css`.
- [ ] T035 [US5] Ajouter les tests Node du rendu usage et les tests Rust de réponse relay dans `crates/bridget-daemon/assets/ui/app.js` et `crates/bridget-daemon/src/ui.rs`.

## Phase 8 - User Story 6: mises à jour et diagnostics informatifs

But: l'opérateur voit version, dernière vérification et diagnostics bornés, sans déclencher une maintenance distante.

Test indépendant: une version inconnue ou source indisponible reste informative et aucun POST de maintenance n'existe.

- [ ] T036 [P] [US6] Ajouter les tests de projection maintenance et de non-divulgation dans `crates/bridget-daemon/src/ui.rs` et `apps/bridget-desktop/src-tauri/tests/secrets_and_diagnostics.rs`.
- [ ] T037 [US6] Exposer une projection lecture seule de version, capacité et diagnostic borné dans `crates/bridget-daemon/src/ui.rs`.
- [x] T038 [US6] Ajouter les vues Mises à jour et Diagnostics dans `crates/bridget-daemon/assets/ui/app.js`, `index.html` et `theme.css`, sans bouton d'installation ou de redémarrage.

## Phase 9 - Finition et preuves transverses

- [ ] T039 Exécuter `cargo fmt --check`, les tests ciblés daemon et transport, puis les tests Bridget Desktop sur macOS; mesurer l'ouverture et la navigation locale sous 150 ms sur fixture et documenter toute indisponibilité factuelle dans `specs/080-centre-controle-bridget/implementation.md`.
- [ ] T040 Exécuter le test Node de `crates/bridget-daemon/assets/ui/app.js`, `git diff --check` et vérifier les contrats `specs/080-centre-controle-bridget/contracts/` contre les réponses réelles.
- [ ] T041 Effectuer la vérification manuelle de `specs/080-centre-controle-bridget/quickstart.md` avec un serveur enregistré, sans appliquer de maintenance, et consigner les résultats dans `specs/080-centre-controle-bridget/evidence/validation.md`.
- [ ] T042 Relire le diff pour minimalisme, complexité, sécurité, accessibilité et responsabilité future; mettre à jour `spec.md`, `implementation.md` et `audit.md` avec les preuves effectives.

## Opportunités de parallélisme

- T003, T004 et T008 touchent des responsabilités distinctes mais T005 attend leurs types et migrations.
- Après T008, T009 et T014 peuvent avancer en parallèle sur UI serveur et stockage Desktop.
- T019 et T024 peuvent être écrits avant les implémentations correspondantes, mais US4 attend US3.
- T030 et T036 n'ont pas de fichiers communs, mais US5 attend les routes et le modèle final de US4.

## Stratégie d'implémentation

1. Rendre le centre lisible et navigationnel, sans mutation.
2. Livrer une unique écriture sûre de serveur avec toutes les garanties de concurrence et de reçu.
3. Ajouter l'usage attesté et les estimations seulement lorsqu'elles peuvent être expliquées.
4. Terminer par les informations de maintenance, les tests et la validation humaine.

## Validation Article XX

- T006, T008, T015 et T031 réduisent la charge de maintenance en étendant les composants qui portent déjà les invariants.
- T003 est une abstraction justifiée: le même catalogue est consommé par validation, route relay et rendu UI.
- T004 et T032 ajoutent de la persistance uniquement parce que reçu et tarif daté sont nécessaires à un comportement vérifiable; aucune dépendance n'est ajoutée.
- Toutes les tâches possèdent une preuve observable, un chemin et un comportement vérifiable.
