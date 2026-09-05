# Journal de réalisation — 089

## 2026-09-05 — T007 : espace d'état indépendant réellement traversé

BRIDGET_HOME contient les états du noyau (défaut HOME/.cache/bridget-core),
BRIDGET_SOCKET reste directement dans cette racine privée, limite portable
104 octets exclue. HOME fournisseur est inchangé : aucune copie de credentials
ni migration des données historiques. Le namespace suit env_clear et les
trois projections MCP. Les API wrapper explicitement injectées refusent une
socket différente de celle du processus avant fichier ou fournisseur.

Refus avant accès des chemins historiques, répertoires/fichiers détournés,
propriétaire/droits inadéquats. Le bootstrap valide les sous-états avant
réconciliation des groupes ; la résolution ordinaire ne scanne PAS les
journaux. Le parcours de bootstrap est borné (100 000 entrées, profondeur 64)
et refuse un dépassement ; il n'est pas une preuve contre un attaquant du
même UID modifiant simultanément l'arbre après inspection (T029 reste ouvert).
PID ouvert O_NOFOLLOW/0600 sous verrou, doublon daemon refusé, purge limitée
au tmp privé. Configurations de fédération, notification humaine et reaper
ne relisent plus les chemins historiques ; leurs entrées explicites sont gardées.

Retraits anticipés de surfaces dangereuses : BRIDGET_RUNTIME_SOCKET et
identity migrate --maicie-config sont refusés. Le second évite qu'une config
neuve redirige vers le magasin privé historique de Maicie ; son implémentation
de migration sera retirée en T009, pas remplacée par une copie de son schéma.

Commandes dans le worktree :
- `cargo test --offline -p bridget-daemon --test core_089_isolation_test` :
  8/8, 0,52 s ; vrais daemon/client/MCP nettoyés sous racines /tmp/b89-<UUID>,
  HOME fournisseur séparé, sentinelles intactes, seconde instance refusée,
  erreurs avant bootstrap et divergence d'API attestées. Arrêts SIGTERM
  propres ; ces tests ne prétendent PAS prouver un crash.
- Sous env_clear privé, filtres
  `projections_mcp_portent_le_namespace_sans_modifier_home_fournisseur` et
  oracle de nom persistant lié : 1/1 chacun ; les formats ACP/Codex/Claude
  portent les mêmes trois variables, la cible liée reste intacte.
- `cargo test --offline -p bridget-daemon --lib --no-run` : compilation verte,
  873 scénarios construits avant les deux derniers oracles, aucun lancement
  implicite de toute cette suite.
- `cargo fmt --all --check` et `git diff --check` : exit 0.

Clippy n'est PAS annoncé vert : avec --no-deps, neuf lints historiques hors
hunks T007 persistent (daemon too_many_arguments/empty_line_after_doc_comments,
artifact_service obfuscated_if_else, artifact_store collapsible_if,
execution_store nonminimal_bool, identity_migration collapsible_if et trois
collapsible_if de UI). Sans --no-deps s'ajoutent trois lints Maicie.
Le gate final T034 doit les éliminer ou constater leur suppression de périmètre.

Revue indépendante : trois réserves initiales réellement corrigées (liens
managed/agent-names, migration transitive, anciennes configurations), puis
APPROVE limité à ces frontières ; contre-run intermédiaire 7/7. Self-review
XIX/XX : une seule résolution, gardes aux points d'accès, retrait des replis
temporaires ; pas de framework/config fournisseur supplémentaire. Le coût
du scan reste au bootstrap, pas à chaque appel. Les tests de flotte, de
fournisseur réel et SSH restent distincts et non validés à ce stade.

## 2026-09-05 — T008 : canon neutre, sans changement de protocole

Les algorithmes historiques issuer_scope et canonical_send sont déplacés dans
communication.rs ; daemon, CLI, MCP, contrôle du référent et relais encore
présent les appellent directement. Le hash de scope n'est explicitement PAS
une authentification. Aucune dépendance nouvelle, aucun second encodeur.
Cette extraction additive et neutre ne supprime pas encore une fonctionnalité ;
elle peut précéder la baseline binaire T005 qui attend l'isolation T007.

