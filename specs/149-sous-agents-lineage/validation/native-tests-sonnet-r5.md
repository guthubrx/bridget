# Session149 - tests natifs r5 (Sonnet 5.5 high, claudeAgent)

Date : 2026-10-10. Aucun modèle réel, aucun service, aucune DB de production, aucun Git, aucun cochage, aucune release, aucune installation.
Aucune ligne de production modifiée. Les rapports r3 et r4 restent intacts.
Mission : tester les correctifs de Sol pour la recette R9.2 (mission en vol après redémarrage du daemon relancée par erreur).

## Résultat

| Mesure | Valeur |
|---|---|
| bridget-daemon (`cargo test -p bridget-daemon --offline --no-fail-fast`, 79 exécutables) | **1478 PASS / 0 FAIL / 61 ignored** |
| bridget-transport (10 exécutables, sources inchangées depuis r4) | **325 PASS / 0 FAIL / 2 ignored** |
| Total | **1803 PASS / 0 FAIL / 63 ignored** |
| Tests natifs ciblés (148 + 149 + lineage + permissions) | 78 PASS / 0 FAIL |
| Observer 149 en série (`--test-threads=1`) | 10 PASS / 0 FAIL |
| Écart avec r4 (1796) | +7 = 5 tests unitaires + 2 tests sur binaire réel, tous nouveaux |

Compilation n'est pas PASS : ces chiffres sont des tests exécutés.
Baseline « vieux 14819 PASS » : chiffre repris du brief r4. Cette ronde ne l'a pas recalculé.
Environnement : `umask 077`, `TMPDIR=/tmp/b149t-r5` (APFS, 0700, hors `/Users/moi`), cible Cargo externe, jobs 2, incrémental 0, offline.

## 1. Revue ciblée des correctifs de Sol

Lecture du code courant (modifié entre 15:06 et 15:08, avant mes tests) :

- `native_delegation.rs` `prepare_restart` (l.762) : `starting`, `mission_pending`, `working` passent `failed` / `unreachable`. Les résultats et `waiting_for_children` ne sont pas touchés. Les enfants `Running` passent `Stopped`. La fonction rend l'ensemble des enfants natifs.
- `daemon.rs` `reserve_managed_recoveries` (l.3892, 3914, 3946) : les deux boucles (baux actifs et flotte désirée) sautent les enfants natifs.
- `daemon.rs` `schedule_idempotent_delivery_recovery` (l.4197) : toute remise dont le message est une mission native passe `indeterminate`, sans rejeu. Un message illisible ou une erreur de lecture du store donne le même résultat (refus par défaut).
- `daemon.rs` `schedule_execution_recovery` (l.4252) : si l'exécution à reprendre est `execution-{mission}` de l'enfant, aucune continuation générique n'est créée.
- `wrapper.rs` (l.4488) : `persistent_relaunch` est faux dès que `BRIDGET_NATIVE_MISSION_BOOTSTRAP` est présent.

Aucun finding de production.

## 2. Tests ajoutés (tests seulement)

Fichier `crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs` (5 tests, 13/13 dans le module). Le harnais garde maintenant le lecteur de la connexion de contrôle ouvert (`fixture_with_reader`), pour qu'un `tick` puisse écrire au parent.

| Test | Chemin réel appelé | Ce qu'il prouve |
|---|---|---|
| `native149_redemarrage_echoue_les_missions_engagees_sans_relancer_ni_toucher_aux_durables` | admission, `spawn_task`, bail de flotte, redémarrage du superviseur, `reserve_managed_recoveries` | 3 missions engagées échouent `unreachable`. Seul l'agent ordinaire est repris. `queued`, `result_available` (résultat intact) et `waiting_for_children` restent identiques. Les enfants natifs passent `Stopped`. |
| `native149_bail_actif_d_un_enfant_natif_n_est_pas_un_candidat_de_reprise` | idem, sans redémarrer la flotte | Le bail natif est candidat (précondition vérifiée), mais n'est pas repris. L'ordinaire est repris. |
| `native149_apres_redemarrage_le_tick_admet_le_queue_et_ne_relance_aucune_mission_echouee` | restart puis `tick` | Un seul `Start`, pour la mission `queued`. Les 3 missions échouées ne reçoivent aucune nouvelle instance. |
| `native149_reconnexion_ne_rejoue_pas_la_remise_incertaine_d_une_mission_native` | `schedule_idempotent_delivery_recovery` | La remise de mission native passe indéterminée. La remise ordinaire est rejouée à l'identique. |
| `native149_reprise_d_execution_ignore_la_mission_native_et_garde_l_ordinaire` | `schedule_execution_recovery` | Témoin ordinaire : continuation créée. Mission native : aucune remise, exécution non marquée reprise, aucune continuation. |

