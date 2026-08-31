# Tâches - SPEC-078 Profils d'agents et notifications

**Entrées**: `spec.md`, `research.md`, `plan.md`, `data-model.md`,
`contracts/agent-profiles-v1.md`, `quickstart.md`, `reuse-audit.md`.

**Principe de livraison**: chaque phase apporte un comportement attestable. Les
changements de snapshot viennent après les migrations et les clients ne
présentent jamais une mutation comme réussie avant le verdict du relais.

## Phase 1 - Préparation et garde-fous

**But**: rendre les contraintes inspectables avant toute modification de
persistence ou de transport.

- [x] T001 Ajouter dans `crates/bridget-daemon/src/store.rs` les tests de migration SPEC-078 couvrant base vide, imports flotte/annuaire/ledger, relance idempotente et absence de réécriture des tables de messages.
- [x] T002 [P] Ajouter dans `crates/bridget-daemon/src/ui.rs` les fixtures de contrat profil et attention v1 avec jeton requis, conflit de nom et projection sans secret.
- [x] T003 [P] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les tests purs de parsing de labels, rendu display name, état de consigne et déduplication d'activité.
- [x] T004 [P] Ajouter dans `apps/bridget-desktop/src-tauri/src/profile_store.rs` les tests de persistance du client_id, préférences par origine et reprise après changement de port.

## Phase 2 - Fondations bloquantes

**But**: créer l'identité durable, le contrat privé et le journal sémantique
sur lesquels toutes les stories s'appuient.

- [x] T005 Implémenter dans `crates/bridget-daemon/src/store.rs` les migrations SQLite transactionnelles `agent_identities`, `agent_routing_aliases`, `agent_profiles`, `agent_profile_labels` et leurs contraintes d'unicité.
- [x] T006 Implémenter dans `crates/bridget-daemon/src/store.rs` l'import idempotent des noms de `fleet.json`, annuaire vivant et ledger, avec alias actifs et historiques sans mutation des messages.
- [x] T007 Implémenter dans `crates/bridget-daemon/src/store.rs` les lectures et écritures atomiques de profil, validation fermée, version optimistic-lock et codes d'erreur sûrs.
- [x] T008 Implémenter dans `crates/bridget-daemon/src/store.rs` les tables et opérations idempotentes `attention_events`, préférences client et états seen/native_notified.
- [x] T009 Implémenter dans `crates/bridget-daemon/src/ui.rs` les DTO communs de projection `profile`, `attention` et les fallbacks tolérants pendant migration ou indisponibilité du store.
- [x] T010 Vérifier dans `crates/bridget-daemon/src/store.rs` et `crates/bridget-daemon/src/ui.rs` que `agent_id`, nom de routage et instructions ne figurent dans aucun libellé ou diagnostic public de ces nouveaux chemins.

**Point de contrôle**: migrations et tests store passent; une lecture sans profil
ne casse ni les messages ni le cycle de vie.

## Phase 3 - US1 Identité visible (P1)

**Objectif**: l'utilisateur renomme un agent dans le modèle utilisateur sans
jamais toucher au routage ou à l'historique.

**Test indépendant**: modifier un display name, consulter fil et liste dans Web
et Desktop, redémarrer le daemon puis constater persistance et absence d'ID.

- [x] T011 [US1] Ajouter dans `crates/bridget-daemon/src/ui.rs` les routes token-authenticated `GET` et `PATCH /v1/agent-profiles/{profile_ref}` selon `contracts/agent-profiles-v1.md`.
- [x] T012 [US1] Étendre dans `crates/bridget-daemon/src/ui.rs` les projections `/v1/snapshot` et `/v1/watch` avec display name et référence opaque non rendue, tout en gardant le fallback de compatibilité.
- [x] T013 [US1] Adapter dans `crates/bridget-daemon/src/ui.rs` les projections de conversations, recherches et erreurs pour qu'elles résolvent le display name au lieu de présenter un nom de routage.
- [x] T014 [US1] Adapter dans `crates/bridget-daemon/assets/ui/app.js` l'état d'agents, sélection, recherche, en-tête, messages et erreurs pour afficher exclusivement le display name sans mettre profile_ref dans le DOM.
- [x] T015 [US1] Ajouter dans `crates/bridget-daemon/src/ui.rs` et `crates/bridget-daemon/assets/ui/app.js` les tests de collision de nom, conflit de révision, redémarrage et non-exposition de l'identité interne.

## Phase 4 - US2 Labels et apparence partagés (P1)

**Objectif**: les rôles visuels et l'avatar deviennent des propriétés partagées
plutôt qu'un réglage fragile de navigateur.

**Test indépendant**: enregistrer deux labels et une bouille depuis un client,
puis les vérifier et rechercher depuis un autre client relié au même relais.

