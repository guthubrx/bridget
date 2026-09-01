# Tâches - SPEC-083 Artefacts HTML sandboxés et navigateur latéral

**Entrée**: `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/`,
`quickstart.md`, `reuse-audit.md`
**Précondition**: les contrats, versions et blobs de SPEC-082 sont livrés.
**Statut de ce fichier**: planifié uniquement. Aucune tâche ci-dessous n'est
implémentée dans cette session de préparation.

## Dépendances et stratégie

~~~text
Fondations Desktop et contrats de SPEC-082
  ├─ US1 Sandbox HTML
  │   └─ US2 Sauvegarde en version enfant
  ├─ US3 Browser et disposition droite
  │   └─ US4 Onglets et recherche
  └─ US5 Frontières et sécurité
~~~

MVP: US1 + US3. Un artefact HTML est utile seulement si son isolement est
prouvé, et l'ouverture développée est utile seulement si le Browser est une
surface humaine sans privilège agent. Les actions DOM par agent restent hors
périmètre.

## Phase 1 - Préparation et fondations

- [x] T001 Vérifier dans `apps/bridget-desktop/src-tauri/Cargo.toml` les APIs Tauri WebView nécessaires à la version cible macOS et activer seulement les features minimales requises.
- [x] T002 Ajouter les types `SandboxArtifactVersionV1`, `SandboxRuntimeStateV1` et `BrowserNavigationRequestV1` dans `apps/bridget-desktop/src-tauri/src/artifact_sandbox.rs`, en réutilisant les IDs et versions de SPEC-082.
- [x] T003 Étendre `apps/bridget-desktop/src-tauri/src/panels.rs` pour gérer le panneau droit unique et ses enfants Browser/cadre, sans seconde régie de WebViews ni augmentation non justifiée de la limite.
- [x] T004 Étendre `apps/bridget-desktop/src-tauri/src/preferences_store.rs` avec `BrowserPanelStateV1`, profil `Ce Mac`, récupération externe désactivée par défaut et migration atomique, sans sérialiser cookies ni secrets.
- [x] T005 [P] Ajouter les capabilities sans privilège `apps/bridget-desktop/src-tauri/capabilities/browser-panel.json` et `artifact-frame.json`, distinctes de `desktop-shell`.
- [x] T006 [P] Préparer les fixtures de confinement dans `apps/bridget-desktop/src-tauri/tests/fixtures/sandbox/` : fetch, WebSocket, popup, iframe, formulaire, parent/top, `file:` et messages hors contrat.
- [x] T007 [P] Ajouter les tests de schéma et de migration des préférences dans `apps/bridget-desktop/src-tauri/tests/browser_preferences_test.rs`.

## Phase 2 - US1 Artefact HTML interactif sans privilège (P1)

**Objectif**: rendre inline un HTML/JavaScript utile mais non fiable, avec ses
données déclarées seulement et une hauteur limitée à 1 200 px.

**Critère indépendant**: une visualisation filtre ses propres données, mais
échoue explicitement à joindre réseau, parent, fichiers ou Tauri ; un contenu
trop haut offre une ouverture développée.

- [x] T008 [US1] Implémenter l'enveloppe de document et les tickets courts dans `apps/bridget-desktop/src-tauri/src/artifact_sandbox.rs`, à partir des blobs canoniques de SPEC-082 et sans interpolation HTML dangereuse.
- [x] T009 [US1] Créer `crates/bridget-daemon/assets/ui/artifact-sandbox-host.js` afin de construire l'iframe `sandbox="allow-scripts"`, la CSP fixe et aucun bridge Tauri ou Bridget.
- [x] T010 [US1] Implémenter le protocole fermé de `contracts/sandbox-runtime-v1.md` dans `artifact-sandbox-host.js`, avec validation d'instance, tailles, hauteur 0..1200 et rejet journalisé des messages inconnus.
- [x] T011 [US1] Ajouter dans `crates/bridget-daemon/src/ui.rs` les routes locales à ticket de lecture sandbox, en refusant `file:`, URL arbitraire, route expirée et artefact hors portée projet.
- [x] T012 [US1] Ajouter le rendu inline et l'action explicite Agrandir dans `crates/bridget-daemon/assets/ui/app.js`, sans changer le renderer Markdown non exécutable.
- [x] T013 [US1] Ajouter les états de blocage, détails de manifeste, copie/export du code et des données non exécutés, et erreurs non muettes dans `crates/bridget-daemon/assets/ui/theme.css` et `artifact-sandbox-host.js`.
- [x] T014 [US1] Écrire les tests de confinement et de protocole dans `apps/bridget-desktop/src-tauri/tests/artifact_sandbox_test.rs`, couvrant chaque fixture hostile et l'absence d'accès Tauri.
- [x] T015 [US1] Écrire les tests renderer dans `crates/bridget-daemon/assets/ui/artifact-sandbox-host.test.mjs`, couvrant hauteur, CSP, origine opaque, message invalide et erreur explicite.

