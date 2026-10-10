# Revue finale 149 (T041) - Sonnet 5.5 high

## Complément final du principal (2026-10-10)

Ce complément prévaut sur les verdicts datés ci-dessous. Le corps de la revue reste historique.

- **Sources :** SOURCE_ONLY_APPROVE (`validation/native-restart-final-source-review-sonnet-r2.md`), sur la production b2b87458 (111 fichiers). Le libellé du journal T3 (sha4abac...) est approuvé.
- **Tests T3 :** `validation/t3-final-boundary-tests-sonnet-r2.md` (262 tests, 16 fichiers, G3 et G5 fermés). Le `tsc` de référence compte 16 erreurs côté serveur et 10 côté web, 0 nouvelle. Lint : 0 erreur, 908 avertissements dont 7 nouveaux non bloquants. Format : 48 PASS. Aucune promesse sur la suite T3 complète.
- **Interface :** 25 PASS sur 4 fichiers, selon le principal. La revue T041 avait compté 24 sur les mêmes 4 fichiers. Cet écart n'est pas tranché dans les sources.
- **Verdict de session :** T001 à T042 validés au principal. Le verdict « PARTIEL - recettes pending » ci-dessus est remplacé. Les essais réels r4 et r7 sont faits et référencés dans `validation/proof-map149.md`.
- **Synthèse :** `validation/prelivraison149.md`.

## Actualisation du principal — portée historique

Cette revue conserve ses constats datés. La revue des quatre sources natives finales et du libellé du journal T3 est maintenant `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-restart-final-source-review-sonnet-r2.md` : SOURCE_ONLY_APPROVE, zéro défaut bloquant. Les recettes réelles et preuves actuelles sont référencées par l'actualisation de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/proof-map149.md`. La mention globale « recettes pending » ci-dessous décrit l'ancien tour. Le principal attend encore la recette Codex r7 avant le GO de livraison ; aucun cochage ni livraison n'est anticipé ici.

Date : 2026-10-10. Relecteur : sous-agent de revue (Sonnet 5.5, effort high).
Cette revue ne coche aucune case. Elle ne lance aucun modèle réel, aucun service, aucun Git, aucune installation, aucun Cargo. Elle n'édite aucun fichier de production ni de test.

## 1. Verdicts (deux niveaux séparés)

| Niveau | Verdict |
|---|---|
| SOURCES (code Rust et T3 effectifs, tests exécutés, contrat) | **SOURCE_ONLY_APPROVE** - zéro finding actif de sûreté ou de correction |
| Session globale (recettes réelles T036 à T039, SC001, SC003 réel, SC004 réel) | **PARTIEL - recettes pending**. Ce rapport n'est pas une fin fonctionnelle. |

SOURCE_ONLY_APPROVE veut dire ceci : le code lu respecte le contrat validé. Les tests exécutés prouvent les niveaux annoncés plus bas. Il ne dit rien du comportement des modèles réels ni du MCP privé authentifié sur le réseau.

Conditions de validité : les empreintes de la section 2 doivent rester identiques. Une nouvelle correction de production rouvre cette revue pour les fichiers touchés.

## 2. Code exact relu (empreintes vérifiées aujourd'hui)

Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`. Base `6807c22b7ada683f757486a6382aeda170ec68ab`. L'arbre n'est pas committé (41 entrées `git status`).

- Empreinte des 196 fichiers `.rs`, `.toml`, `Cargo.lock` de `crates/` et de la racine : `e6f45086b23974604667c93d3b34755d7d73dc5dadbe4dde94c1e09f34285408`. Je l'ai recalculée avec l'algorithme du reçu. Elle est **identique** au reçu `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native149-debug-receipt.json`.
- Les 37 digests individuels du reçu sont tous identiques. Aucun `.rs` n'est plus récent que le binaire debug `bridget-823e8a5fab8a`.

T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`. Base `33f6d04e116430bf7f0011902d6af3868163f2d1` (la base T3 est le HEAD actuel de ce worktree).

- `git diff HEAD` des fichiers suivis : `fc9fa99f4ada2dca` (16 premiers caractères), **identique** au rapport r4.
- `apps/server/src/bridget/BridgetLineage.ts` : `787109f44ddf9a52`. `apps/web/src/components/BridgetTaskJournal.tsx` : `16004d224bac63ef`. Les deux sont identiques au rapport r4.
- Aucun fichier `.ts` ou `.tsx` n'est plus récent que le rapport r4.

