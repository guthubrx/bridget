# Correctif 149 - limite de descripteurs : rapport de test

Date : 2026-10-11. Résultat : **PASS** (4 PASS, 1 ignoré volontairement, 0 échec).

## Ce qui a été testé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/correctif-limite-fichiers149.md#extrait-1).

Fichier de test : `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/tests/service_file_limit149_test.rs`
(310 lignes, sha256 `cb3b6366c23d77e3e0e04e2f3dbea13bf90341e477d9c855d112508b947702f0`).
La copie dans le clone est identique (même sha256). C'est le seul fichier ajouté au clone.

Commande exécutée :

```
cargo test --locked --offline --release -p bridget-daemon --test service_file_limit149_test -- --test-threads=1
```

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/correctif-limite-fichiers149.md#extrait-2).

## Résultat par test

| Test | Limite imposée à l'enfant | Résultat |
|---|---|---|
| `harness_imposes_the_initial_limit` | soft 256 (`/bin/sh -c 'ulimit -Sn'`) | PASS - le banc impose bien 256 |
| `daemon_started_at_256_raises_soft_to_4096_and_holds_over_256_descriptors` | soft 256, hard inchangé | PASS - journal `soft=4096`, hard identique, pas de WARN ; plus de 256 descripteurs avec 120 clients ; `status` répond |
| `daemon_started_at_8192_keeps_its_soft_limit` | soft 8192 | PASS - `soft=8192` conservée, hard inchangé, pas de WARN |
| `daemon_with_hard_512_is_clamped_to_hard_and_warns` | soft 256, hard 512 | PASS - `soft=512, hard=512`, WARN présent, nombre de descripteurs dans ]256, 512], `status` répond |
| `fixture::performance_daemon_worker` | - | IGNORÉ (fixture existante, gardée ignorée, pas de `--ignored`, pas de banc) |

## Ajustements du test faits pendant l'exécution

1. `open_descriptors` ne compte plus que les champs `lsof -Ff` dont le nom est `f` suivi de chiffres seuls
   (`f0`, `f1`, ...). Avant, il comptait aussi `cwd` et `txt`, qui ne sont pas des descripteurs.
2. Arrêt du daemon : `Drop` local à la place de `cleanup_child` (qui envoie `SIGKILL` sans délai de grâce).
   Il vérifie que le PID est toujours notre enfant direct (`is_owned_running_process`, qui exclut Firefox),
   envoie `SIGTERM` au groupe dédié, puis attend au plus 5 s. En cas d'échec, il le signale et ne force rien.
   **Aucun `SIGKILL` dans le nouveau test.** Le fichier partagé `support/idempotent.rs` n'est pas modifié.

Après les tests : aucun processus orphelin (`ps` filtré sur le target, le TMPDIR et le clone : vide).
Le dossier `/private/tmp/b149fd-tests-l8Gv` est vide.

## Limites réelles et limites isolées

| Cible | soft au départ | hard | Source |
|---|---|---|---|
| Pont réel PID 44934 (avant correctif) | 256 | illimité | `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/limite-fichiers149-avant.json` (FD 255) |
| Daemon réel PID 44728 (avant correctif) | - | - | `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/limite-fichiers149-avant.json` (FD 206) |
| Enfant de test, cas 256 | 256 | celui du parent (illimité ici) | imposé par `setrlimit` entre `fork` et `exec` |
| Enfant de test, cas 8192 | 8192 | inchangé | idem |
| Enfant de test, cas hard 512 | 256 puis 512 | 512 | idem |

Le processus de test ne change jamais sa propre limite. Seul le binaire réel `bridget daemon` s'exécute dans les enfants.

## Ce qui n'est pas prouvé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/correctif-limite-fichiers149.md#extrait-3).

## Revue de source du helper (49 lignes)

- Calcul : `target = max(soft, min(4096, hard))`. Le soft n'est jamais baissé (cas 8192 testé).
  Le plafond au hard est respecté (cas 512 testé). Le hard n'est jamais modifié.
- Deux lectures `getrlimit` (avant et après), un seul `setrlimit`, seulement si le soft est inférieur à la cible.
- Erreurs : message avec l'erreur OS (`getrlimit`/`setrlimit`), et erreur de cohérence si le soft final est
  inférieur à la cible ou si le hard a changé.
- Journal : INFO `soft/hard/budget` ; WARN si hard < 4096. Aucune dépendance ajoutée (`libc` existait).
- Les chemins d'erreur (`getrlimit` ou `setrlimit` en échec) ne sont pas exercés par les tests : le système
  ne permet pas de les provoquer proprement sans simulateur.

## Minimalisme et responsabilité future

- Volume de production : 1 helper de 49 lignes, 2 appelants réels (daemon, pont), une seule contrainte OS. Justifié.
- Lignes de production supprimables à comportement constant : **~0**. Le contrôle de cohérence final (~7 lignes)
  protège contre une limite réellement non appliquée ; je le conserve.
- Tests : 310 lignes (1 fichier), réutilise la fixture existante `support/idempotent.rs`. Aucune nouvelle dépendance.
  Lignes suppressibles : ~25 (le test `harness_imposes_the_initial_limit` valide le banc, pas le produit ; il coûte 13 lignes
  et évite un faux PASS, donc je le garde ; le reste est de l'analyse du journal, nécessaire).
- Charge cognitive future : réduite. Un seul point d'entrée (`ensure_open_file_limit`) remplace une dépendance implicite
  au défaut `launchd` de 256. Le test explique son oracle (3 descripteurs par client, 256 impossible).
- Le code peut être assumé et expliqué : intention (budget 4096 sous le hard), invariants (soft jamais baissé, hard intact),
  limites (pont non testé en exécution, pas de RED).

## Fichiers

- Test : `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/tests/service_file_limit149_test.rs`
- Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/correctif-limite-fichiers149.md`
- Aucun autre fichier modifié ; aucun Git, redémarrage ni modèle utilisé. Fichiers libérés.

## Activation réelle par le principal, après ce contrôle isolé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/correctif-limite-fichiers149.md#extrait-4).
