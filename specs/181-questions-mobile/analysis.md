# Analyze181 — protocole manuel des skills

Exécuté dans ce tour le25octobre2026, après génération de tasks.md. La primitive shell a été tentée : .specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks -> exit127, No such file or directory. Skill speckit-analyze lue ; comparaison directe spec/plan/tasks/contrats et relecture après corrections. Aucun contrôle outillé inexistant revendiqué.

| ID | Sévérité | Finding | Résolution vérifiée |
|---|---|---|---|
| A1 | HIGH | « sans processus T3 » pouvait imposer l’arrêt de l’application utilisateur | Exigence précisée : environnement de recette sans composant/connexion T3, hôte utilisateur préservé ; T018/T020 |
| A2 | HIGH | Le nouvel outil MCP ne doit pas donner un détour à une preuve enfant155 | Plan précise refus aux preuves déléguées et réemploi du principal humain uniquement pour répondre ; T004/T005/T006 |
| A3 | MEDIUM | Notification complexe et action directe non distinguées | Plan précise une demande simple compatible RemoteInput, ouverture du formulaire sinon ; T015/T016/T019 |
| A4 | MEDIUM | Catalogue des anciens clients MCP et code installé pouvaient être confondus | Plan distingue nouveaux lancements et processus chargés ; T020/T021 et reçu final |

Couverture : FR001/002/008/009 -> T003–T010/T021 ; FR003/004 -> T011–T013/T017/T018 ; FR005/010 -> T004/T009/T013/T017/T021 ; FR006 -> T011/T014/T017/T019 ; FR007 -> T015/T016/T019 ; FR011 -> T005–T009/T018/T020 ; FR012 -> T006/T007/T010/T021 ; FR013 -> T002/T017–T022. SC001–006 sont couverts par T017–T022 et livraison.

Relecture2 : aucun CRITICAL persistant, aucun marqueur d’arbitrage produit ;22tâches au format strict,2faites,20restantes. Gate avant tasks PASS, aucune duplication évidente ; aucun framework/service/dépendance externe prévu. Les primitives manquantes sont nommées, sans reconstitution du runtime projet dans ce lot.
