# Implémentation 090 — Implemented, adoptée le 2026-09-06

## Complément humain : nom de conversation et menu de reprise

Base `6ee129b5880d`, branche `fix/090-resume-selector`. Codex 0.153.4 accepte
UUID ou nom après `resume` et un sélecteur sans argument (`codex resume --help`,
vérifié localement). Bridget imposait indûment un UUID. Le nom utilisateur
`horizon-original` existe réellement dans l'index fournisseur ; aucune
réécriture de cet index ni lecture de base fournisseur n'est ajoutée au produit.

Le parseur distingue absence de reprise, UUID, nom exact et sélection humaine.
La sélection utilise `thread/list` sur le même app-server privé : pages de 100,
borne de 1 000 résumés et échéance absolue de 10 s pour la lecture. UUID invalides,
curseur manquant/répété, page excessive, catalogue incomplet : refus, jamais
sélection sur un préfixe incomplet. Métadonnées seulement (nom ou aperçu, cwd,
UUID) ; aucun ancien tour chargé. Le sélecteur est un menu Bridget numéroté,
pas la TUI de sélection Codex. La TUI officielle commence après le choix,
la résolution de la liaison d'identité, Register et activation du journal.
Pas de nouveau fil provisoire, pas de sonde fournisseur facturée, pas de cache.

Nom exact absent/ambigu : refus avec invitation au menu. `--name` reste le
nom de présence Bridget, distinct du nom de conversation. Un changement de nom
Codex n'affecte pas la liaison durable UUID-fil→identité. Le terminal du menu
reste canonique ; q/Ctrl-C/HUP/TERM libèrent l'attente et déclenchent le nettoyage
du pilote. Contrôles neutralisés dans les métadonnées affichées.

Preuves : **16/16** recettes du vrai Codex/TUI/daemon/MCP, 64,32 s, HTTP local
et HOME/CODEX_HOME privés. Les cinq nouvelles recettes couvrent nom réel avec
historique+tour humain+envoi Bridget+ACK/retry, menu réel sans fil chargé avant
sélection, numéro invalide, Ctrl-C, nom absent, homonymes refusés. Ancien binaire
installé : même recette nom → rouge `resume exige l'UUID explicite du fil Codex` ;
version corrigée verte. Logs `/tmp/b90-selector-old-red.log`,
`/tmp/b90-selector-native-final.log`, `/tmp/b90-selector-cancel.log`.

Deux défauts du harnais corrigés sur observation réelle : une conversation sans
premier tour n'est pas publiée par thread/list (avec ou sans useStateDbOnly),
donc les scénarios de sélection créent un vrai tour préalable ; le faux shell
PTY interceptait lui aussi Ctrl-C, désormais ignoré par le parent après spawn
comme un shell attendant son job. Le programme testé reçoit toujours le signal.

Gate unitaire : **1 014 succès**, zéro échec, 8 ignorés, plus deux sous-tests
enfants verts ; `cargo test --workspace --lib -- --test-threads=1`, racines
privées courtes/env vide/umask 077, 67,19 s hors compilation. Test de la borne
du catalogue : la onzième page est interdite et une échéance consommée empêche
tout appel. Clippy workspace/all-targets -D warnings et fmt --all --check : PASS.
Logs `/tmp/b90-selector-units.log`, `/tmp/b90-selector-clippy-final.log`.
La dépendance UUID déjà au workspace est réutilisée pour valider les IDs Codex,
sans parseur maison. Aucune modification du protocole daemon ni migration DB.

## Correctif utilisateur du 2026-09-06 : reprise humaine sans UUID Bridget

Base `49710fa66563`, branche `fix/090-human-resume`. Le wrapper créait une
nouvelle identité avant de tenter de lui donner le nom durable déjà possédé
par l'ancienne : `NameConflict` après fermeture, y compris avec `resume`.
Le message natif `codex --remote …` proposait en outre une socket que Bridget
ferme lorsqu'il possède la session ; la dernière instruction de sortie doit
être une vraie commande Bridget, pas cette adresse temporaire.

Ajout d'une lecture fermée `display_name_resolve` version 1 auprès du daemon,
sur le même index normalisé UNIQUE que le renommage. Aucun accès client à la
base, aucune table supplémentaire, aucune suppression de profil. `--name`
réutilise l'identité inactive ; le contrôle d'activité précède le fournisseur
et Register reste l'arbitre atomique en cas de course. Nom, liaison de fil et
UUID explicite contradictoires : refus sans modification. La liaison locale
fil→identité réutilise `agent-names` et est écrite atomiquement, synchronisée,
avant la saisie. La reprise conserve le titre fournisseur et son historique.
Sans `resume`, même nom signifie nouvelle conversation, même identité.
Pour les fils antérieurs sans liaison, fournir le nom une première fois.

