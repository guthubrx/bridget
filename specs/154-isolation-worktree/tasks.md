# Tâches - Session154 isolation optionnelle dans un worktree

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R11), `data-model.md`,
`contracts/workspace154.md`, `reuse-audit.md`, `adversarial-review-codex.md`.
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`, variables `GIT_*` de test).

**Arbitrage reçu (2026-10-11)** : option « comme T3, durcie » (`spec.md`,
« Arbitrage utilisateur - exécution de Git »). Le daemon exécute Git pour tout
parent autorisé. Total : 28 tâches (T001 à T028). Les variantes Git confiné et
isolation coopérative sont écartées.

Règle : une tâche n'est cochée qu'après son test vert. Les tests marqués
« rouge d'abord » sont écrits et vus en échec avant le code. Pas de commit
automatique. Aucun test ne crée de worktree dans le dépôt Bridget : les dépôts
de test vivent sous `$R`. Aucune tâche n'ajoute de suppression de worktree ou
de branche.

## Phase 1 - Préparation

- [ ] T001 Établir la référence de non-régression avant tout changement : lancer les filtres `native148`, `native149`, `native150`, `delegation149`, `spec100` et `spec_087` de `crates/bridget-daemon` avec la recette, et consigner les compteurs PASS/FAIL/ignorés dans `specs/154-isolation-worktree/validation/baseline.md`
- [ ] T002 Mesurer le coût de `git worktree add` sur un clone local du dépôt Bridget (recette « Mesure préalable » du quickstart), 5 répétitions, et consigner médiane et maximum dans `specs/154-isolation-worktree/validation/mesure-worktree.md` ; la mesure règle le délai de Git de T006 (20 s par défaut) ; elle ne conditionne plus l'architecture, car Git tourne hors verrou

## Phase 2 - Fondations (bloquantes)

- [ ] T003 [P] Rouge d'abord puis implémenter dans `crates/bridget-transport/src/protocol.rs` l'énumération `WorkspaceRequest` (`shared`, `worktree{base_ref, branch}`, `deny_unknown_fields`) et le champ `Delegate.workspace` avec `skip_serializing_if = "Option::is_none"` placé après `reply_timeout` ; tests `native154_contrat_*` : une requête sans le champ produit les mêmes octets `serde_json::to_vec` qu'avant, une clé inconnue est refusée, `{"strategy":"shared"}` et l'absence donnent deux enveloppes différentes
- [ ] T004 [P] Ajouter dans `crates/bridget-daemon/src/delegation.rs` la struct `WorkspaceRecord` et le champ `Task.workspace: Option<WorkspaceRecord>` avec `#[serde(default)]` (champs du `data-model.md`, états `active` et `kept` seulement) ; test : un payload 150 se relit avec `None`, et `visible_changed` ignore `workspace` (aucun signal Lineage)
- [ ] T005 [P] Rouge d'abord : écrire dans `crates/bridget-daemon/src/workspace_git_tests.rs` les tests `native154_git_*` de l'exécuteur : environnement sans `GIT_DIR`/`GIT_WORK_TREE`/`GIT_COMMON_DIR`, `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_NOSYSTEM=1`, `GIT_NO_LAZY_FETCH=1`, `core.hooksPath=/dev/null`, `core.fsmonitor=false`, `protocol.allow=never`, `GIT_ALLOW_PROTOCOL=` vide, `--no-pager`, `log.showSignature=false` et `--no-show-signature` (le programme `gpg.program` d'un dépôt hostile n'est jamais lancé), `--no-textconv` et `--no-ext-diff` présents sur chaque appel ; lecture de la sortie bornée en mémoire et en temps (processus lancé avec `spawn`, jamais `output()` sans borne) ; liste blanche FR-015 (une sous-commande hors liste, dont `worktree remove`, `branch -D`, `update-ref -d`, `clean`, `fetch`, est refusée sans lancer de processus) ; délai dépassé tue le processus lancé par Bridget ; sortie tronquée à 1 Mio ; Git en version inférieure à 2.44 → `workspace_git_unsupported` (simulé par un faux `git` en tête de `PATH`) ; validations de noms (SC-004 : `..`, `-x`, `@{`, `.lock`, espace, longueur, `--upload-pack=x` en `base_ref`) ; hook `post-checkout` du dépôt non exécuté pendant `worktree add`
- [ ] T006 Implémenter `crates/bridget-daemon/src/workspace_git.rs` (déclaré dans `crates/bridget-daemon/src/lib.rs`) : exécuteur unique (liste blanche FR-015, environnement nettoyé, configurations système et globale coupées, délais 20 s création et 10 s ailleurs, sortie bornée, `--end-of-options`), contrôle de version, `validate_branch`, `validate_base_ref`, annoter la complexité des fonctions publiques ; T005 passe

