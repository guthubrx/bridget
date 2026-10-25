# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/tasks.md](../../../../../../specs/159-edition-file/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T001 Établir la référence de non-régression avant tout changement : lancer les filtres `spec_087`, `native148`, `native149`, `native150`, `guidance156`, `t3code`, `mcp::`, `send_idempotent`, `idempotency` de `bridget-daemon` et la suite de `bridget-transport` avec la recette ; consigner PASS/FAIL/ignorés dans `specs/159-edition-file/validation/baseline.md` ; mesurer sur un porteur Codex simulé, réglage inexistant, l'intervalle fin de tour vers début du message suivant (référence de SC-004) et le consigner
- [x] T002 [P] Lever les dix vérifications ouvertes de `research.md` (lecture seule, réponses avec `fichier:ligne` dans « Vérifications levées ») : (1) modes de porteurs gérés qui envoient `TurnState`, événements, source de l'état réel de la session (`is_busy`), cadence du battement où attacher la réattestation, types d'agents pouvant déclarer `turn_attest` (`wrapper.rs:4757`, `:5403`, `:6103-6224`) ; (2) période du tick des relances (`daemon.rs:6826`) ; (3) causes d'écart du pont à reproduire (`t3code.rs:1855-1912`) et traitement de `DeliveryIndeterminate` pour `reply=true` (`daemon.rs:16395-16420`) ; (4) un envoi sans intention ne crée aucune exécution liée (`daemon.rs:8828-8850`) ; (5) `IdempotencyStore` peut lire et fermer `tracked_requests` dans sa transaction (repère candidat `send_delivery.rs:89–175`) ; (6) regroupement dans les porteurs gérés (`BatchedDelivery`) ; (7) forme finale de `guidance.rs` et de `bridget_queue list` livrés par 156 ; (8) colonnes de `tracked_requests` (`state`, `deadline_at`, repère candidat `store.rs:170–187`) et lien message vers demande ; (9) contenu de `control_pause_interruptions` et points de levée de la pause (`resume_executions_after_pause`) ; (10) résolution de l'identifiant d'agent à l'admission (`routing_agent_id`) contre `final_agent_id` à l'enregistrement

<a id="extrait-2"></a>

## Extrait 2