- [x] T016 [US2] Étendre dans `crates/bridget-daemon/src/store.rs` et `crates/bridget-daemon/src/ui.rs` la mutation de profil pour normaliser labels séparés, supprimer vides/doublons, borner les valeurs et retourner des pastilles ordonnées.
- [x] T017 [US2] Adapter dans `crates/bridget-daemon/assets/ui/app.js` les fonctions `createAgentAvatar`, recherche et rendu de liste/en-tête afin de consommer `profile.avatar` et les labels projetés, sans persister l'apparence par `name` dans localStorage.
- [x] T018 [US2] Modifier dans `crates/bridget-daemon/assets/ui/index.html` et `crates/bridget-daemon/assets/ui/theme.css` les primitives de pastilles et de picker pour rendre plusieurs labels distincts, accessibles et compatibles 200 %.
- [x] T019 [US2] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` et `crates/bridget-daemon/src/ui.rs` les tests de saisie virgule/Entrée, recherche par label, rendu identique de l'avatar et fallback après snapshot incomplet.

## Phase 5 - US3 Instructions individuelles sans persona (P1)

**Objectif**: la consigne d'un agent influence réellement son prochain contexte
fournisseur, mais Bridget ne ment jamais sur une session déjà active.

**Test indépendant**: changer la consigne d'un agent de chaque famille
disponible, lancer un travail puis observer une application attestée ou un état
en attente explicite.

- [x] T020 [US3] Ajouter dans `crates/bridget-daemon/src/store.rs` et `crates/bridget-daemon/src/ui.rs` l'état versionné d'application des instructions, les verdicts `applied`, `pending_restart`, `unsupported`, `failed` et la projection sans texte.
- [x] T021 [US3] Implémenter dans `crates/bridget-daemon/src/wrapper.rs` le chargement d'un snapshot d'instructions et son ordre stable sous invariants Bridget, mandat et permissions lors de chaque spawn ou reprise.
- [x] T022 [US3] Adapter dans crates/bridget-transport/src/claude_stream_json.rs le contexte privé en mémoire de Claude, GLM et DeepSeek au prochain spawn, sans texte de consigne dans les arguments de processus, AgentDefinition.args, agents.json, diagnostics ou journal Bridget.
- [x] T023 [P] [US3] Adapter dans crates/bridget-transport/src/codex_app_server.rs le contexte privé en mémoire au nouveau thread sans utiliser steer ni annoncer un changement actif fictif.
- [x] T024 [P] [US3] Adapter dans crates/bridget-transport/src/acp.rs le contexte privé en mémoire après session/new, sans détourner session/prompt d un travail actif.
- [x] T025 [US3] Ajouter dans crates/bridget-transport/src/claude_stream_json.rs, crates/bridget-transport/src/codex_app_server.rs et crates/bridget-transport/src/acp.rs les tests cross-provider de révision, reprise, absence de fuite et refus de faux succès.

## Phase 6 - US4 Notifications locales pertinentes (P1)

**Objectif**: chaque client décide de son attention, sur un journal fiable qui
exclut strictement les activités routinières.

**Test indépendant**: deux clients choisissent des types différents, puis une
attente humaine, une réussite, un échec, un outil et du streaming sont produits.

- [x] T026 [US4] Identifier dans `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/fleet.rs` et `crates/bridget-daemon/src/wrapper.rs` les seuls faits attestés qui produisent `human_input_needed`, `task_completed` ou `terminal_failure` et leurs occurrence keys.
- [x] T027 [US4] Câbler dans `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/fleet.rs` et `crates/bridget-daemon/src/store.rs` la création idempotente d'événements sans émission pour outils, commandes, chunks, retries ou statuts routiniers.
- [x] T028 [US4] Ajouter dans `crates/bridget-daemon/src/ui.rs` les routes attention de lecture, préférences par client et marquage seen/native_notified, avec token, pagination et verdict atomique.
- [x] T029 [US4] Adapter dans `crates/bridget-daemon/assets/ui/app.js` le client_id Web, les préférences par agent, le polling ou watch d'attention, le badge et les marqueurs de lecture sans dépendre de `pendingUiMessages`.
- [x] T030 [US4] Modifier dans `crates/bridget-daemon/assets/ui/index.html` et `crates/bridget-daemon/assets/ui/theme.css` le centre d'activité, son bouton à badge, la région ARIA polie et la navigation clavier sans déplacement de focus.
- [x] T031 [US4] Adapter dans `crates/bridget-daemon/assets/ui/app.js` la notification navigateur afin qu'elle respecte préférences, permission, arrière-plan et déduplication par occurrence, tout en conservant le clic vers le bon agent.
- [x] T032 [US4] Étendre dans `apps/bridget-desktop/src-tauri/Cargo.toml`, `apps/bridget-desktop/src-tauri/src/profile_store.rs`, `apps/bridget-desktop/src-tauri/src/lib.rs` et les capacités Tauri le client_id stable, le suivi relayé et la notification macOS sans capacité accordée à la WebView enfant.
- [x] T033 [US4] Ajouter dans `crates/bridget-daemon/src/ui.rs`, `crates/bridget-daemon/assets/ui/app.js` et `apps/bridget-desktop/src-tauri/src/lib.rs` les tests de deux clients, redémarrage, permission refusée, port Desktop variable, déduplication et zéro bruit d'outil.

## Phase 7 - US5 Panneau de profil (P2)

**Objectif**: un panneau latéral intégré permet le réglage sans dégrader le menu
contextuel de cycle de vie.

**Test indépendant**: ouvrir la bouille, modifier un profil, fermer et rouvrir
le panneau dans Web et Desktop, puis utiliser séparément les trois points.

- [x] T034 [US5] Généraliser dans `crates/bridget-daemon/assets/ui/index.html` et `crates/bridget-daemon/assets/ui/app.js` le panneau droit en états `peerExchange` et `agentProfile`, avec fermeture et focus restaurés.
- [x] T035 [US5] Implémenter dans `crates/bridget-daemon/assets/ui/app.js` le formulaire de profil confirmé par le serveur: display name, labels multi-entrée, instructions, statut d'application, apparence et préférences d'attention.
- [x] T036 [US5] Modifier dans `crates/bridget-daemon/assets/ui/theme.css` le panneau, états chargement/conflit, pastilles, picker et raccourcis afin de préserver le style minimaliste Bridget à 1280 x 720 et 200 %.
- [x] T037 [US5] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les tests de bouille vers profil, trois points vers menu SPEC-077, soumission atomique, clavier, erreurs de champ et absence d'ID rendu.

## Phase 8 - Finition et preuves transversales

- [x] T038 Vérifier dans `crates/bridget-daemon/src/store.rs`, `crates/bridget-daemon/src/ui.rs` et `crates/bridget-daemon/src/wrapper.rs` les recherches de fuites: aucun UUID, nom de routage ou texte d'instruction dans les réponses publiques, erreurs, logs et fixtures destinées à l'UI.
- [x] T039 Exécuter et documenter dans `specs/078-profils-agents/implementation.md` les suites Rust ciblées daemon/transport, les tests Node `crates/bridget-daemon/assets/ui/app.js`, les tests Desktop applicables et `git diff --check`.
- [ ] T040 Réaliser et documenter dans `specs/078-profils-agents/evidence/validation.md` les parcours Web/Desktop de `quickstart.md`, incluant fournisseurs disponibles, permission refusée, reconnexion et accessibilité.
- [x] T041 Mettre à jour dans `specs/078-profils-agents/spec.md`, `implementation.md` et `evidence/validation.md` le statut, la couverture des exigences et les limites assumées, sans commit ni déploiement implicite.

## Dépendances et ordre

```text
Fondations (T001-T010)
       ├── US1 identité (T011-T015)
       ├── US2 labels/avatar (T016-T019) après projection US1
       ├── US3 instructions (T020-T025) après store T005-T007
       └── US4 attention (T026-T033) après store T008-T009
                    └── US5 panneau (T034-T037) après US1, US2, US3, US4
                             └── Finition (T038-T041)
