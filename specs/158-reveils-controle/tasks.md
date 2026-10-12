# Tâches - Session158a vue globale, arrêt global, portées MCP

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R8), `data-model.md`,
`contracts/control158.md`, `reuse-audit.md`, `adversarial-review-codex.md`.
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).
**Prérequis** : livraison157 terminée et vérifiée par ROOT, avec chemin du reçu et empreintes source ; aucun SHA livré inventé. Relire chaque fonction citée sur cette base, puis T001 établit la baseline158 fraîche.
**Hors 158a** : budget de tours (question 3) ; partie 158b (spec seule).

Règle : une tâche n'est cochée qu'après son test vert. Les tests marqués
« rouge d'abord » sont écrits et vus en échec avant le code.

## Phase 1 - Préparation

- [x] T001 Établir la référence de non-régression : lancer les filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087` et les tests de `mcp.rs` avec la recette, consigner PASS/FAIL/ignorés dans `specs/158-reveils-controle/validation/baseline.md`

## Phase 2 - Fondations (bloquantes)

- [x] T002 [P] Dans `crates/bridget-transport/src/protocol.rs` : ajouter les variantes `DelegationOverviewRead`, `DelegationOverview`, `DelegationHaltSet`, `DelegationHaltState`, `DelegationCancel` et `DelegationControlRejected` avec le refus fermé de `contracts/control158.md` ; ajouter aussi `NativeDelegationRequest::CallerScope {}` interne (résultat booléen explicite, aucun outil MCP) ; tests : aller-retour serde de chaque variante ; `Delegate` reste inchangé (mêmes octets qu'avant)
- [x] T003 [P] Dans `crates/bridget-daemon/src/delegation.rs` : table `native_delegation_halt` (ligne unique) et table `native_delegation_halt_commands` (reçus, 256 au plus, plus ancien purgé) ; lecture ; callback `FnOnce` de snapshot des racines/count évalué seulement à la première application avant le commit ; pose idempotente par `command_id` : un `command_id` connu rend la réponse ENREGISTRÉE, marquée rejouée, sans réécriture ni parcours d'annulation ; l'état courant reste inchangé même s'il a changé depuis ; tests `control158_halt_store_pose_levee`, `control158_halt_store_a_b_rejeu_a`, `control158_halt_store_a_b_rejeu_a_apres_redemarrage`, `control158_halt_store_recu_porte_la_reponse`, `control158_halt_store_recus_bornes`, `control158_halt_store_motif_borne`
- [x] T004 [P] Dans `crates/bridget-daemon/src/referent_control.rs` : fonction pure `admit_native_delegation(control, halt)` qui compose pause et arrêt dans la garde de l'ADR 027 (arrêt prioritaire : motif `delegations_halted`, sinon `pause`, sinon admis) ; réemployer `AutonomousEffect::NativeDelegation`, déjà présent sur la source157 préparée, et vérifier sa présence sur la base livrée ; tests table de vérité `control158_halt_admission_4_combinaisons`
- [x] T005 [P] Créer `crates/bridget-daemon/src/mcp_scopes.rs` (déclaré dans `lib.rs`) : types `Scope`, `ToolScope { tool, base, read_actions, own_state_actions }` et `scope_of(tool, arguments)` rendant `None` pour un outil inconnu ; table vide pour l'instant ; annoter la complexité (O(1) par recherche, table bornée)

## Phase 3 - US1 Vue globale sans T3 (P1)

**Test indépendant** : filtre `control158_overview`.

- [x] T006 [US1] Rouge d'abord : tests `control158_overview_trois_proprietaires` (10 tâches, 3 propriétaires, ordre récent d'abord), `control158_overview_filtre_actif`, `control158_overview_limite_100`, `control158_overview_sans_corps_ni_resultat`
- [x] T007 [US1] Implémenter dans `crates/bridget-daemon/src/delegation.rs` `overview_page(state_filter, limit)` : parcours borné (n ≤ 4096), projection de `data-model.md` ; annoter `// Complexité : O(n log n) - n = tâches conservées, borné à 4096` ; T006 passe
- [x] T008 [US1] Dans `crates/bridget-daemon/src/daemon.rs` : traiter `DelegationOverviewRead` (négociation `ControlStateV1` puis `control_client_actor` ; requête correctement formée : `negotiation_required`, puis `capability_not_negotiated`, sinon `human_actor_required` selon la première garde en échec)  ; `DelegationOverview` porte aussi since/by/reason Option pour status sans autre variante ; négociation/capacité via ClientRejected, autres refus via DelegationControlRejected ; et `DelegationCancel` (acteur humain, `human_cancel` avec la racine de la tâche, puis `native_delegation::tick`) ; tests `control158_overview_client_agent_negocie_refuse`, `control158_overview_client_non_negocie_refuse`, `control158_overview_cancel_cascade`
- [x] T009 [US1] Dans `crates/bridget-daemon/src/cli.rs` : `control delegations [list] [--state] [--limit] [--json]` et `control delegations cancel <task_id>` (terminal interactif pour `cancel`), usage de `cmd_control` ; tests d'analyse des arguments (hors bornes, option inconnue, UUID invalide → code 2)

