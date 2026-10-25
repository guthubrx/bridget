# Tâches - Session156 guidage, interruption et lecture de file

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R13), `data-model.md`,
`contracts/guidance156.md`, `reuse-audit.md`, `adversarial-review-codex.md`.
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).
**Prérequis** : livraison155 achevée ; revalidation de cette base par T002. Acquis150/152/153/154/155 conservés ; aucune implémentation156 commencée ici.

Règle : une tâche n'est cochée qu'après son test vert. Les tâches « Rouge
d'abord » sont écrites et vues en échec avant le code. Exécution séquentielle
des phases (fichiers communs : `daemon.rs`, `mcp.rs`).

## Phase 1 - Préparation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/tasks.md#extrait-1).

## Phase 2 - Fondations (bloquantes)

- [x] T003 [P] Rouge d'abord puis code dans `crates/bridget-transport/src/protocol.rs` : `ClientCapability::QueueListV1`, `WrapperToDaemon::QueueList`, `DaemonToWrapper::QueueListResult`, `ExecutionControlRefusal::NotPermitted` ; champs optionnels mid_turn_ok sur DeliverIdempotent ET Deliver historique et interrupt_active sur CancelDelivery ; tests : trames historiques sans champs ajoutés byte-identiques, nouvelles valeurs aller-retour ; classer les trames dans la matrice exhaustive de `crates/bridget-daemon/src/daemon.rs` (~`:14541-14610`, sans `_`)
- [x] T004 [P] Rouge d'abord puis code dans `crates/bridget-daemon/src/referent_control.rs` : `AutonomousEffect::AgentGuidance` et son entrée dans `admit_autonomous_effect` ; test `guidance156_pause_garde_table`
- [x] T005 [P] Rouge d'abord : dans le nouveau `crates/bridget-daemon/src/guidance.rs`, tests `guidance156_matrix_*` (humain, parent par lien, parent natif, pair même projet, autre projet, soi-même, non attesté, preuve enfant, `InternalPause`) ; puis `authorize(&GuidanceFacts) -> Result<(), GuidanceRefusal>` pure ; déclarer le module dans `crates/bridget-daemon/src/lib.rs` ; annoter la complexité (O(1))
- [x] T006 [P] Dans `crates/bridget-daemon/src/idempotency.rs` et `idempotency/send_delivery.rs` (tests dans `idempotency/tests.rs`) : table `guidance_events` et ses deux index, `record_guidance_event`, `count_interrupts_target(target, since) -> (count, third_latest_at)`, `count_interrupts_pair(actor, target, since) -> (count, last_at)` (comptent `outcome='admitted'` des acteurs agents ; dates issues de la même table, troisième événement cible le plus récent et dernier événement du couple), purge des événements de plus de 24 h appelée par `run_maintenance` ; index `idx_send_deliveries_recipient_phase` ; `queued_deliveries_for_recipient(instance_id, now, limit)` (requête de `data-model.md`, tri `rowid`) ; tests : comptages et dates exacts aux bornes 30 s et 10 min (horloge injectée ; COUNT inclusif, garde temporelle admet last+30 ou third+600 <= now, retry_after exact sans compteur seul ; coût O(log n + k), k entrées candidates parcourues), la table n'est jamais lue pour un rejeu, lecture ordonnée après une réaffectation, base 150 migrée sans perte

## Phase 3 - US1 Choisir l'intention à l'envoi (P1)

**But** : `bridget_send` et la CLI posent une intention demandée, sans changer l'existant.
**Test indépendant** : filtres `guidance156_mcp`, `guidance156_cli`, `guidance156_send`, `guidance156_t3`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/tasks.md#extrait-2).

## Phase 4 - US2 Droits et garde-fous sur tous les chemins (P1)

**But** : seuls l'humain attaché et le parent guident, sur tous les chemins atteignables par les outils ; pas de boucle ; pause et mission respectées. Les trames brutes sont testées en défense en profondeur (spec, « Cadre de menace »).
**Test indépendant** : filtres `guidance156_guard`, `guidance156_human`, `guidance156_control`, `guidance156_rate`, `guidance156_pause`, `guidance156_mission`, `guidance156_midturn`, `guidance156_cancel`, `guidance156_combo`.
**Dépend de** : T002, T005, T006, T011.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/tasks.md#extrait-3).

## Phase 5 - US3 Voir la file d'un agent (P2)

**But** : `bridget_queue` lit la file, ne modifie rien.
**Test indépendant** : filtre `guidance156_queue`.
**Dépend de** : T003, T006, Phase 4.

