# Archives techniques de l’adaptateur T3

Document d’origine : [specs/157-catalogue-quotas/validation/tests-157.md](../../../../../../../specs/157-catalogue-quotas/validation/tests-157.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Méthode de comptage : chaque marqueur Cargo Running ou Doc-tests ouvre une cible ; seule sa dernière ligne test result compte. La somme brute du runner est 2399/9/74. Une ligne enfant 1 PASS apparaît dans daemon lib, ligne 460 ; deux lignes enfant 1 PASS apparaissent dans transport lib, lignes 3113 et 3117. Ces trois résultats restent des exécutions embarquées, exclues du total 2396/9/74 des 95 cibles Cargo. Les cibles échouées sont daemon lib (1669/1/15), core_089_contract_test (1/2/1), core_089_crash_test (11/1/2), core_089_identity_test (1/3/1), core_089_reply_test (0/1/1) et t3code_098_test (2/1/1).

<a id="extrait-2"></a>

## Extrait 2

La liste157 hors oracle154 est exactement celle des huit modes hérités de156. Les sites d’assertion sont identiques. Sept payloads d’erreur sont identiques ; T3 conserve la même première assertion «1 agent attendu, aucun vu» et le même refus HTTP404, avec ses dates/ports/session propres. Les causes sous-jacentes ne sont pas nouvellement diagnostiquées ni corrigées en157.

<a id="extrait-3"></a>

## Extrait 3

| Test | Site absolu dans le worktree157 | Symptôme inchangé |
|---|---|---|
| cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_contract_test.rs:400 | Identité --from non attestée |
| client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_contract_test.rs:180 | Identité auxiliaire indisponible |
| binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_crash_test.rs:634 | Identité auxiliaire indisponible |
| noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_identity_test.rs:32 | auxiliary_credential_required |
| deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_identity_test.rs:32 | auxiliary_credential_required |
| spec094_mcp_modifie_uniquement_sa_propre_presence | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_identity_test.rs:32 | auxiliary_credential_required |
| cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_reply_test.rs:48 | Identité auxiliaire indisponible |
| spec098_pont_expose_remet_repond_renouvelle_et_retire | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/t3code_098_test.rs:159 | Aucun agent T3 observé, snapshot HTTP404 |
