# Plan 090 — réutiliser le noyau de session

Statut : implémenté, validé et adopté le 2026-09-06. Rust 1.92 / trois crates existants.

## Architecture et réutilisation

Complément reprise nom/menu : callback de sélection humaine avant le bootstrap
sur LE même app-server privé. Il lit `thread/list` paginé sous échéance globale,
puis passe un UUID attesté au `CodexThreadBootstrap::Resume` existant. La liaison
fil→identité est revalidée après résolution et avant Register. Aucun second
processus de catalogue, aucun stockage doublonné, aucune conversation provisoire.
Le menu garde le terminal canonique ; Ctrl-C/HUP/TERM annulent en laissant le
pilote arrêter son enfant. Les métadonnées rendues neutralisent les contrôles.

Amendement du 2026-09-06 : `Launch` distingue nom Bridget, alias explicite et
UUID de reprise. Le wrapper appelle le client partagé `rename_display_name`
après Register et avant la TUI. Le pilote réutilise `CodexThreadBootstrap::Resume`
et négocie réellement le fil privé avant présence ; la reprise automatique gérée
conserve sa garde d'attestation figée. Ni DTO de reprise bis ni écriture client
dans la base des profils. Le harnais PTY existant fournit la preuve de couture.

1. `wrapper::launch` dirige Codex interactif vers la boucle de session existante,
   avant tout enregistrement tmux. `--equipier` et les autres types inchangés.
2. `codex_interactive.rs` dans le daemon porte uniquement le cycle de vie TUI et
   les paramètres natifs : terminal exigé, socket privée, args cohérents, signaux,
   lancement de `codex resume THREAD --remote unix://PATH` après journal prêt.
3. `CodexAppServerTransport::spawn_interactive` possède le vrai app-server avec
   écoute Unix. Le reader, worker, événements, ACK et journal sont réutilisés.
   `codex_socket.rs` adapte la socket WS en flux Read/Write JSONL borné, sans
   interpréter ni reconstruire les messages JSON. Aucun processus proxy.
4. Le reader connaît le fil courant, observe les tours natifs externes, et attend
   les terminaux attestés. L'ACK interagent reste lié à sa vraie remise, jamais à
   un événement du tour humain. Pas de réponse automatique aux permissions natives.
5. Réutilisation du namespace, de l'injection MCP éphémère, des marqueurs typés,
   du tracker de remise et de reconnect_managed_session. L'instance reste stable.

## Répartition des fichiers

- transport : `src/codex_app_server.rs`, `src/codex_socket.rs`, `src/jsonl.rs`, `src/managed_session.rs`, `src/protocol.rs` (fait TerminalSessionReady), `src/lib.rs`, Cargo.toml.
- daemon : `src/wrapper.rs`, `src/codex_interactive.rs`, `src/daemon.rs` (reconnexion), `src/attach.rs` (input humain additif), `src/communication/client.rs`, `src/lib.rs`.
- tests : unitaires voisins + `tests/codex_interactive_090_test.rs` et son harnais Python, infrastructure PTY 089 réutilisée.
- docs : README FR/EN, skill Bridget canonique, installation, ADR 030.

## Invariants et coût

Une session interactive = une identité + un fil explicitement lié. Pas de
persistance automatique du processus à la sortie. Les opérations de navigation
qui changeraient ce lien doivent être prises en charge ou refusées explicitement,
jamais laisser les messages partir vers un ancien fil invisible.

Read/Write WS borné en taille et en temps, mémoire O(taille maximale de trame +
file déjà bornée) ; sélection initiale seule : catalogue limité à 1 000 résumés,
aucun historique de tours chargé ni collection persistante des conversations.
Socket accessible au seul compte Unix ; aucune prétention d'isolation contre
un autre processus malveillant du même compte. Pas de secrets dans les preuves.
Le provider reste expérimental : version incompatible = erreur avant présence.

## Gates constitution XIX/XX

Pas de nouveau service métier ni stockage doublonné. Nouveau module WS justifié
par frontière RFC6455 absente de l'existant ; nouveau module TUI par ownership
du terminal et de ses enfants, pas par simple encapsulation cosmétique.
La boucle wrapper, le canon et la clôture ne sont pas copiés. La nouvelle
dépendance doit être minimale (handshake seulement), auditée et documentée.

## Validation

Sonde actuelle → tests unitaires transport et parsing → vraie TUI sous PTY avec
app-server réel/fournisseur synthétique → daemon+CLI/MCP réels et abonnement si
disponible, demande/réponse/retry → pannes et permissions → workspace/fmt/clippy.
Tests ciblés pendant les tâches ; passe complète seulement après assemblage.
Production intouchée ; build dans le worktree, aucun commit automatique.

## Outillage SpecKit

Sync exécuté. Les scripts/templates officiels ne sont pas présents dans ce clone
extrait ; protocoles des skills specify/plan/audit-existing/tasks/analyze/implement
lus intégralement et appliqués manuellement. Ne pas réinstaller l'ancien produit
pour obtenir ses templates. Aucun hook extensions.yml présent.
