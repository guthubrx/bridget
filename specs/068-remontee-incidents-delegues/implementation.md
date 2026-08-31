# Implementation - SPEC-068

## Statut

Implémentation terminée et validée en worktree. Aucun commit, push, déploiement
ou redémarrage n'a été effectué.

## Réalisation

- Le protocole porte trois trames fermées : publication enfant, remise parent et accusé parent.
- SQLite conserve les incidents non accusés dans `delegated_runtime_events`, déduplique par occurrence et les relit par parent, ordre `cursor`.
- Le parent est obtenu du lien durable enfant-parent. Il ne provient jamais de la trame envoyée par l'enfant.
- Le refus de requête fournisseur Codex connu expose seulement `unsupported_provider_request` et une empreinte `sha256:`, sans méthode ni paramètres bruts.
- Le wrapper enfant publie `warning` pour ce diagnostic et `failed` lors d'un terminal fournisseur en erreur.
- Le wrapper parent transforme la remise en message système `QueueOnly`. Il n'emploie ni interruption, ni steering, ni réponse métier.
- L'accusé part après `PromptDispatched` pour une session gérée et après l'injection tmux synchrone pour une session interactive.
- Après `Registered`, le daemon rejoue les faits non accusés dans l'ordre. Un accusé valide les retire avant toute reconnexion suivante.

## Preuves exécutées

| Commande | Résultat |
|---|---|
| `cargo test -p bridget-transport spec_068 --lib --no-fail-fast` | 1/1 vert |
| `cargo test -p bridget-daemon spec_068 --lib --no-fail-fast` | 7/7 verts |
| `cargo test -p bridget-transport --lib --quiet --no-fail-fast` | 218 verts, 1 ignoré |
| `cargo build -p bridget-daemon --bin bridget --quiet` puis `CARGO_BIN_EXE_bridget=/home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/target/debug/bridget cargo test -p bridget-daemon --lib --quiet --no-fail-fast` | 651 verts, 7 ignorés |
| `cargo check -p bridget-daemon -p bridget-transport --quiet` | vert |
| `cargo fmt --check` | vert |

La première exécution de la suite daemon sans binaire a échoué sur 8 tests de
supervision qui exigent `CARGO_BIN_EXE_bridget`. Après construction explicite
du binaire, la même suite est verte. Ce n'était pas une régression SPEC-068.

## Frontières vérifiées

La recherche des lignes ajoutées ne trouve aucun appel ajouté vers Maicie, le
guichet, `SteerCurrent`, `InterruptAndStart` ou `ControlExecution`.

## Self-review XIX/XX

- Nécessité : le lien de délégation existant ne portait que le cycle de vie, pas les incidents runtime.
- Simplicité : extension du protocole, de l'idempotency store, de Fleet et des wrappers existants. Aucune dépendance ni service nouveau.
- Hypothèse contrôlée : `PromptDispatched` est la frontière sûre de l'adaptateur géré. Le tracker existant est réutilisé.
- Non vérifié : aucun incident n'a été provoqué contre un fournisseur réel et aucun déploiement n'a été réalisé. Les tests utilisent les transports et sockets de test.