```

## Opportunités parallèles

- Après T005-T010, T021-T022 peuvent avancer en parallèle de T026-T028: les
  fichiers fournisseur et attention n'ont pas de conflit direct.
- Après T012, T016-T019 peuvent avancer en parallèle de T020-T025.
- T023 et T024 sont explicitement parallèles car ils touchent des transports
  distincts.
- La validation Desktop T032 peut commencer après le contrat attention T028,
  sans attendre le panneau visuel T034.

## Stratégie d'implémentation

Le premier incrément vérifiable est US1: identité, migration, projection et
nom affiché. US2 rend ensuite l'identité reconnaissable. US3 et US4 sont deux
capacités indépendantes mais doivent converger avant le panneau final. Aucun
jalon n'est une livraison implicite: SPEC-078 reste incomplète tant que T041
n'est pas coché et que les preuves Web/Desktop ne sont pas rassemblées.

## Validation de format et responsabilité future

- Les 41 tâches suivent le format checkbox, identifiant, labels et chemins.
- Les nouvelles tables et le plugin Desktop sont justifiés par des invariants
  testables; aucun wrapper sans comportement propre n'est prévu.
- Les tâches T010, T025, T033, T038 et T040 rendent visibles les limites de
  sécurité, de notification et de fournisseur qui seraient sinon cachées.
