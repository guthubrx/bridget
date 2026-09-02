# Journal d'Implémentation - Reprendre le contrôle, versant Maicie

## Métadonnées
- **Spec** : 087-reprendre-controle
- **Périmètre** : `plugins/maicie/src/**`, `plugins/maicie/tests/**`, ce journal. Aucun fichier de `crates/` touché, aucun commit, aucun push, aucune configuration serveur modifiée, aucun redémarrage.
- **Base** : origin/main `bf89d4c7`
- **Méthode** : édition locale, `rsync` vers `/home/moi/bridget-referent/.worktrees/087-reprendre-controle`, `cargo fmt -p maicie`, compilation sur le cache `/home/moi/bridget-referent/bridget/target-maicie`, tests joués par périmètre déclaré, comptes annoncés (passed/failed), jamais un code retour.
- **Démarré / rédigé** : 2026-09-02

## Conception retenue
- **Garde unique** `maicie::control::admit_autonomous_effect(effect, snapshot)`. Le snapshot est lu **une fois par commande** dans `open_store_with_reconciliation` (`read_control_snapshot`, périmètre `maicie-control-snapshot-v1-read-only`) et posé sur `MaicieStore` (`set_control_snapshot`). Trois états : `Unread` (daemon antérieur sans `control_state_v1`, ou commande sans relève) admet ; `Unknown` (socket injoignable) diffère avec le motif `controle_inconnu` ; `Read{paused, auto_objectives_cap, inbox_open_count}` applique la table de vérité. En pause, seul `OutboxReplay{origin_auto: false}` est admis (une délégation d'origine humaine se rejoue).
- **Motifs de différé** portés par les données : `routine_occurrences.reason` (`pause`, `focus`, `budget`, `controle_inconnu`), `deferred_delegation_dispatch.deferred_reason`, erreur `Conflict` explicite pour la réassignation (le fait reste au guichet et revient à la relève suivante).
- **Dépôts humains durables** : table `human_inbox_outbox` (clé `dedup_key`), déposés à la relève par `reconcile_human_inbox_with_limits` ; décisions relevées puis appliquées une seule fois (`human_decisions_applied`) avant acquittement ; une décision non supportée n'est **pas** acquittée et reste visible.
- **Origine humaine** : `ObjectiveOpeningPermit::human_request` sur le hash des octets canoniques de la requête **sans** `origin` (`canonical_delegate_bytes_without_origin`), consommation unique du `message_id` (`human_origin_consumptions`), aucun passage par `GreffeAuthorizationGate` pour une origine humaine (message du coordinateur, ADR 027). Un `Delegate` sans origine garde le chemin actuel ; `focus` sans origine humaine est refusé (`focus_sans_origine_humaine`).
- **Focus** : `focus_queue` (tête = position 0), `Replace` ferme la tête courante, `Queue` ajoute en file ; la référence `focus:<objective_id>` est injectée dans les octets d'outbox `prepared` non encore tentés (`add_focus_reference_to_pending_outboxes`) pour que la file d'exécution du daemon la lise (`queue_priority_for`, `FOCUS_QUEUE_PRIORITY`) ; champ `references` ajouté à `PublicMessage` (`serde(default)`, omis si vide, donc un daemon antérieur ne voit rien de nouveau).

## Progression

### T009 Clients `ControlStateClient` / `HumanInboxClient` : ✅
- `src/bridget_client.rs` : `ControlStateClient::connect_with_limits_until` (rôle client, hello `[control_state_v1]`, `read_control_state` → `ControlStateReading{state, inbox_open_count}`), `HumanInboxClient::connect_with_limits_until` (rôle service, hello `[human_inbox_v1]`, `deposit_human_inbox`, `fetch_human_decisions(limit)`, `ack_human_decision`) ; refus de capacité ⇒ `CapabilityMissing`, refus de rôle ⇒ `ClientRejected`.
- Tests contre un faux daemon (module `control_inbox_contract_tests`) : `cargo test -p maicie --lib control_inbox_contract` → **3 passed; 0 failed**.
- Propriétés nommées : la lecture négocie sa seule capacité et lit une fois ; une capacité absente est un refus explicite ; dépôt, relève et acquittement passent en rôle service avec la version du contrat sur chaque trame.

### T010 Garde unique et snapshot : ✅
- `src/control.rs` (nouveau) ; `src/main.rs` lit le snapshot après ouverture du store. `cargo test -p maicie --lib control::` → **5 passed; 0 failed** (table de vérité, socket absente ⇒ `Unknown`).

### T015 Routines : ✅
- `src/routines.rs` : garde avant tout choix d'agent ; ordre des motifs : `pause`/`controle_inconnu`, puis `focus`, puis `budget` (`DelegateError::BudgetReached`) avec dépôt unique `budget-reached`.
- `cargo test -p maicie --test contract spec_087` → **4 passed; 0 failed**.

### T016 Réassignation : ✅
- `store.rs::apply_reassignment_fact` refuse par `Conflict("réassignation différée : …")` avant toute réduction ; `spec_087_la_reassignation_est_differee_avant_toute_reduction` vert.

### T017 Relève des outboxes : ✅
- `reconcile.rs` : `ReconcileAction::Differee{objective_id, message_id, motif}` ; les outboxes du focus passent en tête ; en pause, seules les outboxes d'origine humaine sont tentées. `spec_087_la_releve_ne_tente_pas_les_outboxes_automatiques_en_pause` vert.

### T047 Déblocage de dépendance : ✅
- `release_waiting_dependents_on_prerequisite_closure` gardé ; différé consigné dans `deferred_delegation_dispatch.deferred_reason`, rejoué à la relève par `release_ready_dependents`. `spec_087_le_deblocage_de_dependance_attend_la_levee_de_la_pause` vert.

### T021 Migration v24 : ✅
- `SCHEMA_VERSION = 24` ; tables `focus_queue`, `human_origin_consumptions`, `human_decisions_applied`, `human_inbox_outbox`, colonne `deferred_delegation_dispatch.deferred_reason`. Migration sous consentement inchangée. `spec_087_migration_v24_ajoute_focus_consommations_decisions_et_motif` vert (base ramenée à 23 puis rouverte après publication).

### T022 Origine humaine et focus au greffe : ✅
- `guichet.rs` : `DemandeDelegation.origin/focus`, v1 avec origine ou focus refusé, `canonical_delegate_bytes_without_origin`. `greffe_service.rs` : permit humain, consommation unique, `focus_enqueue`, référence focus sur les outboxes. `domain.rs` : `MotifRefusGreffe::OrigineHumaineInvalide` (`human_origin_invalid`).
- `cargo test -p maicie --test human_origin_attestation` → **10 passed; 0 failed**, dont le test croisé `spec_087_le_scelle_transport_et_le_scelle_domaine_sont_identiques` (scellé `bridget_transport::protocol::human_message_content_seal` = `maicie::domain::human_message_content_seal` sur la même trame).
- `spec_087_usage_unique_de_l_attestation_humaine` et `spec_087_file_de_focus_une_tete_et_une_file` verts.

### T023 Instruction de focus : ✅ (déjà couvert)
- Le corps de délégation porte le bloc « IDENTIFIANTS DU MANDAT » depuis SPEC-052 (`domain.rs`) ; mon doublon a été retiré. **Non fait** : mesure git de la base gelée sur `origin/<branche>` pour un focus sans `review_target` (voir Reste à faire).

### T024 Référence de focus (partie Maicie) : ✅
- `add_focus_reference_to_pending_outboxes` ajoute `references: ["focus:<id>"]` sans rien retirer du message. `spec_087_depot_humain_durable_et_reference_focus` vert.

### T025 `maicie status` et `maicie focus` : ✅
- `main.rs` : `ControlStatusOutput` dans `status` (JSON `control` + ligne texte `contrôle=… pause=… plafond_auto=… boîte_ouverte=… focus=… file_focus=… différées=… dispatchs_différés=…`) ; `maicie focus [status|close|queue] [--json]` ; code CLI `budget_reached` ; erreur `HumanOriginInvalid` rendue en usage.

### T027 Focus sans agent disponible : ❌ non fait (voir Reste à faire)

### T028 Producteurs de boîte humaine : ⚠️ partiel
- Fait : `chain_exhausted` (options `cancel`, `ack`), `budget_reached` (`raise_budget`, `ack`). Non faits : `review_verdict_pending`, `activation_approval`.

### T032 Décisions humaines : ⚠️ partiel
- `apply_human_decision` : `ack` ⇒ appliqué ; `cancel` ⇒ annulation de la délégation ; `raise_budget` ⇒ appliqué côté Maicie (le plafond est un réglage daemon, rien à faire ici) ; `reassign:<agent>` ⇒ `Unsupported`, non acquitté, reste visible dans la boîte. `spec_087_decisions_humaines_appliquees_une_seule_fois` vert.

### T033 Clôture d'item : ❌ note seule
- Aucune trame `HumanInboxClose` dans le protocole ; un item ouvert ne peut être fermé par Maicie que par une décision humaine.

### T037 Plafond d'objectifs automatiques : ✅
- `app.rs::delegate` refuse `BudgetReached{cap, open}` pour un permit `AutoGenerated` quand le compte des objectifs automatiques ouverts atteint le plafond ; `count_open_auto_generated_objectives` (O(n) sur les objectifs ouverts, annoté). `spec_087_le_compte_du_budget_suit_les_objectifs_automatiques_ouverts` vert.

## Mutants

Mutant : dans `control.rs`, bras `ControlSnapshot::Read { paused: true, .. }` forcé à `Admission::Admitted` (`_ if true => Admission::Admitted, // MUTANT`), joué sur le serveur puis restauré (`grep -c MUTANT` = 0).

| Suite | Attendu | Ligne brute |
|---|---|---|
| `--test controle_referent_087` | réassignation, relève, dépendance rouges | `test result: FAILED. 6 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s` — rouges : `spec_087_la_reassignation_est_differee_avant_toute_reduction`, `spec_087_la_releve_ne_tente_pas_les_outboxes_automatiques_en_pause`, `spec_087_le_deblocage_de_dependance_attend_la_levee_de_la_pause` |
| `--test contract spec_087` | routine en pause rouge, les trois autres motifs verts (ils ne passent pas par ce bras) | `test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 115 filtered out; finished in 0.06s` — rouge : `routines::spec_087_la_pause_differe_l_ouverture_par_routine_avec_son_motif` |

Chaque garde de pause (routine, réassignation, relève, dépendance) a donc un oracle d'effet qui meurt avec le mutant.

## Gates joués

| Gate | Résultat |
|---|---|
| `cargo fmt -p maicie --check` | propre |
| `cargo test -p maicie --test controle_referent_087` | 9 passed; 0 failed |
| `cargo test -p maicie --test human_origin_attestation` | 10 passed; 0 failed |
| `cargo test -p maicie --test bridget_client_contract` | 13 passed; 0 failed |
| `cargo test -p maicie --test contract` | 114 passed; 5 failed (`profiles::*`, préexistants) |
| `cargo test -p maicie --lib` | 88 passed; 1 failed (`store::coordination_transaction_tests::faute_apres_une_vraie_outbox_…`, préexistant) |
| Suite complète `cargo test -p maicie --no-fail-fast` | non joué jusqu'au bout : la suite complète dépasse 10 minutes (un banc bloque) ; les suites ciblées ci-dessus sont vertes, le rouge `--lib` est le préexistant `agent_retarget_requirements` |
| `cargo clippy -p maicie --all-targets -- -D warnings` | bloqué en amont : `bridget-transport` rougit déjà sur origin/main (`impl` dérivable, taille de variantes), aucune ligne de ce lot |

**Base nue mesurée** (clone frais de `bf89d4c7` dans `/tmp/base-087`, même cache) : `--test contract profiles` → `0 passed; 5 failed`, mêmes cinq noms ; `--lib coordination_transaction_tests` → `2 passed; 1 failed`, même nom. Les six rouges sont donc antérieurs au lot ; `git diff --stat origin/main -- plugins/maicie` ne touche aucun fichier de profils ni de fixture.

**Découverte de méthode** : compiler le clone de base dans `target-maicie` a ensuite fait échouer la compilation du worktree (`unresolved import maicie::control`, `ObservedHumanMessageFrame` introuvable) alors que les sources étaient intactes : le cache a servi les artefacts du clone. Remède : `cargo clean -p maicie -p bridget-transport` (4,7 Gio). Un cache par arbre, jamais deux arbres sur un cache.

## À reporter côté daemon (protocole ou daemon)
1. `GuichetRefusalReason` n'a pas de variante budget : un refus `BudgetReached` remonte au guichet comme `mutation_invalid`. Trame souhaitée : `GuichetRefusalReason::BudgetReached { cap: u32, open: u32 }`.
2. Aucune trame de clôture d'item par le service (`HumanInboxClose { item_id, reason }`) : T033 impossible côté Maicie.
3. `reassign:<agent_id>` n'est pas une décision que Maicie sait appliquer sans un fait de réassignation au guichet ; soit le daemon traduit la décision en fait de réassignation, soit le protocole porte un `HumanInboxDecision` typé. En attendant l'item reste ouvert et non acquitté.
4. Le hash humain est calculé de chaque côté sur la requête canonique sans `origin` ; aucun test croisé daemon/Maicie n'existe pour ces octets (seul le scellé est croisé). À ajouter dans un test d'intégration daemon qui soumet une vraie requête au greffe Maicie.
5. `priority_class="focus"` n'est pas posé par Maicie ; seule la référence `focus:<id>` l'est, conformément au message du coordinateur.

## Reste à faire côté Maicie
- T027 : un focus sans agent disponible doit rester en attente et déposer un item ; aujourd'hui `apply_delegate` refuse comme pour toute délégation sans agent.
- T028 : producteurs `review_verdict_pending` et `activation_approval`.
- T023 : mesure de la base gelée sur `origin/<branche>` quand la requête de focus n'a pas de `review_target`.
- Un test direct de `canonical_delegate_bytes_without_origin` (v2, `origin` retiré, `focus` conservé) et du refus v1 avec origine.

## Résumé
1. Versant Maicie livré sans commit : garde unique `admit_autonomous_effect`, snapshot lu une fois par commande, motifs `pause`/`focus`/`budget`/`controle_inconnu` visibles dans les données.
2. Migration v24 réservée et écrite (quatre tables, une colonne), sous consentement comme avant.
3. Origine humaine : permit `human_request` sur le canonique sans `origin`, usage unique, pas de gate d'autorisation ; scellés transport et domaine prouvés identiques.
4. Focus : file à une tête, `Replace`/`Queue`, référence `focus:<id>` posée sur les outboxes pour la priorité daemon.
5. Boîte humaine : dépôts durables, décisions appliquées une fois puis acquittées, non supportées laissées ouvertes.
6. Tests neufs : 9 (`controle_referent_087`) + 4 (routines) + 1 (scellé croisé) + 3 (clients contre faux daemon) + 5 (garde), tous verts.
7. Mutant « pause admet tout » tue 4 tests, un par garde.
8. Six rouges préexistants, identiques nom pour nom sur la base nue mesurée par clone frais.
9. Non faits : T027, deux producteurs de T028, `reassign` (T032), T033 (trame absente), mesure git du focus (T023).
10. Cinq points à reporter côté daemon, dont deux trames à ajouter au protocole.