- [x] T024 [US3] Rouge d'abord dans `crates/bridget-daemon/src/daemon.rs` (tests) : `guidance156_queue_list_ordre_rowid`, `guidance156_queue_list_apres_reaffectation_meme_ordre`, `guidance156_queue_list_filtre_par_droits` (destinataire, parent, humain attaché voient tout ; autre : ses messages), `guidance156_queue_list_borne_200`, `guidance156_queue_list_ne_modifie_rien` (base identique avant/après), `guidance156_queue_negociation_requise`, `guidance156_queue_preuve_enfant_refusee`, `guidance156_queue_bound_t3_64_sinon_null`. Exception de montage : l’oracle preuve enfant est exercé sur la vraie façade MCP de T026, avant connexion ; QueueList sur le fil ne porte pas ChildProof. Les sept autres oracles restent côté daemon ; aucun nom ni scénario ajouté
- [x] T025 [US3] Dans `daemon.rs` : traitement de `QueueList` (capacité `QueueListV1`, résolution de la cible, lecture de T006, filtre par droits, `preview` de 120 caractères, annotation de complexité O(log n + s) pour le parcours avec jointures indexées en plus, s lignes dispatching examinées dont expirées ; LIMIT 200 borne seulement la sortie) ; T024 passe
- [x] T026 [US3] Outil `bridget_queue` dans `crates/bridget-daemon/src/mcp.rs` (schéma à un argument `target`, identité, négociation `QueueListV1`, résolution de cible comme `bridget_send`, preuve enfant refusée) et `bridget queue --target <cible>` dans `crates/bridget-daemon/src/cli.rs` ; tests `guidance156_mcp_queue_schema`, `guidance156_cli_queue_parse` ; mettre à jour le test de la liste d'outils

## Phase 6 - US4 Compatibilité et finitions

- [x] T027 [P] Documenter : `docs/reference-communication.md` (remplacer la phrase « Cette règle n'annonce aucune option CLI de contrôle nouvelle », ~`:175-180`, par l'intention, la matrice de droits, les refus, la provenance validée, le cadre de menace de la spec, le guidage implicite réservé à l'humain, au parent et à ses enfants, `bridget_queue`, et le rôle de `bridget_cancel`), `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md` Documenter aussi provenance UI/système attestée, Deliver historique, absence de repli interrupt agent et limites de lecture QueueList.
- [x] T028 [P] Écrire l'ADR `docs/decisions/156-guidage-et-lecture-de-file.md` (contexte, cadre de menace et limites hors modèle avec leurs preuves, failles fermées, intention demandée, pas de promesse d'effet, refus de `auto`, report de `cancel`/`reorder`/`promote` avec prérequis : réservation durable avant émission, accusé durable, réconciliation avant relivraison, phase d'annulation, saga de promotion ; statut selon les réponses aux 3 questions)
- [x] T029 [US4] Relancer les filtres de T001 et comparer à `validation/baseline.md` : aucun test ne passe de vert à rouge, sauf les changements visibles acceptés (question 3a, tests de la session 129 réécrits en T021 et fermeture Cursor acceptée par root : steer refusé, busy attend même flagtrue) ; consigner dans `specs/156-guidage-file/validation/tests-156.md` ; test d'égalité d'octets d'un `bridget_send` sans `intent` avant/après
- [x] T030 Lancer la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; rejouer seuls les tests sensibles à la charge avant de conclure ; consigner les compteurs dans `validation/tests-156.md`
- [x] T031 Relire le diff complet (Articles XIX/XX : lignes suppressibles, abstractions, chemins d'erreur), écrire `specs/156-guidage-file/implementation.md` et enregistrer `git diff` dans `specs/156-guidage-file/validation/diff-156.patch`

## Dépendances

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/tasks.md#extrait-4).

## Exemples de parallélisme

- T003, T004, T005, T006 : protocole, pause, garde pure, stockage.
- T027, T028 : documentation seule.

## Stratégie

MVP = Phases 1 à 4 : l'intention existe, la garde couvre tous les chemins
atteignables par les outils (envoi, contrôle, guidage implicite, annulation),
la provenance est validée par le daemon, l'anti-boucle et la pause tiennent. Puis
la lecture de file (Phase 5), puis finitions.

## Charge future (Article XX)

- Réduisent la charge : T015/T017/T021/T022 (une garde, quatre chemins, quatre
  contournements fermés), T012 (booléen remplacé par une valeur, plus de repli
  interruptif caché), plus de réécriture d'intention selon l'état de la cible.
- Ajoutent de la complexité justifiée : T005 (module pur testable par table),
  T006 (table d'audit et de rythme), T015 (neutralisation d'origine), T021
  (champ `mid_turn_ok`, car le pont ne connaît pas la parenté), T022 (méthode
  `cancel_queued`, car `cancel_delivery` mélange retrait et interruption).
- Écartés pour rester minimal : `cancel`/`reorder`/`promote` de file, mode
  `auto`, `edit`, trames daemon vers wrapper, preuve humaine indépendante contre
  un processus hostile (hors modèle, ADR 011).

## Convergence

- [x] T032 [Convergence] Satisfaire FR-016 pour les intentions valides refusées avant la garde par le projet (SendIdempotent, Send, ControlExecution et CancelRequest) et après autorisation par la persistance. Réutiliser guidance_events et la trace existante ; garder canon/réservation/rejeu dans leur ordre, zéro admission/consommation de quota/émission sur refus. Étendre seulement les branches déjà prévues de guidance156_rate_refus_non_compte pour affirmer la trace refused correspondante et le rejeu inchangé, sans scénario nommé supplémentaire. ROOT vérifie le gate réemploi/ArticleXIX si un petit helper interne partagé est nécessaire. Aucun outil/service/type public ni changement d’autorité.
