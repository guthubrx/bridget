# Carte des tests 089 — dispositions avant extraction

## T016 — cycle de réponse sans horloge de complaisance

core_089_reply_test remplace la simple attente du harnais historique par une demande sentinelle réellement rappelée puis expirée. CLI → demande open ; réponse MCP → ACK → answered ; annulation → cancelled ; échéance → timed_out ; ledger sans doublon aux trois lectures. Le mutant qui conserve le pending répondu est refusé sur ses événements ReminderSent, pas sur une formulation humaine du rappel. Aucun fournisseur réel impliqué ; son gate reste T020/T021.

## T015 — ancien rename remplacé par le nom affiché

core_089_identity_test protège l'identité opaque, les refus de nom, le scope d'instance et les octets au redémarrage réel. Ses trois tests passent désormais par le vrai CLI rename, le daemon, le wrapper et MCP : renommage conservé après SIGKILL, refus sans écriture, instances distinctes, fixture filaire fermée. Les deux anciens tests integration_test de renommage supposent une route textuelle modifiable et le namespace historique : leur oracle fonctionnel est remplacé, pas leur ancienne hypothèse de route. Ils ne sont ni exécutés contre la flotte ni comptés verts ; leur disposition physique dans ce fichier mixte sera vérifiée à T034. Mutant supprimant l'UPDATE de nom réellement refusé par la lecture ListAgents du test.

## Disposition exécutée T014 — harnais partagé

Les douze scénarios de idempotency_crash_test.rs sont conservés byte-identiques ; leur seul préambule de fixture devient tests/support/idempotent.rs. Aucun test historique supprimé ni délai augmenté. core_089_contract_test.rs réutilise ces processus isolés pour le canon stocké CLI/MCP, les six formes partielles refusées avant connexion et l'attribution du CLI à UUID. Les sept unités d'attribution maintiennent les refus d'usurpation et ajoutent la propriété de route.

Le guichet historique reste explicitement rouge sur la clôture UUID/ancien nom de service, traité par T018 ; un dépôt maintenant accepté n'est pas une preuve de parcours complet. Les tests de renommage de route historiques seront adaptés à l'identité opaque, pas relancés contre leur ancien HOME/socket.

## Disposition exécutée T013 — paquet physique

Les 129 fichiers du plugin Maicie (dont sa fixture SQLite historique, pas une base utilisateur) sont retirés de l'extraction ; leur disposition métier/consommateur figure dans la table ci-dessous. Ils étaient déjà hors graphe depuis T009. Les retirer ne transforme aucun de leurs tests en succès du noyau ; les oracles du consommateur public restent dus à T018/T019/T029.

Le module disk_trend et ses 13 tests de prédiction sont retirés conformément à T001, sans toucher aux quotas des journaux ni aux huit tests disk_hygiene. L'inventaire Git/target des worktrees sort aussi : cleanup est désormais refusé avant initialisation du namespace. Les deux tests de parseurs multi-commandes conservent leurs autres assertions, seule la forme cleanup disparue est retirée. Le nouveau refus est exercé dans core_089_retired_runtime_test.

Tests disk_trend retirés (périmètre, jamais motif de rouge) :

- `temoin_nominal_pente_reelle_du_28_08`
- `temoin_plateau_serie_reelle_du_28_08_ne_produit_aucune_pente`
- `serie_en_marches_rend_les_marches_et_jamais_une_pente`
- `temoin_refus_fenetre_trop_courte_cas_compilation_112_gio_h`
- `serie_stable_sur_fenetre_courte_reste_un_refus`
- `un_pic_ne_fixe_pas_la_pente`
- `aucun_releve_et_releves_insuffisants_sont_des_refus_distincts`
- `disque_qui_se_libere_ne_predit_aucune_saturation`
- `historique_borne_et_garde_les_plus_recents`
- `trois_observateurs_ignorants_rendent_une_pente_grace_a_l_historique`
- `meme_seconde_remplace_au_lieu_d_empiler`
- `historique_corrompu_donne_un_refus_pas_une_pente`
- `ordre_d_arrivee_indifferent`

Preuve de remplacement structurelle : core_089_dependency_test lit les métadonnées Cargo réelles ; son gate explicite dans un paquet sans plugins/apps/infra vérifie aussi le graphe transitif. Le paquet compile toutes les cibles sans accéder au checkout original. Les tests de communication restent présents, y compris les contrats historiques externes en fixtures.


## Disposition exécutée T011 — moteur projet retiré

Retraits limités au moteur Docker, à son ingress, aux montages/catalogues et aux diagnostics de l'ancienne interface. 88 fonctions de test retirées, recensées ci-dessous par différence avec le commit T010 `8c783cc`. Ce ne sont ni des réussites ni des garanties de communication supprimées. Les six fichiers d'intégration moteur, leurs fixtures Docker/project-profile et les assets infra/project-runtime sont retirés après vérification de l'absence de consommateurs. L'historique Git reste récupérable.

Le refus de nouveau projet sans binding, les variables runtime héritées, la reprise historique sans repli hôte et le rejeu terminal sont prouvés par les nouveaux oracles `core_089_*`. Les tests de liaison historique et les ACL contenus restent présents ; `referent_control`, permissions fournisseur, identité, pass_env et garde de facturation sont conservés. Les deux scénarios de pause/ronde retirés concernent la ronde projet, pas les rappels ni la pause de communication.

### crates/bridget-daemon/src/daemon.rs

- `spec_065_daemon_lie_apres_policy_uid_et_negociation_et_rejoue_l_issue`
- `spec_086_contrat_systeme_exige_service_local_et_emplacement_systeme`
- `spec_065_daemon_refuse_la_mutation_projet_sans_uid_pair`
- `spec_066_daemon_runtime_status_est_local_et_ne_divulgue_pas_la_racine`
- `spec_087_pause_differe_la_ronde_puis_la_reprise_la_livre`
- `spec_079_tick_global_ne_livre_que_la_politique_projet_active`
- `spec_066_ingress_prive_refuse_sans_reservation_avant_toute_inscription`
- `spec_066_projet_docker_ne_replie_jamais_un_spawn_sur_hote`
- `spec_067_preflight_ingress_ne_consomme_pas_la_reservation`
- `spec_066_reservation_ingress_refuse_falsification_et_rejeu`
- `spec_066_ingress_accepte_uniquement_la_reservation_avant_register`
- `spec_066_redemarrage_retablit_ingress_sans_dupliquer_l_agent_docker`
- `spec_066_lifecycle_refuse_l_action_destructive_si_agent_projet_actif`
- `spec_085_capacite_runtime_daemon_ne_projette_que_des_raisons_fermees`
- `spec_085_activation_echouee_compense_conteneur_ingress_et_conserve_host`
- `spec_085_operations_destructives_refusees_agent_actif_et_rejeu`
- `spec_085_redemarrage_reconcilie_runtime_sans_creer_un_second_conteneur`

