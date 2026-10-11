# Rapport d'analyse - Session155 (après contre-revues Codex, rondes 1 et 2)

Date : 2026-10-11. Analyse croisée de `spec.md`, `plan.md`, `tasks.md`,
`data-model.md`, `contracts/child-requests155.md`, `reuse-audit.md`,
`adversarial-review-codex.md`. Lecture seule, aucun test lancé.

## Couverture des exigences par les tâches

| Exigence | Tâches |
|---|---|
| FR-003, FR-012 en mode `deny` (repli fermé) | T000 |
| FR-001 à FR-004 (option, politique inchangée, relecteur) | T003, T010, T011 |
| FR-005 à FR-008 (interception, budget, lien avec la mission) | T004, T012 à T016, T017, T018 |
| FR-009 à FR-013 (verdict, limite, écritures confinées refusées, décisions) | T006 à T009, T012, T013, T017, T018, T024 |
| FR-014 à FR-017 (réponse du parent, outil, pause) | T024, T025, T027 |
| FR-018 à FR-022 (avis, faits d'abord, statut, boîte, lecture exacte) | T019, T020, T026, T028, T029, T032 |
| FR-023 à FR-027b (délais, issues, validité de remise, fermeture sérialisée) | T016, T018, T021 à T023, T029 |
| FR-028 (corrélation) | T005, T017 |
| FR-029 (pas de boucle) | T030, T031 |
| FR-030, FR-031 (trace, rétention) | T005, T018, T033 |
| FR-032 (documentation, ADR) | T036, T037 |
| SC-001 à SC-011 | T006, T012, T017, T022, T028, T030, T033, T035, T039 |

Le nombre de tâches est 41 (T000 à T040), comme dans la fiche de `spec.md`.

## Objections de la contre-revue : état

| # | Objection | État |
|---|---|---|
| 1 | Accord humain hors limite | fermée par FR-010, FR-013, verdict `Forbidden`, T006/T007 |
| 2 | Chemins | ronde 2 : fermée par refus de toute écriture de fichier sous `workspaceWrite` (FR-010, FR-011, T008/T009) ; plus de résolution de chemins |
| 3 | Méthodes et discovery | fermée sous condition de T002 (liste blanche ; toute méthode non établie est refusée) |
| 4 | Relecteur automatique | fermée par FR-002, FR-004, T010 |
| 5 | Repli `acceptForSession` | ronde 2 : fermée en `forward` (T012/T013) ET en `deny` (T000, préexistant en production) |
| 6 | Corrélation | fermée par FR-028, T005, T016, T017 |
| 7 | Accord non remis | ronde 2 : fermée par `sent` avant envoi, verrou du wrapper, `ChildRequestCancel` confirmé, `unknown` sans accusé (FR-026, FR-027, FR-027b, T016, T021 à T023) |
| 12 | (ronde 2) Échéance absente du contrôle de remise | fermée par FR-027, SC-012, T021, T022, T029 |
| 13 | (ronde 2, préexistant) Repli du mode `deny` | fermée par T000, livrable seul |
| 8 | Injection | fermée par suppression de l'approbation par le parent, FR-019, T032 |
| 9 | Décision humaine tardive, famine | fermée par FR-022, T028/T029 |
| 10 | Dépôt non lié | fermée par FR-021, T020 |
| 11 | Outil absent de la liste fermée | fermée par FR-015, T025 |

## Constats corrigés pendant l'analyse

0. Ronde 2 : T000 ajouté en tête ; résolution de chemins retirée ; état `sent` et trames `ChildRequestCancel` / `ChildRequestClosed` ajoutés ; échéance contrôlée à la remise.
1. Élicitation en mode URL : refusée en v1 (plus de route humaine).
2. `publish_approval_wait` coupe l'exécution après 3 demandes (`wrapper.rs:5700`) : écarté (T016).
3. Le passage `never` vers `on-request` n'était utile qu'à refuser : supprimé.
4. La table de couverture pour le parent était vide pour Codex vers Codex : supprimée,
   remplacée par des verdicts.

## Constats restants

| Id | Sévérité | Constat | Suite |
|---|---|---|---|
| A1 | haute | Champs d'effet de chaque méthode Codex, durée des octrois, forme de `requestUserInput` : non lus dans ce dépôt | T002 avant T003 ; la liste blanche remplace les hypothèses |
| A2 | haute | La session 151 n'est pas écrite ; la forme de son outil d'attente est inconnue | T034 conditionnelle, sans code spéculatif |
| A3 | moyenne | Transition `waiting_approval` vers `running` non prouvée | vérifiée en T016 |
| A4 | moyenne | Acquis 150 supposés livrés (`mission_deadline_at`, variante de garde, purge d'arbre) | contrôle en T001 ; sinon T010, T018, T027 ou T033 bloquées |
| A5 | moyenne | Les écritures de fichier sous un parent confiné sont refusées en v1 ; le gain d'une approbation de fichier est perdu jusqu'à une preuve de confinement à l'exécution | session future ; consigné dans l'ADR |
| A10 | moyenne | T000 change un comportement de production du mode `deny` (seule exception à FR-003) | tests rouges, livraison seule, note dans l'ADR 155 |
| A11 | basse | Un accord `unknown` (envoyé sans accusé, wrapper disparu) ne peut pas être prouvé annulé | l'issue le dit ; l'enfant est arrêté avec la tâche |
| A6 | moyenne | Accord humain pour une commande avec un parent `dangerFullAccess` : effets non bornés par nature | c'est le droit du parent ; consigné dans l'ADR |
| A7 | basse | Fenêtres 600 s et 3600 s : valeurs de départ | constantes uniques |
| A8 | basse | `native_delegation.rs` modifié par 150 à 158 | exécution séquentielle ; logique dans un fichier neuf |
| A9 | basse | Note visée : 4 sur 5, pas 5 | 5 demande « oui » aux questions 2 et 3 |

## Contrôles de cohérence

- Sécurité : aucun `accept` sans décision humaine ni validité de remise (SC-001), jamais après l'échéance (SC-012), jamais de décision durable dans aucun mode (SC-013). La
  limite absolue vaut aussi pour l'humain. Les décisions durables ne sont jamais relayées.
- Compatibilité : le défaut `deny` garde les octets canoniques (T003, T010) et le chemin
  Codex actuel (T012, T039). La politique de l'enfant n'est pas touchée.
- Minimalisme : un fichier neuf, une table, cinq trames, un outil ; la résolution de chemins a été retirée. La contre-revue a
  réduit le périmètre (plus d'approbation par le parent, plus de relève par préfixe).
- Complexité : toutes les boucles sont bornées (4, 32, 512, 64, 4096 octets).
- Décision 149 : le changement est explicite dans `spec.md` et l'ADR (T036).

## Questions ouvertes pour l'utilisateur

1. Défaut de `child_requests` : `deny` (proposé) ou `forward`.
2. Le parent peut-il approuver une action : non (proposé) ou oui pour l'enfant discovery seulement.
3. Enfants Claude et GLM : Codex seul (proposé) ou inclusion maintenant.

## Verdict

T000 peut être livré tout de suite et seul. Le reste est prêt après T001 et T002. Deux réserves ouvertes : A1 (se ferme en
T002) et A2 (dépend de la session 151).
