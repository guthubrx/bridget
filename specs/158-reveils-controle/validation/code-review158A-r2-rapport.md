# Revue adverse READONLY - SPEC158A (r2)

Verdict : **APPROVE_WITH_CHANGES**

Les correctifs répondent aux quatre objections de la ronde 1 et à l'objection TEST001.
Aucun défaut bloquant n'est resté. Il reste trois corrections mineures et une limite de preuve.
Le gate de sortie reste ouvert, car aucun FULL n'a été relancé après les correctifs.

Aucun Cargo, test, build, scanner, fournisseur ou service n'a été lancé.
Aucune source n'a été modifiée. Ce rapport et le manifeste sont les seuls fichiers écrits.

## Provenance et intégrité

- Capture finale vérifiée : `scope-files.json` SHA256 `79f34885...a31a541197`, diff `67001c94...fe76`.
  Le diff contient 152 hunks et 5093 lignes.
- Les 27 fichiers du scope ont un SHA identique à la capture (13 Rust, 14 docs).
- Les SHA des patches DELTA correspondent aux valeurs annoncées.
- Le worktree contient déjà les correctifs : `mcp.rs` = `0c525f7c...`, `cli.rs` = `3c428621...`.
  Les trois fichiers de test ont le SHA « après » du manifeste de fixtures.
- Toutes les lignes ajoutées des patches combiné, CLI/docs et fixtures sont présentes dans le worktree.
- La ronde 1 a été faite sur une capture antérieure : SHA `5081979d...` et diff `590c8bbb...`, 137 hunks.
  Ses coordonnées de ligne ne valent donc plus pour l'état final. Je les ai revérifiées ici.
- Le FULL `T021-FULL158-final` (rc 101, démarré 09:32:35 UTC, 804 s) est antérieur aux correctifs.
  Les fichiers `mcp.rs` et `cli.rs` ont été écrits à 09:49:16 UTC.

## Réponse aux objections

| Objection | Statut | Preuve |
|---|---|---|
| F1 MEDIUM : arrêt committé, réponse « refusé » | Résolu en CLI et docs. Résidu : message trop large (C1) | `cli.rs:6310-6315`, `contracts/control158.md`, `FR-009`, ADR R3 |
| F2 LOW : ancêtre absent bloque tout `--cancel-active` | Documenté. Chemin normal non atteint, à confirmer partiellement | `daemon.rs:10849` (snapshot seulement si `cancel_active`), `delegation_lineage.rs:186-187, 209` |
| F3 LOW : `control status` sort en code 1 | Documenté (FR-011, contrat, ADR). Code inchangé | docs finales |
| F4 LOW : oracle `suspended` ajusté | Résolu : contrat « vue courante », oracle précis | patch final ligne 1342 |
| TEST001 MEDIUM : oracle ChildProof affaibli | Résolu : garde avant connexion + oracle de base | `mcp.rs:631-636`, `mcp.rs:2713-2728` |

### F1 (CLI)

Le correctif ne crée aucun nouveau DTO ni succès inventé. Il réutilise `StoreUnavailable`.
Le message « arrêt ou annulations ont pu être appliqués » est une formulation prudente. Elle reste exacte pour le cas après commit.

Le message s'affiche aussi pour `resume-delegations` (`cli.rs:6169-6171`, `cancel_active: false`).
Pour ce cas, il n'existe aucune annulation. Le message est donc trop large (C1).

L'oracle demandé en ronde 1 (test qui force une erreur sur `human_cancel`) n'a pas été ajouté. Le correctif est uniquement textuel (C3).

### F2 (ancêtre absent)

La doc ADR est exacte. Dans le gestionnaire (`daemon.rs:10849-10877`), le snapshot n'est lu que si `cancel_active` est vrai.
Un ancêtre absent, une profondeur de 8 ou un dépassement de `MAX_RETAINED_TASKS` renvoient `Err` depuis la fermeture.
Je déduis que la transaction est annulée (rollback). Je n'ai pas lu `set_delegation_halt`.

Pour la purge, `purge_trees` ne supprime qu'un arbre dont `tree_settled` est vrai (`delegation_lineage.rs:186-187`).
L'appel à `delete_tree` est en `delegation_lineage.rs:209`. Je n'ai pas lu le corps de `tree_settled`.
Aucun chemin normal ne retire donc un parent avec un enfant actif. Le cas corrompu reste documenté.

### TEST001 (ChildProof)

