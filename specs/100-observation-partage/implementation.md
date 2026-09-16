# Journal d'implémentation 100

Statut : Implemented — validation ciblée terminée, non déployé. Début 2026-09-16 à 09:10 CEST.
ETA initiale 72–189 min, fin haute 12:19 CEST, hors arbitrage/quota/déploiement.

## Socle protégé

Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/100-observation-partage
Branche : session-100-observation-partage, HEAD initial 9739fc84.
Les fichiers modifiés de 099 ont été copiés dans le worktree, sans commit ni
modification du dossier source. SHA256 du diff suivi initial dans le worktree :
fe9040518bc01401601994eaef417ad66310491accad579a7c4869e7feead367.
Copiés aussi : spec099, ADR035, Gherkin099. Les autres worktrees ne sont pas nettoyés.

## Préflight

- Synchronisation exécutée : pont constitution/standards déjà à jour.
- Progression exécutée : 099 = 16/16, 098 = 19/19.
- Scripts et modèles .specify absents (ls/lecture effectifs) : protocoles des
  skills appliqués manuellement, sans prétendre qu'une commande absente a tourné.
- Mémoire projet aux deux emplacements documentés absente ; binaire mem absent.
- Aucun outil Sequential Thinking exposé ; décisions et alternatives dans research.md.
- bridget who exécuté : Claude « bdget » joignable ; contre-revue à demander après plan.
- Disque : 3,6 Gio libres lors du contrôle, target existant 24 Gio. Aucun nettoyage
  non autorisé ; builds ciblés avec surveillance d'espace avant suite étendue.
- Installation historique non modifiée, aucun daemon/fournisseur lancé.

## Exécution et décisions finales

ETA recalibrée après Tasks : 80–165 min, borne haute 11:55 CEST.
Les 13 tâches couvrent les trois usages, sans tranche MVP substituée au plan.
Phases Specify/Plan/Reuse/Tasks/Analyze appliquées ; Analyze relu après code.
Primitives de scripts absentes : protocoles manuels documentés, pas d'exécution
fictive. Aucun outil de convergence disponible : confrontation manuelle exigée.

### Résultats vérifiés

Toutes les commandes suivantes ont été exécutées depuis le worktree100 avec :
TMPDIR=/tmp/bg100-validation.tusAkR,
CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target,
CARGO_INCREMENTAL=0. Binaire cargo : /Users/moi/.cargo/bin/cargo.

| Commande (arguments de cargo) | Résultat final |
|---|---|
| test -p bridget-daemon --lib spec100 -- --test-threads=1 | 12/12 |
| test -p bridget-transport --lib spec100 -- --test-threads=1 | 6/6 |
| test -p bridget-daemon --test spec100_observation_test | 2/2 |
| test -p bridget-daemon --lib attach::tests -- --test-threads=1 | 110/110 |
| test -p bridget-daemon --lib mcp::tests -- --test-threads=1 | 44/44 |
| test -p bridget-daemon --lib spec099 -- --test-threads=1 | 26/26 (inclut le témoin100 du module099) |
| test -p bridget-transport --lib journal::tests -- --test-threads=1 | 17/17 |
| test -p bridget-transport --lib protocol::tests -- --test-threads=1 | 62/62 |
| test -p bridget-core --lib -- --test-threads=1 | 39/39 |
| test -p bridget-daemon --lib spec091_politique_mcp_reste_octet_pour_octet_stable | 1/1 |
| test -p bridget-daemon --lib spec094_codex_autorise_exactement_les_outils_de_communication_surs | 1/1 |
| test -p bridget-transport --lib lecteur_interactif_ne_repond_jamais_pour_humain_et_conserve_raw | 1/1 ; tardif/échec/doublon Codex exclus |
| fmt --all -- --check | PASS après dernière modification de code |
| clippy --workspace --all-targets -- -D warnings | PASS après dernière modification de code |
| test --workspace | BLOQUÉ avant lancement : harnais SIGKILL de groupes interdits |

Preuves du blocage de la dernière commande : support/idempotent.rs:88,277,286,472 ;
core_089_crash_test.rs:3 ; claude_stream_json.rs:713. Les suites ciblées ne
remplacent pas une exécution globale, ni une recette avec les fournisseurs réels.
Aucun fournisseur, daemon externe, redémarrage historique ou déploiement effectué.
Les scénarios Gherkin sont reliés aux tests Rust ; aucun runner Gherkin distinct
n'est annoncé exécuté. Certains filtres se recoupent : ne pas additionner les
comptes du tableau comme des tests uniques.

