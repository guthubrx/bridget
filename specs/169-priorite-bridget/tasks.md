# Tâches SPEC169 — Priorité Bridget depuis adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-1).

Gel P2 historique : SHA-256 2f31c10404bfd9f81d273285ebea29605a42dd27df5a1cec27dafafa2c678147, strictement identique avant/après cette phase close. Après sa clôture et sur GO ROOT, seul cet en-tête est actualisé documentairement ; les14 cases et descriptions fonctionnelles restent conservées. Les formulations de la section Convergence relatent le checkpoint antérieur à P2. Aucun nouveau passage ni changement de code n'est déclaré.

Source du besoin : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/spec.md

Source de conception : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/plan.md

Audit de réutilisation : PASS,15/15 items,0 doublon,0 arbitrage ouvert. La contre-revue et les deux corrections Analyze sont intégrées. Aucun test d'exécution n'est déduit de ces gates documentaires. Les scripts et templates projet sont absents ; ce découpage applique manuellement le protocole Tasks lu.

## Phase1 — Préparation existante

Les worktrees, sources et artefacts169 sont préparés par ROOT. Aucun scaffolding, dépendance ou service supplémentaire n'est requis. Le worker principal Cartae garde les fichiers de préférence et d'adaptateurs ; un second worker possède exclusivement les deux fichiers de T012. ROOT garde Git et livraison. Aucun responsable ne rétablit les modifications d'un autre.

## Phase2 — Socle commun bloquant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-2).

## Phase3 — US1 : priorité automatique Bridget

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-4).

## Phase4 — US2 : changer le réglage par fichier

But : le tour suivant prend le réglage fiable, sans toucher l'identité. Test indépendant : absence/OFF/ON/blocked puis changements dans les deux sens.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-5).

## Phase5 — US3 : contraintes et droits conservés

But : la préférence ne choisit aucun axe à la place d'une contrainte humaine et n'accorde aucun droit. Test indépendant : contrainte préservée ou refus nommé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-6).

## Phase6 — US4 : indisponibilité et refus sans repli silencieux

But : chaque raccord transmet un état fidèle. Test indépendant : absence/panne/refus/rejeu ne déclenchent pas de second moteur d'hôte.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-7).

## Phase7 — US5 : suivi par moteur initial

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-8).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-9).

## Phase8 — Validation transversale avant intégration

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-10).

## Dépendances, parallélisme et stratégie

Ordre principal : T001 → T002 → US1(T003–T004) → US2(T005) → US3(T006) → US4(T007–T009) → US5(T010). T010 ET T012 → T011. T012 corrige un prérequis causal indépendant de la préférence ; les IDs antérieurs restent inchangés. T003 appartient à US1 et modifie le socle partagé ; aucun autre fournisseur ne doit réécrire ce fichier en parallèle.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-11).

Les tâches réduisent la maintenance en réutilisant service, input par tour et compositeur. Seule la commande privée Pi ajoute un mécanisme borné, justifié par le hook statique et le comportement réel du SDK. Cet aménagement d'exécution ne change ni couverture, ni exigence, ni dépendance.

Le lot couvre les cinq histoires et les17FR/9SC. Il ne se réduit pas à US1 seule. Chaque tâche exige ses observations avant cochage. ROOT décide le statut `Implemented` seulement à100% d'implémentation, en gardant les jalons suivants ouverts tant que leurs preuves manquent.

## Jalons du pipeline distincts des14 tâches

Ces jalons ne participent pas au gate de100% des tâches avant commit. Ils n'exigent pas un commit futur comme précondition de lui-même.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-12).

L'estimation élargie avec activation sera publiée par ROOT après ce découpage. Les gates documentaires n'attestent aucune tâche exécutée ni livraison future.

## Convergence

Reprise corrective après la contre-revue de code R1, autorisée par ROOT. Le verdict CONVERGED du passage1 décrit le gel antérieur. Les deux corrections T013/T014 sont maintenant acceptées par ROOT sur les preuves R2 ; les12 tâches antérieures restent cochées. Un passage2 doit relire ce nouveau gel. Ce cochage ne ferme aucun jalon de livraison et ne vaut ni contre-revue R2 ni audit.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/169-priorite-bridget/tasks.md#extrait-13).
