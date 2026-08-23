# Implementation Plan: Visibilité du modèle et du niveau d'effort des agents

**Branch**: `session-04-runtime-modele-agents` | **Date**: 2026-08-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-runtime-modele-agents/spec.md`

## Summary

Ajouter deux attributs — modèle et niveau d'effort — à chaque agent de
l'annuaire Bridget, tenus à jour automatiquement quand l'humain change de modèle
en cours de session.

L'approche retenue repose sur un principe unique : **Bridget observe, il ne
demande pas**. Un seul message de protocole `Runtime { model, effort }` écrit
dans l'annuaire, alimenté par trois collecteurs qui n'ont en commun que ce point
de sortie :

1. **Codex** — sonde intégrée au wrapper : `lsof` sur le PID de l'agent pour
   trouver son rollout ouvert, puis lecture du dernier `turn_context`.
2. **Claude** — hook `Stop` déclenché par l'agent lui-même, qui appelle
   `bridget hook claude-runtime` ; le binaire lit le `transcript_path` fourni et
   en extrait la dernière ligne assistant hors sous-agent.
3. **Tout agent** — déclaration explicite `bridget runtime --model … --effort …`.

Le choix des sources est établi sur observation runtime, pas sur documentation :
voir [research.md](./research.md), qui rapporte notamment que le payload de hook
ne contient pas le modèle, que Claude ne garde pas son transcript ouvert alors
que Codex garde son rollout ouvert, et que le chemin des sessions Codex de cette
machine n'est pas le chemin conventionnel.

## Technical Context

**Language/Version**: Rust 2021 (workspace `bridget`, 3 crates)
**Primary Dependencies**: `serde` / `serde_json` (déjà présents), `libc` (déjà
présent, utilisé pour `FD_CLOEXEC`), `rusqlite` (non concerné ici). **Aucune
nouvelle dépendance.**
**Storage**: aucune. Le runtime vit dans `Presence`, en mémoire du daemon, comme
`host`, `os` et `transport` aujourd'hui — arbitrage utilisateur du 2026-08-17.
**Testing**: `cargo test` (36 tests existants), fixtures JSONL en `tests/fixtures/`
**Target Platform**: macOS (poste de travail) et Linux (hôtes fédérés SSH)
**Project Type**: CLI + daemon Unix, mono-workspace
**Performance Goals**: surcoût de sonde imperceptible ; `lsof` (134 ms mesurés)
amorti à une exécution par 5 minutes, `stat` toutes les 20 s
**Constraints**: lecture d'un rollout de 945 Mo interdite en parcours complet →
lecture par fenêtre depuis la fin ; aucune écriture dans les fichiers de session
de l'agent observé
**Scale/Scope**: ~10 agents simultanés, 2 types instrumentés, ~450 lignes Rust

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Article | Exigence | Statut | Preuve |
|---|---|---|---|
| I | Documentation en français | ✅ | tous les artefacts de `specs/004-*` |
| VII | ADR pour décision structurante | ✅ | `research.md` D-001 à D-006 tiennent lieu d'ADR ; aucune décision n'engage l'architecture au-delà de cette feature |
| VIII | Débogage forensic | ✅ | Phase 0 conduite exclusivement sur données runtime réelles, avant toute ligne de code |
| XV | 1 tâche = 1 commit, tests avant de cocher | ✅ | imposé dans `tasks.md` |
| XVIII | Complexité algorithmique | ✅ | lecture en fenêtre depuis la fin, O(taille de fenêtre) et non O(taille du fichier) ; `lsof` amorti ; annotation de complexité sur les deux parseurs |
| XIX | Minimalisme | ✅ | zéro nouvelle dépendance, zéro nouvelle abstraction, un seul message de protocole pour trois producteurs, réutilisation de `current_agent_name()` et de `Presence` |
| XX | Responsabilité du code généré | ✅ | chaque décision porte sa preuve empirique et son alternative écartée ; les risques acceptés sont écrits, pas masqués |

**Points de vigilance retenus, non bloquants** :

- Le hook modifie une configuration hors du dépôt (`~/.claude/settings.json`).
  Traité par FR-012 : action explicite, sauvegarde horodatée, retrait possible,
  insertion additive dans le tableau `Stop` existant.
- `lsof` est un appel à un binaire externe. Il est déjà l'unique méthode
  correcte (D-003) et son absence dégrade proprement : la sonde Codex se tait,
  l'annuaire affiche l'inconnu, rien ne casse (FR-011).

## Project Structure

### Documentation (this feature)

```text
specs/004-runtime-modele-agents/
├── plan.md              # Ce fichier
├── research.md          # Phase 0 — preuves empiriques et décisions
├── data-model.md        # Phase 1 — entités et transitions
├── quickstart.md        # Phase 1 — vérification manuelle bout en bout
├── contracts/
│   └── protocol.md      # Phase 1 — contrat du message Runtime et des sorties
├── checklists/
│   └── requirements.md  # Qualité de la spec
├── reuse-audit.md       # Phase 3 — gate anti-doublon
└── tasks.md             # Phase 3
```

### Source Code (repository root)

```text
crates/
├── bridget-core/src/          # inchangé — logique pure de routage
├── bridget-transport/src/
│   └── protocol.rs            # + WrapperToDaemon::Runtime, + AgentInfo.model/.effort
└── bridget-daemon/
    ├── src/
    │   ├── daemon.rs          # + Presence.model/.effort, traitement Runtime, agent_infos()
    │   ├── cli.rs             # + cmd_runtime, + cmd_hook, + cmd_install_hooks, colonnes who
    │   ├── runtime.rs         # NOUVEAU — parsing transcript Claude et rollout Codex
    │   └── wrapper.rs         # + sonde Codex greffée sur la boucle d'écoute
    └── tests/
        └── fixtures/          # NOUVEAU — extraits JSONL Claude et Codex anonymisés