`cargo test --offline -p bridget-daemon --lib communication::tests -- --test-threads=4` :
3/3, 0,00 s (compilation 18,62 s). Attentes littérales indépendantes pour le
scope et les bytes canoniques ; mutation de in_reply_to distinguée, renommage
d'affichage neutre. L'oracle d'architecture refuse le retour d'un import MCP
par les consommateurs du noyau. Le test historique
`canonical_send_ignore_le_nom_affiche_et_le_timeout_relatif` passe également
1/1, 0,00 s (compilation 5,61 s). Aucun daemon ni fournisseur lancé.

Self-review XIX/XX : le diff déplace les algorithmes, il ne les réécrit pas.
Le module neutre casse la dépendance noyau→présentation et garde un unique
producteur du canon. Les octets restent le contrat ; les trois tests ciblés
ne remplacent pas la future couture CLI/MCP réelle T014. Relecture du diff
effectuée, aucune garantie fonctionnelle globale annoncée.

## 2026-09-05 — P0 vérifiée et baseline T005 partielle

Complément de baseline : 37/37 tests ciblés, 0,01 s, compilation 3,38 s :
communication (3), connection_channel (4), build_info (9), artifact_policy (3),
artifact_types (4), mission_projection (2), runtime (12). Commande dans le
worktree : `env -i PATH=/Users/moi/.cargo/bin:/usr/bin:/bin
HOME=/tmp/bg089-daemon-pure.UDEXtq TMPDIR=/tmp/bg089-daemon-pure.UDEXtq
CARGO_HOME=/Users/moi/.cargo RUSTUP_HOME=/Users/moi/.rustup HOSTNAME=bg089-test
BRIDGET_HOME=/tmp/bg089-daemon-pure.UDEXtq
BRIDGET_SOCKET=/tmp/bg089-daemon-pure.UDEXtq/bridget.sock cargo test --offline
-p bridget-daemon --lib -- communication::tests:: connection_channel::tests::
build_info::tests:: artifact_policy::tests:: artifact_types::tests::
mission_projection::tests:: runtime::tests:: --skip project_runtime::tests::
--test-threads=4`.

Le premier essai sans `--skip project_runtime::tests::` sélectionnait aussi
project_runtime, par sous-chaîne Rust : 53 réussis, 4 rouges en 1,27 s.
Échecs conservés : absent_policy_file_closes_only_docker_runtime (2399),
ingress_prive (2663), policy_loader_rejects_group_writable_file (2522),
spec086attestationcheckout (2261). Inspection des exécutables : Docker était
simulé par tests/fixtures/docker/docker, Git limité à la racine privée ;
aucun fournisseur, daemon ou Docker réel. Ces rouges hors périmètre cible
ne sont ni corrigés ni présentés comme verts.

Ledger : filtre `ledger::tests::` du binaire de tests bridget_daemon-b7902f0f20bc17cc
sous `env -i PATH=/usr/bin:/bin HOME=<racine> TMPDIR=<racine>
BRIDGET_HOME=<racine> BRIDGET_SOCKET=<racine>/bridget.sock`, racine créée avec
`mktemp -d /tmp/bg089-ledger.XXXXXX` : 5/5, 0,06 s. Vérifie corps exact,
demandes globales et états en vol/reçu/indéterminé/orphelin distincts.

Lot SQLite supplémentaire : 32/32 lib (0,15 s, compilation 3,74 s),
artifact_service_test 3/3 (0,04 s), artifact_store_test 4/4 (0,01 s),
execution_resume_test 2/2, work_submission_test 3/3 (0,01 s).
Même env nettoyé, racine /tmp/bg089-sqlite.E4DbLM. Filtres lib :
`agent_profile::tests:: execution_store::focus_priority_tests:: control_settings::
human_inbox::tests:: recovery_trace::tests:: --skip
canal_externe_commande_factice_et_echec_consignes --test-threads=4` ;
les quatre intégrations nommées sont appelées via `cargo test --offline
-p bridget-daemon --test …`. Exclusion explicite du test de notification
humaine par shell ; aucun fournisseur réel, daemon ou crash.

