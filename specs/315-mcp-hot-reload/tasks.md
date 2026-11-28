# Tâches 315 : Rechargement à chaud de `bridget mcp`

- [x] T001 : Créer `crates/bridget-daemon/src/mcp_hot_reload.rs` (résolution du lien actif, validation sécurité fichiers/répertoires, garde anti-boucle, handoff `BRIDGET_MCP_HOT_RELOAD_V1`, `PollLineReader` sans perte, `execv` sûr, tests unitaires complets).
- [x] T002 : Déclarer `mod mcp_hot_reload;` dans `crates/bridget-daemon/src/lib.rs` et intégrer la capacité `capabilities.tools.listChanged: true`, la boucle de rechargement sûr (zéro appel en vol, `active_workers == 0`, sortie de `std::thread::scope`, flush, `execv`, repli sur erreur) et l'émission de `notifications/tools/list_changed` dans `crates/bridget-daemon/src/mcp.rs`.
- [x] T003 : Mettre à jour l'assertion `matrice_fr009_couvre_les_quinze_cas` dans `crates/bridget-daemon/src/mcp_tests.rs` et ajouter les tests unitaires MCP associés.
- [x] T004 : Créer le test d'intégration bout-en-bout réel `crates/bridget-daemon/tests/mcp_hot_reload_315_test.rs` (bascule atomique v1 -> v2, même PID, réception spontanée de `notifications/tools/list_changed`, `tools/list` fonctionnel après bascule, requêtes pipelinées sans perte, refus d'une cible non sûre sans couper le tube, repli sur ENOEXEC et plafond anti-boucle).
- [x] T005 : Mettre à jour `skills/bridget/SKILL.md` (mention concise du rechargement automatique de `bridget mcp` lors d'un déploiement, sans élision).
- [x] T006 : Formater (`rustfmt`), exécuter les tests unitaires, le test d'intégration 315, les tests compacts MCP (`spec309_*`, `compact318_tests`) et `clippy` via `scripts/build.py cargo`.
- [x] T007 : Faire relire le diff par Claude (`bridget_auto_delegate`, posture `discovery`), corriger tout constat fondé, revalider les tests.
- [ ] T008 : Committer et pousser sur `main`, compiler Linux (`sysle2`) + macOS (`arm64`), déployer sur la flotte via `scripts/deploy-fleet.py --backup-db` (sans toucher `com.bridget.t3`), vérifier en conditions réelles, et nettoyer (`target/`, `menage.py`).
