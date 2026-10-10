# Tests natifs 149 - ronde Sonnet r8 (F2 réseau et O4)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 high. Aucun modèle réel, aucun Git, aucune installation, aucun redémarrage, aucun fichier de production touché.
Racine : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`.
Dossier de preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation`.
Environnement : `CARGO_TARGET_DIR=/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target`, `CARGO_BUILD_JOBS=2`, `CARGO_INCREMENTAL=0`, `--offline`, `TMPDIR=/tmp/b149t-r8` (0700), `umask 077`. Outil : `native-r8-tool-env.sh`.

## Verdict

- **Tous les oracles F2 et O4 demandés sont couverts par des tests qui passent.** Aucune erreur de production trouvée. Rien n'a été corrigé.
- **Suites complètes : 1825 PASS, 0 FAIL, 63 ignorés.** Daemon 1492 PASS / 0 FAIL / 61 ignorés (80 exécutables). Transport 333 PASS / 0 FAIL / 2 ignorés (12 exécutables). Deux exécutions distinctes, agrégées. r7 : 1806 PASS. Écart : +19 tests neufs, aucun test existant modifié.
- **Binaires privés debug et release reconstruits** depuis la dernière source. Ils ne remplacent aucun ancien binaire. Aucune installation.
- **Mutations** : 15 mutations de production tuées. 3 survivantes, toutes expliquées plus bas (aucune ne cache un défaut connu).
- **Cargo est libéré.** Aucun processus `cargo` ou `rustc` ne tourne. Aucun fournisseur ni wrapper de fixture ne reste.

## Changements de source

Production : aucun fichier modifié par le testeur. L'empreinte de production est identique au début et à la fin de la ronde.

| Fichier de production (corrections de Sol depuis r6) | sha256 r6 | sha256 r8 |
|---|---|---|
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | `5da3619e…` | `ea4e4316…` |
| `crates/bridget-daemon/src/daemon.rs` | `04d5aca3…` | `7815b199…` |
| `crates/bridget-daemon/src/wrapper.rs` | `e838a821…` | `28e92599…` |
| `crates/bridget-transport/src/codex_app_server.rs` | `4a36668c…` | `b9b68ad5…` |
| `crates/bridget-transport/src/managed_session.rs` | `4803530b…` | `42dfa0a2…` |

Ce sont exactement les cinq fichiers libérés par Sol. Aucun autre fichier de production n'a changé depuis r6. `Cargo.toml` et `Cargo.lock` sont inchangés (pas de `cargo update`, pas de dépendance neuve).

Tests ajoutés (19) :

| Fichier | Tests |
|---|---|
| `crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs` (ajout en fin de fichier ; le début est octet-identique à r6, sha `ea25851d…`) | 10 |
| `crates/bridget-daemon/tests/native_wrapper_stop149_test.rs` (nouveau) | 4 |
| `crates/bridget-transport/tests/native_stop149_test.rs` (nouveau) | 5 |

Mise en forme : `rustfmt` a été appliqué aux deux fichiers neufs et à la seule région ajoutée du fichier partagé. Le reste de ce fichier n'est pas formaté par `rustfmt` et n'a pas été touché.

Empreintes (algorithme r5, outil `hash-latency149-tool-fingerprint.py`) :
- Tout : `1b8ed0269943968a12a1a859776f7bf938da1353cf66fcd6e2a7fe4c10dbe4f9`, 200 fichiers (r7 : `d44190ab…`, 198).
- Production : `3943009ca82db59d850d913b55b18c7a0a17c91c3143ffaeb39ee6cdd600526d`, 111 fichiers (r7 : `6cc9a2be…`). La différence vient des cinq fichiers ci-dessus.
- Stables avant et après les builds (`native-r8-fingerprint-before-build.json` et `native-r8-fingerprint-after-build.json`).

## Ce que chaque test prouve

### F2 - redémarrage, descendants, résultat retenu (daemon)

Le banc utilise l'état réel du daemon, la flotte réelle, le store d'exécution réel. Il passe par `reserve_managed_recoveries` (donc `prepare_restart`) puis `tick`, comme la production.