MCP : binaire de tests sous même env privé créé par
`mktemp -d /tmp/bg089-mcp.XXXXXX`, watchdog `/usr/bin/perl -e
'alarm 120; exec @ARGV'`, filtre `mcp::tests:: --test-threads=4` :
39/39 (1,01 s annoncée par le runner). Sockets de serveurs de fixture,
jamais de connexion à un daemon utilisateur : inclut FR009, huit connexions,
corps riche, in_reply_to, coupures, retries et stdout JSON uniquement.

Identité : même commande, filtre `mcp_identity::tests::`, 9 scénarios :
1 réussi / 8 rouges sous /tmp/bg089-identity.*. Contre-sonde sous
/private/tmp/bg089-identity.* : 3 réussis / 6 rouges. Deux causes distinctes
vérifiées : oracle de chemin canonique sensible à l'alias macOS /tmp ;
anciennes fixtures « avant », « agent-b », etc. incompatibles avec le
validate_agent_id UUID déjà utilisé par read_name. Le diff T007 ne change
pas ce validateur ni ces fixtures ; resolve_identity_with garde sa voie
sans namespace via None. Les échecs sont consignés pour les fixtures T015,
pas effacés par un assouplissement de validation du produit.

CLI : après correction d'un premier filtre sans correspondance (0 test,
non compté), `cli::idempotency_projection_tests::options_`, `depot_`, `who_`
et `cli::ledger_borne_tests::` sous la même enveloppe env-i/watchdog,
racine /private/tmp/bg089-cli.* : 11 réussis / 3 rouges, 0,01 s.
Les trois parseurs de dépôt refusent leurs anciennes fixtures avec
« agent_id doit être un UUID v4 canonique ». Le nom invalide ne doit pas
redevenir admissible pour verdir ces tests ; correction de fixtures en T015/T018.

T002 : gel complet de familles, 17 fichiers, référence produit dfa2134 et
capture bd1cbe0 épinglées séparément. Vérificateur --self-test --require-complete
exit 0, six mutants refusés. T003 : 264 fichiers historiques classés, zéro
oublié/doublon, 12 critères reliés aux scénarios Gherkin. T004 : 14 frontières
de confiance. T006 : revue indépendante PASS sur la stratégie, sans transformer
les gates non exécutées en succès.

| Commande réelle | Résultat et durée |
|---|---|
| PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline --workspace --no-run | Compilation de toutes les cibles, exit 0, 32,12 s ; aucun test lancé par cette commande |
| PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline -p bridget-core --lib | 39/39, 1,10 s |
| env -i PATH=/usr/bin:/bin TMPDIR=<racine privée> HOME=<racine privée> XDG_DATA_HOME=<racine privée> BRIDGET_ARTIFACT_ROOT=<racine privée>/artifacts target/debug/deps/bridget_daemon-b7902f0f20bc17cc 'store::tests::' --test-threads=4 | 43/43, 0,95 s ; inclut receipt_store et artifact_blob_store par filtre, aucun daemon lancé |
| même environnement fermé, filtre 'idempotency::tests::' | 39/39, 0,20 s |
| PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline -p bridget-daemon --test mission_boundary_test | Rouge PRÉEXISTANT confirmé : « Maicie ne doit être disponible que pour les fixtures de test » ; le manifeste produit dépend de Maicie. Le test est conservé, T009 doit fermer ce défaut. |

Les racines des deux lots SQLite sont créées par mktemp -d /tmp/bg089-store.XXXXXX
et /tmp/bg089-idem.XXXXXX ; aucune variable BRIDGET_AGENT ni home de production
n'est héritée. Les tests utilisent uniquement leurs DB de fixture.