## Phase 3 - US2 État temporaire et version enfant explicite (P1)

**Objectif**: laisser explorer l'artefact sans muter l'historique, puis créer
une version nouvelle seulement sur demande visible de l'opérateur.

**Critère indépendant**: modifier un filtre, fermer/réouvrir sans sauvegarder,
puis enregistrer et constater une version enfant avec provenance, sans changer
l'original.

- [x] T016 [US2] Ajouter la validation de `ui_state` borné et la commande explicite de sauvegarde dans `apps/bridget-desktop/src-tauri/src/artifact_sandbox.rs`, conformément à `data-model.md`.
- [x] T017 [US2] Étendre `crates/bridget-daemon/src/artifact_service.rs` de SPEC-082 pour créer une version enfant HTML avec référence parent, provenance et reçu explicite.
- [x] T018 [US2] Ajouter les contrôles Enregistrer comme nouvelle version, Annuler et leurs retours dans `crates/bridget-daemon/assets/ui/artifact-sandbox-host.js` et `app.js`.
- [x] T019 [US2] Écrire les tests de non-mutation, de limite d'état et de version enfant dans `apps/bridget-desktop/src-tauri/tests/artifact_sandbox_test.rs` et `crates/bridget-daemon/tests/artifact_lifecycle_test.rs`.

## Phase 4 - US3 Browser d'opérateur et disposition droite (P1)

**Objectif**: ouvrir, masquer, restaurer et agrandir une vraie surface Browser
isolée de Safari/Chrome et des agents.

**Critère indépendant**: ouvrir une version publiée, une page locale et une URL
HTTPS volontaire, masquer/réafficher le panneau et effacer le profil sans
perdre les artefacts canoniques.

- [x] T020 [US3] Étendre `apps/bridget-desktop/src-tauri/src/panels.rs` et `lib.rs` pour créer le WebView `browser-*` sans capability Tauri, avec profil séparé, navigation contrôlée et réutilisation de la disposition existante.
- [x] T021 [US3] Implémenter dans `apps/bridget-desktop/src-tauri/src/lib.rs` la remise typée de `bridget-open:` au Browser interne au lieu de `Command::new("open")`, après validation HTTPS ou référence publiée.
- [x] T022 [US3] Ajouter l'effacement explicite des données Browser dans `apps/bridget-desktop/src-tauri/src/lib.rs` et `preferences_store.rs`, via l'API WebView puis incrément de génération de profil.
- [x] T023 [US3] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les boutons de disposition en reprenant la forme et les états accessibles de `PanelRightIcon`, `Maximize2Icon` et `Minimize2Icon` documentés depuis T3, sans copier son composant.
- [x] T024 [US3] Ajouter dans `crates/bridget-daemon/assets/ui/theme.css` les états panneau masqué/visible/maximisé et le focus clavier, sans détruire la conversation ou le Browser lors d'un toggle.
- [x] T025 [US3] Écrire les tests Desktop de navigation, URL interdite, redirection, ouverture locale, persistance de panneau et effacement de profil dans `apps/bridget-desktop/src-tauri/tests/browser_panel_test.rs`.

## Phase 5 - US4 Onglets Artefacts, Fichiers, Liens et Activité (P2)

**Objectif**: retrouver le contexte publié dans le panneau sans scanner le Mac
ou déclencher un fetch réseau simplement parce qu'un lien existe.

