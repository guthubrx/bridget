# Tests natifs 149 - r9 : état d'avancement (2026-10-10, ~20:50)

Aucun fichier de production modifié. Aucune mutation supplémentaire ne sera lancée.

## Déjà obtenu (faits)

- Nouveaux tests daemon (lib) : 17 sur 17 PASS (E2 : 3, F3 : 5, O4 : 9). Fichier : `crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs` (début octet-identique à r8, ajout en fin de fichier, rustfmt sur la seule région ajoutée).
- Nouveau test transport : `crates/bridget-transport/tests/native_alias149_test.rs`, 2 sur 2 PASS (alias privé accepté puis retiré à l'arrêt ; 6 alias dangereux refusés avant lancement).
- Fixture réelle `codex_interactive_090.py` adaptée au nouvel alias (hors BRIDGET_HOME, sous `/private/tmp/bridget-codex-*`). Avec le vrai Codex 0.161.0 et un fournisseur HTTP local : 4 recettes PASS (menu annulé, reprise absente, nom absent, nom ambigu), alias présent puis retiré avec son dossier.
- Mutations : 21 mutations de production tuées (E1-E4, F1-F3, O1-O8, I1-I3, A1-A3) + W1 (ancien alias dans BRIDGET_HOME, tuée par la recette réelle). 2 survivants du premier passage (E4, O7) corrigés par durcissement de test, puis tués. Témoins sans mutation : verts.

## Échecs réels / limites

- Recette `--new-thread` (et toute recette qui attend le rendu de la TUI) : TIMEOUT. Contrôle : ancienne fixture + binaire r8 + Codex 0.161.0 donne le même timeout. Cause : dérive de la recette face à Codex 0.161.0 (attend l'affichage du mot "fixture" ; elle date de 0.153.4). Sans lien avec l'alias. Le contrat d'alias est validé avant ce point (`same_thread` atteint).
- Non testable en unitaire : `CodexInteractiveEndpoint` du wrapper (privé) ; couvert seulement par les recettes réelles ci-dessus. Le redémarrage parent vivant avec TUI reste à prouver par la tâche réseau séparée.
- Chemin `native_execution_changed` (CAS refusé en cours d'appel) : exige un écrivain concurrent ; contrat du store testé (r8).
- Ordre de `run()` (état durable lu avant réconciliation, grâce 8 s) : relu dans la source, non exécuté.
- Écart de procédure : j'ai arrêté mon propre script de mutation avec `pkill -f` une fois (PID mien, mais la règle interdit pkill). Rien d'autre touché.
- Répertoires `/tmp/b90-*` (8) laissés par les recettes : à nettoyer. Aucun processus résiduel.

## Reste à faire (liste exacte)

1. UNE régression : `cargo test -p bridget-transport` puis `cargo test -p bridget-daemon` (source actuelle non mutée, --no-fail-fast).
2. Build debug + release privés, copies immuables, SHA256/codesign/version, empreintes.
3. Reçus `native149-debug-receipt-r9.json`, `native149-release-receipt-r9.json`, rapport `native-tests-sonnet-r9.md`.
4. Nettoyage `/tmp/b90-*` (mes répertoires), annonce Cargo libéré.
