# Implémentation 090 — Implemented, adoptée le 2026-09-06

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
