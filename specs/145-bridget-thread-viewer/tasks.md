# Tâches — SPEC145

Date : 2026-10-07. Statut : Implemented ; 20/20 tâches vérifiées sur preuves reçues. Cases cochées uniquement dans T020, après audit validé et Converge2, hors comparaison. Non installé, non activé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/tasks.md#extrait-1).

## Fondations — Transport humain et identité

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/tasks.md#extrait-2).

## Fondations — Service et RPC adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/tasks.md#extrait-3).

## US1 — Trouver les fils dans le panneau natif

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/tasks.md#extrait-4).

## US2 — Lire plusieurs fils et leurs pages exactes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/tasks.md#extrait-5).

## US3 — Recherche locale et rafraîchissement manuel

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/tasks.md#extrait-6).

## US4 — États, clavier et protections natives

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/tasks.md#extrait-7).

## Validation croisée et clôture documentaire

- [x] T017 — Exécuter validation transport/CLI/service/UI isolée sur les deux worktrees ; conserver les preuves dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/validation.md`. Couvrir progression de trois pages, même racine Git avec workspace différent, identité dormant, hors liaison/projet, ancienne version, projection sans secrets, CLI froid sans daemon et non-mutation métier. Aucun processus actif. Une recette visuelle peut utiliser seulement l'aperçu isolé autorisé ; l'ouverture about:blank ne suffit pas. FR03–18, FR21–22 et SC01–06. Dépendances : T006, T009, T010, T016.
- [x] T018 — Exécuter contrôles format, lint, types, build et tests applicables selon les scripts existants, sur les deux worktrees. Stocker commandes exactes, exit codes, compteurs et limites de preuve dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/validation.md`. Ne pas réinstaller ni contourner un échec ; distinguer baseline et nouveaux défauts. Effectuer ensuite une première convergence en lecture seule avant T019, avec tâches inchangées si aucun manque ; écrire ensuite le reçu append-only hors phase Converge. FR19, FR21–22 et SC08. Dépendance : T017.
- [x] T019 — Appliquer audit v14 en mode fix avec auto_commit=false, puis un dernier cycle entièrement en lecture seule. Conserver les artefacts sous `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/audits/2026-10-07/session-2026-10-07-spec-145-01/`. Appliquer la revue hostile proportionnée dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/adversarial-review-plan.md`. Produire scoring et constats sur le diff final dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/adversarial-review.md` ; corriger les défauts puis rejouer les contrôles concernés. Après le dernier fix et ses rejeux, effectuer la convergence finale en lecture seule, avec tâches byte-identiques si aucun manque ; écrire ensuite le reçu append-only hors phase Converge. Ne pas prétendre une approbation interprovider indisponible. FR01–22 et SC01–08. Dépendance : T018.
- [x] T020 — Après la convergence finale T019, effectuer seulement la phase documentaire hors Converge : écrire le reçu/journal de validation et mettre à jour les statuts dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/AGENTS.md`. La première convergence T018 et la convergence finale T019 précèdent cette phase. Ne pas modifier les tâches pendant une comparaison Converge. Cocher uniquement les tâches dont les preuves sont reçues ; l'existence des artefacts ne suffit pas. Signaler toute recette native non autorisée/non réalisée. Livrer l'état vérifié sans commit, merge, push, déploiement ou restart. FR21–22 et SC08. Dépendance : T019.

## Couverture et ordre

Règle Converge : cette phase peut seulement ajouter des tâches manquantes attestées. Si aucun manque n'est trouvé, le fichier des tâches reste byte-identique. Écrire les reçus/journaux seulement après la comparaison, hors phase Converge. La clôture T020 suit le dernier audit readonly et cette convergence finale.

| Exigences | Tâches principales |
| --- | --- |
| FR01–02 | T011–T012, T016 |
| FR03–07 | T001–T010, T011–T017 |
| FR08–10 | T003–T004, T011–T015 |
| FR11–12 | T003–T008, T013–T014, T017 |
| FR13 | T015 |
| FR14–15 | T010–T016 |
| FR16–18 | T001–T010, T013–T017 |
| FR19–20 | T007–T016, T018 |
| FR21–22 | T001–T020 selon le périmètre |

Chemin critique : transport humain → CLI → service/RPC → lecture d'environnement → surface native → historique → recherche/états → validation → revue → convergence. Les RED précèdent les changements correspondants. Les validations peuvent révéler un changement de conception ; dans ce cas, corriger le plan et Analyze avant reprise.