### Échecs rencontrés et corrigés

- Phase rouge initiale : JournalExcerpt absent, puis erreurs de compilation de
  raccordement ; implémentation ajoutée et compilation vérifiée.
- Champs étrangers acceptés par variantes serde unitaires : remplacées par
  variantes à objet fermé, test owner arbitraire rouge puis vert.
- Catalogue MCP/inventaire : 2 échecs sur 44 après ajout des outils ; compte18,
  politique fournisseur14 et documentation synchronisés ; 44/44 puis politique verte.
- Régressions099 : 11 échecs initiaux dus à des chemins de socket macOS trop longs ;
  TMPDIR court, même code testé : 26/26.
- CLI simulé : socket initialement trop permissive ; stderr privé lu, permissions
  0600 puis accept borné dans le témoin défaillant ; 2/2. Le thread accept initial
  a été libéré par connexion locale, sans kill.
- Clippy : deux simplifications locales appliquées ; dernier passage sans warning.
- Revue : drain de faits explicitement limité à256 même si le producteur continue ;
  cache ACP saturé conserve les marqueurs déjà signalés au lieu de les oublier,
  renouvelé à changement de tour ; test saturation vert. Aucune troisième
  correction aveugle sur la même hypothèse.

### Self-review Articles XIX/XX

Nécessité : extrait exact, quatre faits observables, collisions non bloquantes.
Nouveau code métier : observation.rs (filtrage/TTL/déduplication/collision) ;
un test d'intégration100, contrats typés et métadonnées partagées. Aucun package,
service, table ou migration ajouté. Aucune nouvelle couche d'orchestration.

Réutilisé : assemblage attach, budget DaemonConnection, send/reply, identité099,
journal post-flush, relais wrapper et message ordinaire. La file de faits
indépendante est justifiée par le drain conditionnel de la file attach, pas
par un besoin futur imaginaire. Deux aides de chemin servent les trois pilotes.
Arbitrages tracés dans reuse-audit.md avant création.

Vérifié : données Unicode, 50/200 entrées, lacune, délai10s serveur muet,
parité des façades, refus owner/source non autorisés, TTL/once/filtres,
collision et quatre témoins négatifs, saturation, reply et annuaire pendant
writer verrouillé, notification exclue des observations suivantes.
Non vérifié : fournisseurs réels, T3 natif universel, performances production,
fédération des faits, reprise durable après redémarrage et suite globale.
Ce ne sont pas des promesses cachées : couverture et pertes sont documentées.
Code évité : second lecteur JSONL, watcher récursif, moteur de règles libre,
BDD d'abonnements, boucle LLM, mandat et verrou de fichiers.

### Revue externe

Claude « bdget » détecté au préflight ; demande non remise : MCP identity_not_found
et CLI UUID v4 requis. Reprise post-implémentation : même refus MCP d'identité.
Pas de verdict externe ni d'objection inventée ; aucune usurpation pour contourner.
Voir adversarial-review-claude.md. La self-review n'est pas équivalente.

## Converge — CONVERGED, un passage

Après la dernière modification de code : lecture spec/plan/tasks/checklist,
diff100 et dépendances directes ; exigences confrontées aux réalisations et tests
ci-dessous. Aucune tâche ajoutée. Pendant cette phase, aucune modification de
code ou d'artefact ; SHA256 tasks avant/après identique :
9384e1633564d14acf4d4f30822401938f26e94333e1cc1fe03cc41303cdbaf1.
Cette trace est écrite après la phase ; T013 ne sera coché qu'après audit validé.

Chemins ci-dessous relatifs à la racine absolue indiquée en tête de ce document.