**Critère indépendant**: des artefacts et fichiers de deux projets restent
cloisonnés dans les onglets ; une recherche globale est volontaire ; un lien
ne charge rien avant clic opérateur.

- [x] T026 [US4] Ajouter les projections d'onglets Browser, Artefacts, Fichiers, Liens et Activité dans `crates/bridget-daemon/assets/ui/app.js`, en réutilisant les index/actes de SPEC-082.
- [x] T027 [US4] Ajouter les routes de listes bornées dans `crates/bridget-daemon/src/ui.rs`, sans lecture de système de fichiers ni prévisualisation distante automatique.
- [x] T028 [US4] Ajouter la recherche globale explicitement déclenchée et la portée projet par défaut dans `crates/bridget-daemon/assets/ui/app.js`.
- [x] T029 [US4] Écrire les tests de portée, fichiers publiés seulement, lien inactif et recherche globale dans `crates/bridget-daemon/tests/artifact_relay_test.rs` et `apps/bridget-desktop/src-tauri/tests/browser_panel_test.rs`.

## Phase 6 - US5 Frontières, provenance et dégradations (P2)

**Objectif**: rendre la séparation Browser/artefact/agent observable, tout en
renvoyant collecte et restauration à Bridget avec leur provenance.

**Critère indépendant**: ouvrir les détails de sécurité, demander une source,
simuler une indisponibilité et vérifier que aucune session Browser ou secret ne
traverse le contrat.

- [x] T030 [US5] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les détails de capacités, données injectées, accès refusés et provenance de chaque artefact HTML.
- [x] T031 [US5] Raccorder les demandes d'ouverture de source au collecteur et aux reçus de `crates/bridget-daemon/src/artifact_fetch.rs`, sans réseau depuis le renderer ou l'iframe.
- [x] T032 [US5] Ajouter les erreurs de contenu non restituable et les actions de récupération dans `crates/bridget-daemon/assets/ui/artifact-sandbox-host.js`, sans placeholder muet.
- [x] T033 [US5] Écrire les tests de non-exposition de cookies, secrets, IPC et contenu Browser aux artefacts/agents dans `apps/bridget-desktop/src-tauri/tests/browser_security_test.rs`.
- [x] T034 [US5] Écrire les tests de provenance, source indisponible et restauration par Bridget dans `crates/bridget-daemon/tests/artifact_lifecycle_test.rs`.

## Phase 7 - Finition, preuve et documentation

- [x] T035 Vérifier les capabilities finales et leur absence de privilège dans `apps/bridget-desktop/src-tauri/capabilities/` avec un test négatif par label de WebView.
- [x] T036 Vérifier les licences, attribution et limites de réutilisation des contrôles T3 dans `docs/decisions/023-artefacts-structures-et-sandbox.md`, sans importer de code T3 inutile.
- [x] T037 Jouer la recette `specs/083-artefacts-html-sandbox/quickstart.md` sur macOS et instance distante, puis consigner les résultats dans `specs/083-artefacts-html-sandbox/implementation.md`.
- [x] T038 Documenter l'exclusion explicite des actions DOM et d'automatisation agent dans `docs/decisions/023-artefacts-structures-et-sandbox.md` afin qu'aucune évolution ne les introduise par défaut.
- [x] T039 Raccorder les agents Codex gérés au serveur MCP Bridget déclaré statiquement avant `app-server` dans `crates/bridget-daemon/src/wrapper.rs`, puis vérifier que le catalogue fournisseur contient `bridget_publish_artifact` et son contrat `kind: html`.

## Opportunités de parallélisme

- Après T001 : T005, T006 et T007 sont indépendantes.
- Après T008 : les routes T011 et le host T009/T010 sont séparables.
- Après T020 : la disposition T023/T024 et l'effacement T022 progressent en parallèle.
- Après SPEC-082 : les projections T026/T027 peuvent avancer en parallèle avec les détails de sécurité T030.

## Vérification de format

Les 38 tâches sont toutes des checkboxes, portent un identifiant séquentiel et
un chemin cible. Les tâches de récits portent un label US. T003, T035 et T038
limitent explicitement la dette future : registre unique, preuves de permissions
et exclusion persistante de l'automatisation agent.