```

**Structure Decision**: la feature s'insère dans les trois crates existants sans
en créer un nouveau. Un seul fichier neuf, `runtime.rs`, justifié par une
responsabilité propre et testable isolément — le parsing des deux formats de
session — qui n'a sa place ni dans `cli.rs` (déjà 769 lignes) ni dans
`wrapper.rs` (676 lignes) puisqu'il sert les deux. Toutes les autres
modifications étendent des structures existantes plutôt que d'en ajouter.

## Approche par exigence

| Exigence | Mise en œuvre | Fichier |
|---|---|---|
| FR-001, FR-010 | champs `model: Option<String>` et `effort: Option<String>` dans `Presence` | `daemon.rs:65` |
| FR-002, FR-004 | colonnes `MODÈLE` / `EFFORT` calculées comme les colonnes existantes, `—` si `None` | `cli.rs:659` |
| FR-003 | `AgentInfo.model` / `.effort` sérialisés par `serde` | `protocol.rs:83` |
| FR-005, FR-006 | hook `Stop` (Claude) et sonde 20 s (Codex) | `cli.rs`, `wrapper.rs` |
| FR-007 | comparaison à la dernière valeur émise, côté producteur | `wrapper.rs`, `runtime.rs` |
| FR-008, FR-009 | `cmd_runtime` réutilise `current_agent_name()`, refuse `human` | `cli.rs:216` |
| FR-011 | tout échec de sonde est journalisé en `debug`, jamais propagé | `runtime.rs` |
| FR-012 | `install-hooks` : sauvegarde horodatée, insertion additive, `--remove` | `cli.rs` |
| FR-013 | lecture seule, fenêtre bornée, cadence amortie | `runtime.rs`, `wrapper.rs` |

## Complexity Tracking

> Aucune violation de constitution à justifier.

| Point | Décision | Pourquoi ce n'est pas une violation |
|---|---|---|
| Nouveau fichier `runtime.rs` | Accepté | Responsabilité unique partagée par deux appelants, testable seule ; l'alternative est de dupliquer le parsing dans `cli.rs` et `wrapper.rs` |
| Appel à `lsof` | Accepté | Seule méthode correcte prouvée (D-003) ; dégradation propre en cas d'absence |
| `crates/bridget-daemon/src/managers.rs` non modifié | Assumé | Ce module (197 lignes, `ConnectionManager`, `PresenceManager`, `RequestManager`) est déclaré dans `lib.rs` mais **aucun de ses types n'est référencé** ailleurs dans le workspace : c'est un refactor jamais branché. Y répliquer les champs reviendrait à étendre du code mort. Sa suppression est un finding à remonter à l'audit, pas une tâche de cette feature — le scope reste celui demandé (Article V) |