| Test | Oracle |
|---|---|
| `native149_f2_descendant_perdu_en_running_est_ferme_puis_la_racine_livre_son_vrai_resultat_une_fois` | État F2 reproduit : descendant `failed/unreachable`, exécution `running`, racine bloquée (`descendants_busy` vrai, précondition). Après redémarrage : exécution fermée une seule fois par CAS (révision +1, génération inchangée), racine débloquée, `result_available`, **une seule remise** du vrai résultat, une seconde passe n'en ajoute aucune. F1 : seule la tâche `queued` est lancée, le descendant garde son instance et son état. |
| `native149_f2_reponse_reelle_capturee_survit_au_redemarrage_et_au_descendant_perdu` | Même chaîne sans écrire l'état à la main : l'enfant racine répond sur sa vraie connexion (`capture_reply`), il meurt, son descendant est perdu, le résultat capturé est livré une fois, enfant mort, sans aucun lancement. |
| `native149_f2_execution_terminale_n_est_jamais_rouverte_au_redemarrage` | `completed`, `failed`, `unreachable` gardent état, révision et génération. Témoin : un `running` voisin est fermé. Un second redémarrage ne réécrit rien. |
| `native149_f2_execution_d_un_autre_agent_est_refusee_sans_etre_modifiee` | Exécution adressée à un autre agent : `native_execution_mismatch`, exécution intacte. |
| `native149_f2_cas_d_execution_refuse_un_instantane_perime` | Contrat du store : état, révision ou génération périmés donnent `Rejected` et laissent l'exécution intacte. Témoin : l'instantané exact est appliqué. |
| `native149_f2_mission_engagee_est_fermee_avec_son_execution_et_l_ordinaire_reste_intact` | Nouveau chemin (tâche `working`) : mission et exécution échouent ensemble. L'exécution d'un agent ordinaire n'est pas touchée et reste récupérable. Témoin négatif : un descendant lié actif bloque toujours la racine, même après `prepare_restart`. |
| `native149_f2_ligne_de_reprise_d_une_mission_native_perdue_ne_bloque_pas_la_racine` | Ancienne interruption de contrôle : la ligne de reprise ne bloque pas une mission native perdue. Témoin : même ligne sur une mission encore active, racine bloquée. |
| `native149_f2_proprietaire_hors_ligne_garde_le_resultat_en_attente_puis_livre_une_seule_fois` | Propriétaire hors ligne : l'attente des descendants est levée, état `result_available`, `result_sent` reste faux, erreur `native_result_owner_offline`, aucune trame. Reconnexion : **une** remise. Passe suivante : aucune. Écriture de `result_sent` perdue : le rejeu retrouve la clé d'idempotence, aucune remise de plus, `result_sent` rétabli. |
| `native149_f2_autorite_retenue_recoupe_chaque_champ_du_resultat` | Témoin positif puis refus pour : corps, corps vide, expéditeur, destinataire, identifiant, corrélation (changée et absente), réponse attendue, intention, connexion, nom, instance, portée d'émetteur, connexion non auxiliaire, tâche non publiée, résultat durable différent, autre instance propriétaire, propriétaire hors ligne. Un message refusé **ne réserve rien** (le bon message passe ensuite, pas d'`EnvelopeMismatch`). Propriétaire révoqué : refus `native_result_authority_revoked`, même sur rejeu. |
| `native149_f2_message_externe_au_meme_format_ne_recoit_aucune_capacite_privee` | Garde par défaut inchangé : sans capacité, même le message exact de la saga est refusé (« non attestée »). Un agent attesté ne peut pas usurper l'enfant. `capture_reply` refuse ce format. Rien n'est réservé : la capacité de la saga passe ensuite. |

### O4 - arrêt du wrapper natif et du fournisseur

