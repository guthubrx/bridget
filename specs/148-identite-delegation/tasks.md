# Tâches148

- [x] T001 Formaliser les contrats et preuves dans specs/148-identite-delegation/.
- [x] T002 [US1] Ajouter l'introspection de session authentifiée et ses refus dans T3 apps/server/src/mcp/.
- [x] T003 [US1] Résoudre l'identité MCP attestée dans crates/bridget-daemon/src/mcp_identity.rs et tester deux sessions partagées.
- [x] T004 [US1] Injecter la preuve propre à chaque montage dans T3 CodexAdapterV2 et ClaudeMcp, avec tests de configuration et opt-out.
- [x] T005 [US3] Exposer la sélection disponible et ses refus dans crates/bridget-daemon/src/mcp.rs.
- [x] T006 [US2] Implémenter la délégation et ses étapes durables dans crates/bridget-daemon/src/ ; tester le rejeu et les coupures.
- [x] T007 [US4] Corréler les états et résultats natifs avec le parent dans Bridget ; tester la fin de tour et les enfants actifs sans T3.
- [x] T008 [US5] Exposer l'annulation appartenant au parent et couvrir les descendants/permissions.
- [x] T009 [US6] Actualiser les guides et la skill dans le périmètre de livraison148.
- [x] T010 [US6] Exécuter les régressions, recette isolée et builds ; écrire validation/final.md et implementation.md.

Deux agents Codex high ont été explicitement autorisés : moteur natif et
connecteur T3. Le principal conserve l'identité Rust, l'intégration et la livraison.
Les revues et tests indépendants utilisent GLM 5.3 via l'orchestrateur T3,
sur demande explicite de l'utilisateur. Cela ne crée aucune dépendance du produit.
T003/T004 dépendent de T002. T006 dépend du catalogue natif ; T007/T008 précèdent
la recette réelle. Les propriétaires de fichiers sont distincts et coordonnés.
T001/T005–T009 validées par les revues GLM natives R2/R3 et la recette native
indépendante R2 (rejeu, résultat, annulation, redémarrage).
T002–T004 validées par la revue identité R2, 389 tests T3 et 21 tests Rust,
puis la recette réseau R4 (6 PASS) : deux fils, DELETE, 404, révocation A/401,
identité A fermée et B encore valide. Le partage app-server vient des tests
adapters ; aucun processus modèle réel dans cette recette réseau.
T010 validée après revue et recette GLM réelle R2, builds privés des commits
fusionnés, signatures et contrôles des fichiers installés. Aucun redémarrage.