La commande finale `Reprendre : bridget codex … resume <fil>` conserve les
options explicites, protège les arguments shell et ne rejoue pas le prompt
initial. Aucun UUID Bridget requis. README FR/EN et skill canonique alignés.

Preuves exécutées (HOME, CODEX_HOME, BRIDGET_HOME privés, HTTP local, vraie TUI
et vrai app-server Codex 0.153.4, aucun compte ni fil utilisateur modifié) :

- `cargo test -p bridget-daemon --test codex_interactive_090_test -- --include-ignored --test-threads=1` : **11/11**, 40,09 s.
- Nouvelle recette `--human-resume` : démarrage nommé sans UUID, tour humain,
  refus d'un second actif avant fournisseur et sans nouveau profil, fermeture,
  redémarrage daemon, reprise nom+fil puis fil seul, nouvelle conversation
  sous le même nom ; identité/historique inchangés, zéro prompt implicite,
  termios restauré, sockets supprimées. Refus fil/UUID contradictoires.
- Couture RPC : réponse nom→UUID+activité réelle, refus champ futur et version
  inconnue. Mutation du résolveur exécutée : ignorer l'identité retrouvée
  fait échouer la recette sur `name-new-thread` (exit 1, `NameConflict`).
  Restauration : **1/1**, 12,07 s, corpus RPC fermé inclus.
- `cargo test --workspace --lib -- --test-threads=1` : **1 012 succès**, zéro
  échec, 8 ignorés, plus deux sous-tests enfants verts ; 61,83 s hors build.
  Racines courtes sous `/tmp`, environnement vide et `umask 077`.
- `cargo test -p bridget-daemon --features test-support --test core_089_identity_test -- --test-threads=1` : **3/3**, 1 worker ignoré, 6,49 s.
- `cargo clippy --workspace --all-targets -- -D warnings` et
  `cargo fmt --all --check` : PASS. Pas de nouvelle dépendance.

Logs locaux : `/tmp/b90-human-all-native.log`, `/tmp/b90-human-mutant.log`,
`/tmp/b90-human-restored.log`, `/tmp/b90-human-units.log`,
`/tmp/b90-human-identity.log`, `/tmp/b90-human-clippy-final.log`.
La mutation n'est pas livrée. L'adoption du daemon nécessite la nouvelle RPC ;
les anciens wrappers restent compatibles avec ce protocole additif.

## Correctif utilisateur du 2026-09-06 : reprise d'un historique volumineux

Branche `fix/090-codex-history`, base `264597c44c32`. Cause mesurée sur une
copie privée d'un rollout de 338 072 566 octets : `thread/resume` renvoie une
trame de 79 814 612 octets, rejetée par la borne WebSocket Bridget de 16 Mio.
Le serveur reste vivant ; le reader effaçait l'erreur de lecture en annonçant
faussement `stdout Codex fermé`. Avec `excludeTurns: true`, la réponse de la
sonde est de 2 653 octets. Paramètre vérifié dans le schéma généré du Codex
installé 0.153.4, aucune nouvelle dépendance ni augmentation de la borne.

Le contrôleur demande les métadonnées sans récupérer les anciens tours. Codex
conserve l'historique et sa TUI le consulte nativement. Reprise explicite :
`legacy` ET `paginated` acceptés, mode inconnu refusé ; nouveau fil : garde
legacy inchangée. UUID, nom Bridget, projection yolo et attestations du chemin
géré restent inchangés. Une erreur I/O restitue désormais sa cause bornée.

Contre-épreuve exécutée : même copie, ancien binaire installé = rouge avec le
message utilisateur exact ; corrigé = même UUID, nom attesté, TUI réellement
en raw mode affichant une saisie non validée, zéro prompt fournisseur, puis
termios restauré et socket supprimée. HOME/CODEX_HOME isolés, aucun secret
copié, fournisseur HTTP local, aucun fichier du fil utilisateur modifié.
Logs privés : `/tmp/b90-history-copy-old-red.log` et
`/tmp/b90-history-copy-final.log`. Reproduction opt-in : fixture
`crates/bridget-daemon/tests/fixtures/codex_interactive_090.py`, option
`--copied-resume`, source fournie par `BRIDGET_CODEX_090_ROLLOUT` et copiée.

