# Carte des tests 089 — dispositions avant extraction

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
