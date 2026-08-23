# Implementation Plan: Domaines d'agents et statut « ne pas déranger »

**Branch**: `session-05-domaines-dnd` | **Date**: 2026-08-17 | **Spec**: [spec.md](./spec.md)

## Summary

Deux attributs de plus sur la présence d'un agent, sur le modèle exact de ce
qu'a livré la session 004 : un **domaine** qui range l'annuaire, et un **statut
de disponibilité** qui protège le temps de travail.

La conception a été arbitrée avec l'utilisateur avant rédaction :

1. `bridget who` affiche **tout** avec une colonne `DOMAINE` ; `--domain <nom>`
   filtre à la demande. Aucune commande existante ne change de comportement.
2. Un envoi vers un agent en « ne pas déranger » est **refusé explicitement**,
   avec le temps restant. Pas de file d'attente, pas de livraison silencieuse.
3. Le statut se lève à la main **ou** à l'expiration d'une durée de sécurité de
   60 minutes ; `--duration` la remplace.

## Technical Context

**Language/Version**: Rust 2021, workspace `bridget`
**Primary Dependencies**: aucune nouvelle. `git` est appelé comme `lsof`, `tmux`
et `hostname` le sont déjà.
**Storage**: présence en mémoire du daemon, plus un fichier pour le domaine
surchargé — même motif que le nom persistant (`agent-names/`)
**Testing**: `cargo test`, 81 tests existants
**Target Platform**: macOS et Linux
**Constraints**: aucune régression sur les colonnes et champs existants ;
dérivation du domaine sans configuration ; refus DND en moins d'une seconde
**Scale/Scope**: ~390 lignes, 6 tâches

## Constitution Check

| Article | Statut | Preuve |
|---|---|---|
| I — français | ✅ | artefacts et messages utilisateur |
| VII — ADR | ✅ | les trois arbitrages sont consignés dans ce plan et dans la spec |
| XVIII — complexité | ✅ | dérivation git une fois au démarrage, jamais en boucle ; le filtre `--domain` est un `filter` linéaire sur une liste de moins de dix éléments ; l'expiration du statut est une comparaison d'instants |
| XIX — minimalisme | ✅ | zéro dépendance, zéro fichier source nouveau, deux champs sur une structure existante, réutilisation du motif de persistance du nom |
| XX — responsabilité | ✅ | la limite est écrite dans la spec : le domaine n'est pas un mécanisme de sécurité et ne le deviendra pas sans authentification de l'émetteur |

**Point de vigilance** : le statut « ne pas déranger » introduit un refus de
livraison. Un refus mal signalé transformerait Bridget en trou noir. D'où
l'exigence que la raison ET le temps restant remontent à l'émetteur, et que
l'annuaire rende le statut visible (FR-011, FR-013).

## Approche par exigence

| Exigence | Mise en œuvre | Fichier |
|---|---|---|
| FR-001, FR-002 | `derive_domain()` : `git rev-parse --show-toplevel` sur le cwd, repli sur le `cwd` ; appelé une fois au lancement | `wrapper.rs` |
| FR-003, FR-005 | champ `domain` sur `AgentInfo`, colonne dans `cmd_who`, sortie `serde` | `protocol.rs`, `cli.rs` |
| FR-004 | option `--domain <nom>` sur `who` et `agents` | `cli.rs` |
| FR-006, FR-008 | `bridget domain <nom> \| --reset`, refus si `human` | `cli.rs` |
| FR-007 | fichier `agent-domains/<nom>`, relu au `Register` et à la reconnexion | `wrapper.rs`, `cli.rs` |
| FR-009, FR-010, FR-014 | `dnd_until: Option<Instant>` sur `Presence`, expiration par comparaison | `daemon.rs` |
| FR-011 | contrôle dans le routage, `Nack` avec minutes restantes | `daemon.rs` |
| FR-012 | contrôle dans la boucle d'escalade | `daemon.rs` |
| FR-013 | `state` vaut `dnd` quand le statut est actif | `daemon.rs` |

## Décisions de conception

**D-101 — Le domaine vit dans la présence, comme le modèle.** Pas de table, pas
de registre séparé. Un agent sans présence n'a pas de domaine à afficher.

**D-102 — Le statut est un instant d'expiration, pas un booléen.** `Option<Instant>`
rend l'expiration automatique gratuite : il n'y a rien à balayer, la valeur est
comparée à l'instant de lecture. Un booléen aurait exigé une tâche de fond.

**D-103 — `state` porte le statut plutôt qu'une colonne dédiée.** `connected`,
`dnd`, `unreachable` sont mutuellement exclusifs du point de vue de l'appelant :
dans les trois cas la question est « puis-je lui écrire maintenant ». Une colonne
de plus aurait allongé une sortie déjà large de huit colonnes.

**D-104 — Le refus DND est vérifié dans le daemon, pas dans le client.** Un
contrôle côté client serait contournable par un client d'une version antérieure,
et surtout il devrait interroger l'annuaire avant chaque envoi.

**D-105 — La persistance du domaine réutilise le répertoire d'état existant.**
`~/.cache/bridget/agent-domains/<nom>`, en miroir de `agent-names/`. Le motif est
connu, testé, et déjà nettoyé par les mêmes mécanismes.

## Structure

```text
crates/
├── bridget-transport/src/protocol.rs   # + Register.domain, + AgentInfo.domain,
│                                       #   + Domain{..}, + Availability{..}
└── bridget-daemon/src/
    ├── daemon.rs                       # + Presence.domain/.dnd_until,
    │                                   #   contrôle de routage, escalades
    ├── cli.rs                          # + cmd_domain, cmd_dnd, colonnes, filtre
    └── wrapper.rs                      # + derive_domain, domaine persistant
```

Aucun fichier source créé : la feature étend des structures et des commandes
existantes.
