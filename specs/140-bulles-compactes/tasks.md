# Tâches SPEC140 — Bulles Bridget compactes

Statut : Implemented (code et vérifications, sans livraison production). Gate reuse-audit : PASS validé par le principal.
Primitive speckit-tasks lue. setup-tasks.sh, template local et extensions.yml absents, constatés par ls : génération manuelle suivant le même contrat. Aucun commit automatique. Chaque case nécessite une preuve observable.

## Phase1 — Préparation

- [x] T001 Vérifier les deux worktrees, la checklist et les points de réutilisation avant création ; consigner le périmètre et les recherches dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/implementation.md. Résultat : liste de fichiers autorisés et isolation confirmées, aucune base active utilisée.

## Phase2 — Fondation

Les décisions de compatibilité et de réutilisation sont prises dans les artefacts. T001 est le prérequis commun. Aucun framework, store, API ou dépendance à installer. Les tests de chaque story précèdent son code.

## Phase3 — US1 : bulle compacte accessible (P1)

Test indépendant : cinq enveloppes complètes se replient sur une ligne et s'ouvrent ; messages ordinaires inchangés.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/tasks.md#extrait-1).

## Phase4 — US2 : libellés sans invention (P1)

Test indépendant : reply, nom absent et compteurs ont des libellés fiables ; aucun mot du corps ne devient un type de mission.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/tasks.md#extrait-2).

## Phase5 — US3 : vrai titre de fil compatible (P1)

Test indépendant : destinataire membre reçoit le titre ; absent, ancien format ou non membre ne reçoit aucun titre indu ; canon et journal sont identiques.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/tasks.md#extrait-3).

## Phase6 — US4 : conservation et virtualisation (P1)

Test indépendant : anciennes et nouvelles entrées gardent texte, copie, pièces jointes et actions ; changer de fil ne transporte pas l'état ouvert.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/tasks.md#extrait-4).

## Phase7 — Vérification transversale

- [x] T013 Lancer les tests Rust concernés, les tests MessagesTimeline, format/lint/TypeScript ciblés et build web. Consigner commandes exactes, codes retour et limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/implementation.md. Résultat : aucun succès global supposé, aucune écriture production ni fournisseur payant.
- [x] T014 Confronter toutes les FR au code et aux tests après la dernière correction ; demander la contre-revue inter-fournisseurs si disponible puis exécuter l'auditv14 ou son fallback documenté. Consigner preuves et limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes/specs/140-bulles-compactes/implementation.md et les rapports voisins. Résultat : Analyze documenté dans ce tour, Converge CONVERGED sans mutation de tasks hors ajout de manques, audit sans finding bloquant ; sinon statut In Progress et première tâche exacte restante.

## Dépendances et parallélisme

T001 → T002 → T003 → T004 → T005 → T006 → T007 → T010 → T011 → T012 → T013 → T014.
Branche Rust indépendante des fichiers frontend : T001 → T008 → T009 → T013.
US3 s'intègre au rendu US1 après T011. US2 et US4 utilisent le même composant frontend : ne pas faire écrire deux agents sur ses fichiers en parallèle. Les tests RED Rust peuvent avancer pendant les tests/rendu US1. Les deux branches doivent avoir fini avant T013.

## Stratégie de mise en œuvre

Incréments testables, mais pas de livraison MVP présentée comme complète : les quatre stories sont obligatoires. Les composants, la copie et l'ancrage existants réduisent la maintenance. Le seul nouvel asset représente le logo officiel. Le champ titre et sa ligne JSON portent une compatibilité nécessaire, pas une abstraction de convenance. Aucun commit, paquet installé ou service redémarré automatiquement.

## Convergence

Réservé aux éventuelles tâches ajoutées en append-only si un écart à la spec est démontré. Une tâche cochée n'est pas une preuve de conformité à elle seule.
