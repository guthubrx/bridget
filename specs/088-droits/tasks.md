# Tasks: Droits lisibles et vérifiables

**Input**: `specs/088-droits/{spec,plan,research,data-model}.md`, `contracts/rights-v1.md`
**Tests**: demandés par la spec (gardes avec mutants, contrôles positifs). Chaque tâche cite son résultat observable.

## Phase 1: Setup

- [x] T001 Créer `tests/features/088-droits.feature` avec les scénarios des trois stories et des cas limites de `spec.md` (Gherkin français)
- [x] T002 [P] Ajouter à `crates/bridget-transport/src/protocol.rs` `AgentPosture` fermé, `ControlStateFrame { agent_posture: Option<AgentPosture>, auto_reassignment: Option<bool> }` (`serde(default)`, `None` = inconnu) et `ControlStateSet { agent_posture, auto_reassignment }` optionnels ; jouer `cargo test --workspace --features test-support --no-run` et corriger CHAQUE initialiseur exhaustif ; résultat : seules les deux erreurs préexistantes connues restent

## Phase 2: Foundational

- [x] T003 Créer `crates/bridget-transport/src/refusals.rs` : `RefusalLayer` fermé (8 valeurs de R7), `Refusal { layer, prevented, gesture, raw, at, attributed_to }`, `recognize_sandbox_refusal(line: &str) -> Option<Refusal>` sur les motifs de `contracts/rights-v1.md` ; déclarer le module dans `lib.rs` ; tests : `bwrap: … Operation not permitted` ⇒ `provider_sandbox`, `bwrap: … Permission denied` ⇒ idem, `sandbox-exec: deny` ⇒ idem, ligne vide ⇒ `None`, ligne `ls: cannot access` ⇒ `None` (contrôle positif de l'absence)
- [x] T004 Ajouter la variante `Refusal` à `JournalUpdateKind` dans `crates/bridget-transport/src/act_kind.rs` (`as_str = "refusal"`, `is_act = true`, validation de charge dans `validate_journal_write` : `layer`, `raw` ≤ 512 caractères, `provider`, `posture`) ; tests : `parse("refusal")`, charge sans `layer` refusée
- [x] T005 `crates/bridget-daemon/src/control_settings.rs` : `AgentPosture`, `RightsProfile`, `ProfileValues`, `PROFILE_MATRIX` (R6), `profile_values`, `profile_for` (`Custom` si divergence) ; AUCUN fichier de droits (révision contre-revue) ; tests : matrice, `Custom`

## Phase 3: User Story 1 - Refus explicites (P1)

- [x] T006 [US1] `crates/bridget-transport/src/codex_app_server.rs` : (a) mesurer sur le serveur la forme réelle de `item/completed` type `commandExecution` (trace `BRIDGET_CODEX_TRACE` d'un tour réel) et la consigner dans `implementation.md` ; (b) journaliser la fin de commande (`state`, `exit_code`, `output_tail` ≤ 512, `item_id`) ; (c) mémoriser par `item_id` les lignes reconnues dans `outputDelta` ; (d) à la fin en ÉCHEC d'un item ayant une ligne reconnue, écrire UN acte `refusal` (`evidence: output_and_exit`, `provider`, `posture`, `raw`, `item_id`) ; témoins avec la fixture : ligne + échec ⇒ 1 acte ; ligne + succès ⇒ 0 ; échec sans ligne ⇒ 0 ; deux lignes même item ⇒ 1 ; mutant : gating retiré ⇒ le témoin « ligne + succès » tombe
- [x] T007 [P] [US1] `app.js` : `renderRefusal(documentRef, refusal, options)` unique ; `local_toggle` honoré seulement si `options.allowLocalToggle === true` ; `normalizeRefusal(value, { localOrigin })` retire tout geste local d'une source non locale ; tests Node : 8 couches, `unknown`, sans geste, essai adverse : un refus distant portant `local_toggle` ne rend aucun bouton et ne touche pas `bridget.content-security.v1`
- [x] T008 [US1] `app.js` `renderContentReferences` : remplacer « Bloqué par vos réglages locaux. » par `renderRefusal` avec `layer: browser_content`, `prevented` selon le type, geste `local_toggle` qui appelle `writeContentSecurityPreferences` puis re-rend le fil ; refus désactivé quand `desktopManaged` (geste ⇒ renvoi Desktop) ; tests Node : clic `isTrusted` active la préférence et la carte montre le bouton d'ouverture ; événement non `isTrusted` ⇒ rien ; Desktop ⇒ texte de renvoi
- [x] T009 [US1] `app.js` rendu du journal : un acte `refusal` devient une carte `renderRefusal` attribuée à Bridget, `prevented: "shell"`, geste `rights_line: "shell"` ; tests Node : acte ⇒ carte ; acte avec `layer` inconnu ⇒ « couche inconnue »
- [x] T010 [US1] `app.js` refus existants : `ControlStateRejected` (`human_principal_required`, `generation_mismatch`, `budget_out_of_range`), `capability_not_negotiated`, `control_paused`, `budget_reached` passent par `renderRefusal` avec la couche `referent_control` ou `bridget_policy` ; tests Node par cas ; résultat : plus aucun texte de refus construit ailleurs (`rg "Bloqué par" app.js` vide)
- [ ] T011 [US1] Gate story 1 : `cargo test -p bridget-transport` (compte), `node --test app.js` (compte ≥ 136 + nouveaux), `quickstart.md` §1 et §2 joués sur le serveur avec un Codex en posture découverte ; consigner dans `implementation.md`

## Phase 4: User Story 2 - Page Droits (P2)

- [x] T012 [US2] `crates/bridget-daemon/src/ui.rs` : `GET /v1/control/rights` (projection : état de contrôle, profil dérivé, lignes serveur `requested/actual/mechanism/storage`, dogfooding, `runtime_capability`, dernières tentatives de test résolues) ; `POST /v1/control/rights/apply` : validation profil/matrice, UN `ControlStateSet` (posture, réassignation, plafond ; jamais `paused`) ; tests : sans principal humain ⇒ 403 ET ligne consignée dans le journal du relais ; génération décalée ⇒ 409 ; profil nommé aux valeurs étrangères ⇒ 400 ; appliquer un profil pendant la pause ne modifie jamais `paused`
- [x] T013 [US2] `crates/bridget-daemon/src/referent_control.rs` + `daemon.rs` : colonnes `agent_posture`, `auto_reassignment` (migration : neuve ⇒ `discovery`/0, existante ⇒ `complete`/1), `ControlMutation` étendue, `ControlEventKind::RightsSet`, `read` renvoie `Some(...)`, bras `ControlStateSet` transmet les champs ; tests : migration des deux cas, `set` posture seule ⇒ génération +1 et événement `rights_set`, rejeu idempotent, rien à changer ⇒ `NothingToChange`
- [x] T014 [US2] Maicie : `plugins/maicie/src/bridget_client.rs` `ControlStateWire.auto_reassignment: Option<bool>` ; `control.rs` `ControlSnapshot::Read { auto_reassignment: Option<bool> }` et `AutonomousEffect::Reassignment` dans `admit_autonomous_effect` (`None` ou `Some(false)` ⇒ `Deferred { motif: "droits" }`, après la pause) ; l'appelant de `reduire_reassignation` consulte la garde et consigne `Differee { motif: "droits" }` ; test d'effet dans `plugins/maicie/tests/controle_referent_087.rs` ; mutant : garde retirée ⇒ rouge
- [x] T015 [US2] `crates/bridget-daemon/src/daemon.rs` (lancement géré) : lire `agent_posture` dans `control_state` ; `discovery` ⇒ définition `PROJECT_DISCOVERY_AGENT_PREFIX + type` ; type sans définition découverte ⇒ refus explicite du lancement avec motif consigné, pas de repli silencieux ; test : posture découverte ⇒ arguments contiennent `--sandbox read-only` ; complète ⇒ contournement présent
- [x] T016 [US2] `app.js` + `index.html` : entrée « Droits » dans `CONTROL_CENTER_NAVIGATION` (mots-clés : droits, permissions, profil, sandbox, liens), page à trois blocs, `RIGHTS_LINES` constante fermée (11 lignes : clé, bloc, phrase par valeur, mécanisme, stockage, local ou serveur), sélecteur de profil, `Personnalisé`, lignes locales composées depuis `localStorage` + `bridget.rights.local-profile.v1`, mention « Réglage local à ce navigateur » ; tests Node : Prudent/Équilibré/Confiant ⇒ valeurs de la matrice ; réponse serveur avec profil `balanced` mais préférences locales fermées ⇒ `Personnalisé` ; une réponse serveur contenant des clés locales est ignorée et consignée une fois
- [x] T017 [US2] `app.js` mode expert de la page (préférence `bridget.rights.expert.v1`) : chaque ligne dépliable montre mécanisme, stockage, valeur brute, posture commandée, et « Demandé / Réel » quand ils diffèrent ; modification d'une ligne seule ⇒ profil `Personnalisé` ; tests Node
- [x] T018 [US2] `theme.css` : styles de la page et des cartes de refus (fond, repli de la ligne brute), sans nouvelle dépendance
- [ ] T019 [US2] Gate story 2 : `cargo test -p bridget-daemon --lib --features test-support` avec comparaison des NOMS de rouges contre `/tmp/baseu.txt` (serveur), `cargo test -p maicie`, `node --test app.js`, `quickstart.md` §3 ; consigner

## Phase 5: User Story 3 - Tester (P3)

- [x] T020 [US3] `control_settings.rs` : `RightsTestAttempt`, `RightsTestOutcome` fermé, `test_gesture(line, token, project_file, system_checkout)`, lecture/écriture 0600 de `server-rights-tests.json`, `resolve_attempt(attempt, journal_records, now)` pure selon le contrat ; tests : commande démarrée sans fin ⇒ `pending` ; fin en échec après ligne reconnue ⇒ `refused_provider_sandbox` ; jeton dans un autre `message_id` ⇒ `pending` ; fin réussie avec jeton ⇒ `passed` ; 121 s sans fin ⇒ `unknown_expired` ; aucune commande n'écrit (`rg -n ">|tee|rm |mv " ` sur les gestes vide)
- [x] T021 [US3] `ui.rs` `POST /v1/control/rights/test` : `busy` si présence occupée à l'admission, `no_agent`, `refused_bridget` sans envoi ; sinon envoi par `post_ui_message`, `message_id` enregistré, réponse `pending` ; `GET /v1/control/rights` résout chaque tentative par lecture du journal filtré sur son `message_id` ; tests de route pour chaque issue
- [x] T022 [US3] `app.js` : bouton « Tester » sur les lignes du bloc 2, choix parmi les agents connectés et libres, résultat daté attribué avec ligne brute repliée, « ancien » au-delà de 24 h ; tests Node : chaque issue a son libellé ; résultat de 25 h ⇒ « ancien »
- [x] T023 [US3] Gate story 3 : gates complets + `quickstart.md` §4 sur le serveur avec un Codex découverte puis complet ; consigner les deux résultats bruts

## Phase 6: Polish

- [x] T024 Documentation : `docs/regles-chantier.md` (paragraphe Droits), `docs/decisions/028-…md` statut Accepté, `specs/088-droits/implementation.md` (journal par tâche, gates, mutants)
- [ ] T025 Validation opérateur : dérouler `quickstart.md` en entier avec le référent ; consigner
- [x] T026 Mise en service après fusion : compilation release sur le checkout principal, binaire versionné, bascule du lien, redémarrage `bridget-ui.service` puis `bridget-daemon.service` (une seule fois), relance des agents gérés par UUID, `bridget who` sans avertissement `identity_version`

## Révision après contre-revue (2026-09-03, Jim, BLOCKED)

T002, T005, T006, T007, T012, T013, T014, T015, T020, T021 réécrites : plus de fichier de droits, état dans `control_state` ; signalement de sandbox conditionné à la fin en échec ; tentative de test corrélée ; geste local réservé au rendu local. T002 et T007 sont réouvertes pour appliquer la révision (coché seulement après).

## Dependencies

- T002 et T003 avant tout le reste ; T004 avant T006 et T009 ; T005 et T013 avant T012 et T015.
- US1 (T006-T011) indépendante de US2 sauf le geste `rights_line` qui pointe vers une page qui n'existe qu'après T016 : le lien est inerte tant que la page n'est pas là, c'est accepté.
- US2 (T012-T019) avant US3 (T020-T023).
- T024-T026 en dernier.

## Parallel Execution

- T003, T004, T005 sur trois fichiers distincts.
- T007 pendant T006 ; T016 pendant T012-T015.

## Implementation Strategy

- MVP = Phase 1, Phase 2, US1 : les refus deviennent lisibles sans page nouvelle.
- Puis US2, puis US3.

## Article XX

- Réduit la charge future : T010 (un seul rendu de refus au lieu de quatre), T005 (matrice en constante, pas de moteur), T015 (refus explicite plutôt qu'un repli silencieux).
- Complexité à justifier : T021 (observation bornée du journal, 30 s, un agent), T013 (le daemon lit un fichier du relais ; même modèle que `project_root_policy_path`).

## Convergence

Passage 1, 2026-09-03 15:00 CEST. Lecture de `spec.md` FR par FR contre le code ; manques ci-dessous, ajoutés sans rien décocher.

- [x] T027 [US1] FR-003 : la carte de signalement de sandbox ne nommait pas le serveur ; `renderRefusal` affiche « sur <hôte> » à partir de l'hôte de l'agent sélectionné (`refusalFromAct(act, { host })`) ; test Node
- [x] T028 [US2] FR-014 : en mode expert, modifier une ligne serveur seule (posture sur la ligne Shell, réassignation, plafond) envoie `profile: custom` avec les valeurs courantes ; le profil affiché devient Personnalisé
- Écarts assumés, documentés dans `implementation.md` : FR-012 (base existante conserve son comportement, Prudent réservé à une base neuve) ; FR-023 (« occupé » rendu comme « En cours » puis « Inconnu », la demande attend son tour sans interrompre) ; FR-003 limité à Codex.