### crates/bridget-daemon/src/wrapper.rs

- `spec_066_handshake_runtime_exige_toutes_les_identites`
- `spec_066_socket_runtime_est_explicite_et_n_derive_pas_de_home`

### crates/bridget-daemon/src/lifecycle.rs

- `cwd_projet_accepte_racine_descendante_et_worktree_lie_mais_refuse_un_voisin`
- `spec_066_runtime_docker_n_exige_jamais_la_commande_fournisseur_sur_l_hote`

### crates/bridget-daemon/src/control_settings.rs

- `spec_088_profile_for_rend_custom_quand_une_ligne_diverge_ou_manque`
- `spec_088_chaque_profil_nomme_a_ses_valeurs_et_custom_aucune`
- `spec_088_gestes_fermes_portent_le_jeton_et_n_ecrivent_jamais`
- `spec_088_l_enveloppe_bash_de_codex_est_reconnue_mais_pas_une_commande_voisine`
- `spec_088_commande_demarree_sans_fin_reste_pending_puis_expire`
- `spec_088_fin_reussie_avec_jeton_passe_et_autre_message_ne_compte_pas`
- `spec_088_fin_en_echec_apres_ligne_reconnue_est_un_refus_de_sandbox`
- `spec_088_internet_injoignable_reconnu_par_curl`
- `spec_088_une_tentative_par_ligne_et_document_0600`
- `catalogue_est_ferme_et_seule_la_politique_projet_devient_modifiable`
- `spec_084_catalogue_v2_previsualise_applique_rejoue_et_refuse_generation_obsolete`
- `spec_085_migration_du_reglage_execution_conserve_host_et_ne_touche_aucune_liaison`
- `spec_085_execution_default_refuse_docker_tant_que_la_capacite_est_incomplete`
- `spec_085_execution_default_docker_est_versionne_et_idempotent`
- `spec_086_dogfooding_est_desactive_par_defaut_et_refuse_les_transitions_non_admissibles`

### crates/bridget-daemon/src/project_policy.rs

- `spec_065_politique_absente_ferme_les_mutations`
- `spec_065_politique_refuse_schema_vide_inconnu_ou_frontiere_large`
- `spec_065_politique_refuse_proprietaire_ou_mode_non_prive`
- `spec_065_politique_canonicalise_et_contient_les_racines_candidates`
- `spec_065_politique_refuse_les_alias_de_politique_et_canonicalise_les_symlinks`
- `spec_076_politique_avance_generation_atomiquement`
- `spec_080_applique_avec_recu_et_accepte_un_rejeu_identique`
- `spec_084_catalogue_v2_separe_creation_import_et_compatibilite_v1`
- `spec_084_catalogue_v2_refuse_ids_et_chemins_ambigus`
- `spec_084_catalogue_v2_applique_atomiquement_et_rejoue_le_recu`

### crates/bridget-daemon/src/project_runtime.rs

- `project_environment_refuses_invalid_transitions_and_reserves_current_epoch`
- `policy_change_requires_recreation_and_invalidates_reservation`
- `spec_086_topologie_mount_change_epoch_et_exige_recreate`
- `spec_086_attestation_checkout_refuse_absent_faux_main_et_common_dir_etranger`
- `absent_policy_file_closes_only_docker_runtime`
- `spec_085_la_politique_docker_est_bornee_a_linux_amd64`
- `policy_config_rejects_mutable_images_and_invalid_runtime_values`
- `docker_arguments_are_closed_and_do_not_use_a_shell`
- `docker_arguments_refuse_host_root_and_docker_socket_mounts`
- `policy_loader_rejects_group_writable_file`
- `docker_fixture_covers_timeout_hostile_json_image_and_daemon_failure`
- `attestation_divergente_impose_recreate_required`
- `environnement_a_recreer_est_arrete_et_supprime_sans_recherche_globale`
- `arret_et_suppression_sont_deux_transitions_locales_distinctes`
- `ingress_prive_est_deterministe_et_refuse_toute_identite_divergente`
- `preflight_refuse_un_uid_ou_gid_incompatible_avec_le_state_root_prive`
- `politique_runtime_resout_uniquement_un_executable_interne_declare`
- `docker_exec_runtime_transporte_uniquement_les_identites_attestees`
- `spec_066_raisons_runtime_oom_pid_exec_et_redemarrage_sont_fermees`
- `spec_085_montages_incluent_checkout_et_worktrees_lies_aux_memes_chemins`

### crates/bridget-daemon/src/project_workspace.rs

- `spec_076_previsualisation_create_ne_cree_ni_n_ecrase`
- `spec_084_previsualisation_v2_refuse_les_politques_v1_et_parents_libres`
- `spec_084_previsualisation_v2_limite_creation_import_et_collisions`
- `spec_084_previsualisation_v2_refuse_lien_symbolique_hors_workspace_et_systeme`
- `spec_076_import_non_git_ne_modifie_pas_le_dossier`
- `spec_076_import_git_diagnostique_propre_modifie_et_worktree_sans_contenu`
- `spec_076_duree_decouverte_est_bornee`
- `spec_076_configuration_coordinateur_exige_le_digest_atteste`

### crates/bridget-daemon/src/cli.rs

- `spec_066_cli_runtime_projet_exige_operation_et_identifiant_fermes`
- `spec_079_cli_ronde_separe_politique_projet_et_tick_global`

### crates/bridget-daemon/tests/project_runtime_ingress_test.rs

- `spec_066_ingress_prive_isole_projets_et_refuse_les_identites_forgees`

### crates/bridget-daemon/tests/project_runtime_agents_test.rs

- `spec_066_deux_agents_partagent_un_conteneur_sans_melanger_deux_projets`

### crates/bridget-daemon/tests/project_runtime_mounts_test.rs

- `spec_066_worktrees_du_meme_common_dir_sont_montes_sans_elargir_le_projet`

### crates/bridget-daemon/tests/project_runtime_integration_test.rs

- `spec_066_prepare_docker_atteste_image_montages_abi_et_limites`
- `spec_066_docker_exec_lance_bridget_et_un_fournisseur_de_fixture_isole`

### crates/bridget-daemon/tests/project_profile_host_compat_test.rs

- `spec_067_backend_host_refuse_un_profil_a_ressource_et_garde_le_profil_vide`

### crates/bridget-daemon/tests/project_profile_resources_test.rs

