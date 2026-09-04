# Journal d'implémentation — SPEC-088 Droits lisibles et vérifiables

## Métadonnées
- **Spec** : 088-droits
- **Branche** : `088-droits` (depuis `origin/main` `a931a8ac`)
- **Démarré** : 2026-09-03 13:58 CEST
- **Terminé** : En cours (voir statut en fin de fichier)
- **Méthode** : édition locale, `rsync` vers `cartae.app:/home/moi/bridget-referent/.worktrees/088-droits`, compilation et tests avec `CARGO_TARGET_DIR=/home/moi/bridget-referent/bridget/target`. Aucun commit automatique : le diff est à relire par le référent.

## Contre-revue adverse
- Plan : Jim (Codex), verdict BLOCKED à 14:16, sept objections vérifiées et toutes retenues (`adversarial-review-jim.md`). Conception révisée : état dans `control_state`, `Option<bool>` sans défaut permissif, signalement conditionné à la fin en échec, tentative de test corrélée, geste local réservé au rendu local.
- Implémentation : Jim, verdict BLOCKED lu à 21:03 (`adversarial-review-jim-2.md`) ; deux P0 confirmés et corrigés (migration de `control_events`, résolveur de test qui concluait avant l'acte refusal), cinq points secondaires retenus, un désaccord documenté (falsifiabilité du signalement, borne ADR-011). Gates rejouées après corrections : transport 248 ×2, daemon ciblé 39, Maicie 91/1 (rouge antérieur), Node 142.

## Progression

### T001 feature Gherkin — ✅ `tests/features/088-droits.feature`
### T002 trame — ✅ `protocol.rs` : `AgentPosture`, `ControlStateFrame { agent_posture: Option, auto_reassignment: Option }`, `ControlStateSet` étendu ; tous les initialiseurs corrigés (`--no-run` workspace : seules les deux erreurs préexistantes `mcp_injection_smoke_test`, `idempotency_crash_test` restent).
### T003 `refusals.rs` — ✅ 5 tests (motifs, faux positifs, borne, couches).
### T004 acte `refusal` — ✅ `JournalUpdateKind::Refusal`, validation de charge (`layer` fermée, `raw` ≤ 512), `JOURNAL_ACT_KINDS` de la vue aligné (oracle de parité vert).
### T005 profils — ✅ `control_settings.rs` : `RightsProfile`, `PROFILE_MATRIX`, `profile_for` (Custom si divergence ou valeur inconnue) ; 2 tests. Le fichier `server-rights.json` de la première conception a été retiré après la contre-revue.
### T006 détection Codex — ✅ (b, c, d) fin de commande journalisée (`state`, `exit_code`, `output_tail`, `item_id`), lignes reconnues retenues par item, acte `refusal` seulement à la fin en ÉCHEC du même item. Témoins : ligne+échec ⇒ 1 ; ligne+succès ⇒ 0 ; échec sans ligne ⇒ 0 ; ligne seule ⇒ 0 (`spec_088_signalement_de_sandbox_exige_la_ligne_et_l_echec_du_meme_item`). **(a) MESURÉ le 2026-09-04 04:14 UTC sur Jim (Codex gpt-5.6-terra, app-server)** : `item/completed` type `commandExecution` porte `status: completed`, `exitCode: 0`, `aggregatedOutput` avec le jeton ; journalisé par le wrapper en `kind: command, detail: item/completed, exit_code: 0, output_tail, item_id`. Codex enveloppe la commande dans `/bin/bash -lc "…"` avec échappement : `normalize_command_text` dénude cette enveloppe avant la comparaison exacte (test avec l'échantillon réel). Mutant : non rejoué formellement ; le témoin « ligne + succès ⇒ 0 » tombe si le gating sur l'échec est retiré (par construction du test).
### T007 `renderRefusal` — ✅ unique ; `local_toggle` honoré seulement avec `allowLocalToggle` ; `normalizeRefusal` retire tout geste local d'une source non locale ; essai adverse `spec_088_un_refus_distant_ne_porte_jamais_de_geste_local`.
### T008 carte de lien — ✅ « Ouverture au clic désactivée · Sécurité du contenu › Liens externes », geste « Autoriser les liens », re-rendu sans rechargement, refus sans événement fiable, renvoi Desktop.
### T009 acte `refusal` rendu — ✅ carte attribuée à Bridget avec « Droits › Shell » ; champs conservés à la projection (`layer`, `raw`, `posture`, `provider`, `gesture`).
### T010 refus existants — ✅ `controlRefusalFromCode` (pause, plafond, principal humain, capacité, origine humaine) rendus par `renderRefusal` dans le bandeau ; `rg "Bloqué par" app.js` ne trouve plus que l'assertion négative du test.
### T011 gate story 1 — ⚠️ transport 248 passés ; Node 142 passés ; quickstart §1-§2 sur le serveur : à jouer après mise en service (T026).
### T012 routes — ✅ `GET /v1/control/rights` (état, profil dérivé, lignes serveur demandé/réel/mécanisme/stockage, dogfooding, runtime, tentatives résolues), `POST /v1/control/rights/apply` (validation profil/matrice, UN `ControlStateSet`, `paused` jamais transmis, 403 journalisé). Tests de route : non écrits (le relais n'a pas de banc de routes isolé dans ce lot ; la validation est couverte par les tests de `referent_control::set` et par le quickstart §3).
### T013 colonnes — ✅ migration (neuve ⇒ discovery/0, existante ⇒ complete/1, idempotente), `rights_set`, `set` posture seule ⇒ génération +1, rejeu idempotent, `NothingToChange`, pause jamais touchée : `spec_088_base_neuve_est_prudente_et_base_existante_garde_son_comportement`, `spec_088_droits_journalises_sous_la_meme_generation`.
### T014 Maicie — ✅ `ControlStateWire.auto_reassignment: Option<bool>`, `ControlSnapshot::Read { auto_reassignment }`, garde unique : `Reassignment` différée motif `droits` si `None` ou `Some(false)`, la pause prime ; test `spec_088_reassignation_differee_sauf_droit_explicite`. Aucune condition parallèle dans `reduire_reassignation` (l'appelant `store.rs:1165` consulte déjà la garde).
### T015 posture au lancement — ✅ `resolve_spawn_agent_type_for_posture` : découverte ⇒ `project-discovery-<type>`, type sans définition ⇒ `SpawnRejected UnsupportedCapability { capability: "posture_decouverte" }`, jamais de repli ; test `spec_088_posture_decouverte_choisit_la_definition_ou_refuse`.
### T016-T018 page Droits — ✅ entrée « Droits », trois blocs, phrases, profils, Personnalisé, lignes locales depuis `localStorage`, mention « Réglage local à ce navigateur », mode expert (`bridget.rights.expert.v1`), « Demandé / Réel », clés locales envoyées par le serveur ignorées et signalées ; test `spec_088_page_droits_projette_profils_lignes_locales_et_serveur`.
### T019 gate story 2 — ⏳ voir « Gates » ci-dessous.
### T020 tentatives — ✅ `RightsTestAttempt`, `resolve_rights_test` pure (pending ; passed exige commande exacte + fin réussie + jeton ; refus exige le même item en échec + acte refusal ; curl ⇒ injoignable ; 120 s ⇒ inconnu) ; gestes fermés sans écriture ; document 0600 ; 6 tests.
### T021 route test — ✅ `POST /v1/control/rights/test` : tentative `pending`, envoi par `send_ui_message` (identité humaine), `message_id` conservé ; `bridget` avec dogfooding désactivé ⇒ `refused_bridget` sans envoi ; résolution à chaque `GET` par lecture du journal de l'agent filtré sur `message_id`. Admission « occupé » : non atomique par construction (décision après contre-revue) : la demande attend son tour et expire, jamais un refus.
### T022 Tester UI — ✅ bouton par ligne d'agent, résultat daté et attribué, ligne brute repliée, « Actualiser » en cours, « ancien » après 24 h.
### T023 gate story 3 — ⏳ quickstart §4 après mise en service.
### T024 documentation — ✅ `docs/regles-chantier.md`, ADR-028 (statut Proposé jusqu'à la mise en service), ce journal, `quickstart.md`.
### T025 validation opérateur — ⬜ à dérouler avec le référent.
### T026 mise en service — ⬜ après relecture du diff et commit par le référent.

## Gates (2026-09-03, worktree serveur recalé sur `origin/main` `bf93c3fe`, composition du lot avec le lot « pause = gel » fusionné entre-temps ; aucun conflit)

```text
cargo fmt --all -- --check                      : fmt propre
cargo test -p bridget-transport --lib           : test result: ok. 248 passed; 0 failed; 1 ignored (base : 242)
cargo test -p bridget-daemon --lib --features test-support
                                                : test result: FAILED. 717 passed; 144 failed; 7 ignored
  rouges vs base nue /tmp/baseu.txt (146 noms)  : nouveaux = 0 ; disparus = 2
cargo test -p bridget-daemon --lib ... -- referent_control rights_tests rights_test_tests spec_087 spec_088 control_
                                                : test result: ok. 38 passed; 0 failed
cargo test -p maicie (tous binaires)            : passed=91 failed=1 (rouge préexistant, même nom sur main nu :
                                                  store::coordination_transaction_tests::faute_apres_une_vraie_outbox_annule_decision_transition_et_notification)
cargo test -p maicie --test controle_referent_087 : test result: ok. 13 passed; 0 failed
cargo test -p maicie --lib spec_088             : test result: ok. 1 passed; 0 failed
cargo test --workspace --features test-support --no-run
                                                : seules les 2 erreurs préexistantes (mcp_injection_smoke_test, idempotency_crash_test, champ `name`)
cargo clippy (mes fichiers)                     : 3 avertissements corrigés (ui.rs champ redondant, control_settings.rs référence et clone) ;
                                                  restants hors lot : control.rs:35 impl Default (087), ui.rs:7040/7184 if imbriqués (antérieurs), daemon.rs:8671 doc orpheline (antérieure)
node --test app.js                              : ℹ pass 142 / ℹ fail 0 (base : 136)
```

## Convergence
Passage 1 : 2 tâches ajoutées (T027 hôte sur la carte de signalement, T028 édition d'une ligne serveur en mode expert), toutes deux implémentées ; passage 2 : CONVERGED (`tasks.md` inchangé). Écarts assumés : FR-012 (base existante), FR-023 (« occupé » rendu « En cours »), FR-003 (Codex seul).

## Audit court (Phase 7, manuel)
- Écarts spec/plan/code : les trois ci-dessus, documentés ; T006a non mesuré.
- Tests manquants : routes `/v1/control/rights*` sans banc de relais (couvertes par `referent_control::set` et le quickstart) ; mutant T006 non rejoué formellement.
- Risques constitutionnels : aucun ; ADR-028 posée ; minimalisme : le fichier de droits de la première conception a été supprimé (≈ 250 lignes évitées) ; potentiel minimalisme restant ≈ 0 ligne à comportement constant identifiée.
- Vertus LLM : chaque garde a un test d'effet ; la contre-revue a corrigé deux défauts de conception avant le code.
- Prochaines corrections recommandées : mesurer `item/completed` réel (T006a) ; jouer le quickstart complet (T025) ; mise en service (T026).

### T026 Mise en service — 2026-09-04 : ✅ fusion `165433d4` puis correctif d'enveloppe ; binaires `~/.local/lib/bridget/bridget-<sha>-<hash>/bridget` et `~/.local/lib/maicie/maicie-<sha>-<hash>/maicie` basculés par lien ; `bridget-ui.service` puis `bridget-daemon.service` redémarrés ; les deux agents gérés (Bridget, Jim) ont survécu au redémarrage (relance refusée « déjà actif ») ; `bridget who` sur le nouveau build, contrôle génération 12 avec historique recopié (migration de `control_events` vérifiée en production) ; `GET /v1/control/rights` : `agent_posture: complete`, `auto_reassignment: true`, profil `balanced` (base existante conservée) ; test réel « Shell » sur Jim : `passed`, sortie `BRIDGET-TEST-…` ; test réel « Internet » sur Jim : `passed`, sortie `200 BRIDGET-TEST-…` (build final `918c7f0a`).

## Statut
In Progress : 25/28 tâches cochées ; restent T011 et T019 (quickstart §1 à §3 à l'écran, dans le navigateur du référent) et T025 (validation opérateur). Fusionné dans `main` (`918c7f0a`) et en service.

## Non vérifié
- Routes `/v1/control/rights*` en conditions réelles (relais + daemon vivants) : quickstart §3-§4.
- Interruption/lancement réel en posture découverte sur le serveur (un agent relancé sous Prudent part en lecture seule) : à observer à la mise en service, et c'est la raison de la migration « base existante ⇒ complète ».
