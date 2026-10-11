# Candidat154 — passage R4 et cochage autorisé

R4 : 11,59 s, RC101, sources stables avant/après. Finale : 4 PASS, 6 FAIL.
Les six résumés helpers 0 PASS/1 FAIL ne sont pas additionnés.
Reçu : /Users/moi/.cache/bridget-delivery154.l50umk90/candidat154-green-r4.json
Journal : /Users/moi/.cache/bridget-delivery154.l50umk90/candidat154-green-r4.log
SHA256 : `b8a84a01fc53723c083e9ce44bd3d103549ff13df5430d970dcbd7be3216b1de`.

Quatre oracles PASS :
- `daemon::native_delegation::native_workspace154_tests::native154_conservation_retention_150_active_kept_et_rejeu_expire`
- `daemon::native_delegation::native_workspace154_tests::native154_creation_quarante_appels_trois_proprietaires`
- `daemon::native_delegation::native_workspace154_tests::native154_isolation_petit_enfant_et_conservation_emboitee`
- `workspace_git::tests::native154_git_rapport_zero_deux_soixante_commits_et_tete_detachee`

La quatrième preuve inclut désormais le changement de branche obligatoire T014,
ajouté au test direct existant, sans scénario de confort nouveau. Le mapping
fermé et le plafond des 40 appels sont éprouvés dans ce passage.

Les six FAIL viennent de la fixture : BRIDGET_HOME omis après env_clear.
Preuves relues : environment.rs:21-55 utilise HOME/.cache/bridget-core si cette
variable manque ; daemon.rs:20748-20797 impose registry_home à HOME. La socket
privée devient alors trop longue. L’owner des tests corrige la recette privée ;
R5 rejouera seulement ces six tests. Aucun défaut de production déduit de cela.

GO principal reçu : cochages proposés R3 désormais autorisés T005/T006/T010/
T013/T014/T015/T016/T020. T012 acquise par R3 paires/parent intact et R4 emboîté.
T019 acquise par purge R4. T023/T024 acquises après correction documentaire des
mots collés ; noms de clés/codes et blocs de commandes conservés.
Total : 19/28 tâches acquises, 9 ouvertes. Aucun GREEN global ni livraison.
T007/T008/T009/T017/T018/T025/T026/T027/T028 restent ouvertes. T026 est préparée
mais attend la suite et la revue finale. Aucun cochage au-delà du GO.

Self-review : hash du log recalculé, stabilité sources comparée, quatre noms
PASS exacts et six helpers vérifiés, règles d’environnement relues. Aucun test,
Git, Cargo ou service exécuté par l’owner documentaire.
