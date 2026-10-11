# Rapport d'analyse - Session160

Date : 2026-10-11. Analyse croisée de `spec.md`, `plan.md`, `tasks.md`,
`research.md`, `data-model.md`, `contracts/reprise160.md`. Lecture seule du code.
Aucune exécution de test. Une contre-revue Codex a été faite (ronde 1, voir
`adversarial-review-codex.md`) ; ses huit objections de la ronde 1 et les trois points de la ronde 2 sont
traités.

## Couverture des six exigences d'entrée

| Exigence (contre-revue 157, ronde 2) | Exigences | Tâches (rouge puis vert) |
|---|---|---|
| 1 Compteur durable | FR-008 | T007 puis T010, T022 |
| 2 Reconstruction ciblée de `failed` | FR-013 | T004 puis T011, T018 |
| 3 Identité exacte enregistrée avec la continuation, remise unique et complète | FR-012, FR-009, FR-011b, FR-011c | T008, T032 puis T011, T018, T019, T020, T034 |
| 4 Cause distincte du compteur de pannes | FR-010, FR-018 | T006 puis T021 |
| 5 Gel cumulé sans double comptage | FR-016 | T005 puis T012, T023 |
| 6 Gestionnaire `DeliveryRejected`, refus ancien, avis durable | FR-006, FR-006b, FR-007 | T003, T033 puis T015, T016, T017, T035 |

## Couverture des exigences

| Exigence | Tâches | Couverte |
|---|---|---|
| FR-001 | T009, T025 | oui |
| FR-002, FR-003 | T013, T014, T015 | oui |
| FR-004, FR-004b | T016, T033 | oui |
| FR-005 | T014, T022 | oui |
| FR-006, FR-006b, FR-006c, FR-007 | T015, T017, T035, T036, T037 | oui |
| FR-008, FR-009 | T010, T011, T016, T022 | oui |
| FR-010 | T018, T021 | oui |
| FR-011, FR-011b, FR-011c, FR-012 | T008, T018, T019, T020, T032, T034 | oui |
| FR-013 | T018 | oui |
| FR-014 | T020, T023 | oui |
| FR-015 | T026 | oui |
| FR-016 | T012, T023 | oui |
| FR-017 | T024 | oui (réconciliation en `relaunching`) |
| FR-018 | T016, T021 | oui |
| FR-019 | T010, T017, T025 | oui |
| FR-020 | T025, T028 | oui |

| Critère | Preuve prévue |
|---|---|
| SC-001 | T003, T015, T016, T017 |
| SC-002 | T016 |
| SC-003 | T022 |
| SC-004 | T008, T019, T020 |
| SC-005 | T022 |
| SC-006 | T005, T023 |
| SC-007 | T004, T018 |
| SC-008 | T026 |
| SC-009 | T009, T025, T027 |
| SC-010 | T020, T023 |
| SC-011 | T033, T035, T036 |
| SC-013 | T015, T037 |
| SC-012 | T032, T034 |

Toutes les exigences ont au moins une tâche. Tous les tests rouges (T003 à T008, T032, T033, T037)
précèdent le code qui les rend verts. Chaque phase de code a un filtre de test
indépendant.

## Constats restants

