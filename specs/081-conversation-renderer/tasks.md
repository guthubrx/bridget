# Tâches - SPEC-081 Conversation structurée et rendu technique sûr

**Entrées** : `spec.md`, `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md` et `reuse-audit.md`.

**Règle de livraison** : aucune tâche n'autorise commit, push, déploiement ou redémarrage automatique. Les preuves sont exécutées avant une demande de livraison.

## Phase 1 - Préparation et garde-fous communs

**But** : fixer fixtures, licences et contrat de test avant de modifier le fil.

- [X] T001 Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les fixtures Node de tours multi-segments, Markdown hostile, tableau, fence, lien, chemin et image, avec identités stables et résultats attendus.
- [X] T002 [P] Vérifier puis documenter dans `crates/bridget-daemon/assets/ui/vendor/NOTICE.md`, `crates/bridget-daemon/assets/ui/vendor/SHA256SUMS` et `specs/081-conversation-renderer/implementation.md` la version, licence, hash et procédure de repli du colorateur choisi.
- [X] T003 [P] Ajouter dans `crates/bridget-daemon/assets/ui/vendor/NOTICE.md` l'avis MIT complet de T3 Tools Inc. et dans `specs/081-conversation-renderer/implementation.md` la table des helpers T3 réellement adaptés, sans attribuer de code non repris.
- [X] T004 Vérifier le point de départ avec `node crates/bridget-daemon/assets/ui/app.js`, `cargo test -p bridget-daemon ui`, les tests Desktop applicables et `git diff --check`, puis consigner seulement les résultats réels dans `specs/081-conversation-renderer/implementation.md`.

---

## Phase 2 - Fondations transverses

**But** : disposer de fonctions pures de tours, références et préférences que les récits utilisateur consomment sans réécrire les autorités existantes.

- [X] T005 Ajouter et tester dans `crates/bridget-daemon/assets/ui/app.js` `normalizeContentSecurityPreferences`, lecture, écriture, migration et défaut sûr de `ContentSecurityPreferencesV1` ; une valeur absente, corrompue ou inconnue doit rendre les trois autorisations à `false`.
- [X] T006 Ajouter et tester dans `crates/bridget-daemon/assets/ui/app.js` le classificateur pur de destination `external_link`, `project_file`, `remote_image` ou `blocked`, en refusant `javascript:`, `data:`, `file:`, SVG, URL relative ambiguë et entrée surdimensionnée sans requête réseau.
- [X] T007 Ajouter et tester dans `crates/bridget-daemon/assets/ui/app.js` `deriveConversationTurns` au-dessus de `projectTimeline`, en préservant ordre, état attesté, actes, erreurs, rondes et dédoublonnage des messages humains.

**Checkpoint** : les données de rendu sont stables, sûres et testées. Les récits utilisateur peuvent démarrer sans nouveau protocole agent.

---

## Phase 3 - US1 Lire un tour complet sans reconstruire mentalement le fil (P1)

**Objectif** : une demande, son travail et son résultat forment un tour lisible sans masquer les preuves existantes.

**Test indépendant** : ouvrir une fixture de cinq tours, erreurs et rondes, puis identifier chaque résultat sans développer tous les détails.

