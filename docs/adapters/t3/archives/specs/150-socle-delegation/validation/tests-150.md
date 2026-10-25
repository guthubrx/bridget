# Archives techniques de l’adaptateur T3

Document d’origine : [specs/150-socle-delegation/validation/tests-150.md](../../../../../../../specs/150-socle-delegation/validation/tests-150.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Suite complète du workspace (T023, à la charge du coordinateur).
- `run_maintenance` en entier : il touche `$HOME/Library/Logs`. Le test couvre
  `maintain_stores`, qui contient la recopie, la purge des envois et la rétention.
- Recette avec un vrai T3 ouvert sur la vue Lineage : seule la socket de test est utilisée.

<a id="extrait-2"></a>

## Extrait 2

- `core_089_contract_test` : `cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere`, `client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation`
- `core_089_crash_test` : `binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee`
- `core_089_identity_test` : `spec094_mcp_modifie_uniquement_sa_propre_presence`, `deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance`, `noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper` (message : `auxiliary_credential_required`)
- `core_089_reply_test` : `cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent`
- `t3code_098_test` : `spec098_pont_expose_remet_repond_renouvelle_et_retire`
