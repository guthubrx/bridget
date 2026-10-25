# Livraison155 — 2026-10-12

Statut : Livré sur Git, installé et activé ; 42/42 tâches. La clôture documentaire
conserve les preuves sans changer les sources du binaire.

Sources : 59da3968c7d14dbd7acd1370875dd9d135ba7dcb, intégrées en ff-only sur
main et poussées vers github/main. Les 214 sources correspondent aux empreintes
du dernier reçu de tests. Trois passages Converge, puis contrôle cwd ciblé
PASS_TARGETED_CWD ; ce contrôle n’est pas un passage 4.

Validation : 71 PASS post-cwd, RC0, 64,81 s, 214 sources stables. Quatre régressions
canal087/STDIO restent distinctes et antérieures à cwd. T040 est validée après
accord explicite utilisateur, avec quatre cas réels PASS. Le Human a été exécuté
après H1/H2/H3, avant cwd ; les trois autres captures sont réutilisées et gardent
leur ancien binaire. Les quatre cas ne sont pas annoncés sur le dernier binaire.
La suite complète UNIQUE antérieure garde 2259 PASS / 15 FAIL / 72 ignored,
554,43 s et 180/180 baseline PASS. Sept nouveaux échecs ont leurs replays PASS ;
huit échecs historiques restent distincts. Aucune seconde suite ni GREEN global.

Audit final : A, 0 CRITICAL / 0 HIGH / 0 MEDIUM / 2 LOW ; couverture 27,46 %
(67/244 hunks). Validateur : 0 erreur / 0 warning selon le reçu du principal.
Les LOW PERF-082/DUP-001 restent acceptés. Cette couverture partielle ne prouve
pas une lecture complète. Un canal hostile qui ignore SIGTERM peut survivre ;
aucune terminaison inconditionnelle n’est revendiquée.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/155-demandes-enfant/validation/livraison155.md#extrait-1).

WT/branche 155 retirés proprement à 05:49 CEST ; 81 fichiers ignorés archivés
avec empreintes vérifiées. Aucun worktree de session ne reste. Intermédiaires :
0,29 Gio logiques retirés, aucun gain physique déduit. DB de production et
sidecars, DB privées de recette, rollback, binaire final, archives et cache Cargo
utile 156–160 conservés. Archive privée :
/Users/moi/.cache/bridget-delivery155.um4xccv4/worktree155-ignored.
Rollback : /Users/moi/.cache/bridget-delivery155.um4xccv4/rollback-t000155-bridget.

Reçus canoniques :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/git155.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/build155.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/install155.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/activation155.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/cleanup155.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/native155-cwd-r1.json.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/recette155-postfix-final.json (SHA256 153b70cbbbaa94315b87f192a8fa67fc217f607eb1cc70c5e9ea5b047b6cc9f3).
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/audit-delta155-cwd.json.

Les chemins de worktree et anciens statuts des artefacts précédents sont
historiques. Source livrée 59da3968… et commits documentaires sont distincts.
Aucun code, tâche, test, Git, build ou service n’a été modifié par ce rédacteur.
