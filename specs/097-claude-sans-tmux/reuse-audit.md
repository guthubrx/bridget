# Audit de reutilisation de l'existant — 097 Claude natif et interactif sans tmux

## Decision

Statut: PASS
Date: 2026-09-13
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/097-claude-sans-tmux/specs/097-claude-sans-tmux
Conclusion courte: le plan réutilise le trait `Transport`, l'enveloppe, la validation de contenu tmux, le tracker idempotent, le journal, le relais attach, la sonde de transcript et le motif `openpty` déjà présents. Deux créations sont retenues et justifiées (transport PTY, module de session Claude), une exposition d'utilitaire (géométrie du terminal), une extraction (validation de contenu). Aucune duplication évidente.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 14 |
| Items audites | 14 |
| Reutilisations deja prevues | 8 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Trait `Transport` pour le PTY | `pub trait Transport` | `crates/bridget-transport/src/transport.rs:30` | deuxième implémenteur interactif après tmux |
| Enveloppe du message remis | `wrap_envelope` | `crates/bridget-core/src/envelope.rs:11` | identique tmux/PTY |
| Validation du contenu injecté | `validate_tmux_content` | `crates/bridget-transport/src/tmux.rs:100` | extraite en fonction partagée, sans changer ses règles |
| Plafond d'injection | `INJECTED_BODY_MAX_CHARS` | `crates/bridget-daemon/src/wrapper.rs:300` | inchangé |
| Remise idempotente et accusés | `deliver_idempotent_to_interactive`, `IdempotentDeliveryTracker` | `crates/bridget-daemon/src/wrapper.rs:751`, `:572` | inchangés, le PTY remplace la fermeture `inject` |
| Journal et relais attach | `JournalWriter::enqueue`, `AttachRelayWorker` | `crates/bridget-transport/src/journal.rs:427`, `crates/bridget-daemon/src/wrapper.rs:1590` | mêmes événements `turn_start` / `update` / `turn_end` |
| Localisation du transcript Claude | `ClaudeTranscriptLocator` | `crates/bridget-daemon/src/wrapper.rs:1019` | source des tours humain/assistant du journal |
| Vérification du terminal | `codex_interactive::Launch::check_terminal` | `crates/bridget-daemon/src/codex_interactive.rs:184` | même règle, message Claude |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Mode brut du terminal hôte | garde termios d'attach (désactive ICANON/ECHO/ISIG, protocole clavier) | `crates/bridget-daemon/src/attach.rs:100-160` | arbitré ci-dessous : garde `cfmakeraw` dédié |
| Taille de fenêtre | `terminal_geometry(fd)` | `crates/bridget-daemon/src/attach.rs:2556` | réutiliser en la rendant `pub(crate)` |
| Pseudo-terminal `openpty` | `PseudoTerminal` des tests d'attach | `crates/bridget-daemon/src/attach.rs:5450` | motif repris ; code de test, non partageable tel quel |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucune | — | — | — |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XIX | pas d'abstraction < 3 usages, pas de dépendance nouvelle | `libc` seul, pas de `portable-pty` ; pas de trait nouveau |
| `~/.speckit/constitution.md` XX | code assumable, self-review, hypothèse vérifiable | sondes d'authentification consignées dans research.md |
| `~/.speckit/constitution.md` XVIII | complexité documentée | relais O(octets), digestion bornée comme tmux |
| `~/.speckit/ref/standards-tests.md` | une spec = un `.feature` Gherkin | créer `tests/features/097-claude-sans-tmux.feature` |
| Mémoire projet `bridget-tests-umask-077-env-prive` | gates en environnement privé, umask 077 | commande de gate fixée dans quickstart |
| `AGENTS.md` | pas de daemon/wrapper sans home/socket isolés | recettes sous BRIDGET_HOME privé ; HOME réel seulement pour l'authentification du fournisseur, décision de research.md |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 090 Codex interactif | interface native + mode `cli` + transport nommé, harnais Python sous PTY, refus sans terminal | schéma repris pour Claude ; `probe_shared_session.py` réutilisable pour le harnais |
| 089 noyau | gate réel `core_089_native_test`, isolation `fixture::isolated_command` | gate Claude corrigé plutôt que recréé |
| 091/093 attach | événements de journal rendus par attach (`turn_start`, `update`, `turn_end`) | aucun nouvel événement à rendre |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "PseudoTerminal|openpty|Pty[A-Z]"` | crates/*/src | uniquement tests d'attach (5450-5560) |
| `rg "dyn Transport"` | crates | aucun ; transport typé `Option<TmuxTransport>` à wrapper.rs:1712 |
| `rg "fn validate_tmux_content|INJECTED_BODY_MAX_CHARS"` | crates/*/src | tmux.rs:100, wrapper.rs:300 |
| `rg "\"turn_start\"|\"turn_end\"|\"user_message\""` | crates | journal.rs, claude_stream_json.rs:775, acp.rs, attach.rs:3264 |
| `rg "cfmakeraw|tcsetattr|SIGWINCH|TIOCGWINSZ"` | crates/*/src | attach.rs seulement, aucun SIGWINCH |
| `rg "200~|bracketed"` | crates | harnais 090 (collage encadré déjà utilisé côté humain simulé) |
| `rg "fn legacy_non_protocol_transport"` | daemon.rs | `claude_pty` n'est pas filtré : affiché tel quel en mode cli |
| `ls tests/features` | tests | un `.feature` par spec depuis 088 |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Garde termios hôte | créer un garde `cfmakeraw` dédié dans `claude_interactive.rs` | le garde d'attach garde OPOST/IXON et pousse un protocole clavier ; un relais transparent exige le brut complet. Généraliser le garde d'attach toucherait un fichier de 8 000 lignes pour un drapeau | 2026-09-13 |
| Géométrie du terminal | réutiliser `attach::terminal_geometry` en `pub(crate)` | fonction pure de 8 lignes, déjà testée | 2026-09-13 |
| Validation du contenu | extraire `validate_tmux_content` en `validate_injection_content` partagée | mêmes règles pour deux voies ; tmux garde son alias | 2026-09-13 |
| Événements journal | réutiliser `turn_start`/`update`/`turn_end` | rendus par attach sans changement | 2026-09-13 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
