# Audit de reutilisation de l'existant — 090

## Decision

Statut: PASS
Date: 2026-09-05
Feature dir: specs/090-codex-interactif

Un seul pilote et une seule boucle wrapper. Les deux modules nouveaux portent
respectivement la compatibilité WS et l'ownership TUI, absents de l'existant.
Exploration indépendante codex_session_reuse puis vérification locale des symboles.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 8 |
| Items audites | 8 |
| Reutilisations deja prevues | 6 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes non arbitrees | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Session Codex | CodexAppServerTransport | crates/bridget-transport/src/codex_app_server.rs:247 | Généraliser E/S, pas second pilote. |
| Wrapper interactif | launch_acp_with_status | crates/bridget-daemon/src/wrapper.rs:3320 | Paramétrer amorçage et supervision TUI, ne pas copier boucle. |
| Connexion Unix bornée | connect_nonblocking | crates/bridget-daemon/src/communication/client.rs:322 | Déplacer en couche transport, réexporter à l'ancien point. |
| Paramètres CLI | codex_option_takes_value/positionals | crates/bridget-daemon/src/wrapper.rs:544 | Réutiliser classification avant spécialisation serveur/TUI. |
| Identité/namespace/ACK | managed_adapter_environment, tracker | crates/bridget-daemon/src/wrapper.rs:3941, :670 | Même scope stable, fichier du nom, marqueur PID serveur. |
| Tests PTY | fixture 089 + attach | crates/bridget-daemon/tests/core_089_host_session_test.rs:128 ; src/attach.rs:2407 | Réutiliser motif, éviter framework de terminal. |
| WS Unix | aucun | Cargo.lock/Cargo.toml sans WS | codex_socket.rs + tungstenite sans TLS, framing maintenu par bibliothèque. |
| TUI ownership | aucun équivalent complet | wrapper.rs:1511 tmux ; managed_process.rs:403 spawn supervisé | codex_interactive.rs avec garde terminal/sockets/arrêt, pas gestionnaire de flotte. |

## Existant potentiellement pertinent non mentionne

Lecture JSONL bornée : transport/src/jsonl.rs:21. Pas de recopie de son algorithme :
les frames WS sont déjà délimitées par tungstenite, dont la borne s'applique.
Le parser commun reçoit la chaîne source entière, sans re-sérialisation.

## Duplications evidentes

Aucune : connexion Unix existante déplacée (pas copiée), classification CLI
existante réutilisée, mécanismes métier laissés à leur point d'autorité.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| Constitution XIX/XX | minimalisme | Une bibliothèque framing, aucune sémantique doublonnée. |
| AGENTS.md | isolation | Worktree, home/socket de test privés ; production intacte. |
| Noyau 089 | raw/provenance/idempotence | Ne pas remplacer les oracles par des mocks internes. |
| my-specify-all | pas de commit automatique | Diff relisible, aucun changement de déploiement. |
| standards-tests | preuve observable | Scénarios BDD et tests Rust/PTY adaptés au projet. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 089 | namespace et noyau autonome | Obligatoire pour chaque test processus. |
| 008 | journal, attach, PTY | Conservation du flux et de son identité. |
| 012 | canon/retry/ACK | Aucun nouveau store ou canon de livraison. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg CodexAppServerTransport/launch_acp/ManagedSession | crates | pilote et boucle réutilisables |
| rg tungstenite/websocket | manifestes/lock | aucun framing WS existant |
| rg connect_nonblocking/read_unix_line | crates | primitives existantes trouvées |
| rg openpty/forkpty | tests/attach | infrastructure PTY trouvée |
| rg codex_option_takes_value/codex_positionals | wrapper | parsing déjà présent |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Pilote et boucle wrapper | réutiliser | mêmes responsabilités, préserver invariants prouvés | 2026-09-05 |
| codex_socket.rs/tungstenite | créer | WS absent ; proxy officiel incompatible empiriquement ; bibliothèque plus sûre qu'un RFC6455 maison | 2026-09-05 |
| codex_interactive.rs | créer | ownership terminal et paramètres interactifs distincts du superviseur de flotte | 2026-09-05 |
| sonde Python | créer | protocole réel sans secrets ni coût fournisseur, aucun usage production | 2026-09-05 |
| Documentation/ADR/scénarios | créer 090, étendre README/skill | traçabilité de la nouvelle feature, pas doublon produit | 2026-09-05 |

## Gate avant tasks

Arbitrages d'implémentation : `stop_owned_child` étend managed_session.rs après
recherche des chemins `child.wait/kill/process_group` : les arrêts existants de
flotte sont des opérations de superviseur, non un garde TUI ; factorisation de
l'arrêt borné TUI/app-server, sans recopier flotte ni changer le stdio géré.
`validate_unix_socket_path` extrait la validation déjà présente dans la connexion
JSONL : réutilisée AVANT lancement WS, pas une deuxième règle SUN_LEN.
La présence Cli + journal_ready est aussi celle du Codex géré : elle ne permet
PAS de déduire le propriétaire du cycle de vie. `TerminalSessionReady`, fait
additif au protocole existant, est envoyé après lancement de la TUI et à chaque
reconnexion. Un ensemble par connexion, purgé à la fermeture, permet au daemon de
laisser la TUI reconnecter ; aucun nouveau canal ni store. La suite native gérée
a réfuté la première inférence, désormais remplacée et gardée par un test.
Le connect wrapper réutilise `jsonl::connect_nonblocking` ; write_timeout borne
le flush, puis l'échec ferme le lien partagé avant destruction de BufWriter.
Le heartbeat passe par le même point d'émission, sans seconde gestion d'erreur.
`user_message` réemploie from/body et le renderer d'entrée existant : un steer
humain n'est ni un début de tour supplémentaire ni une réponse de l'assistant.
Le harnais terminal garde son shell-parent vivant pour mesurer termios sur Darwin
avant révocation du PTY ; aucune infrastructure terminal n'entre en production.
Le harnais commun 089 ne doit importer test_sync qu'avec test-support : gate
de compilation conditionnelle restauré, sans désactiver un test ni modifier
la production. Armer une barrière dans le build sans feature est refusé.
Les branches Codex/tmux après le retour natif sont inaccessibles : suppression
de leurs helpers de reprise et de 5 tests de leur ancienne transformation argv.
Les 3 scénarios faux-Codex/faux-tmux de managed_parity sont remplacés par les
recettes Codex natives explicites ; pas de voie cachée pour faire passer le banc.
Le corpus automatique géré ACP, les tests exacts du prompt Claude et les tests
MCP de réponse/canon/identité core_089 restent exécutés. Cette réaffectation est
un changement de contrat interactif assumé, pas une non-régression du mode tmux.

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md prévoit la réutilisation et les divergences sont justifiees
