# Validation r1 — tests permissions natives 149 (sous-agent test GP)

Date : 2026-10-10. Ronde : r1. Périmètre : ownership exclusif des 5 fichiers listés
plus bas. Aucun fichier L lu, édité ou compilé volontairement. Aucun modèle réel
lancé. Aucun correctif production ni script.

## Verdict

| Vérification | Commande exacte | Résultat réel |
|---|---|---|
| Check transport ciblé (mes fichiers) | `cargo check -p bridget-transport --test native_permissions149_test --test native_permissions149_e2e` | **PASS** — Finished, 0 erreur, 0 warning |
| Test transport ciblé (mes fichiers) | `cargo test -p bridget-transport --test native_permissions149_test --test native_permissions149_e2e` | **PASS** — 12/12 : 8 + 4, 0 failed |
| Check daemon lib en mode test | `cargo check -p bridget-daemon --lib --profile test` | **BLOQUÉ hors ownership** — 11 erreurs, aucune dans mes 3 fichiers daemon |
| Check transport complet (--tests) | `cargo check -p bridget-transport --tests` | **BLOQUÉ hors ownership** — 2 E0063 dans vieilles fixtures inline `codex_app_server.rs` |
| Exécution des tests daemon | — | Non lancée. Prérequis non levés (voir « Prérequis mécaniques ») |

Environnement build : `CARGO_TARGET_DIR=/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target`,
`CARGO_BUILD_JOBS=2`, `CARGO_INCREMENTAL=0`, umask 077. Aucun target interne créé.
Pas de réseau, pas de modification Cargo.toml/Cargo.lock.

## Fichiers livrés (ownership GP)

1. `crates/bridget-transport/tests/native_permissions149_test.rs` — 8 tests, compile et passe.
2. `crates/bridget-transport/tests/native_permissions149_e2e.rs` — 4 tests, compile et passe.
3. `crates/bridget-daemon/src/native_permissions149_tests.rs` — 11 tests, compile (0 erreur propre à ce fichier).
4. `crates/bridget-daemon/src/native_permission_observer149_tests.rs` — 9 tests, compile (idem).
5. `crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs` — 9 tests, compile (idem).
6. 3 déclarations `#[cfg(test)] #[path=...] mod` ajoutées en fin de
   `src/native_permissions.rs`, `src/native_permission_observer.rs`,
   `src/daemon/native_delegation.rs` (droit accordé par la mission).

Total : 41 tests écrits. Les 12 transport s'exécutent pour de vrai.

## Couverture des oracles

- **S149-29 / formes fermées du fait** (fichier 1) : `validate_permissions`
  accepte codex readOnly / workspaceWrite+networkAccess / externalSandbox / dangerFullAccess,
  refuse le reste ; claude : kind, permission_mode, tools (liste ou preset claude_code),
  callback native_wrapper tool_approval prompt|allow plan_exit deny,
  settings_sources provider_default ; bornes identité (256), révision 1..=HUMAN_LINEAGE_MAX_SEQ,
  taille du fait ≤ 64 Kio ; cwd absolu ; sources dupliquées ou hors vocabulaire refusées.
- **S149-30 / launch_context** (fichier 1) : chemins absolus obligatoires pour
  cli_path / resolved_cli_path / config_dir, révisions digest sha256:hex[64],
  permission_sources ≤ 32 formes fermées kind∈{user,project,local,managed,cli},
  settings_sources ⊆ {user,project,local} sans doublon, settings_overrides
  fermé sur `permissions` (allow/ask/deny = règles atomiques ≤ 512 ;
  additional_directories = chemins absolus ; defaultMode ∈ vocabulaire ;
  disableBypassPermissionsMode ∈ {disable}) ; clé étrangère → `permission_source_unavailable`.
- **S149-31 / révision de source** (fichier 1) : fichier stable → révision stable,
  mutation → révision muée, absent → `"absent"`, symlink et sous-dossier →
  `permission_source_unavailable`, dossier → révision triée indépendante de l'ordre.
- **S149-32 / recheck** (fichier 1) : `recheck_permission_context_sources` refuse
  `settings_revision_changed` sur mutation, suppression, apparition et remplacement CLI ;
  `recheck_frozen_inputs` re-résout le launcher dans le PATH fourni, refuse la
  divergence, refuse une commande multi-composants non absolue, et refuse
  `permission_attestation_unavailable` sur des sources non JSON.
- **E2E can_use_tool** (fichier 2) : deny corrélé par request_id exact →
  `DeliveryRejected{reason:"provider_permission_denied"}`, jamais `TurnFinished`,
  la réponse `control_response` écrite sur stdin porte le même request_id et
  `behavior:"deny"` ; remise positive → `TurnFinished` Completed sans refus ;
  result avec `permission_denials` → refus sans trame ; control_request sans
  request_id → aucune réponse, remise refusée. Processus réels (`/bin/sh` factice),
  aucun modèle.
- **Daemon catalogue + admission** (fichier 3) : 7 clés top-level exactes triées,
  inherit/development par agent selon sandbox et posture, refus nommés en `failure()`
  JSON exact, posture omise → development sur fait full, v1 sans inherit →
  `permission_attestation_unavailable`, fait d'une autre demande → refus,
  retombée du fait → refus sans invalider Status/Cancel en cours.
