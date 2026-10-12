# T039 — comparaison du passage complet155-r1

**Résultat du passage** :2259 PASS /15 FAIL /72 ignored ;94 cibles complètes,554,43 s,RC101.
**Référence T001** :180/180 noms exacts PASS, chacun une fois sur la bonne cible ;167 daemon/lib,9 transport/lib,4 Claude. Aucun manquant, doublon ou ignored reclassé.
**Échecs** :8 historiques présents,7 nouveaux ; aucun historique absent. Aucun succès global.

**État des sources** :214 hashes avant/après égaux. Passage AVANT le correctif T040 environmentId=local. Il ne valide pas les sources corrigées.
Transport pendant le run :aeedb4500ad540cb53007d14e2e2030d19bfec761cfb4a1f3b01cf468d3ab9b3

## Preuves

/Users/moi/.cache/bridget-delivery155.um4xccv4/workspace155-final-r1.log
SHA256 :decd7bf7246e62d12cb47a98caa6af9bf96627ef06a238e11350ba4eab8d9183
/Users/moi/.cache/bridget-delivery155.um4xccv4/workspace155-final-r1.json
SHA256 :2e77e91dce76b0819fe16a45dfee7c2b2baa36331509b7de371534834e811785
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/baseline155-index.json
SHA256 :d563d09a587da6279d9c0b279430548fff082d1c5e0cf1422783c552a18f20ab
/Users/moi/.cache/bridget-delivery152.kzq0fhzj/proofs152/workspace-152-r1-comparison.json
SHA256 :ef806aa33540aaee73eaa8d6d75814a59d0e556e53bbc9918b9e4de02effdc83
/Users/moi/.cache/bridget-delivery154.l50umk90/analyze-workspace154.py
SHA256 :54005eccf6799a5c65fcccafd36859124165e2ea29ad6a1fe595744edc09e271

## Sept nouveaux échecs — données observées

### exemples_skill_envoient_repondent_et_cloturent_sans_service_compagnon

Running tests/core_089_skill_test.rs
État du même nom au passage complet154 :ok
Actual 10 JSON publiés ; expected 9 bancs explicites.
Cause non établie ; aucun classement flaky.

Log lignes LF 2531–2542 :
```text

running 2 tests
test fixture::performance_daemon_worker ... ignored, sous-processus privé des bancs, jamais un daemon de production

thread 'exemples_skill_envoient_repondent_et_cloturent_sans_service_compagnon' (69814602) panicked at crates/bridget-daemon/tests/core_089_skill_test.rs:150:5:
assertion `left == right` failed: les neuf JSON publiés doivent tous avoir un banc explicite
  left: 10
 right: 9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test exemples_skill_envoient_repondent_et_cloturent_sans_service_compagnon ... FAILED

failures:
```

### daemon::native_delegation::native_workspace154_tests::native154_partage_discovery_termine_et_statut_compatibles150

Running unittests src/lib.rs
État du même nom au passage complet154 :ok
Actual JSON ajoute pending_requests=[] et recent_requests=[] ; expected ne contient pas ces deux champs.
Cause non établie ; aucun classement flaky.

Log lignes LF 561–571 :
```text
test daemon::native_delegation::native_workspace154_tests::native154_isolation_petit_enfant_et_conservation_emboitee ... ok
test daemon::native_delegation::native_workspace154_tests::native154_isolation_worktree_absent_au_lancement ... ok
test daemon::native_delegation::native_workspace154_tests::native154_partage_autres_ecrivains_seulement ... ok

thread 'daemon::native_delegation::native_workspace154_tests::native154_partage_discovery_termine_et_statut_compatibles150' (69776827) panicked at crates/bridget-daemon/src/daemon/native_workspace154_tests.rs:425:5:
assertion `left == right` failed: contrat courant 150 complété par 152/153
  left: Object {"agent_type": String("native-test"), "child_agent_id": String("1f0d5651-b5d7-4fcd-ba28-df2f86200058"), "continuable": Bool(false), "cwd": String("/private/tmp/b.3e54c4z5/bridget-n149-com-e24e09d2-27954/home/project-149"), "detail": Null, "effort": String("high"), "error": Null, "idle_deadline_at": Null, "mailbox": Null, "message_id": String("c8d3b006-4fa0-4d88-81e0-a01109961cb5"), "mission_deadline_at": Null, "model": String("custom/model"), "pending_requests": Array [], "posture": String("development"), "recent_requests": Array [], "result": Null, "result_bytes": Null, "resume_attempts": Number(0), "round": Number(1), "status": String("queued"), "summary": Null, "summary_truncated": Bool(false), "suspended": Bool(false), "task_id": String("95838fb9-0244-4981-8c0c-a2285709a3b1"), "version": Number(1), "work_state": String("working")}
 right: Object {"agent_type": String("native-test"), "child_agent_id": String("1f0d5651-b5d7-4fcd-ba28-df2f86200058"), "continuable": Bool(false), "cwd": String("/private/tmp/b.3e54c4z5/bridget-n149-com-e24e09d2-27954/home/project-149"), "detail": Null, "effort": String("high"), "error": Null, "idle_deadline_at": Null, "mailbox": Null, "message_id": String("c8d3b006-4fa0-4d88-81e0-a01109961cb5"), "mission_deadline_at": Null, "model": String("custom/model"), "posture": String("development"), "result": Null, "result_bytes": Null, "resume_attempts": Number(0), "round": Number(1), "status": String("queued"), "summary": Null, "summary_truncated": Bool(false), "suspended": Bool(false), "task_id": String("95838fb9-0244-4981-8c0c-a2285709a3b1"), "version": Number(1), "work_state": String("working")}
test daemon::native_delegation::native_workspace154_tests::native154_partage_discovery_termine_et_statut_compatibles150 ... FAILED
test daemon::native_delegation::native_workspace154_tests::native154_isolation_sous_dossier_deux_branches_et_lancement_par_paire ... ok
test daemon::native_delegation::native_workspace154_tests::native154_rapport_cas_et_premier_rapport_gagnant ... ok
```

