# Tâches141 — Messages Bridget groupés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/tasks.md#extrait-1).

## Préparation et fondations

- [x] T001 Vérifier les worktrees, la synchronisation, le format réel et la baseline ; lire le gate PASS et fixer le contrat dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/plan.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/reuse-audit.md. Preuves : baseline 269 PASS transmise par le principal et gate validé avant ce fichier.

## US4 — Repli sûr et projection (P1)

Cette histoire correspond à US14104. Elle précède le rendu car le découpage doit être certain avant de présenter les auteurs. Test indépendant : chaque entrée ambiguë conserve le corps entier et la copie source.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/tasks.md#extrait-2).

## US1 — Groupe compact et sections nommées (P1)

Cette histoire correspond à US14101. Test indépendant : trois messages reçus donnent trois sections dans l'ordre, avec les noms et extraits littéraux.

## US2 — Lecture et interactions (P1)

Cette histoire correspond à US14102. Test indépendant : les sections s'ouvrent au clic et au clavier, restent indépendantes et ne déplacent pas l'état vers un autre fil/message.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/tasks.md#extrait-3).

## US3 — Conservation et contrôles ciblés (P1)

Cette histoire correspond à US14103. Test indépendant : copie égale au message reçu et pièces jointes/actions conservées, sans source pour les lots directs.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/tasks.md#extrait-4).

## Convergence et audit

- [x] T008 Relier FR14101–FR14115 et SC14101–SC14107 aux preuves dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/converge.md. Relire le diff ciblé, complexité O(n), minimalisme et responsabilité future ; consigner l'audit et les limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/audits/2026-10-07/session-2026-10-07-spec141-01/scoring.md. Mettre à jour tests/statuts uniquement sur preuve. Aucune installation, aucun restart de production et aucun commit automatique.

## Dépendances et exécution

Ordre : T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008. Les histoires ont toutes priorité P1. Le repli sûr est fondamental ; il est traité avant le rendu. Aucun marqueur [P] : les modifications des deux fichiers d'implémentation et de leurs tests se suivent. Les observations de recette peuvent être préparées séparément, mais ne doivent pas éditer les mêmes fichiers.

L'incrément de projection reste testable avant le rendu. Le rendu complète ensuite le besoin validé ; aucune version réduite n'est livrée à la place du plan complet. Chaque tâche possède un résultat observable. T003 et T005 réutilisent les règles locales. T008 vérifie les lignes et abstractions conservées pour limiter la charge future.

## Corrections issues de Converge — passe 1

Ces deux tâches complètent le périmètre existant. Elles ont suivi RED puis GREEN. Leurs cases ont été mises à jour après les preuves, dans une phase de journalisation distincte de Converge. Aucun statut ni aucune case antérieure n'a changé pendant la passe 1. La nouvelle passe Converge et le verdict final d'audit restent nécessaires pour T008.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/tasks.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/141-messages-groupes/tasks.md#extrait-6).
