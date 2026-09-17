# Audit de reutilisation de l'existant — 102-fils-inter-agents

## Decision

Statut: PASS
Date: 2026-09-16
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents/specs/102-fils-inter-agents

Conclusion : aucune reconstruction d'un fil collectif existant détectée. Le plan
étend le daemon, son stockage et ses façades ; il garde un état métier propre
pour les obligations que les observations et le ledger ne couvrent pas.
L'intégration101 reste une précondition future de code, pas un doublon à arbitrer.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 21 |
| Items audites | 21 |
| Reutilisations deja prevues | 11 |
| Existants potentiellement pertinents | 10 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 7 |

## Reutilisations correctement identifiees

Preuves sous la racine /Users/moi/Nextcloud/10.Scripts/64.bridget ; `D/` désigne
crates/bridget-daemon/src/ ; les lignes101 désignent explicitement le worktree101.

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| 01 Identité de session | registered_connection + preuve auxiliaire | D/communication/client.rs:766 ; D/mcp_identity.rs:109 | aucune nouvelle authentification |
| 02 Autorité serveur | live_connection_identity | .worktrees/101-abonnements-t3/crates/bridget-daemon/src/daemon.rs:7339 | contrôle avant appartenance |
| 03 Client CLI/MCP | DaemonConnection et observation_request | D/communication/client.rs:19 | nouvelle méthode, même budget10s |
| 04 Famille CLI | dispatch/parse strict | D/cli.rs:178, :718 | ajout thread, pas shell générique |
| 05 Outil MCP | catalogue fermé et execute_tool_at_with_scope | D/mcp.rs:438, :2060, :2172 | ajout bridget_thread et inventaire tests |
| 06 Connexion/migration Store | Store et sous-modules SQL | D/store.rs:32, :40, :56 | même DB, schéma additif séparé |
| 07 Livraison dédupliquée | IdempotentDeliveryTracker + ReceiptStore + scope interne | D/wrapper.rs:572, :617, :649 ; D/idempotency/send_delivery.rs:176, :623 ; D/idempotency.rs:488 ; D/daemon.rs:4140 ; D/communication.rs:64 | ACK injection seulement ; étendre canon conditionnellement et exclure notices de réaffectation, sans casser DM |
| 08 Pagination et bornes | forme de lecture attach | D/attach.rs:30 | modèle réutilisé, pas le stockage du journal |
| 19 Négociation des alertes | Register/Registered et connect_and_register_at | crates/bridget-transport/src/protocol.rs:2462/3356 ; D/wrapper.rs:1570 | étendre avec champs facultatifs, pas ClientCapability |
| 20 Réponse implicite | fichier last-sender et parse reply | D/wrapper.rs:2239 ; D/cli.rs:3689 | marqueur typé dans le contexte existant ; pas de second registre |
| 21 Autorisations des outils | BRIDGET_SAFE_MCP_TOOLS | D/wrapper.rs:4568, :4584, :4792 | 14→15 outils nommés, aucune autorisation globale |

## Existant potentiellement pertinent non mentionne

Les dix candidats proches sont désormais tous mentionnés et tranchés dans le plan.

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| 09 Contrat ThreadRequest/Result | ObservationRequest et CoordinationSubscribe | crates/bridget-transport/src/protocol.rs ; D/observation.rs | CREER contrat fermé : ni abonnement temporaire ni guichet Maicie |
| 10 Métadonnée ThreadNoticeV1 | BridgetMessage origin/intent/references | crates/bridget-core/src/message.rs:73 | ETENDRE : aucun champ de fil collectif typé existant |
| 11 Module threads.rs | observation.rs | D/observation.rs:427 (101) ; D/daemon.rs:1280 | CREER : droits, curseurs, coalescence durable ; pas un autre transport |
| 12 SQL store/threads.rs | store/ledger_requests.rs | D/store/ledger_requests.rs:794 | CREER sous-module métier suivant structure existante |
| 13 discussion_threads | conversation_key ledger et thread_id T3 | D/store.rs:68 ; D/t3code.rs | CREER : conversation_key point à point, thread_id session fournisseur |
| 14 discussion_members | abonnements observations | D/observation.rs:118, :242 (101) | CREER : ACL durable ≠ abonnement interrompu après restart |
| 15 discussion_entries | ledger et journal exécution | D/store.rs:68 ; D/attach.rs:30 | CREER : historique collectif avec droit de lecture membre |
| 16 thread_operations | idempotency_records | D/idempotency.rs:23, :512 ; D/idempotency/send_delivery.rs:48 | CREER : socle fermé Send/Spawn, ne pas usurper Send |
| 17 thread_reads | AttachClientState / ACK livraison | D/attach.rs:30 ; D/wrapper.rs:649 | CREER : reçu de page lié à acteur/fil, pas preuve de tour |
| 18 thread_wakes | queue notifications observation + send_deliveries | D/daemon.rs:1280/1305 ; D/idempotency.rs:541 | CREER état coalescé métier ; réutiliser transport099 après commit |

