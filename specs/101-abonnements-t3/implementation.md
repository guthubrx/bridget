# Implémentation101 — Implemented, non commité

## Complément documentaire après adoption — 2026-09-16

Sur demande utilisateur, publication de la documentation101 dans l'arbre
principal : README, skill Bridget, référence des commandes et ADR037. Les trois
installations Codex/Claude/Agents pointent déjà vers cette source par symlink ;
elles ne pointaient pas vers le worktree101. Les quatre documents sont désormais
identiques dans les deux arbres. Main porte donc ces changements documentaires
non commités ; les mentions « main propre » ci-dessous sont historiques.

Ajouts : choix de l'outil selon la demande, recettes MCP pour abonnement,
prolongation par remplacement non atomique, annulation, partage/relecture et
alertes de fichiers ; expiration sans rappel, remise incertaine et absence de
réponse automatique aux notifications. Pas de nouvelle commande ni de workflow.

Vérifications : quick_validate.py réussi sur les trois installations ; 20
exemples JSON vérifiés contre les schémas du catalogue du binaire installé
(MCP temporaire isolé, initialize/tools-list seulement, aucun tools/call) ;
identité des quatre documents et résolution des six liens vérifiées ;
git diff --check réussi dans les deux arbres. Paramètres et reçus recoupés avec
le code MCP/observation/journal101. Ces contrôles documentaires ne sont pas une
nouvelle recette de livraison réelle. Aucun code Rust, binaire, service,
abonnement ou configuration fournisseur modifié, aucun commit ni merge.
Les processus ayant déjà chargé l'ancienne skill doivent la relire ; aucune
relance n'est nécessaire pour lire ces fichiers.

## Résultat final de la reprise

13/13tâches vérifiées, T011 close après réception réelle. Les sections plus bas
conservent le déroulé des passes antérieures et leurs blocages alors réels.
Version101 en service, main propre/inchangée, aucun commit ni merge automatique.

Converge reprise : CONVERGED au premier passage, zéro tâche ajoutée. Spec/plan/
tasks/checklist et réalisations réexaminés ; mappingFR001–009 de la première
passe inchangé, FR010 désormais appuyé par la recette ci-dessous. tasks.md
strictement inchangé pendant ce passage : SHA256 avant/après
567cedaf10906013ba97a1fd042fbc8fb73355a70aa2b7be1af8829afeab034f.
Historique : trois passages lors de l'implémentation initiale, arrêt explicite
sur autorisation ; un passage de reprise après levée de ce blocage et recette.

Audit du jour réutilisé (aucun code Rust modifié dans cette reprise), cycle-adoption
readonly ajouté ; TEST-001 résolu. Grade mécaniqueA99,70, seul QUAL-001 reste
(longueur de project_journal). Limite diagnostique lsof générique exposée dans la
contre-revue, erreur détaillée au log ; aucune anomalie nominale observée.
validate_session.py :0erreur/0warning, exit0 ; fmt et diffcheck à17:07 verts.
Dernier contrôle : abonnnementHorizon toujoursactive, pertesnotifications0,
pertesfaits0, lacunes0 ; daemon/pont vivants, T3 et fournisseurs initiaux inchangés.

