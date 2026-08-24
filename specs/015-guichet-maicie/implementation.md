# Implémentation 015 — Guichet Maicie

## T1513 — Non-régression totale du 2026-08-23

**Tête validée** : `69ad00d`
**Verdict** : **STOP** — tests et Clippy verts, mais le contrôle de formatage
est rouge. Aucun correctif ni formatage n'a été appliqué dans cette tâche de
validation.

### Préparation du binaire des coutures réelles

Commande exécutée :

```bash
/usr/bin/time -p /Users/moi/.cargo/bin/cargo build -p bridget-daemon --bin bridget
```

Résultat : PASS, binaire exécutable produit à
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/debug/bridget`.
Durée réelle : **0,21 s**.

### 1. Workspace complet

Commande exécutée :

```bash
/usr/bin/time -p /Users/moi/.cargo/bin/cargo test --workspace
```

Résultat : **PASS**, zéro échec. Les trois coutures réelles T1513 sont bien
signalées `ignored` par la passe par défaut et ont donc été lancées séparément
ci-dessous. Durée réelle : **190,41 s**.

### 2. Coutures `#[ignore]` explicites

#### Client public Maicie ↔ daemon réel

Commande exécutée :

```bash
BRIDGET_MVP_GATE_BIN=/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/debug/bridget /usr/bin/time -p /Users/moi/.cargo/bin/cargo test -p maicie --test guichet_client_contract 'guichet_client::client_public_et_daemon_reel_partagent_la_negociation_canonique' -- --ignored --exact --nocapture
```

Résultat : **PASS**, 1 passé, 0 échec. Durée réelle : **0,25 s**.

#### Gate réel G1504

Commande exécutée :

```bash
BRIDGET_MVP_GATE_BIN=/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/debug/bridget /usr/bin/time -p /Users/moi/.cargo/bin/cargo test -p maicie --test guichet_gate_integration 'guichet_gate::parcours_reel_g1504_releve_une_lettre_et_ne_la_duplique_pas' -- --ignored --exact --nocapture
```

Résultat : **PASS**, 1 passé, 0 échec ; observable interne
`dépôt absent→relève→greffe→answered=909 ms`. Durée réelle : **1,23 s**.

#### Gate MVP Maicie

Commande exécutée :

```bash
BRIDGET_MVP_GATE_BIN=/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/debug/bridget /usr/bin/time -p /Users/moi/.cargo/bin/cargo test -p maicie --test mvp_gate 'delegation_reelle_est_accusee_et_visible_sans_fausse_correlation_de_reponse' -- --ignored --exact --nocapture
```

Résultat : **PASS**, 1 passé, 0 échec ; observables internes
`livraison+ACK+status=1659 ms` et `clôture=1716 ms`. Durée réelle : **2,16 s**.

### 3. Clippy workspace

Commande exécutée :

```bash
/usr/bin/time -p /Users/moi/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
```

Résultat : **PASS**, zéro warning. Durée réelle : **4,46 s**.

### 4. Formatage

Commande exécutée :

```bash
/usr/bin/time -p /Users/moi/.cargo/bin/cargo fmt --all --check
```

Résultat : **ÉCHEC**, durée réelle : **0,82 s**. Premier écart brut :

```text
Diff in /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/crates/bridget-daemon/src/cli.rs:950:
     }
     validate_agent_name(&from)?;
     let retry = idempotent_options(id, issued_at, issuer_scope)?;
-    let scope_identity = std::env::var("BRIDGET_AGENT_INSTANCE_ID").unwrap_or_else(|_| from.clone());
+    let scope_identity =
+        std::env::var("BRIDGET_AGENT_INSTANCE_ID").unwrap_or_else(|_| from.clone());
```

Le contrôle signale ensuite d'autres écarts de formatage dans les fichiers
Rust de la tête 015. Conformément au mandat T1513, la validation s'arrête sur
ce rouge et ne modifie aucun fichier de production.