- [X] T008 [P] [US1] Écrire dans `crates/bridget-daemon/assets/ui/app.js` les assertions Node sur un tour terminé, ouvert, échoué, interrompu et inter-agent, dont la réponse humaine n'apparaît qu'une fois.
- [X] T009 [US1] Modifier `renderThread` dans `crates/bridget-daemon/assets/ui/app.js` pour consommer `deriveConversationTurns`, conserver les identifiants DOM de tours inchangés et éviter de remonter le lecteur lorsqu'une entrée non lue arrive.
- [X] T010 [US1] Modifier `crates/bridget-daemon/assets/ui/theme.css` pour présenter la demande humaine dans une bulle compacte à droite et la réponse agent comme document à gauche, avec les fonds et gris Bridget existants.
- [X] T011 [US1] Adapter `renderActivityBatch`, `renderWork` et le rendu d'erreur dans `crates/bridget-daemon/assets/ui/app.js` afin d'offrir un résumé visible et des détails repliables, focusables et associés au tour, sans présenter une erreur comme une réponse.
- [X] T012 [US1] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` puis styliser dans `crates/bridget-daemon/assets/ui/theme.css` l'indicateur de nouvelles sorties et l'action explicite de retour au direct, avec conservation de l'ancre quand un détail change de hauteur.
- [X] T013 [US1] Exécuter les tests Node de `crates/bridget-daemon/assets/ui/app.js` pour les fixtures de tours et documenter les limites de défilement réellement mesurées dans `specs/081-conversation-renderer/implementation.md`.

**Checkpoint** : US1 est lisible et testable sans coloration, fichier ou image.

---

## Phase 4 - US2 Lire et réutiliser un contenu technique (P1)

**Objectif** : Markdown GFM, code et tableaux deviennent propres et copiables, tout en restant du contenu passif.

**Test indépendant** : consulter une fixture avec listes, citation, appelout, tableau et fences de plusieurs langages, puis copier code et tableau.

- [X] T014 [P] [US2] Adapter avec provenance MIT dans `crates/bridget-daemon/assets/ui/app.js` l'extraction du langage et du titre de fence issue de `apps/web/src/components/ChatMarkdown.tsx` de T3, avec tests pour métadonnées, fichier et langage inconnu.
- [X] T015 [P] [US2] Ajouter le colorateur local validé à `crates/bridget-daemon/assets/ui/vendor/`, le référencer depuis `crates/bridget-daemon/assets/ui/index.html` et prouver son chargement conditionnel, son hash et le repli texte brut dans `crates/bridget-daemon/assets/ui/app.js`.
- [X] T016 [US2] Remplacer le rendu brut des fences dans `crates/bridget-daemon/assets/ui/app.js` par un composant de bloc code avec titre ou langage, coloration thémée, bouton Copier, état de réussite ou échec accessible et copie exacte du texte source.
- [X] T017 [US2] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` le retour à la ligne local par bloc de code, indépendant de la préférence globale, et couvrir sélection, texte long et grammaire indisponible.
- [X] T018 [US2] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` la sérialisation de tableau Markdown et CSV, les boutons de copie et leurs tests, sans utiliser `innerHTML` non assaini ni perdre les cellules.
- [X] T019 [US2] Modifier `crates/bridget-daemon/assets/ui/theme.css` pour les blocs, contrôles, tableaux défilants, citations, alertes textuelles et états de copie dans les thèmes clair et sombre, en réutilisant les variables de police Bridget.
- [X] T020 [US2] Tester dans `crates/bridget-daemon/assets/ui/app.js` que contenu actif, événements HTML et attributs de lien restent refusés pendant le rendu enrichi, puis exécuter le programme Node.

**Checkpoint** : US2 est complet avec code, copie et tables, sans ouvrir aucun lien, fichier ou image.

---

## Phase 5 - US3 Maîtriser l'affichage des liens, fichiers et images (P1)

**Objectif** : l'opérateur contrôle localement chaque type de contenu, avec valeurs par défaut sûres et persistance application réelle sur macOS.

**Test indépendant** : avec les réglages désactivés aucun contenu actif ne se charge ; avec un réglage activé, seule sa capacité autorisée apparaît après geste explicite et sans mutation serveur.

- [X] T021 [P] [US3] Étendre et tester `apps/bridget-desktop/src-tauri/src/preferences_store.rs` vers le document V2 incluant `content_security`, migration V1 valide, reset sûr, permissions 0600 et choix actif de l'opérateur existant sans changer le défaut d'une nouvelle installation.
- [X] T022 [P] [US3] Étendre les tests de `apps/bridget-desktop/src-tauri/tests/desktop_commands.rs` et `apps/bridget-desktop/src-tauri/tests/secrets_and_diagnostics.rs` pour lecture, écriture, corruption et absence de profil, jeton ou requête tunnel dans les préférences de contenu.
- [X] T023 [US3] Ajouter dans `apps/bridget-desktop/ui/index.html`, `apps/bridget-desktop/ui/app.js` et `apps/bridget-desktop/ui/desktop.css` les trois contrôles de sécurité locaux, leur explication et leur état persistant, sans modifier les réglages d'un serveur.
- [X] T024 [US3] Modifier `apps/bridget-desktop/src-tauri/src/lib.rs` pour injecter l'instantané non modifiable de contenu avant les scripts du panneau relayé, diffuser une mise à jour après sauvegarde et intercepter uniquement `bridget-open:` afin d'ouvrir au navigateur système une URL HTTPS revalidée après geste utilisateur, sans donner de capability Tauri d'écriture au label `panel-*`.
- [X] T025 [US3] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` la consommation de l'instantané Desktop, le fallback `localStorage`, l'événement de mise à jour et les tests prouvant qu'un panneau ne peut pas élargir seul ses autorisations.
- [X] T026 [US3] Modifier `renderMessageMarkdown` et l'enrichissement DOM de `crates/bridget-daemon/assets/ui/app.js` pour rendre un état bloqué explicite si une préférence est inactive, puis un lien HTTPS ou une image HTTPS différée seulement si la préférence correspondante est active, avec `rel=noopener noreferrer`, `referrerpolicy=no-referrer` et une activation de lien exclusivement issue d'un clic utilisateur fiable.
- [X] T027 [US3] Ajouter et tester dans `crates/bridget-daemon/src/ui.rs` la route `GET /v1/content/file-preview` conforme à `contracts/file-preview-v1.md`, réutilisant `ProjectRootPolicy`, canonicalisation, plafond, types sûrs, token et refus bornés.
- [X] T028 [US3] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` le clic explicite d'aperçu de chemin de projet, les états chargement, refus et succès et l'affichage texte ou image raster, sans shell, écriture, `file:`, navigation automatique ni chemin racine absolu divulgué.
- [X] T029 [US3] Ajouter dans `crates/bridget-daemon/assets/ui/theme.css` les cartes de contenu bloqué, puces de lien, fichier et image, aperçus bornés et états clavier dans les deux thèmes.
- [X] T030 [US3] Exécuter les suites Node, `cargo test -p bridget-daemon ui` et les tests Tauri ciblés de cette story, puis consigner dans `specs/081-conversation-renderer/implementation.md` la preuve que les trois préférences restent locales.

**Checkpoint** : US3 est complet lorsque nouveau ou reset est désactivé, le profil actuel peut conserver son opt-in local, et aucun contenu d'agent ne possède de permission native ou d'exécution.

---

## Phase 6 - US4 Retrouver une interaction ancienne sans perturber la lecture (P2)

**Objectif** : consulter l'historique et revenir au direct volontairement, même pendant des sorties en cours.

**Test indépendant** : remonter dans une fixture de vingt tours, injecter une activité et vérifier que l'ancre ne bouge pas avant l'action de retour au direct.

- [X] T031 [P] [US4] Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les fixtures et assertions Node sur ancre de lecture, nœuds de tour stables, messages non lus et conservation du brouillon sur ré-affichage.
- [X] T032 [US4] Modifier la stratégie de `renderThread` dans `crates/bridget-daemon/assets/ui/app.js` pour mettre à jour seulement les tours changés ou restaurer précisément l'ancre de lecture, plutôt que réinitialiser le fil entier lors de toute sortie.
- [X] T033 [US4] Ajouter dans `crates/bridget-daemon/assets/ui/theme.css` les repères de tour et l'action de retour au direct visibles mais discrets, y compris sous taille de police élevée et réduction de mouvement.
- [ ] T034 [US4] Vérifier le scénario vingt tours de `specs/081-conversation-renderer/quickstart.md`, puis consigner la mesure ou limite factuelle dans `specs/081-conversation-renderer/evidence/validation.md`.

**Checkpoint** : US4 préserve la lecture d'historique sans casser les stories P1 déjà validées.

---

## Phase 7 - Finition, conformité et preuves croisées

**But** : éliminer le code remplacé, vérifier contrats et produire des preuves maintenables.

- [X] T035 Retirer ou simplifier dans `crates/bridget-daemon/assets/ui/app.js` et `crates/bridget-daemon/assets/ui/theme.css` les chemins de rendu devenus inatteignables, puis prouver par les tests qu'aucune bulle doublon ni surface active ne subsiste.
- [X] T036 [P] Relire `crates/bridget-daemon/assets/ui/vendor/NOTICE.md`, `LICENSE.*` et `SHA256SUMS` pour vérifier les avis T3 et colorateur, puis documenter le résultat dans `specs/081-conversation-renderer/implementation.md`.
- [X] T037 [P] Exécuter `cargo fmt --check`, le programme Node, les tests Rust et Tauri ciblés, ainsi que `git diff --check`, puis inscrire chaque commande, succès ou échec réel dans `specs/081-conversation-renderer/evidence/validation.md`.
- [ ] T038 Jouer entièrement `specs/081-conversation-renderer/quickstart.md` sur Bridget Desktop et un serveur approuvé, avec thèmes clair et sombre, puis mettre à jour `spec.md`, `tasks.md`, `implementation.md` et `audit.md` sans cocher une preuve non observée.
- [X] T039 Tenter la contre-revue externe du diff via Bridget si le daemon est joignable ; sinon consigner objectivement son indisponibilité dans `specs/081-conversation-renderer/reviews/adversarial-code.md`. En cas de verdict, corriger les constats prouvés avant toute livraison.

---

## Dépendances et ordre d'exécution

```text
T001-T004
    |
