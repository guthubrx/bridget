# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-tests-sonnet-r3.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-tests-sonnet-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `crates/bridget-daemon/src/native_permissions149_tests.rs`
  - Fixture : vrai Mach-O `/bin/echo` copié en `<fixture>/bin/claude`, command `claude`, PATH `<fixture>/bin:/usr/bin:/bin`, dossier `.git` à la racine (borne la remontée des ancêtres, sinon le vrai `~/.claude/settings.json` avec hooks est lu).
  - `claude_meme_famille` : le fait est capturé à la racine demandée, puis `fact.cwd` est changé. Avant, le contexte était capturé à l'autre racine, donc `settings_revision_changed` masquait `permission_mapping_unavailable`.
  - `snapshot_fige_et_deduplique` : le test « deux contextes » mettait le second contexte dans l'enfant seul, avec un parent Codex sans contexte (donc un seul contexte). Il utilise maintenant un parent Claude et un enfant Claude.
  - `capture_refuse_les_sources_opaques` : l'assertion « toutes les sources sont absentes » ignore les sources `managed` de l'hôte (`/Library/Application Support/ClaudeCode/managed-settings.json` existe sur cette machine). Ajout du cas F5 : lanceur binaire `wrapper-binaire` différent de `claude` donne `permission_source_unavailable`, contrôle `claude` direct accepté.
- `crates/bridget-daemon/src/native_permission_observer149_tests.rs`
  - `Observer::start` lit le PATH du processus : un `claude` Mach-O est placé en tête du PATH une seule fois (`Once`, `unsafe set_var`, voir risque).
  - Racine de test raccourcie (socket Unix limité à 104 octets, `bind` échouait).
  - Client Python : lit jusqu'au `\n`, retire l'espace, rend `SILENT` si aucun octet. `wait_file` attend un contenu non vide (lecture d'un fichier créé mais pas encore écrit).
  - `pid_mort` : le fournisseur lié est un processus séparé, tué avant la connexion du client. Avant, le client était le fournisseur vivant, donc la demande était traitée.
- `crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs`
  - Même fixture `claude` / `.git` / PATH. `AgentRegistry::from_json` fusionne le registre hôte (claude, codex, cursor, gemini avec leurs vrais binaires) : les boucles du catalogue ne portent que sur `native-test` et `claude-child` (`fixture_entries`, 2 entrées exigées).
  - `revocation_de_la_racine` : ajout de la tâche parente (`child = enfant-149`, racine `racine-149`) pour que `for_child` trouve le descendant.
- `crates/bridget-daemon/src/delegation_lineage_tests.rs`
  - `chain_payload` : `origin` du maillon i>0 = `inst-child-{i-1}`.
  - `take_changes()` consommé avant les assertions « aucun changement » (3 tests).
  - Pagination : identifiants UUID (le curseur exige un UUID canonique ; avec `t-a2` toutes les formes étaient refusées, donc le test était vide de sens).
  - `show` : arithmétique UTF-8 corrigée (« héllo wörld » fait 13 octets ; offset 2 est au milieu de `é`, offset 3 est valide).
- `crates/bridget-daemon/src/daemon/native_lineage_tests.rs`
  - Formes `cancel` : les deux champs fournis, un seul invalide.
  - `sessions/<child>` : reliquat supprimé avant et après le test (le dossier est sous le parent du `db_path`, partagé entre exécutions).
  - Fichier journal 0644 : `set_permissions` explicite (l'umask 077 masquait `mode(0o644)`).
- `crates/bridget-daemon/tests/lineage_cli_test.rs`
  - Imports `DirBuilderExt`, `OpenOptionsExt` ; plus de `try_clone` sur `ChildStdout` ; `mut` inutiles retirés.
  - Fixture T3 : aller-retour `CommunicationProjectFact` après le fait de liaison (le fait n'a pas de réponse, la CLI courait avant son traitement : `binding_unavailable` intermittent).
  - Liste : une tâche d'une autre racine est semée, la liste attend les 2 tâches de la racine.
- `crates/bridget-daemon/tests/native_delegation_e2e.rs:280` : `Some(SpawnPosture::Discovery)`.
- `crates/bridget-daemon/src/daemon/native_delegation.rs` (bloc `#[cfg(test)]` seulement), voir ci-dessous.

<a id="extrait-2"></a>

## Extrait 2

- `ensure_claude_on_path` modifie le PATH du processus de test une seule fois, avant tout `Observer::start`. `set_var` reste une écriture d'environnement globale pendant que d'autres tests tournent. Aucun flocon observé sur 2 batteries daemon complètes, mais le risque n'est pas nul.
- Le test `wrapper::reconnect_tests::le_domaine_derive_nomme_le_depot_courant` échoue si `TMPDIR` est sous `/Users/moi` (ce dossier contient un `.git`, le domaine devient `moi`). Avec `TMPDIR=/tmp/b149t` : PASS. La consigne « TMPDIR sous `/Users/moi/.cache` » est donc incompatible avec ce test. Les 4 passes finales utilisent `/tmp/b149t`.
- `Observer` et les fixtures `claude` copient `/bin/echo` : ce sont des fixtures unitaires, jamais une preuve de comportement du modèle ou du CLI Claude réel. T037/T038 restent à faire.
- Non vérifié : comportement des 6 échecs sur HEAD ; recette réelle T3 ; les hooks Claude réels.
- Fichiers que j'aurais pu supprimer et que je garde : aucun.
