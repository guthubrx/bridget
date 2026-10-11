# Rapport d'analyse - Session153 rondes multiples et contexte transmis

Date : 2026-10-11. Analyse en lecture seule des artefacts `spec.md`, `plan.md`,
`research.md`, `data-model.md`, `contracts/rounds153.md`, `tasks.md`,
`reuse-audit.md`, `checklists/requirements.md`. Aucun code n'a été modifié.

## Couverture

| Élément | Résultat |
|---|---|
| Exigences FR-001 à FR-033 | 33 sur 33 reliées à au moins une tâche |
| Critères SC-001 à SC-017 | 17 sur 17 reliés à un test ou à la recette (SC-002 : recette réelle T024 + test `child_instance` T013/T019) |
| Scénarios US1 à US5 | 5 sur 5 ont une phase de tâches et un filtre de test |
| Tâches sans exigence | 0 (T001, T021, T025 à T027 sont des tâches de clôture) |
| Tests rouges d'abord | T004, T005, T008, T013 précèdent leur code |

Table FR vers tâches : FR-001 à FR-006 : T005-T007. FR-007 à FR-014 : T008-T012.
FR-015 à FR-022 : T004, T013-T016. FR-023, FR-024 : T006, T009, T018. FR-025 :
T002, T003, T021. FR-026 : T012, T029. FR-027 : T004, T020. FR-028 : T017, T022.
FR-029 : T004, T029. FR-030 : T028. FR-031 : T014. FR-032 : T004, T015.
FR-033 : T008, T010, T011.

## Cohérence entre artefacts

| Point contrôlé | Verdict |
|---|---|
| Nombre de refus : 7 dans spec (FR-003, FR-007, FR-019), contrat et plan | cohérent |
| Nombre de rondes : 8 dans spec, contrat, data-model, tâches | cohérent |
| Bornes de garde 60 à 7 200 dans spec, contrat, catalogue (T009) | cohérent |
| Ordre des refus : plan et contrat renvoient au plan | cohérent |
| Aucun nouvel état (R2) et neuf états Lineage | cohérent |
| Identifiant de résultat par ronde (R4, data-model, T016) | cohérent |
| Numéro des tâches dans la fiche : 29 (T028 et T029 ajoutées après contre-revue, placées dans leur phase ; T030 supprimée après réalignement sur 151 révisée) | cohérent |
| Table nouvelle (missions passées) : plan, data-model, reuse-audit, tâches T003/T029 | cohérent |
| Format `tasks.md` : identifiant, marqueur `[P]`, `[USn]`, chemin | conforme |
| Statuts et priorités SpecKit valides | conformes |
| Tiret cadratin | absent |

## Constats

| # | Sévérité | Constat | Action |
|---|---|---|---|
| A1 | Moyenne | SPEC-150 n'est pas livrée dans le checkout lu. Les prédicats de comptage et de purge, `AutonomousEffect::NativeDelegation` et la lecture de phase `acked` sont cités par spec, pas par ligne de code. | T001(b) relit le code de 150 avant T002. T012 et T016 en dépendent. |
| A2 | Moyenne | Le lecteur T3 n'est pas dans ce dépôt. Une tâche qui passe de `result_available` à `mission_pending` n'est pas vérifiable côté T3. | Mutation visible normale (relecture complète) ; test T020 côté Bridget ; consigné « non vérifié » côté T3. |
| A3 | Moyenne | SPEC-151 et SPEC-152 restent en évolution. `prepare_restart` est réécrit par 152 ; 151 ajoute `work_state` et peut ajouter des champs par ronde. | T001 liste les points ; invariant I5 ; T011(d) et T018 s'adaptent au code livré. |
| A4 | Basse | Le délai d'inactivité n'est pas gelé par une pause : 152 ne gèle que les tâches non terminales (`suspended_at`). | Assumé et documenté (R11). Aucun résultat perdu. |
| A5 | Basse | Une attente 151 lancée sur une tâche inactive rend aussitôt le résultat de la ronde précédente. | `round` ajouté aux vues (T018) ; sémantique écrite dans la documentation (T022). |
| A6 | Basse | Le résultat de la ronde N est remplacé par celui de la ronde N+1 dans la tâche. Le résultat N reste dans le message remis au parent seulement. | Assumé. La remise de N reste protégée de la purge par `unacked_rounds` (FR-029). |
| A7 | Basse | Corps de mission jusqu'à 80 Kio contre 120 000 caractères de plafond injecté. | Test de borne T005. |
| A8 | Info | Les noms `ProjectRound` et `Continuation` (garde de pause) ressemblent aux rondes de tâche. | Différence notée dans `reuse-audit.md`, `research.md` R11 et l'ADR (T023). |
| A10 | Moyenne | Le recalcul de la politique de l'enfant (`child_policy`) puis l'égalité avec la politique figée peut produire de faux refus si le JSON contient des champs variables. | T014 : test « ronde immédiate sans changement admise » ; repli : comparer les seuls champs de droits. |
| A12 | Basse | Réalignement sur 151 révisée : l'attente est une lecture pure, `tick` seul écrit la remise. `continuable` est faux jusqu'à 1 s après la fin de ronde. | Refus réessayable `result_delivery_pending` (spec, contrat, T013, T015, T022) ; FR-033, I6 partagé et T030 supprimés ; `mark_result_sent` retiré (un seul appelant, Art. XIX) : la pose de la garde est locale à `tick`. |
| A13 | Moyenne | Contre-revue ronde 2, point 1 : l'heure d'admission peut être périmée après les revérifications de sources. | `begin_round` relit l'heure (horloge injectable) ; I7, FR-032, T004, T015, SC-017. |
| A14 | Moyenne | Contre-revue ronde 2, point 2 : une annulation reçue avant l'envoi du résultat n'avait aucun effet, puis la garde était posée. | `guard_released` durable ; FR-012, FR-033, I8, T008, T010, T011, SC-016. |
| A15 | Basse | Contre-revue ronde 2, point 3 : la posture effective à la place de la posture demandée change la politique Codex. | Recalcul avec la posture demandée ; matrice de tests posture, parents natifs, conversions (T014, SC-017). Si l'égalité reste instable, arbitrage utilisateur, pas d'affaiblissement. |
| A11 | Basse | T028 et T029 ne sont pas dans l'ordre numérique du fichier. | Voulu : numéros stables après contre-revue ; l'ordre d'exécution suit les phases (note dans Dépendances). |
| A9 | Info | `human_cancel` sur une tâche inactive lève la garde sans mutation visible. Le contrat 149 dit « une tâche terminale rend son état courant ». | Compatible : le `status` ne change pas. Ajouter une phrase au contrat dans l'ADR 153. |

Aucun constat bloquant. Aucune duplication non arbitrée.

## Questions ouvertes (3, avec défaut)

1. Bornes de la garde : 60 à 7 200 s, sans défaut.
2. Plafond de rondes : 8.
3. Brief : dossier de passation seul ; pas d'extrait d'historique du parent.

## Verdict

Prête pour l'implémentation, sous réserve de T001 (relecture du code de 150,
151 et 152). Si T001 révèle un écart avec `plan.md`, corriger `plan.md` avant T002.
