# Rapport d'analyse - Session151 (version révisée après contre-revue)

Date : 2026-10-11. Portée : `spec.md`, `plan.md`, `tasks.md`, `research.md`,
`data-model.md`, `contracts/attente151.md`, `reuse-audit.md`, `quickstart.md`,
`adversarial-review-codex.md`, contrôle contre la constitution (Art. III, VII, XVIII,
XIX, XX). Lecture seule ; aucun test ni build lancé. Les affirmations sur le code
viennent de la lecture du dépôt principal à la base `1f2c661d` et de
`specs/150-socle-delegation/` (supposée livrée).

## Constats

| # | Gravité | Constat | Statut |
|---|---|---|---|
| A1 | haute | Première version : l'attente écrivait « lu » et `delivery_acked` avant l'écriture de la réponse (contre-revue 1, 2, 6) | Corrigé : l'attente est une lecture pure (FR-004, FR-019, FR-020), `result_read_at`, verrou d'attente et branche de `tick` supprimés |
| A2 | haute | `ClientCapability` inconnue et variante inconnue de `NativeDelegationRequest` rendent un ancien daemon muet (contre-revue 5) | Corrigé : sonde par `DaemonIdentityReport.features` (FR-031), négociation de capacité supprimée |
| A3 | haute | `bridget_send` avec attente cachait `id` et `issued_at` jusqu'au retour (contre-revue 7) | Corrigé : outil séparé `bridget_request_wait`, `bridget_send` inchangé |
| A4 | haute | `reply_message_id` vers le ledger : clé `(id, target)` remplaçable, identifiants par scope (contre-revue 3) | Corrigé : `reply_summary` figé dans le `UPDATE` de `answered` (FR-028) |
| A5 | moyenne | `answered` sans réponse (guichet, lignes anciennes) alors que le contrat exigeait `reply` (contre-revue 4) | Corrigé : `reply: null` et `reply_reason` (FR-028, contrat) |
| A6 | moyenne | 8 attentes saturent les 8 créneaux de la façade ; annulation MCP sans effet sur l'exécution (contre-revue 2, 8) | Corrigé : FR-012, FR-013, T012 ; ordre des verrous écrit (R3) |
| A7 | moyenne | Le couplage de 153 sur l'écriture de lecture de l'attente disparaît | Restant hors de ce dossier : 153 doit être corrigée (voir contre-revue, « Impact sur 153 ») |
| A8 | haute | Trois faits ne sont pas prouvés : délai d'outil de Codex et de Claude Code ; absence d'inactivité sur une connexion cliente du daemon ; absence de tests qui figent `task_view`, le catalogue ou `DaemonIdentityReport` | Restant : T001 (a) à (f). Si (e) échoue, une tâche s'ajoute avant T010 |
| A9 | moyenne | Nom exact de la lecture de phase de remise de 150-T005, non lisible dans le dépôt principal | Restant : T001 (b) |
| A10 | moyenne | `DaemonIdentityRequest` est-il accepté sans enregistrement, et `RequestWait` doit-il figurer dans les deux listes de trames (`daemon.rs:14384`, `:14615`) ? | Restant : T001 (f) ; T016 le traite |
| A11 | moyenne | Doublon : l'attente rend le résumé, puis le message arrive (et la réponse d'un envoi est déjà injectée) | Accepté et documenté (FR-021, FR-030, question 3) |
| A12 | faible | Un redémarrage du daemon tue les attentes ouvertes | Accepté : lecture pure, on rappelle ; les messages repartent par `tick` |
| A13 | faible | Colonne ajoutée à deux définitions de `tracked_requests` | Couvert : T004 (migration gardée, test d'ancienne base) |
| A14 | faible | `delegation_mcp.rs` a un test qui exige 4 outils ; le plan en ajoute 1 (puis 2 avec US7) | Couvert : T012 et T025 mettent à jour le test |
| A15 | faible | La spec 150 n'est pas entièrement cochée (T012 à T024 de 150) | Noté : 151 s'exécute après 150 ; prérequis : `delivery_acked`, T005 de 150, `reply_timeout` |
| A16 | faible | Le regroupement hors attente (US4) existe déjà depuis la session 114 | Conforme à l'Art. XIX ; T014 ajoute un test |
| A18 | haute | Ronde 2 : annulation MCP avant enregistrement du socket (R2-1) | Corrigé : registre `{annulé, socket}` sous un verrou, enregistrement avant l'authentification (FR-013, T026) |
| A19 | haute | Ronde 2 : EOF de l'entrée MCP, `thread::scope` attend des appels d'une heure (R2-2) | Corrigé : `cancel_all` par garde `Drop` avant la jonction (FR-038, SC-012, T026) |
| A20 | moyenne | Ronde 2 : sommeil fixe de 5 s (R2-3) | Corrigé : `min(5 s, délai restant)` (FR-007, SC-013, T003, T011, T016) |
| A21 | moyenne | Ronde 2 : la réponse du guichet n'incrémente pas le compteur (R2-4) | Corrigé : incrément au site d'appel `daemon.rs:16074` (FR-039, T001(g), T016) |
| A17 | faible | Aucun terme interdit (tiret cadratin) dans les artefacts | Vérifié par relecture |

## Couverture FR et SC vers tâches

| Exigence | Tâches |
|---|---|
| FR-001, FR-002, FR-003 | T002, T010, T011, T012 |
| FR-004, FR-019, FR-020, FR-021, FR-022 | T010, T013, T023 |
| FR-005, FR-007 | T003, T010, T011, T016 |
| FR-006 | T010, T013 |
| FR-008 | T010, T011, T016 |
| FR-039 | T001, T016 |
| FR-009 | T010, T011 |
| FR-010 | T010 |
| FR-011 | T012 |
| FR-012 | T012 |
| FR-013, FR-038 | T026 |
| FR-014, FR-015, FR-016, FR-017 | T006, T007 |
| FR-018 | T008, T009 |
| FR-023, FR-024 | T014 |
| FR-025, FR-026, FR-027, FR-029 | T015, T016, T017 |
| FR-028 | T004, T015 |
| FR-030 | T019 (documentation) |
| FR-031 | T002, T005, T012, T018 |
| FR-032, FR-033 | T012 |
| FR-034 | T007, T021 |
| FR-035 | T019 |
| FR-036, FR-037 | T024, T025 |
| SC-001, SC-002, SC-003, SC-006 | T013 |
| SC-004 | T014 |
| SC-005 | T010 |
| SC-007 | T021 |
| SC-008 | T012, T022 |
| SC-009 | T004, T015, T017 |
| SC-010, SC-012 | T026 |
| SC-013 | T010, T016 |
| SC-011 | T018 |

Toutes les FR et tous les SC ont au moins une tâche. FR-030 est une limite
documentée, sans comportement à tester.

## Contrôle constitution

| Article | Verdict | Remarque |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches, analyse, contre-revue |
| VII ADR | OK | T020 |
| XVIII Complexité | OK | O(k), k au plus 16 ; 64 attentes ; 4 par façade ; aucun balayage de `send_deliveries` |
| XIX Minimalisme | OK | pas de table, pas de module, plus de champ de livraison ; US7 supprimable |
| XX Responsabilité | OK avec réserve | faits non prouvés en T001 (A8 à A10) ; l'invariant « l'attente n'écrit pas la livraison » est testé (T010) et relu (T023) |

## Constats restants

- A7 : corriger la spec 153 (hors de ce dossier).
- A8, A9, A10 : à lever en T001 avant tout code.
- Ronde 2 de la contre-revue : quatre points intégrés (A18 à A21). Un test d'intégration
  réel `serve` avec un hôte qui ferme stdin n'est pas prévu : T026 utilise un exécuteur
  injecté et des paires de sockets.
- Les trois questions de la spec ont une valeur par défaut et ne bloquent pas T001.

## Analyze exécuté dans ce tour avant code

Primitive : protocole de la skill `speckit-analyze`, contrôle manuel des fichiers
(son script de prérequis manque réellement dans le dépôt ; commande tentée).
Première passe : six incohérences non critiques identifiées. Auto-fix du pipeline
puis seconde lecture : aucune contradiction restante dans ces six points.

| ID | Gravité | Constat initial | Correction vérifiée en deuxième passe |
|---|---|---|---|
| RUN1 | haute | deux phrases du reuse-audit attribuaient une écriture à l'attente | lecture pure et envoi inchangé |
| RUN2 | moyenne | `ready.result` nul dans le contrat mais absent dans FR-014 | clé absente, conforme à FR-014 |
| RUN3 | moyenne | worktree 150 alors que l'implémentation est isolée en 151 | chemins actuels et import du socle tracés |
| RUN4 | moyenne | expiration et mutations directes hors compteur | T003 couvre human_cancel/delete_tree, T016 couvre claim_timeout |
| RUN5 | faible | nombre d'outils sûrs 21 au lieu de 23 | audit corrigé, test fermé maintenu |
| RUN6 | moyenne | deux cases de forme ouvertes | levées explicitement avec justification dans la checklist |

Inventaire : 39 FR, 13 SC, 26 tâches, couverture documentaire 100 %.
0 CRITICAL. Aucun service/table/module/dépendance inutile identifié.
Contre-revue du plan demandée à Claude Sonnet 5.5 via T3, en cours.
Les vérifications empiriques T001 se terminent avant tout code.
Les questions avec valeurs par défaut ne bloquent pas l'exécution complète.

### Troisième lecture après contre-revue inter-fournisseurs

Verdict reçu APPROVE_WITH_CHANGES. Objections vérifiées dans le code ; plan et
contrat complétés par plomberie du socket, peek non bloquant, phases absentes/orphaned,
SELECT unique, première clôture stable, sonde annulable et ordre des refus par couche.
Ces ajouts précisent des tâches existantes : toujours 26 tâches, aucun CRITICAL
restant avant implémentation. L'allégation « flux inaccessible » est retenue comme
plomberie manquante dans le plan, pas comme impossibilité du code : la connexion
possède bien le socket.
