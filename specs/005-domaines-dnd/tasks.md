# Tasks: Domaines d'agents et statut « ne pas déranger »

**Prerequisites**: spec.md, plan.md

**Règle** : `cargo test` vert et zéro warning de compilation avant de cocher.

---

## Phase 1 : Domaine

- [x] **T101** [US1] Ajouter `domain: Option<String>` à `Register` et `AgentInfo`
  (`crates/bridget-transport/src/protocol.rs`) et à `Presence`
  (`crates/bridget-daemon/src/daemon.rs`), propagé dans les deux branches de
  `agent_infos()`. Écrire `derive_domain()` dans `wrapper.rs` :
  `git rev-parse --show-toplevel` sur le répertoire courant, repli sur le nom du
  répertoire de travail, aucun embellissement du nom.
  **Observable** : tests d'aller-retour du protocole et de compatibilité
  ascendante ; test de dérivation avec repli hors dépôt git.

- [x] **T102** [US1] Colonne `DOMAINE` dans `cmd_who`, ligne de `cmd_agents`,
  et option `--domain <nom>` sur les deux commandes (`cli.rs`).
  **Observable** : `bridget who` montre tous les agents avec leur domaine ;
  `bridget who --domain bridget` ne montre que ceux-là ; `bridget agents --json`
  expose le champ.

- [x] **T103** [US2] `bridget domain <nom> | --reset` (`cli.rs`) : refus hors
  agent Bridget, validation du nom, persistance dans
  `~/.cache/bridget/agent-domains/<nom-agent>`, nouveau message de protocole
  `Domain`. Le wrapper lit ce fichier au premier enregistrement **et** à chaque
  reconnexion, comme il le fait désormais pour le nom.
  **Observable** : surcharge visible dans `who`, conservée après redémarrage du
  daemon, `--reset` rend le domaine dérivé ; hors agent, code 1 et message clair.

---

## Phase 2 : Ne pas déranger

- [x] **T104** [US3] Champ `dnd_until: Option<Instant>` sur `Presence`, message
  de protocole `Availability { until_secs: Option<u64> }`, commande
  `bridget dnd [off] [--duration <durée>]` (`cli.rs`) acceptant `30m`, `2h`,
  `90s`. Durée de sécurité de 60 minutes à défaut de précision. `state` vaut
  `dnd` tant que l'instant n'est pas dépassé.
  **Observable** : `bridget dnd` puis `bridget who` affiche `dnd` ; `bridget dnd off`
  rend `connected` ; un test prouve qu'une échéance dépassée rend l'agent
  joignable sans intervention.

- [x] **T105** [US3] Refuser le routage vers un agent en « ne pas déranger »
  (`daemon.rs`) : `Nack` portant le nom et les minutes restantes. Suspendre les
  rappels d'escalade vers cet agent dans la boucle de surveillance des demandes.
  **Observable** : test unitaire du refus avec temps restant ; test prouvant
  qu'un rappel dû n'est pas délivré à un agent en statut, et que la demande n'est
  pas perdue pour autant.

---

## Phase 3 : Finition

- [x] **T106** Documenter dans `README.md` (commandes, section courte sur les
  domaines et la disponibilité), tenir `specs/005-domaines-dnd/implementation.md`,
  et vérifier sur les agents réellement connectés.
  **Observable** : le journal contient la sortie réelle de `bridget who` avec un
  domaine surchargé, le filtre `--domain`, un agent en statut et le refus motivé
  reçu par l'émetteur. La confrontation de deux domaines *dérivés* distincts
  attend le prochain lancement d'agents, les wrappers connectés datant du binaire
  précédent — écart consigné dans `implementation.md`.

---

## Dépendances

```text
T101 ──┬──► T102
       └──► T103
T104 ──► T105
T102, T103, T105 ──► T106
```

T101 et T104 touchent les mêmes fichiers : à traiter en séquence.