- [x] T003 Rouge d'abord puis code dans `crates/bridget-daemon/src/idempotency.rs`, ses tests existants `idempotency/tests.rs`, et `store_schema.rs` : migration v7. Tests `held159_migration_*` : `sans_perte` (copie de la base158B effectivement livrée avec remises dans chaque phase, comptages identiques), `ordre_preserve` (`queue_rank` = ancien `rowid`), `recipient_agent_rempli` (`json_extract(message_bytes,'$.to')`, lignes illisibles signalées), `liens_execution_conserves` (lignes de `send_delivery_execution_links` présentes après, `foreign_key_check` à zéro), `phases_held_et_cancelled_admises`, `turn_gate_defaut_zero`, `rejeu_sans_effet`, `rollback_si_echec`, `fantomes_ecartes_avec_warn`, `precontrole_accepte_7_refuse_8` ; puis : reconstruction de `send_deliveries` (phases `held` et `cancelled`, `queue_rank INTEGER PRIMARY KEY AUTOINCREMENT`, `delivery_id UNIQUE NOT NULL` conservant les cibles FK, `recipient_agent`, `turn_gate`), copie des liens avant `DROP TABLE` et réinsertion après le renommage, index (`idx_send_deliveries_operation`, `idx_send_deliveries_kind_key`, `idx_send_deliveries_recipient_phase` (non unique, étendu à `(recipient_instance_id, phase, queue_rank)`), `idx_send_deliveries_agent_phase`), version 7, `MAX_IDEMPOTENCY_SCHEMA_VERSION = 7` ; test documenté : un binaire antérieur (constante 6) refuse la base v7 ; ne pas désactiver `foreign_keys`
- [x] T004 Rouge d'abord puis code dans `crates/bridget-daemon/src/idempotency/send_delivery.rs` et `daemon.rs` : rang, rattachement par agent, phase `held` dans les opérations existantes. Tests `held159_rank_*` : `monotone_sous_concurrence`, `survit_a_la_reaffectation`, `reprise_dans_l_ordre`, `sans_collision_entre_instances`, `orphelinage_couvre_les_retenus` (notice d'auteur écrite), `reprise_ne_rend_pas_les_retenus` ; `held159_adopt_*` : `adoption_a_l_enregistrement`, `adoption_sans_presence_apres_redemarrage_du_daemon`, `double_redemarrage_nouvelle_instance` (SC-015), `autre_agent_non_touche` ; puis : INSERT omettant `queue_rank`, attribué par SQLite AUTOINCREMENT dans la transaction Immediate de begin_send_delivery_with_guidance avec le mode Held ; séquence durable après purge sans nouveau compteur ni table métier (pas de lien d'exécution) et `recipient_agent` écrit, `dispatching_deliveries_for_instance` triée par rang (remplace `ORDER BY delivery_id`), `reassign_dispatching_deliveries` et `orphan_dispatching_for_instance` étendues à `held`, `adopt_held_for_agent` appelé à l'enregistrement d'un agent (`daemon.rs:6311+`) ; adapter les tests existants qui supposaient l'ordre par `delivery_id`
- [x] T005 Rouge d'abord puis code : réglage et décision de rétention. Tests `held159_hold_*` en table de vérité (`research.md` R4) : réglage fermé, instance sans `turn_attest`, intention présente, origine `Human`, origine `System`, alerte de fil, observation, incident délégué, mission native, fil T3, présence non `busy` sans retenus, présence non `busy` avec retenus, présence `busy`, file pleine (`queue_full` mémorisé) ; puis : champ `held_queue` (`#[serde(default)]`) dans `AgentDefinition` (`crates/bridget-daemon/src/registry.rs`), fonction pure `should_hold(facts)`, appel dans `handle_idempotent_send_with_result_authority` après `prepare_dispatch` et avant la poussée (`daemon.rs:8686+`) : en cas de rétention, insertion `Held`, aucun push, ajout dans `held_ids`, résultat d'admission inchangé ; réglage fermé : aucune différence d'octets ni de phase (`held159_hold_ferme_identique`)

<a id="extrait-3"></a>

## Extrait 3

- [x] T022 [US4] Dans `crates/bridget-daemon/src/mcp.rs` : `bridget_queue` gagne `action` (`cancel`, `reorder`, `promote`), `message_id`, `before_message_id`, `request_id` (obligatoire pour les écritures), négociation `QueueEditV1`, preuve enfant refusée, résultat selon `contracts/edition159.md`, clés `queue` de `bridget_capabilities` ; dans `crates/bridget-daemon/src/cli.rs` : `bridget queue cancel|reorder|promote` ; tests `edit159_mcp_schema`, `edit159_mcp_request_id_obligatoire`, `edit159_mcp_preuve_enfant_refusee`, `edit159_cli_parse` ; mettre à jour les tests qui figent les schémas et la liste d'outils
- [x] T023 Tests de bout en bout du lot B dans `crates/bridget-daemon/tests/edit159_e2e.rs` : scénario complet (retenir, lister, réordonner, retirer, promouvoir, libérer) sur un porteur Codex simulé réglage ouvert ; aucun doublon côté daemon, aucune perte avant libération ; réglage fermé : toutes les actions rendent `not_held`
- [x] T024 [P] Documenter le lot B : `docs/reference-communication.md` (actions, droits, refus, reçu, `queue_fallback`, garanties réduites de la promotion), `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md`
- [x] T025 [P] Étendre pour le lot B l’ADR `docs/decisions/159-file-retenue-et-edition.md` déjà écrite par T012 pour le lot A (contexte, racine commune des 11 défauts, file tenue par le daemon, `execution_queue` écartée avec preuves, découpage, coûts mesurés, signal de tour véridique, réservation de tour, pause et `HeldRelease`, migration v7 à sens unique avec sauvegarde et restauration répétée, promotion et ses garanties réduites, report d'`edit`, option 159c pour T3, statut selon les réponses aux 3 questions)
- [x] T026 Non-régression du lot B : relancer les filtres de T001 et `held159`, comparer à `validation/baseline.md` ; suite complète `cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` ; rejouer seuls les tests sensibles à la charge avant de conclure ; test d'égalité d'octets d'un envoi sans édition avant et après ; consigner dans `validation/tests-159.md`
- [x] T027 Relire le diff complet du lot B (Art. XIX/XX : lignes suppressibles, abstractions, chemins d'erreur, atomicité de chaque transaction, aucune promesse plus forte que les tests), écrire `specs/159-edition-file/implementation-b.md` et enregistrer `git diff` dans `validation/diff-159b.patch`

<a id="extrait-4"></a>

## Extrait 4

MVP = Lot A : signal de tour véridique, file retenue, réglage fermé par défaut,
mesurée, avec tableau de pannes et restauration répétée. Ensuite Lot B : `cancel`
(P1), `promote` (P1, garanties réduites), `reorder` (P2). Les fils T3 restent hors
lot (159c).

<a id="extrait-5"></a>

## Extrait 5

- Réduisent la charge : une seule copie de l'ordre ; édition en une transaction ;
  plus de tombe, de saga d'édition, de réconciliation ni de trame vers le porteur ;
  ordre de reprise déterministe ; refus `queue_full` net à l'admission ; signal de
  tour véridique (profite aussi aux relances et à la présence).
- Ajoutent de la complexité justifiée : T003 (migration à sens unique, liens
  protégés, quatre colonnes), T006 (signal véridique, exigé par la libération),
  T007 (réservation de tour et conditions revérifiées), T008 (parité des causes
  d'écart et suivi mémoire), T020 (marqueur de promotion).
- Écartés pour rester minimal : `edit`, `read`, tombe durable chez le porteur,
  saga d'édition, snapshot du porteur, trames daemon vers porteur, `execution_queue`,
  extension aux fils T3, reprise durable du porteur après `Seen`.
