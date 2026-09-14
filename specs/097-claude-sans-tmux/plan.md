# Plan 097 — Claude natif et interactif sans tmux

Statut : proposé le 2026-09-13, Rust 1.92, trois crates existants, aucune
dépendance nouvelle. Socle : session 089 (noyau), schéma interactif 090.

## Architecture et réutilisation

1. `wrapper::launch` dirige `bridget claude` (hors `--equipier`) vers un
   nouveau module `claude_interactive.rs`, symétrique de `codex_interactive.rs` :
   vérification du terminal, ouverture du PTY, lancement du fournisseur, relais,
   restauration. Les autres types (`gemini`, `gclaude`, `--`) conservent la voie
   tmux, mais un pane absent devient un refus explicite au lieu d'un `warn!`.
   Le bypass de permissions ajouté implicitement pour Claude interactif est
   retiré (FR-09710) : l'humain est devant la TUI, il décide ; un bypass
   explicite de l'utilisateur est relayé sans modification.
2. `bridget-transport/src/pty.rs` : `PtyTransport` implémente le trait
   `Transport` existant à côté de `TmuxTransport`. `deliver` valide le contenu
   (validation extraite de `tmux.rs` et partagée), écrit l'enveloppe
   `wrap_envelope` en collage encadré dans le maître du PTY, attend un court
   délai de digestion puis écrit `\r`. `is_alive` = le processus enfant vit.
   Aucune capture d'écran ni heuristique de composer : l'écriture est
   synchrone et son succès vaut `PromptDispatched`, comme aujourd'hui pour tmux.
3. Le thread d'écoute existant du wrapper interactif est conservé tel quel :
   `transport` devient `Option<Box<dyn Transport>>` alimenté par tmux ou PTY,
   la remise idempotente, les rappels, la notification de reconnexion et le
   journal `turn_start` ne changent pas.
4. Présence : `connect_and_register(..., protocol = "claude_pty", mode =
   PresenceMode::Cli, location = None)`. Le daemon projette déjà `transport =
   valeur déclarée` pour le mode `cli` (daemon.rs §5852) ; `who` affiche donc
   `claude_pty | cli | —`. Aucune trame ni variante ajoutée.
5. Journal : `turn_start` Bridget déjà écrit avant injection. Les tours
   humains et assistant sont ajoutés en lisant le transcript déjà localisé par
   `ClaudeTranscriptLocator` (même fichier que la sonde modèle/effort), avec le
   vocabulaire déjà rendu par attach : `turn_start {from:"human"}`, `update
   {kind:"text"}` pour le texte assistant, `turn_end` ; corps bornés en taille.
   `bridget attach` fonctionne alors comme sur Codex interactif.
6. Recette Claude géré : le gate `gate_reel_claude_stream_json_reponse_liee_et_attach`
   cesse de lire le trousseau ; il lance le wrapper géré avec HOME réel et
   `USER` transmis, BRIDGET_HOME privé et registre privé (`--setting-sources ""`,
   `--strict-mcp-config`, `--tools ""`, modèle et effort explicites). Le reste du
   gate (demande suivie, réponse liée, attach, arrêt, zéro survivant) est
   inchangé. La preuve est consignée expurgée dans `implementation.md`.

## Répartition des fichiers

- transport : `src/pty.rs` (nouveau), `src/tmux.rs` (extraction de la
  validation de contenu), `src/lib.rs` (export).
- daemon : `src/claude_interactive.rs` (nouveau), `src/wrapper.rs` (dispatch,
  transport dynamique, refus sans pane, journal transcript), `src/lib.rs`.
- tests : `tests/claude_interactive_097_test.rs` + `tests/fixtures/claude_interactive_097.py`
  (faux fournisseur sous PTY : écho des octets reçus, taille de fenêtre,
  sortie sur commande) ; `tests/core_089_native_test.rs` (gate Claude corrigé) ;
  unitaires voisins dans `pty.rs`, `claude_interactive.rs`, `cli.rs` (who).
- docs : README FR/EN, `skills/bridget/SKILL.md` et références, ADR 033,
  `specs/097-claude-sans-tmux/implementation.md`.

## Invariants et coût

Une session interactive = une identité + un PTY possédé par le wrapper. Le
wrapper ne lit jamais la sortie du fournisseur pour décider d'une remise ; il
la relaie seulement. Le terminal réel est restauré à la sortie normale, sur
signal et sur panique (garde `Drop`). Les octets relayés ne sont ni journalisés
ni interprétés : seuls le transcript (source déjà utilisée) et les messages
Bridget alimentent le journal. Complexité : relais O(octets), une allocation de
tampon par sens ; remise O(taille du message) ; aucune boucle de sondage hors
digestion bornée (30 × 50 ms comme tmux).

Coût estimé : `pty.rs` ~180 lignes dont tests, `claude_interactive.rs` ~260,
`wrapper.rs` ±120, harnais Python ~150, test Rust ~200, gate 089 ±40.
Lignes retirées : contexte tmux et `TmuxTransport` pour Claude (~40).

## Gates constitution XIX/XX

- Réutilisation : trait `Transport`, `wrap_envelope`, validation de contenu,
  tracker idempotent, journal, relais attach, `ClaudeTranscriptLocator`,
  `check_terminal` (schéma 090), motif `openpty` des tests.
- Pas de dépendance nouvelle, pas de trame protocole, pas d'outil MCP, pas de
  variante d'énumération.
- Abstraction : `PtyTransport` est le deuxième implémenteur d'un trait qui en
  a déjà trois usages ; `claude_interactive.rs` suit un module existant.
- Non retenu : maintenir tmux pour Claude ; réécrire une TUI ; capturer l'écran
  pour vérifier le composer.

## Validation

- Harnais : faux `claude` sous PTY prouve relais frappe→enfant, enfant→écran,
  SIGWINCH→TIOCSWINSZ, remise Bridget = collage encadré + `\r`, saisie humaine
  partielle préservée, restauration termios, exit code transmis, refus sans
  terminal, refus PTY indisponible.
- Présence : `who`/`agents --json` montrent `claude_pty`/`cli` ; refus des types
  tmux sans pane.
- Recette réelle interactive : `bridget claude` dans iTerm, envoi depuis un
  Codex connecté, réponse liée au ledger, rejeu de l'incident du 13/09.
- Recette réelle gérée : gate 089 corrigé, un tour, preuves expurgées.
- Gates : fmt, clippy `-D warnings`, suite complète workspace en environnement
  privé (umask 077, racine courte sous /private/tmp).

## Outillage SpecKit

`research.md`, `data-model.md`, `contracts/claude-interactif.md`,
`quickstart.md`, puis `reuse-audit.md` avant `tasks.md`.
