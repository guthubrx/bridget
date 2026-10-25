# Revue code natif r1 — session149 (GLM 5.3 Flash, indépendante)

Date : 2026-10-10. Base : `6807c22b7ada683f757486a6382aeda170ec68ab`, worktree `149-sous-agents-lineage`, tout le diff production est non commité.
Revueuse : GLM 5.3 Flash. Aucune exécution, aucun build, aucun test lancé, aucun modèle réel. Lecture seule du diff natif et des fichiers nouveaux.

## Verdict : REQUEST_CHANGES

Un finding Medium bloque (F1, élargissement possible dans le mappage Claude→Codex). Quatre findings Low ne bloquent pas seuls. Le reste de l'implémentation examinée respecte les contrats149 sur les axes demandés.

## Findings

### F1 — Medium — mappage Claude→Codex : `allowed_tools` du parent ignoré

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-code-review-r1.md#extrait-1).

### F2 — Low — révision du fait Claude insensible au changement de `cwd`

- Chemin : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/claude_stream_json.rs` ligne 999.
- Attendu (contrat) : « Revision augmente lorsque le fait courant change. » Le `cwd` fait partie du fait.
- Réel : l'incrément ne porte que sur `session` et `policy` (`o.0!=session||o.1!=policy`). Le pilote Codex inclut `cwd` (`codex_app_server.rs`, `observe_native_permissions`). Un `cwd` changé à session et politique identiques garde la révision stable.
- Impact borné : le fait stocké reste à jour (la valeur `cwd` est remplacée) ; seule la sémantique de révision faiblit. Aucun élargissement démontré.
- Fix minimal : ajouter `||o.3!=cwd` à l'incrément, sur le modèle Codex.

### F3 — Low — `tools: []` vide accepté puis lu comme « lecture seule »

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-code-review-r1.md#extrait-2).

### F4 — Low — `initialize_lineage` fait échouer l'ouverture du magasin sur donnée héritée pathologique

- Chemin : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/delegation_lineage.rs` lignes 79-92.
- Attendu : la borne 8 missions est une règle d'admission et de migration ; rien n'impose d'arrêter le daemon pour une chaîne héritée trop profonde.
- Réel : `visited.len()>8` et `records.len()>4096` retournent `Err` depuis `DelegationStore::new` → le daemon ne démarre plus. Les liens rompus (`child_instance` différent) suivent déjà un `break` doux qui racine la sous-chaîne ; une chaîne de 9 ou un cycle, non.
- Fix minimal proposé (décision Sol) : traiter le dépassement de profondeur comme le `break` existant (raciner la sous-chaîne de 8), garder `Err` seulement pour un cycle vrai ou un magasin corrompu. La direction fail-closed reste défendable si Sol la assume.

### F5 — Low — lanceur binaire non prouvé transparent accepté

- Chemin : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions.rs` lignes 78-85.
- Attendu (contrat) : « Une chaîne non observable … rend permission_source_unavailable pour cette voie seulement. » « Un nom de wrapper ne constitue jamais cette preuve. »
- Réel : un script (`#!`) est refusé sauf empreinte `gclaude` épinglée ; tout binaire non script passe avec `resolved_cli_path == cli_path`, sans preuve qu'il transmet les drapeaux à `claude`. Le registre est une entrée opérateur de confiance, donc l'escalade exige une définition hostile ou maladroite.
- Fix minimal : réserver l'indirection à l'empreinte épinglée — si `launcher != resolved("claude")` et `launcher_revision != TRANSPARENT_GCLAUDE`, retourner `permission_source_unavailable`.

## Axes permissions (T009–T014) — vérifié par lecture

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-code-review-r1.md#extrait-3).

## Axes lineage (T016–T022) — vérifié par lecture

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-code-review-r1.md#extrait-4).

## Limites de cette revue

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-code-review-r1.md#extrait-5).

## Synthèse

F1 est le seul refus : une ligne de garde manque dans le mappage Claude→Codex. F2–F5 sont des corrections locales que Sol peut prendre avec F1. Le fix Sol de `initialize_lineage` est correct et vérifié indépendamment de l'admission. Aucune recette n'est déclarée validée par cette lecture.
