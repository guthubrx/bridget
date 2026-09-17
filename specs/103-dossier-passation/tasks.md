# Tâches 103

Statut: In Progress — 0/22. Toutes les tâches sont NON TENTÉES :
exclusion explicite de l'implémentation par l'utilisateur dans cette commande documentaire.
Les tests indiqués sont à créer/exécuter ultérieurement, pas des preuves déjà acquises.
Gate reuse-audit.md PASS relu ; contre-revue bdget reçue et corrections intégrées avant cette liste.

Chaque tâche a un résultat observable. Les tâches de tests avant code établissent l'oracle
et documentent les échecs attendus ; ne pas appeler ces tests « verts » avant le comportement.
Les gates de story exigent ensuite tous les tests concernés verts avant livraison de l'incrément.

## Préparation

- [ ] T001 Relire le socle, l'état Git, AGENTS.md et les gates ; confirmer que 103 reste indépendante de 102/104, puis noter les écarts de base dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/implementation.md ; attendu : aucune mutation de main ni dépendance inventée. Couverture : FR-013.

- [ ] T002 Préparer les fixtures synthétiques et l'isolation home/socket/base dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/tests/handoff_103_test.rs ; attendu : arrêt propre de chaque PID de test, aucun SIGKILL ni fournisseur réel ; relire support/idempotent.rs sans copier son kill de groupe. Couverture : FR-010, FR-013.

## US1 — Préparer et transmettre

- [ ] T003 [US1] Écrire les tests de structure, valeurs par défaut et limites exactes dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/src/handoff.rs ; attendu : tests nommés spec103, cas invalides avant tout effet ; documenter les échecs attendus avant implémentation. Couverture : FR-001, FR-002, FR-007, SC-004. Scénarios : S01–S05.

- [ ] T004 [US1] Implémenter les structures strictes et la validation commune dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/src/handoff.rs ; attendu : deux champs obligatoires, listes facultatives, null/inconnus refusés, mesures UTF-8 ; déclarer le module dans src/lib.rs. Couverture : FR-001, FR-002, FR-007. Scénarios : S01–S05.

- [ ] T005 [US1] Implémenter le rendu v1 déterministe et l'aperçu dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/src/handoff.rs ; attendu : ordre/échappement/defaults du contrat, zéro date volatile, borne de 16 384 octets, zéro entrée-sortie ; tests précédents verts. Couverture : FR-003, FR-007, FR-014, SC-004. Scénarios : S01–S05.

- [ ] T006 [US1] Écrire les contrats MCP preview/send et leurs non-effets dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/tests/handoff_103_test.rs ; attendu : preview sans daemon fonctionne, send conserve exactement le corps et l'auteur attesté. Couverture : FR-003, FR-004, FR-010, SC-002. Scénarios : S02, S06.

- [ ] T007 [US1] Exposer bridget_handoff dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/src/mcp.rs ; attendu : validation avant transport, action preview sans connexion, action send délègue execute_send sans toucher canonical_send ; reçus complets. Couverture : FR-003, FR-004, FR-008, FR-010. Scénarios : S02, S06.

- [ ] T008 [US1] Ajouter handoff preview et la lecture JSON stdin bornée dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/src/cli.rs ; attendu : 65 537e octet détecté avant parse, erreurs nominatives, action cohérente avec sous-commande ; --json identique à MCP. Couverture : FR-003, FR-007, FR-012. Scénarios : S18, S19.

## US2 — Références et limites honnêtes

- [ ] T009 [US2] Écrire les tests sources inertes, contenu hostile et conservation dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/tests/handoff_103_test.rs ; attendu : aucune ouverture fs/HTTP/source, messages conservés puis purgés selon configuration, pas de signature implicite du contenu. Couverture : FR-005, FR-006, FR-011, FR-014, SC-006. Scénarios : S07–S11.

- [ ] T010 [US2] Compléter les unions de références et warnings dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/src/handoff.rs ; attendu : source_label déclaratif, path absolu lexical, URL sans userinfo, séquences cohérentes, aucune résolution de droits. Couverture : FR-002, FR-005, FR-006, FR-007, FR-011, FR-014. Scénarios : S07–S11.

- [ ] T011 [US2] Faire restituer les limites et neutraliser les contrôles terminal dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/src/cli.rs ; attendu : rendu humain inerte, sortie JSON conserve le corps exact ; ne pas annoncer confidentialité ou conservation permanente. Couverture : FR-011, FR-014, SC-006. Scénarios : S09–S11.

## US3 — Rejeu et réponse facultative

- [ ] T012 [US3] Ajouter les tests 099 de rejeu, mismatch et perte du reçu dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/tests/handoff_103_test.rs ; attendu : une opération après dix rejeux, ancien corps intact, clé/date conservées à issue inconnue. Couverture : FR-008, SC-002. Scénarios : S12–S14, S17.