## Phase 3 - US2 Création sûre et droits hérités (P1)

**But** : valider tout avant d'écrire sur le disque, ne jamais tenir le verrou pendant Git, ne laisser aucun reste après un refus décidé avant la création.
**Test indépendant** : filtre `native154_creation`.

- [ ] T007 [US2] Rouge d'abord : écrire dans `crates/bridget-daemon/src/daemon/native_workspace154_tests.rs` (déclaré dans `native_delegation.rs` à côté de `native_delegation_permissions149_tests`) les tests `native154_creation_*` avec un dépôt Git réel sous `$R` : un cas par refus du contrat (`workspace_requires_development`, `workspace_inheritance_unavailable`, `workspace_not_git`, `workspace_repo_unsupported` pour chacune des quatre familles de configuration, `workspace_base_unavailable`, `workspace_branch_invalid`, `workspace_branch_exists`, `workspace_dir_not_ignored`, `workspace_limit` avec 32 worktrees semés, `workspace_path_unsafe` avec `.worktrees` en lien symbolique, `workspace_create_failed` par délai simulé), chacun vérifiant 0 worktree (`git worktree list`), 0 branche et 0 tâche ; **filtre hostile** : dépôt avec `filter.x.smudge` qui crée un marqueur et `.gitattributes` `* filter=x` → refus, marqueur absent ; **objet absent** : dépôt partiel simulé (remote vers un chemin inexistant, blob de la base supprimé) → refus ou `workspace_create_failed`, aucune tentative réseau ; **inclusion** de configuration → refus ; rejeu de la même clé : un seul worktree ; autre `workspace` à la même clé : `envelope_mismatch` ; limite d'admission (`task_limit`) atteinte entre les deux passes : refus, restes nommés dans le message, rien supprimé ; pause du référent : refus avant toute création ; **réservations concurrentes** (T027) : deux appels simultanés de même clé créent un seul worktree ; 40 appels simultanés ne dépassent pas 32 ; **revalidation cohérente** : la branche réservée et la 32e place ne sont pas refusées par la seconde passe ; **verrou libre** : avec un faux `git` qui dort 2 s pendant `worktree add`, une lecture de statut d'une autre tâche répond en moins de 500 ms (SC-009)
- [ ] T008 [US2] Dans `workspace_git.rs` et `handle_with_fact` de `crates/bridget-daemon/src/daemon/native_delegation.rs`, écrire l'avant-vol hors verrou : `--version`, `rev-parse --show-toplevel`, `config --get-regexp` sur la configuration locale (refus des familles de FR-013), `rev-parse --verify --end-of-options <ref>^{commit}`, `check-ref-format --branch`, `show-ref --verify --quiet` pour un nom explicite, `check-ignore -q .worktrees/x`, `worktree list --porcelain` (plafond 32) ; ne lancer ni `status` ni `worktree add` avant l'autorisation ; au puits, comparer le dossier canonique au résultat de l'avant-vol ; le nom de branche par défaut est contrôlé par la création elle-même
- [ ] T009 [US2] Dans l'admission de `native_delegation.rs`, passer en deux passes avec une seule fonction : générer le `task_id` dans la réservation ; après le premier calcul de `child_policy`, appliquer `workspace_requires_development` et `workspace_inheritance_unavailable` (paire `claude_stream_json` vers `claude_stream_json`) ; après `task_limit`, `retention_saturated`, lignée et `reply_timeout_exceeds_parent`, rendre la réservation à `handle_with_fact` qui libère le verrou, écrit au journal le chemin et la branche, puis crée le worktree (`<cwd>/.worktrees/<task8>`, `.worktrees` contrôlé par `symlink_metadata`, Bridget ne pré-crée jamais le dossier final : seul Git le crée, donc son nettoyage interne d'un échec ne touche que ce que Git a créé) ; seconde passe sous verrou : refaire seulement les contrôles qui restent vrais après la création (rejeu, pause, droits, `task_limit`, `retention_saturated`, lignée, délai, plafond par les enregistrements et les réservations, existence du chemin attendu sur le disque) et pas ceux qui portent sur l'état Git créé par Bridget (branche, ignorance de `.worktrees/`, configuration, liste des worktrees) ; recontrôler que le chemin canonique du worktree et le dossier de départ (worktree plus décalage) sont les chemins attendus sous la racine, recalculer `child_policy` et `snapshot` sur ce dossier, écrire la tâche ; aucun retour arrière : tout échec après création renvoie un refus qui nomme chemin et branche ; T007 passe
- [ ] T010 [US2] Catalogue : ajouter `workspace_strategies` à la racine et `worktree` / `worktree_refusal` à chaque fournisseur dans `native_delegation.rs:380-392` avec la logique de paire de T009 ; mettre à jour la liste de clés figée de `native_delegation_permissions149_tests.rs:195` ; test `native154_creation_catalogue_par_paire` (Codex vers Codex vrai, Claude vers Claude faux avec motif)
- [ ] T011 [US2] Exposer `workspace` dans `crates/bridget-daemon/src/delegation_mcp.rs` : schéma d'entrée (objet fermé), entrée de la liste blanche, lecture de l'objet avec refus des clés inconnues ; test : l'outil refuse `start_from_origin`, `push`, `merge` et `cleanup`