L'audit a trouvé que le daemon historique peut ramasser le TMPDIR partagé même
avec HOME isolé. Le plan avance donc T007 avant les bancs de daemon de T005 :
isoler avant d'exécuter, pas un skip de gate. La baseline totale, les fournisseurs
réels et SSH restent non validés. L'utilisateur a autorisé les revues/validations
et le travail complet ; aucune bascule de la flotte n'est nécessaire ni engagée.

## 2026-09-05 — Matérialisation du codec de référence (suite T002)

42 trames de cinq familles passent dans le codec de production inchangé de
dfa2134 : envoi/reply/idempotence (sept issues), annuaire/ledger non vide,
spawn/stop (cinq issues), attach et claim/lease/réponse guichet. Les entrées
sont des scénarios de caractérisation ; les sorties sont une capture du codec,
pas des chaînes devinées. `core_089_wire_test` relit ces sorties indépendantes
et compare leur émission octet pour octet. Le générateur est ignoré par défaut
et ne réécrit jamais les attentes. Cela protège le fil, pas encore le canon SQL
ni la livraison réelle, réservés aux gates T014/T018.

Deux lignes natives sont extraites des faux fournisseurs historiques :
codex_app_server.rs:3303 et claude_stream_json.rs:1467 au commit source.
Les tests de session consomment les vrais flux de ces sous-processus et
comparent raw/source/origine aux fixtures, sans normalisation des espaces.
Ce n'est pas une recette auprès des comptes fournisseurs réels.

Commandes : `cargo test --offline -p bridget-transport --test core_089_wire_test`
(1 réussi, 1 générateur ignoré, 0,00 s) et `cargo test --offline -p
bridget-transport --lib session_native_ -- --test-threads=2` (2 réussis,
0,04 s ; faux fournisseurs, TMPDIR=/tmp/bg089-native.QwxMC2).
Le codec protocol.rs et le modèle core/message.rs sont identiques à dfa2134
(`git diff dfa2134 --` sur ces deux sources : vide).

Self-review : aucune dépendance ni DTO produit ajouté ; golden externe nécessaire
car les anciens round-trip se comparaient principalement à eux-mêmes. Les
attentes incluent corps UTF-8, espaces, corrélation et limites déclarées. La
preuve runtime du daemon reste distincte. Seuls les deux oracles de tests
natifs changent dans les sources, pas les pilotes.

