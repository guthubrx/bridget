# Audit de réutilisation - SPEC-072

## Méthode

Primitive speckit-audit-existing indisponible sur le serveur : audit manuel effectué le 2026-08-30 après lecture du registre, des transports, du cycle de lancement, des identités fournisseur, des contrats Cursor et de la SPEC-071 non fusionnée.

## Inventaire et décision

| Besoin proposé | Existant trouvé | Preuve | Décision |
|---|---|---|---|
| Lancer Cursor | AcpTransport et définition native Cursor | crates/bridget-daemon/src/registry.rs, crates/bridget-transport/src/acp.rs | RÉUTILISER |
| Lancer un compatible Anthropic | ClaudeStreamJsonTransport | crates/bridget-transport/src/claude_stream_json.rs | RÉUTILISER |
| Configurer un type d agent | AgentRegistry et AgentDefinition | crates/bridget-daemon/src/registry.rs | ÉTENDRE |
| Isoler l environnement enfant | build_environment | crates/bridget-daemon/src/lifecycle.rs | ÉTENDRE |
| Persister la définition et le digest | ResolvedAgentDefinition | crates/bridget-transport/src/protocol.rs | ÉTENDRE |
| Publier provenance d exécution | ManagedProviderIdentity et ManagedExecutionBinding | crates/bridget-daemon/src/wrapper.rs | ÉTENDRE |
| Afficher le runtime | SPEC-071 en cours | /home/moi/bridget-referent/.worktrees/session-071-identite-runtime-agent | NE PAS MODIFIER |
| Adapter Cursor séparé | aucun besoin, ACP est fonctionnel | tests/features/064-plan-controle-bridget-maicie.feature | REFUSER |
| Nouveau client HTTP GLM ou DeepSeek | aucun besoin, Claude Code est le client existant | crates/bridget-transport/src/claude_stream_json.rs | REFUSER |
| Carte générique de variables | aucun besoin démontré, trois profils Claude seulement | ADR 017 | REFUSER |

## Arbitrages

- Un champ claude_config_dir est créé plutôt qu une carte de variables. Il porte une règle de sécurité et de compatibilité réelle : sélectionner un settings.json isolé sans mettre un jeton dans le registre ou le digest.
- Le type d agent sert de provenance déclarée car il est déjà transmis au wrapper et enregistré dans le binding. Aucun nom affiché ou modèle n est utilisé comme indice.
- La présentation UI de runtime reste propriétaire de SPEC-071. SPEC-072 ne modifie aucun de ses fichiers non commités.

## Risques de chevauchement

| Zone | Risque | Mesure |
|---|---|---|
| app.js, theme.css, ui.rs de SPEC-071 | conflit d édition | aucun changement SPEC-072 dans ces fichiers |
| agents.json de production | secret ou changement de profil existant | sauvegarde, validation JSON, ajout strictement additif et permissions 0600 |
| profils Claude | contamination Anthropic | répertoires distincts et test parallèle |
| Cursor ACP | version CLI évolutive | vérifier help, authentification et échange réel avant activation |

## Gate avant tasks

- [x] Les transports existants couvrent Cursor et les compatibles Anthropic.
- [x] Aucun adaptateur Cursor ni client HTTP supplémentaire n est justifié.
- [x] Le seul nouveau champ proposé porte une contrainte sécurité réelle et trois usages actuels.
- [x] Les fichiers possédés par SPEC-071 sont exclus.
- [x] Le plan comprend des preuves d échec honnête et de non contamination.
- [x] Aucun doublon évident ne réclame arbitrage utilisateur.

## Verdict

CLEAR - Les tâches peuvent être générées.