| Id | Gravité | Constat | État | Action |
|---|---|---|---|---|
| F1 | HAUTE | 152 (révisée ronde 2) et 157 sont planifiées, pas livrées. Toutes les primitives de `contracts/reprise160.md` sont des noms de tâches. | Ouvert | Porte T001 : arrêt si une primitive manque ou diffère. À revérifier à la livraison. |
| F2 | MOYENNE | Le chemin d'échec d'un vrai refus de quota (A, B, raison, ordre avec le fait) n'a jamais été vu en réel. | Ouvert | T002 cherche un exemple. Sans exemple, la validation le dit. La décision `qualify_quota` est commune à tout chemin connu. |
| F3 | MOYENNE | La décision en attente de 5 s n'est pas mesurée en réel. | Ouvert | Constante unique, tests aux deux ordres. À ajuster après T002. |
| F4 | MOYENNE | La relance ne passe pas par la continuation gouvernée (budget). La session158 devra compter ces relances. | Ouvert | Décision D-H, T002 relit `reserve_governed_continuation`. ADR T029. |
| F5 | MOYENNE | Avis d'échec perdu ou doublé après un crash si la décision différée rejouait le message direct. | Corrigé | Pas de rejeu : l'avis durable `native-failure-<task_id>` est le seul avis (T015, test de crash avant et après `failed`, réservation, gravure, solde). |
| F6 | MOYENNE | La mémoire de limites de 157 n'expose peut-être pas l'instant d'observation par fenêtre. | Ouvert | T013 l'ajoute sans changer les sorties de 157 si absent. |
| F7 | MOYENNE | Faux positif : un échec sans rapport pendant un quota épuisé déclenche une relance inutile. | Accepté | Opt-in, fait postérieur à l'envoi, borne de 2 rondes. |
| F8 | BASSE | Le gel de l'enfant ne gèle pas son parent, qui peut expirer. | Accepté | Limite notée, annulation en cascade de 152. |
| F9 | BASSE | Numéros de ligne de la base avant la session 150. | Accepté | Les tâches citent des noms de fonction. |
| F10 | INFO | La mémoire des limites est volatile. Une entrée `waiting` garde son retour, donc la relance n'en dépend pas. | Accepté | Dette de 157. |
| F11 | INFO | Une seule ronde de contre-revue Codex sur 160. | Ouvert | Une ronde 2 est possible avant l'implémentation (150, 152 et 157 en ont eu deux ou trois). |
| F12 | MOYENNE | La première hypothèse « la transition `failed` précède `DeliveryRejected` » était fausse pour l'injection synchrone et la réponse vide. | Corrigé | Corrélation par exécution et génération (trame étendue, compatible), échec explicite sans corrélation après une relance (T033, T035, T036). |
| F14 | MOYENNE | Un refus de quota par l'injection synchrone ou la réponse vide n'est pas repris (la feuille n'est pas `failed`). | Accepté | Échec avec l'erreur d'origine. À revoir si T002 montre que le vrai refus passe par là. |
| F15 | MOYENNE | Une remise de relance mise en quarantaine à la reconnexion (`indeterminate`, absorbant) ne peut pas être reprise. | Corrigé | Solde à `acked` seulement, sinon `native_relaunch_failed` (T008, T020). |
| F16 | BASSE | Si la liaison du wrapper manque au refus d'une injection synchrone, la trame n'a pas de corrélation. | Ouvert | T036 le vérifie ; règle : refus sans corrélation après une relance = échec explicite. |
| F13 | BASSE | Pour une tâche opt-in, le message direct « Échec de livraison » est remplacé par l'avis durable : changement visible pour le parent. | Accepté | Documenté (T028), contrat. |

Aucun constat bloquant pour la planification. F1 bloque le démarrage du code
(porte T001).

## Cohérence entre artefacts

- Les champs de `data-model.md` correspondent à FR-008 à FR-013.
- Les codes et champs de `contracts/reprise160.md` correspondent à FR-005, FR-017,
  FR-019 et FR-020.
- Les fichiers du plan correspondent aux tâches. Pas de module nouveau.
- La spec 157 renvoie à cette session (« Session future 160 »).
- Le statut des tâches est « 0 faite sur 37 ». La fiche synthèse le dit.
- Les noms de tâche de 152 et 157 cités existent dans leurs `tasks.md` du
  worktree à la date de l'analyse.

## Note visée par critère

- Résilience : 4 vers 5, sous réserve que la porte T001 passe et que les six
  exigences soient vertes.
- Lancement : inchangé (5 grâce à la 157).

## Verdict

Prêt pour l'implémentation, sous réserve de la porte T001 et des trois questions
de la spec (défauts notés). Une ronde 2 de contre-revue reste souhaitable (F11).