Donc les 1796 PASS natifs et les 120 PASS serveur T3 portent bien sur le code que j'ai relu.

## 3. Ce que j'ai fait moi-même

Lecture : diffs complets de `native_delegation.rs`, `delegation.rs`, `registry.rs`, `t3code_contract*.rs`, `claude_stream_json.rs`, `codex_app_server.rs`, `protocol.rs` ; lecture intégrale de `delegation_lineage.rs`, `daemon/native_lineage.rs`, `native_permissions.rs`, `native_permission_observer.rs` ; côté T3 le diff de `Orchestrator.ts` et de `BridgetReader.ts`. Lecture des tests nommés des sections 5 et 6 (assertions, pas les titres seuls).

Exécutions (lecture seule, sans modèle) :

| Contrôle | Résultat |
|---|---|
| `claude --version` et `claude --help` (sondes sans modèle) | 2.1.296 ; `--permission-prompts host\|none`, `--permission-mode`, `--tools`, `--allowedTools` existent |
| `claude --permission-mode default --version` | accepté (alias caché) ; `bogus` refusé : le CLI valide bien les choix |
| Recherche de chaînes dans le binaire CLI 2.1.296 (lecture seule) | la liste interne est `acceptEdits, auto, bypassPermissions, default, dontAsk, plan` et `manual` est normalisé en `default` (`rh(e)=e==="manual"?"default":e`). L'événement `system` porte donc `default`. Le Rust, qui n'accepte pas `manual`, est correct. |
| `tsc --noEmit` : `packages/contracts`, `packages/client-runtime`, `packages/provider-core`, `apps/mobile` | 0 erreur chacun |
| `tsc --noEmit` : `apps/server` | 16 diagnostics, **tous dans des fichiers que le diff ne touche pas** : `BridgetRustInterop.test.ts` (2), `BridgetRustInterop.testkit.ts` (12), `BridgetRustInteropObserver.test.ts` (1), `provider/Drivers/CodexMcp.ts` (1). Ce sont des règles de style du plugin Effect (imports Node, `Date.now`). Aucune n'est une erreur de type. Le compte est le même qu'en r4. |
| `tsc --noEmit` : `apps/web` | 10 diagnostics, tous dans `MessagesTimeline.logic.test.ts` (7) et `MessagesTimeline.test.tsx` (3), non touchés par le diff |
| `vp test run` : journal web, `ThreadRelationshipsControl.bridget149`, `orchestration.bridget149`, journal mobile | **24 PASS / 0 FAIL, 4 fichiers** (2,47 s). Le chiffre « 22 » de r4 ne compte pas le mobile (3 fichiers, 9+9+4). Le mobile ajoute 2 tests. |
| Recomptage des logs r4 natifs | daemon 1471 PASS / 0 FAIL / 61 ignorés ; transport 325 / 0 / 2 ; observer en série 10 / 0 ; **0 avertissement de compilation** |
| Ports de recette 14775 et 15735 | libres |

Non exécuté, avec la raison :

- Lint T3 (`vp lint`) : impossible dans ce worktree. Le plugin `oxlint-plugin-t3code` exige le paquet `@oxlint/plugins`, absent de `node_modules`. Ce n'est pas un finding de code. C'est une limite d'environnement.
- Clippy et rustfmt : interdits par le brief (Cargo). Le build debug r4 et les 1796 tests prouvent que le code compile sans avertissement.
- Les 120 tests serveur T3 : non rejoués. Les sources sont identiques à r4 (empreintes ci-dessus).

## 4. Revue du contrat, point par point

Les noms de rapports sans dossier se trouvent dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/`. Les références de ligne Rust sont dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/` sauf mention contraire.