Pipeline de reprise : Specify/Plan/Tasks conservés ; sync, audit-existing,
Analyze, T011, contre-revue, Converge exécutés ; audit complet non relancé car
audit du jour existant, preuve/score actualisés. Scripts SpecKit manquants :
primitives lues puis protocole manuel, sans réinstallation. Aucun nouveau code.
Durée reprise16:52–17:07 :15min ; ETA initiale15–35min, recalibrée15–30min après
constat d'une tâche restante. Écart au milieu initial25min :−40%, notamment
grâce au cache de compilation et à la recette témoin disponible. Aucun dépassement.
Durée depuis démarrage initial15:49 :78min calendaires (inclut pause d'accord),
contre62–129min initialement :−18,3% au milieu95,5min. Estimation intermédiaire
après Tasks70–120min restantes rappelée dans le déroulé, pas de mesure unique
de durée cumulée hors pause. Première tâche non cochée restante :aucune.
Prochaine action : relecture du diff puis commit/merge sur décision humaine,
pas de commit automatique. Sauvegarde et archive sources permettent le retour arrière.

### Preuve FR010/SC001 — vrai fil T3

Agent témoin bdgetClaude127bccff-8490-453a-8182-884b749ec41e.
Appelant Codex reconnu via son MCP existant ; témoin Claude via CLI native
(MCP non exposé chez lui, repli déclaré, aucune identité/secret emprunté).
Abonnement témoin97a33158-195e-490e-aed2-aeb8c1879009, propre à bdget,
ponctuel600s ; observation de sa propre fin, aucune modification de T3.

| Fait attesté | Preuve | UTC2026-09-16 |
|---|---|---|
| Fin du tour témoin | bridget_journal bdget seq232, stop_reason=completed, tour e0e2f0ac-14bd-4902-8391-50ce5f069bbf |15:02:04|
| Notification remise à l'API T3 | log pont, message bridget-observation:ba10f9407d894, séquenceT343313 |15:02:05|
| Notification présente dans le vrai fil | bridget_journal seq233, message natif805def4b-04f2-470b-88f7-50358dfcb790, abonnement exact |15:02:08|
| Réponse témoin confirme réception unique | bridget_journal seq234 |15:02:24|
| Fin du tour système marquée anti-boucle | seq235, message_id bridget-observation:f89ef1e6-4c8f-4f96-9669-829b178f67b3 |15:02:24|

Délai mesuré après observation :1s jusqu'à remise,4s jusqu'à présence confirmée
par le journal ; inférieur10s pour ce témoin disponible. Une mesure, pas un SLO
statistique ni garantie absolue. L'abonnement témoin a disparu (once consommé).
La garde anti-boucle est prouvée par le marqueur réel et les tests du collecteur ;
la simple absence d'une seconde notification once ne suffirait pas à la prouver.
Pas de contenu de conversations privées archivé ici, seulement références de recette.

Horizon-3D : abonnement7134abe1-8c4a-486f-8aa7-d9996b4fa917 actif, once24h,
expiration2026-09-17 16:57:59CEST. Fin de tour != fin de projet. Une réception
Horizon future reste distincte de la recette témoin ; aucune fin inventée.

### Self-review T011 (Articles XIX/XX)

Nécessité : prouver l'usage, pas seulement annoncer des tests verts. Réutilisation
du binaire, du harnais101 release, des serviceslaunchd et de l'agent témoin existants.
Aucun nouveau service, helper ou dépendance ; archive locale de sauvegarde seulement.
Hypothèses : destinataire disponible pour SC001, limites T3 déjà documentées.
Vérifications : tests debug/release, sauvegardeSQLite, SHA/cmp, PID/naissances avant
et après, propriétaire réel des abonnements, notification présente dans le fil.
Non vérifié : Linux, variante Claude node/bun, suite workspace aux harnais SIGKILL.
Pas de couverture universelle revendiquée. Aucun fournisseur existant arrêté.

Contre-revue adverse Claude : APPROVE_WITH_CHANGES ; quatre objections vérifiées,
traitement détaillé dans adversarial-review-bdget.md. Commit rejeté conformément
à l'accord/skill ; confusion Codex/Claude réfutée ; diagnostic lsof générique
documenté ; manque de recette maintenant résolu. Aucun changement de code requis.

Sources de la version conservées pour retrouver le diff non commité :
/Users/moi/.cache/bridget-adoptions/101-20260916.uO85Gk/source101.tgz
SHA256 :9c3c427106f4bfbe9d18ef85f25295046ed49c5ef2b50f794eef1adce1842a10.
Archive avant clôture des documents : sources exécutables identiques, aucune
modification Rust depuis la compilation testée. Binaire installé décrit ci-dessous.

## Reprise du16septembre à16:52 CEST — adoption autorisée

L'utilisateur a autorisé sauvegarde, installation après tests et relance des
seuls services Bridget/pont, sans arrêt de T3/fournisseur, sans commit.
Sync et Analyze réexécutés ; audit-existing PASS confirmé ; checklist9/9.
ETA reprise15–35min, recalibrée15–30min après confirmation de T011 seule ouverte.
Les sections suivantes relatives à «pas d'accord» décrivent la première passe,
pas l'autorité actuelle.

Revalidation :31/31tests101 et recette privée1/1(6,51s). Compilation optimisée
séparée34,88s ; même recette sur binairerelease1/1(6,43s). Copie du cache par
clone APFS, aucun remplacement prématuré du binaire installé.
Sauvegarde privée contrôlée (SQLitequick_check=ok), installation atomique,
TERM individuels vérifiés et relance launchd des seuls services18471/19624.
T3application33910/serveur33979, fournisseurs2344/5315/6697/87537 inchangés.
Build installée1738a072ba22-dirty ; SHA256
e5ededb628287e47d4d11f52ffdbeabbc334f7d03bc45c208261152434a88287.
Empreinte du binaire final reconstruit par cargo test --release ; cmp confirme
que la version installée est exactement celle de cette recette.
Preuve détaillée et retour arrière :
/Users/moi/.cache/bridget-adoptions/101-20260916.uO85Gk/receipt.md

MCPwho et events types réussissent depuis le MCP préexistant ; aucun fournisseur
redémarré. Propriétaire réel confirméb280f81d-a418-4dbc-bfd8-35c50d8fedd1.
AbonnementHorizon once24h confirméactive :7134abe1-8c4a-486f-8aa7-d9996b4fa917.
La réception futureHorizon n'est pas encore revendiquée. Contre-revueClaude et
auto-abonnement témoin demandés à bdget ; réception du messagemcp-6800-6aaaae77-1
confirmée dans le ledger, preuve de notification attendue avant de cocherT011.

Démarrage 2026-09-16 15:49 CEST. Branche session-101-abonnements-t3.
ETA initiale62–129min. Après Tasks : 11 tâches, 70–120min restantes.
Pas de commit, pas de déploiement à ce stade.

## Préparation

Sync, Specify, Plan, audit-existing PASS, Tasks et Analyze exécutés.
Checklists exigences : 9/9 ; gate réutilisation : 5/5.
T001 : huit scénarios métier écrits, mapping des FR dans analysis.md relu.
Self-review : besoin demandé, conventions existantes réutilisées ; pas de code
produit ni de test technique déclaré passé à ce stade.

## Autorité et accès

MCP Bridget who : identity_not_found. Aucune identité empruntée ni autre route
utilisée pour communiquer. Contre-revue autre fournisseur différée.
Accord asynchrone demandé pour adoption101 après tests/sauvegarde sans arrêt T3.

## Non disponible

Scripts/templates SpecKit absents ; protocole des primitives appliqué manuellement.
mem et Sequential Thinking absents ; décisions tracées dans research.md.
Mémoire projet conventionnelle absente ; specs098–100 utilisées.

## Vérifications isolées obtenues

Commande commune : env -u BRIDGET_HOME -u BRIDGET_SOCKET TMPDIR=/tmp
CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target CARGO_INCREMENTAL=0
cargo test. Tests lib daemon exécutés avec --test-threads=1.

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

La recette101 emploie un vrai daemon privé, preuves099, journal et conversion
réels, pompage socket explicite (pas le worker de relais privé), partageCLI,
collision, absence de boucle, perte/reprise, restartinterrupted. Aucun provider.
TestHTTP supplémentaire : refresh lit un serveur local contrôlé, annonce les
capacitésCodex/Claude correctes et une lacune non quantifiée, sans fausse
déconnexion/reconnexion ni consommation d'un abonnement once.

Premier clippy a signalé nonminimal_bool (corrigé) ; passes suivantes vertes.
Dernière validation après T013 : cargo clippy --workspace --all-targets -- -D warnings
verte, 14,43s ; cargo fmt --all -- --check et git diff --check verts.
Les filtres de tests se recouvrent : leurs nombres ne représentent pas des tests
uniques additionnables. Couverture des lignes non mesurée.
Premier lot099 : 14échecs de fixtures «socket trop longue» avec TMPDIRmacOS.
Donnée réelle : préfixe /var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/.
Relance avec TMPDIR=/tmp et sérialisation des fixtures HOME : 31/31.
Premier test101 : log privé créé0644, refus préflight attendu ; harnais corrigé0600.
Pas de modification de la sécurité de production pour passer ces tests.
Workspace complet non lancé : anciens harnais à SIGKILL/groupe exclus selon
instruction processus. macOS testé ; chemin OS Linux non exécuté.

## Relecture et convergence

Convergence1 : une tâche T012 ajoutée, deux écarts identité confirmés par relecture
indépendante : aliasclaudeAgent et ambiguïté masquée par rolloutillisible.
T012 corrigée, 12/12 tests identité ; incluse dans les 31/31 finaux101.
MCP who retenté : toujours identity_not_found sur version100 installée.
Aucun autre fournisseur joignable par le canalautorisé ; relecture mêmefournisseur,
pas équivalente à la contre-revue inter-fournisseurs.
ETA recalibrée après convergence : 25–50min restantes, hors accord/recetteproduction.

Convergence2 : T013 ajoutée pour lacunes de la file de faits et attente SQLite
sous verrou partagé. Corrigée et validée par cinq tests supplémentaires : vraie
saturation300→256+44 ; filtrage/régulation ; primaire/DND/compteurs ; contention
réelle entre deux connexions ; rollback création et consommation once. Le délai
SQLite global est restauré après la sauvegarde à délai nul. La lacune ne consomme
pas once et ne se transforme jamais en faux événement de fin de tour.
ETA recalibrée après T013 :30–55min restantes, hors accord/recette réelle.

Convergence3 : aucun nouveau manque technique identifié sur FR001–009 ; FR010
et SC001 restent sans preuve réelle. Pas de tâche doublon pour T011 déjà ouverte.
Issue globale : NON CONVERGED / In Progress après trois passages. Aucun
Implemented déclaré. Détail des réalisations/preuves ci-dessous.

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
| FR010 | tests isolés réalisés, README/commandes/ADR037 rédigés | Preuve réelle T3 absente : T011 ouverte, SC001 non mesuré |

## Audit final

Audit v14 mode fix, un cycle puis scoring readonly, limité au diff101.
Répertoire : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3/audits/2026-09-16/session-2026-09-16-spec-101-01
Validation mécanique validate_session.py :0 erreur,0 warning, exit0.
Note mécanique A (99,55), sans autorisation de mise en service : deux MEDIUM
restants, dette de maintenance de project_journal et recette T3 réelle absente.
Aucun CRITICAL/HIGH identifié dans ce périmètre. Ni audit complet du dépôt,
ni scan CVE, ni preuve d'absence de toute anomalie. Aucun commit/déploiement.
Les pertes amont et le délai SQLite ont été corrigés dans T013.

## Réemploi et self-review avant clôture T010/T013

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

## Synthèse du pipeline et durée

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

## Ce qui n'est pas encore prouvé

T011 : aucune installation101, aucun redémarrage de service réel, aucune réception
T3 réelle n'est revendiquée. Accord d'adoption demandé et pas encore reçu.
Code, documentation et vérifications isolées prêts ; statut In Progress.
Main vérifiée propre et inchangée. Pas de commit, merge ou adoption101.