## T1513-bis — Solder le contrôle Rustfmt

**Commit code dédié** : `f0c682c`
**Verdict** : **PASS** — les trois commandes finales sont vertes.

### Audit du diff de formatage

Commande appliquée au worktree :

```bash
/Users/moi/.cargo/bin/cargo fmt --all
```

Rustfmt 1.8.0-stable a modifié 16 fichiers Rust. Pour exclure tout changement
étranger au formateur, une archive propre de `HEAD` a été extraite dans un
répertoire temporaire, formatée indépendamment avec le même
`rust-toolchain.toml`, puis comparée fichier par fichier avec `cmp`. Résultat :

```text
fichiers_audites=16
AUDIT_RUSTFMT_IDENTIQUE
```

Le commit `f0c682c` contient exclusivement ces 16 fichiers `.rs`. Les WIP
documentaires préexistants restent hors index.

### Validations finales

Commande exécutée :

```bash
/usr/bin/time -p /Users/moi/.cargo/bin/cargo test --workspace
```

Résultat : **PASS**, zéro échec. Durée réelle : **191,61 s**.

Commande exécutée :

```bash
/usr/bin/time -p /Users/moi/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
```

Résultat : **PASS**, zéro warning. Durée réelle : **5,96 s**.

Commande exécutée :

```bash
/usr/bin/time -p /Users/moi/.cargo/bin/cargo fmt --all --check
```

Résultat : **PASS**, aucun diff. Durée réelle : **0,66 s**.

## Non-régression finale avant merge du 2026-08-24

**Tête validée** : `7c9a1522a5920eededaccdad0593032ca27a6616`
**Verdict** : **STOP** — workspace, coutures réelles et Clippy sont verts,
mais le contrôle Rustfmt est rouge sur le test de nettoyage G1504 ajouté après
la passe T1513 précédente. Aucun fichier Rust n'a été corrigé ou reformaté.

### Préparation du binaire réel

Commande exécutée :

```bash
PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo build -p bridget-daemon --bin bridget
```

Résultat : **PASS**. Le binaire exact du worktree est
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/debug/bridget`.
Durée réelle : **6,26 s**.

### 1. Workspace complet

Commande exécutée :

```bash
PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo test --workspace
```

Résultat : **PASS**, zéro échec. Durée réelle : **181,30 s**.

### 2. Coutures `#[ignore]` explicites

#### Client public Maicie ↔ daemon réel

```bash
BRIDGET_MVP_GATE_BIN=/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/debug/bridget PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo test -p maicie --test guichet_client_contract 'guichet_client::client_public_et_daemon_reel_partagent_la_negociation_canonique' -- --ignored --exact --nocapture
```

Résultat : **PASS**, 1 passé, 0 échec. Durée réelle : **0,24 s**.

#### Gate réel G1504 — parcours nominal

```bash
BRIDGET_MVP_GATE_BIN=/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/debug/bridget PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo test -p maicie --test guichet_gate_integration 'guichet_gate::parcours_reel_g1504_releve_une_lettre_et_ne_la_duplique_pas' -- --ignored --exact --nocapture
```

Résultat : **PASS**, 1 passé, 0 échec ; observable interne
`dépôt absent→relève→greffe→answered=860 ms`. Durée réelle : **1,39 s**.

#### Gate réel G1504 — échec injecté après le spawn

```bash
BRIDGET_MVP_GATE_BIN=/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/debug/bridget PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo test -p maicie --test guichet_gate_integration 'guichet_gate::g1504_nettoie_le_groupe_apres_un_echec_injecte' -- --ignored --exact --nocapture
```

Résultat : **PASS**, 1 passé, 0 échec. La panique
`échec G1504 injecté après le spawn : la garde doit nettoyer` est provoquée et
capturée par l'oracle ; le test se termine `ok`. Durée réelle : **1,44 s**.

