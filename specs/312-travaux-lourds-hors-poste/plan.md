# Plan d'implémentation — Spécification 312, Lot 1a (compilation release déportée)

- **Spécification** : [spec.md](file:///Users/moi/Nextcloud/10.Scripts/64.bridget/specs/312-travaux-lourds-hors-poste/spec.md) (v4.2, validée à 8/10 sans bloquant)
- **Branche** : `main` (décision branche unique)
- **Objectif du Lot 1a** : déporter automatiquement `cargo build --release` lancé via `scripts/build.py` depuis le poste protégé vers une machine ouverte capable (`macbook-m1` pour `macos-arm64`, `sysle2` pour `linux-x86_64-glibc2.36`), sous politique maître + baux + verrou distant + gardien à battement + contrôles complets, tout en gardant un défaut sûr hors politique/daemon et un entretien des caches compatible Python 3.9.

---

## 1. Découpage par modules et contrats

### 1.1 Protocole transport (`crates/bridget-transport/src/protocol.rs`)
Ajouts additifs, compatibles avant/arrière (aucune rupture de `storage_compatibility.rs` ni de `agents.json`) :
- Types fermés (`#[serde(deny_unknown_fields)]`) :
  - `HeavyWorkRole` : `Open` (`ouverte`), `Protected` (`protégée`), `Excluded` (`exclue`).
  - `HeavyWorkOrigin` : `User` (`user`), `Mission` (`mission`).
  - `HeavyWorkUnavailableAction` : `Wait` (`attendre`), `Fallback` (`repli`).
  - `HeavyWorkFallbackAction` : `Discrete` (`discret`), `Refuse` (`refuser`).
  - `HeavyWorkInstallPolicy` : `Ask` (`demander`), `CatalogTools` (`outils_du_catalogue`), `Never` (`jamais`).
  - `HeavyWorkSshProfile` : `{ host, user, port, identity_rel, known_hosts_rel }` (chemins relatifs à `$HOME`, validés sans `..`, sans chemin absolu, sans caractère de contrôle).
  - `HeavyWorkMachinePolicy` : `{ role, targets: Vec<String>, ssh: Option<HeavyWorkSshProfile>, max_leases: u8 (1..=4), install_policy }`.
  - `HeavyWorkOriginPolicy` : `{ on_busy_or_unreachable, on_no_open_target, fallback_after_wait }`.
  - `HeavyWorkPolicyFile` (version 1) : `{ version: 1, max_wait_secs: u64 (60..=3600), user: HeavyWorkOriginPolicy, mission: HeavyWorkOriginPolicy, machines: BTreeMap<String, HeavyWorkMachinePolicy> }`.
  - `HeavyWorkRequest` :
    - `Status`
    - `Plan { caller_machine, target, origin, requested_host, excluded_machines, preflight_failures }`
    - `Acquire { caller_machine, target, origin, repo_owner, requested_host, excluded_machines, preflight_failures }`
    - `Renew { lease_id }`
    - `Release { lease_id }`
  - `HeavyWorkDecision` (réponse fermée) :
    - `PolicyAbsent` (déport inactif → défaut sûr local discret sous garde cible)
    - `RunRemote { lease_id, expires_at_unix, machine_id, ssh: HeavyWorkSshProfile, reason }`
    - `RunLocalLease { lease_id, expires_at_unix, machine_id, reason }` (appelant lui-même machine `ouverte` retenue : local sous bail sans SSH ni mode discret)
    - `Wait { retry_after_secs, max_wait_secs, candidates, reason }` (cas A avec `attendre`)
    - `FallbackDiscrete { reason, case_kind }` (cas A repli ou cas B `discret`, après vérif que l'appelant n'est pas `exclue`)
    - `Refused { code, message, action, case_kind }` (cas A/B `refuser`, ou `requested_host` `exclue` / sans cible, ou appelant `exclue`)
- Nouveaux variants :
  - `WrapperToDaemon::HeavyWork { request: HeavyWorkRequest }`
  - `DaemonToWrapper::HeavyWorkResult { response: HeavyWorkResponse }`

### 1.2 Politique et gestionnaire de baux (`crates/bridget-daemon/src/heavy_work.rs`)
- **Stockage** : `<namespace_root>/heavy-work-policy.json` (mode `0600`, propriétaire UID courant, non-lien symbolique, max 256 Kio).
- **Défaut sûr** : si le fichier est absent, `Status` indique `configured: false` et `Plan`/`Acquire` rendent `PolicyAbsent` (déport inactif).
- **Écriture locale au maître (`policy`)** :
  - Vérifie l'autorité via `crate::console_ui::resolve_registry_authority` (seul `master_local` avec `writable == true` est accepté ; jamais par socket distante).
  - Supporte `show`, `init`, `set-machine`, `set-global` avec mode aperçu (`--preview`, défaut sûr si non `--apply`) et application atomique (`--apply` : verrou `heavy-work-policy.lock`, sauvegarde horodatée `0600`, écriture temporaire + `fsync` + `rename`, relecture et validation).
- **Sélection et baux (`HeavyWorkLeaseManager` dans `DaemonState`)** :
  - Purge automatique des baux expirés (`expires_at <= now`, TTL = `60 s`) avant chaque opération.
  - Filtre de classe avant tout score (§3.2.3) :
    1. Si `requested_host` (`--host`) est fourni : vérifier qu'il n'est pas `exclue` (sinon `Refused`), qu'il est déclaré (ou connu) et dispose d'un profil SSH s'il n'est pas l'appelant. Vérifier place de bail libre (sinon `Wait` ou `Refused`).
    2. Sinon, lister toutes les machines `ouverte` déclarant `target`.
    3. Retirer celles dans `excluded_machines` (échecs de pré-vérification de l'essai courant).
    4. Parmi les machines `ouverte` restantes, trier par :
       - place de bail libre (`active_leases < max_leases`) en premier ;
       - `compilation_capacity` décroissant (lu depuis `AgentRegistry`, 0 si absent) ;
       - ordre alphabétique de `machine_id`.
    5. Si la meilleure machine ouverte a une place libre : allouer un bail (`lease_id` UUID, TTL `60 s`) → `RunLocalLease` si `machine_id == caller_machine`, sinon `RunRemote`.
    6. Si aucune machine ouverte libre ne reste :
       - Déterminer cas A vs cas B (y compris motifs mixtes de `preflight_failures` §3.2.3.5) :
         - **Cas A** s'il existe au moins une machine ouverte déclarant `target` qui est occupée (baux pleins) OU écartée pour motif `unreachable`/`busy` ;
         - **Cas B** sinon (aucune machine ouverte ne déclare `target`, ou toutes ont échoué sur `prerequisite_missing`).
       - Si l'appelant est `exclue`, tout repli `discret` est transformé en `Refused` (`caller_excluded`).
       - Appliquer le tableau §3.1 selon `origin` (`user` vs `mission`).

### 1.3 CLI et garde délégués (`crates/bridget-daemon/src/cli.rs`)
1. **`bridget build-info --json`** :
   - Grammaire stricte : seul `--json` est accepté (sinon exit 2).
   - Exempté de `command_initializes_namespace` (pas de namespace, pas de daemon).
   - Exempté de `command_requires_principal` (ouvert aux sous-agents délégués).
   - Rend `{"version": "<CARGO_PKG_VERSION>", "build_id": "<BUILD_ID>", "os": "<std::env::consts::OS>", "arch": "<std::env::consts::ARCH>"}`.
   - `bridget --version` / `-v` / `version` strictement inchangés.
2. **Garde CLI sur couple `(commande, sous-commande)`** :
   - `command_requires_principal(command, subcommand)` remplace la garde sur le seul premier mot.
   - `("heavy-work", Some("status" | "plan" | "acquire" | "renew" | "release"))` et `("build-info", _)` → ouverts aux sous-agents délégués.
   - `("heavy-work", Some("policy"))` et `("heavy-work", None | Some(_))` → fermés aux sous-agents délégués (`DelegatedMcpOnly`).
3. **Résolution de l'origine (§3.1, fail-closed)** :
   - `resolve_heavy_work_origin()` vérifie dans l'ordre :
     1. `has_delegated_marker(&namespace.root.join("delegated-pids"), pid, &SystemProcessTree)` ;
     2. `crate::mcp_identity::resolve_current_identity()` (tout `Ok` ou `Err(DelegatedMcpOnly)` → `Mission`) ;
     3. `crate::mcp_identity::resolve_current_mcp_identity()` et `crate::t3code_mcp::resolve_current()` (tout `Some(Ok(_))` ou erreur d'attestation hors absence → `Mission`).
   - Seule l'absence confirmée de tout contexte agent, T3 et délégué rend `HeavyWorkOrigin::User`.
4. **Sous-commandes `bridget heavy-work`** :
   - `status [--json]`
   - `plan --dry-run --target <cible> [--host <machine>] [--exclude <machine>:<motif>] [--json]`
   - `acquire --target <cible> --repo-owner <sha> [--host <machine>] [--exclude <machine>:<motif>] [--json]`
   - `renew --lease <id> [--json]`
   - `release --lease <id> [--json]`
   - `policy show | init | set-machine | set-global [--preview | --apply] [--json]` (local au maître uniquement).

### 1.4 Lanceur et entretien `scripts/build.py` (compatible Python 3.9+)
1. **Entretien des caches Python 3.9** (§4.5) :
   - Suppression sûre sans `shutil.rmtree(dir_fd=...)` : parcours récursif ancré par descripteurs (`os.open(..., os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)` + `os.listdir(fd)` + `os.stat(name, dir_fd=fd, follow_symlinks=False)` + `os.unlink`/`os.rmdir(..., dir_fd=fd)`). Jamais de suivi de lien symbolique.
   - `digest(path)` par lecture en blocs de 64 Kio (`hashlib.sha256().update(...)`) au lieu de `hashlib.file_digest`.
   - Retrait de la garde `sys.version_info < (3, 11)`.
   - Tout échec d'entretien est journalisé et repris dans le bilan final, sans faire échouer une compilation réussie.
2. **Propriétaire canonique et racine distante** (§3.2.4) :
   - `canonical_owner_id(repo)` : `BRIDGET_BUILD_OWNER` si présent et valide (SHA 40/64 hex), sinon SHA du premier commit racine (`git rev-list --max-parents=0 HEAD`), repli sur `legacy_owner_id(repo)` uniquement hors Git sans env.
   - `owned_staging(repo)` reconnaît **à la fois** `canonical_owner_id(repo)` et `legacy_owner_id(repo)` (migration transparente des caches existants).
   - `configure_build_target(repo, arguments)` conserve exactement sa clé locale existante (`target-dir` inchangé pour `Makefile`).
   - `remote_cache_root()` : si `~/.config/bridget/build.json` existe → `external_staging_parent()` (refus sans repli si invalide) ; sinon `~/.cache`. Dossier distant : `bridget-build-<repo_slug>-<target>`.
3. **Options CLI de `scripts/build.py cargo`** :
   - Nouvelles options avant la sous-commande Cargo (`cargo [--local] [--host <machine>] [--build-target <cible>] [--ssh-host ...] [--ssh-user ...] [--ssh-port ...] [--ssh-key ...] [--ssh-known-hosts ...] <args cargo>`).
   - Périmètre Lot 1a : seul `build` avec `--release` est candidat au déport ; `build` debug, `check`, `clippy`, `test` restent locaux (en mode discret si besoin ou normal).
4. **Garde de la compilation locale et mode discret** (§3.3.2) :
   - Cible par défaut = cible native de la machine appelante (`macos-arm64` sur macOS arm64, `linux-x86_64-glibc2.36` sur Linux x86_64).
   - Avant toute compilation locale (`--local`, défaut sûr sans politique/daemon, ou repli `discret`) : vérifier que la cible est produisible en local (même OS, même arch, glibc locale <= cible sur Linux). Sinon refus immédiat (exit 2) avec cause et action (`--host`).
   - Mode discret : `taskpolicy -b` sur macOS ; `nice -n 15` (et `ionice -c 3` si présent) sur Linux ; `CARGO_BUILD_JOBS = max(1, min(2, cpu_count // 4))`.
5. **Exécution distante vérifiée** (§3.2.4) :
   - Vérification Git source : aucun fichier suivi modifié (`git diff --quiet HEAD` et `git diff --cached --quiet`), commit `HEAD` ancêtre de la branche distante suivie (`@{u}` ou `origin/main` ou `github/main` sans fetch implicite). Avertissement si fichiers non suivis présents.
   - Création d'archive `git archive --format=tar <HEAD>`, calcul SHA-256 local.
   - Profil SSH : résolution de `identity_rel` et `known_hosts_rel` sous `$HOME` de l'appelant, contrôle `0600` + non-symlink ; options SSH strictes (`-F /dev/null`, `-o BatchMode=yes`, `-o StrictHostKeyChecking=yes`, `-o IdentitiesOnly=yes`, `-o ForwardAgent=no`, `-o ServerAliveInterval=15`, `-o ServerAliveCountMax=3`).
   - Pré-vérification SSH rapide (OS, arch, glibc sur Linux, version exacte de `rust-toolchain.toml`, espace libre et résolution de la racine de cache distante). Si échec : `release` du bail, ajout à `excluded_machines` avec motif (`unreachable` vs `prerequisite_missing`), nouvel appel `acquire`.
   - Gardien distant Python (déposé comme fichier dans un dossier privé `0700` distant et exécuté par son chemin, jamais `bash -s`) :
     - vérifie SHA-256 de l'archive puis extrait avec `--no-same-owner` ;
     - acquiert `fcntl.flock` bloquant sur `<cache>/.heavy-work.lock` ;
     - lance `cargo` (enveloppé par `caffeinate -i` sur macOS) dans son propre groupe (`start_new_session=True`), `stdin=DEVNULL`, `BRIDGET_BUILD_ID=<sha12>`, `BRIDGET_BUILD_OWNER=<root_sha>`, `MACOSX_DEPLOYMENT_TARGET=11.0` (macOS) ;
     - lit un battement toutes les `10 s` sur son `stdin` (envoyé par un thread du lanceur) ; sur EOF ou `30 s` sans battement, envoie `SIGTERM` au groupe puis `SIGKILL` `10 s` plus tard ;
     - lance les contrôles sur l'hôte distant avant retour : `scripts/check-linux-abi.py` (Linux) ou `codesign --verify --strict` (macOS), démarrage + `build-info --json` (vérifie `build_id == <sha12>`), SHA-256 du binaire ;
     - nettoie uniquement le dossier source temporaire (jamais le cache ni `.heavy-work.lock`).
   - Thread de renouvellement de bail toutes les `20 s` côté lanceur, `release` garanti en `finally`.
   - Rapatriement du binaire, re-vérification SHA-256 après transfert, pose atomique à :
     - `$CARGO_TARGET_DIR/release/bridget` + `register_staging` si cible native ;
     - `$CARGO_TARGET_DIR/<cible>/release/bridget` si cible étrangère (`--build-target`).
   - Bilan clair (machine, motif de choix, durées par phase : archive/envoi, compilation, contrôles, retour, état d'entretien).

---

## 2. Plan de tests (Lot 1a)

### Tests Rust (`bridget-daemon` + `bridget-transport`)
1. `build-info --json` : sortie JSON exacte, refus d'arguments inconnus, `--version` inchangé, pas d'initialisation de namespace.
2. Garde CLI sur `(commande, sous-commande)` : un sous-agent délégué passe sur `build-info` et `heavy-work status|plan|acquire|renew|release`, mais est refusé (`delegated_mcp_only`) sur `heavy-work policy`.
3. Origine : `mission` si marqueur `delegated-pids`, identité agent ou session T3 présente ; `user` uniquement en absence totale.
4. Politique `heavy-work-policy.json` : défaut sûr si absent (`PolicyAbsent`), validation stricte (`deny_unknown_fields`, baux 1..=4, attente 60..=3600, profil SSH relatif sans `..`), écriture refusée hors maître local, sauvegarde + écriture atomique + relecture sur maître local.
5. Sélection et baux :
   - filtre de classe (`ouverte` bat toujours `protégée`, même avec `compilation_capacity` inférieur) ;
   - `exclue` jamais choisie (y compris avec `--host`, et refuse le repli local si appelant `exclue`) ;
   - tri déterministe (bail libre → `compilation_capacity` → ordre alphabétique) ;
   - baux simultanés et expiration à `60 s` + renouvellement ;
   - bascule après échec de pré-vérification et motifs mixtes (cas A si au moins une injoignable/occupée, cas B si toutes prérequis absent) ;
   - appelant choisi (`RunLocalLease`).

### Tests Python (`scripts/tests/test_heavy_work_312.py` + `test_build_cleanup.py`)
1. Suppression sûre par descripteurs `O_NOFOLLOW | O_DIRECTORY` (refuse un symlink sans le suivre, fonctionne sur Python 3.9+ sans `shutil.rmtree(dir_fd=...)`).
2. `digest()` par blocs (compatible Python 3.9+).
3. `BRIDGET_BUILD_OWNER` canonique (SHA commit racine) + double reconnaissance dans `owned_staging` avec l'ancien chemin `.git` + `target-dir` inchangé + refus sans repli si `build.json` invalide.
4. Garde de compilation locale et cible étrangère (`--build-target linux-x86_64-glibc2.36 --local` refusé sur macOS ; binaire étranger posé sous `$CARGO_TARGET_DIR/<cible>/release/bridget`).
5. Gardien distant (test réel local sur processus factice) : battement OK, coupure stdin → `SIGTERM` du groupe en < 2 s (timeout raccourci en test), sérialisation `fcntl.flock` de deux appels concurrents.
6. Validation de source Git (arbre sale refusé, commit non poussé refusé, fichier non suivi averti et exclu de l'archive).
7. Mode `--host` sans daemon (exige paramètres SSH explicites, avertit que `exclue` n'est pas vérifiable).

### Validation réelle de bout en bout (après livraison)
- Déploiement de la version sur les 5 machines (`cartae`, `ksstore`, `sysle2`, `monordinateur`, `macbook-m1`).
- Initialisation de `heavy-work-policy.json` sur `cartae` (`monordinateur` = `protégée`, `macbook-m1` = `ouverte` `macos-arm64`, `sysle2` = `ouverte` `linux-x86_64-glibc2.36`).
- Essai réel `python3 scripts/build.py cargo build --release` depuis `monordinateur` : zéro `cargo`/`rustc` local, CPU local < 5 s, binaire Mac produit sur `macbook-m1` vérifié par `codesign`, `build-info --json` et SHA-256.
