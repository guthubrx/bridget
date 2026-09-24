# Tâches 116
Statut: Implemented — 13/13, livré le 2026-09-24 à 07:57.
- [x] T001 Mesures : appariement sur 20 fils vivants, points de contrôle écartés, comportement de `lsof` avec un processus disparu, inventaire de l'état accumulé.
- [x] T002 Appariement par origine prouvée : `record_turn_origin`, `prune_turn_origins`, `proven_turn_answer`, `Correlation::Silent` engagé après SETTLE_ATTEMPTS lectures (`t3code.rs`).
- [x] T003 Inventaire tolérant aux processus disparus : `only_vanished`, `process_gone` (`runtime.rs`).
- [x] T004 Sagas d'envoi : `settle_expired_dispatching`, `purge_expired_sends` (`idempotency/send_delivery.rs`).
- [x] T005 État d'identité des disparus : `purge_stale_identity_files` (`mcp_identity.rs`).
- [x] T006 Rotation des journaux : `rotate_log_if_large`, `service_logs` (`disk_hygiene.rs`).
- [x] T007 Entretien horaire du daemon : `run_maintenance` (`daemon.rs`).
- [x] T008 Worktrees fusionnés : `prune_merged_worktrees` (`scripts/build.py`).
- [x] T009 Refus d'identité expliqué : `unattested_hint` (`cli.rs`).
- [x] T010 Tests : 13 tests Rust spec116 (pont 5, inventaire 2, sagas 2, identité 1, journaux 2, CLI 1), 4 tests Python du script de construction.
- [x] T011 Documentation : référence, ADR 042, README.
- [x] T012 fmt, clippy, recette complète sur l'état final (1543 réussis ; 3 échecs de charge dans `search_104_test`, 23/23 relancé seul trois fois ; Python 26/26).
- [x] T013 Fusion, construction (retrait automatique des worktrees), relance du daemon et du pont, vérification en production.
- [x] T014 Correctifs trouvés en livrant : identifiant de build figé (`build.rs`), fixtures de test abandonnées (`tests/support/idempotent.rs`).