| Point du brief | Conclusion | Preuve lue |
|---|---|---|
| Révocation : priorité 148 conservée | Conforme. Révocation directe de l'agent ou de l'instance : `delegation_grant_required`. Descendant d'une racine révoquée : `permission_not_inherited`. Aucun droit élargi : ce sont deux refus. | `daemon/native_delegation.rs:40` et `:42` ; `:535` refuse de nouveau au spawn |
| Rejeu avant fait | Conforme. `by_agent_request` (ligne 415) est évalué **avant** `parent_fact` (ligne 420). Un rejeu ne réévalue ni droits ni révocation. | `daemon/native_delegation.rs:415-420` |
| Gardes d'identité | Conforme. `handle_attested` revérifie `live_connection_identity` et `human_authority_guard` deux fois : hors verrou puis au puits, sous verrou. | `daemon/native_delegation.rs:267-312` |
| Permissions figées | Conforme. Snapshot créé à l'admission. Au spawn : recheck des contextes et du `launch_context`. Au lancement transport : `recheck_frozen_inputs`. Les variables `BRIDGET_NATIVE_CHILD_POLICY` et `BRIDGET_NATIVE_PERMISSION_SOURCES` sont retirées de l'environnement du CLI fournisseur. Pas de nouvelle conversion de posture au spawn. | `daemon/native_delegation.rs:546-559` ; `crates/bridget-transport/src/claude_stream_json.rs` et `codex_app_server.rs` (`env_remove`) |
| Mode observé | Conforme. Les modes acceptés sont `default, acceptEdits, bypassPermissions, plan, dontAsk, auto`. Un mode inconnu efface le fait (fail-closed). `manual` est un alias du CLI, normalisé avant l'événement (sonde du binaire). | `claude_stream_json.rs` (bloc `system`) ; `native_permission_observer.rs` (liste `valid`) |
| Réduction des droits | Conforme, fail-closed. `allowed_tools` non vide ou `tools:[]` vers Codex : refus. Lanceur non épinglé : `permission_source_unavailable`. Source `--restricted` jamais ajoutée en 149. Alias `--allowed-tools` et `--disallowed-tools` retirés puis réémis en forme figée. Aucune direction ne donne plus de droits que le parent. | `native_permissions.rs:81-91, 206-277, 284-321` |
| Profondeur et cycles | Conforme. Une chaîne a au plus 8 missions : `parent_lineage` refuse à 8 ancêtres, la migration refuse un cycle, plus de 8 ou plus de 4096 enregistrements (le daemon refuse alors de démarrer : choix assumé, à documenter en exploitation). `descendants` est borné (profondeur 8, 4097 lignes). | `delegation_lineage.rs:80-92, 126-147` |
| Propriété (ownership) | Conforme. `save_projected` conserve racine et parent de l'ancien enregistrement : un retry ou une nouvelle instance ne les change jamais. Toutes les lectures filtrent sur `root_owner()`. Le reçu d'annulation est indexé par `(racine, request_id)` : un autre propriétaire ne rejoue rien. | `delegation_lineage.rs:161-175, 206-247` ; `daemon/native_lineage.rs:68-76` |
| Charges fermées | Conforme. `validate` côté Rust refuse les formes ouvertes, taille de curseur, UUID non canonique, bornes de page. Côté T3 : `onExcessProperty:"error"` sur chaque décodage. | `daemon/native_lineage.rs:43-54` ; `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader.ts` |
| Séparation du marqueur de tâche | Conforme. `is_native_projection` ferme les 7 clés exactes, UUID, borne de séquence, statuts. Le préfixe seul ne suffit pas. Un marqueur invalide refuse tout l'instantané. Le filtre est appliqué dans les deux analyseurs (v1 ligne 320, v2) avant toute normalisation. | `t3code_contract.rs:197-209, 320` ; `t3code_contract_v2.rs:64-70` |
| Outbox, aucune exécution | Conforme. Aucun nouvel effet : `bridget.lineage.sync` n'émet que des événements de projection. Les compteurs SQL `runs`, `provider_turns`, `provider_sessions`, `bindings`, `runtime_requests`, `workflows` valent 0 en fin de recette. `effect_outbox` = 4 lignes `terminal.cleanup` et `preview.cleanup` de `thread.delete`, statut `succeeded`. Les consommateurs d'origine ont été audités par moi (grep sur tout le code) : tous les `origin !== "app_owned"` excluent `bridget_native`. La seule ligne qui changeait de sens est `Orchestrator.ts:8324` : `!== "app_owned"` devient `=== "provider_native"`, ce qui exclut `bridget_native`. | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.ts:8324, 10306` ; `t3-runtime-hardening-sonnet-r4.md` |
| Natif depuis la vue, autonomie | Conforme. La CLI Rust expose seulement lecture, journal, watch et `cancel`. La vue T3 interdit toute commande sur un fil virtuel sauf `thread.visit` et `thread.stop`. `cancel` déclenche seulement la saga existante. Aucun message, aucun lancement. | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.ts:10300-10310` ; `daemon/native_lineage.rs:94-119` |
| Dernier correctif Sol (révocation) | Accepté. Il ne change que l'ordre des deux refus. Les 5 adaptations d'oracles 148 de r4 sont conformes au contrat nommé (`permissions.md` lignes 14-17, `lineage.md` ligne 190). Les 3 fixtures périmées `spec094`, `spec102_v33`, `spec102_v34` sont des attentes sur des ensembles exacts (20 outils MCP, 9 actions, migration 3). Je les accepte : l'égalité d'ensemble exacte est conservée. | `native-tests-sonnet-r4.md` sections 1 à 3 |

