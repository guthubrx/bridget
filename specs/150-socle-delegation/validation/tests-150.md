# Résultats de tests - Session150 (T019)

Date : 2026-10-11. Code : worktree `.worktrees/150-socle-delegation`, base `1f2c661d`,
changements T002 à T018 non commités. Recette : `umask 077`, racine `/tmp/b.XXXX`,
`env -i`, `BRIDGET_HOME` privé, `cargo test -p bridget-daemon --features test-support
--no-fail-fast <filtre> -- --test-threads=4`.

## Non-régression 148 / 149 / 087

| Filtre | Référence (baseline.md) | Maintenant | Échecs | Ignorés |
|---|---:|---:|---:|---:|
| native148 | 27 | 27 (23 + 1 + 3) | 0 | 1 |
| native149 | 78 | 78 (72 + 2 + 4) | 0 | 0 |
| delegation149 | 10 | 10 | 0 | 0 |
| spec_087 | 11 | 11 | 0 | 0 |
| **Total** | **126** | **126** | **0** | **1** |

Aucun test 148, 149 ou 087 ne passe de vert à rouge.

Deux tests existants ont été adaptés, avec leur raison :

- `delegation149_migration_fermee_cycle_profondeur_et_volume` : le volume de 4097 lignes
  est désormais accepté à l'ouverture (T014). Cycle et profondeur restent refusés.
- `native149_catalogue_avec_fait_ouvre_le_developpement_sans_grant_humain` : la liste
  fermée des clés du catalogue contient `mission_reply_min_secs` et `mission_reply_max_secs`
  (T018).

## Session 150

Après les tâches de convergence T025 à T035 (2026-10-11).

| Filtre | Réussis | Échecs |
|---|---:|---:|
| native150 (bridget-daemon, lib) | 46 | 0 |
| native150 (bridget-transport) | 1 | 0 |

Non-régression rejouée après T025 à T035 : native148 27 (23 + 1 + 3, 1 ignoré),
native149 78 (72 + 2 + 4), delegation149 10, spec_087 11 : 126 réussis, 0 échec,
1 ignoré, identique à la référence.

Détail par thème (daemon) :

| Thème | Tests |
|---|---:|
| purge (magasin, vivacité, pierres, accusés, lot de flotte, échec de recopie, ouverture sans purge, résync) | 23 |
| entretien : purge de borne sans admission | 1 |
| admission (5000 terminées, saturation, limites actives, purge de borne) | 4 |
| ouverture 10 000 lignes | 1 |
| champs de tâche | 1 |
| pause (dont droits, gestionnaire complet, annulation humaine) | 7 |
| délai (bornes, rejeu, enfant explicite, implicite, sans échéance parent, application, suivi aligné, schéma MCP) | 8 |
| statut d'une tâche purgée | 1 |
| **Total** | **46** |

Contrôle complémentaire (avant convergence) : `cargo test --lib mcp` donne 91 réussis,
0 échec, 2 ignorés.

## Non vérifié

- Suite complète du workspace (T023, à la charge du coordinateur).
- `run_maintenance` en entier : il touche `$HOME/Library/Logs`. Le test couvre
  `maintain_stores`, qui contient la recopie, la purge des envois et la rétention.
- Recette avec un vrai T3 ouvert sur la vue Lineage : seule la socket de test est utilisée.

## Suite complète du workspace (T023)

Date : 2026-10-11. Recette : `umask 077`, racine `/tmp/b.XXXX`, `BRIDGET_HOME`
privé, `cargo test --workspace --features bridget-daemon/test-support
--no-fail-fast -- --test-threads=4`.

| Réussis | Échecs | Ignorés |
|---:|---:|---:|
| 1969 | 8 | 72 |

Les 8 échecs, tous hors délégation :

- `core_089_contract_test` : `cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere`, `client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation`
- `core_089_crash_test` : `binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee`
- `core_089_identity_test` : `spec094_mcp_modifie_uniquement_sa_propre_presence`, `deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance`, `noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper` (message : `auxiliary_credential_required`)
- `core_089_reply_test` : `cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent`
- `t3code_098_test` : `spec098_pont_expose_remet_repond_renouvelle_et_retire`

**Preuve qu'ils sont préexistants** : les mêmes 5 binaires de test, rejoués
avec la même recette sur la base intacte `1f2c661d` (worktree détaché
temporaire, supprimé ensuite), donnent exactement les mêmes 8 échecs. Ils ne
viennent pas de la session150.

**Origine probable, non prouvée** : ces tests passaient à la livraison 149
(`072161262a7d`, journal `specs/149-sous-agents-lineage/validation/native-r9-*`).
Entre les deux, les seuls commits de code sont `0d5107af`, `8d3eff76`
(limite de fichiers), `90e1fab0` et `9f2ccbf2` (identifiants opaques et noms
de fichiers de reprise, travail « Complément149 identité durable » d'un autre
fil). Non corrigé ici : hors périmètre et propriété d'un autre fil.

## Suite complète après la convergence (T025-T035)

| Réussis | Échecs | Ignorés |
|---:|---:|---:|
| 1977 | 9 | 72 |

- 8 échecs : les mêmes que ci-dessus, préexistants sur la base `1f2c661d`.
- 1 échec sous charge : `daemon::presence_tests::stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels`
  (`BrokenPipe` à `daemon.rs:30091`). Rejoué seul 3 fois : 3 réussites. Classé
  sensible à la charge, pas régression.

## Convergence, passage 2 (T036-T039)

Recette obligatoire (umask 077, `BRIDGET_HOME` privé, `--test-threads=4`, `--no-fail-fast`).

| Filtre | Réussis | Échecs | Ignorés |
|---|---:|---:|---:|
| `native148` | 27 | 0 | 1 |
| `native149` | 78 | 0 | 0 |
| `delegation149` | 10 | 0 | 0 |
| `spec_087` | 11 | 0 | 0 |
| Total non-régression | 126 | 0 | 1 |
| `native150` | 50 | 0 | 0 |

- T036 : rouge vu (4095 restantes au lieu de 4172 : 205 arbres retirés dans un passage), corrigé dans `maintain_retention`.
- T038 : deux tests de comportement existant, verts du premier coup.
- T039 : rouge vu (suivi resté à l'échéance de création, écart de 30 s), corrigé par `align_request_deadline` appelée à chaque tick d'une mission `working`.
- Suite complète non rejouée dans ce passage.