| Exigence | Réalisation | Preuve exécutée |
|---|---|---|
| FR01,SC01 | attach.rs:44,138 ; cli.rs:663 ; mcp.rs:460 | attach.rs:5771 (50/200), mcp.rs:2535 et tests/spec100_observation_test.rs:13,126 |
| FR02 | attach.rs:68,96,104,138 ; AttachClientState existant | attach.rs:5771,5799,5807,5876 : Unicode fragmenté, séquences, limite, erreur et budget |
| FR03 | cli.rs:713 ; mcp.rs:464 réutilisent send | mcp.rs:2535 ; daemon.rs:8846 préserve pending_replies sous pression |
| FR04 | protocol.rs:2118,2127 ; observation.rs:51,227 | cli.rs:770 ; observation.rs:316,362 ; erreur CLI événement inconnu |
| FR05,SC04 | observation.rs:51,109,126 : purge TTL, retrait once, suppression | observation.rs:269,316 ; owner/list/unsub/expiration/once |
| FR06 | daemon.rs:9769,9795 ; identité099 live_connection_identity:7285 | daemon.rs:8846, régressions099 identité et preuve auxiliaire |
| FR07 | daemon.rs:1280,1305 ; journal.rs:533 ; wrapper.rs:2687 | daemon.rs:8846 writer verrouillé ; journal.rs:1176 anti-boucle ; wrapper.rs:8457 relais sans attach |
| FR08 | journal.rs:549 : seuls turn_end explicites ; observation.rs:51 catalogue/notice | journal.rs:1176 : cancelled reste fin de tour, natif non corrélé exclu |
| FR09,SC05 | observation.rs:126,207 ; daemon.rs:9792 | observation.rs:297 hôte/auteur/chemin/fenêtre ; daemon.rs:8846 livraison et reply témoin |
| FR10,SC03 | journal.rs:577,595 ; acp.rs:1790 ; claude_stream_json.rs:1350 ; codex_app_server.rs:2848 | acp.rs:2078,2119 ; claude_stream_json.rs:1515 ; journal.rs:1123 ; lecteur Codex testé |
| FR11 | attach.rs:104 ; observation.rs:51,126 ; journal.rs:191 ; daemon.rs:1280 | bornes200/64Kio, abonnements128, cache4096, débit, queue et writer lent vérifiés |
| FR12 | README.md:424 ; README.en.md:395 ; contracts/observation.md ; ADR036 | relecture des reçus réels et des exemples ; couverture, DND, pertes et durée mémoire explicites |
| FR13 | worktree100, tests privés, send099 inchangé | régressions099 26/26 ; aucun service installé modifié ; suite globale bloquée explicitement |
| SC02 | observation.rs:51 répond sans attendre un fait ; daemon.rs:9767 | fixture daemon100 entière exécutée sous1s, test de communication témoin impose <1s |
| SC06 | frontières CLI/MCP/daemon, writer/producteurs | 20 tests ciblés, suites détaillées ci-dessus, fmt/clippy PASS |

Le contrat de vérification autorise une commande marquée bloquée avec sa preuve :
T012 clôt la vérification possible, sans présenter test --workspace comme exécuté.

## Audit initial — avant demande de fusion et installation

Audit v14 mode fix, un cycle puis scoring readonly, périmètre diff100 (14 fichiers).
Aucun finding résiduel confirmé ; note A limitée à ce diff, pas au projet entier.
jscpd final : 424 paires, 4453/80599 lignes, 5,52% sur les fichiers complets ;
11 paires touchant les changements, petits préparatifs de tests, pas de doublon
métier confirmé. Aucun fingerprint à produire en absence de finding.
Validation déterministe : 0 erreur, 0 warning, exit0.

Session d'audit :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/100-observation-partage/audits/2026-09-16/session-2026-09-16-spec-100-01
Ce répertoire suit l'exclusion Git audits/ existante ; il reste consultable
localement et n'est pas ajouté de force au suivi.

Au bilan initial de 10:22 : 13/13 tâches closes, aucune tâche de développement restante. Analyze manuel exécuté et relu ;
Converge CONVERGED un passage ; revue adverse tentée mais non remise ; audit
terminé. Pas de commit automatique, ni fusion, installation ou redémarrage.
Branche session-100-observation-partage ; diff non commité à revoir avant intégration.
Le diff Git brut comprend le socle099 copié ; comparer au099 non commité pour
isoler100 et ne pas traiter les modifications héritées comme de nouveaux travaux.

Temps final relevé : environ72 min (09:10–10:22 CEST). ETA initiale72–189 min ;
recalibrée80–165 min. Écart environ−41% par rapport au milieu122,5 min de l'ETA
recalibrée, environ−10% sous sa borne basse. Cause principale : assemblage/send
et identités réutilisés ; validations locales ciblées rapides, suite intégrale
non exécutée pour le motif de sécurité documenté. Ce gain ne prouve pas une
validation fournisseur de production. Attentes externes et déploiement hors ETA.

Prochaine action recommandée : revue du diff puis recette avec fournisseurs
dans un namespace isolé avant toute installation. La couverture native/T3
universelle et l'adaptation des anciens harnais sont des travaux séparés.