Fichier `crates/bridget-daemon/tests/native_provider_death149_test.rs` (nouveau, 2 tests, binaire `bridget` réel, daemon réel, `managed-wrapper` réel, faux fournisseur ACP Python) :

| Test | Résultat observé |
|---|---|
| `native149_temoin_fournisseur_ordinaire_tue_est_relance` | Fournisseur persistant tué (SIGTERM) : 2 démarrages, agent de nouveau joignable. |
| `native149_fournisseur_natif_tue_n_est_jamais_relance` | Même scénario avec `BRIDGET_NATIVE_MISSION_BOOTSTRAP` transmis par `pass_env` : 1 seul démarrage pendant 5 s, groupe de processus du wrapper terminé, agent non joignable. |

Limite : la valeur du bootstrap du test ne correspond à aucun bail. La garde du wrapper ne regarde que la présence de la variable. Le test ne prouve donc pas la corrélation instance/mission du bootstrap (déjà couverte ailleurs).
Un premier essai du témoin a échoué par erreur de mon harnais (l'agent relancé est `busy` pendant le tour de la carte de reprise). J'ai corrigé le harnais ("joignable" = `connected` ou `busy`). Aucun oracle abaissé.

## 3. Preuve que les tests détectent la régression (mutations)

Méthode : copie jetable du code dans `/tmp/b149t-r5/mut` (supprimée), cible Cargo séparée (supprimée), un correctif de Sol annulé à la fois. L'arbre de travail n'a pas été modifié. Outil archivé : `native-r5-mutate.py`.

| Mutation | Résultat |
|---|---|
| M0 aucune mutation (copie fidèle) | 5/5 PASS |
| M1 `prepare_restart` n'échoue plus les missions engagées | ROUGE : redemarrage, tick |
| M2 reconnexion rejoue la remise native | ROUGE : reconnexion |
| M3 reprise d'exécution générique | ROUGE : reprise_d_execution |
| M4 flotte sans exclusion des natifs | ROUGE : bail_actif (une première série à 4 tests restait verte, d'où le 5e test) |
| M5 enfants natifs laissés `Running` | ROUGE : redemarrage |
| M4+M5 | ROUGE : redemarrage, tick, bail_actif |
| M6 wrapper relance aussi un enfant natif | ROUGE : test réel natif (témoin vert) |

Observation : sur le chemin « redémarrage », M4 seule ne change rien, parce que `prepare_restart` arrête déjà les enfants natifs (défense en profondeur). Le 5e test couvre le chemin « bail actif dans le même processus ».
Journaux : `native-r5-mutation-*.log`, `native-r5-mutations-summary.txt`, `native-r5-mutations-summary-2.txt`.

## 4. Limites et risques

- Les fixtures `claude` restent des copies de `/bin/echo`. Le faux fournisseur ACP est un script Python. Aucune preuve de comportement d'un modèle.
- Le redémarrage est simulé par réouverture du superviseur de flotte sur la même base, avec purge des tables mémoire. Ce n'est pas un `kill` du daemon. La recette réelle R9.2 reste à rejouer par un autre Sonnet avec le nouveau binaire.
- Les enveloppes illisibles ne peuvent pas être stockées par `begin_send_delivery` (refus à l'écriture). Je n'ai donc pas de test du cas « enveloppe corrompue » dans `schedule_idempotent_delivery_recovery`.
- Le test sur binaire réel dépend de temps (repli de 1 s, fenêtre de 5 s). Il a passé 2 fois sur 2 après correction du harnais (étape seule, batterie complète). Le témoin a passé une 3e fois dans la copie de mutation M6. Un flocon sous forte charge reste possible.
- Les fixtures de l'observer modifient le PATH du processus de test une seule fois. Le module passe en série (10/10).
- Aucune validation T036, T037, T038 ou T039 n'est déduite de ces tests unitaires.

## 5. Binaire debug réutilisable : DISPONIBLE (aucune installation)

| Champ | Valeur |
|---|---|
| Copie stable (nouvelle) | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-ec6b18b5d468` |
| SHA-256 | `ec6b18b5d468dd0031eb6282da10bdc27cb38b7193f07b4a8b9107cbe92d8b0f` |
| Taille / version | 54 023 592 octets, `bridget 0.1.3`, Mach-O arm64 |
| Signature | ad hoc (signée par l'éditeur de liens), `codesign -v` OK |
| Construit | 2026-10-10 15:45:43 +0200, `cargo build -p bridget-daemon --bins --offline`, journal `native-r5-build-debug.log` |
| Chemin cible (écrasé au prochain build) | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target/debug/bridget` |
| Empreinte sources (197 fichiers Rust/toml, tests inclus) | `e9b0eb9ffed3635b112dcb3b7d871bfdaf59e0d23c85f7f4817190b149950d22` |
| Empreinte sources production (111 fichiers) | `65403980d033e2859591f8b455be191a2e27ade809517c4f40c20a2c6930b049` (r4 : `e0d86f01...`) |
| Diff Git borné `git diff HEAD -- crates` | sha256 `572681855690f77c911965d58afa08db0e23fc893568cf281fecb905c766f8b5` (20 fichiers suivis, +802/-96) |
| Fraîcheur | empreinte identique avant et après le build ; aucun `.rs` ni `Cargo.*` plus récent que le binaire |
| État Git | arbre non committé : aucune prétention d'état propre |

Les fichiers `wrapper.rs`, `native_delegation.rs` et `daemon.rs` figurent avec leur hash dans le reçu, section `includesCorrectionsSolR9_2`.
Binaire r4 `bridget-823e8a5fab8a` : **IMMUTABLE vérifié** (même SHA-256 `823e8a5f...`, même date 14:08). Il est périmé pour R9.2 : il précède les correctifs de Sol. Les recettes en vol peuvent le finir. La recette R9.2 doit utiliser `bridget-ec6b18b5d468`.
Reçu machine : `.../validation/native149-debug-receipt.json`. Ancien reçu, copie identique : `.../validation/native149-debug-receipt-r4.json`.
Outil de calcul des empreintes : `.../validation/native-r5-fingerprint.py` (l'algorithme est décrit dans le reçu).

## 6. Fichiers modifiés par cette ronde (tests seulement)

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs` (+5 tests, harnais `fixture_with_reader`)
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/native_provider_death149_test.rs` (nouveau)

## 7. Journaux (même répertoire que ce rapport)

`native-r5-step1-new-recovery-1.log` (échec harnais conservé), `-2.log`, `native-r5-step2-provider-death-1.log` (échec harnais conservé), `-2.log`, `native-r5-step3-native-targeted.log`, `native-r5-final-daemon.log`, `native-r5-final-transport.log`, `native-r5-observer-serial.log`, `native-r5-build-debug.log`, `native-r5-mutation-*.log`.

## Espace disque

Cible externe : 3,6 Go (inchangé). Cible des mutations (1,2 Go) supprimée. Nouvelle copie du binaire : 54 Mo. `/tmp/b149t-r5` supprimé en fin de ronde.

## Pour le parent et Sol

Aucun finding actif pour Sol. Aucun bug de production nouveau.
Pour le parent : lancer la recette R9.2 avec `bridget-ec6b18b5d468`.
