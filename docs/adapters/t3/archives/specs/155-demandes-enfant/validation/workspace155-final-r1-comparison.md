# Archives techniques de l’adaptateur T3

Document d’origine : [specs/155-demandes-enfant/validation/workspace155-final-r1-comparison.md](../../../../../../../specs/155-demandes-enfant/validation/workspace155-final-r1-comparison.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Running unittests src/lib.rs
État du même nom au passage complet154 :ok
Thread anonyme t3code.rs:5187 read unwrap : WouldBlock/code35/Resource temporarily unavailable ; puis join unwrap Any à5262.
Cause non établie ; aucun classement flaky.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

- Running tests/core_089_contract_test.rs — cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere
- Running tests/core_089_contract_test.rs — client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation
- Running tests/core_089_crash_test.rs — binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee
- Running tests/core_089_identity_test.rs — deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance
- Running tests/core_089_identity_test.rs — noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper
- Running tests/core_089_identity_test.rs — spec094_mcp_modifie_uniquement_sa_propre_presence
- Running tests/core_089_reply_test.rs — cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent
- Running tests/t3code_098_test.rs — spec098_pont_expose_remet_repond_renouvelle_et_retire
