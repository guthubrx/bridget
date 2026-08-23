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