- [ ] T027 [US2] Réservations concurrentes : table en mémoire de réservations dans l'état du daemon (clé `(propriétaire, request_id)` et compte par dossier `.worktrees/`), garde qui libère la réservation à la fin, y compris sur panique ; la première passe refuse une seconde création de même clé avec `workspace_creation_in_progress` (rejouable, aucune écriture) ; le plafond de 32 compte les worktrees listés à l'avant-vol, les enregistrements de tâches et les réservations en cours ; tests de T007 (SC-010)
- [ ] T028 [US3] Rapport hors verrou dans le tick : sous verrou, le tick note un travail de rapport (chemin, base, branche) pour toute tâche en worktree dont l'enfant est arrêté ou a rendu son résultat et qui n'a pas encore de rapport (`native_delegation.rs` près des lignes 959 et 1111), puis rend le verrou ; Git lit le worktree ; un verrou court range le rapport si aucun n'est rangé (le premier gagne) ; la transition vers `result_available` et la pose de `cleanup_done` attendent le rapport ; test `native154_rapport_hors_verrou` : avec un faux `git` lent, une lecture de statut d'une autre tâche répond en moins de 500 ms pendant le rapport (SC-011)

## Phase 4 - US1 Isolation effective (P1)

**But** : l'enfant démarre dans son worktree, le parent ne change pas.
**Test indépendant** : filtre `native154_isolation`.
**Dépend de** : Phase 3.

- [ ] T012 [US1] Rouge d'abord puis vérifier : tests `native154_isolation_*` : le `cwd` du lancement (`FleetSpawnOrder.cwd`) est le worktree ; HEAD, index et `git status` du dossier du parent identiques avant et après (SC-001) ; dossier du parent en sous-répertoire du dépôt (décalage conservé) ; deux enfants en worktree sur deux branches distinctes ; petit-enfant en `worktree` dans le worktree du premier, accepté, son dossier sous la racine du premier ; contexte de lancement recontrôlé au lancement pour Codex vers Codex, Codex vers Claude et Claude vers Codex ; aucune racine inscriptible ajoutée à la politique du bac à sable
- [ ] T013 [US1] Test `native154_isolation_worktree_absent_au_lancement` : si le dossier du worktree a disparu avant le lancement de l'enfant, la tâche échoue avec `cwd_unavailable` et Bridget ne le recrée pas ; adapter `spawn_task` seulement si le test l'exige

## Phase 5 - US3 Rapport de branche et de commits (P1)

**Test indépendant** : filtre `native154_rapport`.

