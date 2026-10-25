# Archives techniques de l’adaptateur T3

Document d’origine : [specs/101-abonnements-t3/implementation.md](../../../../../../specs/101-abonnements-t3/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Audit du jour réutilisé (aucun code Rust modifié dans cette reprise), cycle-adoption
readonly ajouté ; TEST-001 résolu. Grade mécaniqueA99,70, seul QUAL-001 reste
(longueur de project_journal). Limite diagnostique lsof générique exposée dans la
contre-revue, erreur détaillée au log ; aucune anomalie nominale observée.
validate_session.py :0erreur/0warning, exit0 ; fmt et diffcheck à17:07 verts.
Dernier contrôle : abonnnementHorizon toujoursactive, pertesnotifications0,
pertesfaits0, lacunes0 ; daemon/pont vivants, T3 et fournisseurs initiaux inchangés.

<a id="extrait-2"></a>

## Extrait 2

Agent témoin bdgetClaude127bccff-8490-453a-8182-884b749ec41e.
Appelant Codex reconnu via son MCP existant ; témoin Claude via CLI native
(MCP non exposé chez lui, repli déclaré, aucune identité/secret emprunté).
Abonnement témoin97a33158-195e-490e-aed2-aeb8c1879009, propre à bdget,
ponctuel600s ; observation de sa propre fin, aucune modification de T3.

<a id="extrait-3"></a>

## Extrait 3

| Fait attesté | Preuve | UTC2026-09-16 |
|---|---|---|
| Fin du tour témoin | bridget_journal bdget seq232, stop_reason=completed, tour e0e2f0ac-14bd-4902-8391-50ce5f069bbf |15:02:04|
| Notification remise à l'API T3 | log pont, message bridget-observation:ba10f9407d894, séquenceT343313 |15:02:05|
| Notification présente dans le vrai fil | bridget_journal seq233, message natif805def4b-04f2-470b-88f7-50358dfcb790, abonnement exact |15:02:08|
| Réponse témoin confirme réception unique | bridget_journal seq234 |15:02:24|
| Fin du tour système marquée anti-boucle | seq235, message_id bridget-observation:f89ef1e6-4c8f-4f96-9669-829b178f67b3 |15:02:24|

<a id="extrait-4"></a>

## Extrait 4

Nécessité : prouver l'usage, pas seulement annoncer des tests verts. Réutilisation
du binaire, du harnais101 release, des serviceslaunchd et de l'agent témoin existants.
Aucun nouveau service, helper ou dépendance ; archive locale de sauvegarde seulement.
Hypothèses : destinataire disponible pour SC001, limites T3 déjà documentées.
Vérifications : tests debug/release, sauvegardeSQLite, SHA/cmp, PID/naissances avant
et après, propriétaire réel des abonnements, notification présente dans le fil.
Non vérifié : Linux, variante Claude node/bun, suite workspace aux harnais SIGKILL.
Pas de couverture universelle revendiquée. Aucun fournisseur existant arrêté.

<a id="extrait-5"></a>

## Extrait 5

L'utilisateur a autorisé sauvegarde, installation après tests et relance des
seuls services Bridget/pont, sans arrêt de T3/fournisseur, sans commit.
Sync et Analyze réexécutés ; audit-existing PASS confirmé ; checklist9/9.
ETA reprise15–35min, recalibrée15–30min après confirmation de T011 seule ouverte.
Les sections suivantes relatives à «pas d'accord» décrivent la première passe,
pas l'autorité actuelle.

<a id="extrait-6"></a>

## Extrait 6

Démarrage 2026-09-16 15:49 CEST. Branche session-101-abonnements-t3.
ETA initiale62–129min. Après Tasks : 11 tâches, 70–120min restantes.
Pas de commit, pas de déploiement à ce stade.

<a id="extrait-7"></a>

## Extrait 7

MCP Bridget who : identity_not_found. Aucune identité empruntée ni autre route
utilisée pour communiquer. Contre-revue autre fournisseur différée.
Accord asynchrone demandé pour adoption101 après tests/sauvegarde sans arrêt T3.

<a id="extrait-8"></a>

## Extrait 8

| Suite | Résultat |
|---|---|
| daemon --lib spec101, dernière passe après T013 | 31/31, 0 échec, 0,78s |
| daemon --lib spec100 | 14/14 |
| daemon --lib spec099 (inclut tests imbriqués100/101) | 33/33, 4,87s |
| daemon --lib t3code (contrats, identité et adaptateur) | 55/55, 1,32s |
| daemon --lib mcp::tests | 44/44 |
| daemon --lib mcp_identity::tests | 10/10 |
| daemon --lib attach::tests | 110/110 |
| daemon --lib communication::client::security_tests | 13/13 |
| daemon --lib runtime::tests | 12/12 |
| daemon --test spec100_observation_test | 2/2 |
| transport --lib | 281passés, 1ignoré existant |
| core --lib | 39/39 |
| daemon --test spec101_observation_test | 1/1, 6,57s |

<a id="extrait-9"></a>

## Extrait 9

| Exigence | Réalisation et preuve de code (chemins relatifs à ce worktree) | Vérification |
|---|---|---|
| FR001–002 | crates/bridget-daemon/src/t3code_identity.rs:51, :353, :518 ; mcp_identity.rs publication privée | 12 tests identité à partir de t3code_identity.rs:661, contrôles auxiliaires daemon.rs:9034 |
| FR003 | crates/bridget-daemon/src/t3code.rs:1887, :2394 | t3code.rs:2565, :2583, :2620, :2700 |
| FR004 | crates/bridget-daemon/src/t3code.rs:1887, marqueurs de notification et confirmation journal | t3code.rs:2573, :2620, :2671, recette101 |
| FR005 | crates/bridget-daemon/src/t3code.rs:1887, :2730 ; wrapper.rs:2688 ; observation.rs:77 | t3code.rs:2730, :2785 ; wrapper.rs:8478 ; réception après abonnement conservée dans moteur100 |
| FR006 | crates/bridget-daemon/src/daemon.rs:10243 ; observation.rs:192, :247 | daemon.rs:9034, :9160 ; observation.rs:523 |
| FR007 | crates/bridget-daemon/src/observation.rs:118, :192, :238 ; store.rs:48 | observation.rs:562 ; recette101 restart ; daemon.rs:8905 |
| FR008 | crates/bridget-daemon/src/observation.rs:77, :247 ; daemon.rs:1306, :10222 | daemon.rs:8905, :8979 ; observation.rs:493 ; régressions100 |
| FR009 | moteur/journal100 réutilisés ; aucune dépendance T3 ajoutée au cœur | spec100_observation_test2/2 ; spec101_observation_test.rs:252 ; transport281/281 exécutés |
| FR010 | tests isolés réalisés, README/commandes/ADR037 rédigés | Preuve réelle dans un environnement natif isolée : T011 ouverte, SC001 non mesuré |

<a id="extrait-10"></a>

## Extrait 10

Audit v14 mode fix, un cycle puis scoring readonly, limité au diff101.
Répertoire : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3/audits/2026-09-16/session-2026-09-16-spec-101-01
Validation mécanique validate_session.py :0 erreur,0 warning, exit0.
Note mécanique A (99,55), sans autorisation de mise en service : deux MEDIUM
restants, dette de maintenance de project_journal et recette T3 réelle absente.
Aucun CRITICAL/HIGH identifié dans ce périmètre. Ni audit complet du dépôt,
ni scan CVE, ni preuve d'absence de toute anomalie. Aucun commit/déploiement.
Les pertes amont et le délai SQLite ont été corrigés dans T013.

<a id="extrait-11"></a>

## Extrait 11

Nécessité : avertir d'une surveillance lacunaire sans perdre la réactivité demandée.
Réemploi : Observations, Store, JournalLiveFeed, AttachRelayWorker, file de
notifications, identité099, inventaire OS et payload d'écriture existants.
Nouveaux éléments justifiés : module d'identité T3, publication exclusive de
marqueur, table bornée d'abonnements, deux trames capacités/lacunes, recette101
et ADR037. Aucune dépendance ou service nouveau ; arbitrages dans reuse-audit.md.
Pas de bus durable, verrou de fichier, scheduler ou workflow Maicie ajouté.
Hypothèses : limites des données T3 documentées ; fin de tour != succès métier.
Vérifications : tests ci-dessus, sécurité des producteurs, bornes et rollback.
Non vérifié : réception native réelle, variante Claude node/bun et exécution Linux.
La variante Claude native est testée par fixtures, pas par lancement de provider.

<a id="extrait-12"></a>

## Extrait 12

Feature101 ; branche session-101-abonnements-t3 ; 12/13 tâches cochées.
Sync, Specify, Plan, audit-existing, Tasks, Analyze, Implement et Audit exécutés.
Primitives appliquées manuellement si scripts absents ; Converge trois passages,
arrêt honnête sur T011. Contre-revue autre fournisseur indisponible, pas simulée.
Artefacts : spec.md, plan.md, tasks.md, research.md, data-model.md,
contracts/observation.md, quickstart.md, reuse-audit.md, analysis.md,
checklists/requirements.md, implementation.md, scénarios Gherkin et rapports d'audit.
À16:49 CEST :60min écoulées depuis15:49. ETA initiale62–129min (milieu95,5),
après Tasks70–120min restantes. Écart provisoire au milieu initial :−37,2% ;
ce n'est pas un gain final, le pipeline n'est pas terminé. La durée totale et
son écart final restent inconnus jusqu'à l'accord et à la recette réelle T011.

<a id="extrait-13"></a>

## Extrait 13

T011 : aucune installation101, aucun redémarrage de service réel, aucune réception
T3 réelle n'est revendiquée. Accord d'adoption demandé et pas encore reçu.
Code, documentation et vérifications isolées prêts ; statut In Progress.
Main vérifiée propre et inchangée. Pas de commit, merge ou adoption101.