## Phase 4 - US2 Arrêt global des délégations (P1)

**Test indépendant** : filtre `control158_halt`. **Dépend de** : T003, T004, T008.

- [x] T010 [US2] Rouge d'abord : dans `crates/bridget-daemon/src/daemon/native_delegation.rs`, test `control158_halt_refuse_nouvelle_cle` (délégation admise pendant l'arrêt sur le code actuel)
- [x] T011 [US2] Dans les admissions Delegate et Continue de `native_delegation.rs` : remplacer l'appel de garde de pause par `admit_native_delegation` (T004), après le contrôle de rejeu et avant `parent_fact` ; refus `delegations_halted` sans écriture ; exposer `delegations_halted` dans le catalogue ; tests `control158_halt_rejeu_cle_admise_rend_la_tache`, `control158_halt_cle_libre_apres_levee`, `control158_halt_statut_et_annulation_disponibles`, `control158_halt_et_pause_4_combinaisons_a_l_admission`  ; étendre les quatre combinaisons existantes à Continue sans nouveau nom ; demandes d'enfant ouvertes et approbations restent pause seule, missions queued/en cours inchangées hors `cancel_active` ; les tests `native150_pause_*` restent verts
- [x] T012 [US2] Dans `crates/bridget-daemon/src/daemon.rs` : traiter `DelegationHaltSet` (gardes de négociation/capacité puis acteur humain attesté selon le contrat, sinon refus fermé correspondant ; `command_id` via T003 ; `cancel_active` refusé si `halted = false`) ; avec `cancel_active`, `human_cancel` une fois par arbre actif présent à cet instant puis `tick`, SEULEMENT à la première application (reçu et état dans une transaction ; un `command_id` connu rend la réponse enregistrée sans aucun parcours d'annulation) ; choisir les tâches actives sans ancêtre actif, figer racines/count avant commit via le callback T003 seulement sur première application et garder st verrouillé jusqu'à `human_cancel` ; `cancelled` compte les arbres visés, pas des arrêts physiques confirmés ; journaliser (`info!` structuré : auteur, motif, nombre d'arbres visés) ; tests `control158_halt_exige_acteur_humain`, `control158_halt_cancel_active_annule_les_arbres`, `control158_halt_sans_cancel_active_ne_touche_rien`, `control158_halt_commande_rejouee`, `control158_halt_a_b_c_rejeu_a_ne_touche_pas_c` (A arrêt + annulation, B reprise, C nouvelle tâche, rejeu de A : C reste active), `control158_halt_a_b_c_rejeu_a_apres_redemarrage`
- [x] T013 [US2] Dans `crates/bridget-daemon/src/cli.rs` : `control halt-delegations [--reason] [--cancel-active]`, `control resume-delegations`, ligne d'arrêt dans `control status` ; terminal interactif obligatoire ; test de surface `control158_halt_aucun_outil_mcp`

## Phase 5 - US3 Portées MCP et plafond (P1)

**Test indépendant** : filtre `mcp158_`. **Dépend de** : T005 et de la livraison des outils de 151 à 157 dans `tools()`.

- [x] T014 [US3] Rouge d'abord : `mcp158_scopes_couvrent_tous_les_outils` (échoue : la table est vide)
- [x] T015 [US3] Remplir `TOOL_SCOPES` selon `contracts/control158.md`, avec classement par action de `bridget_thread` (lecture : `list`, `show`, `history` ; état propre : `read`, `ack` ; écriture : `create`, `add_members`, `post`, `close`), `bridget_events`, `bridget_handoff`, `bridget_journal` (base `sends`, abaissé à `reads` sans `to`), `bridget_ledger` ; `scope_of` lit `action` et `to` (journal : absent/null Reads, toute autre valeur même vide Sends) ; T014 passe
- [x] T016 [US3] Tests exhaustifs dans `mcp_scopes.rs` : chaque nom de `tools()` a une entrée ; chaque entrée désigne un outil de `tools()` ; `readOnlyHint` vaut vrai si et seulement si la portée de base est `reads` (corriger les annotations divergentes ; `bridget_journal` : faux) ; aucune portée « contrôle humain » dans le type ; un outil factice sans entrée fait échouer la vérification (`mcp158_outil_sans_portee_echoue`) ; inclure les outils de 151 à 157
- [x] T017 [US3] Plafond d'appelant : (a) test rouge `mcp158_plafond_enfant_discovery_refuse_ecriture` ; (b) mesurer la recherche de la tâche d'un appelant par nom d'enfant (index `json_extract` ou parcours borné), consigner dans `validation/plafond-recherche.md` ; (c) dans `mcp.rs`, avant le `match` de `execute_tool_at_with_identity`, obtenir le plafond par `CallerScope {}` (identité inscrite côté daemon, lookup UNIQUE child existant, résultat booléen explicite ; panne RPC/SQL ou résultat invalide ferme tout appel ; aucune DB client ni fallback standard), puis appliquer : `read_only` si une tâche native avec `child == identity.name` et sans `cleanup_done` a `effective_posture.or(request.posture) == discovery` ; refuser `capability_denied` toute portée au-dessus ; tests `mcp158_plafond_posture_omise_suit_effective`, `mcp158_plafond_tache_terminale_avant_nettoyage`, `mcp158_plafond_enfant_conserve_entre_rondes`, `mcp158_plafond_enfant_nettoye_retrouve_les_droits`, `mcp158_plafond_parent_garde_tous_les_outils`, `mcp158_plafond_aucun_effet_de_bord_au_refus`
- [x] T018 [US3] Tests d'inventaire du plafond : `mcp158_discovery_lecture_ack_page_suivante` (un fil de 3 pages lu en entier, jamais la même page deux fois), `mcp158_discovery_chaque_action_de_fil` (lecture et état propre permis, écriture refusée), `mcp158_discovery_journal_avec_et_sans_to`, `mcp158_discovery_events_sub_unsub_permis`, `mcp158_discovery_reponse_automatique_au_parent` (la réponse finale automatique part), `mcp158_discovery_lecture_artefact_permise`, `mcp158_discovery_publication_artefact_refusee_sans_effet`, `mcp158_discovery_partage_journal_refuse_sans_effet`, `mcp158_journal_read_only_hint_faux` ; politique FR-029 appliquée telle quelle : aucun élargissement pendant l'implémentation, tout besoin contraire fait l'objet d'une nouvelle spec

## Phase 6 - Compatibilité et finitions

- [x] T019 [P] Documenter la CLI `control delegations`, `halt-delegations`, `resume-delegations`, le refus `delegations_halted` et les portées dans `docs/delegation-native.md`, `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md` ; noter les ruptures voulues de la politique FR-029 : un enfant `discovery` ne poste pas dans un fil, ne partage pas son journal et ne publie pas d'artefact
- [x] T020 [P] Écrire l'ADR `docs/decisions/158-controle-global-portees.md` : décisions R1 à R8, extension de la garde de l'ADR 027 (admission commune pause et arrêt), retrait du budget de tours avec ses raisons et les conditions de reprise, levée de la mention « tokens en 158 » de la spec 150, limites (garantie par CI, pas de compilateur ; borne ADR 011) ; statut Accepté
- [x] T021 Relancer les filtres de T001 et la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; comparer à `validation/baseline.md` ; rejouer seuls les tests sensibles à la charge ; consigner dans `validation/tests-158.md`
- [x] T022 Relire le diff complet (Articles XIX et XX), écrire `specs/158-reveils-controle/implementation.md`, enregistrer `git diff` dans `specs/158-reveils-controle/validation/diff-158.patch`

## Dépendances

- T001 avant tout. T002 à T005 en parallèle, avant les phases 3 à 5.
- Phase 3 avant Phase 4 (T012 utilise le chemin d'annulation de T008).
- Phase 4 touche `native_delegation.rs` : sur la base157 livrée et sans autre édition concurrente.
- Phase 5 indépendante des phases 3 et 4, sauf T016 qui attend les outils de 151 à 157.
- T019 et T020 en parallèle ; T021 et T022 en dernier.

## Exemples de parallélisme

- T002, T003, T004, T005 : fichiers distincts. T019, T020 : documentation seule.

## Stratégie

MVP = Phases 1 à 4 : l'humain voit tout et peut tout arrêter, avec une garde
unique. Puis portées et plafond, puis finitions. Chaque phase laisse le projet
buildable et testable.

## Charge future (Article XX)

- Réduisent la charge : T016 (un outil sans portée échoue en CI), T009 (une vue
  pour tous les parents), T004 (une seule décision d'admission).
- Complexité justifiée : T017 (plafond, comble l'écart de sécurité avec T3),
  T003 (reçus bornés, évitent de remettre un arrêt par un rejeu).
- Non faits volontairement : budget de tours et de tokens (compteur non fiable),
  historique d'audit de l'arrêt, enveloppe de compilation des outils.

## Base158A vérifiée par le principal

Date : 2026-10-12. Branche : session-158-reveils-controle. Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle. Base Git : bd37e927eee0b28f058df9d6a6992e38665a6ad0. Sources157 : 8f21dfd7b553e822fcabe0a0e7eabc6741a8e45b. Reçu canonique157 : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/validation/livraison157.md. SHA256 : 6ac3d08948f39d6999b0a5c425235a70a1ad7d99d433866887abbdf52a86b705. Intégration, push, activation et nettoyage157 vérifiés. Les déclarations de préparation ci-dessus restent historiques ; cette vérification lève leur prérequis de livraison.