- [ ] T014 [US3] Rouge d'abord : tests `native154_rapport_*` : 0 commit propre ; 2 commits ; 60 commits (50 listés, `commit_count` 60, `truncated` vrai) ; fichier non suivi donne `dirty` vrai, y compris avec `status.showUntrackedFiles=no` dans la configuration du dépôt ; tête détachée et branche changée donnent `head_on_branch` faux avec `branch_commit` distinct ; sujet de commit avec caractères de contrôle assaini ; sujet qui imite la ligne `[Bridget] espace de travail` ne modifie aucun champ structuré ; ligne de fin présente et unique dans le résultat remis
- [ ] T015 [US3] Implémenter dans `workspace_git.rs` la fonction de rapport en lecture seule (`rev-parse HEAD`, `symbolic-ref -q HEAD`, `rev-parse <branche>`, `rev-list --count <base>..HEAD`, `log --max-count=50 --format=%H%x1f%s`, `status --porcelain=v1 --untracked-files=normal` borné) et l'appeler hors verrou (voir T028) avant le passage à `result_available` (près de la ligne 1111) : stocker dans `Task.workspace`, ajouter la ligne de fin au résultat avant la sauvegarde ; T014 passe
- [ ] T016 [US3] Ajouter l'objet `workspace` à `task_view` (`native_delegation.rs:68-86`) et au reçu de création (`state`, `path`, `branch`, `base_commit`) ; test : statut d'une tâche sans worktree inchangé octet pour octet (SC-006)

## Phase 6 - US4 Conservation sans suppression (P1)

**Test indépendant** : filtre `native154_conservation`.
**Dépend de** : Phases 3 et 5.