Baseline qualité : Clippy a aussi révélé trois erreurs préexistantes côté
transport (variante ACP trop volumineuse, Default dérivable, format constant).
Corrections ciblées : message terminal dans Box, restitué intact à l'adaptateur ;
Default dérivé identique et literal JSON de test. Le test structurel borne la
taille de l'événement et vérifie le message restitué. `cargo clippy --offline
-p bridget-transport --all-targets -- -D warnings` passe ; `acp::tests::`
passe 41 tests, 1 ancien micro-banc ignoré, 4,09 s. Ces corrections suivent la
capture initiale ; elles ne modifient aucun octet filaire. Le contrôle global
fmt ne signalait que les deux include_bytes nouveaux, désormais formatés.

## 2026-09-05 — Préparation uniquement

Base : dfa2134dcfe2a2522e3ae77d93561e6ae72556b3, main de l'ancien dépôt. Clone indépendant créé par :

```sh
git clone --no-local --no-hardlinks --single-branch --branch main /Users/moi/Nextcloud/10.Scripts/bridget /Users/moi/Nextcloud/10.Scripts/XX.bridget
```

Résultat : exit 0, 0,5 s observée. L'origine locale a ensuite été retirée du nouveau clone pour empêcher un push accidentel vers l'ancien dépôt. Aucune identité Git modifiée.

Worktree dédié : `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core`, branche `session-089-communication-core`, créée par le hook SpecKit git-feature. Synchronisation SpecKit limitée au nouveau projet ; aucun adaptateur global modifié. La préparation officielle plan/tasks résout correctement ce dossier via feature.json.

`cargo metadata --no-deps --format-version 1 --offline` : exit 0. Quatre membres encore présents, dont Maicie ; la dépendance directe maicie du daemon est constatée. **Le clone n'est pas encore le noyau extrait.**

Le dépôt original présentait les modifications suivantes avant et après préparation : Cargo.toml de l'application desktop modifié ; répertoires .claude/.gstack et fichier watch_20260825-084049 non suivis. Rien de ce WIP n'a été copié ni modifié. Aucun daemon, wrapper, client de production, tunnel ou outil de communication inter-agent n'a été lancé.

## Registre des preuves à fournir

Vérifications de préparation : IDs T001–T036 uniques et ordonnés, aucun placeholder de template restant, prérequis plan/tasks reconnus, métadonnées Cargo lues hors ligne. Le vérificateur SpecKit refuse le préfixe Git `session-` sans sélection explicite ; il passe avec `SPECIFY_FEATURE=089-communication-core`, sans patch des scripts officiels. Ce résultat porte sur les artefacts, pas sur le logiciel.

SC-08901..SC-08912 : **NON EXÉCUTÉS**. Aucun test Rust, gate fournisseur, crash-test ni scénario SSH réalisé lors de cette préparation. Les vérifications documentaires et Git ne valent pas non-régression de l'extraction.

Chaque future entrée doit contenir : commit, commande exacte, espace de test, résultat, durée, oracle et éventuel mutant ; liste séparée des gates non exécutés. Les résultats préexistants des anciens chantiers ne sont pas réattribués à 089.

## 2026-09-05 — T001 achevée, T002 partielle

L'utilisateur a autorisé la poursuite de l'inventaire et des tests avec les deux cases préalables ouvertes. La revue reste obligatoire avant suppression. Aucun code de production ni données utilisateur modifiés.

T001 : baseline.md couvre les 45 modules du daemon et 22 modules core/transport, les sept sources de schéma SQL, les commandes du dispatch et les résolutions de chemins. La comparaison des tables aux trois lib.rs ne laisse aucun module racine sans disposition. La lecture a rectifié le propriétaire de DaemonConfig (daemon.rs, pas runtime.rs) et identifié un mélange fixture de forme/contrat filaire Codex ; aucune correction opportuniste du pilote.

Première tranche T002 : 13 fixtures historiques, manifeste et vérificateur local. Toutes les familles restantes sont nommées ; **T002 n'est pas cochée**. Cette livraison partielle empêche précisément de déclarer un corpus complet à partir des seules trames faciles à copier.

| Commande exécutée dans le worktree 089 | Résultat | Durée observée |
|---|---|---|
| PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline -p bridget-transport --lib protocol:: -- --test-threads=4 | 65 réussis, 0 échec, 0 ignoré, 184 filtrés ; aucun processus fournisseur/daemon | Compilation 9,73 s ; tests 0,03 s |
| sh -n scripts/verify-089-contracts.sh | Syntaxe shell valide | Incluse dans le lot de vérification <1 s |
| sh scripts/verify-089-contracts.sh --self-test | 13 fichiers égaux à Git ; 6 mutants refusés | <1 s |
| sh scripts/verify-089-contracts.sh --require-complete | Exit 1 attendu : six familles manquantes | <1 s |

### Self-review XIX/XX

- Nécessité : protéger les bytes et distinguer manque de fixture de régression fonctionnelle avant extraction.
- Choix : un seul vérificateur local, Git + bibliothèque standard Python déjà requise par l'outillage ; aucune dépendance du produit ajoutée, aucun nouveau framework.
- Hypothèse : le commit source est accessible dans le clone indépendant ; contrôlé par Git. Un fichier de fixture historique n'est pas nécessairement un protocole valide en production.
- Vérifié : lecture des coutures, couverture des modules, hash + comparaison aux objets, échecs discriminants et tests de protocole.
- Non vérifié : suite complète, crashs, pilotes réels et SSH ; aucun SC de livraison déclaré clos.
- Complexité évitée : aucun parseur Rust maison pour fabriquer des fixtures depuis les sources ; aucun nouveau DTO, daemon ou magasin de données.
- Charge de maintenance : un manifeste relie chaque fixture à son origine ; une seule liste fermée des familles oblige à rendre visibles les lacunes.
