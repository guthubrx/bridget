# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/quickstart.md](../../../../../../specs/156-guidage-file/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Tests du crate transport : même environnement, `cargo test -p bridget-transport <filtre>`.
Suite complète avant clôture : même environnement,
`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast`.
Une modification de texte injecté ou de trame exige aussi tous les tests
`t3code`, `mcp::` et transport, pas seulement les nouveaux.

<a id="extrait-2"></a>

## Extrait 2

1. Table des droits : filtre `guidance156_matrix`.
2. Intention à l'envoi (MCP, CLI, daemon, canon) : filtres `guidance156_mcp`, `guidance156_cli`, `guidance156_send`.
3. Garde sur les chemins, origine, autorité humaine, contrôle : filtres `guidance156_guard`, `guidance156_origin`, `guidance156_human`, `guidance156_control`.
4. Pont T3 (`steer`, `restart`, hors lot, tête bloquée) : filtre `guidance156_t3`.
5. Anti-boucle : filtre `guidance156_rate`.
6. Pause : filtre `guidance156_pause`.
7. Enfant en mission : filtre `guidance156_mission`.
8. Guidage implicite T3, annulation, tests combinés : filtres `guidance156_midturn`, `guidance156_cancel`, `guidance156_combo`.
9. Lecture de file : filtre `guidance156_queue`.
10. Bout en bout : `cargo test -p bridget-daemon --features test-support --test guidance156_e2e`.
11. Non-régression : filtres `spec_087`, `native148`, `native149`, `native150`, `t3code`, `mcp::`.

<a id="extrait-3"></a>

## Extrait 3

Aucun service n’est relancé pendant la préparation documentaire. La série
152–160 autorise build signé, installation et activation Bridget après validation,
avec zéro mission native non terminale, puis nettoyage. T3 reste sans relance
explicite. Aucune nouvelle permission n’est nécessaire. Après cette activation,
les commandes ci-dessous viennent d’un parent attesté dans Bridget ;
un terminal nu est refusé identity_unattested. Steer avec --reply est refusé
invalid_intent. Aucun from=humain ne vaut preuve. Après relance :