- `spec_067_catalogue_refuse_une_source_sans_projet_autorise`
- `spec_067_catalogue_refuse_projet_etranger_et_wildcard`
- `spec_067_catalogue_refuse_source_hors_racines`
- `spec_067_catalogue_refuse_un_secret_place_sous_les_extensions`
- `spec_067_catalogue_refuse_collision_de_source`
- `spec_067_attestation_secrete_ne_porte_ni_valeur_ni_contenu`

Les huit anciens libellés de fixtures lifecycle ont été remplacés par des UUIDv4 littéraux : mêmes identités pour collision, distinctes pour quota. Aucun assouplissement de la validation de production, aucun helper qui transformerait implicitement un nom en identité.

Référence analysée : `dfa2134dcfe2a2522e3ae77d93561e6ae72556b3`.
Racine de travail : /Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core.
Les chemins des tableaux sont des coordonnées dans cet objet Git, pas des chemins de données de production.

## Portée et méthode

Inventaire de **264 fichiers** : tous les fichiers Rust du commit portant `#[test]`, `#[tokio::test]` ou `#[cfg(test)]`, plus les sources dans les répertoires `tests/`, les scripts `test-*.sh`, les fichiers `.test.mjs` et les scénarios `.feature`. Inclut les tests intégrés dans `src/`, les petits lanceurs `#[path]` et leur module cible : le nombre de fichiers n'est donc **pas** le nombre de tests uniques. Les fixtures de données sont inventoriées séparément ci-dessous. Les anciens rapports et spikes documentaires ne sont pas des tests exécutables du produit.

Base reproductible en lecture seule :

```sh
git ls-tree -r --name-only dfa2134dcfe2a2522e3ae77d93561e6ae72556b3
git grep -l -E '#\[(test|tokio::test)|#\[cfg\(test\)\]' dfa2134dcfe2a2522e3ae77d93561e6ae72556b3 -- '*.rs'
```

Filtre complémentaire sur la première liste : sources `.rs/.sh/.py/.mjs/.js/.ts/.tsx/.feature` sous `tests|test|__tests__` hors `fixtures`, suffixes `.test/.spec` JS/TS et noms `test-*/test_*` shell/Python. Union sans doublon avec la deuxième liste. Toute disparition d'un fichier de cet univers doit avoir sa disposition ci-dessous ; un nouveau fichier n'autorise pas à retirer un ancien oracle.

- **C — conserver** (58 fichiers) : même exigence et nom historique, adaptation des chemins isolés seulement si nécessaire.
- **M — déplacer/scinder** (67) : fichier mixte ou harnais dépendant d'un composant sorti. **Aucune suppression globale autorisée** ; préserver l'oracle transport dans les trois crates ou dans le harnais client public sans Maicie, avant de sortir la partie métier/présentation. La revue du diff doit nommer chaque test déplacé.
- **R — retirer du paquet cible** (139) : comportement explicitement hors périmètre. Le test et son implémentation sortent ensemble après T006 ; l'historique Git reste. Cela ne supprime pas l'exigence équivalente de sécurité de la communication.

Ces dispositions ne dépendent d'aucun résultat rouge. Elles ne déclarent aucun scénario validé. **Aucun daemon, compte fournisseur ni tunnel n'a été lancé pour établir cette carte.** T005 reste le point d'audit puis d'exécution de la référence.

## Correspondance des douze critères

Les symboles ci-dessous existent dans la référence ; leur lecture n'est pas une nouvelle exécution. « À compléter » est une preuve 089 à produire, jamais un test supposé déjà présent. Les nouveaux noms de fichiers du plan sont des destinations possibles : privilégier l'extension d'un oracle historique à sa duplication.