La garde `bridget_queue` avec `delegated_origin` est placée avant `registered_connection` (`mcp.rs:631-636`).
Le test `guidance156_mcp_queue_schema` garde le listener non bloquant. Il vérifie `WouldBlock` après l'appel (`mcp.rs:2713`).
Si une connexion avait eu lieu, `accept` la verrait dans le backlog. L'oracle est donc aussi fort que la version de base.

Le worktree a retiré le faux RPC et le serveur de la section ChildProof. Le reste du test diffère de la racine par une seule ligne, `reply_caller_scope` (`mcp.rs:2763`, positive section). Cette ligne est nécessaire, car chaque appel fait désormais un `CallerScope`.

### Aperçu handoff (preview103)

- `scope_of` ne renvoie `Reads` que si `action == "preview"` (`mcp_scopes.rs:57-70`).
- `parse_request` refuse tout champ de transport présent, même `null` (`handoff.rs:548`, `contains_key`).
- Le chemin pur (`mcp.rs:639-648`) est exécuté avant le RPC. Il ne lit aucune base et ne crée aucun identifiant.
- Un envoi ou une autre lecture passent par le RPC. Une panne ferme l'appel sans repli.

Cette exception respecte le contrat : `Reads` seulement, sans effet, sans secret.

### Fixtures 089 et 152

- `core_089_concurrency` : la lecture `CallerScope` est comptée (`scope_reads == 8`). Les huit appels bloqués et le neuvième `Busy` restent asserts.
- `core_089_security` : la première connexion répond uniquement au `CallerScope`. Les cinq fautes restent injectées sur le `SendIdempotent` métier.
- `native_delegation_e2e` : le parent Codex est enregistré via le protocole réel, avec un fait de permission et un ACK accepté.
  Pas d'insertion DB ni de tâche forgée.
  La posture enfant passe à `Development` pour que le petit-enfant soit admis, ce qui suit FR-029.
  Les asserts `discovery` et `plan` du petit-enfant restent en place.

Je ne vois pas d'affaiblissement d'oracle dans ces trois fichiers. Ce n'est pas une preuve de PASS.

## Constats retenus

### C1 - LOW : message CLI trop large

- Fichier : `crates/bridget-daemon/src/cli.rs:6310-6315`.
- Constat : la condition est `command_id.is_some()`. Elle couvre aussi `resume-delegations`, qui n'annule rien.
- Scénario : `resume-delegations` reçoit `StoreUnavailable` avant écriture. Le message dit « l'arrêt ou les annulations ont pu être appliqués ».
- Correctif : restreindre la branche à une demande avec `cancel_active`, ou écrire « l'état d'arrêt peut avoir changé ».

### C2 - LOW : rapport de correctif inexact

- Constat : le rapport combiné dit que « le test156 entier retrouve son corps historique byte-identique ».
  C'est exact pour la section ChildProof seulement. La section positive garde `reply_caller_scope`.
- Correctif : corriger la formulation dans le rapport de correctif, sans toucher au code.

### C3 - LOW : pas de test pour le message CLI

- Constat : l'oracle de ronde 1 (erreur forcée sur `human_cancel`, réponse attendue) n'est pas implémenté.
  La branche de message n'est couverte par aucun test.
- Décision possible : accepter comme limite documentée, ou ajouter un test ciblé.

## Limites et preuves

- FULL final non relancé : le FULL `2440 P / 12 F / 74 I` est antérieur aux correctifs.
  Les quatre nouveaux FAIL sont couverts par des rejeux ciblés, un par test.
  Les 8 FAIL hérités 157 restent documentés. Le gate global reste à rejouer.
- Rejeux T021 (rc 0, un test chacun, format `[statut, passés, échoués, ignorés]` inféré) :
  - preview103 : 09:50:05 UTC, `1 ok`
  - concurrency089 : 09:56:32 UTC, `1 ok`
  - security089 : 09:57:12 UTC, `1 ok`
  - native152 : 10:00:01 UTC, `1 ok`
  - review-fixes lib : 09:49:16 UTC, `22 ok`, dont `guidance156_mcp_queue_schema`.
    Cette exécution démarre à la seconde où `mcp.rs` et `cli.rs` ont été écrits.
    Je ne peux pas prouver quel contenu a été compilé.
- Compilation et tests non relancés ici (interdit).
- Lecture partielle des hunks non lus en ronde 1 : `daemon.rs` presence_tests, `native_delegation.rs` tests Continue/pause (hors l'oracle `suspended`), `delegation.rs` tests store, `mcp.rs` tests, docs skill.
- Non lu : corps de `tree_settled` et de `set_delegation_halt`.
- Aucun succès global, fournisseur, modèle ou runtime 158 n'est établi par cette revue.