- [ ] T013 [US3] Brancher handoff send CLI sur le chemin idempotent existant dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/src/cli.rs ; attendu : identité du wrapper, to UUID, même id/issued_at et corps ; aucune dépendance CLI→parseur MCP ni second transport. Couverture : FR-004, FR-008, FR-010, FR-012. Scénarios : S12–S14, S18.

- [ ] T014 [US3] Vérifier reply, in_reply_to, DND, indisponibilité et révocation dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/tests/handoff_103_test.rs ; attendu : reply=false par défaut, succès transport≠mission, zéro relance/abonnement ; nouveau dossier n'écrase pas l'ancien. Couverture : FR-008, FR-009, FR-010, FR-013. Scénarios : S15–S17.

## US4 — Parité et usage quotidien

- [ ] T015 [US4] Figer les fichiers dorés du rendu dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/tests/fixtures/handoff_103_golden.json ; attendu : Unicode, slash, contrôles, listes vides, ordre ; assertions CLI/MCP et rejouabilité byte-for-byte lors d'une mise à jour de serde_json. Couverture : FR-008, FR-012, SC-003. Scénarios : S18, S20, S23.

- [ ] T016 [US4] Ajouter uniquement bridget_handoff à la liste fermée dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/src/wrapper.rs ; attendu : préserver tous les outils réellement intégrés dont 102 si présent, aucun allow global ; aligner catalogue/tests dans mcp.rs. Couverture : FR-012, FR-013. Scénarios : S20.

- [ ] T017 [US4] Terminer la matrice CLI/MCP dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/tests/handoff_103_test.rs ; attendu : entrées, erreurs, exitcodes et corps identiques ; vieux client affiche un texte ordinaire ; aucun rechargement forcé. Couverture : FR-012, FR-013, SC-003. Scénarios : S18–S20.

- [ ] T018 [US4] Documenter préparation, envoi, rejeu et reprise dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/skills/bridget/references/commandes.md ; attendu : mettre à jour aussi skills/bridget/SKILL.md et README.md ; exemples validés, catalogue réel et conservation explicites, trois exercices synthétiques de reprise. Couverture : FR-001, FR-002, FR-011, FR-012, FR-013, FR-014, SC-001. Scénarios : S21.

- [ ] T019 [US4] Mesurer le rendu borné et les non-effets dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/crates/bridget-daemon/tests/handoff_103_test.rs ; attendu : 200 dossiers, p95<100ms hors réseau, pas de modèle/I/O ; reporter matériel et valeurs, pas seulement PASS. Couverture : FR-003, FR-005, SC-005, SC-006. Scénarios : S22.

## Validation finale

- [ ] T020 Exécuter fmt, tests ciblés et régressions sûres, clippy, puis consigner commandes/résultats dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/implementation.md ; attendu : inspecter le harnais avant processus ; tout test non exécuté porte sa cause exacte, aucun résultat inventé. Couverture : FR-012, FR-013, SC-001, SC-002, SC-003, SC-004, SC-005, SC-006.

- [ ] T021 Confronter spec/plan/tasks au code et mettre à jour /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/analysis.md ; attendu : Analyze puis Converge réel, revue du diff et exigences/tests ; ajouter tâches de manque au lieu de marquer succès. Couverture : FR-001, FR-013.

- [ ] T022 Préparer la livraison et le point de reprise dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation/specs/103-dossier-passation/quickstart.md ; attendu : état exact du code/tests, surfaces partagées, aucune installation/commit sans demande ; valider la skill et ses exemples existants. Couverture : FR-012, FR-013.

## Dépendances et stratégie

Exécution T001→T022. Le premier incrément utile est US1,
mais la livraison de la spec exige TOUTES les stories et gates. Ce n'est pas une autorisation
à s'arrêter après un sous-ensemble. Aucun statut Implemented si une tâche reste ouverte.

Pas de marqueur[P] : les tâches de chaque story se partagent les mêmes fichiers. Une revue
lecture seule peut se faire en parallèle ; l'écriture des fixtures/doc peut être déléguée
après accords explicites et propriété séparée. Ne pas lancer deux agents sur mcp.rs/cli.rs.

## Responsabilité et simplicité

Chaque création doit rester reliée au reuse-audit. Aucun nouveau service/table/dépendance
; seul module métier et outil prévus, format figé vérifiable.
La documentation doit permettre une reprise sans accéder à cette conversation.
Après réalisation : revue du diff, tests négatifs, Analyze, Converge, puis audit de code.
Pas de commit automatique. Installation et agents réels : autorisation nouvelle requise.