Recettes natives automatisées : **10/10, 35,19 s**, dont historique paginé
réel rendu et reprise legacy avec message interagent/ACK/retry. Le test de
construction de la requête verrouille `excludeTurns` uniquement pour la
reprise interactive ; le test du reader vérifie la conservation de l'erreur
de lecture. Logs : `/tmp/b90-history-native-final.log`.
Le premier essai de fixture volumineuse artificielle a été retiré : ajouter
des tours au rollout seul ne modifie pas l'index paginé du fournisseur et ne
constitue donc pas un oracle de longueur. La copie réelle fournit cette preuve.
Clippy workspace/all-targets -D warnings et fmt --all --check : PASS.
Gate unitaire finale, `cargo test --workspace --lib -- --test-threads=1`,
sous HOME/BRIDGET_HOME/TMPDIR privés et courts, `umask 077` : **1 010 succès,
0 échec, 8 ignorés**, plus deux sous-tests enfants verts ; 60,16 s de tests
hors compilation. Log : `/tmp/b90-history-units.log`. Pas de nouvelle passe
des intégrations sans rapport avec ce correctif ; les dix recettes natives
couvrent la couture modifiée.

## Correctif utilisateur du 2026-09-06 : yolo / resume / name

Branche `fix/090-codex-options`, base `cd0ec467d870`. La commande complète
`bridget codex --name <nom> --yolo resume <UUID>` est traitée, pas seulement
l'alias. Le nom utilise le service durable partagé, distinct de l'UUID.
La reprise privée négocie réellement `thread/resume` avant Register, vérifie
l'UUID retourné et l'historique reconnu et ne renomme pas le titre existant.
La garde de reprise automatique gérée reste inchangée.

La première recette a réellement échoué : l'ancien `approval_policy=on-request`
survivait à `--yolo` malgré le sandbox désactivé. Correction : `config/read`
fournit les réglages effectifs du serveur (profils/-c compris), projetés dans
`thread/resume` sans parseur TOML concurrent ni valeur permissive inventée.
Le contexte durable Codex prouve désormais `never` + `danger-full-access`.
Schémas vérifiés sur le binaire 0.153.4 via `generate-json-schema --experimental` :
`ConfigReadParams/Response` et `ThreadResumeParams/Response`.
Documentation officielle : https://learn.chatgpt.com/docs/app-server et
https://learn.chatgpt.com/docs/developer-commands?surface=cli.