`native149_*` dans `native_wrapper_stop149_test.rs` : vrai binaire `bridget managed-wrapper`, faux daemon, faux fournisseur ACP lancé dans **son propre groupe** (`setsid`). Cela reproduit O4. Le tour du fournisseur reste ouvert (`notify_timeout_secs` 600) : sans signal, le wrapper ne sortirait pas.

| Test | Oracle |
|---|---|
| `native149_sigterm_du_wrapper_natif_arrete_le_fournisseur_sans_reconnexion` | SIGTERM sur le seul wrapper : sortie **coopérative** (pas tué par le signal), fournisseur d'un autre groupe arrêté, aucune reconnexion. Mesure : sortie 81 ms après le signal. |
| `native149_eof_du_daemon_arrete_le_wrapper_natif_et_son_fournisseur` | EOF du daemon : wrapper terminé, fournisseur arrêté, aucune reconnexion. |
| `native149_temoin_ordinaire_eof_se_reconnecte_et_garde_son_fournisseur` | Témoin : sans bootstrap natif, le même EOF fait reconnecter le wrapper (plusieurs `Register`), fournisseur gardé. |
| `native149_temoin_ordinaire_sigterm_garde_la_disposition_par_defaut` | Témoin : sans bootstrap, SIGTERM tue le wrapper par le signal (aucun gestionnaire installé). |