## Duplications evidentes

Aucune. Le module SQL et les tables nouvelles ne sont pas des copies du journal,
du ledger ou des abonnements : les invariants et droits diffèrent. Aucun package,
daemon, broker, outil de résumé ou nouveau framework n'est proposé.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | isolation, français, pas d'interruption fournisseurs | worktree102, docs seules |
| /Users/moi/.speckit/constitution.md XVIII | complexité | index, pages et fanout bornés |
| même fichier XIX–XX | réutilisation, responsabilité | état spécialisé justifié, aucun wrapper décoratif |
| .specify/memory/constitution.md | pont global | constitution lue, aucune règle locale inventée |
| .specify/memory/standards.md et standards-tests.md | tests traçables | BDD + Rust natif, pas pytest hors technologie |
| skill my-specify-all + demande utilisateur | pas de commit ; sans implémentation | arrêt après Analyze, tâches non cochées |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 089 | communication/journal/authentification | socle |
| 094 | parité CLI/MCP/skill | surfaces et inventaires obligatoires |
| 097 | Claude interactif sans tmux | adapter sans faux DM |
| 098 | T3 et relais final automatique | distinguer explicitement sollicitation de fil |
| 099 | identité/proof/idempotence bornée | réutiliser pour livraison, pas pour persistance de post |
| 100 | lecture bornée, observations non bloquantes | réutiliser patron, préserver sémantique |
| 101 | observations T3 et identité MCP (worktree non intégré) | référence examinée ; intégration exigée avant modification commune |

## Journal de recherche

Exploration unique déléguée à reuse_102 en lecture seule, résultats reçus avant
tasks. Lecture ciblée principale du protocole, Store et CLI en complément.

| Requete | Portee | Resultat |
|---|---|---|
| rg thread/group/channel/conversation | crates, specs89–101 | sessions T3 et ledger point à point ; aucun fil collectif |
| rg core_operations/idempotency_records/OperationKind | idempotency et communication | aucun core_operations, Send/Spawn fermés |
| rg DeliverIdempotent/DeliverAcked/ReceiptStore | protocol, wrapper, T3 | ACK injecté, pas lu |
| rg observation/once/pending | daemon/observation/T3 | best effort et piège relais final |
| rg registered_connection/live_connection_identity | communication/MCP/daemon101 | réutilisation attestation |
| manifests Cargo.toml | workspace et crates | serde/rusqlite/uuid/sha2 existants, zéro dépendance nouvelle |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Fil dans ledger | créer tables spécialisées | ledger lisible plus largement et schéma1:1 | 2026-09-16 |
| Idempotence post | créer thread_operations | Send écrit ledger et une seule livraison ; pas même opération | 2026-09-16 |
| Mention comme abonnement | créer thread_wakes | once consommé avant livraison, reprise interrupted | 2026-09-16 |
| Queue nouvelle | réutiliser remise099 | seules intentions métier nouvelles, aucun broker | 2026-09-16 |
| Résumé | réutiliser agent/history/post | pas de fonction de génération supplémentaire | 2026-09-16 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

Le plan a été rédigé directement sur ces résultats ; aucun refactor de réutilisation
postérieur nécessitant arbitrage utilisateur. Gate PASS avant création tasks.md.
