# Tasks: Visibilité du modèle et du niveau d'effort des agents

**Input**: Design documents from `/specs/004-runtime-modele-agents/`
**Prerequisites**: spec.md, plan.md, research.md, data-model.md, contracts/protocol.md, reuse-audit.md (statut `OK`)

**Tests**: inclus — SC-005 exige au moins un test par source d'observation.

**Format**: `[ID] [P?] [Story] Description`
**[P]** = parallélisable (fichiers disjoints, aucune dépendance).

**Règle de commit** (Article XV) : une tâche = un commit, `cargo test` vert
avant chaque commit, aucune case cochée sans vérification.

---

## Phase 1 : Socle partagé (bloquant pour toutes les stories)

- [x] **T001** Ajouter le message `WrapperToDaemon::Runtime { model: String, effort: Option<String>, source: RuntimeSource }` dans `crates/bridget-transport/src/protocol.rs`, avec `RuntimeSource` en énumération fermée (`codex-rollout`, `claude-hook`, `declared`), et les champs `#[serde(default)] model/effort: Option<String>` sur `AgentInfo`.
  **Observable** : `cargo test -p bridget-transport` vert, dont un nouveau test d'aller-retour `encode`/`decode` du message `Runtime` et un test prouvant qu'un `AgentInfo` sérialisé sans ces champs reste décodable (compatibilité ascendante).

- [x] **T002** Porter le runtime dans la présence du daemon : champs `model`/`effort` sur `Presence` (`crates/bridget-daemon/src/daemon.rs:65`), initialisés à `None` dans `Register`, propagés dans `agent_infos()` (`daemon.rs:285`) pour les deux branches — agents connectés et présences `unreachable`.
  **Observable** : `cargo test -p bridget-daemon` vert, dont un test qui vérifie qu'une présence passée `unreachable` conserve son modèle (FR-010).

- [x] **T003** Traiter `Runtime` côté daemon (`daemon.rs`, table de dispatch `WrapperToDaemon`) : refus si la connexion n'est pas enregistrée, validation longueur ≤ 100 et absence de caractères de contrôle, **remplacement atomique** du couple `(model, effort)`, `Ack { id: "runtime" }` en réponse. Ne pas faire transiter ce message par le routeur ni par le disjoncteur.
  **Observable** : test unitaire prouvant qu'un passage d'un modèle avec effort à un modèle sans effort **efface** l'effort précédent (défaut soulevé par la contre-revue `agent-1`) ; `RUST_LOG=debug` journalise la `source` et le nom de l'agent.

---

## Phase 2 : User Story 1 — Voir le modèle et l'effort (P1)

- [x] **T004** [US1] Ajouter les colonnes `MODÈLE` et `EFFORT` avant `ÉTAT` dans `cmd_who` (`crates/bridget-daemon/src/cli.rs:659`), largeurs calculées comme les colonnes existantes, `—` pour une valeur absente. Étendre la ligne de `cmd_agents` en mode texte (`cli.rs:620`) de la même façon.
  **Observable** : `bridget who` avec un agent sans modèle et un agent au modèle long reste aligné (SC-004) ; `bridget agents --json` expose `model` et `effort` (FR-003).

---

## Phase 3 : User Story 3 — Déclaration explicite (P2, socle des sondes)

> Placée avant US2 volontairement : c'est le chemin d'écriture que les deux
> sondes réutilisent. L'implémenter d'abord rend US2 vérifiable sans agent réel.

- [x] **T005** [US3] Implémenter `bridget runtime --model <m> [--effort <e>]` dans `cli.rs` : réutiliser `current_agent_name()` (`cli.rs:216`), refuser `human` avec « runtime indisponible hors d'un agent Bridget », valider via les fonctions existantes de `cli.rs:25-45`, envoyer `Runtime { source: "declared" }` sur le motif de `send_rename_to_daemon` (`cli.rs:495`). Déclarer la commande dans le dispatch (`cli.rs:77`) et dans `print_usage()` (`cli.rs:140`).
  **Observable** : depuis un agent, `bridget runtime --model test-model --effort low` puis `bridget who` affiche la valeur ; hors agent, code de sortie 1 et message explicite.

---

## Phase 4 : User Story 2 — Mise à jour automatique (P1)