| Critère | Oracles historiques nommés | Limite et preuve 089 à produire |
|---|---|---|
| SC-08901 | `integration_test.rs::test_two_agents_communicate`, `test_request_cancellation_stops_reminders`, `redelivery_idempotente_apres_reconnexion_n_injecte_qu_un_prompt` ; `managed_parity_test.rs::reprise_codex_utilise_le_client_bridget_et_clot_les_demandes_liees` | Harnais à faux fournisseur ≠ deux abonnements réels. T015/T016/T020 : deux pilotes distincts réels, réponse liée, compteur prompt/ledger et aucune relance après answered. |
| SC-08902 | `src/cli.rs::projection_cli_et_reference_partagent_le_canon_du_daemon_reel` (6984), `rendu_ledger_cli_reste_octet_pour_octet_stable` (5472) ; `idempotency_crash_test.rs::outil_mcp_rejette_la_reponse_liee_divergente_sans_muter_les_demandes` (962), `binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee` (1131) ; `artifact_publication_test.rs::publication_accepte_rejeu_identique_et_refuse_parametre_ou_provenance_hors_contrat` | T014/T022 : nouveau paquet, CLI/MCP réels même daemon, tous champs divergents et références exactes ; comparer le record durable avant/après, pas deux encodages locaux. |
| SC-08903 | `idempotency_crash_test.rs::matrice_crash_sc001_redelivre_cinquante_prompts_uniques` (675), `reprise_daemon_redelivre_les_octets_immuables_a_la_meme_instance` (767), `destination_remplacee_reste_indeterminee_sans_reroutage` (819), `recovery_acked_wrapper_finalise_accepted_apres_crash_daemon` (1309) | T017 : lancer explicitement la matrice test-support avec watchdog et processus isolés ; ses skips Linux/kqueue ne valent pas validation distante. Crash wrapper et horizon restent des limites, pas un exactly-once universel. |
| SC-08904 | `scripts/test-federate-ssh.sh` (syntaxe/usage uniquement) ; `managed_parity_test.rs::matrice_fr008_compare_le_meme_corpus_et_les_frames_attach` (coupure de socket locale) | **Aucun de ces tests ne prouve deux serveurs.** T025–T027 : SSH réel isolé, même annuaire/ledger maître, perte/reprise, compteur prompt et PID fournisseur inchangé. |
| SC-08905 | `sc005_attach_budget.rs::sc002_rejeu_vers_suivi_traverse_la_rotation_sans_perte_ni_doublon` (736) ; `coordination_events_test.rs::gap_et_unavailable_restant_des_observations_distinctes` (655), `reprise_cursee_survit_aux_crashs_reels_et_conserve_les_octets` (488) ; `transport/src/codex_app_server.rs::session_native_respecte_sequence_saturation_et_octets_sources` | T019 : conserver raw/provenance de bout en bout ; vérifier SnapshotCaughtUp avant émission live ; mutants Gap→Fresh et Gap→Unavailable doivent échouer dans le bon parcours. |
| SC-08906 | `mission_boundary_test.rs::le_daemon_n_a_pas_de_dependance_maicie_en_production` (4) : source guard existante, pas preuve du paquet | T007/T009–T013 : construction et exécution d'un paquet sans GUI/Maicie/Docker/tmux ; graphe de dépendances réel. L'oracle historique référence encore les sources UI et ne suffit pas à l'extraction. |
| SC-08907 | `execution_store_test.rs::migration_execution_store_garde_les_tables_heritees_et_rejoue_sans_effet` (19), `migration_execution_store_complete_les_lignes_heritees_sans_les_effacer` (44) ; `artifact_store_test.rs::migration_publication_et_rejeu_sont_atomiques` (74) ; `identity_migration_test.rs`, `idempotency_test.rs` | T031 : copies historiques de chaque magasin retenu, IDs/bytes/terminaux intacts, schéma futur refusé et empreinte des sources inchangée. Une réouverture seule ne prouve pas la migration de toutes les tables. |
| SC-08908 | `capabilities_integration_test.rs::modele_non_declare_est_refuse_avant_processus_et_ordre_durable` (196), `commande_native_absente_est_refusee_sans_residu_puis_reparable_au_meme_id` (244) ; `src/mcp_identity.rs::refuse_pid_recycle_hors_agent_legacy_et_instance_divergente` (584) ; `src/mcp.rs::huit_connexions_simultanees_gardent_le_principal_resolu_et_la_neuvieme_est_busy` (3354) | T029/T030 : permissions/symlinks/UID/capacités, taille filaire LF compris et timeout global ; pour chaque refus vérifier aucun processus/record/accès interdit. Ne pas retirer un contrôle parce que son premier consommateur était la GUI. |
| SC-08909 | `codex_native_test.rs::gate_reel_codex_app_server_gpt_5_6_terra_et_attach` (333, ignored explicite), `wrapper_codex_sans_signal_laisse_effort_et_limite_inconnus` (288) ; `transport/src/codex_app_server.rs::eof_pendant_un_tour_reveille_le_worker_et_nettoie_le_groupe` (5362), `stop_pendant_un_tour_termine_le_groupe_enfant` (5437) ; `claude_native_permissions_test.rs`, `managed_wrapper_test.rs` | T020/T021 : pilotes installés réels, Claude natif/ACP et GLM via Claude Code avec abonnement autorisé ; compte absent = gate non validé. Fixtures/simulations ne certifient pas le fournisseur. |
| SC-08910 | `src/mcp.rs::matrice_fr009_couvre_les_quinze_cas` (1845), `schema_send_expose_in_reply_to_non_vide` (2694), `dto_ledger_respectent_le_contrat_outil` (3456) ; `src/cli.rs::rendu_ledger_distingue_en_vol_et_recu` (5533) | T023 : skill livrable courte conforme aux statuts, mini-scénario CLI/MCP sans Maicie exécuté ; aucun ancien test n'atteste à lui seul cette nouvelle skill. |
| SC-08911 | L'inventaire complet ci-dessous + tests du vérificateur de corpus T002 ; les suites Cargo historiques gardent leurs oracles | T034/T036 : zéro disparition non justifiée, fmt/clippy/tests/audit deps consignés ; aucun nouvel ignore, aucun filtre cachant un rouge. Les .feature sont une spécification lisible, pas un runner installé. |
| SC-08912 | `sc005_attach_budget.rs::sc005_deux_vues_reelles_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent` (517), `sc001_append_vers_rendu_attach_reel_reste_sous_les_seuils_locaux` (549, ignored explicite) ; `execution_scale_test.rs::projection_execution_reste_bornee_sur_256_agents` | T005/T028/T033 : références et candidat sur même hôte, 600 événements/60 s locaux et SSH, pertes/p95/max/clock skew/RSS ; compter modules et dépendances réels. Ne pas changer les seuils pour faire passer. |

Pour les symboles sans préfixe de crate dans ce tableau, `*_test.rs` est sous `crates/bridget-daemon/tests/` et `src/` sous `crates/bridget-daemon/`. Les coordonnées sont celles du commit épinglé, pas des lignes promises après refactor.

## Dispositions exécutées T009 — retrait du métier, maintien des garanties

Dans wrapper.rs, quatre anciens témoins mixtes ont des remplaçants sans greffe :
`carte_de_reprise_reconstruit_mission_et_worktree_durables` →
`carte_de_reprise_reconstruit_identite_et_worktree_durables` ;
`carte_de_reprise_compte_sept_lignes_apres_refus_d_un_nom_avec_lf` →
`carte_de_reprise_refuse_l_identite_injectant_une_ligne` ;
`TEMOIN_carte_de_reprise_instruction_lf_ne_cree_pas_de_ligne_de_consigne` →
`carte_de_reprise_ne_transforme_pas_un_champ_externe_en_consigne` ;
`carte_de_reprise_signale_chaque_source_indisponible` →
`carte_de_reprise_signale_git_indisponible_sans_inventer`.

Six témoins purement Maicie sortent du wrapper :
`carte_de_reprise_nomme_la_reecriture_du_sha_juge`,
`carte_de_reprise_sans_mission_ne_l_invente_pas`,
`carte_de_reprise_mission_close_prescrit_attente`,
`carte_de_reprise_en_attente_prerequis_ne_relance_pas`,
`carte_de_reprise_annulee_hors_clos_ne_promet_pas_de_suite`,
`carte_de_reprise_a_evaluer_avec_levier_affiche_suite_du_greffe`.
La borne de carte, la protection checkout principal vs clone, le prompt
versionné et les tests ManagedSession/raw/reconnexion restent conservés.

Dans daemon.rs, le crash de reprise conserve générations/groupe/commande
figée/compte d'exécutions/Git ; seules la fixture et les assertions de mission
Maicie sortent. Le test busy remplace le binaire Maicie par ListAgents puis
SendIdempotent sur le chemin socket public, avec corps exact reçu.
identity_migration_test garde deux tests ledger/flotte/sauvegardes et retire
la migration de la configuration privée Maicie. mission_boundary_test gagne
la garde des dépendances de test et des lectures implicites de coordination.
project_registration_e2e.rs (un test) est retiré selon sa disposition R.

Comptage statique de ces six fichiers : 282→276 tests (−6), pas un résultat
d'exécution. Le détail des validations et des exclusions demeure dans
implementation.md ; aucun retrait n'est motivé par un rouge.

## Dispositions exécutées T010 — contenu sans interface

ui.rs, ses assets et apps/bridget-desktop sortent ensemble : HTTP/WebView,
cache de présentation, JS/CSS et renderers sont retirés, avec leurs tests R.
artifact_fetch.rs sort avec `collecte_refuse_destinations_et_source_distante_non_autorisee`
(collecte HTTP hors périmètre). mission_projection.rs sort avec son lecteur UI.
Aucune table de contenu/provenance/ledger ni fixture épinglée n'est supprimée.
Les fichiers retirés, dont les deux icônes binaires, restent dans Git.