`native149_*` dans `native_stop149_test.rs` : faux app-server Codex (script Python en stdio). Il note chaque trame, répond à `turn/interrupt` (terminal du tour d'abord, réponse RPC 0,6 s plus tard) et note son arrêt.

| Test | Oracle |
|---|---|
| `native149_stop_native_interrompt_le_tour_exact_puis_arrete_le_fournisseur` | Une seule interruption, avec le bon `threadId` et le bon `turnId`, **avant** la fermeture du canal et le signal d'arrêt. Fournisseur arrêté. Plus de travail accepté. |
| `native149_stop_native_est_idempotent` | Rappels (natif x2, puis `stop()`) sans blocage, sans nouvelle trame, une interruption au total. |
| `native149_stop_native_ferme_l_admission_et_ne_demarre_aucun_message_en_file` | Un message en file ne démarre jamais (un seul `turn/start`) même pendant l'intervalle de réponse. `deliver` après l'arrêt échoue. |
| `native149_stop_native_d_une_autre_mission_n_interrompt_pas_le_tour_actif` | Identifiant de mission étranger : aucune interruption du tour d'une autre mission. |
| `native149_temoin_stop_ordinaire_n_envoie_aucune_interruption` | Témoin : `stop()` ordinaire garde sa sémantique en mode stdio (aucune interruption). |

## Résultats des suites

| Commande | PASS | FAIL | Ignorés |
|---|---|---|---|
| `cargo test -p bridget-transport --offline --no-fail-fast` (12 exécutables) | 333 | 0 | 2 |
| `cargo test -p bridget-daemon --offline --no-fail-fast` (80 exécutables) | 1492 | 0 | 61 |
| **Total** | **1825** | **0** | **63** |
| Vecteurs SHA-256 (transport) | 3 | 0 | 0 |
| Observer 149 en série (`--test-threads=1`) | 10 | 0 | 0 |
| Natifs 149, lib, filtre `native149` | 55 | 0 | 0 |
| Démarrage R9, lib, filtre `native_delegation_permissions149` | 23 | 0 | 0 |
| Mort du fournisseur 149 (vrai binaire, vrai daemon) | 2 | 0 | 0 |
| Wrapper natif 149 (nouveau) | 4 | 0 | 0 |
| Arrêt Codex 149 (nouveau) | 5 | 0 | 0 |

Répétitions contre les flocons : F2 x15 (15/15 PASS), arrêt Codex x8 (8/8), wrapper natif x4 (4/4). Le flocon `capabilities_integration_test` de r6 n'est pas revenu (corrigé en r7).

Journaux : `native-r8-final-transport.log` (sha `ff26f65e…`), `native-r8-final-daemon.log` (sha `580fd6c9…`), `native-r8-vectors.log`, `native-r8-observer-serial.log`, `native-r8-native149.log`, `native-r8-r9-startup-lib.log`, `native-r8-provider-death.log`, `native-r8-wrapper-stop.log`, `native-r8-transport-stop.log`, `native-r8-repetitions.log`.

## Mutations (oracles de sensibilité)

Méthode : copie jetable `/tmp/b149t-r8/mut` (supprimée), cible Cargo **séparée** (`bridget149-target-mut`, supprimée). Un motif unique est remplacé par mutation. L'arbre de travail n'est jamais modifié. Outil : `native-r8-mutate.py`. Résumés : `native-r8-mutations-summary.txt` (F1 à F10 valides ; ses lignes O, T2 et T4 sont périmées, banc moins strict), `-2.txt` (F10, O1 à O4, T4 avec le banc final) et `-3.txt` (T0 à T3 avec le faux serveur réactif) ; un journal par mutation `native-r8-mutation-*.log`.

| Id | Mutation | Tests qui échouent |
|---|---|---|
| F1 | `prepare_restart` ne ferme plus l'exécution | 5 (descendant perdu, capture réelle, hors ligne, mission engagée, terminal) |
| F2 | `prepare_restart` rouvre aussi les terminaux | terminal |
| F3 | exécution d'un autre agent acceptée | mauvais agent |
| F4 | autorité sans contrôle du corps | autorité retenue |
| F5 | autorité sans contrôle du destinataire | autorité retenue |
| F6 | autorité sans contrôle de révocation | autorité retenue |
| F7 | propriétaire hors ligne : faux `result_sent` | hors ligne |
| F8 | garde expéditeur par défaut relâché | message externe |
| F9 | autorité sans contrôle du propriétaire vivant | autorité retenue |
| F10 | `descendants_busy` ignore les missions fermées | ligne de reprise (test ajouté après un premier survivant) |
| O1 | wrapper sans gestionnaire SIGTERM | SIGTERM natif |
| O3 | wrapper natif se reconnecte sur EOF | EOF natif |
| T1 | arrêt Codex sans `turn/interrupt` | tour exact, idempotence |
| T2 | arrêt Codex sans fermeture d'admission | drainage (test durci après un premier survivant, voir plus bas) |
| T3 | arrêt Codex interrompt un autre tour | mission étrangère |

Témoins sans mutation (F0, T0) : tous les tests passent.

### Survivantes (3)

- **O2 - retrait de `transport.stop_native_mission` dans le wrapper.** Équivalent dans ce banc. `transport.stop()` suit juste après `relay.shutdown()`. Seul le délai de drain du relais change. Un wrapper ACP de fixture n'a pas de relais. Le délai orphelin de 6,9 s de r2 n'est donc pas mesuré ici.
- **O4 - retrait du contrôle du signal juste après `read_line`.** Équivalent dans ce banc. Le contrôle en tête de boucle sort au tour suivant (lecture à délai de 1 s). Seule une trame arrivée en même temps que le signal change.
- **T4 - `stop_native_mission` par défaut sans `cancel_delivery`.** Non observable de façon déterministe. `AcpTransport::shutdown()` écrit déjà `session/cancel` avant de tuer le fournisseur.

### Deux premiers survivants corrigés dans les tests

- **F10** : la branche `closed_native` n'était pas atteinte. Test ajouté (ligne de reprise `control_pause_interruptions`).
- **T2** : le faux serveur Codex dormait pendant l'intervalle de réponse, donc il ne notait pas le `turn/start` du message en file. Il répond maintenant dans un fil et reste réactif. La mutation est tuée.

## Alerte de méthode : cible Cargo partagée

Pendant la ronde, j'ai exécuté des mutations dans une copie qui partageait la cible Cargo de l'arbre de travail. Elles ont écrasé `target/debug/bridget`. Trois lancements du test SIGTERM dans l'arbre de travail ont alors échoué : ils utilisaient un binaire muté (sans gestionnaire de signal). Ce n'était pas un défaut du code.

Diagnostic : exécution sans signal (le wrapper sortait seul après environ 7 s ; cause probable : l'échéance de 6 s du tour fournisseur, non prouvée à part), puis `sample` sur le wrapper, puis rejeu dans la copie. Corrections :
1. Cible Cargo de mutation séparée.
2. `cargo clean -p bridget-daemon` dans la cible principale, puis reconstruction depuis l'arbre de travail.
3. `notify_timeout_secs` à 600 dans le banc (le tour ouvert ne ferme plus le wrapper seul) et assertion de délai (< 8 s).

Les binaires finaux viennent de cette reconstruction propre. Ils contiennent le code r8 (chaînes `mission native arrêtée avant le lancement`, `native_execution_mismatch`, `native_result_authority_revoked` présentes). Aucun fichier de production plus récent que les binaires.

## Binaires privés

Aucun n'est installé. Mode 0700. Signature ad hoc (`codesign -v` OK). `bridget 0.1.3`, Mach-O arm64.

| Build | Copie stable | SHA-256 | Taille |
|---|---|---|---|
| debug r8 | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-833a030545c3` | `833a030545c324339013486d2ed7056ad7aec892bac38765f35b4ae8bb6af8a6` | 54 076 872 |
| release r8 (candidat privé) | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb` | `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6` | 17 490 160 |

Commandes : `cargo build -p bridget-daemon --bins --offline` (20,5 s) et `cargo build --release -p bridget-daemon --bins --offline` (77 s). Journaux : `native-r8-build-debug.log`, `native-r8-build-release.log`.

Anciens binaires immuables, SHA relus après la ronde : `bridget-620e729fca53` (debug r6), `bridget-ec6b18b5d468` (debug r5), `bridget-823e8a5fab8a`, `bridget-ed5e28bc8ddc`, `bridget-abfb346e23cc` (release r6). Aucun n'est remplacé. Aucun `target/release` à la racine du dépôt. Aucun binaire installé touché.

Reçus : `native149-debug-receipt-r8.json` et `native149-release-receipt-r8.json`. Les reçus r6 et r7 restent en place.

Pour les recettes réelles : utiliser le **release r8** pour un parent Claude lancé en direct (le hook de hachage est rapide en release).

## Archivage

48 journaux bruts des rondes r3 à r5 sont déplacés vers `archive-r8-logs-r3-r5/`. L'index `archive-r8-logs-r3-r5.index.json` donne nom, taille et SHA-256 de chaque fichier. Les rapports `.md`, les reçus `.json`, les outils et les journaux r6 à r8 restent en place. Les anciens rapports citent encore les noms d'origine : lire l'index pour retrouver un fichier.

## Limites (non vérifié)

- Aucun modèle réel. Les fixtures `claude` et ACP sont des scripts. **Rien ne prouve T036 à T039.**
- Mode interactif Codex (socket WebSocket, TUI) non simulé. `stop_native_mission` y est le même code, mais le drainage gracieux de l'app-server réel n'est pas mesuré.
- Le chemin `native_execution_changed` de `prepare_restart` (CAS refusé en cours d'appel) exige un écrivain concurrent. Je teste le contrat du store, pas ce chemin.
- « ACK » côté wrapper (accusé de remise d'un tour) n'est pas rejoué ici. Je teste l'idempotence durable de la clé (rejeu sans seconde remise).
- Le `tick` ne s'exécute qu'avec les connexions de fixture : pas de socket réseau entre le daemon et le propriétaire dans les tests F2.
- Pas de `clippy` lancé. Pas de `cargo fmt` global. L'arbre Git n'est pas propre (travail non commité) : aucun reçu ne prétend le contraire.
- La charge machine était élevée pendant la ronde (load average 20 à 30).

## Pour Sol et le principal

Aucune erreur de production à signaler. Deux remarques, sans action demandée :
1. Le wrapper natif sort en 81 ms après SIGTERM dans le banc. La marge sur le plafond de 8 s est large.
2. Le fichier partagé `native_delegation_permissions149_tests.rs` n'est pas formaté par `rustfmt` (avant mon ajout). À traiter hors de cette ronde si le principal le veut.
