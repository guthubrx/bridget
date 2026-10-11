# Rapport d'analyse - Session157

Date : 2026-10-11 (révisé après la contre-revue Codex, ronde 2). Analyse croisée de
`spec.md`, `plan.md`, `tasks.md`, `research.md`, `data-model.md`, `contracts/`.
Lecture seule du code. Aucune exécution de test.

## Périmètre final

Catalogue fiable (verdict unique, préflight complet, contexte déclaré), quotas
visibles et refusés à l'admission (épuisement attesté par fenêtre), remplacement
explicite à l'admission (`fallback`). La reprise sur limite d'usage est sortie
(session 160). Aucune dépendance de 152.

## Couverture des exigences

| Exigence | Tâches | Couverte |
|---|---|---|
| FR-001, FR-002 | T007, T008, T009 | oui |
| FR-003, FR-005 | T009, T011 | oui |
| FR-004 | T005, T010 | oui |
| FR-006 | T009 (le verdict ne fait que lire) | oui |
| FR-007, FR-008, FR-011 | T004, T014 | oui |
| FR-009 | T013, T015 | oui |
| FR-010 | T013, T015, T016 | oui |
| FR-012, FR-013, FR-014 | T017, T018, T020 (ordre type/modèle/effort : T008, T017) | oui |
| FR-015 | T018, T021 | oui |
| FR-016 | T019 | oui |
| FR-017, FR-018 | T021, T018 | oui |
| FR-019 | T003, T022 | oui |
| FR-020 | T020, T023 | oui |

| Critère | Preuve prévue |
|---|---|
| SC-001, SC-002 | T007 |
| SC-003 | T010 |
| SC-004 | T013 |
| SC-005 | T017, T021 |
| SC-006 | T003, T022 |
| SC-007 | T001, T012 |

Toutes les exigences ont au moins une tâche. Toutes les tâches de code
commencent par un test rouge (T004, T007, T013, T017).

## Constats restants

| Id | Gravité | Constat | État | Action |
|---|---|---|---|---|
| F1 | MOYENNE | Un vrai refus de quota et son chemin d'échec n'ont pas été vus en réel. | Reporté | Concerne la reprise : 160. Sans effet sur le refus à l'admission (fait attesté par fenêtre). |
| F2 | MOYENNE | La cause R1 est un scénario de code à deux variantes. L'incident réel n'est pas confirmé. | Ouvert | T007 écrit un test par variante. T002 cherche un fait réel expurgé. Aucun document ne dit « confirmé » sans lui. |
| F3 | MOYENNE | Le catalogue ajoute un 3e calcul de politique par fournisseur. | Ouvert | Mesure T001 et T012, seuil 2 fois, repli en R8. |
| F4 | MOYENNE | `Presence.agent_type` supposé égal au nom du registre. Quota non partagé entre fournisseurs d'un même compte. | Ouvert | T013 teste les trois sources. Limite notée. |
| F5 | MOYENNE | Le catalogue ne couvre pas un `cwd` différent, un fait limité à une demande ni un changement d'état entre deux appels. | Accepté | Garantie bornée et `verdict_context` (D-I). Tests hors garantie en T007. |
| F6 | BASSE | Numéros de ligne de la base avant la session 150. | Accepté | Les tâches citent des noms de fonction. |
| F7 | INFO | Authentification non vérifiée. | Accepté | Question 3, `checks.auth: false`. |
| F8 | INFO | Quotas volatiles : perdus au redémarrage jusqu'à la prochaine republication. | Accepté | Dette notée. |
| F9 | INFO | Reprise sur quota (US4 d'origine) retirée : 4 défauts nouveaux et 1 objection ouverte en ronde 2. | Reporté | `spec.md`, section « Session future 160 », six exigences d'entrée et condition de démarrage. |

Aucun constat bloquant pour l'implémentation des 26 tâches.

## Cohérence entre artefacts

- Les codes de refus de `contracts/delegation157.md` correspondent à FR-010,
  FR-013 et FR-016.
- Le seul champ ajouté à la tâche est `substitution` (FR-015).
- Les fichiers du plan correspondent aux tâches (un module nouveau, T004).
- Le statut des tâches est « 0 faite sur 26 ». La fiche synthèse le dit.
- Aucun contrat de reprise n'existe plus dans la 157 (`reprise-quota157.md` supprimé).

## Note visée par critère

- Lancement : 4 vers 5 (catalogue fiable, refus précoces, remplacement explicite).
- Résilience : reste à 4 (note de 152). La 157 évite les échecs prévisibles à
  l'admission. Le passage à 5 est reporté à la 160.

## Verdict

Prêt pour l'implémentation, sous réserve de la validation des trois questions de
la spec (défauts notés).