#### Gate MVP Maicie

```bash
BRIDGET_MVP_GATE_BIN=/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/debug/bridget PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo test -p maicie --test mvp_gate 'delegation_reelle_est_accusee_et_visible_sans_fausse_correlation_de_reponse' -- --ignored --exact --nocapture
```

Résultat : **PASS**, 1 passé, 0 échec ; observables internes
`livraison+ACK+status=1619 ms` et `clôture=1672 ms`. Durée réelle : **2,04 s**.

### 3. Clippy workspace

```bash
PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo clippy --workspace --all-targets -- -D warnings
```

Résultat : **PASS**, zéro warning. Durée réelle : **0,39 s**.

### 4. Formatage

```bash
PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo fmt --all --check
```

Résultat : **ÉCHEC**, durée réelle : **0,69 s**. Sortie brute :

```text
Diff in /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/plugins/maicie/tests/integration/guichet_gate.rs:717:
         .unwrap();
     let before = managed_g1504_process_count();
     let failed = catch_unwind(AssertUnwindSafe(|| run_g1504(true)));
-    assert!(failed.is_err(), "la branche d'échec doit réellement paniquer");
+    assert!(
+        failed.is_err(),
+        "la branche d'échec doit réellement paniquer"
+    );
     assert_managed_g1504_process_count(before);
 }

real 0.69
user 0.60
sys 0.04
```

### 5. Absence de nouvelle fuite G1504

Le comptage reprend exactement le motif du harnais :

```bash
ps -axo pid=,ppid=,pgid=,command= | awk -v a=managed-wrapper -v b=g1504_fixture -v c=g1504-agent 'index($0,a" "b" "c){print; n++} END{print "G1504_COUNT=" n+0}'
```

Résultat : **15 avant, 15 après, delta 0**. Les quinze processus antérieurs à
cette passe n'ont pas été touchés ; les parcours nominal et injecté n'ont créé
aucun orphelin supplémentaire.

## Soudure du formatage et de la barrière de test

**Commit de style séparé** : `d0ed0bf`
**Verdict** : **PASS** — la mise en forme G1504 et la barrière de contenu sont
validées ; workspace, Clippy et Rustfmt sont verts.

Le premier rejeu du workspace après le seul formatage a révélé la course
héritée suivante : `wait_for_path` observait la création de `descendant.pid`
avant l'écriture de son contenu, puis le test tentait de parser une chaîne vide.
Le helper de test attend désormais, sous la même échéance de cinq secondes, une
lecture non vide que le consommateur sait parser. Ses deux appelants concernés,
le PID descendant et `boundary.json`, utilisent cette même barrière.

### Preuve ciblée 10/10

Commande exécutée :

```bash
for run_index in {1..10}; do PATH=/Users/moi/.cargo/bin:$PATH cargo test -p bridget-daemon --lib 'managed_process::tests::stop_force_termine_l_intermediaire_npx_qui_ignore_l_annulation_et_son_descendant' -- --exact || exit 1; done
```

Résultat : **PASS**, dix exécutions consécutives, dix réussites, zéro échec.

### Validations finales

```bash
PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo test --workspace
```

Résultat : **PASS**, zéro échec. Durée réelle : **185,97 s**.

```bash
PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo clippy --workspace --all-targets -- -D warnings
```

Résultat : **PASS**, zéro warning. Durée réelle : **2,67 s**.

```bash
PATH=/Users/moi/.cargo/bin:$PATH /usr/bin/time -p cargo fmt --all --check
```

Résultat : **PASS**, aucun diff. Durée réelle : **0,69 s**.

Les gates `#[ignore]` ne sont pas rejoués dans cette passe : ils ont réussi sur
le même code fonctionnel juste avant `d0ed0bf`, et les deux changements depuis
ne touchent que la mise en forme du harnais G1504 et une barrière interne aux
tests de `managed_process.rs`.
