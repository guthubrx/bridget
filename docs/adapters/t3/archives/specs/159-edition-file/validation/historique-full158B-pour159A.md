# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/validation/historique-full158B-pour159A.md](../../../../../../../specs/159-edition-file/validation/historique-full158B-pour159A.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Nom exact | Cible | Ligne FULL | Classe et preuve ultérieure |
|---|---|---:|---|
| `daemon::spec099_classic_delivery_tests::spec100_owner_source_collision_and_nonblocking_notifications` | `unittests src/lib.rs` | 1079 | regression_DND_corrigee ; PASS_replay_DND |
| `daemon::spec099_classic_delivery_tests::spec101_upstream_gap_requires_primary_and_dnd_preserves_loss_counters` | `unittests src/lib.rs` | 1084 | regression_DND_corrigee ; PASS_replay_DND |
| `cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere` | `tests/core_089_contract_test.rs` | 2307 | herite_158A_deja_FAIL ; FAIL_historique_sans_replay_correctif_credité |
| `client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation` | `tests/core_089_contract_test.rs` | 2310 | herite_158A_deja_FAIL ; FAIL_historique_sans_replay_correctif_credité |
| `binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee` | `tests/core_089_crash_test.rs` | 2338 | herite_158A_deja_FAIL ; FAIL_historique_sans_replay_correctif_credité |
| `recovery_acked_wrapper_finalise_accepted_apres_crash_daemon` | `tests/core_089_crash_test.rs` | 2345 | defaut_historique_deadline099_revele_dans_FULL158B ; PASS_replay_sans_correctif_defaut_historique_conserve |
| `reprise_daemon_redelivre_les_octets_immuables_a_la_meme_instance` | `tests/core_089_crash_test.rs` | 2349 | defaut_historique_deadline099_revele_dans_FULL158B ; FAIL_replay_deadline099 |
| `deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance` | `tests/core_089_identity_test.rs` | 2416 | herite_158A_deja_FAIL ; FAIL_historique_sans_replay_correctif_credité |
| `noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper` | `tests/core_089_identity_test.rs` | 2417 | herite_158A_deja_FAIL ; FAIL_historique_sans_replay_correctif_credité |
| `spec094_mcp_modifie_uniquement_sa_propre_presence` | `tests/core_089_identity_test.rs` | 2418 | herite_158A_deja_FAIL ; FAIL_historique_sans_replay_correctif_credité |
| `cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent` | `tests/core_089_reply_test.rs` | 2503 | herite_158A_deja_FAIL ; FAIL_historique_sans_replay_correctif_credité |
| `spec098_pont_expose_remet_repond_renouvelle_et_retire` | `tests/t3code_098_test.rs` | 3033 | herite_158A_deja_FAIL ; FAIL_historique_sans_replay_correctif_credité |