### daemon::native_delegation::tests::native152_rouge_annulation_multi_ticks

Running unittests src/lib.rs
État du même nom au passage complet154 :ok
Actual [cancelling,cancelling,cancelling] ; expected [cancelling,starting,working].
Cause non établie ; aucun classement flaky.

Log lignes LF 728–738 :
```text
T029 registre réel après normalisation : "agent_type_unknown"
test daemon::native_delegation::tests::native152_refus_texte_dynamique_jamais_stocke ... ok
T040 réel : arrêts=[false, false], états par tick=[["cancelling", "cancelling", "cancelling"], ["cancelling", "cancelling", "cancelling"], ["cancelling", "cancelling", "cancelling"]]

thread 'daemon::native_delegation::tests::native152_rouge_annulation_multi_ticks' (69780382) panicked at crates/bridget-daemon/src/daemon/native_delegation.rs:5718:13:
assertion `left == right` failed
  left: ["cancelling", "cancelling", "cancelling"]
 right: ["cancelling", "starting", "working"]
test daemon::native_delegation::tests::native152_rouge_annulation_multi_ticks ... FAILED
test daemon::native_delegation::tests::native151_attente_client_ferme_libere_la_reservation ... ok
T007 réel : arrêts=[false, false], états=["cancelling", "cancelling", "cancelling"], erreur=Some("native_cancel_unverifiable:a99c676a-3255-4fb7-8d8c-49990929168d")
```

### daemon::native_delegation::tests::native152_rouge_annulation_partielle

Running unittests src/lib.rs
État du même nom au passage complet154 :ok
Actual cancelling ; expected starting. Trace T007 : arrêts=[false,false], trois cancelling et native_cancel_unverifiable.
Cause non établie ; aucun classement flaky.

Log lignes LF 736–746 :
```text
test daemon::native_delegation::tests::native152_rouge_annulation_multi_ticks ... FAILED
test daemon::native_delegation::tests::native151_attente_client_ferme_libere_la_reservation ... ok
T007 réel : arrêts=[false, false], états=["cancelling", "cancelling", "cancelling"], erreur=Some("native_cancel_unverifiable:a99c676a-3255-4fb7-8d8c-49990929168d")

thread 'daemon::native_delegation::tests::native152_rouge_annulation_partielle' (69780417) panicked at crates/bridget-daemon/src/daemon/native_delegation.rs:5695:9:
assertion `left == right` failed
  left: "cancelling"
 right: "starting"
test daemon::native_delegation::tests::native152_rouge_annulation_partielle ... FAILED
T002 diagnostic réception : task=working, phase=Some("dispatching"), delivery=("70c51388-d6a6-40b6-a462-b9089fe27b0e", 867214191786871085), execution=Some(ExecutionSnapshot { execution_id: "execution-1aee814b-3d50-40c2-b9ae-2525554d42e5", project: None, state: "starting", generation: 1, revision: 0, created_at: 1791769754 })
T002 après accusé réel : phase=Some("acked"), suivi=true
```

### mcp::tests::matrice_fr009_couvre_les_quinze_cas

Running unittests src/lib.rs
État du même nom au passage complet154 :ok
Actual 29 ; expected 28.
Cause non établie ; aucun classement flaky.

Log lignes LF 1421–1431 :
```text
test mcp::tests::lecture_de_contenu_refuse_l_elargissement_de_portee_avant_connexion ... ok
test mcp::tests::ledger_requests_scope_mine_par_defaut_et_all_accepte ... ok
test mcp::tests::limite_d_appels_simultanes_refuse_avant_toute_connexion ... ok

thread 'mcp::tests::matrice_fr009_couvre_les_quinze_cas' (69787528) panicked at crates/bridget-daemon/src/mcp.rs:3856:28:
assertion `left == right` failed
  left: 29
 right: 28
test mcp::tests::matrice_fr009_couvre_les_quinze_cas ... FAILED
daemon périmé sur machine non attestée (test-build) — client c24f6e3af548-dirty sur monordinateur : machine du daemon non attestée — on ne sait pas où relancer ; renseigner HOSTNAME de part et d'autre avant toute relance (machine non attestée)
test mcp::tests::la_coupure_apres_ecriture_rend_les_cles_de_rejeu_et_les_trois_invariants ... ok
```