## 5. Findings

**Zéro finding actif de sûreté, de correction ou de contrat.**

### Observation O1 - non bloquante, non mesurée (performance, Article XXII)

`thread.stop` sur un fil ordinaire appelle maintenant `lineageSnapshot` avant d'interrompre le tour fournisseur, dès que le service `BridgetReader` est présent (`/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.ts:9073-9094`). Cela lance la CLI Bridget (timeout 6 s par page, 30 s au total). Si la CLI échoue et qu'aucun enfant natif n'est actif, l'erreur est ignorée et l'arrêt continue. Le coût n'a pas été mesuré. Le cas pire est un daemon qui accepte la connexion mais ne répond pas : le bouton Arrêter attendrait jusqu'à 6 s. Le daemon arrêté échoue vite (code 3). Aucune correction demandée. Conseil : mesurer une fois le temps d'un arrêt ordinaire avec un daemon bloqué, puis décider. Le choix de lire l'état natif à l'arrêt est volontaire (il trouve les enfants non encore projetés).

### Gaps de preuve réels et minimaux

Chaque gap a un niveau de preuve actuel, ce qui manque, et le plus petit test utile. Aucun ne bloque SOURCE_ONLY_APPROVE. Le principal décide s'il les ferme avant le cochage.

| Id | Tâche | Niveau actuel | Ce qui manque | Test minimal |
|---|---|---|---|---|
| G1 | T009 | Refus de fait absent ou étranger : unitaire | Le chemin T3 privé : `delegation_mcp::execute` (message `NativeDelegationT3`), `t3code_mcp::reattest`, `native_delegation::handle_attested`. Aucun test Rust ne les appelle. La preuve réelle relève de T036 (HTTP vers T3). | Aucun test unitaire nécessaire si T036 passe. Sinon test de `handle_attested` avec un faux `reattest`. |
| G2 | T011 | Mappage des politiques : unitaire (`native149_codex_*`, formes fermées transport) | L'injection `approvalPolicy`, `approvalsReviewer`, `sandboxPolicy`, `cwd` dans `turn/start` (`codex_app_server.rs`, `start_turn_with_retry`) n'est assertée par aucun test d'octets. C'est le point d'application des droits d'un enfant Codex. | Un faux serveur d'application Codex qui capture le `turn/start` et compare les quatre champs à `BRIDGET_NATIVE_CHILD_POLICY`. T037 le prouvera aussi avec un vrai Codex. |
| G3 | T006 | Publication Codex : un test nommé | Le retrait du fait sur échec, fin de tour et finalisation (`retireBridgetPermissions`, `Effect.onError`) n'a pas de test Codex nommé. Côté Claude, « retires at turn end » existe. | Un test Codex : démarrage d'un tour qui échoue puis lecture du fait = absent. |
| G4 | T015 / S149-29 | Transport, processus réel avec faux CLI `/bin/sh` : refus corrélé par `request_id` exact, `behavior:deny`, `DeliveryRejected` nommé `provider_permission_denied`, jamais de `TurnFinished` | Le maillon daemon : `wrapper` envoie `DeliveryRejected{reason}` puis `native_delegation::rejected` (ligne 743) met la tâche `failed` avec `error="provider_permission_denied"`. Aucun test ne vérifie cette dernière étape. Le code est correct à la lecture. | Un test de `rejected` : la tâche passe `failed`, `error` égale le code, `result_available` jamais atteint. |
| G5 | T035 | Audit statique des consommateurs d'origine : fait par moi, aucune fuite | La matrice de tests « trois origines par quatre branches (start, resume, remise, outbox) » demandée par T035 n'est rapportée nulle part. | Le principal peut accepter l'audit statique et les 33 tests Orchestrator. Sinon un test paramétré. |
| G6 | T022 | Marqueur et exclusion V2 : unitaire | La branche `parse_snapshot` v1 (`t3code_contract.rs:320`) n'a pas de test direct. Le validateur `is_native_projection` est testé. | Un snapshot v1 avec marqueur valide : le fil est absent du résultat. |
| G7 | T012 / S149-30 reprise | `recheck` des sources : unitaire (mutation, ajout, suppression, symlink, PATH) ; spawn : test daemon | Le refus **à la reprise du wrapper** (`wrapper.rs:4034` et `:4553`) n'a pas de test. Ce recheck a lieu après le lancement du CLI : il retire le fait publié, il ne bloque pas le CLI. Le blocage avant effet est dans `spawn_task` et `recheck_frozen_inputs`, testés. | Optionnel. T039 (reprise sans relance) le couvrira. |
| G8 | T020 / T032 | Page de journal natif (`lineage_journal_page`) : unitaire (`native149_page_journal_bornes_permissions_et_fermeture`). Suivi vu en recette UI avec le daemon **simulé**. | Le suivi réel `--follow` : `native_lineage::journal_follow` et `attach::lineage_journal_follow` (relais AttachRelay, 128 flux au plus, invalidation par garde). Aucun test Rust ne les exécute. `native149_validate_ferme_les_formes_ouvertes` n'en teste que la forme de requête, et `lineage_cli_test.rs` ne contient pas `--follow`. J'ai relu ces deux fonctions : aucun défaut vu. | Un test CLI : `lineage inspect --action journal --follow` sur un enfant vivant, un événement ajouté, puis arrêt propre. Ou un reçu de T036 si l'interop lit un journal en suivi. |
| G9 | T017 / S149-27 | Borne et version de séquence : `lineage149_version_et_borne_seq_figees` ; signal durable : `delegation149_insert_seq_signal_durable_et_horodatages` | Le débordement de séquence : `increment` (`delegation_lineage.rs:68`) crée une nouvelle génération et repart à 1 quand `seq` atteint la borne. Aucun test ne place `seq` près de la borne. J'ai relu le code : il est correct. | Fixer `seq` à la borne moins un en base, insérer deux tâches, vérifier la génération puis `seq`. |

