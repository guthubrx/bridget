# T019 — Stockage durable des avertissements de contrôle

Date : 2026-10-06. Périmètre terminé : stockage et tests du stockage.
La garde daemon et le changement de fait projet appartiennent au noyau.
Aucun commit, fournisseur, installation ou daemon de production utilisé.

## Réutilisation et changement

La table existante `execution_control_commands` reste l'autorité des commandes.
La migration 11 ajoute seulement `project_warnings TEXT NOT NULL DEFAULT '[]'`.
Elle reprend le contrôle de colonne `pragma_table_info` des migrations existantes.
Elle s'exécute dans leur transaction immédiate et reste idempotente.
Aucune table, service, dépendance ou abstraction de migration n'est ajouté.

`mark_control_dispatched(issuer_scope, command_id, observed_at, project_warnings)`
écrit les avertissements prévalidés et l'état `dispatched` dans le même UPDATE.
La condition `state = 'prepared'` interdit leur remplacement après dispatch.
`StoredControlCommand.project_warnings` restitue le JSON durable lors du rejeu
et de la consultation. Les octets canoniques et les raisons de refus restent
dans leurs champs existants. Les anciens verdicts et dates ne sont pas réécrits.

Le coût supplémentaire de sérialisation/lecture est O(W), où W est le volume
des avertissements. Les accès gardent la clé primaire de commande existante.

Fichiers possédés et relus :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/execution_store.rs
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/execution_store_test.rs

## RED réel avant modification de production

Les deux tests de migration ont été ajoutés avant le code. Le noyau a exécuté
ce filtre dans son créneau Cargo isolé, avec BRIDGET_HOME, TMPDIR et socket privés :

```sh
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Volumes/8TB2/01-workflow/bridget-138-cargo.HjVvbG cargo test -p bridget-daemon --test execution_store_test spec_138_control_warnings -- --nocapture
```

Sortie relue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/core-control-store-red.log

```text
running 2 tests
spec_138_control_warnings_new_store_has_empty_default ... FAILED
spec_138_control_warnings_migrate_legacy_commands_idempotently ... FAILED
assertion `left == right` failed
  left: 10
 right: 11
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.01s
```

Code de sortie : 101. Le contrat de migration absent est constaté à l'exécution.
Le RED séparé de la garde daemon a aussi été capturé avant le changement d'API.
Le signal GO production a ensuite été reçu du noyau.

## GREEN réel après modification

Commande exacte exécutée par le propriétaire du stockage :

```sh
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet
umask 077
control_store_home=$(mktemp -d /tmp/b138cs.XXXXXX)
export TMPDIR="$control_store_home"
export BRIDGET_HOME="$control_store_home"
export BRIDGET_SOCKET="$control_store_home/s"
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/Volumes/8TB2/01-workflow/bridget-138-cargo.HjVvbG
cargo test -p bridget-daemon --test execution_store_test -- --nocapture
```

Le répertoire créé est /tmp/b138cs.4ulDef, permissions `drwx------` vérifiées.
Les tests de stockage ouvrent uniquement SQLite. Ils ne créent aucune socket
et ne lancent aucun daemon ou fournisseur. Les fichiers de test sont supprimés
par chaque test après fermeture des connexions.

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 18.28s
running 17 tests
test spec_138_control_warnings_new_store_has_empty_default ... ok
test commande_de_controle_rejouee_ne_declenche_pas_deux_actions ... ok
test spec_079_payload_absent_ferme_le_parent_sans_inventer ... ok
test migration_execution_store_est_additive_et_idempotente ... ok
test provider_binding_verifie_generation_thread_et_tour_sans_branche_cursor ... ok
test projection_par_agent_distingue_file_et_attente_durable ... ok
test spec_079_reconstruction_conserve_message_soumission_projet_et_lignee ... ok
test demarrage_sans_preuve_devient_injoignable_a_sa_borne ... ok
test spec_079_deux_actifs_refusent_une_reconstruction_aveugle ... ok
test admission_de_declenchement_ne_cree_pas_de_faux_element_de_file ... ok
test reference_projet_du_snapshot_survit_au_redemarrage_sans_reduction ... ok
test reprise_apres_crash_retrouve_les_executions_non_terminales ... ok
test migration_execution_store_garde_les_tables_heritees_et_rejoue_sans_effet ... ok
test transition_conditionnelle_refuse_un_evenement_tardif ... ok
test migration_execution_store_complete_les_lignes_heritees_sans_les_effacer ... ok
test spec_138_control_warnings_survive_restart_and_outcomes_without_rewriting_canon ... ok
test spec_138_control_warnings_migrate_legacy_commands_idempotently ... ok
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

Code de sortie : 0. Les 14 tests existants et les trois nouveaux tests passent.

## Assertions couvertes

- Base neuve : version 11, préparation/rejeu/dispatch sans avertissement rendent `[]`.
- Ancienne base : migration et réouverture idempotentes ; ancien canon, refus,
  raison et expiration conservés ; JSON et lectures historiques rendent `[]`.
- Avertissement Unicode : sérialisation exacte et consultation après réouverture.
- Issue inconnue `dispatched`, puis `accepted` et `refused` : mêmes avertissements.
- Second dispatch avec liste différente : UPDATE refusé, avertissements conservés.
- Rejeu canonique exact : résultat durable exact sans modification.
- Canon modifié : `EnvelopeMismatch`, résultat et octets anciens conservés.

La conservation après modification des faits de connexion est vérifiée par
les tests daemon du noyau. Ce rapport ne déclare pas ce résultat à leur place.
La vérification complète du workspace, clippy et release reste au principal.

## Relecture locale

Diff ciblé relu. `git diff --check` ciblé : code 0.
`rustfmt --edition 2024 --check` sur les deux fichiers possédés : code 0.
Minimalisme : réutilisation des lectures et de l'UPDATE existants ; aucun helper
ou wrapper supplémentaire. Potentiel minimalisme : 0 ligne identifiée à retirer
à comportement utile constant dans le code de production ajouté.
Le changement garde un contrat explicable : avertissement préparé, figé avant
l'effet, puis relu. Les tests utilisent de vrais fichiers SQLite et des canons
inchangés. La charge future reste bornée à une colonne et ses lectures.