Scission de ui_relay_test.rs (25 tests) :

- Quatre oracles déplacés vers channel_observation_test.rs :
  `spec_024_reconnexion_inconnue_explicite_efface_le_canal_precedent`,
  `spec_024_reconnexion_historique_omise_conserve_le_canal_precedent`,
  `spec_024_inconnu_explicite_interdit_repli_transport_historique`,
  `spec_024_omission_historique_conserve_repli_transport`.
  Register/ListAgents réels remplacent HTTP, mêmes faits et contre-oracles.
- Quatre attestations propres au relais HTTP retirées :
  `spec_024_ui_locale_attestee_projette_unix_dans_agent_info`,
  `spec_024_ui_federee_projette_ssh_unix_dans_agent_info`,
  `spec_024_ui_sans_attestation_reste_inconnue_dans_agent_info`,
  `spec_024_ui_aux_attestations_divergentes_reste_inconnue_dans_agent_info`.
- Huit tests de surface HTTP retirés avec leurs routes :
  `requete_loopback_sans_jeton_est_refusee`,
  `get_sur_v1_send_reste_interdit_apres_ouverture_du_post`,
  `spec_073_route_stop_verrouille_methode_jeton_et_version`,
  `spec_073_route_stop_relaie_le_verdict_correle`,
  `spec_073_route_stop_ferme_l_erreur_de_protocole`,
  `post_v1_send_corps_vide_rend_le_code_ferme_invalid_body`,
  `post_v1_send_valide_repond_202_et_livre_un_identifiant_non_vide`,
  `relais_sert_les_trois_assets_hors_du_source_rust`.
- `spec_074_cli_endpoint_lit_l_etat_sans_demarrer_de_relais_ni_divulguer_en_erreur`
  et l'unité CLI `endpoint_ui_exige_un_contrat_json_ferme` remplacés par
  `ui_est_refusee_avant_namespace_socket_endpoint_ou_execution` : ancien verbe
  refusé avant même l'initialisation, zéro lecture/connexion/divulgation.
- Huit projections UI retirées ; garanties transport toujours conservées :
  `loopback_rend_snapshot_et_relaie_un_fragment_attach_d_un_agent_vivant`,
  `watch_annonce_reconnecting_puis_connected_apres_coupure_daemon` → journal,
  attach_journal_attestation_test, managed_parity_test, sc005_attach_budget ;
  `post_v1_send_reply_true_cree_une_demande_suivie`,
  `post_v1_send_destinataire_inconnu_refuse_sans_archiver` → integration_test,
  idempotency_crash_test et ledger ;
  `snapshot_compose_la_ligne_agent_avec_les_faits_du_ledger`,
  `snapshot_sans_agent_omet_les_pairs_et_watch_agent_les_projette`,
  `watch_pousse_thread_message_sortant_apres_ouverture`,
  `relais_ui_expose_separement_connexion_vitalite_tour_attente_et_file` →
  projections publiques annuaire/ledger/exécution. Cette correspondance ne
  prétend pas rejouer toutes les gates : T019/T022/T025–T028 restent à faire.

execution_observability_test retire seulement
`alertes_couvrent_vieillissement_sans_progres_autorisation_et_saturation`
(seuils UI), conserve les compteurs bornés. project_profile_surface_test
conserve l'interdiction MCP d'approbation distante sans inclure ui.rs.
mission_boundary_test ne lit plus mission_projection.rs.
artifact_service_test remplace save_html_interaction par refresh explicite :
enfant distinct/HTML canonique/blob absent refusé restent exercés.

act_kind.rs garde le vocabulaire fermé et ses deux oracles d'écriture ; seuls
les quatre oracles JS/Node sortent :
`TEMOIN_vocabulaire_vue_et_ecriture_ne_divergent_pas`,
`mutant_parse_source_reste_vert_sur_map_equivalent_mais_runtime_voit_le_sens`,
`TEMOIN_projectTimeline_filtre_par_defaut_est_JOURNAL_ACT_KINDS`,
`mutant_REAL_ACT_KINDS_tue_TEMOIN_projectTimeline_filtre_par_defaut`.
Le pilote Codex ne préfixe plus de consigne UI à tous les tours : son témoin
réel côté adaptateur compare les deux corps reçus (saturation/retry) au
document source ; raw/provenance conservés. Les lectures CLI/MCP relisent
les octets stockés et refusent les extensions de portée. Le cas HTML à
512K couvre aussi l'enrichissement historique, après correction de la borne
trop étroite trouvée par contre-revue. Aucun retrait n'est motivé par un rouge.

## Points de vigilance avant exécution ou retrait

1. **Faux fournisseur vs fournisseur réel.** Un vrai daemon et une fausse commande shell sont une couture réelle de Bridget, pas une preuve de compatibilité de l'abonnement Codex/Claude/GLM. Les deux niveaux ont leur utilité et doivent rester étiquetés.
2. **Guichet externe.** Les tests publics aujourd'hui dans Maicie (`contract/bridget_client.rs`, `guichet_client.rs`, `coordination_client.rs`, `runtime_subscription.rs`) protègent le contrat Bridget ; ils sont **M**, pas R. Le nouveau client de test ne peut ni importer le store Maicie ni reconstruire un DTO filaire parallèle.
3. **Contenus.** HTML inerte transmissible n'est pas renderer HTML. `artifact_service_test.rs::html_sandboxe_est_canonique_et_blob_absent_ne_devient_pas_un_succes` mélange les deux ; son assertion « blob absent ≠ succès » doit survivre, même si la preview disparaît. Une référence de projet existante ne devient pas une visibilité globale.
4. **Source guard trop étroite.** `mission_boundary_test.rs` vérifie le texte du manifest et des `include_str!` ; elle ne remplace pas `cargo metadata` ni le build du paquet sans sources Maicie. Le manifest épinglé contient une dépendance de production Maicie : à caractériser en T005, pas supprimer le test.
5. **Fixture Codex historique.** `provider-contracts/codex-0.150.1.jsonl` contient `jsonrpc`, alors que l'oracle du pilote vérifie son absence sur le fil. Conserver le test de forme sans le promouvoir en canon natif ; T002 doit documenter/geler une émission de référence.
6. **SSH.** Le script historique appelle syntaxe, grep et refus d'arguments ; aucune attestation serveur distant. Les anciens rapports T712/T809 sont des preuves historiques, pas une recette du nouveau paquet.
7. **Frontière de nettoyage.** Avant tout harnais : HOME/cache/artifacts/socket propres, aucun outil fournisseur authentifié lancé sans gate explicite, validation des PIDs enfants et watchdog global. Aucun test ici n'est lancé automatiquement par sa présence dans la carte.
8. **Tests mixtes intégrés.** C/M/R sont des dispositions de fichiers, pas un permis de supprimer les tests d'un module M en masse. Au moment de la coupe, le diff doit associer nom historique → destination ou raison précise de retrait. T003 ne prétend pas effectuer cette revue future.
9. **Ignores historiques.** Auditer `git grep -n -E '#\[ignore|ignore *=' <référence> -- crates plugins` avant T005. La matrice crash est explicitement ignorée par défaut ; des tests de coordination exigent test-support ; des auxiliaires crash de fleet/desired_state/managed_process sont lancés par leurs parents. Les trois catégories ne doivent pas être confondues.
10. **Ancien micro-banc.** `transport/src/acp.rs:2957` ignore le micro-banc remplacé par SC-005 réel : retrait permis avec ce remplacement nommé, pas parce qu'un benchmark gêne. Le banc réel reste conservé.