T005-T007
    |
    +--> US1 : T008-T013
    +--> US2 : T014-T020
    +--> US3 : T021-T030
    |
US1 + US2 + US3
    |
US4 : T031-T034
    |
T035-T039
```

### Dépendances par récit

- **US1** dépend de T007.
- **US2** dépend de T001 à T003 et du Markdown assaini existant.
- **US3** dépend de T005 et T006. T027 précède T028.
- **US4** dépend de la structure stable obtenue par US1.
- Les trois stories P1 sont toutes requises par cette SPEC : le checkpoint US1 n'est pas une permission de livrer une version tronquée.

## Opportunités de parallélisme

- T002 et T003 peuvent avancer indépendamment des fixtures.
- Après T005-T007, US1, US2 et le travail native de T021-T024 peuvent avancer sur des fichiers distincts, sous réserve de synchroniser le contrat de préférences avant T025.
- T021 et T022 sont parallèles ; T014 et T015 sont parallèles.
- T036 et T037 sont parallèles après les implémentations.

## Stratégie de réalisation

1. Mettre en place transformations pures et tests qui empêchent toute régression de remise ou d'assainissement.
2. Livrer le fil lisible, puis le Markdown technique passif.
3. Étendre ensuite préférences locales et contenus enrichis avec bornes relayées, jamais dans l'ordre inverse.
4. Stabiliser les gros historiques seulement une fois la structure de tour réelle rendue.
5. Terminer par preuves d'intégration, licence et contre-revue externe.

## Validation Article XX

- T005, T007, T021, T024 et T027 étendent des autorités existantes au lieu de créer stores, listes de chemins ou permissions parallèles.
- T014 et T003 rendent le réemploi MIT modifiable et vérifiable, au lieu de laisser une copie sans provenance.
- T032 porte une responsabilité de stabilité mesurable : activité live, détail replié et retour au direct.
- Aucune tâche n'ajoute terminal, éditeur, shell, proxy web ou capacité de mutation attachée à un contenu conversationnel.