- **Observer CLI** (fichier 4) : settings préexistant → refus sans trace,
  cycle de vie 0600 + RAII, fait publié → ACK true avant reprise du hook,
  ACK false → blocage nommé, overlay mué → `permission_attestation_unavailable`,
  request_id absent → refus sans fait, nonce inconnu et pair mort → silence
  sans récupération, PID mort → `permission_source_unavailable`, filtre session,
  capacities informationnel corrélé.
- **Délégation native** (fichier 5) : catalogue full/lecteur, refus lecteur-dev
  exact, posture omise, v1 no-inherit et fait périssable, fait d'une autre demande,
  révocation racine → `permission_not_inherited` au parent_fact ET au spawn_task,
  entrées gelées : admission + spawn OK avant mutation, puis
  `settings_revision_changed` au spawn, à l'admission après mutation, et à
  l'admission après suppression.

## Corrections d'oracle faites cette ronde (mes fichiers uniquement)

1. `allow:["relative"]` attendu refusé → corrigé. La prod valide allow/ask/deny
   comme règles atomiques (`array(v,false)`), pas comme chemins ; seul
   `additional_directories` exige l'absolu. Ajout d'un vrai cas de fermeture
   (`allow:[42]` refusé). Preuve : `crates/bridget-transport/src/native_permissions.rs:96` vs `:78`.
2. `recheck_frozen_inputs` : le test gelait le launch_context nu. La prod lit
   `BRIDGET_NATIVE_CHILD_POLICY` comme policy complet et en extrait
   `policy["launch_context"]` (`src/native_permissions.rs:122`) ; sans enveloppe,
   early-return `Ok`. Le test gèle désormais `{"launch_context": {...}}` avec un
   guard Drop qui nettoie l'env même si un assert panique.

Aucune attente de contrat n'a été affaiblie : les deux corrections portent sur la
forme des données d'entrée de mes tests, pas sur les refus nommés attendus.

## Divergence constatée (documentée, non contredite par le contrat)

Le recheck `settings_revision_changed` pour claude→claude se déclenche aussi À
L'ADMISSION (child_policy appelle recheck_context avant filetage), pas seulement
au spawn_task. Les deux points de contrôle sont couverts par le test « entrées
gelées » : admission avant mutation OK + spawn OK, puis refus au spawn après
mutation, refus à l'admission après mutation, refus à l'admission après
suppression. Launch count inchangé sur refus : aucun lancement.

## Bugs production à rapporter à Sol (aucun patch appliqué)

1. **E0282 ×2 — `crates/bridget-daemon/src/native_permission_observer.rs:54` et `:94`**
   (bloque toute CU lib-test du daemon) :
   - :54 `let acknowledgements=Arc::new((Mutex::new(None),Condvar::new()));`
     type non inféré. Fix suggéré : `None::<(String,bool)>` ou annotation équivalente.
   - :94 cascade `wait_timeout(...).unwrap_or_else(|e|e.into_inner())` — même cause.
2. **E0063 ×2 — `crates/bridget-transport/src/codex_app_server.rs:3816` et `:4120`**
   (vieilles fixtures inline cfg(test), bloquent `cargo check -p bridget-transport --tests`) :
   `ReaderContext` construit sans le nouveau champ `permission_observation`.

## Prérequis mécaniques avant `cargo test` daemon (à traiter par GLM prep L)

7× E0308 + 1× E0061 dans les fixtures 148 inline de
`crates/bridget-daemon/src/daemon/native_delegation.rs` (posture attendue
`Option<SpawnPosture>`) : lignes 1262, 1378, 1395, 1415, 1439, 1958, 2155 (E0308)
et 1720 (E0061, `handle_locked_with_project` prend désormais 5 args — attested
manquant). Fix mécanique : `Some(...)` / `None` selon le cas, conformes au nouveau
contrat. Warning associé : `delegation.rs:10:16` unused import `lineage::task_entry`.

Constats passifs (hors ownership, aucune action de ma part) : 5 warnings `mut`
inutile dans `daemon/native_lineage_tests.rs` (fichier L, compilé passivement car
même CU — inévitable via `--lib --profile test`).

## Mesures

- df `/Volumes/8TB2` avant : 153Gi disponibles (96% occupé). Après : 153Gi —
  aucun changement significatif.
- Taille du target externe : **607M** (`du -sh`), 11 artefacts `native_permissions149`.
- Runtimes : check transport initial ~5,5 s puis ~0,2 s ; check daemon ~12 s par
  itération ; test transport complet 8,5 s (dominé par l'E2E, polls 8 s).

## Next actions

1. Sol : lever les 2 E0282 de `native_permission_observer.rs` (fix suggéré ci-dessus)
   et les 2 E0063 de `codex_app_server.rs` (fixtures inline) — alors
   `cargo check -p bridget-transport --tests` et la CU lib-test daemon passent.
2. GLM prep L : adapter les 7 E0308 + 1 E0061 des fixtures 148 inline
   (`native_delegation.rs`) au contrat posture Option.
3. Ensuite seulement : `cargo test -p bridget-daemon --lib` pour exécuter mes
   29 tests daemon (11 + 9 + 9). Le check prouve leur compilation ; l'exécution
   reste à faire après prérequis.
4. Mes 12 tests transport sont exécutables dès maintenant et passent.