## Inventaire exhaustif des fichiers historiques

### Sources unitaires et tests des trois crates

| Fichier historique | Décision | Motif / destination d'oracle |
|---|---|---|
| `crates/bridget-core/src/circuit_breaker.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-core/src/dedup.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-core/src/envelope.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-core/src/execution.rs` | M | Faits de cycle d'exécution gardés ; décisions métier séparées (T012). |
| `crates/bridget-core/src/host.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-core/src/message.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-core/src/router.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-core/src/text_guards.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/assets/ui/artifact-renderer.test.mjs` | R | Renderer et sandbox HTML hors produit extrait ; conserver le contenu source et ses droits ailleurs (T010). |
| `crates/bridget-daemon/assets/ui/artifact-sandbox-host.test.mjs` | R | Renderer et sandbox HTML hors produit extrait ; conserver le contenu source et ses droits ailleurs (T010). |
| `crates/bridget-daemon/src/agent_profile.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/artifact_blob_store.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/artifact_fetch.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/artifact_policy.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/artifact_types.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/attach.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/build_identity.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/build_info.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/cli.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/connection_channel.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/control_settings.rs` | R | Module hors communication classé retirer par T001 ; aucune suppression avant revue T006. |
| `crates/bridget-daemon/src/daemon.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/desired_state.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/disk_hygiene.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/disk_trend.rs` | R | Module hors communication classé retirer par T001 ; aucune suppression avant revue T006. |
| `crates/bridget-daemon/src/execution_store.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/fleet.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/greffe_policy_refresh.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/human_inbox.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/idempotency.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/identity_migration.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/ledger.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/lifecycle.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/managed_process.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/managed_supervisor.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/mcp.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/mcp_identity.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/mission_projection.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/project_policy.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/project_runtime.rs` | R | Module hors communication classé retirer par T001 ; aucune suppression avant revue T006. |
| `crates/bridget-daemon/src/project_workspace.rs` | R | Module hors communication classé retirer par T001 ; aucune suppression avant revue T006. |
| `crates/bridget-daemon/src/reaper.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/receipt_store.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/recovery_trace.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/referent_control.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/registry.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/reprise.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/runtime.rs` | C | Garanties du noyau, fichiers privés, identité, registre, journal ou livraison ; garder noms et oracles. |
| `crates/bridget-daemon/src/store.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/src/ui.rs` | M | Retirer serveur/rendu HTTP ; porter au préalable les oracles de lecture, authentification et vérité des projections (T010/T022/T029). |
| `crates/bridget-daemon/src/wrapper.rs` | M | Module mixte T001 : conserver tous les oracles communication/autorisation ; déplacer ou retirer uniquement ceux de politique/présentation (T007–T012). |
| `crates/bridget-daemon/tests/agent_graph_test.rs` | M | Liens/corrélations et reprise conservés ; quotas/politiques d'orchestration séparés (T012). |
| `crates/bridget-daemon/tests/artifact_lifecycle_test.rs` | M | Préserver références, refus de blob absent, annulation et versions ; collecte HTTP optionnelle hors noyau (T010). |
| `crates/bridget-daemon/tests/artifact_policy_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/artifact_publication_test.rs` | M | Préserver publication/provenance/rejeu/isolement ; retirer uniquement attente de renderer HTML après preuve sans rendu (T010). |
| `crates/bridget-daemon/tests/artifact_service_test.rs` | M | Préserver transaction et contenu exact ; séparer sandbox d'affichage de contrôle d'accès des pièces jointes (T010). |
| `crates/bridget-daemon/tests/artifact_store_test.rs` | C | Store de contenus/références : migration, accès ciblé, version et corruption restent exigibles (T010/T031). |
| `crates/bridget-daemon/tests/attach_journal_attestation_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/build_id_integration_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/capabilities_integration_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/claude_native_permissions_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/cli_arguments_integration_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/codex_native_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/coordination_events_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/daemon_shutdown_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/execution_budget_test.rs` | M | Garder usage attesté/absence de zéro inventé ; politiques d'autorisation de continuation hors transport (T012). |
| `crates/bridget-daemon/tests/execution_lifecycle_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/execution_observability_test.rs` | M | Garder compteurs bornés ; alertes UiAlertThresholdsV1 relèvent de la vue retirée (T010/T012). |
| `crates/bridget-daemon/tests/execution_resume_test.rs` | M | Conserver identité/génération/reprise fidèle ; séparer décision de continuation du transport (T012). |
| `crates/bridget-daemon/tests/execution_scale_test.rs` | M | Borne de lecture des exécutions conservée ; adapter la projection sans UI, pas le budget (T012/T033). |
| `crates/bridget-daemon/tests/execution_store_test.rs` | M | Migrations et bytes de reprise obligatoires ; sortir seulement politiques/projections liées au projet (T012/T031). |
| `crates/bridget-daemon/tests/greffe_policy_refresh_test.rs` | M | Garder capacité/permissions de service ; retirer contenu et renouvellement de politique métier Maicie (T018). |
| `crates/bridget-daemon/tests/guichet_integration_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/idempotency_crash_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/idempotency_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/identity_migration_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/identity_probe_timeout_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/integration_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/managed_parity_test.rs` | M | Garder parité, facturation, registres, corrélation et crashs ; adapter seulement branche interactive/tmux non requise (T020/T021). |
| `crates/bridget-daemon/tests/managed_wrapper_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/mcp_injection_smoke_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/mission_boundary_test.rs` | M | Porter l'oracle anti-import privé vers graphe du paquet extrait ; supprimer ses include_str de sources retirées, pas l'exigence (T009/T013). |
| `crates/bridget-daemon/tests/project_binding_integration_test.rs` | R | Parcours de projet/runtime retiré ; absence de permissions fournisseur jamais réinterprétée comme autorisation (T011/T029). |
| `crates/bridget-daemon/tests/project_profile_host_compat_test.rs` | R | Parcours de projet/runtime retiré ; absence de permissions fournisseur jamais réinterprétée comme autorisation (T011/T029). |
| `crates/bridget-daemon/tests/project_profile_resources_test.rs` | R | Parcours de projet/runtime retiré ; absence de permissions fournisseur jamais réinterprétée comme autorisation (T011/T029). |
| `crates/bridget-daemon/tests/project_profile_surface_test.rs` | R | Parcours de projet/runtime retiré ; absence de permissions fournisseur jamais réinterprétée comme autorisation (T011/T029). |
| `crates/bridget-daemon/tests/project_registration_e2e.rs` | R | Parcours de projet/runtime retiré ; absence de permissions fournisseur jamais réinterprétée comme autorisation (T011/T029). |
| `crates/bridget-daemon/tests/project_runtime_agents_test.rs` | R | Parcours de projet/runtime retiré ; absence de permissions fournisseur jamais réinterprétée comme autorisation (T011/T029). |
| `crates/bridget-daemon/tests/project_runtime_ingress_test.rs` | R | Parcours de projet/runtime retiré ; absence de permissions fournisseur jamais réinterprétée comme autorisation (T011/T029). |
| `crates/bridget-daemon/tests/project_runtime_integration_test.rs` | R | Parcours de projet/runtime retiré ; absence de permissions fournisseur jamais réinterprétée comme autorisation (T011/T029). |
| `crates/bridget-daemon/tests/project_runtime_mounts_test.rs` | R | Parcours de projet/runtime retiré ; absence de permissions fournisseur jamais réinterprétée comme autorisation (T011/T029). |
| `crates/bridget-daemon/tests/sc005_attach_budget.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/spawn_refusal_hosts_test.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-daemon/tests/ui_relay_test.rs` | M | Retirer HTTP/jeton/HTML avec relais ; porter les oracles annuaire/journal et sources honnêtes vers CLI/MCP/attach (T010/T019/T022). |
| `crates/bridget-daemon/tests/work_submission_test.rs` | M | Préserver admission/rejeu et bytes durables ; priorité métier séparée sans perdre ordre de remise (T012). |
| `crates/bridget-transport/src/acp.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-transport/src/act_kind.rs` | M | Canon du journal conservé ; tests de concordance UI (node et lecture app.js) sortis avec le renderer, pas avec les variantes consommées par attach (T010/T019). |
| `crates/bridget-transport/src/claude_provider_session.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-transport/src/claude_stream_json.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-transport/src/codex_app_server.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-transport/src/fsutil.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-transport/src/greffe_authorization.rs` | M | Conserver codec/canon, capacité et faits publics ; isoler familles de projet et politique métier (T002/T011/T018). |
| `crates/bridget-transport/src/greffe_policy_refresh.rs` | M | Conserver codec/canon, capacité et faits publics ; isoler familles de projet et politique métier (T002/T011/T018). |
| `crates/bridget-transport/src/journal.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-transport/src/managed_session.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-transport/src/project_profile_protocol.rs` | R | DTO de runtime/profil projet retirés après séparation des références communicables (T011). |
| `crates/bridget-transport/src/protocol.rs` | M | Conserver codec/canon, capacité et faits publics ; isoler familles de projet et politique métier (T002/T011/T018). |
| `crates/bridget-transport/src/refusals.rs` | C | Communication, observation ou cycle de vie fournisseur dans le périmètre ; conserver sans ignorer de rouge. |
| `crates/bridget-transport/src/tmux.rs` | M | Porter les oracles génériques d'enveloppe/réponse ; enlever le besoin tmux dans la recette principale seulement après gates natifs (T021). |
| `crates/bridget-transport/tests/project_runtime_contract_test.rs` | R | Contrat du runtime projet retiré du paquet ; pas le contrat des sessions fournisseur (T011). |
| `crates/bridget-transport/tests/provider_contract_test.rs` | C | Garder les fixtures historiques de forme ; elles ne remplacent pas les émissions natives réelles (T002/T020). |

