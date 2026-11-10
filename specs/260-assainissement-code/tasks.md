# Tâches 260 - Assainissement du code et du dépôt

Statut : In Progress. 73 tâches cochées sur 157 (voir `implementation.md` pour le détail et les lots bloqués). La mise en oeuvre est en cours : lots 0, 1 (sauf T024), 2, 3, 4, 12, 14, 13 (T137 partiel), 15, 17 traités.
Spécification : `spec.md`. Plan : `plan.md`. Règles : `contracts/regles-implementeur.md`. Portes : `decisions.md`. Démarrage et clôture : `quickstart.md`.

## Comment lire ce fichier

- Une ligne = une tâche = un commit (sauf mention). Le détail (fichier, symbole à chercher, commande exacte, résultat attendu, règle d'arrêt) est dans le guide du lot, section portant le numéro de la tâche.
- Cocher une case seulement après la vérification de la tâche. Après chaque commit : cocher, puis écrire l'entrée dans `implementation.md`.
- Une tâche arrêtée ou différée reste décochée. Écrire la raison dans `implementation.md`.
- Chercher par **symbole**, jamais par numéro de ligne (les numéros datent du commit `065f54f0`).
- Ordre : voir `plan.md`, « Ordre d'exécution ». Un lot bloqué par une porte ne bloque pas les autres.
- `[Dn]` : la tâche dépend de la porte `Dn` de `decisions.md`. Sans réponse, appliquer l'action par défaut et l'écrire dans le journal.

## Fondations - Lot 0 (guide : `quickstart.md`)

- [x] T001 [L0] Lire `contracts/regles-implementeur.md` et `decisions.md` ; créer `specs/260-assainissement-code/implementation.md` (sections Journal, Lots, Idées).
- [x] T002 [L0] Vérifier les outils (`rg`, `python3`, `gh`, `ruff`, `git`, `scripts/build.py`) et le disque (`C-DISQUE`, 12 Gio minimum).
- [x] T003 [L0] Ligne de base des tests : `C-TESTS` (`base`) puis `validation/base-echecs.txt` ; valide la recette d'environnement.
- [x] T004 [L0] Ligne de base des noms de tests : `validation/base-noms.txt`.
- [x] T005 [L0] Ligne de base de l'analyse : `validation/base-clippy.txt` (3 erreurs, environ 385 avertissements).
- [x] T006 [L0] Ligne de base du format : `validation/base-format.txt` (environ 120 fichiers), sans rien reformater.
- [x] T007 [L0] Committer journal et lignes de base ; intégrer à `main` (procédure « Fin de lot » de `quickstart.md`).

## Lot 1 - Filet automatique (guide : `guides/lot-01-filet-automatique.md`)

- [x] T008 [L1] `Makefile` : cibles `fmt`, `fmt-check`, `clippy`, `lint`. Vérifier `make -n lint`.
- [x] T009 [L1] `Cargo.toml` racine et 3 crates : `[workspace.lints]` avec `allow` documentés (dette du lot 9) ; `C-CHECK`.
- [x] T010 [L1] `.github/workflows/rust.yml` en observation (clippy, format, tests) ; valide la syntaxe par un passage réel.
- [x] T011 [L1] [D2] `native_lineage.rs` : `never_loop`, `while receiver.recv().is_ok()` devient `if` (première modification de code).
- [x] T012 [L1] `native_workspace154_tests.rs` : `from_mode(0)` devient `from_mode(0o000)`.
- [x] T013 [L1] `mobile_operations.rs` : `reversed_empty_ranges`, `#[allow]` commenté (plage vide voulue).
- [x] T014 [L1] Règle `collapsible_if` (110 cas) par correction automatique.
- [x] T015 [L1] Règles à correction automatique restantes, un commit par règle.
- [x] T016 [L1] `nonminimal_bool` et `bool_assert_comparison`, un commit par règle.
- [x] T017 [L1] Petites règles restantes de l'analyse.
- [x] T018 [L1] `zombie_processes` (5 cas).
- [x] T019 [L1] Analyse sans avertissement (`-D warnings`) puis bloquante localement et en CI.
- [x] T020 [L1] `.github/workflows/rust-audit.yml` (audit hebdomadaire, outil à version fixe).
- [x] T021 [L1] Ligne de base Linux : trois passages du job `tests`, `validation/base-linux-echecs.txt`.
- [x] T022 [L1] Documentation « Contrôles avant livraison » ; `python3 scripts/check-docs.py`.
- [x] T023 [L1] Porte du lot : `C-TESTS`, `C-ECHECS`, analyse à 0 ; bilan ; fin de lot.
- [ ] T024 [L1] [D16] Rendre les tests bloquants en CI si D16 est tranchée et la base Linux stable ; sinon « différé ».

## Lot 2 - Dépôt sans résidus (guide : `guides/lot-02-depot-sans-residus.md`)

- [x] T025 [L2] [D3] Constat : 345 fichiers suivis mais ignorés, 15 résidus de racine ; `validation/lot02-constat.txt`.
- [x] T026 [L2] [D5] `git mv` des 15 résidus vers `specs/archives/residus-racine-20261110/` après contrôle par `rg`.
- [x] T027 [L2] `.gitignore` : `/.wip-*` et `/watch_*`.
- [x] T028 [L2] [D3] `.gitignore` : commentaire qui liste les 4 exceptions historiques (rien n'est retiré).
- [x] T029 [L2] Inventaire des fichiers suivis de plus de 1 Mio, `validation/lot02-gros-fichiers.txt` (propositions seulement).
- [x] T030 [L2] Porte : `check-secrets.sh`, `check-docs.py`, 0 résidu à la racine ; bilan ; fin de lot.

## Lot 3 - Consignes allégées (guide : `guides/lot-03-consignes-allegees.md`)

- [x] T031 [L3] Constat sur `AGENTS.md` (taille, bornes du bloc des reçus) ; `validation/lot03-constat.txt`.
- [x] T032 [L3] Copier le bloc octet pour octet dans `specs/recus-de-session.md` ; `diff` identique.
- [x] T033 [L3] Remonter 8 consignes vivantes en règles permanentes dans `AGENTS.md`.
- [x] T034 [L3] Retirer le bloc d'`AGENTS.md` par motif ; moins de 20 Ko.
- [x] T035 [L3] Porte : taille, sections, `check-docs.py`, `check-secrets.sh` ; bilan ; fin de lot.

## Lot 4 - Tests hors des fichiers de production (guide : `guides/lot-04-tests-externalises.md`)

- [x] T036 [L4] Préparation : lignes de base présentes ; test à sec de `outils/extraire_module_tests.py` dans `/tmp`.
- [x] T037 [L4] `crates/bridget-daemon/src/daemon.rs` : 7 modules de tests (du plus grand numéro de ligne au plus petit).
- [x] T038 [L4] `crates/bridget-daemon/src/daemon/native_delegation.rs` : 3 modules.
- [x] T039 [L4] `crates/bridget-daemon/src/remote_worker.rs` : 1 module.
- [x] T040 [L4] `crates/bridget-daemon/src/mcp.rs` : 1 module.
- [x] T041 [L4] `crates/bridget-daemon/src/wrapper.rs` : 5 modules.
- [x] T042 [L4] `crates/bridget-transport/src/codex_app_server.rs` : 1 module.
- [x] T043 [L4] `crates/bridget-daemon/src/attach.rs` : 1 module.
- [x] T044 [L4] `crates/bridget-daemon/src/cli.rs` : 4 modules.
- [x] T045 [L4] `crates/bridget-transport/src/protocol.rs` : 3 modules.
- [x] T046 [L4] `crates/bridget-daemon/src/t3code.rs` : 1 module.
- [x] T047 [L4] `crates/bridget-daemon/src/delegation.rs` : 1 module.
- [x] T048 [L4] `crates/bridget-transport/src/claude_stream_json.rs` : 1 module.
- [x] T049 [L4] `crates/bridget-transport/src/acp.rs` : 1 module.
- [x] T050 [L4] `crates/bridget-daemon/src/delegation_mcp.rs` : 3 modules.
- [x] T051 [L4] `crates/bridget-daemon/src/daemon/mobile_operations.rs` : 1 module.
- [x] T052 [L4] `crates/bridget-daemon/src/registry.rs` : 1 module.
- [x] T053 [L4] Porte : `C-TESTS`, `C-ECHECS`, `C-NOMS` identiques ; tests en ligne restants sous 15 000 lignes ; bilan ; fin de lot.

## Lot 5 - Aiguillage découpé (guide : `guides/lot-05-aiguillage.md`)

- [ ] T054 [L5] Mesure de départ ; créer `daemon/wrapper_dispatch.rs` et `mod wrapper_dispatch;`.
- [ ] T055 [L5] Extraire le bloc `service_refusal` (environ 265 lignes).
- [ ] T056 [L5] Extraire le bloc `client_refusal` (environ 321 lignes).
- [ ] T057 [L5] Branches Send, ServiceRequest, RelaunchOrder, CancelRequest (un commit par branche).
- [ ] T058 [L5] Branches SpawnOrder, ServiceHello, ControlExecutionReported, DecommissionOrder.
- [ ] T059 [L5] Branches ArtifactPublish, Register, GuichetReply, CoordinationSubscribe.
- [ ] T060 [L5] Branches ClientHello, QueueEditRequest, ObservationRequest, Subscribe.
- [ ] T061 [L5] Branches DisplayNameSet, DeliveryRejected, ThreadRequest.
- [ ] T062 [L5] Branches DelegatedRuntimeEvent, NativePermissionFact, WaitAgentLinks.
- [ ] T063 [L5] Branches T3ThreadBindingFact, DaemonIdentityRequest, Subscribed.
- [ ] T064 [L5] Branches ObservedActivity, TurnState, Unsubscribe.
- [ ] T065 [L5] Branches DeliveryIndeterminate, ListRequests, AdoptStoppedOrder, DisplayNameResolve, DirectoryScoped.
- [ ] T066 [L5] Branches AttachRejected, JournalReady, LiveJournalFragment, Gap.
- [ ] T067 [L5] Contrôle : 37 appels `wrapper_dispatch::on_`, fonction sous 1500 lignes ; `validation/lot05-fin.txt`.
- [ ] T068 [L5] Porte : `C-TESTS`, `C-ECHECS`, `C-NOMS` ; bilan ; fin de lot.

## Lot 6 - `unsafe` justifiés (guide : `guides/lot-06-unsafe.md`)

- [ ] T069 [L6] `fsutil.rs` : `effective_uid()` et son test.
- [ ] T070 [L6] Remplacer les lectures `geteuid` du crate de transport.
- [ ] T071 [L6] Remplacer les lectures `geteuid` du crate du démon et de ses tests d'intégration.
- [ ] T072 [L6] Remplacer les variantes sur plusieurs lignes ; plus aucun `geteuid` hors `fsutil.rs`.
- [ ] T073 [L6] Commentaires `SAFETY` : `workspace_commit_view.rs`, `claude_interactive.rs`.
- [ ] T074 [L6] Commentaires `SAFETY` : `managed_process.rs`, `attach.rs`.
- [ ] T075 [L6] Commentaires `SAFETY` : `daemon.rs` et le dossier `daemon/`.
- [ ] T076 [L6] Commentaires `SAFETY` : crate `bridget-transport`.
- [ ] T077 [L6] Commentaires `SAFETY` : autres fichiers de `crates/bridget-daemon/src`.
- [ ] T078 [L6] Commentaires `SAFETY` : `crates/bridget-daemon/tests/`.
- [ ] T079 [L6] Lint `undocumented_unsafe_blocks` dans les lints du workspace ; contre-épreuve.
- [ ] T080 [L6] Porte : `C-TESTS`, `C-ECHECS`, `C-NOMS` (+1 test) ; base actualisée ; bilan ; fin de lot.

## Lot 7 - Concurrence observable (guide : `guides/lot-07-concurrence.md`)

- [ ] T081 [L7] `poison.rs` : trait `RecoverPoison`, compteur, 2 tests.
- [ ] T082 [L7] Relevé des sites `unwrap_or_else(|e| e.into_inner())` avant remplacement.
- [ ] T083 [L7] Remplacer dans `daemon.rs`.
- [ ] T084 [L7] Remplacer dans `native_delegation.rs` et `native_lineage.rs`.
- [ ] T085 [L7] Remplacer dans `codex_app_server.rs`, `acp.rs`, `claude_stream_json.rs`.
- [ ] T086 [L7] Remplacer dans `wrapper.rs` et `fleet.rs`.
- [ ] T087 [L7] Remplacer dans les fichiers restants ; `C-CRATE` des deux crates.
- [ ] T088 [L7] Câbler le compteur de connexions et son maximum (garde de sortie), 1 test.
- [ ] T089 [L7] Mesure de la durée du tour de délégation et trace au-delà du seuil.
- [ ] T090 [L7] [D7] Regroupement des requêtes du tour : fermée tant que D7 n'est pas tranchée ; note de mesure.
- [ ] T091 [L7] Porte : `C-TESTS`, `C-ECHECS`, `C-NOMS` (+3 tests) ; base actualisée ; bilan ; fin de lot.

## Lot 8 - Erreurs sans texte fragile (guide : `guides/lot-08-erreurs.md`)

- [ ] T092 [L8] Inventaire des comparaisons de texte et classement A/B/C/D ; `validation/lot08-classement.md`.
- [ ] T093 [L8] Niveau 1 : constantes partagées, `native_delegation.rs`, `delegation_mcp.rs`, `native_permission_change.rs`.
- [ ] T094 [L8] Niveau 1 : `codex_app_server.rs`, `attach.rs`, `cli.rs`, `communication/client.rs`.
- [ ] T095 [L8] Niveau 1 : `daemon.rs`, `wrapper.rs`, `mcp.rs`, `reaper.rs`, `registry.rs`, autres.
- [ ] T096 [L8] Classe B : commentaire qui nomme la source du texte externe.
- [ ] T097 [L8] [D13] Niveau 2 (types d'erreur) : fermée sans liste validée par François ; liste candidate écrite.
- [ ] T098 [L8] Porte : `C-TESTS`, `C-ECHECS`, `C-NOMS` ; contrôle des chaînes durables ; bilan ; fin de lot.

## Lot 9 - Fonctions lisibles (guide : `guides/lot-09-fonctions.md`)

- [ ] T099 [L9] Constat : `outils/longues_fonctions.py`, fichiers `validation/lot09-depart-*.txt`.
- [ ] T100 [L9] Structure d'arguments : `prompt_request` (21 paramètres).
- [ ] T101 [L9] Structure d'arguments : `handle_register_with_channel` (17).
- [ ] T102 [L9] Structure d'arguments : `resolve_target` (17).
- [ ] T103 [L9] Structures d'arguments : `reconnect_managed_session`, `connect_and_register_at`, `connect_and_register_with_domain_at`.
- [ ] T104 [L9] Structures d'arguments : `post` (threads), `apply` (réparation d'espace), `spawn_reader` (flux JSON Claude).
- [ ] T105 [L9] Découper `launch_session_with_status` (1371 lignes) sous 1000.
- [ ] T106 [L9] Découper `spawn_reader` de `codex_app_server.rs` (1101 lignes) sous 1000.
- [ ] T107 [L9] Réduire `tick` (983 lignes) sans déplacer de verrou, sinon arrêt.
- [ ] T108 [L9] Réduire `launch` (796 lignes).
- [ ] T109 [L9] Réduire `admit_delegate` (777 lignes).
- [ ] T110 [L9] Réduire `apply_specialist_inner` (727 lignes).
- [ ] T111 [L9] Réduire `handle_idempotent_send_with_result_authority` (625 lignes).
- [ ] T112 [L9] Exemptions (`tools`, deux `init_schema`), commentaire des fonctions à 12-14 paramètres, `validation/lot09-restes.md`.
- [ ] T113 [L9] Porte : `C-TESTS`, `C-ECHECS`, `C-NOMS` ; aucune fonction de 1000 lignes ou plus ; bilan ; fin de lot.

## Lot 10 - Format uniforme (guide : `guides/lot-10-format.md`)

- [ ] T114 [L10] [D9] Porte D9, dépôt calme, lots 4, 5, 9 intégrés ; sinon « différé ».
- [ ] T115 [L10] `cargo fmt --all` en un commit isolé ; `C-CHECK`.
- [ ] T116 [L10] `.git-blame-ignore-revs` avec le SHA du commit de formatage.
- [ ] T117 [L10] Rendre le contrôle de format bloquant (CI et `make lint`).
- [ ] T118 [L10] Porte : contrôle de format à 0 fichier ; `C-TESTS`, `C-ECHECS`, `C-NOMS` ; bilan ; fin de lot.

## Lot 11 - Modules nommés par rôle (guide : `guides/lot-11-modules.md`)

- [ ] T119 [L11] [D10] Porte D10, dépôt calme, lot 10 intégré ; `validation/lot11-renommages.txt`.
- [ ] T120 [L11] Renommer les modules 1 à 6 de la table.
- [ ] T121 [L11] Renommer les modules 7 à 12.
- [ ] T122 [L11] Renommer les modules 13 à 18.
- [ ] T123 [L11] Contrôle des restes (anciens noms) ; `validation/lot11-restes.txt`.
- [ ] T124 [L11] Porte : `C-ECHECS` et `C-NOMS` avec lignes de base transformées ; bases actualisées ; bilan ; fin de lot.

## Lot 12 - Stockage documenté (guide : `guides/lot-12-stockage.md`)

- [x] T125 [L12] Inventaire des mécanismes de version, ouvertures et modes de journal ; `validation/lot12-inventaire-stockage.md`.
- [x] T126 [L12] ADR proposé `docs/decisions/260-stockage-versions-schema.md` (statut Proposé).
- [x] T127 [L12] Porte : aucun fichier de code modifié ; `check-docs.py` ; bilan ; fin de lot.

## Lot 13 - Scripts contrôlés (guide : `guides/lot-13-scripts.md`)

- [x] T128 [L13] Constat : ruff, `bash -n`, scripts sans `set -e`.
- [x] T129 [L13] `ruff.toml` avec règles ignorées justifiées (S101, S603, S607).
- [x] T130 [L13] F401 (imports inutiles) et B904 (chaînage des exceptions).
- [x] T131 [L13] B905 (Python 3.9), B007, B023.
- [x] T132 [L13] S110, S108, S105, S606, S324 : justifications sans changement de comportement.
- [x] T133 [L13] [D11] `scripts/sentinel.py` : commentaire d'invariant et `# noqa: S102` sur l'`exec`.
- [x] T134 [L13] `scripts/update.py` S310 et restes ; `py_compile` sans exécution.
- [x] T135 [L13] `set -eu` dans les 2 scripts de test shell sans référence.
- [x] T136 [L13] Documenter l'absence de `set -e` des 4 scripts vivants.
- [ ] T137 [L13] Cible `lint-scripts`, job CI `scripts`, porte du lot ; bilan ; fin de lot.

## Lot 14 - Tests ignorés expliqués (guide : `guides/lot-14-tests-ignores.md`)

- [x] T138 [L14] [D14] Inventaire des `#[ignore]` sans motif (8 attendus).
- [x] T139 [L14] Écrire un motif sur chacun, sans exécuter les tests.
- [x] T140 [L14] Lint `ignore_without_reason` ; contre-épreuve ; porte ; bilan ; fin de lot.

## Lot 15 - Proposition de signature (guide : `guides/lot-15-signature.md`)

- [x] T141 [L15] [D8] Rédiger `validation/lot15-proposition-signature.md` (constat, options A/B/C, décision demandée).
- [x] T142 [L15] Porte : rien hors `specs/` modifié ; D8 reste en attente ; bilan ; fin de lot.

## Lot 16 - Écrans Android séparés (guide : `guides/lot-16-android.md`)

- [ ] T143 [L16] Constat Gradle, ligne de base des tests Android ; arrêt si Gradle ne fonctionne pas.
- [ ] T144 [L16] `Texts.kt` : 11 fonctions de texte.
- [ ] T145 [L16] `ConversationScreen.kt`.
- [ ] T146 [L16] `SettingsScreen.kt`.
- [ ] T147 [L16] `AgentDirectoryScreen.kt`.
- [ ] T148 [L16] `Panels.kt`.
- [ ] T149 [L16] [D15] `MobileViewModelPure.kt` : fonctions pures seulement (15 au plus).
- [ ] T150 [L16] Porte : imports, `MainActivity.kt` sous 300 lignes, compilation et tests ; bilan ; fin de lot.

## Lot 17 - Code mort (guide : `guides/lot-17-code-mort.md`)

- [x] T151 [L17] Constat des 9 `allow(dead_code)`.
- [x] T152 [L17] Traiter les attributs hors `attach.rs` (retrait, `cfg(test)` ou `expect` avec raison).
- [x] T153 [L17] [D12] `attach.rs` : `expect` avec la raison « boucle T806b » ; rien n'est supprimé.
- [x] T154 [L17] Porte : plus d'`allow(dead_code)` ; `C-CLIPPY`, `C-TESTS`, `C-ECHECS`, `C-NOMS` ; bilan ; fin de lot.

## Clôture (guide : `quickstart.md`)

- [ ] T155 [L-] Critères SC01 à SC14 : mesure, lot, preuve (ou « non atteint » avec raison) dans `implementation.md`.
- [ ] T156 [L-] Contrôle des portes D1 à D16 sur `origin/main` : tranchée ou action par défaut appliquée.
- [ ] T157 [L-] Nettoyage des worktrees et branches `260-*`, disque, statut final du programme.

## Dépendances

| Tâche ou lot | Dépend de |
|--------------|-----------|
| Tout lot de code | T003 à T007 (lignes de base) |
| Lots 2 à 17 (sauf 2, 3, 12, 14, 15) | Lot 1 fini jusqu'à T009 au moins |
| Lot 4 | T036 (outil validé) |
| Lot 5 | Lot 4 intégré |
| Lot 9 | Lots 4 et 5 intégrés |
| Lot 10 | D9 tranchée ; lots 4, 5, 9 intégrés ; dépôt calme |
| Lot 11 | D10 tranchée ; lot 10 intégré ; dépôt calme |
| T024 | D16 et T021 |
| T090, T097, T114, T119, T141, T149, T153 | portes D7, D13, D9, D10, D8, D15, D12 |
| T155 à T157 | tous les lots traités ou différés |

Une preuve absente conserve la tâche ouverte et le statut In Progress.
