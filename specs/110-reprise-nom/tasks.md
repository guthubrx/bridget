# Tâches 110

Statut: In Progress — 5/6 (T006 : livraison en production).

- [x] T001 Écrire les tests unitaires de reprise dans crates/bridget-daemon/src/agent_profile.rs ; attendu : rouges avant T002.
- [x] T002 Ajouter le prédicat de vivacité et le transfert atomique dans rename_display_name ; attendu : T001 verts.
- [x] T003 Brancher l'annuaire vivant au point d'appel dans crates/bridget-daemon/src/daemon.rs ; attendu : compilation et comportement inchangé pour un détenteur vivant.
- [x] T004 Adapter les autres appelants avec un prédicat refusant le transfert ; attendu : aucune régression.
- [x] T005 fmt, clippy, recette complète ; attendu : 0 échec.
- [ ] T006 Livrer en production : reconstruire, relancer daemon et pont, vérifier les trois fils et le journal ; consigner dans specs/110-reprise-nom/implementation.md.