### Clients desktop historiques

| Fichier historique | Décision | Motif / destination d'oracle |
|---|---|---|
| `apps/bridget-desktop/src-tauri/src/artifact_cache.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/artifact_sandbox.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/connection.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/fleet.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/host_identity.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/panels.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/preferences_store.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/profile.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/profile_service.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/profile_store.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/src/ssh.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/artifact_sandbox_test.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/browser_panel_test.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/browser_preferences_test.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/browser_security_test.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/capabilities.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/connection_lifecycle.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/desktop_commands.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/remote_connection.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/secrets_and_diagnostics.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/spec_081_fleet_projection.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |
| `apps/bridget-desktop/src-tauri/tests/two_panels.rs` | R | Coque desktop/affichage client retirés (T010) ; sécurité IPC/SSH indépendante couverte par T029/T025, pas par cette coque. |

### Consommateur métier Maicie (hors paquet final)

| Fichier historique | Décision | Motif / destination d'oracle |
|---|---|---|
| `plugins/maicie/src/app.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/bridget_client.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/src/catalogue.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/citation.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/config.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/control.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/domain.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/guichet.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/src/install_publish.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/main.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/preuve.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/project_profile.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/review_continuity.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/runtime.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/src/store.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/src/ui_projection.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/ack_lost_recovery_integration.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/approval_atomicity_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/arbitration_link_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/bridget_client_contract.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/catalogue_arbitration_reconcile_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/catalogue_session_gate.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/cli_delegate_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/cli_objective_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/cli_profile_activation_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/cli_routine_surface.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/config_contract.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/bridget_client.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/contract/catalogue.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/config.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/coordination_client.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/contract/coordination_domain.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/delegate.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/domain.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/domain_json.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/duration_timeout.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/execution_projection.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/f36_f37_suite_citations.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/guichet_client.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/contract/guichet_domain.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/profiles.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/review_criticality.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/review_submission.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/routines.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/contract/runtime_subscription.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/contract/telemetry.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/controle_referent_087.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/coordination_client_contract.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/coordination_dispatch_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/coordination_store_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/delegate_contradiction_guichet_contract.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/delegation_outcomes_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/delegation_soldee_par_cloture.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/direct_message_isolation.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/duration_timeout_contract.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/greffe_central_channel_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/guichet_client_contract.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/guichet_forbidden_operations_contract.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/guichet_gate_integration.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/guichet_greffe_integration.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/guichet_projections_integration.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/human_origin_attestation.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/identity_migration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/install_republish_before_migrate.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/ack_lost_recovery.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/integration/approval_atomicity.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/arbitration_link.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/catalogue_arbitration_reconcile.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/cli_delegate.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/cli_objective.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/cli_profile_activation.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/coordination_dispatch.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/coordination_store.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/delegation_outcomes.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/direct_message_isolation.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/guichet_gate.rs` | M | Porter l'oracle du consommateur public dans un harnais externe sans crate Maicie ; conserver bytes/capacités/délais/autorisation (T018/T019/T029). |
| `plugins/maicie/tests/integration/guichet_greffe.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/integration/guichet_projections.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/integration/maicie_confirmation.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/profile_approval.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/spawn_order.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/integration/status_benchmark.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/integration/status_sources.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/integration/store_outbox.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/maicie_confirmation.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/migration_consent.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/migration_v19_guichet_refusals.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/mvp_gate.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/plage_resource_ranges.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/profile_approval_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/project_registration_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/provenance_objective_contract.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/review_continuity_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/review_git_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/schema_migration_guard.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/schema_preflight.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/spawn_order_integration.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/status_benchmark_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/status_sources_integration.rs` | M | Conserver couture transport/rejeu/fraîcheur via client externe ; laisser les assertions métier au projet Maicie (T017–T019). |
| `plugins/maicie/tests/store_outbox_integration.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |
| `plugins/maicie/tests/support/historical_guichet_receptions.rs` | R | Métier/configuration/catalogue/store/CLI Maicie hors noyau ; historique reste dans Git, pas de garantie transport supprimée (T009). |