Recettes natives : 9/9, 31,58 s, vraie TUI/app-server/daemon/CLI/MCP,
fournisseur HTTP local déterministe, pas de compte ni fil utilisateur touché.
Nouvel oracle : historique antérieur et titre conservés, UUID inchangé,
nom affiché, permissions attestées, message Bridget dans le fil repris,
ACK durable attendu avant retry (un terminal de tour n'est pas encore cet ACK),
retry sans seconde injection, termios et socket nettoyés. Fil inexistant :
refus explicite `thread/resume`, zéro présence et zéro socket résiduelle.
Log : `/tmp/bridget-090-options-native.log`.

Les premières passes unitaires n'utilisaient pas complètement l'environnement
de validation 090 : TMPDIR macOS trop long puis créations non privées. Les
gardes ont refusé ces fixtures ; aucun assouplissement de production effectué.
La passe privée parallèle a ensuite donné 712 succès, 1 échec dans la couture
MCP T1208 (daemon fermé sans réponse), 7 ignorés. Contre-passe sérialisée exécutée,
avec `umask 077`, env vide, HOME/BRIDGET_HOME/TMPDIR privés et courts :
`cargo test --workspace --lib -- --test-threads=1`, 39 core + 713 daemon +
258 transport = 1 010 succès, 0 échec, 8 ignorés (et deux sous-tests enfants
rapportés séparément, verts). Temps de tests 59,02 s, hors compilation.
Logs : `/tmp/bridget-090-options-serial-lib.log` ; namespace `/tmp/b90s.BiAY`.
Le rouge parallèle reste documenté, sans modifier un oracle MCP hors de ce lot.
Clippy workspace/all-targets -D warnings et fmt --all --check : PASS.
Les tests complets d'intégration non liés à cet amendement ne sont pas relancés
à chaque modification : la précédente recette globale reste consignée ci-dessous.
L'adoption suit le commit ; binaire précédent et reçu externe conservés sous
`/Users/moi/.cache/bridget-adoptions/090-options-20260906.YwSxn3`.

Début : 2026-09-05 19:46 CEST. ETA initiale : 125–240 min.
Branche session-090-codex-interactif, base 6cfbc4d33ca7, production inchangée.

## Préparation

Sync utilisateur effectué. Scripts/templates officiels absents de l'extraction :
skills lues et protocoles appliqués manuellement, pas de réinstallation massive.
Specify, Plan, Audit Existing PASS, Tasks (14 dont 12 ouvertes), Analyze exécutés.
Contre-revue autre fournisseur : bridget who = aucun agent connecté. Exploration
locale indépendante de réutilisation faite ; ne vaut pas revue inter-fournisseurs.

## T001 / T002

Sonde réelle : deux clients WS, même fil après nommage, tour synthétique achevé.
Variante approbation : même requestApproval côté A/B, refus B accepté ; aucun
secret, HOME isolé, serveur arrêté et attendu. La sonde conserve des répertoires
temporaires de diagnostic privés ; aucun daemon utilisateur arrêté.
Recherche de doublons faite avant nouveaux modules/helpers/dépendance ; conclusions
dans reuse-audit.md. Sonde Python hors production, absence de bibliothèque WS
Python constatée ; son framing réduit n'est pas réutilisé dans le produit.

Self-review XIX/XX : besoin utilisateur explicite ; réemploi pilote/wrapper/ACK ;
risques restants natifs/TUI/permissions couverts par tâches ; aucun succès de
production revendiqué ; aucun commit automatique.

## Assemblage et preuves ciblées

Codex installé 0.153.4, véritable app-server et véritable TUI, sans tmux.
Même pilote, worker, reader et boucle wrapper ; nouveau cadrage WS Unix et garde
TUI, aucune base ou flotte parallèle. Fait TerminalSessionReady par connexion,
après démarrage natif et à la reconnexion. Échec d'écriture daemon : lien fermé,
pas de flush répété sur une trame partielle. JournalFailed rejoint le même arrêt
TUI puis serveur. Une socket de récupération restante produit une erreur.

| Tâches | Résultat exécuté |
|---|---|
| T003 | 8 security_tests communication::client verts : backlog réel, budget commun, goutte-à-goutte, LF et poison. |
| T004/T005/T008/T009 | 58 tests transport codex_* verts ; source raw atypique, ACK filtré par clientId/thread/turn et une seule fois, input humain sans vol de corrélation, aucune réponse automatique aux permissions. |
| T006 | 2 tests de parsing et refus binaire sans TTY verts. |
| T007/T008/T009/T010/T011 | 7 recettes du binaire Codex, exécutées explicitement, 0 échec en 23,29 s : tour humain actif→FIFO sans ACK précoce ; daemon redémarré→identité et thread inchangés ; vrai MCP→answered ; retry→une réponse ; attach replay/live ; deny→session encore joignable ; HUP/EOF ; new/resume historique→fin explicite. |
| T010 | Crash fournisseur SIGABRT au jalon permission visible : retour 1,015 s ; HUP wrapper : 1,099 s ; termios restauré, aucun enfant direct ni socket restant. Core dumps désactivés dans l'enfant du harnais ; aucun SIGKILL utilisé. |
| T011 | Test daemon TerminalSessionReady + purge/connexion inconnue, round-trip exact du fait, et wrapper Codex géré historique arrêté au shutdown : verts. |
| T012 | Abonnement ChatGPT local, gpt-5.6-luna : vraie TUI→permission ponctuelle→vrai outil MCP→answered→retry accepted unique→attach replay/live→termios et socket nettoyés, exit 0. Aucun fallback API ni configuration utilisateur modifiée. |
| T013 | README FR/EN, skill canonique et installation alignés ; paquet autonome vérifié, empreintes et mutation d'un octet refusée. |

Recettes mock : `BRIDGET_CODEX_090_BIN=/opt/homebrew/bin/codex cargo test -p bridget-daemon --features test-support --test codex_interactive_090_test -- --include-ignored --test-threads=1 --nocapture`.
Preuves : /tmp/b90-58j_ektu (crash), /tmp/b90-g6rrgyo_ (HUP),
/tmp/b90-_uquqo7v (new), /tmp/b90-i0wvl9uy (deny),
/tmp/b90-4ntoprgv (resume), /tmp/b90-8tbi6at8 (parcours complet).
Recette abonnement : `BRIDGET_CODEX_090_AUTH=/Users/moi/.codex/auth.json /usr/bin/python3 crates/bridget-daemon/tests/fixtures/codex_interactive_090.py target/debug/bridget /opt/homebrew/bin/codex --subscription` ;
log /tmp/b090-subscription.sHamRM, preuves /tmp/b90-ooyk1x32. Seule une copie
0600 d'authentification a été créée dans la fixture puis effacée dans finally ;
jamais reproduite dans les logs. Les preuves temporaires ne sont pas versionnées.
Paquet : `bash scripts/tests/package_089_test.sh`, PASS,
/private/tmp/b89-package-test.wnQtrk.

### Corrections issues des contre-épreuves

- Le statut Cli+journal ne distingue pas géré/terminal : le test historique natif
  a échoué à l'arrêt, puis a passé après TerminalSessionReady explicite.
- Signal TERM/INT au fournisseur : drain gracieux pendant élicitation, pas EOF.
  Le test de crash emploie donc ABRT sur son seul enfant identifié.
- Le renderer fragmente légitimement les deltas : l'oracle live compare les
  deltas réels et le terminal, pas un découpage HTTP artificiel.
- Cold resume 0.153.4 émet thread/status/changed, pas thread/started : les deux
  faits étrangers sont gardés, y compris sous-agent interne. Limite explicite :
  un seul fil par invocation ; quitter et relancer pour changer.
- Le harnais shell-parent doit continuer de drainer le PTY même au nettoyage ;
  sinon le banc bloque le fournisseur sur son terminal rempli.
- Une réouverture de test sous /tmp public provoque des refus de fichiers privés.
  Les gates utilisent un TMPDIR court ET privé (0700), pas /tmp directement.
- Un import test_sync non gaté empêchait la compilation workspace sans feature :
  import et armement conditionnels restaurés, aucun test désactivé.
- Les 3 tests historiques Codex/faux-tmux et 5 tests de transformation argv ne
  décrivent plus une surface accessible. Sous-harnais et branches mortes retirés,
  sans ajouter de bypass de test. Correspondances : prompt exact Claude conservé ;
  canon/identité/réponse MCP = core_089_contract/identity/reply ; corps riches/FIFO/
  reconnexion gérés = run_corpus de managed_parity ; interactif réel = recettes090.
  Aucun résultat n'est présenté comme une conservation du mode Codex/tmux.

### Contre-relecture

Autre fournisseur : `bridget who` de nouveau exécuté à 22:00 CEST,
« Aucun agent connecté ». Pas de lancement implicite pour fabriquer un relecteur.
Lecture indépendante locale de codex_session_reuse : navigation, bornes, corrélation,
cleanup et disparition du code dormant. Constats vérifiés et corrigés ; ne vaut
pas une contre-revue d'un autre fournisseur.

## Gates finaux

Gates terminés le 2026-09-05 vers 22:24 CEST :

| Commande | Résultat | Log |
|---|---|---|
| cargo test --workspace | PASS, 1175 succès rapportés / 0 échec / 35 ignorés | /tmp/g90.ULJzwS/default.log |
| cargo test --workspace --features test-support | PASS, 1203 succès rapportés / 0 échec / 41 ignorés | /tmp/g90.9Ssalx/features.log |
| recette native 090 --include-ignored | 7/7, 29,50 s, zéro échec | /tmp/b090-native-final.dwMWvl |
| cargo clippy --workspace --all-targets -- -D warnings | PASS | sortie Cargo, 4,08 s |
| cargo clippy --workspace --all-targets --features test-support -- -D warnings | PASS | sortie Cargo, 4,01 s |
| cargo fmt --all --check ; git diff --check | PASS, aucune sortie | contrôle final |

Les sommes Cargo incluent les résultats de sous-processus ; ce ne sont pas des
tests tous distincts. Les ignorés historiques ne sont PAS déclarés exécutés :
les six recettes natives ignorées par défaut sont exécutées explicitement avec
le septième test sans TTY. Les sommes de durées des résultats Cargo sont 172,07 s
et 207,61 s, pas les durées murales de compilation.

Environnement des deux gates globaux : env -i, HOME/BRIDGET_HOME/TMPDIR privés
0700 sous leurs racines /tmp/g90.*, BRIDGET_SOCKET court dans ce namespace,
CARGO_HOME=/Users/moi/.cargo, RUSTUP_HOME=/Users/moi/.rustup,
PATH=/Users/moi/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin.
Watchdog absolu 900 s via /usr/bin/perl (alarm puis exec cargo).
Aucun daemon utilisateur ni binaire installé touché.

Audit v14 demandé par la skill, mode fix puis cycle-scoring readonly : un LOW
(deux helpers de test sans appel) corrigé, zéro finding restant confirmé.
Clippy et fmt réexécutés après cette seule correction sans effet runtime.
Rapport local ignoré par Git :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/090-codex-interactif/audits/2026-09-05/session-2026-09-05-spec-090-01/scoring.md.
validate_session.py : 0 erreur, 0 warning. Note A sur le diff, pas sur tout le
produit. Pas de scanner CVE/historique secrets installé ; revue sécurité
indépendante locale, aucun autre fournisseur joignable.

## Convergence

Deux passages manuels (primitive dédiée absente), lecture spec/plan/tasks/
checklist/code. CONVERGED ; zéro tâche ajoutée ; tasks inchangé pendant chaque
phase de convergence. Alignements documentaires effectués HORS de ces phases.

| Exigence | Réalisation et oracle |
|---|---|
| FR-09001 | codex_interactive.rs:19/104, wrapper.rs:1439 ; parsing et refus sans TTY, recette native. |
| FR-09002 | codex_app_server.rs:288/453/2411 ; même fil puis fermeture sur new/resume historique. |
| FR-09003 | reader item/completed corrélé, worker et tracker réutilisés ; vraie recette MCP, answered, retry et ledger. |
| FR-09004 | wrapper.rs:3479/3500 ; présence Cli, modèle observé, TerminalSessionReady explicite. |
| FR-09005 | codex_app_server.rs:2636 et attach input/user_message ; raw atypique à :3237, vraie attache replay/live. |
| FR-09006 | reader requêtes réservé à la TUI à :2442 ; oracle aucun auto-reply et vraie acceptation/refus natifs. |
| FR-09007 | spawn_worker à :955, input_seen à :2662 ; humain actif puis FIFO et ACK clientId/thread/turn. |
| FR-09008 | NativeTui::close à :204, stop_owned_child à managed_session.rs:17 ; HUP/ABRT/TERM, termios et ressources. |
| FR-09009 | reconnect_managed_session à wrapper.rs:4342/4383, writer borné à :1322 ; daemon réellement relancé, fil/identité inchangés. |
| FR-09010 | namespace/socket configurable et connect_nonblocking partagés, WS exclusivement Unix privée ; sécurité client et noyau 089 rejoués. SSH intermachine non rejoué pour 090. |
| FR-09011 | gates complets et managed_parity conservés ; documentation distingue persistant/interactif/attach ; ancien Codex/tmux retiré explicitement. |

SC-09001/02/03/04/05/06 : recettes natives + unités référencées ci-dessus.
SC-09007 : gates complets ci-dessus, retrait des seuls oracles du comportement
Codex/tmux remplacé documenté, pas de désactivation cachée pour obtenir du vert.

## Clôture du pipeline

14/14 tâches, aucune restante. Specify → Plan → Reuse PASS → Tasks → Analyze
(fallback manuel exécuté et relu) → Implement → Converge → Audit.
Nouveaux modules : frontière WS Unix et propriétaire TUI. Réemplois :
pilote/worker/reader/wrapper/journal/ACK/identité/connexion ; arbitrages complets
dans reuse-audit.md. Ni commit, ni merge, ni installation automatique.

ETA initiale 125–240 min (milieu 182,5). Recalibrage à 20:10 : 110–210 min
restantes (milieu 160). Durée observée jusqu'à clôture vers 22:24 : ~158 min
depuis 19:46 ; écart -13,4% au milieu initial, ~134 min après recalibrage soit
-16,3% au milieu recalibré. Fourchettes respectées. Principal coût : recettes
réelles permissions/navigation/nettoyage ; tests ciblés puis suites finales.
Prochaine action : revue/adoption explicite du diff, pas de relance de production.

## Adoption demandée le 2026-09-06

Instruction utilisateur « finis tout » : intégration dans main et installation
autorisées, sans push distant. Préflight : main toujours à 6cfbc4d33ca7, arbre
propre ; annuaire installé vide ; Codex 0.153.4 inchangé. Les skills globales
sont déjà des liens vers la source canonique de 64.bridget : la fusion actualise
leur contenu sans dupliquer les fichiers. Les suites complètes ci-dessus restent
applicables au code inchangé ; formatage et recette native release rejoués pour
l'adoption. Le reçu opérationnel et le retour arrière sont conservés dans
/Users/moi/.cache/bridget-adoptions/ (hors Git, aucun secret versionné).

### Résultat de l'adoption

Code e06d5c9531b7 intégré dans main par fast-forward, release construite avec
`cargo build --locked --offline --release -p bridget-daemon` : PASS, 41,42 s.
Ancien daemon arrêté gracieusement, annuaire vide vérifié juste avant ; binaire
remplacé atomiquement après copie de sauvegarde, plist et namespace inchangés.
Daemon neuf réellement en ligne, build-id identique au client, base quick_check
OK et entrée ledger préexistante conservée. Aucun agent personnel arrêté.
Les trois liens de skills (Codex/Claude/agents) lisent bien le nouveau contenu.

Recettes du binaire release : parcours complet humain actif/FIFO/reconnexion/
MCP/answered/retry/attach/nettoyage PASS ; EOF en permission = 1,047 s, PASS.
Le premier re-jeu abonnement via le binaire installé a révélé un oracle trop
strict : le vrai modèle a écrit une introduction avant OK-090. La réponse liée
et le rejeu étaient déjà corrects ; seul le test de texte exact échouait.
Correction test-only : le modèle réel doit finir par la sentinelle, le modèle
synthétique reste comparé strictement ; chaque delta réel doit toujours apparaître
dans attach après l'abonnement et avant la fin. Pas de correction de production.
Second essai abonnement gpt-5.6-luna PASS, y compris attach, termios et socket ;
parcours synthétique strict rejoué PASS. Les copies privées d'authentification
ont été effacées par le harnais, y compris après le premier échec.

Preuves et retour arrière :
/Users/moi/.cache/bridget-adoptions/090-20260906.6xAQvh/
(release-recette.log, release-eof.log, installed-subscription.log,
installed-subscription-final.log, installed-synthetic-final.log).
Ancien binaire 6cfbc4d33ca7, base arrêtée et plist conservés dans ce répertoire
0700 ; aucun secret/version utilisateur envoyé à Git. Les suites complètes du
05 restent valides pour le code de production inchangé. Aucun push distant.

## 2026-09-06 — coupure sur sous-agent interne (corrigée et installée)

Arbitrage humain : l'équipe interne Codex reste hors administration Bridget.
L'identité Bridget reste liée au fil principal ; créer/reprendre un autre fil
ne prouve aucune navigation et ne doit pas interrompre le serveur.

Diagnostic réel : enfant `/root/communication_mcp` créé à 13:10:30Z, puis
reprise à 13:15:20.847Z et interruption du parent à 13:15:21.052Z. Le daemon
principal est resté en vie. Le reader arrêtait explicitement la session sur
`thread/started` ou `thread/status/changed` d'un autre fil ; son passage à
`alive=false` déclenchait la fermeture du serveur et de la TUI par le wrapper.
Les notifications secondaires sont désormais filtrées sans arrêter le reader.
Ce changement corrige un contrat 090 trop restrictif, pas une panne fournisseur
démontrée. La source de production antérieure reste conservée dans Git.

Validation initiale pilote (équipier sandboxé, builds délégués au pilote) :

- `PATH=/Users/moi/.cargo/bin:$PATH TMPDIR=/tmp/b92.LVqy1o CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target cargo test -p bridget-transport spec093_lecteur_interactif -- --nocapture` : 1/1 vert, compilation 2,59 s. Le nom technique `spec093` identifie le test ajouté pendant la reprise ; il couvre T018 de cet amendement 090.
- Mutation exécutée dans `/tmp/b090-mutant.heaSLg`, jamais dans le worktree : restauration du `break` sur les deux notifications étrangères ; même commande, 0/1, code 101, oracle « le reader doit traiter l'événement parent suivant ». Compilation 8,61 s. Cela prouve que le test refuse l'ancienne coupure.
- Premier essai de compilation : assertion test `Result<ServerResponse, String>` non comparable (E0369), remplacée par comparaison de l'erreur. Warning `dropping_references` encore présent à cette première passe ; gate clippy non acquis à ce stade.

Validation finale :

- Filtre `codex_app_server` : 58/58 PASS, 3,06 s (compilation 8,14 s). Warning corrigé par bloc lexical, aucune modification du type de production pour satisfaire une assertion.
- `BRIDGET_CODEX_090_BIN=/opt/homebrew/bin/codex cargo test -p bridget-daemon --test codex_interactive_090_test non_destruct -- --include-ignored --test-threads=1 --nocapture` (même PATH/TMPDIR/CARGO_TARGET_DIR) : 2/2 PASS, 8,53 s, compilation 2,28 s. Racines `/tmp/b90-vzasgbrq` et `/tmp/b90-v20wmg2y`.
- Création secondaire via le vrai `thread/start` et un vrai tour secondaire, histoire legacy explicitement choisie ; reprise via `/resume` dans la vraie TUI. Même identité/connexion parent, message retrouvé exclusivement dans son histoire, corps secondaire absent du journal parent, sortie native `/quit`, socket nettoyée et terminal restauré. Le protocole des fils est réel ; les réponses modèle sont fournies par la fixture HTTP locale, sans dépense de modèle ni compte de production.
- Deux corrections de harnais, après observation : `/new` utilise un historique dont `thread/read(includeTurns=true)` renvoie `-32601 list_turns is not supported yet` ; ne pas assimiler cette erreur à une histoire vide. La recette création emploie donc le bootstrap legacy pris en charge. Ctrl-C deux fois dans un seul bloc provoquait `no active turn to interrupt` après la fin native : sortie explicite `/quit` rendue avant Entrée, sans sleep de synchronisation.
- `cargo test --workspace --quiet` : 1 219 PASS, 0 FAIL, 47 ignorés, somme des durées de suites 180,01 s ; les deux résumés de processus fils fsutil ne sont pas recomptés.
- `cargo fmt --all --check` et `git diff --check` : PASS. `cargo clippy --workspace --all-targets -- -D warnings` : PASS, 7,98 s.
- `cargo build --release -p bridget-daemon --bin bridget` avec target séparé du worktree : PASS, 31,82 s. Recettes `--new-thread` et `--resume-thread` rejouées directement avec ce release : 2/2 PASS, racines `/tmp/b90-ud72lnld` et `/tmp/b90-0ppyi8l2`.

Limite explicitement conservée : la recette optionnelle historique
`eof_fournisseur_pendant_permission_borne` reste rouge AVANT l'injection EOF :
elle attend un écran de permission pour `bridget_send`, alors que cette opération
fait désormais partie des outils MCP préapprouvés (091). Journal réel : le tour
est terminé, aucune permission en attente. La même recette inchangée et le
binaire installé ANTÉRIEUR reproduisent le dépassement du budget global :
`/tmp/b90-pwrzxtlp` (candidat) et `/tmp/b90-j59173pe` (ancien release). Aucun
assouplissement de production ni retrait de test pour masquer ce résultat ; le
test lecteur ainsi que les tests transport EOF/stop restent verts. L'adaptation
de cet ancien scénario permission est un suivi distinct.

Adoption demandée par l'utilisateur : remplacement atomique du seul binaire
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget` après sauvegarde.
SHA-256 release installé et raccourci `/Users/moi/.local/bin/bridget` identiques :
`5e8383ea666a0043ddefedc13085427f0ecf749dd8f407b777b5279e14ee0ebc`.
Ancien release : `43296aefe68aa137a97390c870292ec29c7800ada709e09e2f1f0aae6db1d1fa`,
conservé dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/.install-codex-subagents.Gf9ONr/bridget.previous`.
Le SHA attach 092 reste `acbc3e690457a27a933e3ab174d6a33dd278efca6894722fb9a881bec22e3f31` : aucun changement de ce lot.
Le pilote Codex final est `2881ba0fb818688322b803d238d928a5ecd57ddf17f32fc6ad1657121e7d637b`.

Skill canonique publiée également dans le dépôt actif, sans changer ses liens
Codex/Claude/agents. `quick_validate.py` : PASS ; copie publiée identique à celle
du worktree. Cette mise à jour étroite suit le guide skill-creator : retrait de
la restriction, aucun nouveau mécanisme d'autorisation/gestion de sous-agents.
Les README et artefacts versionnés restent dans le worktree avec le lot 092.
Aucun redémarrage de production, aucun commit, aucune nouvelle identité Bridget
ni aucun registre de sous-agents. Le daemon 73764, l'équipier 74282 et le wrapper
humain 30818 ont conservé leurs PID. Le wrapper humain déjà chargé reste l'ancien
code jusqu'à sa reprise volontaire ; aucun `spawn_agent` de recette n'a été tenté
dans ce processus encore ancien.
# Correctif reprise par nom — 07/09/2026, installé

`thread/list` demande désormais `useStateDbOnly: true` sur toutes ses pages :
lecture du catalogue public sans scan/réparation des historiques Codex. Cela
conserve les bornes, le menu, la comparaison exacte et les refus d'ambiguïté.
Le scan lent a été mesuré sur le CLI 0.153.4 ; le catalogue complet de 364 fils
sans scan revient en 1,811 s. Aucun cache ni accès direct à la base depuis Bridget.
Preuves dans `verification-reprise-catalogue.md` : rouge puis vert, vraie TUI
nom/menu/refus, 1252 tests workspace réussis, fmt/clippy verts et release installé
sans redémarrage de flotte. Les sections suivantes conservent les preuves historiques.
