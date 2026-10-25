# Rapport d'analyse - Session159 (spec, plan, tâches)

Date : 2026-10-11, après les rondes 1 et 2 de la contre-revue
(`adversarial-review-codex.md`). Analyse non destructive de
`specs/159-edition-file/`. Le code de 156 n'est pas encore livré dans le checkout
lu : ses acquis sont supposés présents.

## Verdict

PASS avec 10 vérifications ouvertes (T002) et 3 questions utilisateur à défaut
prudent. 27 tâches (lot A : 13, lot B : 14). Les garanties ont été réduites là où
le code ne les permet pas (après libération, promotion). Aucun test ne doit
affirmer plus que `research.md` R17.

## Exigences d'entrée vers tâches

| Entrée | Exigence | Tâches | Critère |
|---|---|---|---|
| E1 retrait en file seulement | FR-013, FR-014 | T017 | SC-007 |
| E2 état final `cancelled` | FR-014 | T017, T018 | SC-007 |
| E3 pas de saga, reprise inchangée | FR-004, FR-005 | T007, T010 | SC-002 |
| E4 migration valide du `CHECK` | FR-010 | T003, T011 | SC-006 |
| E5 attente contre engagé, rang atomique | FR-003, FR-008, FR-009 | T004, T009 | SC-003, SC-008 |
| E6 reçu complet, résultat final | FR-017 | T014, T017 | SC-011 |

## Couverture exigences vers tâches

| Exigence | Tâches |
|---|---|
| FR-001, FR-002, FR-003 | T005 |
| FR-004, FR-005 | T007 |
| FR-006 | T008 |
| FR-007 | T003, T004 |
| FR-008 | T004 |
| FR-009 | T009 |
| FR-010 | T003, T011 |
| FR-011 | T001, T010 |
| FR-012 | T022 |
| FR-013, FR-014 | T017 |
| FR-015 | T019 |
| FR-016 | T020, T021 |
| FR-017 | T014, T017 |
| FR-018 | T015, T017, T019, T021 |
| FR-019 | T016, T020 |
| FR-020 | T016, T022 |
| FR-021 | T012, T024, T025 |
| FR-022 | T005 |
| FR-023 | T006 |
| FR-024 | T007 |
| FR-025 | T006, T007 |
| FR-026 | T007 |

26 exigences sur 26 couvertes.

| Critère | Tâches de preuve |
|---|---|
| SC-001 | T005, T013, T026 |
| SC-002 | T004, T007, T010 |
| SC-003 | T004, T010 |
| SC-004 | T001, T010 |
| SC-005 | T006, T007 |
| SC-006 | T003, T011 |
| SC-007 | T017, T018 |
| SC-008 | T019 |
| SC-009 | T020, T021 |
| SC-010 | T015, T017, T019, T021 |
| SC-011 | T014, T017 |
| SC-012 | T007 |
| SC-013 | T007 |
| SC-014 | T007, T021 |
| SC-015 | T004, T010 |

15 critères sur 15 couverts.

## Sort des points de la contre-revue

| Ronde | Points | Sort |
|---|---|---|
| 1 | 1, 2, 3, 5, 10 | sans objet par la conception (confirmé par Codex ronde 2) |
| 1 | 4, 6, 7, 8, 9, 11 | fermés par tâche, puis affinés par la ronde 2 (voir ci-dessous) |
| 2 | R2-1 réservation de tour | T006, T007, SC-012 |
| 2 | R2-2 signal perdu | T006, T007, SC-005 corrigé |
| 2 | R2-3 double redémarrage | T003, T004, SC-015 |
| 2 | R2-4 zéro perte après libération | promesse réduite, T010 (point d'arrêt `Seen`) |
| 2 | R2-5 promotion sans rejet garanti | promesse réduite, T020, T021 |
| 2 | R2-6 clôture et échéance en transaction | T007, T021, SC-014 |
| 2 | R2-7 suivi mémoire | T008, T017 |
| 2 | R2-8 pause | T007, SC-013 |
| 2 | R2-9 restauration | T011 |
| 2 | R2-10 deux ordres de validation | T019, SC-008 |

## Cohérence entre artefacts

- Phases, refus (`not_held`, `no_active_turn`, `steer_unavailable`, `queue_full`) et états identiques dans `spec.md`, `contracts/edition159.md`, `data-model.md` et `tasks.md`.
- Les trois colonnes de la v7 (`queue_rank`, `recipient_agent`, `turn_gate`) figurent dans le modèle, le plan, R5, R6 et T003.
- `turn_attest` conditionne la rétention (spec FR-002, contrat, modèle, T005, T006). `queue_fallback` conditionne seulement la promotion.
- Les promesses sont alignées : SC-002, SC-005, SC-009 et R17 disent la même chose.
- Constitution : III, VII (T025), XVIII (R13), XIX (deux lots, réglage fermé), XX (preuves `fichier:ligne`, garanties alignées sur les tests), XXII (mesure en T001 et T010).

## Constats restants

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/analyze-report.md#extrait-1).

## Risques de conflit avec les autres sessions

- 157 et 158 modifient `daemon.rs`, `mcp.rs`. Exécution séquentielle.
- 152 (reprise des missions) : les messages de mission ne sont jamais retenus.
- 155 pourra ajouter d'autres types de remise : par défaut non retenus tant qu'ils ne sont pas classés (R11).
- 087 (pause) : `HeldRelease` s'ajoute à la garde unique ; T007 le teste avec la reprise.

## Questions utilisateur (défauts prudents retenus)

1. Le destinataire peut-il retirer ou réordonner sa propre file ? Défaut : non.
2. La file retenue est-elle ouverte par défaut ? Défaut : non, réglage par type d'agent, fermé au départ.
3. Faut-il `edit` réservé à l'auteur ? Défaut : non, reporté.