### Scripts, infrastructure et scénarios historiques

| Fichier historique | Décision | Motif / destination d'oracle |
|---|---|---|
| `infra/project-runtime/production/tests/validate-production-assets.sh` | R | Assets du runtime Docker/projet retirés (T011). |
| `scripts/test-018-pilotage-install.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `scripts/test-053-review-witness-selection.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `scripts/test-bridget-idle.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `scripts/test-bridget-rapport-vp.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `scripts/test-bridget-ronde-dispatch.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `scripts/test-bridget-ronde.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `scripts/test-federate-ssh.sh` | M | Syntaxe/usage SSH gardés et complétés : aucun échange distant réel dans ce script historique (T024–T028). |
| `scripts/test-git-pre-push-authorship.sh` | C | Convention de livraison et protection de l'identité auteur, indépendante du produit. |
| `scripts/test-install-k1-exclusion.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `scripts/test-install-k1-macos-releve.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `scripts/test-install-k1-preflight.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `scripts/test-open-remote-ui.sh` | R | Ouverture de l'interface distante supprimée, pas le tunnel de communication (T010/T024). |
| `scripts/test-registre-nature-reclasser.sh` | R | Pilotage/greffe/installation Maicie hors produit communication ; mécanisme d'installation noyau couvert séparément T035. |
| `tests/features/019-trace-interactions-codex.feature` | C | Scénarios de trace/corrélation/refus fournisseur conservés (T019/T020). |
| `tests/features/023-observabilite-tour-refus.feature` | C | Scénarios de trace/corrélation/refus fournisseur conservés (T019/T020). |
| `tests/features/025-carte-criticite-regime.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/064-plan-controle-bridget-maicie.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/065-registre-identite-projets.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/066-environnement-partage-projet.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/067-profils-extensions-secrets-projet.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/079-continuite-travaux-redemarrage.feature` | M | Préserver reprise/autorisation/contrôle humain ; enlever seulement parcours UI/projet (T012/T029). |
| `tests/features/080-centre-controle-bridget.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/081-pilotage-rondes-projet-ui.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/084-projets-multiserveurs-emplacements.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/085-runtime-docker-production.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/086-projet-systeme-bridget-dogfooding.feature` | R | Scénarios de coordination métier, projet ou interface retirés (T009–T011). |
| `tests/features/087-reprendre-controle.feature` | M | Préserver reprise/autorisation/contrôle humain ; enlever seulement parcours UI/projet (T012/T029). |
| `tests/features/088-droits.feature` | M | Préserver reprise/autorisation/contrôle humain ; enlever seulement parcours UI/projet (T012/T029). |

## Fixtures et ressources des harnais

Les ressources ne sont pas des tests exécutés. Elles accompagnent leur lecteur ; leur retrait n'est permis que lorsque tous leurs lecteurs sont eux-mêmes R ou déjà portés.

| Famille historique | Disposition |
|---|---|
| `crates/bridget-daemon/tests/fixtures/mcp/`, `attach-*.jsonl`, `prompts/`, `registry/` | Conserver les octets/provenance ; adapter les seuls chemins d'exécutables synthétiques privés. |
| `crates/bridget-daemon/tests/fixtures/artifacts/` | Conserver contenus et provenance ; un artefact contenant du HTML n'implique pas de moteur HTML dans le noyau. |
| `crates/bridget-daemon/tests/fixtures/docker/`, `project-profile/` | Retirer avec les seuls tests projet ; secrets sont des fixtures synthétiques, ne pas importer de secrets réels. Porter les cas de droits nécessaires aux tests fournisseur avant coupe. |
| `crates/bridget-transport/tests/fixtures/acp/`, `journal/`, `provider-contracts/` | Conserver ; réserve explicite sur le canon Codex ci-dessus. |
| `plugins/maicie/tests/fixtures/coordination-active/service-frames-v1.jsonl` et `observation-oracles-v1.json` | Porter avec le consommateur public ; conserver la distinction octets 015 et oracles de fraîcheur 016. |
| Autres `plugins/maicie/tests/fixtures/`, y compris `migration-v19/base-v19-empty.sqlite3` | Métier Maicie hors produit. Ne pas utiliser cette base privée comme fixture de migration du store Bridget. |
| `apps/bridget-desktop/src-tauri/tests/fixtures/` | Retirer avec la coque/browser ; contenu communicable reste protégé dans artifact_store. |
| `specs/*/contracts/fixtures/` et spikes historiques | Sources documentaires de T002, pas une preuve de comportement exécuté ; garder la référence Git et le canon retenu dans le corpus 089. |

## Gate d'exhaustivité et état réel

Le contrôle d'inventaire doit reconstruire l'union de la section méthode **depuis le commit épinglé**, comparer son ensemble aux 264 lignes explicites des quatre tableaux (pas aux occurrences incidentes dans le texte), et refuser un fichier absent ou doublonné. Les douze tags `@SC_08901` à `@SC_08912` de /Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/tests/features/089-communication-core.feature sont associés au tableau de critères.

État de T003 : carte et scénarios écrits ; vérification statique de couverture uniquement. Aucun résultat fonctionnel nouveau. Les preuves d'exécution, durées, mutants et éventuelles limites de fournisseur/plateforme doivent être enregistrés par les tâches de réalisation, sans cocher les SC à partir de cette carte.
