# Rapport d'analyse - Session158 (158a plan et tâches ; 158b spec)

Date : 2026-10-11, révisé après la contre-revue Codex. Analyse non destructive
de `spec.md`, `plan.md`, `tasks.md`, `data-model.md`, `contracts/control158.md`,
`reuse-audit.md`, `adversarial-review-codex.md`.

## Couverture exigence vers tâche (158a)

| Exigence | Tâches | Test |
|---|---|---|
| FR-001, 002, 005 | T006, T007 | `control158_overview_*` |
| FR-003 | T008 | `control158_overview_client_agent_negocie_refuse`, `..._client_non_negocie_refuse` |
| FR-004 | T008, T009 | `control158_overview_cancel_cascade` |
| FR-006, 012 | T012, T013 | `control158_halt_exige_acteur_humain` |
| FR-007 | T004, T011 | `control158_halt_admission_4_combinaisons`, `control158_halt_et_pause_4_combinaisons_a_l_admission` |
| FR-008 | T010, T011 | `control158_halt_refuse_nouvelle_cle`, `control158_halt_rejeu_cle_admise_rend_la_tache` |
| FR-009 | T012 | `control158_halt_cancel_active_annule_les_arbres` |
| FR-009, FR-010 | T003, T012 | `control158_halt_store_a_b_rejeu_a`, `..._apres_redemarrage`, `control158_halt_a_b_c_rejeu_a_ne_touche_pas_c`, `control158_halt_a_b_c_rejeu_a_apres_redemarrage` |
| FR-011 | T013 | affichage dans `control status` |
| FR-020, 021, 023, 025 | T005, T014 à T016 | `mcp158_scopes_couvrent_tous_les_outils`, `mcp158_outil_sans_portee_echoue` |
| FR-022, 026, 029 | T015, T018 | `mcp158_discovery_lecture_ack_page_suivante`, `..._chaque_action_de_fil`, `..._journal_avec_et_sans_to`, `..._reponse_automatique_au_parent`, `..._publication_artefact_refusee_sans_effet`, `mcp158_journal_read_only_hint_faux` |
| FR-024 | T017 | `mcp158_plafond_*` (posture omise, terminal avant nettoyage, entre rondes) |
| FR-027 | T001, T021 | non-régression |
| FR-028 | T019, T020 | relecture de documentation |

| Critère | Tâches |
|---|---|
| SC-001 | T006, T008 |
| SC-002 | T010, T011 |
| SC-003 | T012 |
| SC-004 | T013, T016 |
| SC-005 | T003 |
| SC-006 | T016 |
| SC-007 | T017, T018 |
| SC-008 | T001, T021 |

Toutes les exigences 158a ont au moins une tâche. 158b n'en a volontairement aucune.

## Suivi des 10 objections de la contre-revue

Les 10 objections sont retenues. Les points 1 à 5 et 9 sont corrigés dans les
artefacts. Les points 6, 7, 8 et 10 sont résolus par le retrait du budget de
tours (détail dans `adversarial-review-codex.md`). Aucune objection rejetée.

## Ronde 2 de la contre-revue

Objections 1, 3, 4, 5, 9 fermées ; 2 fermée dans la fenêtre de 256 commandes ; 6, 7, 8, 10 sans objet. Trois points restants intégrés : rejeu de `--cancel-active` (FR-009, FR-010, T003, T012), politique `discovery` explicite (FR-029, T018), `readOnlyHint` de `bridget_journal` (FR-022, FR-023, T016). Le point d'élargissement laissé à l'implémenteur est supprimé.

## Cohérence entre artefacts

- Plus aucune mention de `max_turns`, `turns_used`, `budget_reached` en 158a, sauf dans les textes de retrait et de reprise future.
- L'ordre des refus du contrat étend celui de 150 sans le contredire.
- Les filtres de `quickstart.md` correspondent à ceux de `tasks.md`.
- La garde commune (T004) est utilisée par l'admission (T011) et prévue pour 158b (`ScheduledWake`).

## Constats restants

| ID | Sévérité | Constat | Traitement |
|---|---|---|---|
| A-1 | Moyenne | La recherche de la tâche d'un appelant par nom d'enfant n'est pas mesurée. | T017(b) mesure avant le code. Repli : cache par nom d'enfant. |
| A-2 | Moyenne | Les lignes citées viennent du checkout principal, pas du worktree après 150 à 157. | Relecture imposée par `tasks.md` ; noms de fonction cités. |
| A-3 | Moyenne | La liste des outils de 151 à 157 n'est pas connue précisément. | Le test exhaustif (T016) l'impose ; T015 classe à l'implémentation. |
| A-4 | Moyenne | Le critère Contrôle humain atteint 4, pas 5, sans budget de tours. | Écrit dans la spec et la contre-revue ; question 3. |
| A-5 | Faible | `issuer_scope` est déclaré par le client : l'attestation humaine reste bornée par l'ADR 011. | Écrit en R1 et en hypothèses ; non refermé. |
| A-6 | Faible | Reçus de commande limités à 256 ; annulation au plus une fois (crash entre l'état et l'annulation). | Limites documentées (data-model, FR-009). |
| A-7 | Faible | La garantie de portée repose sur la CI, pas sur le compilateur. | Déclaré en R6 et ADR. |
| A-8 | Info | 150 n'est pas entièrement cochée dans le worktree au moment de l'écriture. | Prérequis écrit en tête de `tasks.md`. |
| A-9 | Info | Le secret (US8) n'a pas de consommateur Bridget. | Reporté avec condition préalable. |

Aucun constat bloquant.

## Vérifications de forme

- Statuts et priorités conformes. Pas de tiret cadratin. Phrases courtes.
- Aucune dépendance ajoutée. Un module créé et justifié.
- `tasks.md` : T001 à T022, marqueurs `[P]` et `[US#]`, chemins dans chaque tâche, tests rouges d'abord en phases 4 et 5.

## Verdict

PASS pour passer à l'implémentation de 158a, sous réserve de A-1 (mesure en T017) et A-3 (ordre du pipeline). 158b reste en spec.
