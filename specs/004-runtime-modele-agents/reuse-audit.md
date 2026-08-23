# Audit de l'existant — Gate avant `tasks.md`

**Feature** : 004-runtime-modele-agents
**Date** : 2026-08-17
**Méthode** : fallback manuel — la primitive `speckit-audit-existing` n'est pas
installée dans `.agents/skills/`. Recherches `rg` par nom et par responsabilité,
plus lecture de `AGENTS.md`, `.specify/memory/constitution.md` et
`.specify/memory/standards.md`.
**Statut** : `OK` — aucune duplication évidente, aucun arbitrage utilisateur requis.

## Items proposés par le plan

| Item proposé | Recherche menée | Équivalent trouvé | Issue |
|---|---|---|---|
| Message `WrapperToDaemon::Runtime` | lecture de l'enum `protocol.rs:13` | `Register` porte des attributs statiques d'identité, `Heartbeat` ne porte aucune charge utile | **CRÉER** — aucun message existant ne transporte un état mutable d'agent |
| Champs `model` / `effort` sur `Presence` | `rg 'struct Presence'` → `daemon.rs:65`, `managers.rs:83` | `Presence` (daemon) porte déjà `host`, `os`, `transport` : même nature d'attribut | **RÉUTILISER** — extension de la structure existante, pas de nouvelle entité |
| Champs `model` / `effort` sur `AgentInfo` | `protocol.rs:83` | même structure, même sérialisation `serde` | **RÉUTILISER** |
| Module `crates/bridget-daemon/src/runtime.rs` | `rg 'serde_json::'` → 8 occurrences, toutes du protocole ligne à ligne ; aucun parseur de fichier de session | aucun | **CRÉER** — voir justification ci-dessous |
| Commande `bridget runtime` | `rg 'fn cmd_runtime'` → néant ; table des sous-commandes `cli.rs:77` | aucune | **CRÉER** |
| Commande `bridget hook claude-runtime` | `rg -i 'claude-runtime\|hook'` dans `crates/` et `scripts/` → néant | aucune | **CRÉER** |
| Commande `bridget install-hooks` | `rg -i 'install.hooks\|settings\.json'` → néant | aucune | **CRÉER** |
| Identification de l'agent courant | `rg 'BRIDGET_AGENT_NAME_FILE'` | `current_agent_name()` `cli.rs:216`, déjà utilisé par `cmd_rename` | **RÉUTILISER** — aucun nouveau mécanisme |
| Envoi d'un message au daemon depuis le CLI | lecture de `send_rename_to_daemon` `cli.rs:495` | fonction existante spécialisée `Rename` | **RÉUTILISER le motif**, une fonction sœur pour `Runtime` ; pas de généralisation prématurée (< 3 usages) |
| Colonnes de `bridget who` | `cmd_who` `cli.rs:659` | calcul de largeur déjà présent pour 5 colonnes | **RÉUTILISER** — deux colonnes de plus dans la même boucle |
| Appel à un binaire externe (`lsof`) | `rg 'Command::new'` → `hostname`, `tmux` | motif déjà employé (`host_name()` `wrapper.rs:45`, `get_current_pane_id()` `wrapper.rs:84`) | **RÉUTILISER le motif** existant |
| Dépendance externe nouvelle | lecture de `Cargo.toml` racine et daemon | `serde`, `serde_json`, `libc`, `log`, `uuid` déjà déclarés | **AUCUNE** dépendance ajoutée |
| Fixtures de test | `ls crates/bridget-daemon/tests/` → `integration_test.rs` seul | aucun dossier de fixtures | **CRÉER** `tests/fixtures/` |

## Justification des créations

**`runtime.rs`** — trois appelants distincts en auront besoin (`cmd_hook`,
`cmd_runtime`, sonde du wrapper) et sa responsabilité est unique : transformer
un fichier de session d'agent en couple `(model, effort)`. Le seuil de
l'Article XIX (3 usages réels) est atteint. L'alternative — dupliquer le parsing
dans `cli.rs` (769 lignes) et `wrapper.rs` (676 lignes) — dégraderait deux
fichiers déjà longs et rendrait le parsing non testable isolément.

**Les trois sous-commandes** — chacune porte une responsabilité utilisateur
distincte, exposée dans le contrat : déclarer, être appelé par un hook,
installer le hook. Aucune n'est un wrapper passthrough : `cmd_runtime` valide et
émet, `cmd_hook` parse un payload et dégrade en silence, `cmd_install_hooks`
sauvegarde et modifie une configuration hors dépôt.

## Findings hors périmètre (à remonter à l'audit, pas à traiter ici)

1. **`crates/bridget-daemon/src/managers.rs`** — 197 lignes déclarées dans
   `lib.rs` mais dont aucun type n'est référencé dans le workspace :
   `ConnectionManager`, `PresenceManager` et `RequestManager` dupliquent en code
   mort des structures vivantes de `daemon.rs`. Y ajouter les nouveaux champs
   serait étendre du code mort ; les supprimer serait du nettoyage opportuniste
   hors scope (Article V). **Signalé, non traité.**
2. **`Store::purge_if_too_large`** (`store.rs:255`) — corps vide qui ignore son
   paramètre. Sans rapport avec cette feature. **Signalé, non traité.**

## Gate avant `tasks`

- [x] Chaque item proposé a fait l'objet d'une recherche par nom et par responsabilité
- [x] Chaque création est justifiée par écrit
- [x] Aucune duplication évidente d'un service, composant, table, endpoint ou commande existant
- [x] Aucune nouvelle dépendance externe
- [x] Aucune abstraction créée pour moins de 3 usages réels
- [x] Les findings hors périmètre sont consignés sans élargir le scope
- [x] Aucun arbitrage utilisateur en attente

## Arbitrages

Aucun à ce stade. Toute création décidée pendant l'implémentation alors qu'un
équivalent proche existe devra être ajoutée ici avec sa preuve `fichier:ligne`.