- [ ] T017 [US4] Rouge d'abord : tests `native154_conservation_*` : tâche terminée, échouée ou annulée → rapport final lu après l'arrêt confirmé de l'enfant (jamais avant), `state` = `kept`, `cleanup_done` vrai ; worktree sale : fichiers intacts ; **worktree emboîté** : le worktree d'un petit-enfant sous celui de l'enfant reste intact après la fin de l'enfant ; `status.showUntrackedFiles=no` : fichier non suivi intact ; l'exécuteur enregistreur ne voit jamais `worktree remove`, `--force`, `branch -d/-D`, `update-ref`, `reset`, `clean`, `checkout`, `push`, `fetch`, `merge` (SC-002, SC-005) ; rapport relu une seule fois après redémarrage du daemon
- [ ] T018 [US4] Implémenter dans `native_delegation.rs` le rapport final pour `failed` et `cancelled` dans les chemins qui posent `cleanup_done` (`cancel_tree` après arrêt confirmé, nettoyage après remise du résultat, nettoyage après avis d'échec), par le mécanisme hors verrou de T028 ; `state` passe à `kept` et `cleanup_done` est posé après ce rapport ; aucune suppression ; T017 passe
- [ ] T019 [US4] Tests d'articulation avec la session150 `native154_conservation_retention_150_*` : un arbre dont le worktree est `kept` reste purgeable selon les règles 150 et la purge n'ouvre aucun fichier du disque ; un arbre dont le worktree est encore `active` n'est pas purgeable (via `cleanup_done` faux) ; le rejeu d'une clé purgée rend `request_expired` sans recréer de worktree

## Phase 7 - US5 Partage par défaut et collision (P2)

**Test indépendant** : filtres `native154_partage`, `spec100`.

- [ ] T020 [US5] Rouge d'abord puis implémenter `shared_writers` dans le reçu de création (`native_delegation.rs`) : nombre d'autres tâches natives actives en développement avec le même dossier, présent seulement si au moins 1 et si la stratégie est partagée en développement ; tests `native154_partage_*` : 0 autre tâche donne un reçu identique à 150 octet pour octet, 1 autre tâche donne `shared_writers: 1`, tâche en `discovery` ou terminée non comptée, aucun refus ; annoter la complexité (O(128))
- [ ] T021 [P] [US5] Ajouter dans les tests de `crates/bridget-daemon/src/observation.rs` le test `native154_worktree_paths_do_not_collide` : deux agents écrivant `<racine>/a.rs` et `<racine>/.worktrees/x/a.rs` dans les 30 secondes ne produisent aucun `file_collision`, tandis que deux écritures du même chemin en produisent un ; aucun code de production modifié

## Phase 8 - Finitions

- [ ] T022 [P] Faire passer les lectures Git du wrapper (`managed_resume_worktree`, `is_protected_principal_checkout`, `crates/bridget-daemon/src/wrapper.rs:572-590` et `:623-641`) par l'exécuteur de `workspace_git` en lecture seule : `rev-parse`, `symbolic-ref`, `status`, `worktree list` ; sortie et temps bornés (plus de `.output()` sans borne) ; `status` omis, avec la mention « indisponible (configuration exécutable) » dans la carte, si la configuration locale contient un filtre exécutable ; rouge d'abord : tests `native154_wrapper_*` : marqueur d'un `core.fsmonitor` hostile absent, marqueur d'un filtre `clean` hostile absent, sortie énorme tronquée sans épuiser la mémoire, délai respecté (dépend de T006)
- [ ] T023 [P] Documenter dans `docs/delegation-native.md` (section « Espace de travail ») : stratégies, treize refus, ce que Bridget fait et ne fait pas côté Git, aucune suppression, recette de nettoyage humain (`git worktree list`, `git branch --list 'bridget/*'`, `git worktree remove`), dépôts refusés (LFS, filtres, inclusions, clone partiel), Git 2.44, `.worktrees/` à ignorer dans le dépôt, plafond de 32, différence avec les modifications non commitées du parent, fichiers ignorés, limite du bac à sable ; reporter l'essentiel dans `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md`
- [ ] T024 [P] Écrire l'ADR `docs/decisions/154-isolation-worktree-optionnelle.md` (contexte, décisions D1 à D11, rejet de la création par le wrapper et du nettoyage automatique avec les raisons de la contre-revue, conséquences positives et négatives dont l'accumulation des worktrees et le résiduel de course sur `.worktrees`, **section « Risque accepté » reprenant le texte de l'arbitrage et renvoyant au cadre de menace de l'ADR 011**, comparaison avec T3 (`GitVcsDriverCore.ts:3418-3419`, `:3789`), alternatives écartées, statut Accepté)
- [ ] T025 Comparer aux compteurs de `validation/baseline.md` (T001) : aucun test 148, 149, 150, 087 ou 100 ne passe de vert à rouge ; lancer la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; rejouer seuls les tests connus sensibles à la charge avant de conclure ; consigner dans `validation/tests-154.md` ; vérifier que `git worktree list` du dépôt Bridget ne contient aucun reste de test
- [ ] T026 Relire le diff complet (Articles XIX et XX : lignes suppressibles, abstractions, chemins d'erreur, usage de `unwrap`), écrire `specs/154-isolation-worktree/implementation.md` et enregistrer `git diff` dans `specs/154-isolation-worktree/validation/diff-154.patch`

## Dépendances

- T001 et T002 avant tout ; T002 règle un délai, pas l'architecture.
- T003, T004, T005 en parallèle ; T006 après T005 ; T008, T009 après T003 à T006.
- Phase 3 avant Phases 4, 5, 6.
- T015 avant T018 (le rapport final des échecs réutilise la fonction de rapport) ; T028 avant T015 et T018.
- T027 avec T009.
- T020 touche `native_delegation.rs` : séquentiel avec T009, T015, T016, T018.
- T021, T022, T023, T024 en parallèle ; T025 et T026 en dernier.
- Session150 livrée : champ `reply_timeout`, `cleanup_done` comme condition de purge, refus nommés, comptages d'admission.

## Exemples de parallélisme

- T003, T004, T005 : fichiers distincts.
- T021, T022, T023, T024 : test d'observation, wrapper, documentation, ADR.

## Stratégie

MVP = Phases 1 à 4 puis 5 et 6 : création sûre hors verrou, isolation effective,
rapport et conservation sans suppression. Le rapport est livré avec le MVP car
le parent doit connaître la branche. Puis partage et collision (Phase 7),
finitions.

## Charge future (Article XX)

- Réduisent la charge : T009 (collision évitée par construction, daemon jamais
  bloqué par Git), l'absence de code de suppression (aucune décision de
  nettoyage à maintenir), T020 (avertissement au lieu de diagnostic après coup).
- Ajoutent de la complexité justifiée : T006 (exécuteur Git, un point de
  sécurité testé), T008 (refus des configurations exécutables, imposé par la
  contre-revue), T009 (deux passes, imposées par le verrou global), T015
  (rapport lu dans Git, imposé par l'exigence « faits non forgeables »).
- Risque résiduel accepté par l'utilisateur : Un parent confiné qui peut modifier la configuration du dépôt entre le contrôle et l'exécution de Git peut faire exécuter du code avec les droits du daemon. Ce risque est accepté explicitement, au même niveau que T3, dans le cadre de menace de l'ADR 011. Il est écrit dans l'ADR 154 (T024).
- Coût porté par l'humain : le nettoyage des worktrees `kept`, borné par le
  plafond de 32 et décrit dans la documentation.
