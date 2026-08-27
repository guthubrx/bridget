# Plan 040 — Observateur d'espace disque fédéré

## Dessin retenu

Le fait de capacité suit immédiatement l'inscription réussie depuis la machine
qui lance l'agent. Il est optionnel et horodaté ; le daemon le conserve dans la
présence et le projette dans `AgentInfo`. Cette circulation est
unidirectionnelle : aucune logique de sélection ne la relit.

L'inventaire des arbres reste dans `reaper`, dont la phase est déjà
explicitement observatrice. Le nouveau prédicat ne touche pas
`disk_hygiene::purge_orphan_bridget_tmp`, qui demeure limité à ses copies
`bridget-*` et à sa suppression locale existante.

## Algorithme

1. Le wrapper relève l'espace libre du volume `/` par `statvfs` et l'heure
   UNIX, puis sérialise ce fait juste après `Register`.
2. Le daemon propage l'option dans la présence et chaque projection
   `AgentInfo`; une absence historique reste `None`.
3. `who` affiche une colonne compacte de capacité si elle est attestée, sinon
   un tiret ; le JSON conserve le fait complet.
4. Le reaper lit uniquement la racine donnée à `--tmp`. Pour chaque enfant
   direct, il cherche un nom d'agent avec une frontière `-` ou `.`.
5. Une correspondance avec une présence active est immédiatement protégée.
   Une correspondance arrêtée, injoignable ou absente d'un inventaire
   disponible devient éligible après l'âge minimal. Toute ambiguïté ou
   inventaire indisponible demeure incertain.
6. Le rapport garde son action théorique « JAMAIS exécuté » et ses protections
   existantes. Aucune branche de suppression n'est ajoutée.

## Fichiers prévus

- `crates/bridget-transport/src/protocol.rs` : fait fil optionnel et projection
  d'annuaire.
- `crates/bridget-daemon/src/wrapper.rs` : attestation locale à l'inscription
  et à la reconnexion.
- `crates/bridget-daemon/src/daemon.rs` : conservation dans la présence et
  projection `AgentInfo`.
- `crates/bridget-daemon/src/cli.rs` : colonne informative de l'inventaire.
- `crates/bridget-daemon/src/reaper.rs` : classificateur local non destructif
  et ses trois oracles.

## Validation

- lister les univers de tests avant les exécutions ;
- tests unitaires transport, daemon, CLI et reaper ciblés ;
- mutant retirant la garde d'agent actif ;
- mutant ramenant la racine explicite à `temp_dir()` ;
- vérification que `crates/bridget-daemon/src/disk_hygiene.rs` ne diffère pas ;
- `cargo fmt --check`, `git diff --check`, `cargo check --workspace --all-targets`
  et comparaison base/tête lorsque l'univers est exécutable.
