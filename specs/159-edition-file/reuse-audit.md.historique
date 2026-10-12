# Audit de reutilisation de l'existant — 159-edition-file

## Decision

Statut: PASS
Date: 2026-10-11 (conception refondue après contre-revue)
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/159-edition-file

Conclusion courte: La nouvelle conception réutilise la saga de remise existante
(`send_deliveries`), la reprise, le signal de fin de tour, la migration par
reconstruction de la v4, la table de notices d'auteur, la garde et l'outil de 156.
Elle crée une phase (`held`), une phase (`cancelled`), trois colonnes (`queue_rank`,
`recipient_agent`, `turn_gate`), un réglage de registre, une déclaration de porteur
(`turn_attest`), une table (`queue_edits`), deux trames client et un champ
optionnel de `Register`. Aucun nouveau module, aucune trame vers un porteur, aucun
changement du pont T3. La table `execution_queue` est écartée avec preuves.
Aucune duplication évidente. Dix vérifications ouvertes sont affectées à T002.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 16 |
| Items audites | 16 |
| Reutilisations deja prevues | 13 |
| Existants potentiellement pertinents | 5 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 8 |
| Specs existantes applicables | 8 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Stockage de la file | `send_deliveries` et sa saga | `idempotency.rs:527`, `send_delivery.rs:86-140` | une phase de plus, mêmes colonnes |
| Libération et reprise | `schedule_idempotent_delivery_recovery`, `defer_idempotent_delivery`, `dispatching_deliveries_for_instance` | `daemon.rs:4112-4230`, `send_delivery.rs:379-449` | la libération passe par `dispatching`, donc par la reprise |
| Signal de tour | `TurnState`, `set_turn_state` | `daemon.rs:3473-3491`, `:17812`, `wrapper.rs:5403`, `:6103-6224` | même trame ; le contenu devient véridique et réattesté (`wrapper.rs:6224` corrigé) |
| Filet contre un signal perdu | `release_stale_busy_turns` | `daemon.rs:3496-3520` | déclencheur de libération |
| Échéance de tour | `stamp_turn_deadline_for_delivery` | `daemon.rs:7320-7345` | appliquée à la libération (règle de la session 112) |
| Migration | reconstruction de table de la v4 | `idempotency.rs:709-806` | même méthode, avec protection des liens |
| Précontrôle | `MAX_IDEMPOTENCY_SCHEMA_VERSION` | `store_schema.rs:4` | relevé à 7 dans la même modification |
| Résultat d'envoi après retrait | `PublicResult::Rejected` | `idempotency.rs:89-101` | précédent `Orphaned` (`send_delivery.rs:585-665`) |
| Prévenance de l'auteur | `orphan_emitter_notices` | `idempotency.rs:835-841`, `send_delivery.rs:667-697` | même table, même vidage |
| Création de table | `CREATE TABLE IF NOT EXISTS` | `idempotency.rs:835` | `queue_edits` sans nouvelle version |
| Réaffectation, orphelinage | `reassign_dispatching_deliveries`, `orphan_dispatching_for_instance` | `send_delivery.rs:214-245`, `:585-665` | clause de phase étendue à `held` |
| Plafond de file | `queue_capacity` du type | `registry.rs:67` | refus `queue_full` à l'admission |
| Droits et outil | `guidance.rs`, `bridget_queue` (156) | 156 T005, T026 | trois actions de plus |
| Points d'arrêt de test | `crate::test_sync::checkpoint` | `daemon.rs:8654-8656` | tableau de pannes |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| File durable | `execution_queue`, `take_next_submission` | `execution_store.rs:456-500`, `:1555` | Écartée : aucun consommateur de production, autre magasin que la saga, pas d'échéance ni d'instance destinataire, autre contrat (`work_submissions`). Dette notée |
| File du pont T3 | `worker.queue` (64), regroupement | `t3code.rs:44`, `:50-53`, `:3150` | Hors lot (159c) : pas de `TurnState`, regroupement propre |
| Retrait chez le porteur | `cancel_delivery`, `CancelDelivery`, `report_discard` | `codex_app_server.rs:1081-1093`, `protocol.rs:4147-4151`, `t3code.rs:1855-1880` | Abandonnés : annulent le tour actif ou rendent `indeterminate` |
| Reçu durable du porteur | `ReceiptStore` | `receipt_store.rs:123-205` | Non modifié : plus de tombe |
| File de T3 | `queued-run.*` | T3 `handlers.ts:262-293` | Option 159c, non engagée |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | R13 |
| `~/.speckit/constitution.md` XIX | minimalisme | deux lots, réglage fermé, `edit` et T3 reportés |
| `~/.speckit/constitution.md` XXII | mesurer avant et après | latence en T001 et T009 |
| `docs/decisions/007-socle-idempotence.md` | rejeu explicite, horizon | `queue_edits` de 7 jours |
| `docs/decisions/011-approbation-exposee-puis-second-facteur.md` | borne : agent par ses outils | cadre de menace repris de 156 |
| `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` | garde unique, reprise des remises acceptées | `promote` refusé en pause, `cancel` et `reorder` admis |
| `docs/decisions/044-relai-sous-agent-sous-autorite-parent.md` | preuve enfant limitée | refusée |
| mémoire `bridget-sessions-127-132` | tester `t3code`, `mcp::`, transport pour toute trame | T001, T026 |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 156 | garde commune, origine fabriquée, `bridget_queue` | socle direct |
| 150 | refus nommés, pause, migration de rétention | patron suivi |
| 111-114, 129 (pont T3) | file 64, lots, remise en cours de tour | inchangés, hors lot |
| 112 | budget d'exécution à la poussée réelle | échéance posée à la libération |
| 099/138 (envois) | garde de projet | appliquée avant la rétention |
| 087 (pause) | reprise des remises acceptées | `cancel` et `reorder` ne créent pas de travail |
| 148-149 | tâche native | mission native jamais retenue |
| 133 (ADR 044) | preuve enfant | refusée |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "TurnState"` | `t3code.rs` | aucune occurrence : le pont ne signale pas le tour |
| `rg "TurnState"` | `wrapper.rs`, `daemon.rs` | émis par les porteurs gérés, consommé par `set_turn_state` |
| `rg "take_next_submission"` | `crates/` | tests seulement |
| `rg "Store::open\|IdempotencyStore::open\|ExecutionStore::open"` | `daemon.rs:3058-3071` | quatre connexions sur le même fichier |
| `rg "REFERENCES send_deliveries"` | `crates/` | un enfant : `send_delivery_execution_links` |
| `rg "MAX_IDEMPOTENCY_SCHEMA_VERSION"` | `store_schema.rs` | 6 ; échec fermé au-delà |
| `rg "BATCH_BOUND"` | `t3code.rs` | regroupement propre au pont |
| `rg "DEFAULT_NOTIFY_TIMEOUT_SECS\|TIMEOUT_GRACE_PERIOD"` | `registry.rs`, `daemon.rs` | 2700 s + 30 s |
| lecture T3 `tools.ts`, `handlers.ts` | T3 `origin/main` | `queued-run.*`, `beforeRunId`, `targetRunId` |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Où vit la file | base du daemon, phase `held` | une transaction, plus de saga d'édition | 2026-10-11 |
| `execution_queue` | ne pas réutiliser | R14 | 2026-10-11 |
| Fils T3 | hors lot | pas de signal, regroupement | 2026-10-11 |
| Réglage par type d'agent | creer nouveau, fermé par défaut | mesurer avant d'étendre | 2026-10-11 |
| Table `queue_edits` | creer nouveau | reçu complet et audit, même transaction que l'effet | 2026-10-11 |
| Promotion | libération immédiate + marqueur | pas de perte, pas de saga de renvoi | 2026-10-11 |
| `edit` | ne pas faire | enveloppe idempotente et attribution | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication évidente non arbitrée
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les règles projet applicables ont été lues
- [x] Les specs existantes proches ont été vérifiées
- [x] Le plan.md a été refactoré ou les divergences sont justifiées