- [x] **T006** [US2] Créer `crates/bridget-daemon/src/runtime.rs` avec les deux parseurs purs, plus la lecture par fenêtre depuis la fin de fichier partagée par les deux (256 Kio, extensible à 4 Mio avec recouvrement ; première ligne jetée sauf si la fenêtre atteint le début du fichier ; scan à rebours des seules lignes JSON complètes) : `parse_claude_transcript` (dernière ligne `type=assistant`, `isSidechain != true`, **portant réellement `message.model`** → modèle + `effort`) et `parse_codex_rollout` (dernier `type=turn_context` portant un modèle → `payload.model` + `payload.effort`). Aucune émission si le modèle est absent. Annoter la complexité de chaque fonction. Déclarer le module dans `lib.rs`.
  **Observable** : `cargo test` vert avec des fixtures générées dans le test (plus lisibles qu'un fichier séparé, et aucune ne dépasse quelques lignes hors remplissage volumétrique) couvrant : transcript nominal ; transcript dont la dernière ligne est un sous-agent ; transcript sans champ `effort` (cas Haiku de research.md D-001) ; transcript dont la dernière ligne assistant ne porte pas de modèle ; rollout nominal ; rollout dont la fenêtre coupe une ligne aux deux bornes ; dernier `turn_context` sans modèle ; fichier vide ; fichier illisible. Les quatre dernières fixtures viennent de la contre-revue `agent-1`.

- [x] **T007** [US2] Implémenter `bridget hook claude-runtime` dans `cli.rs` : lire le payload JSON sur l'entrée standard, sortir silencieusement en code 0 si `BRIDGET_AGENT_NAME` est absent ou si le transcript est inexploitable, sinon émettre `Runtime { source: "claude-hook" }`. Sortie standard toujours vide, code de retour toujours 0.
  **Observable** : la commande de diagnostic de `quickstart.md` met à jour `bridget who` ; la même commande sans `BRIDGET_AGENT_NAME` ne produit rien et rend 0.

- [x] **T008** [US2] Implémenter `bridget install-hooks [--remove]` dans `cli.rs` : sauvegarde `~/.claude/settings.json.bak-<AAAAMMJJ-HHMMSS>` écrite avant modification et son chemin affiché, insertion **additive** dans le tableau `hooks.Stop` existant, idempotence, retrait ciblé de la seule entrée Bridget, indentation à 2 espaces.
  **Observable** : sur une copie de `~/.claude/settings.json` contenant déjà quatre hooks utilisateur, l'installation puis le retrait rendent un fichier équivalent à l'original ; une seconde installation ne duplique rien.

- [x] **T009** [US2] Greffer la sonde Codex sur la boucle d'écoute du wrapper (`crates/bridget-daemon/src/wrapper.rs:396`) : résolution du rollout par `lsof -p <pid>` (le `.jsonl` de `mtime` le plus récent parmi les descripteurs ouverts, tri couvert par une fixture à `mtime` inversés), re-résolution au plus toutes les 60 s ou si le chemin disparaît, avec remise à zéro de la date mémorisée quand le chemin change, vérification du `mtime` toutes les 20 s, lecture et émission uniquement sur changement de `(model, effort)`. Sonde active pour `agent_type == "codex"` uniquement. Tout échec en `log::debug`, jamais fatal.
  **Observable** : sur un agent `bridget codex` réel, `bridget who` affiche le modèle dans les 20 s suivant un tour ; un agent inactif n'émet rien (SC-003, vérifié par les journaux `debug` du daemon sur 5 minutes).

---

## Phase 5 : Finition

- [x] **T010** Documenter la fonctionnalité dans `README.md` : nouvelles commandes dans la table des commandes, section courte sur la détection du modèle et sur `install-hooks`, mention explicite que le hook modifie `~/.claude/settings.json` et comment revenir en arrière.
  **Observable** : un lecteur du README sait installer, vérifier et désinstaller sans lire les specs.

- [x] **T011** Vérification bout en bout selon `quickstart.md` sur les agents réellement connectés (`agent-2`, `agent-1`), puis consigner les résultats mesurés de SC-001 à SC-004 dans `implementation.md`.
  **Observable** : `implementation.md` contient les trois mesures de délai de SC-002 et la sortie réelle de `bridget who`.

---

## Dépendances

```text
T001 ──► T002 ──► T003 ──┬──► T004        (US1, livrable seul)
                         ├──► T005        (US3, livrable seul)
                         └──► T006 ──┬──► T007 ──► T008   (US2 Claude)
                                     └──► T009            (US2 Codex)
T004 ─────────────────────────────────────► T010 ──► T011
```

- **T004** [P] avec **T005** : fichiers différents dans `cli.rs`, mais mêmes
  zones — à traiter en séquence pour éviter un conflit d'édition.
- **T007** et **T009** [P] : `cli.rs` et `wrapper.rs` sont disjoints, une fois
  T006 livré.

## Points d'arrêt de livraison

| Après | Ce qui est utilisable |
|---|---|
| T004 | L'annuaire affiche les colonnes ; toutes les valeurs sont `—`. Aucune valeur fausse. |
| T005 | Tout agent peut se déclarer à la main. US3 complète. |
| T008 | Les agents Claude se mettent à jour seuls. US2 à moitié. |
| T009 | Les agents Codex aussi. US2 complète. |
| T011 | Feature vérifiée sur données réelles. |
