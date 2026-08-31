# Audit de réutilisation de l'existant - SPEC-068

## Decision

Statut: PASS
Date: 2026-08-30
Feature dir: /home/moi/bridget-referent/.worktrees/session-068-remontee-incidents-delegues/specs/068-remontee-incidents-delegues

Conclusion courte: le registre de liens, la flotte, le protocole
daemon-wrapper et les notifications fournisseur existent déjà et sont étendus.
Aucun composant existant ne persiste un incident runtime délégué avec son
accusé parent. Le flux de coordination Maicie est volontairement non réutilisé:
sa sémantique dépend du guichet et ne peut pas devenir une vérité runtime.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 7 |
| Items audites | 7 |
| Reutilisations deja prevues | 5 |
| Existants potentiellement pertinents | 1 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Lien parent-enfant | `AgentLinkRecord` | `crates/bridget-daemon/src/idempotency.rs:311` | source d'autorité de parent et enfant |
| Recherche de lien enfant | `Fleet::agent_link_for_child` | `crates/bridget-daemon/src/fleet.rs:557` | aucun parent ne vient du client |
| Réveil de lecteurs | `agent_link_changed` | `crates/bridget-daemon/src/fleet.rs:605` | même notification locale après insertion |
| Contrat interne | `WrapperToDaemon` et `DaemonToWrapper` | `crates/bridget-transport/src/protocol.rs:897` | extension de trames, pas nouveau canal |
| Remise parent | `Transport::deliver` | `crates/bridget-daemon/src/wrapper.rs:3334` | notification à file normale, sans interruption |
| Diagnostic Codex stable | `unsupported_provider_request_response` | `crates/bridget-transport/src/codex_app_server.rs:1559` | code et référence déjà redacted |
| Terminal enfant | `ManagedTerminal::Failed` | `crates/bridget-daemon/src/wrapper.rs:4426` | seule source de l'échec terminal |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Flux durable cursé | `guichet_coordination_events` | `crates/bridget-daemon/src/store.rs:313` | Ne pas réutiliser: il porte une autorité et des identifiants Maicie, incompatibles avec le runtime Bridget |

## Duplications evidentes

Aucune. Le flux Maicie voisin est volontairement écarté avec preuve de
frontière, pas ignoré.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/home/moi/.speckit/constitution.md` Art. XVIII | complexité documentée | sélection indexée, pas de balayage de flotte |
| `/home/moi/.speckit/constitution.md` Art. XIX | réutiliser avant créer | extension des liens, de la flotte et du protocole |
| `/home/moi/.speckit/constitution.md` Art. XX | code explicable | deux catégories fermées, aucun wrapper vide |
| `/home/moi/.speckit/research/01-ai-agents-agentic-ai.md` | limites et échecs explicites | avertissement distinct d'une assertion d'échec |
| `/home/moi/.speckit/research/04-architectures-patterns.md` | observabilité des défaillances | fait durable, corrélé et redacted |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-052 | corrélation durable sans parser un texte | identifiants typés, pas de décision depuis le corps projeté |
| SPEC-063 | interruption et terminal distincts | aucun incident ne simule une interruption |
| SPEC-064 | Bridget exécution, Maicie mission | pas de transition Maicie depuis le runtime |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg AgentLinkEvent agent_link` | daemon et transport | liens persistants mais lifecycle seul |
| `rg ManagedEventKind::Error` | wrapper et adaptateurs | erreur gérée actuellement ignorée par le wrapper |
| `rg acknowledged_at receipt event` | crates et specs | aucun accusé runtime délégué équivalent |
| `rg CoordinationEvent` | store et protocole | flux Maicie présent mais hors frontière |
| `rg SteerCurrent` | trois adaptateurs | non portable, notification à file normale retenue |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Persistance de faits runtime | creer dans `IdempotencyStore` | aucun équivalent hors du flux Maicie qui ne peut être mélangé | 2026-08-30 |
| Livraison parent | reutiliser `Transport::deliver` | les adaptateurs gèrent déjà leurs files; aucune interruption forcée | 2026-08-30 |
| Diffusion Maicie | ne pas reutiliser | mission et exécution sont des autorités distinctes | 2026-08-30 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
