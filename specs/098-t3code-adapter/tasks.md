# Tâches 098 — Adaptateur adaptateur de bureau (option A, plan révision 3)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/tasks.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/tasks.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/tasks.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/tasks.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/tasks.md#extrait-5).

## US4 — Journal, attach, ré-authentification (P2)
- [x] T010 [US4] Projection du journal : repère d'installation, séquence durable par fil, lecture paginée seulement si `updatedAt` ou `latestTurn` a changé, événements `turn_start {from:"human"}` / `update {kind:text}` / `turn_end` dans l'ordre du fil, lacune annoncée ; `bridget attach` admis. Observable : fil achevé avant le premier passage et fil créé pendant l'arrêt projetés une fois ; fil de 2 000 messages lu par pages ; fil inchangé non relu.
- [x] T011 [US4] Ré-authentification et panne : 401 répété → un renouvellement puis état d'échec explicite dans `status` et le journal ; coupure puis reprise du faux serveur → identités conservées, zéro doublon. Observable : oracles US4 verts.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/tasks.md#extrait-6).

Dépendances : T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T013 → T014. T001 est un gate : une route absente sur l'application réelle arrête tout avant T004.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/tasks.md#extrait-7).

## Convergence (après contre-revue #2)
- [x] C001 Corrélation prouvée : n'apparier que si les messages utilisateur et les tours assistants se correspondent un pour un après l'ancre ; sinon `Waiting` (fil au travail) ou `Ambiguous` (fil au repos) avec journalisation. Tests : tour interrompu sans texte, tour de sous-agent.
- [x] C002 Attente durable écrite avant le dispatch, identifiants de message et de réponse déterministes, état sauvegardé après chaque décision.
- [x] C003 Verrou `auth_failed` : aucune nouvelle émission après le refus d'une session neuve. Oracle : compte d'émissions stable dans la durée.
- [x] C004 Aucune session administrative orpheline : révocation immédiate si la persistance échoue, consignation des échecs, `uninstall` refusant d'effacer sans révocation acquise, rollback du manifeste.
- [x] C005 Politique du fil lue au moment du dispatch, jamais mise en cache. Oracle : politique changée entre deux sondages.
