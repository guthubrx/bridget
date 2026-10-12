# Converge158A — passage final

**CONVERGED : 33/33 exigences, 22/22 tâches. Aucune tâche ajoutée.**

Les 22 FR, 8 SC et 3 US sont reliés au code réel, aux tests et aux reçus. Les passages provisoires avant puis après corrections sont conservés. Le passage final suit le GO ROOT pour T021/T022, la lecture du dernier littéral C1 et la vérification des empreintes.

Tâches avant/après : bd382d53768f980b1bafabb27a08cd0d284a3b6206fd66f9ecaff0a6ba64b21c. Elles restent byte-identiques durant cette lecture.
Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle
Capture : /Users/moi/.cache/bridget-audit158-delivery-20261012/scope-files.json
SHA256 : 2951edefe71e5b32c68cb48ea98cf1f7528796eba42afb59f200e84ca36649db.
Diff : /Users/moi/.cache/bridget-audit158-delivery-20261012/diff158A-final.patch
SHA256 : 23594335c28e4c399cad286309a20419bc3be67da1c488eb7177d9526c8fb925.

La capture de livraison compte 27 fichiers, dont 13 Rust et 14 documents, avec 151 hunks. La revue T022 précédente couvre 152 hunks : 126 corps identiques réutilisés et 26 relus. La réduction d’un hunk suit les deux cases cochées. Le dernier delta de code est un littéral CLI. Cet owner ne revendique pas une lecture entière de l’architecture.

La FULL historique conserve 2440 PASS, 12 FAIL et 74 ignored sur 95 cibles. Elle a terminé en 804,17 s, RC101. Les 255 statuts de référence sont identiques. Les 47 oracles158 passent. Les quatre nouveaux échecs sont résolus par quatre replays distincts. Le lot lib de 22 PASS couvre les correctifs de plafond et la preuve ChildProof. Huit échecs historiques restent déclarés. Aucun GREENglobal, seconde FULL ou livraison n’est déduit.

R2 est APPROVE_WITH_CHANGES, sans blocage. C1 : le message CLI est neutre, y compris pour resume. C2 : seule la section ChildProof avant `let queue = tools()` est byte-identique. Elle compte 1710 octets, SHA256 c107269454f8b8d3923eaf36a546972137760630a74a86fd6f61f3a010010d69. La section positive garde une ligne CallerScope. C3 : le message de panne n’a pas de test forcé. Cette limite reste acceptée selon les tests prévus. Aucun scénario ni outil n’est ajouté. La compilation de livraison reste une preuve à venir.

Les 22 tests lib et les quatre replays portent exactement les mêmes mcp0c525… et cli3c428… avant/après, avec 218 sources stables. Les dates ne remplacent pas cette preuve. Le littéral C1 ultérieur n’est pas attribué à ces runs.

| Exigence | Tâches | État |
|---|---|---|
| FR-001 | T006, T007, T009 | PROUVE_CIBLE |
| FR-002 | T006, T007 | PROUVE_CIBLE |
| FR-003 | T002, T008 | PROUVE_CIBLE |
| FR-004 | T008, T009 | PROUVE_CIBLE |
| FR-005 | T006, T007 | PROUVE_CIBLE |
| FR-006 | T003, T012, T013 | PROUVE_CIBLE |
| FR-007 | T004, T011 | PROUVE_CIBLE |
| FR-008 | T010, T011 | PROUVE_CIBLE |
| FR-009 | T003, T012 | PROUVE_CIBLE |
| FR-010 | T003, T012 | PROUVE_CIBLE |
| FR-011 | T013 | PROUVE_LECTURE_ET_RECU |
| FR-012 | T004, T012, T013 | PROUVE_CIBLE |
| FR-020 | T005, T015 | PROUVE_CIBLE |
| FR-021 | T005, T013, T016 | PROUVE_CIBLE |
| FR-022 | T015, T018 | PROUVE_CIBLE |
| FR-023 | T014, T016 | PROUVE_CIBLE |
| FR-024 | T017, T018 | PROUVE_CIBLE |
| FR-025 | T015, T016 | PROUVE_CIBLE |
| FR-026 | T018 | PROUVE_CIBLE |
| FR-027 | T001, T002, T021, T022 | PROUVE_CUMULATIF |
| FR-028 | T019, T020 | PROUVE_DOCUMENTAIRE |
| FR-029 | T018, T019 | PROUVE_CIBLE |
| SC-001 | T006, T008 | PROUVE_CIBLE |
| SC-002 | T010, T011 | PROUVE_CIBLE |
| SC-003 | T012 | PROUVE_CIBLE |
| SC-004 | T013, T016 | PROUVE_CIBLE |
| SC-005 | T003, T012 | PROUVE_CIBLE |
| SC-006 | T014, T016 | PROUVE_CIBLE |
| SC-007 | T017, T018 | PROUVE_CIBLE |
| SC-008 | T001, T002, T021 | PROUVE_CUMULATIF |
| US1 | T006, T007, T008, T009 | PROUVE_CIBLE |
| US2 | T003, T004, T010, T011, T012, T013 | PROUVE_CIBLE |
| US3 | T005, T014, T015, T016, T017, T018 | PROUVE_CIBLE |

Les limites ADR011, CLI hors plafond MCP, annulation partielle après commit, reçus256 et mesure SQLite seule sont conservées. 158B reste spec seule. La matrice JSON porte les chemins absolus, lignes, tests, empreintes et limites.