### mcp::tests::native151_ack_facade_schema_ferme_bornes_et_annotations

Running unittests src/lib.rs
État du même nom au passage complet154 :ok
Actual 29 ; expected 28.
Cause non établie ; aucun classement flaky.

Log lignes LF 1431–1441 :
```text
test mcp::tests::la_coupure_apres_ecriture_rend_les_cles_de_rejeu_et_les_trois_invariants ... ok
test mcp::tests::mutation_mcp_refuse_toute_seconde_source_de_principal_ou_de_registre ... ok
test mcp::tests::mutation_mcp_declare_l_identite_une_fois_et_ne_vend_pas_queued_comme_un_effet ... ok

thread 'mcp::tests::native151_ack_facade_schema_ferme_bornes_et_annotations' (69787542) panicked at crates/bridget-daemon/src/mcp.rs:2538:9:
assertion `left == right` failed
  left: 29
 right: 28
test mcp::tests::native151_ack_facade_schema_ferme_bornes_et_annotations ... FAILED
ACK151 quatre attentes en vol : trames=[Object {"type": String("DaemonIdentityRequest")}, Object {"type": String("RegisterAuxiliary")}, Object {"request": Object {"operation": String("ack"), "task_ids": Array [String("task-z"), String("task-a"), String("task-m")]}, "type": String("NativeDelegation")}], réponse={"id":6,"jsonrpc":"2.0","result":{"content":[{"text":"{\"acknowledged\":[\"task-z\",\"task-m\"],\"already\":[\"task-a\"],\"version\":1}","type":"text"}],"structuredContent":{"acknowledged":["task-z","task-m"],"already":["task-a"],"version":1}}}
ACK151 quatre attentes en vol : trames=[Object {"type": String("DaemonIdentityRequest")}, Object {"type": String("RegisterAuxiliary")}, Object {"request": Object {"operation": String("ack"), "task_ids": Array [String("active")]}, "type": String("NativeDelegation")}], réponse={"id":6,"jsonrpc":"2.0","result":{"code":"native_delegation_refused","content":[{"text":"task_not_ready","type":"text"}],"isError":true}}
```

### t3code::tests::spec099_post_accepte_issue_illisible_conserve_la_correlation

Running unittests src/lib.rs
État du même nom au passage complet154 :ok
Thread anonyme t3code.rs:5187 read unwrap : WouldBlock/code35/Resource temporarily unavailable ; puis join unwrap Any à5262.
Cause non établie ; aucun classement flaky.

Log lignes LF 1752–1760 :
```text

thread '<unnamed>' (69789601) panicked at crates/bridget-daemon/src/t3code.rs:5187:45:
called `Result::unwrap()` on an `Err` value: Os { code: 35, kind: WouldBlock, message: "Resource temporarily unavailable" }

thread 't3code::tests::spec099_post_accepte_issue_illisible_conserve_la_correlation' (69789556) panicked at crates/bridget-daemon/src/t3code.rs:5262:42:
called `Result::unwrap()` on an `Err` value: Any { .. }
test t3code::tests::spec099_post_accepte_issue_illisible_conserve_la_correlation ... FAILED
test t3code::tests::spec099_refus_publication_est_repris_sur_tick_inchange ... ok
test t3code::tests::spec099_reponse_non_acquittee_et_autres_attentes_survivent ... ok
```

## Huit échecs historiques conservés

- Running tests/core_089_contract_test.rs — cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere
- Running tests/core_089_contract_test.rs — client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation
- Running tests/core_089_crash_test.rs — binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee
- Running tests/core_089_identity_test.rs — deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance
- Running tests/core_089_identity_test.rs — noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper
- Running tests/core_089_identity_test.rs — spec094_mcp_modifie_uniquement_sa_propre_presence
- Running tests/core_089_reply_test.rs — cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent
- Running tests/t3code_098_test.rs — spec098_pont_expose_remet_repond_renouvelle_et_retire

## Fixture UTC154

source_brute_et_rotation_traversent_snapshot_puis_live_et_curseur_perime_annonce_gap :PASS dans ce run. Elle reste exclue des huit historiques ; un FAIL futur serait nouveau.

## Limites et garde-fous

- RC101 et sept nouveaux échecs : aucun GREEN workspace/global.
- Les180 PASS ne couvrent pas les sept nouveaux FAIL ; les deux résultats restent distincts.
- Aucun échec n’est présumé flaky ; classification cause nécessite diagnostic des propriétaires.
- Aucun ignored reclassé PASS ; 72 ignored réels conservés.
- Panique injectée d’un test PASS conservée comme telle, jamais comptée comme FAIL sans finale extérieure.
- Aucun test/Cargo/build/Git/service/model lancé ; aucun replay180.
- Logs et reçus représentent les sources pendant ce run avant correction T040 ; pas le futur SHA corrigé.

Les15 traces de panne, les autres paniques de tests PASS, les94 cibles et les180 correspondances exactes sont dans le JSON.