### Documentation à corriger par le principal (hors de mes fichiers)

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/tasks.md`, en-tête : « G-P REQUEST_CHANGES ciblé r3 » est périmé. `permissions-contract-deltas-r4.md` donne APPROVE.
- Même fichier : coquilles `àS149-33`, `Phase0`, `profondeur8`, `liste100/128KiB`, `pages16KiB/100`.

## 6. Scénarios S149-29 à S149-33 : preuve exacte par niveau

Les tests sont lus. Les noms sont ceux du code. Tous apparaissent `ok` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-r4-final-daemon.log` ou `native-r4-final-transport.log`.

| Scénario | Niveau prouvé | Tests nommés | Ce qui reste |
|---|---|---|---|
| S149-29 refus corrélé `can_use_tool` | Processus avec faux CLI (transport) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/tests/native_permissions149_e2e.rs` : `native149_can_use_tool_deny_est_correle_par_request_id_exact_et_refuse_la_remise` (une seule `control_response`, `request_id` `perm-149`, `deny`, aucun `TurnFinished` malgré un résultat « OK »), `native149_result_avec_permission_denials_est_refuse_sans_trame`, `native149_control_request_sans_request_id_n_obtient_aucune_reponse`, `native149_remise_positive_termine_le_tour_sans_refus` | Gap G4 (maillon daemon). Aucune trame réelle d'un vrai CLI. |
| S149-30 divergence de digest | Unitaire + daemon (fixture, compteur de lancement) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/tests/native_permissions149_test.rs` : `native149_recheck_des_sources_detecte_mutation_ajout_et_suppression`, `native149_revision_de_source_fichier_dossier_symlink_et_absent`, `native149_recheck_gele_resout_le_launcher_par_path_et_detecte_le_changement` ; `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs` : `native149_spawn_refuse_les_entrees_gelees_changees_sans_lancement` (mutation avant spawn, mutation avant admission, suppression : `settings_revision_changed`, aucun `Start`) | Reprise du wrapper : gap G7. Aucune preuve avec un CLI réel. |
| S149-31 source opaque | Unitaire | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions149_tests.rs` : `native149_capture_refuse_les_sources_opaques_et_garde_les_inline` (5 assertions `permission_source_unavailable`, un cas positif de même famille reste accepté) ; `native149_claude_meme_famille_exige_contexte_vivant_et_meme_racine` | Aucune. |
| S149-32 union v1/v2 | Unitaire daemon + unitaire serveur T3 | Daemon : `native149_sans_fait_le_v1_est_refuse_et_les_lectures_survivent_au_tombstone` (v1 seule et tombstone : `permission_attestation_unavailable`, aucun lancement ; lecture, status et cancel survivent). T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts` (9 tests : v1 sans fait, v2, tombstone sans repli v1, fait d'un autre run, fait invalidé, credential remonté) | L'entrée du Rust autonome par le MCP privé : T036/T038. |
| S149-33 observer PTY | Unitaire + fixture (branches a, b, c) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permission_observer149_tests.rs` : `native149_fait_publie_acquitte_le_hook_avant_sa_reprise`, `native149_acquittement_negatif_bloque_le_hook_par_nom`, `native149_overlay_modifie_est_refuse_par_un_refus_nomme`, `native149_delegation_sans_request_id_est_refusee_sans_fait`, `native149_nonce_inconnu_et_pair_etranger_restant_silencieux`, `native149_liaison_d_un_pid_mort_est_refusee_et_ne_recupere_jamais`, `native149_cycle_de_vie_fichiers_prives_et_nettoyage_raii` (10/10, y compris en série) | **Branche d (vrai CLI en PTY) : pending T038.** Composition réelle de `--settings` avec le CLI : non prouvée. Ne rien déduire des copies de `/bin/echo`. |

## 7. Limites réel et synthétique

1. Aucun modèle réel n'a tourné dans cette revue. Les fixtures `claude` des tests natifs sont des copies de `/bin/echo` ou `/bin/sh`. Elles ne prouvent aucun comportement de modèle.
2. La recette UI (T040) utilise le serveur T3 du worktree (réel), la base privée (réelle), une interface web ouverte dans la preview T3 (réelle), un **daemon Bridget simulé** (`bridget_fixture.mjs`) et des **données synthétiques** `[recette149]`. Ce n'est pas la coque desktop, ni le moteur natif.
3. Le MCP privé authentifié n'est pas prouvé. La matrice réseau valide les scopes RPC.
4. Le mobile est testé en jsdom (2 tests) et par `tsc` (0 erreur). Pas d'appareil ni d'iOS. Le plan n'en demande pas.
5. SC001 (un enfant réel unique avec tâche, résultat, zéro lancement T3), SC003 (fichiers réellement écrits par GLM et Codex, refus observé hors politique) et SC004 (runtime sans T3, annulation native) ne sont pas prouvés. Ils dépendent de T036 à T039.
6. Les points déjà mesurés en recette UI (1280, 480, 375 px ; 1024 et 768 px en r3 seulement) restent ceux de `ui-recipe149.md`.

## 8. Correction de la matrice de preuves

La matrice intermédiaire `proof-map149.md` contenait des erreurs de classement. Je l'ai réécrite. Les corrections de fond :

- T022 : « Non prouvé » était faux. Deux tests Rust nommés existent et passent.
- T029 : « Non prouvé » était faux. Sept tests `thread.stop` existent dans `Orchestrator.bridget149.test.ts` et passent. Le classement ne cherchait que les rapports.
- T016 à T023 : la colonne « non nommés » ignorait des tests nommés (profondeur, atomicité, rollback, watch, pages de journal, annulation, rejeu).
- T001 et T002 : classés partiels à tort. Leur critère est documentaire et il est rempli.
- T040 : classé partiel à tort. Le critère est une recette UI web avec preuves visuelles, et elle existe (r2, r3, r4).
- Le « 22 tests UI » de r4 n'inclut pas les 2 tests mobiles. Total vérifié aujourd'hui : 24.
- Les chemins tronqués `.../` sont remplacés par des chemins absolus.

## 9. Pour le principal

- Verdict sources : SOURCE_ONLY_APPROVE. Candidats au cochage sur cette base, au niveau de preuve de `proof-map149.md` : T001, T002, T004, T005, T007, T008, T010, T016 à T019, T021 à T023 (T020 partiel : gap G8), T024 à T031 (T029 compris), T033, T040. Restent partiels : T006, T009, T011, T012, T013, T014, T015, T020, T032, T034, T035 (voir `proof-map149.md`). Le principal décide. Cette revue ne coche rien.
- À fermer ou accepter avant le cochage : G1 à G9 (aucun ne bloque).
- À attendre : T036 à T039 (pending chez d'autres propriétaires), puis T042 (`quickstart.md`, `implementation.md`), T043 à T045.
- Fichiers de ce propriétaire libérés : `review-final149.md`, `proof-map149.md`, `ui-recipe149.md`.