## Intégration et recette de livraison du 2026-09-16

Demande explicite : finaliser, fusionner et rendre la nouvelle version utilisable.
La 099 finale est déjà dans main (e1b83e95). Aucun remote Git n'est configuré :
une fusion locale de la branche100 suffit, aucune PR distante n'est inventée.
L'ancien travail est sauvegardé dans le stash nommé
`sauvegarde-session-100-avant-integration-099`, conservé après application.
La branche100 a avancé sur main ; les correctifs humains/annuaire et fixtures099
ont été conservés lors de la résolution des conflits. Aucun changement100
ne remplace les dernières corrections099.

Deux relectures indépendantes, autorisées par l'utilisateur, ont contrôlé la
couverture et les exigences. Un défaut ACP a été corrigé : conserver le type
edit entre trames et le chemin fourni par une permission avant réussite outil.
Une régression couvre ces deux ordres et les doublons. Deux tests supplémentaires
prouvent la saturation réelle de la file64, son refus non bloquant, sa vidange
et la perte d'une notification expirée sans bloquer la suivante.
La contre-relecture confirme ces corrections. Couverture estimée par lecture
90 %, pas une mesure instrumentée ; aucune affirmation de couverture à 100 %.
Le volume ajouté sert les trois usages ; aucun service/dépendance/framework
nouveau. La preuve d'identité099, le lecteur attach et l'envoi existant sont réutilisés.

Recette sur le socle intégré, TMPDIR court privé et CARGO_INCREMENTAL=0 :

| Suite | Résultat |
|---|---|
| daemon --lib spec100, après corrections | 14/14 |
| transport --lib spec100, après corrections | 7/7 |
| daemon --test spec100_observation_test | 2/2 |
| daemon --lib spec099 | 27/27 avant ajout des deux tests de sortie100 |
| daemon --lib mcp::tests | 44/44 |
| daemon --lib attach::tests | 110/110 |
| daemon --lib communication::client::security_tests | 13/13 |
| politiques MCP091/094 | 1/1 chacune |
| transport journal::tests / protocol::tests | 17/17 et 62/62 |
| lecteur Codex interactif avec écritures100 | 1/1 |
| core --lib | 39/39 |
| fmt et clippy workspace all-targets -D warnings | PASS |

Le premier lancement avait deux erreurs de préparation : BRIDGET_SOCKET ne
correspondait pas au home des fixtures unitaires, et une preuve auxiliaire
était lue deux fois alors que le helper099 la traite déjà. Préparation corrigée,
sans affaiblir le contrat ni modifier le chemin d'envoi ; relance verte.
Le filtre communication::client::tests ne sélectionnait aucun test : remplacé
par security_tests, 13 tests effectivement exécutés, pas un zéro-test compté PASS.

Smoke sur vrai daemon isolé : catalogue CLI, abonnement once puis notification
reçue, collision reçue entre deux identités, désabonnement, journal fragmenté
transmis avec provenance/reply, catalogue MCP contenant les deux nouveaux outils.
Dernier état privé : /tmp/b100.20260916-71764-1f5cz4a. Daemon arrêté par SIGTERM
individuel après contrôle PID/parent/commande ; aucun fournisseur lancé.
Script reproductible conservé dans la sauvegarde de livraison :
/Users/moi/.cache/bridget-adoptions/100-20260916.iMZZbR/smoke.rb

Limites : la suite workspace intégrale n'a pas été lancée, car ses harnais
SIGKILL/groupes restent interdits. La recette1345/0/49 de la099 est celle de
l'autre intervenant, pas une preuve de cette intégration. La chaîne avec un
fournisseur réel et un retour de notification est testée par tronçons, pas
par un scénario fournisseur bout-en-bout. Aucun comportement universel T3/natif
ou shell n'est annoncé. Sauvegarde binaire099, base SQLite cohérente vérifiée
quick_check=ok, plists inchangés et état du pont avant adoption :
/Users/moi/.cache/bridget-adoptions/100-20260916.iMZZbR

Précision de FR05 à la relecture : « futur » est ordonné par la réception du
fait au daemon, et non par une horloge source. Aucun rejeu du journal, mais un
fait déjà en transit peut déclencher un abonnement récent. Cette limite est
explicitée dans le contrat, les deux README et la référence des commandes.
Un filtrage temporel strict à la source constituerait une garantie supplémentaire,
non implémentée ici. Les deux régressions ACP TEMOIN_TOOL passent également.
