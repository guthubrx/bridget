# Audit de réutilisation — SPEC145

Date : 2026-10-07. Verdict de conception : PASS. Réutilisations vérifiées par l'implémentation145, les tests et l'audit final. Statut : Implemented, 20/20 tâches vérifiées, non installé. L'inventaire initial reste historique ; la clôture suit Converge2 hors comparaison.

## Inventaire vérifié

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/reuse-audit.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/reuse-audit.md#extrait-2).

## Arbitrages CRÉER / RÉUTILISER

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/reuse-audit.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/reuse-audit.md#extrait-4).

L'ancien `ui.rs` est supprimé et n'est pas réutilisable. L'existence des lectures agent ne prouve pas une autorisation humaine. La capacité humaine ne doit pas accepter les actions mutation de l'enum agent.

## Gate avant tasks

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/reuse-audit.md#extrait-5).

## Preuves attendues après implémentation

Complément I004 — RÉUTILISER la notification historique stockée, objet mode none/targets/all et targets UUID[] borné16. Ne pas réutiliser le type de requête Post all|UUID[] pour cette sortie. RÉUTILISER dates Unix i64secondes, closed_at nullable ; pas chaînes ISO. Les sources store/threads.rs624/1035 ont été relues dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/threads.rs`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/reuse-audit.md#extrait-6).

Les tests devront prouver non-mutation des données métier, refus, bornes, projections, contexte obsolète et intégration native. Ce sont des critères d'implémentation, pas des gates de réutilisation exigeant du code avant les tâches. Aucun PASS runtime, sécurité ou recette n'est déduit de l'inventaire. Si l'implémentation révèle une voie différente, réviser l'audit avant de poursuivre.
