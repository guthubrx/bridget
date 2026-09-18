# Dossier de livraison et de reprise — 104-recherche-echanges

## État (2026-09-18)

Implémentation terminée dans le worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges` (branche
`session-104-recherche-echanges`, base `2720a0c1` sur main `218c5cc1` qui contient 102 et 103).
Statut détaillé, commandes et mesures : `implementation.md` ; correspondance exigences → code → tests :
`analysis.md` ; décision : `docs/decisions/040-recherche-bornee-ledger.md` (Accepté).

Aucune installation du daemon n'a été faite : le binaire installé (`~/.cache/bridget-core`) reste
l'ancien ; `bridget ledger search` sur le poste répondra `daemon_protocol` tant que le daemon n'est pas
reconstruit et relancé (autorisation distincte : `launchctl kickstart -k gui/$UID/com.bridget.daemon`).

## Ce qui a été livré

- Protocole : `WrapperToDaemon::LedgerSearch/LedgerRead`, `DaemonToWrapper::LedgerSearchResult/LedgerReadResult`
  (`crates/bridget-transport/src/protocol.rs`, types `LedgerSearch*`, `LedgerRead*`).
- Store : index `idx_ledger_sender_page`, `idx_ledger_target_page` ; `Store::open_read_only` ; module
  `store::ledger_requests::search` (plages, chargement par clé primaire, repli unique, offset original) ; ancien
  moteur LIKE retiré.
- Daemon : `ledger::search` (validation, curseur, page, relecture, permis) ; bras hors verrou avec identité
  revérifiée (`daemon.rs`).
- Surfaces : `bridget_ledger` `action=recent|search|read` (MCP, schéma fermé, aucun nouvel outil) ;
  `bridget ledger search|read [--json]` (CLI, codes 0/2/1) ; `ledger --limit` inchangé.
- Documentation : `skills/bridget/references/commandes.md` § « Recherche dans les échanges (104) »,
  `skills/bridget/SKILL.md` § « Chercher, continuer, relire, citer », `README.md`.
- Tests : `crates/bridget-daemon/tests/search_104_test.rs` (24), unitaires `spec104_*` (transport 3, store 3,
  ledger 1, client 1).

## Recette humaine (après reconstruction et relance du daemon)

```sh
bridget ledger search --query "pagination erreur" --limit 20
bridget ledger search --query "pagination erreur" --limit 20 --cursor <next_cursor>
bridget ledger search --source thread --thread-id <uuid> --query "décision" --json
bridget ledger read --id <id> --target <uuid> --offset <match_offset> --digest <body_digest>
bridget ledger --limit 20   # inchangé
```

Attendus : zéro résultat = code 0 avec « fin de la partie conservée » ; `hits=[]` avec « suite disponible »
est normal ; `--cursor` avec une requête textuellement différente → code 2 `invalid_cursor` ; un message d'un
tiers → `not_found_or_forbidden` ; un extrait contenant des séquences ANSI s'affiche neutralisé mais reste brut
en `--json`.

## Commandes de vérification

Depuis le worktree, `umask 077`, `BRIDGET_HOME` pointant sur un répertoire vide (sinon l'identité T3 du poste
est héritée par les tests CLI) :

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p bridget-transport --lib spec104
cargo test -p bridget-daemon --lib -- spec104
cargo test -p bridget-daemon --test search_104_test
cargo test -p bridget-daemon --release --test search_104_test spec104_s2 -- --nocapture --test-threads=1   # mesures
```

Régressions voisines exécutées : `spec102_threads_test`, `handoff_103_test`, `core_089_ledger_test`,
`core_089_skill_test`, `claude_native_permissions_test`, puis recette release complète (`--workspace`).

## Limites héritées et pièges

- Rétention du ledger (sept jours par défaut) : les fixtures doivent être datées récentes.
- Casse/accents précomposés repliés seulement ; formes décomposées non assimilées.
- Corpus des messages vivant entre deux pages ; fils : instantané de borne haute.
- Identifiant hérité > 256 octets dans la fenêtre : `source_metadata_too_large`, restreindre les dates.
- Relecture intégrale d'un corps de 16 Mio : coût `B × ceil(B/16 Kio)` (compromis ADR 040).
- Ne jamais lancer le daemon de test sur le namespace de production ; jamais de `kill -9`.

## État Git et prochaines actions

Commits et fusion : selon l'autorisation en vigueur pour cette session (voir `implementation.md`).
Prochaine action technique après fusion : reconstruire le daemon installé (`t3-local-build` ou script 106) et
relancer `com.bridget.daemon` sur autorisation, puis exécuter la recette humaine ci-dessus.
