# Journal de réalisation — 089

## 2026-09-05 — T029 : frontières Unix, réception bornée et accusés corrélés

Les CLI de communication et MCP partagent maintenant la connexion Unix, la deadline avant connect, l'écriture avec budget restant et une seule lecture JSONL bornée. Le daemon réutilise cette lecture : **16 Mio maximum par ligne filaire, LF inclus**, pas de trame partielle ; le guichet conserve sa limite plus stricte de **64 Kio** (CR et LF comptés). Une trame commencée doit finir dans 10 s ; l'inactivité entre deux trames d'un wrapper n'a pas ce timeout. Aucun schéma ni octet canonique n'est modifié. Le statut historique possède encore son lecteur spécialisé : consolidation à T035, pas de prétention « toutes les connexions sont déjà unifiées ».

Défauts vérifiés avant correction : quatre tests clients initialement rouges (EOF sans LF accepté ; réponse sans borne ; lecture goutte-à-goutte renouvelant le budget ; diagnostic recopiant un canari). Un cinquième défaut a été reproduit via le processus MCP réel : un JSON décodable mais de mauvaise famille était classé daemon_protocol après transmission. La couche partagée exige désormais IdempotencyResult, operation_kind=send et la même clé ; absence d'accusé VALIDE = outcome_unknown, avec id/issued_at conservés. Erreur de lecture/décodage/corrélation = connexion empoisonnée et fermée ; une réponse tardive déjà bufferisée ne peut devenir l'accusé suivant. Les catégories déterministes corrélées restent intactes.

`core_089_security_test` : **5/5**, 0,34 s (compilation 2,15 s). Vrai daemon pour permissions 0700/0600, substitution symlink/fichier public refusée avant socket, sentinelle privée inchangée ; trame sans LF/hors borne sans réponse et daemon encore accessible ; borne guichet exacte puis +1 ; capacité non héritée ; portée de dépôt valide avec émetteur voisin refusée DeclaredSenderMismatch, zéro ligne dans dépôt/demandes/ledger. MCP binaire réel contre pair fautif explicitement synthétique : sans LF, type inconnu, autre famille, autre clé, autre opération → outcome_unknown sans canari. Aucun faux serveur présenté comme preuve de durabilité daemon.

Contrôles complémentaires : client privé **5/5**, 0,22 s ; lecture JSONL transport **2/2** ; propriétaire attendu **1/1**, 0,00 s ; sortie BrokenPipe **1/1**, 0,04 s. Le dernier traverse réellement handle_connection et observe la map de capacités vide après erreur puis après seconde connexion ; son ancien setup agent-2 était invalide depuis UUID v2, remplacé par un état sans agent ni changement global de HOME. La preuve propriétaire utilise les métadonnées d'un vrai fichier 0600 avec UID attendu différent injecté dans le prédicat partagé, **pas un chown réel privilégié**. Les trois préflights de production passent geteuid à ce même prédicat. Les limites de confiance même-UID et SSH de threat-model.md restent inchangées.

Non-régression ciblée, mêmes binaires : contrat CLI/MCP **3/3**, 1,36 s ; ledger **2/2**, 0,45 s ; skill exécutable **1/1**, 1,23 s. Tous sous environnement privé T014, offline/locked, watchdog 180 s. Commandes : `cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_security_test --test core_089_contract_test --test core_089_ledger_test --test core_089_skill_test -- --nocapture`, puis sélections lib `communication::client::security_tests`, `service_capability_is_cleaned_after_a_broken_response_socket` et `fichier_prive_ne_suffit_pas`. Une entrée ignorée par harnais est le worker privé des bancs, pas un scénario sécurité désactivé.

Difficultés de harnais conservées : le premier faux serveur MCP attendait à tort un Register avant RoleHandshake ; corrigé d'après execute_send et la trame réellement reçue. Le listener macOS transmettait son mode non bloquant au socket accepté, entraînant WouldBlock puis fermeture prématurée ; passage explicite en mode bloquant borné côté harnais. Côté client, poll avant remplissage évite le setsockopt sur EOF que macOS peut refuser par EINVAL alors que les octets complets restent lisibles. Aucun délai de grâce ajouté pour rendre le test vert.

## 2026-09-05 — T028 : charge locale/SSH et budget historique

Source : 61da1b7 + présent lot test-support/harnais (aucun changement de comportement release). Daemon réel en enfant séparé, wrapper ACP réel dans le processus instrumenté, fournisseur synthétique compté, une connexion attach locale et une connexion attach distante via le transfert SSH livré. Émission de 200 tours idempotents pendant 60 s : trois événements par tour, un tour toutes les 300 ms, soit **10 événements/s en moyenne, en petits groupes de trois**, pas un flux prétendument uniforme à 100 ms. Les remises différées ne bloquent pas la cadence : chaque issue est ensuite vérifiée Accepted par rejeu des bytes d'origine, sans nouvelle injection. Les séquences attendues sont exactement 1..600 sur les deux vues et les bytes rendus sont identiques.

Le rendu mesuré est celui du consommateur public attach : décodage, assemblage, écriture de la ligne JSONL puis flush. Ce n'est ni une mesure GUI, ni la latence d'un modèle externe. Origine locale = Instant post-flush du journal, avant publication au relais. Sonde test-support nouvelle « tous événements », distincte de l'échantillonnage historique start/end de SC-005 ; test dédié vérifiant respectivement les séquences [1,3] et [1,2,3]. La durabilité électrique du disque n'est pas mesurée.

Horloges : sept allers-retours sur une connexion SSH DÉJÀ établie, avant et après la campagne. Estimation par milieu du meilleur aller-retour ; incertitude conservative = maximum des demi-RTT + dérive mesurée. Aucune durée négative écrêtée. Première méthode rejetée pour le chiffre publié : inclure le démarrage SSH/Python donnait une estimation négative et ±113 ms, quoique le majorant restât sous le budget. La méthode resserrée donne +7,285477 ms avant, +7,589852 ms après, ±3,389875 ms retenus.

| Mesure exécutée | Résultat | Budget inchangé |
|---|---|---|
| Local, 600/600 reçus | p95 12,242375 ms ; max 13,064542 ms | p95 <1 s ; max <3 s |
| Linux, 600/600 reçus | p95 brut 23,077190 ms ; corrigé 15,791713 ms ; max corrigé 19,051622 ms | p95 <3 s |
| Majorant distant p95, incertitude comprise | 19,181588 ms | <3 s |
| Pertes / doublons / différences d'octets | 0 / 0 / 0 ; 200 issues Accepted | Aucun |

Rapport complet des 600 échantillons par mesure : `artifacts/charge-600-local-linux.json`. Durée du test 73,34 s, dont 60 s d'émission ; compilation 1,35 s. Commande exacte, dans le worktree 089 (les trois répertoires locaux étaient créés en 0700 avant la commande) :

```sh
env -i HOME=/private/tmp/b089load-7hSrWe/provider BRIDGET_HOME=/private/tmp/b089load-7hSrWe/state BRIDGET_SOCKET=/private/tmp/b089load-7hSrWe/state/bridget.sock TMPDIR=/private/tmp/b089load-7hSrWe/tmp CARGO_HOME=/Users/moi/.cargo RUSTUP_HOME=/Users/moi/.rustup PATH=/Users/moi/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin BRIDGET_SSH_LOAD_GATE=1 BRIDGET_SSH_REMOTE_GATE=1 BRIDGET_SSH_REMOTE_HOST=37.59.185.67 BRIDGET_SSH_REMOTE_USER=moi BRIDGET_SSH_REMOTE_PORT=2222 BRIDGET_SSH_IDENTITY=/Users/moi/.ssh/id_ed25519 BRIDGET_SSH_KNOWN_HOSTS=/Users/moi/.ssh/known_hosts BRIDGET_SSH_REMOTE_PARENT=/home/moi BRIDGET_SSH_REMOTE_BIN=/home/moi/bg089-cb62fdf/bin/bridget /usr/bin/perl -e 'alarm 180; exec @ARGV' /Users/moi/.cargo/bin/cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_federation_test charge_locale_distante -- --include-ignored --test-threads=1 --nocapture
```

Pour reproduire, créer une NOUVELLE racine `/private/tmp/b089load-XXXXXX` et remplacer ses quatre occurrences : ne jamais réutiliser une base déjà mesurée. La commande configure un enfant de test avec la boucle daemon de production et un disjoncteur porté explicitement à 10 000 échanges. Diagnostic préalable conservé : avec le réglage normal 8/180 s, SQLite attestait 8 Accepted et 192 refus circuit_breaker ; ce résultat n'est pas un test de charge réussi. Aucune hausse du disjoncteur de production.

**SC-005 conservé, pas remplacé par la charge SSH.** Le harnais historique crée désormais un processus/namespace par banc, daemon récolté, UUID v2 et fichiers privés ; plus de daemon-thread survivant ni changement global d'environnement. Le parent alterne toujours les envois entre les DEUX bancs simultanés. 100 tours de chauffe, 1 000 mesurés, deux bornes par tour, cinq paires internes inchangées. Les instants append/rendu restent dans le même worker pour SC-001 ; pas de conversion approximative entre horloges de processus. La jonction SC-002 garde le témoin SnapshotCaughtUp unique AVANT le tour live, séquences [5] puis [5,6,7,8].

Commande sous l'environnement privé T014 et watchdog 420 s : `cargo test --offline --locked -p bridget-daemon --features test-support --test sc005_attach_budget -- --test-threads=1 --nocapture` : **3/3**, 33,97 s, compilation 2,49 s. Deux entrées ignorées sont des workers réellement lancés par les parents ; la troisième est la campagne historique SC-001 de 21×60 s, **non exécutée ici**, conservée pour recette explicite. p95 médian sans vue 20,875 µs ; avec deux vues 22,667 µs ; deltas APPARIÉS [-917,-708,334,542,1792] ns ; médiane 334 ns <= max(5 % × médiane baseline, 5 µs) = 5 000 ns. Aucune exemption sous 100 µs ni modification du seuil.

`cargo test --offline --locked -p bridget-transport --features test-support --lib journal::tests:: -- --test-threads=4` : 15/15, 0,39 s (compilation 7,93 s). Clippy workspace/all-targets/test-support -D warnings vert, 8,87 s ; fmt check vert ; gel 17 fichiers + six mutants vert. Les essais ayant échoué ont conduit à corriger le harnais : garde de PID adaptée aux seuls workers enfants exacts, pas d'élargissement aux processus de la flotte ; fichiers historiques de fixture créés privés ; socket de contrôle acceptée repassée en mode bloquant avec timeout (héritage non bloquant macOS).

Nettoyage : daemon/wrapper/SSH de la recette finale arrêtés et récoltés, namespace distant retiré par contrôle de type/UID/inode et refus de connexion, puis rmdir uniquement si vide. Huit namespaces distants résiduels des essais précédents ont été contrôlés et retirés de même. Aucun processus worker restant au contrôle ps. Les diagnostics locaux restent privés jusqu'au nettoyage final T035 ; aucune donnée utilisateur effacée. Le seul déploiement client privé `/home/moi/bg089-cb62fdf` reste disponible pour la recette finale, aucun service système installé.

## 2026-09-05 — T027 : coupure SSH et reprise sans réinjection

La recette distante étend T026 sans remplacer son scénario : journal historique de fixture seq=5 puis vrai tour seq=6..8 ; arrêt/récolte du SEUL SSH, ledger et relève en erreur explicite sans donnée/fraîcheur inventée. La socket stale est nettoyée dans le harnais seulement, après refus de connexion et vérification du couple device/inode relevé AVANT coupure, type et propriétaire ; le script livré conserve StreamLocalBindUnlink=no. Nouveau tunnel au même chemin, retry CLI avec les mêmes id/issued_at/cible/corps : accepted, un seul prompt et une ligne de ledger. Relève Seq(8) inclusive : exactement les mêmes bytes de l'événement 8 puis UN SnapshotCaughtUp. Connexions ACTOR/ACP_AGENT et PID enfant fournisseur du wrapper inchangés : le tunnel ne redémarre pas les agents.

Puis retrait de l'UNIQUE fixture historique (rétention simulée) : la relève distante Seq(5) commence par Gap(5,5), pas une indisponibilité ni une fraîcheur. Mutation réellement compilée/exécutée du daemon local : remplacer ce Gap par AttachRejected/JournalUnavailable fait échouer le client distant en 6,15 s. Source daemon restaurée, `git diff -- daemon.rs` vide AVANT validation finale. Aucun changement produit dans ce lot, seulement le harnais.

Commande T026 avec les MÊMES paramètres distants, ajouter BRIDGET_SSH_LOCAL_GATE=1 et sélectionner l'intégralité du binaire avec `-- --include-ignored --test-threads=1 --nocapture` : 3/3 en 11,50 s (compilation 2,40 s). Reconnexion seule auparavant 1/1 en 7,24 s. Clippy workspace/all-targets/test-support -D warnings vert ; fmt contrôlé. Les trois gates SSH ignorés par défaut ont donc été exécutés, pas comptés à partir d'un skip. La coupure est un arrêt du tunnel, pas un crash daemon ; les crashs SIGKILL restent les preuves T017/T018/T019.

## 2026-09-05 — T026 : macOS ↔ Linux exécuté, sans flotte historique

Déploiement exécuté depuis le worktree 089 propre à cb62fdf :

```sh
bash scripts/deploy-remote.sh --label core-089 --host 37.59.185.67 --user moi --port 2222 --identity /Users/moi/.ssh/id_ed25519 --known-hosts /Users/moi/.ssh/known_hosts --source /Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core --remote-prefix /home/moi/bg089-cb62fdf --remote-cargo /home/moi/.cargo/bin/cargo
```

Client Linux construit hors ligne en 47,38 s avec Rust 1.92.0 déjà installé. Binaire privé `/home/moi/bg089-cb62fdf/bin/bridget` ; aucun daemon, wrapper, profil, skill ou service distant démarré. Cette copie demeure pour T027/T028, distincte du namespace de chaque recette. Le gate a révélé le SHA complet injecté par le script contre les 12 caractères du build local : même représentation désormais utilisée, verrouillée par la doublure Cargo. Les écarts du binaire de test local dirty restent honnêtement avertis, pas masqués.

Recette exécutable (préfixer avec l'environnement privé HOME/BRIDGET_HOME/TMPDIR/CARGO_HOME/RUSTUP_HOME/PATH documenté T014 et watchdog 180 s) :

```sh
BRIDGET_SSH_REMOTE_GATE=1 BRIDGET_SSH_REMOTE_HOST=37.59.185.67 BRIDGET_SSH_REMOTE_USER=moi BRIDGET_SSH_REMOTE_PORT=2222 BRIDGET_SSH_IDENTITY=/Users/moi/.ssh/id_ed25519 BRIDGET_SSH_KNOWN_HOSTS=/Users/moi/.ssh/known_hosts BRIDGET_SSH_REMOTE_PARENT=/home/moi BRIDGET_SSH_REMOTE_BIN=/home/moi/bg089-cb62fdf/bin/bridget cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_federation_test deux_machines -- --include-ignored --nocapture
```

Résultat final 1/1 en 4,44 s, compilation 1,96 s. Daemon macOS enfant sous `/private/tmp/bid-<uuid>/state`, tunnel -R vers `/home/moi/bg089-bid-<uuid>/peer.sock` sous compte Linux autorisé. Vrai CLI Linux : who puis agents --json UUID ; send suivi avec corps UTF-8/apostrophe/newline ; réponse liée ; les deux retries donnent accepted ; deux messages uniques et une demande answered dans le store maître. Ledger CLI macOS/Linux identique octet pour octet. Un troisième envoi traverse le VRAI wrapper ACP vers un fournisseur synthétique compté : un prompt ; client Python public côté Linux via la socket tunnelée relève journal séquences 1..3, payloads comparés byte à byte au fichier du wrapper. Cela prouve SSH et le wrapper, pas un compte fournisseur supplémentaire ni une nouvelle recette Codex/Claude.

Nettoyage nominal : arrêt/récolte SSH, daemon, wrapper ; socket distante retirée uniquement après ConnectionRefused, type/propriétaire/inode contrôlés ; rmdir du seul tmp privé vide créé par le CLI puis de la racine vide. Aucun rm récursif distant. Les échecs intermédiaires ont conservé leurs répertoires privés de diagnostic, pas de processus ; ils seront inventoriés au nettoyage final. L'assert initial de who cherchait l'UUID dans la vue humaine : remplacé par la projection agents --json ; les docs donnent maintenant cette commande pour trouver une adresse.

Tests scripts après correction build-id : 23/23 en 3,249 s. Clippy workspace/all-targets/test-support -D warnings vert en 6,45 s ; fmt vérifié. La mesure de charge 600 événements, le biais d'horloge et la reprise du curseur après coupure restent T027/T028 ; les 4,44 s de ce scénario ne sont PAS un p95 ni une mesure de livraison fournisseur.

## 2026-09-05 — T025 : transfert Unix à travers OpenSSH réel

`BRIDGET_SSH_LOCAL_GATE=1 cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_federation_test -- --include-ignored --nocapture`, environnement privé T014, watchdog externe 180 s : 1/1, 0,45 s, compilation 1,84 s. sshd OpenSSH 10.0p2 enfant sans privilèges, port loopback dynamique, clés hôte/client neuves, authorized_keys de fixture sous target et StrictModes conservé, aucun ~/.ssh ou /etc/ssh modifié. Le script de production établit le vrai transfert -R. CLI et client protocole lisent le même annuaire/ledger byte-identique, socket distante 0600/racine 0700 ; après arrêt/récolte du seul client SSH, lecture en échec sans DB locale, daemon et présence locale toujours disponibles. Clôture/récolte du sshd et du daemon puis nettoyage des seules racines du test. Il n'y a encore ni deuxième machine ni fournisseur dans CET oracle.

Deux refus réels ont guidé le harnais : StrictModes refuse AuthorizedKeysFile sous le parent partagé /private/tmp, donc clé de test déplacée sous target privé ; AllowTcpForwarding=no désactive aussi l'ACL Unix dans OpenSSH 10 (session.c::do_authenticated, https://github.com/openssh/openssh-portable/blob/V_10_0_P2/session.c#L329-L344). Le sshd de fixture autorise remote mais borne le TCP à 127.0.0.1:1 (inutilisé), avec PermitOpen none ; le produit n'élargit aucune configuration système.

Le gate a également détecté une garde de script trop stricte : le vrai bind du daemon sous umask 077 crée une socket 0700, pas 0600. Le script accepte désormais ces DEUX modes privés et le même propriétaire, sans chmod ; aucun droit groupe/autres admis. Oracle supplémentaire : test-federate-ssh passe maintenant 23/23 en 3,034 s. OpenSSH laisse une socket stale après fermeture du transfert : fait conservé explicitement, aucun unlink automatique ajouté. La reprise contrôlée à mêmes identifiants/journal reste T027, pas déclarée couverte par une simple reconnexion réussie.

## 2026-09-05 — T024 : scripts SSH privés, effets bornés

federate-ssh run reste au premier plan et exige racines/socket/label/clés explicites. Aucun launchd, configuration SSH héritée, suppression de socket stale ou remplacement d'une cible occupée. Préflight propriétaire/0700/0600, composants sans symlink, alphabet de chemins fermé, socket courte ; StrictHostKeyChecking=yes et known_hosts explicite sans écriture. Le transfert reste une socket Unix, pas un protocole réseau supplémentaire. Le masque de la socket distante relève du serveur SSH et reste à constater au gate réel.

deploy-remote est client-only, préfixe neuf, source Git explicite et fichiers suivis uniquement ; aucune installation Rust, pas de fichiers non suivis/profils globaux ni secret copié volontairement. Cargo doit être déjà disponible avec cache : --locked --offline, pipefail empêche tail de masquer l'échec. L'identité de build accompagne la source dépourvue de .git. Les gardes ne prétendent pas empêcher une modification concurrente du dépôt par le même compte durant le transfert ; exécuter depuis une tranche figée.

`bash scripts/test-federate-ssh.sh` : 22/22 en 2,583 s. SSH/Git/Cargo sont des doublures ; les scripts distants sont réellement exécutés dans des répertoires privés et le parseur du VRAI rsync -e est exercé avec chemins à espaces et faux shell, sans serveur. Les permissions, cibles occupées/stale, injections, ancienne syntaxe, dry-run strict sans effets, non-suivis exclus et erreurs de build/forwarding sont vérifiés. Les deux scripts passent bash -n. Ces tests ne sont PAS une preuve SSH interserveur, réservée à T025–T028.

Préflight réseau lecture seule distinct : SSH authentifié avec la clé existante, contrôle strict des clés d'hôte et sans configuration héritée vers moi@37.59.185.67:2222 ; Linux x86_64, utilisateur moi, Cargo 1.98.0 détecté. Aucun déploiement, service ni socket distante créé à cette étape.

## 2026-09-05 — T023 : skill exercée, documentation communication seule

La skill du dépôt est courte et n'orchestre aucune tâche métier : annuaire UUID, envoi/réponse liée, reçu et retry identiques, lecture maître et signaux inconnus conservés. Aucun fichier de skill global n'est modifié. Les README FR/EN remplacent les surfaces du produit complet et distinguent les recettes réellement passées de Claude/GLM/SSH encore ouverts. Le quickstart est aligné sur l'isolation déjà livrée (BRIDGET_HOME/BRIDGET_SOCKET), sans installation implicite.

Le nouveau core_089_skill_test lit et exécute les QUATRE blocs JSON de SKILL.md, sans copie locale des exemples : deux pairs publics, vrai daemon et vrais processus MCP. in_flight avant ACK, accepted après ACK, retry exact, deux lignes uniques de ledger, demande open puis answered et lecture CLI cohérente. La mutation réelle supprimant in_reply_to de l'exemple publié échoue en 0,63 s (open au lieu d'answered) ; exemple restauré avant passe verte. La commodité CLI reply cible le dernier expéditeur : les exemples privilégient send --to UUID --in-reply-to pour éviter cette ambiguïté, sans changer le protocole.

Environnement privé T014, watchdog 180 s : `cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_skill_test --test core_089_ledger_test --test core_089_reply_test` : 4/4 ; skill 1,20 s, ledger 0,55 s, réponse 7,27 s, compilation 1,90 s. `quick_validate.py skills/bridget` valide la skill. Clippy workspace/all-targets/test-support -D warnings vert 5,45 s ; fmt contrôlé. Le scénario ne lance ni Maicie, ni fournisseur, ni tunnel : les paires simulent uniquement les extrémités publiques et accusent explicitement les remises.

## 2026-09-05 — T022 : ledger maître unique, golden exécuté

Le test réel sans socket rendait `Ledger vide.` avec exit 0 et créait une base cliente (rouge en 0,16 s). Suppression de CE repli dans cmd_ledger conformément à FR-08905/US3 : erreur explicite sans données stdout ni nouvelle DB. Le renderer ne change pas. C'est notamment nécessaire pour ne pas masquer une coupure SSH par une base vide.

`core_089_ledger_test` : CLI et MCP binaires → daemon réel → même projection de deux messages, corps multiligne/UTF-8 et champs de DTO exacts ; golden CLI indépendant byte-identique, borne 1 et notice d'omission inchangées ; demandes entrantes/sortantes, portée globale distincte et limites des deux collections. Les données de lecture sont des fixtures SQL, avec timestamps fixes en 2100 pour ne pas être supprimées par la rétention au bootstrap ; ce n'est pas une preuve d'émission, assurée séparément par T014–T018. Les deux tests passent en 0,48 s (compilation 2,45 s).

`cargo test --offline --locked -p bridget-daemon --features test-support --lib ledger -- --test-threads=4` : 17/17 en 0,08 s. L'ancien cas daemon enregistrait inutilement `agent-2`, refusé depuis les UUID v2 ; il réutilise maintenant l'autre fixture d'état existante sans présence. Aucun assouplissement du validateur d'identité, aucun changement des autres harnais. Clippy workspace/all-targets/test-support -D warnings : vert 5,27 s. Environnement privé et watchdog 180 s identiques à T014. T020/T021 restent ouverts sur recettes de compte, pas dissimulés par cette tranche indépendante.


## 2026-09-05 — T020 partielle : Codex réel vert, authentification Claude ouverte

Le harnais historique codex_native_test devient core_089_native_test : mêmes oracles effort/limite/absence de signal, mais UUID v2, pair adressable (pas CLI éphémère pour reply), racine/socket indépendantes, enfants groupés suivis, logs 0600, répertoire fournisseur neuf. Nettoyage normal désormais par arrêt du daemon puis attente des wrappers ET vérification de disparition des PID/groupes fournisseur réellement observés. Les tests unitaires de pilotes sont distincts de la recette de compte.

CLI locaux observés : Codex 0.153.4, Claude Code 2.1.258 ; aucune installation ni mise à jour. `codex login status` indique ChatGPT ; `claude auth status` indique claude.ai/Max. Gate Codex : uniquement copie privée du fichier auth explicite (effacée par garde même après panique), sans config/MCP utilisateur, clés API exclues, permissions deny, modèle terra et effort low demandés explicitement. Le premier essai n'avait aucun effort configuré et le fournisseur renvoyait None ; aucun effort n'a été inventé pour passer le test. La recette corrigée reçoit le fait low et la limite réelle.

Environnement privé T014, variables supplémentaires `BRIDGET_CODEX_NATIVE_GATE=1 BRIDGET_CODEX_APP_SERVER_BIN=/opt/homebrew/bin/codex BRIDGET_TEST_CODEX_AUTH_FILE=/Users/moi/.codex/auth.json` : `cargo test -p bridget-daemon --features test-support --test core_089_native_test codex -- --include-ignored --test-threads=1` : 3/3, 5,87 s, compilation 1,75 s. Réponse liée réelle terra, annuaire/CLI who, journal par attach et zéro processus fournisseur survivant. `cargo test --offline --locked -p bridget-transport --lib codex_app_server::tests:: -- --test-threads=4` : 50/50 en 4,04 s ; même commande claude_stream_json::tests:: : 22/22 en 0,09 s. EOF en plein tour, arrêt du groupe, annulation et provenance brute sont exercés par ces pilotes avec processus synthétiques indépendants des comptes.

**Claude réel NON validé.** Test explicite avec `BRIDGET_TEST_CLAUDE_KEYCHAIN_SERVICE='Claude Code-credentials' BRIDGET_TEST_CLAUDE_KEYCHAIN_FILE=/Users/moi/Library/Keychains/login.keychain-db BRIDGET_TEST_CLAUDE_BIN=/Users/moi/.local/bin/claude`, lecture du trousseau seulement et jeton transmis au CLI officiel sans persistance ni affichage. Journal fournisseur du test `/private/tmp/bid-ce1f17e68de9/provider/.claude/projects/-private-tmp-bid-ce1f17e68de9-provider/95f544f2-1502-4f61-9582-b8bf33f335b8.jsonl` : `authentication_failed`, `Not logged in · Please run /login`. L'événement Bridget api_error est donc un refus réel, pas une réussite ni un blocage de la socket. Une vérification utilisateur de la connexion Claude a été demandée ; pas de refresh/modification du trousseau ni de substitution facturée. T020 reste ouverte, pas de recette GLM déclarée.

La documentation officielle décrit CLAUDE_CODE_OAUTH_TOKEN comme authentification d'abonnement du CLI : https://code.claude.com/docs/en/authentication et https://code.claude.com/docs/en/env-vars (consultées le 2026-09-05). Ce constat ne garantit ni quota restant ni identité du budget interactif/non interactif ; le produit ne promet pas ces propriétés commerciales.


## 2026-09-05 — T019 : attache brute, rotation et curseur périmé

Le nouveau core_089_attach_test traverse daemon et wrapper enfants réels, fournisseur synthétique compté : fixture JSONL du jour précédent avec espaces/UTF-8/champ inconnu → SnapshotCaughtUp(seq=5) → seulement ensuite envoi → journal live seq=6..8, sans seconde bascule ni doublon. Comparaison indépendante des bytes du fichier et des fragments ; le SEUL LF délimiteur est exclu selon le codec historique. Cela prouve les octets du journal, pas que le journal contiendrait toutes les notifications brutes d'un fournisseur (frontière native distincte couverte à T003/T020).

Oracle écrit avant correction : après retrait de l'unique fixture historique (rétention simulée), SubscribeSeq(5) repartait à 6 sans Gap ; rouge réel en 1,44 s. Le relais suit désormais la prochaine séquence attendue du snapshot, annonce le trou avant le fragment suivant et refuse une séquence rétrograde. Les événements trop grands avancent ce témoin sans doubler leur Gap. Pas de modification du relais mémoire, du canon filaire ni de la politique de fraîcheur. L'attente d'activation vérifie JournalReady via le refus typé JournalUnavailable, pas seulement Register.

Environnement privé T014/watchdog 180 s : `cargo test -p bridget-daemon --features test-support --test core_089_attach_test --test coordination_events_test -- --include-ignored --test-threads=1` : 7/7, coordination 6,35 s (dont deux SIGKILL), attache 1,48 s, compilation 2,59 s. Mutations réellement exécutées du daemon : Gap→Unavailable puis Gap→SnapshotCaughtUp, toutes deux refusées par l'oracle réel (0,09 s et 0,17 s). Daemon restauré, diff vide, AVANT la passe verte.

Six tests unitaires voisins du relais passent : bascule_snapshot, perte_du_flux, lignes_illisibles, today_et_date, troncature_et, disparition_d_un ; respectivement 0,03/0,04/0,23/0,04/0,06/0,04 s. Six autres sélectionnés par relais_ passent en 0,09 s. Clippy workspace/all-targets/test-support -D warnings : vert 5,13 s ; fmt et vérificateur 17 fixtures/six mutants verts. Nouvelle fixture 089 indépendante, aucune fixture amont réécrite. Les anciens bancs SC-005 ne sont pas comptés comme exécutés ici : leurs harnais sont encore à porter pour T028/T034.


## 2026-09-05 — T018 : clôture service sous UUID et reprise brute

La dette identifiée à T014 est corrigée : reply_guichet ne passe plus le nom historique maicie au helper de clôture. Dans LA transaction IMMEDIATE déjà existante, il lit l'émetteur de la demande liée dont target égale le déposant attesté. Le helper partagé contrôle toujours sender/target/state=open, puis l'événement s'écrit dans cette même transaction. Pas de nouvel UPDATE de clôture, pas de relecture d'annuaire ou de nom humain. La capacité, owner, token, génération et lease ne changent pas. Le rapport demeure durable lorsqu'une demande est déjà terminale, sans inventer un answered.

Oracle AVANT correction : la faute d'écriture d'événement étendue à l'émetteur UUID ne se déclenchait même pas, résultat indûment accepted (test rouge en 0,03 s). Après correction : rollback replied+answered sur faute SQL pour noms historiques ET UUID ; demande d'un autre destinataire inchangée et terminal jamais rouvert. La couture réelle CLI → service externe → replied/answered/lifecycle passe maintenant. SIGKILL du daemon après événement, redémarrage et nouvel abonnement : ligne JSONL reçue byte-identique, même event_id et LF. Sept scénarios de service au total, dont quatre opérations refusées sur une nouvelle connexion sans capacité. Les six scénarios historiques ne sont pas supprimés, seulement déplacés.

Environnement privé T014/watchdog 180 s : `cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_storage_test --test core_089_service_test --test core_089_reply_test` : 11/11, réponse 7,27 s, service 0,48 s, stockage 0,06 s ; compilation 3,74 s. Clippy workspace/all-targets/test-support -D warnings et fmt contrôlés. Aucun processus de service Maicie n'est requis, aucun import de sa crate ni lecture de sa base. Les captures proviennent de clients publics indépendants et du binaire daemon isolé ; les SIGTERM du test de nettoyage restent des arrêts propres, seuls les appels kill documentés constituent les crashs.


## 2026-09-05 — T017 : cinquante crashs et douze scénarios portés

idempotency_crash_test.rs devient core_089_crash_test.rs (les douze scénarios restent présents). Environnement privé T014, watchdog 420 s et watchdog interne 360 s : `cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_crash_test -- --include-ignored --test-threads=1` termine en 182,59 s, compilation 2,35 s. **La matrice N=50 réussit, un scénario séparé de reconnexion échoue** : le fournisseur synthétique sortait après son premier prompt. Le journal /private/tmp/bid-b120b3ff371a/wrapper.log et le script de fixture confirment la sortie/BrokenPipe ; ce n'est pas un blocage du daemon.

Correction TEST uniquement : ce témoin attend désormais un deuxième prompt éventuel (il ne le reçoit pas au nominal), la reconnexion est observée, puis le compteur final est lu APRÈS arrêt/join du vrai wrapper. L'ancien sleep 250 ms disparaît. Ce scénario passe seul en 2,82 s. Nouvelle passe des onze scénarios courts : même commande sans --include-ignored, 11/11, 7,16 s ; compilation 1,82 s. La matrice N=50 déjà exécutée n'a aucun changement de logique, de fixture ni de délai dans cette correction ; son succès est une preuve séparée, pas attribué au skip de la passe courte.

Clippy workspace/all-targets/test-support -D warnings vert (4,77 s), fmt --check et vérificateur 17 fixtures/six mutants verts. Aucun changement de code produit. Les quatre barrières SIGKILL, les cinquante prompts uniques, les canons et issues terminales identiques ne prouvent pas un exactly-once universel : un crash fournisseur après effet et avant son propre accusé reste potentiellement ambigu. Le skip Linux/kqueue demeure explicite, pas une recette Linux déclarée verte.


## 2026-09-05 — T016 : réponse liée et arrêt des rappels exécutés

core_089_reply_test utilise les vrais CLI/MCP/daemon et deux pairs du protocole public. La demande naît par CLI, pas par INSERT de fixture. Réponse MCP in_flight : demande encore open avant ACK ; ACK réel : answered ; retry exact : même issue, une entrée ledger visible par SQL, CLI et MCP. Une seconde demande est annulée via le protocole et sa cible reçoit CancelDelivery. Une troisième demande témoin traverse les vrais rappels puis expire ; la réception sur socket et la lecture RequestList constituent la barrière, sans sleep de synchronisation. Les événements de rappel ne concernent QUE cette sentinelle : aucune relance de la demande answered ou cancelled. Le délai stocké est comparé à issued_at + timeout (canon), pas à l'heure d'insertion qui peut franchir une seconde.

Mutant réellement exécuté dans handle_delivery_ack : garder le pending après ACK au lieu de le retirer. L'oracle échoue en 7,13 s sur deux événements ReminderSent corrélés à la demande déjà answered. Restauration du daemon vérifiée par diff vide avant la passe finale. Aucun code de production modifié dans cette tranche.

Environnement et watchdog 180 s T014 : `cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_reply_test --test core_089_contract_test --test core_089_identity_test` : 7/7, réponse 7,22 s, contrat 1,32 s, identité 3,73 s ; compilation 2,16 s. Clippy workspace/all-targets/test-support -D warnings vert (4,69 s), fmt/diff contrôlés. Les premières erreurs du nouveau harnais (signature ListRequests, dédup de trois corps identiques, horloge de l'assert) ont été corrigées sur preuves du protocole et du code ; aucune règle produit assouplie. Les douze crash-tests historiques restent réservés à T017 et les lifecycle de service à T018.


## 2026-09-05 — T015 achevée : nom humain par le vrai CLI

La première tranche ci-dessous est historique. `bridget rename "Équipe B"` passe maintenant par la socket et une extension versionnée fermée display_name_set/result ; le client ne lit aucune base. L'autorité est l'identité/instance active de LA connexion, extraite du contrôle déjà utilisé par la lecture de contenus. Aucun agent_id cible déclarable dans la commande. Le helper d'unicité SQL est partagé avec update_profile ; la transaction IMMEDIATE ne modifie que le nom, sa clé normalisée, la révision et updated_at. Instructions historiques (espaces inclus), révision d'instructions et autres champs restent identiques. Un retry du nom courant ne modifie pas la révision.

Oracle écrit avant correction : le vrai CLI refusait le nom humain (exit de test 101, 1,74 s). Après raccord : nom projeté, refus de collision/champ inconnu/version/contrôle/instance étrangère, profil inchangé après refus, arrêt brutal du daemon puis reprise au même UUID/instance/canon ; aucun deuxième prompt. Fixture JSONL additive indépendante, les 17 fixtures amont restent inchangées. Mutant exécuté : supprimer l'UPDATE de nom en gardant la réponse Applied fait échouer ListAgents (Agent au lieu de Destinataire renommé, 1,69 s). Mutant restauré avant validations.

Commandes sous l'environnement privé et watchdog 180 s décrits à T014 :

- `cargo test --offline -p bridget-daemon --features test-support --test core_089_identity_test --test core_089_contract_test --test core_089_content_test --test core_089_isolation_test` : 21/21 ; identité 3,19 s, contrat 1,19 s, contenus 5,17 s, isolation 0,38 s ; compilation 0,41 s.
- `cargo test --offline --locked -p bridget-daemon --features test-support --lib -- mcp_identity::tests identity_migration::tests agent_profile::tests --test-threads=4` : 14/14, 0,34 s ; compilation 9,60 s.
- `cargo clippy --offline --locked --workspace --all-targets --features test-support -- -D warnings` : vert, 4,76 s après restauration.
- `cargo fmt --all --check`, `git diff --check`, `bash scripts/verify-089-contracts.sh --self-test --require-complete` : verts, 17 fichiers et six mutations refusées.

Le harnais commun extrait seulement run_command de run_isolated pour les variables d'identité du vrai CLI ; mêmes bornes et gestion des enfants. Pas de recette SSH ni de compte fournisseur réel dans T015. Le WIP des scripts de fédération est conservé séparément, non inclus dans ce lot.

## 2026-09-05 — T015 partielle : invariants d'identité exécutés

core_089_identity_test réutilise le harnais 012, sans nouveau processus de production. Deux tests avec vrais binaires daemon/MCP/wrapper et fournisseur ACP déterministe :

- Mise à jour du nom humain via LA primitive AgentProfileStore::update_profile conservée, puis lecture par ListAgents ; collision et nom vide refusés, profil identique après refus. Après Accepted du vrai wrapper et observation du nouveau nom, SIGKILL du daemon, redémarrage et reconnexion du même wrapper : identité-file inchangée, nom humain conservé, MCP relancé sous la même instance → issue terminale et tous les champs du record identiques, compteur session/prompt toujours à un.
- Deux processus MCP du même binaire, même principal et même clé, instances différentes → deux scopes/records et deux remises distinctes ; chaque retry réutilise son propre résultat, exactement deux prompts au total. Mutant exécuté : dériver le scope du principal au lieu de l'instance → échec, le second prompt n'arrive jamais (borne 5 s), 6,53 s pour le scénario. Restauration de mcp.rs vérifiée contre HEAD.

Première passe corrigée à partir des réponses réelles, sans modifier la production : état de remise en vol = in_flight ; who retourne agents, pas un status fictif ; Accepted ne promet pas delivery_id. L'unicité des remises est donc vérifiée dans send_deliveries, pas déduite d'un champ absent. Le profil de l'émetteur est créé par son vrai appel MCP who/Register, pas par un INSERT de test.

`cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_identity_test -- --test-threads=2` sous l'environnement isolé T014 : 2/2, 3,05 s (compilation 1,56 s), avant mutation. **T015 reste ouverte** : l'ancien cmd_rename refuse toujours tout renommage et renvoie vers les réglages GUI retirés. La primitive testée n'est PAS une surface CLI ; il reste à raccorder une commande de nom affiché à la socket avec autorisation d'instance, sans réintroduire le renommage de route ni lire la base depuis le client. Aucun nouveau type filaire inventé dans cette tranche de preuve.

Après restauration : `cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_identity_test --test core_089_contract_test --test core_089_storage_test --test core_089_content_test -- --test-threads=4` : 14/14 ; identité 2,86 s, contrat 1,19 s, stockage 0,04 s, contenu 5,34 s ; compilation 2,07 s. Clippy workspace/all-targets/test-support -D warnings : vert 4,53 s ; fmt --check et diff --check verts. Diff mcp.rs/communication.rs vide, donc aucun mutant restant. Aucun enfant de ces harnais ne reste observé après les gates. L'état Git du dépôt original est identique au relevé initial.

## 2026-09-05 — T014 : couture CLI/MCP et canon durable

Le nouveau core_089_contract_test traverse trois clients indépendants (socket de référence, CLI binaire, MCP stdio binaire) et un daemon réel isolé. Le scope attendu est une constante du corpus, pas le résultat du helper à tester. Le test relit les canonical_bytes réellement stockés et photographie TOUS les champs des quatre tables idempotency_records/send_deliveries/tracked_requests/ledger après ACK ; chaque refus de divergence doit laisser cette photographie inchangée. Corps riche UTF-8, cible, reply, deadline, in_reply_to et issued_at sont exercés ; CLI/MCP couvrent chacun leurs champs exposés. Le JSON structuré MCP est aussi comparé au TextContent.

Défaut reproduit avant correction : un vrai dépôt CLI était refusé DeclaredSenderMismatch. Register utilise désormais un UUID et le type cli, alors que trois gardes reconnaissaient encore le préfixe cli-send-. Le helper commun vérifie maintenant le type de route ET sa propriété par la connexion. Les gardes d'usurpation explicite, d'émetteur adressable, de demande éphémère, de capacité et de portée restent en place. Aucun changement du protocole ni du canon.

Mutations réellement exécutées puis restaurées :

- Retour au préfixe cli-send- dans le helper : le test CLI réel échoue sur Deliver.from (UUID temporaire au lieu de l'identité active attendue).
- Suppression de deadline_at dans canonical_send : le corpus échoue sur la divergence deadline, devenue indûment acceptable. communication.rs est redevenu identique à HEAD.

Harnais 012 réutilisé dans tests/support/idempotent.rs, pas recopié : les douze scénarios historiques restent identiques octet pour octet depuis run_amont_cycle jusqu'à EOF (comparaison contre HEAD). Les enfants, barrières, watchdogs et durées ne changent pas. Les nouveaux tests refusent également les six ensembles partiels id/issued_at/issuer_scope AVANT la socket ; listener de test privé 0600, répertoire 0700.

Commandes dans /Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core, préfixées par `env -i HOME=/private/tmp/b9t011.7nVSnj/home BRIDGET_HOME=/private/tmp/b9t011.7nVSnj/state TMPDIR=/private/tmp/b9t011.7nVSnj/tmp CARGO_HOME=/Users/moi/.cargo RUSTUP_HOME=/Users/moi/.rustup PATH=/Users/moi/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin`. Watchdog externe Perl alarm/exec 180 s (420 s pour la matrice), watchdog interne 360 s ; aucune compilation concurrente pendant les crashs.

- `cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_contract_test -- --test-threads=3` : 3/3, 1,26 s (compilation 1,55 s).
- `cargo test --offline --locked -p bridget-daemon --features test-support --lib attribution_emetteur_cli_tests` : 7/7, 0,00 s (compilation 7,92 s).
- `cargo clippy --offline --locked --workspace --all-targets --features test-support -- -D warnings` : vert, 4,82 s.
- `cargo test --offline --locked -p bridget-daemon --features test-support --test idempotency_crash_test -- --include-ignored --test-threads=1` : 12/12, aucun ignoré, 178,18 s. La matrice complète de cinquante cycles aux quatre frontières termine avec cinquante prompts uniques ; wrapper ACP réel, fournisseur synthétique. Ce n'est pas une recette de compte fournisseur.
- `cargo fmt --all --check`, `git diff --check` et `bash scripts/verify-089-contracts.sh --self-test --require-complete` : verts ; 17 fixtures et six mutants.

Dette distincte, NON masquée : le harnais guichet historique passe désormais son dépôt CLI, puis échoue sur la clôture (5/6 réussis, 3,76 s). service_events.rs appelle encore mark_answered_in_transaction avec le nom historique maicie alors que l'expéditeur suivi est un UUID. Cette jointure relève de T018 ; elle devra conserver l'autorité du claim et l'atomicité, pas être remplacée par une clôture inconditionnelle. T014 ne prétend donc pas fermer tout SC-08902, ni la suite globale.

## 2026-09-05 — T013 : paquet source indépendant exécuté

Suppression physique du plugin Maicie (129 fichiers récupérables dans Git), de disk_trend et de l'inventaire des worktrees de l'hôte. Le nettoyage borné des ressources propres et les alertes d'espace restent en place. Le refus cleanup intervient avant le namespace ; aucune commande hôte de remplacement. Les trois manifests et Cargo.lock étaient déjà fermés à T009/T010 : aucun changement artificiel de dépendance. serde_yaml reste utilisé par reprise.rs pour lire une épingle, pas par Maicie.

Paquet neuf : /private/tmp/b089pkg.0s2Dq5/source, produit par
`bash scripts/package-089-core.sh /private/tmp/b089pkg.0s2Dq5/source`.
Liste d'autorisation : trois crates, manifests/lock/toolchain/licence, fixtures de protocole 015/016/089. Aucun .git, plugins, apps, infra, node_modules ni script historique d'installation. Les types historiques du protocole et tables de compatibilité restent volontairement présents : ce ne sont pas un moteur de projet. Le script refuse une cible existante (exit 2 vérifié), ne déploie rien et refuse les symlinks des sources.

Environnement des commandes suivantes : `env -i HOME=/private/tmp/b089pkg.0s2Dq5/home BRIDGET_HOME=/private/tmp/b089pkg.0s2Dq5/state TMPDIR=/private/tmp/b089pkg.0s2Dq5/tmp CARGO_HOME=/Users/moi/.cargo RUSTUP_HOME=/Users/moi/.rustup PATH=/Users/moi/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin`, umask 077. Cwd = paquet ci-dessus sauf Clippy/fmt et vérificateur qui opèrent dans le worktree 089. Watchdog externe Perl alarm/exec : 900 s compilation, 180 s gates, 120 s unités.

- `cargo metadata --offline --locked --format-version 1 --filter-platform aarch64-apple-darwin` : 73 paquets/nœuds transitifs, exactement trois crates locales ; aucun paquet Maicie/UI/Docker. JSON conservé dans /private/tmp/b089pkg.0s2Dq5/metadata.json.
- `cargo test --offline --locked --workspace --features test-support --no-run` : toutes les cibles compilent depuis le paquet neuf, 21,09 s. Ce n'est PAS une exécution de la suite complète.
- `cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_dependency_test --test core_089_host_session_test --test core_089_content_test --test core_089_storage_test --test core_089_retired_runtime_test -- --include-ignored --test-threads=4` : 14/14, respectivement 0,01 s / 1,37 s / 5,14 s / 0,04 s / 0,10 s. Gate dépendances avec BRIDGET_CORE_PACKAGE_ROOT=/private/tmp/b089pkg.0s2Dq5/source et BRIDGET_CORE_PACKAGE_METADATA=/private/tmp/b089pkg.0s2Dq5/metadata.json. Vrai daemon, wrapper et CLI, faux fournisseur ACP déterministe sans compte ; send/attach/stop et accès au contenu passent sans aucune implémentation retirée.
- Après ajout de l'identification ps au watchdog metadata : copie du seul test dans le paquet, `diff -qr crates /private/tmp/b089pkg.0s2Dq5/source/crates` vide ; gate dépendances recompilé puis 2/2 en 0,01 s.
- Mutant exécuté : création du répertoire plugins dans le paquet → oracle FAILED « source interdite dans le paquet : plugins » (exit 101). Suppression du seul répertoire vide créé, contre-passe 2/2 verte. Aucun fichier source original ni processus de flotte modifié.
- `cargo test --offline --locked -p bridget-daemon --features test-support --lib -- disk_hygiene::tests:: --test-threads=4` : 8/8, 0,09 s ; même commande avec filtre `parseurs_structures` : 2/2, 0,00 s.
- `cargo clippy --offline --locked --workspace --all-targets --features test-support -- -D warnings` : vert, 7,12 s ; `cargo fmt --all --check` et `git diff --check` verts.
- `bash scripts/verify-089-contracts.sh --self-test --require-complete` : 17 fichiers égaux aux objets Git, six mutants refusés.

SC-08906 est prouvé pour construction/exécution du paquet minimal. Pas de déclaration de livraison globale : recettes comptes fournisseurs, SSH distant, suite complète et installateur final restent T020/T021/T026/T034/T035. Les scripts d'installation historiques encore dans le dépôt de travail ne sont PAS distribués dans ce paquet. Aucun remplacement du produit en service.


## 2026-09-05 — T012 : modules SQL, transactions inchangées

Store est réparti entre ledger_requests, service_events et project_compat ;
IdempotencyStore entre send_delivery, spawn_commands et agent_links. Aucun
trait de repository, aucune connexion ou table supplémentaire. Ouverture,
ordre des migrations et purge transversale restent dans le propriétaire
initial. Les tests historiques sont déplacés dans tests.rs, pas supprimés.
Les types publics et chemins des helpers transactionnels sont réexportés.
L'oracle d'absence de dépendance MCP couvre aussi les nouveaux fichiers.

Contrôle mécanique contre 1ba0eb9 : 136 corps de fonctions Store et81
Idempotency, tous identiques après neutralisation de la seule indentation
de début de ligne. Aucun corps de production ni SQL de migration réécrit.
Store.rs passe de6793 à439lignes, idempotency.rs de5093 à1168 ; il s'agit
de séparation de responsabilités, PAS d'une baisse équivalente du volume.

Le modèle de données/plan corrige une erreur de description : le ledger
était déjà visible au dispatch, avant ACK. Le découpage conserve la transaction
dispatch+ledger+suivi et celle ACK+Accepted+answered ; reply de service garde
replied+answered+événement ensemble. Aucun fait d'émission n'est promu en ACK.

Preuves sous les mêmes racines privées/env-i/umask077 que T011 :

- `cargo test --offline -p bridget-daemon --lib -- store::tests::
  idempotency::tests:: --test-threads=4` :82/82,1,38s (compilation3,92s).
  Le filtre substring inclut les14 tests receipt/blob, conservés dans le total.
  Réservations concurrentes, migrations historiques, même canon, insertion
  de suivi forcée en erreur et finalisation de saga annulée sont exécutées.
- `cargo test --offline -p bridget-daemon --features test-support
  --test coordination_events_test --test guichet_integration_test
  -- --test-threads=2 --skip depot_cli_reel_mesure_head_et_remote_au_lieu_de_copier_le_mandat
  --skip depot_cli_reel_et_reponse_guichet_cloturent_une_demande_liee_une_seule_fois` :
  coordination6/6,3,71s et guichet4/4,0,55s. Les deux exclusions CLI sont
  le défaut d'attribution déjà consigné T014/T018, pas des succès implicites.
  SIGKILL réel, génération de claim, reprise à mêmes bytes/event_id et
  distinction Gap/Unavailable conservés.
- `cargo test --offline -p bridget-daemon --features test-support
  --test idempotency_crash_test -- --test-threads=1 --include-ignored` :
  12/12,178,15s, dont la matrice50crashs/50prompts uniques (quatre frontières).
  Watchdog externe420s, interne360s ; délais de jalon/prompt5s inchangés.
  PREMIER RUN rouge : cycle14/50, reçu wrapper seen, phase indeterminate,
  aucun turn_start, événement interne98octets correspondant au refus
  « échéance de livraison dépassée ». Le faux fournisseur déclarait1s alors
  que la reconnexion prend déjà1s ; le TTL de mission de cette fixture passe
  à30s en c937928. Ce n'est PAS une extension du watchdog ni un changement
  produit. Les11 autres crash-tests passaient déjà ; le cas Seen indéterminé
  reste explicitement testé. Aucune compilation concurrente du binaire lors
  de la contre-passe verte ; comptes/API/fournisseurs réels non utilisés.
- `cargo test --offline -p bridget-daemon --test core_089_storage_test
  --test core_089_host_session_test --test core_089_content_test
  -- --test-threads=4` :10/10 (2/2,0,04s ;1/1,1,35s ;7/7,5,12s).
  Deux nouveaux triggers SQLite font échouer LA dernière écriture : après
  réouverture, aucun ACK/terminal/answered partiel, canon et ledger inchangés ;
  retrait de la faute puis retry produit une unique issue/événement.
  Mutants réellement exécutés, puis RETIRÉS : COMMIT intermédiaire avant
  événement → answered au lieu de open ; COMMIT avant mark_answered →
  issue terminale au lieu d'OutcomeUnknown. Les deux tests deviennent rouges
  aux assertions d'état, pas au build, puis repassent verts après restauration.
  Le premier essai du test guichet nommait le service « service » au lieu de
  l'identité publique historique « maicie » : fixture corrigée, aucune
  adaptation de la clôture de production pour contourner son autorité.
- Clippy workspace/all-targets, fmt-check, diff-check et vérificateur des
  17fixtures/six mutations verts. La suite workspace totale reste T034.

Les racines de l'essai N50 rouge sont conservées pour diagnostic, sans enfant
vivant ; aucun objet de la flotte n'a été visé. Dépendances/paquet T013 suivent.

## 2026-09-05 — T011 : moteur Docker/projet retiré, sessions hôte conservées

Suppression des lanceurs Docker, ingress, moteurs de projet, catalogue de
ressources et assets infra/project-runtime. Disposition des 88 tests retirés
dans test-map.md : périmètre projet uniquement, aucune suppression motivée
par leur couleur. Fichiers récupérables par Git ; aucune donnée utilisateur
ni processus de production supprimé. DTO et tables historiques restent
lisibles dans project_compat/store : pas de DROP ni conversion Docker→hôte.
Nouveau spawn.project refusé avant réservation ; reprise/relaunch refusés
aussi sur runtime_execution même sans project. Le retry terminal historique
conserve son issue et ses octets, sans nouveau processus. Anciennes commandes
et options runtime refusées avant initialisation ; BRIDGET_RUNTIME_* refusé
dès résolution du namespace (vide/non-UTF8 inclus, valeur jamais affichée).
Permissions, billing/pass_env, observations fournisseur, raw/source et ACL
des contenus restent en place.

La recette a révélé un réglage de posture accessible uniquement par l'UI
retirée. `bridget control posture discovery|complete` réutilise maintenant
ControlStateRead/ControlStateSet, génération et autorisation existantes.
Stdin ET stdout TTY obligatoires ; défaut discovery inchangé. Aucun bypass
pour faire passer le test : CLI réel dans un pseudo-terminal, pas de SQL
direct pour le changement de posture.

Preuves root, env-i/umask077 sous /private/tmp/b9t011.7nVSnj, HOME/state/tmp
privés, watchdog180s (recette hôte40s, CLI TTY10s) :

- `cargo test --offline --workspace --no-run` vert8,52s ;
  `cargo test --offline --workspace --features test-support --no-run`
  vert12,91s : compilation, PAS exécution de tous les anciens harnais.
- `cargo test --offline -p bridget-daemon --test core_089_host_session_test
  --test core_089_retired_runtime_test --test core_089_content_test
  --test channel_observation_test -- --test-threads=4` :14/14.
  Respectivement1/1 en1,35s,2/2 en0,09s,7/7 en5,13s,4/4 en0,35s.
  Daemon/CLI/wrapper réels, fournisseur ACP factice déterministe sans compte :
  refus projet sans réservation/processus, puis refus posture au même ID,
  réglage TTY, spawn accepté, SnapshotCaughtUp AVANT Send, journal contenant
  la réponse, StopOrder et groupe disparu. PATH sans Docker.
- `cargo test --offline -p bridget-daemon --lib lifecycle::tests::core_089
  -- --test-threads=3 --nocapture` :3/3,0,16s. Métadonnées historiques et
  canon terminal conservés. Auteur : lifecycle11/11 + registry34/34 +
  wrapper ciblé19/19, soit64/64 chevauchants. Huit anciens command_id de
  fixture invalides remplacés par UUIDv4 explicites, mêmes collisions/quota.
  Environnement3/3 et wrapper sous-processus9clés×3entrées validés.
- Contre-revue T011 APPROVE : contrôles de portée des contenus, Register et
  posture conservés, aucune réintroduction d'ACL par inférence. Les fonctions
  artifact_access_scope/artifact_scope_for_agent/handle_register_with_channel
  et resolve_spawn_agent_type_for_posture sont inchangées.
- `cargo clippy --offline --workspace --all-targets -- -D warnings` :
  vert4,61s, aucune dérogation. Deux dettes de T010 retirées avec le moteur,
  trois expressions équivalentes simplifiées en af483d3, boucle de test
  mono-élément aplatie. Trois tests de migration attendaient encore v9 :
  assertions alignées sur v10 DÉJÀ présente à HEAD, préservation maintenue.
- `cargo test --offline -p bridget-daemon --test artifact_service_test
  --test artifact_store_test --test execution_store_test --test execution_budget_test
  --test project_profile_surface_test -- --test-threads=4` :24/24,0,11s
  cumulées hors compilation1,90s. Une tentative antérieure par filtres lib
  sélectionnait0test : elle n'est PAS comptée comme preuve.
- `cargo fmt --all --check` et `git diff --check` verts ;
  `bash scripts/verify-089-contracts.sh --self-test --require-complete` :
  17 fixtures byte-identiques, six mutations refusées.

Aucun enfant de recette restant au contrôle. Les scripts SSH préparés pour
T024 restent un lot séparé. T012 doit découper le SQL sans changer les
transactions ; fournisseurs réels, SSH et suite totale restent à prouver.

## 2026-09-05 — T010 : sortie effective de l'interface, contenus préservés

Retrait de ui.rs, apps/bridget-desktop, assets web et renderers, projection
Maicie d'affichage et collecteur HTTP artifact_fetch. Reqwest et sa fermeture
de dépendances disparaissent de Cargo.lock. Le CLI refuse ui AVANT namespace,
socket, ancien endpoint ou repli sur un programme homonyme. Les suppressions
et les 25 tests de ui_relay_test sont classés nommément dans test-map.md.
Récupération possible par Git ; aucune modification du projet d'origine.

Lecture des contenus sur le protocole Unix existant : ArtifactRead v1 fermé,
pages16K, références exactes, identité/instance/projet attestés par le daemon.
Blob relié à la version ET à la portée, SHA-256 vérifié sur un même descripteur,
aucun chemin client. Manifestes stockés relus sans re-sérialisation.
CLI artifact read et MCP bridget_read_artifact partagent le client Unix extrait
de mcp.rs, sa résolution d'identité et son inscription auxiliaire. Catalogue
MCP : ajout lecture, aucune approbation humaine ajoutée. Les outils publics
de service restent des clients sans dépendance au métier Maicie.

Le canon HTML historique reste intact (runtime_policy inclus), mais plus aucun
rendu ni exécution n'est promis. Le pilote Codex ne préfixe plus une consigne
UI aux tours ; corps exact reçu sur les deux tentatives de saturation/retry.
La contre-revue a trouvé un manifeste HTML valide enrichi dépassant512K :
borne corrigée à plafond d'entrée+256octets pour les trois métadonnées
historiques, avec oracle d'entrée EXACTEMENT512K, stockage supérieur, lecture
de tous les bytes/digest. Aucune valeur historique réécrite. Verdict final
de lecture : APPROVE T010 après cet amendement, pas audit global achevé.

Validations ciblées, sans fournisseur ni daemon de flotte :

- Construction `cargo test --offline --workspace --no-run` : vert8,77s ;
  `cargo test --offline --workspace --features test-support --no-run` : vert16,42s.
  Compilation de tous les tests, PAS exécution de tous les anciens harnais.
- Auteur contenu :24/24 (15 historiques,7 daemon/CLI/MCP réels,2 dispatcher
  liaison/génération projet). Les deux derniers traversent Register/socketpair,
  pas un binaire fournisseur. Les coutures binaires portent sur la portée privée.
- Contre-passe root finale :57/57. Lib `mcp::tests:: cli::artifact_read_tests::
  communication::tests:: --test-threads=4` :44/44,1,02s ;
  core_089_content_test :7/7,5,21s ; channel_observation_test :4/4,0,23s ;
  core_089_removed_surface_test :2/2,0,02s. Aucun ignore de ces sélections.
- Autres contre-runs root : artifact_lifecycle3, policy2, publication3,
  service3, store4, execution_observability1, mission_boundary2,
  project_profile_surface1 :19/19. Résultats chevauchant ceux de l'auteur,
  ne pas additionner les passes en prétendant de nouveaux scénarios.
- Adaptateur Codex sans compte réel :7/7,0,04s, faux fournisseur documenté ;
  corps HTML intact, handler de lecture partagé, outil inconnu/refus fermé.
  Clippy transport/lib vert4,08s. Aucun modèle fournisseur réellement appelé.
- Fmt workspace et diff-check verts ; vérificateur17fixtures/six mutants vert.
  Clippy workspace/all-targets reste rouge sur CINQ lints préexistants :
  daemon (doc mal placée, helper Docker9arguments), artifact_service
  (if/else obscur), artifact_store (if imbriqué), execution_store (booléen).
  Les trois lints UI ont disparu avec le code, aucun allow ajouté. Les cinq
  restants restent à fermer avant T034, pas de dérogation au gate final.

Racine root de contre-test /private/tmp/b9t010.KTJro3, umask077 ; env -i
HOME=.../home, BRIDGET_HOME=.../state, BRIDGET_SOCKET=.../state/s,
TMPDIR=.../tmp, PATH=/usr/bin:/bin:/usr/sbin:/sbin ; exécutables de test
construits ci-dessus sous watchdog Perl alarm90s(lib)/60s(intégrations),
les tests créent leurs sous-namespaces privés. Aucun enfant de recette restant.
La mutation du plafond est discriminée par l'oracle, pas exécutée dans le
produit dans cette passe. Les dettes historiques client read_line/échéance
restent T029/T030 ; le découplage ne les déclare pas réparées. SSH et les
fournisseurs réels restent à prouver, la suite complète n'est pas dite verte.

## 2026-09-05 — T009 : Maicie réellement hors du graphe de construction

Retrait de Maicie du workspace, du manifeste daemon et de Cargo.lock, sans
dépendance de remplacement en tests. La migration d'identité ne lit ni ne
modifie plus son magasin/configuration ; migration ledger/flotte/profils,
sauvegardes et journal Bridget conservés. La carte du wrapper conserve les
faits identité/Git, les contrôles de texte et les bornes ; elle ne choisit
plus une mission à partir du greffe. `reprise` ne lance plus le binaire
Maicie. La projection JSON publique reste provisoirement pour le lecteur UI,
qui sortira en T010 ; aucun import de types privés n'est réintroduit.

Contrats ServiceHello/guichet/capacités/claims/événements/curseurs inchangés.
La couture busy→annuaire→envoi est maintenant un client de socket publique,
sans bibliothèque Maicie : corps exact reçu par le pair wrapper. Son serveur
de test est arrêté et joint même en unwind, lectures bornées à trois secondes.
Deux erreurs initiales de portage du HARNAIS ont été corrigées sur observations :
socket acceptée non bloquante sous Darwin et ListAgents envoyé sur le rôle
Client 012 qui ne le négocie pas. Aucun correctif produit pour les contourner.

Première passe ciblée : **32/32** (migration 2, frontière 2, cartes/prompt 14,
reprise 9, couture publique 1, contrats service 2 et coordination 2).
Contre-passe root : **30/30**, en trois sélections daemon/identité/frontière
plus trois fixtures transport ; résultats chevauchants, ne pas les additionner.
Environnement nettoyé : HOME=/private/tmp/b9v.4WFP7M/home,
TMPDIR=/private/tmp/b9v.4WFP7M/tmp, BRIDGET_HOME=/private/tmp/b9v.4WFP7M/ns,
BRIDGET_SOCKET=/private/tmp/b9v.4WFP7M/ns/s, umask077, watchdogs60/90s.
Filtres daemon : `wrapper::prompt_tests:: reprise::tests:: daemon::presence_tests::tour_non_abouti_redevient_mandatable_et_le_mandat_parvient --test-threads=4 --skip reprise::tests::la_trace_de_reprise_n_est_pas_lue_quand_la_base_est_ailleurs` : **23/23, 0,49s**.
Exécutables identity_migration_test **2/2, 0,14s** et mission_boundary_test
**2/2**. Fixtures service_negotiation_v1/coordination_events_v1/coordination_stream_v2
**3/3**. Le test de reprise au chemin fixe est exclu de cette contre-passe,
pas promu en preuve d'isolation. Vérificateur des 17 fixtures et six mutants vert.

`cargo test --offline --workspace --no-run` : vert **17,99s**.
Copie indépendante /private/tmp/b9pkg.HAZ5po SANS dossier plugins :
`cargo metadata --offline --format-version 1 --filter-platform aarch64-apple-darwin`
ne contient aucun package Maicie ; `cargo test --offline --workspace --no-run`
y construit les tests en **19,17s**. Le filtre de plateforme évite une
dépendance Windows absente du cache offline ; ce n'est pas une validation Windows.
Le paquet final sans UI/runtime reste à prouver en T013.

Le no-run avec test-support a exposé E0559 dans mcp_injection_smoke_test :
ancien Registered.name. Correctif test séparé f6017a1 : agent_id repris du
Register réel, même destinataire pour Deliver. Contre-run
`cargo test --offline --workspace --features test-support --no-run` :
**vert, 7,28s**. AUCUN smoke fournisseur de ce fichier n'a été exécuté :
ancien namespace et environnement partagé restent à porter en T020/T021.

Formatage et diff-check verts. Clippy reste rouge sur huit lints préexistants
(daemon 2, artifact_service 1, artifact_store 1, execution_store 1, UI 3),
aucun allow ajouté. Le crash historique de supervision est conservé mais non
exécuté avant son portage d'isolation. Le défaut CLI/guichet DeclaredSenderMismatch
et le dry-run de migration mutateur restent explicitement T014/T018 et T031.

Revue indépendante : **APPROVE T009**, lecture du diff et du nettoyage du
harnais ; résultats de tests contre-lus, pas un troisième run. Dispositions
test par test consignées dans test-map.md. Le code Maicie reste consultable
dans le clone/historique mais n'est plus requis pour construire Bridget.

## 2026-09-05 — Référence avant coupe : clôture T005, pas de gate fonctionnel global

La référence mesurée et reproductible est maintenant disponible : suites
core/transport auditées, passe large daemon avec liste brute des rouges,
71 unités wrapper, CLI/MCP/ledger, guichet, événements cursés, flotte et
matrice de cinquante crashs. T005 clôt la COLLECTE de référence, pas une
promesse de suite entièrement verte. Les exclusions de sécurité nommées
ci-dessous restent des gates à porter dans T014–T021/T029–T034 ; les
fournisseurs réels et SSH ne sont pas remplacés par les adaptateurs factices.
Aucun de leurs résultats n'est inventé. Les scénarios contextuels Maicie et
UI/Docker seront retirés selon T003 ; leurs garanties transport utiles doivent
rester exercées par les coutures publiques, pas par une copie de leur métier.

Contre-run final des tests d'identité/CLI/attach/fleet modifiés : **36/36**,
un helper ignoré mais lancé par son parent, **0,42 s** ; même commande privée
que les 21 tests, avec filtre supplémentaire `fleet::tests::`, watchdog90s.
Les quatre fichiers de production n'ont changé que dans leurs modules de
tests. `cargo fmt --all --check` et `git diff --check` verts. La coupe T009
peut désormais commencer ; les défauts CLI/guichet et lints listés restent
des travaux obligatoires, aucune dérogation au gate final.

## 2026-09-05 — Fixtures d'identité : garanties restaurées, pas de modification produit

Portage exclusivement dans les modules de tests de cli.rs, attach.rs,
mcp_identity.rs et fleet.rs. Les UUID sont explicites/stables ; --agent-id
remplace --name dans les fixtures du parseur. L'absence de choix de survie,
le rejet des choix contradictoires, les bytes mémorisés du retry et les
refus d'options de dépôt interdites restent testés. Les arguments négatifs
attach utilisent maintenant une identité VALIDE : ils atteignent réellement
la mauvaise fenêtre demandée, plutôt qu'un rejet d'identité préalable.

Attach compare une sélection indépendante d'UUID routables, pas des noms
affichés. Le DTO sans persistent reste inconnu avec agent_id requis ; un DTO
sans agent_id est explicitement refusé, aucune fausse compatibilité nom-seul.
MCP identity : naissance, filiation, trois ancêtres, legacy, instance et
PID recyclé conservés. Le test de relecture du fichier d'identité n'est PAS
une preuve de rename métier ; cette preuve distincte reste en T015.

Tests CLI/attach/MCP identity ciblés sous le même env privé que la baseline
lib : **21/21 verts, 0,03 s** (compilation 3,95 s). Filtres :
`explique_les_refus_non_acp_et_nom_inconnu json_publie_la_persistance_meme_indeterminee mcp_identity::tests:: arguments_attach_ spawn_neuf_refuse_de_partir_sans_choix_de_survie rejeu_d_un_ordre_memorise_n_exige_pas_de_redeclarer_la_survie spawn_cli_rejoue_l_enveloppe_memorisee_octet_pour_octet stop_cli_genere_ou_reutilise_un_command_id depot_guichet_ depot_delegate_versionne_atomiquement_sa_cible_de_revue --test-threads=4`.

Fleet : **15/15 verts, 0,43 s**, un helper enfant ignoré dans la sélection
ordinaire MAIS réellement appelé par le test parent aux trois frontières.
Le harnais a un enfant/groupe propre, env_clear, garde de secours et attente
bornée, SIGKILL après barrière. Quota, réservations concurrentes, canon,
générations, clôtures et reprise sont inchangés dans le code produit.

Baseline wrapper supplémentaire auditée : **71/71 verts, 1,11 s**, sous
racine privée /private/tmp/bw-9by53dt9 (HOME=h, namespace=n, TMPDIR=t),
watchdog 90 s. Aucun fournisseur réel/tmux/daemon ; relais, jonction live,
rotation, receipts et namespace MCP exécutés. Quinze exclusions explicites :
onze scénarios contextuels Maicie, refus Zed au chemin /tmp fixe, shutdown
avec thread volontairement parqué, faux Claude sans garde de panique et
enfant du test de livraison interactive à exécuter séparément. Aucun succès
n'est attribué à ces exclusions. Clippy reste rouge hors de ces hunks ;
les lints répertoriés ne sont pas supprimés pour présenter une gate verte.

## 2026-09-05 — T005 : idempotence, 50 crashs réels exécutés

idempotency_crash_test.rs porté sans code produit : namespace indépendant,
UUID v4 actuels, HOME fournisseur privé, env_clear et états 0700/0600. Les
deux wrappers historiquement lancés en threads deviennent de vrais enfants
isolés exécutant le même chemin wrapper ACP. L'adaptateur fournisseur est
une fixture qui compte les prompts, pas un compte réel.

Avant signal : enfant direct (PPID), exécutable Bridget du banc, groupe créé
par le harnais et PID vivant vérifiés ; aucun PID arbitraire. Les Drop ne
paniquent pas, l'attente de fin est bornée ; watchdog global 360 s et suivi
des enfants. SIGKILL aux jalons test-support, pas de SIGTERM présenté comme
crash. Les comptes/réponses/canons attendus n'ont pas été relâchés.

Commandes :
- `/Users/moi/.cargo/bin/cargo test --offline -p bridget-daemon --features test-support --test idempotency_crash_test -- --test-threads=1 --nocapture`
  : **11/11 réussis, 7,00 s**. CLI/MCP réels, quatre issues d'une réponse
  liée, divergence sans mutation, ACK/answered atomiques et vrai wrapper.
- `/Users/moi/.cargo/bin/cargo test --offline -p bridget-daemon --features test-support --test idempotency_crash_test -- --ignored --exact matrice_crash_sc001_redelivre_cinquante_prompts_uniques --test-threads=1 --nocapture`
  : **50 cycles / quatre barrières, exactement 50 prompts, 172,96 s**.
  Replays intermédiaires puis terminaux stables ; watchdog non déclenché.

Premier essai de matrice : arrêt au cycle 0 après réarmement coopératif,
car SIGTERM envoyait Disconnect et fermait correctement le wrapper externe.
Le journal attestait EOF ACP. Le réarmement intermédiaire est désormais
lui aussi un crash réel, cohérent avec le scénario ; aucune correction du
daemon pour empêcher son arrêt propre. Les anciens fichiers privés des
essais rouges restent des traces, aucun processus /tmp/bid- résiduel.

Format ciblé et diff-check verts ; Clippy reste bloqué sur les neuf lints
historiques daemon déjà consignés. Ce portage exécute la baseline de crash ;
il ne déclare ni les gates SSH/fournisseurs ni toute l'extraction livrés.

Revue indépendante du seul diff du banc : APPROVE (contre-lecture,
pas un second run N=50), isolement et gardes PPID/binaire/PGID vérifiés,
oracles de prompts/canon/issue conservés.

L'oracle human_inbox des permissions est également adapté, sans code produit :
payload valide en 0644 refusé, puis mêmes bytes en 0600 acceptés ; ensuite
seulement la commande relative est testée. Cela évite un faux positif dû au
payload déjà invalide ou à un libellé d'erreur. Répertoire UUID indépendant.
Sous le même env privé que la passe lib : filtre exact
`human_inbox::tests::configuration_du_canal_exige_0600_et_chemin_absolu`
**1/1 vert, 0,00 s** (build 4,66 s). La première invocation avec filtre court
et --exact sélectionnait zéro test et ne compte pas comme validation.

## 2026-09-05 — T005 : reprise du flux public après crash

coordination_events_test.rs porté exclusivement côté harnais : env_clear,
namespace 0700 court/canonique, identités UUID v4, daemon possédé par une
garde dès spawn, contrôle du PID/exécutable avant SIGKILL, wait borné même
sur erreur. La sonde SQLite ouvre en lecture seule ; elle ne fabrique plus
un fichier avant les migrations du daemon. Les FIFO des jalons sont dans
une racine sœur privée : le namespace de production refuse les fichiers
spéciaux et cette garde n'a PAS été relâchée pour le test.

`cargo test --offline -p bridget-daemon --features test-support --test coordination_events_test -- --test-threads=2`,
sous env nettoyé et racine /private/tmp/b89ce-ukgi9u56 : **6/6 réussis**, zéro
ignoré, 3,67 s (5,846 s total), watchdog global 180 s non atteint. Vrais
SIGKILL avant/après persistance et relectures conservant bytes/event_id/curseur ;
Gap et Unavailable restent deux observations distinctes. Les oracles de refus
entrant et de non-inférence depuis le texte sont inchangés. Le premier run
5/6 s'arrêtait avant le jalon sur la FIFO placée dans l'état ; le déplacement
du seul harnais ferme ce refus, pas une modification de la sécurité produit.

`cargo clippy --offline -p bridget-daemon --features test-support --test coordination_events_test -- -D warnings`
est rouge sur les trois lints Maicie déjà consignés (control Default,
guichet large_enum_variant, store too_many_arguments). Format ciblé et
diff-check verts. Aucun gate fournisseur ni SSH n'est confondu avec ces tests.

## 2026-09-05 — T005 : passe lib élargie auditée, pas un gate vert

Après audit indépendant des lancements, 646 scénarios lib sélectionnés sous
env_clear/private HOME et TMPDIR, umask 077, watchdog 600 s, quatre threads :
**585 réussis, 55 échecs, 6 ignorés, 228 filtrés ; 8,39 s.** Commande exacte
et liste brute des échecs : baseline-daemon-lib-2026-09-05.txt.
Les totaux des passes ciblées ne s'ajoutent PAS à ce total : elles se recouvrent.

Exclusions de sécurité, pas masquage d'un rouge : wrapper et presence_tests
(anciens bootstrap/fournisseurs et cargo Maicie imbriqué), test managed_process
utilisant le PGID du harnais, test reaper scannant l'hôte, deux sondes sur socket
/tmp fixe. Elles restent à porter ; aucun --ignored global.

La majorité des rouges concerne des fixtures antérieures à UUID/--agent-id
(CLI, fleet, lifecycle, MCP identity, helper daemon). Fleet crash échoue
avant son oracle sur EOF de son enfant. Six rouges UI viennent de SUN_LEN
sous TMPDIR long ; les scénarios runtime/projet ont aussi des gardes de
politique/activation en échec. Ils seront retirés pour leur périmètre T010/T011,
jamais pour leur couleur. Un rouge human_inbox est une adaptation T007 encore
nécessaire : l'ancien test attend « 0600 », mais la nouvelle garde rejette
bien 0644 avec le libellé « état privé de type/propriétaire valide requis ».
Il ne s'agit pas d'un refus de chemin extérieur au namespace. Ne pas
présenter ces 55 rouges comme tous antérieurs à T007 sans distinction.

## 2026-09-05 — T005 : reprise des crash-tests autorisés et baseline guichet

Passe complémentaire auditée des unités daemon : compilation `cargo test
--offline -p bridget-daemon --lib --no-run` (3,06 s), puis sous env -i,
umask 077, HOME/TMPDIR=/private/tmp/bg089-lib-safe.ZF3jFl et
BRIDGET_HOME=.../state, BRIDGET_SOCKET=.../state/bridget.sock :
`/usr/bin/perl -e 'alarm 180; exec @ARGV' target/debug/deps/bridget_daemon-b7902f0f20bc17cc registry::tests:: lifecycle::tests:: attach::tests:: runtime::tests:: disk_hygiene::tests:: reprise::tests:: --skip project_runtime:: --skip la_trace_de_reprise_n_est_pas_lue_quand_la_base_est_ailleurs --test-threads=4`.
Résultat : **121 réussis, 5 échecs**, 0,50 s, 748 filtrés. Les PTY réels,
POLLIN|POLLHUP, raw/termios, journal fragmenté, reprise last_seq+1, gaps,
limites de mémoire et saturation attach sont verts. Pas de fournisseur,
Docker ou tmux réel ; le test Git initialise seulement un dépôt privé.
Échecs historiques d'identité : attach::explique_les_refus_non_acp_et_nom_inconnu
compare désormais des UUID à ses anciens libellés ; les quatre tests
lifecycle le_refus_de_cwd_nomme_la_machine_cherchee_et_la_machine_demandeuse,
matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel,
session_claude_native_est_preparable_comme_equipier_gere et
spec_066_runtime_docker_n_exige_jamais_la_commande_fournisseur_sur_l_hote
construisent un agent_id de spawn invalide. Leurs attendus ne sont pas
affaiblis ; adaptation des fixtures dans T015/T019 et retrait justifié du
seul scénario Docker dans T011. Le filtre reprise exclu utilise une socket
fixe /tmp : il n'est pas compté parmi les réussites.

Autorisation explicite de l'utilisateur (« oups pardon oui kill ») : SIGKILL
uniquement sur les enfants créés par les harnais isolés. Aucun signal à la
flotte existante, à Firefox ou à un processus tiers. Le banc guichet vérifie
le PID et sa commande avant signal, puis récolte l'enfant ; sa garde protège
aussi le chemin de panique. Aucun déploiement ni changement de configuration
globale.

Portage de guichet_integration_test.rs, sans correction du produit :
env_clear, HOME/TMPDIR/BRIDGET_HOME/BRIDGET_SOCKET privés, racine canonique
0700 courte et UUID aléatoire complet ; politique créée en 0600 ; identités
UUID v4 et BRIDGET_AGENT_ID conformes au protocole actuel. Les anciens noms
de test ne passaient plus Register. Le nom du service « maicie » reste la
cible publique, ce n'est pas un import de son implémentation.

Commande : `cargo test --offline -p bridget-daemon --test guichet_integration_test --no-run`,
puis sous umask 077, env -i et HOME/TMPDIR=/private/tmp/bg089-guichet-run.Zd0Ouj :
`/usr/bin/perl -e 'alarm 180; exec @ARGV' target/debug/deps/guichet_integration_test-9fb41b9c3d04aab5 --test-threads=4`.
Premier résultat : **4/6 réussis, 2 échecs**, 0,84 s (compilation 2,11 s).
Le SIGKILL après réception du claim durable, la génération neuve, le refus
claim_stale et l'égalité byte-à-byte des dépôts A/B sont verts ; les neuf
refus Git exacts, le dépôt autorisé et la garde de panique également.

Les deux échecs sont une couture historique du produit, pas masquée par le
portage : `cmd_guichet` ouvre une connexion via `cli_register` (UUID v4 neuf),
mais ServiceRequest n'accepte l'expéditeur délégué que si le nom enregistré
commence par `cli-send-`. Sortie réelle dans les deux cas :
`REJET: DeclaredSenderMismatch`. Le rapport Git et la clôture CLI ne sont
donc PAS prouvés verts. À corriger dans T014/T018 avec une autorité de
connexion explicite, sans réintroduire des noms invalides ni élargir les
droits à toute connexion UUID.

Contre-run après relecture de la garde : mêmes 4/6 et mêmes deux refus,
0,77 s. `cargo fmt --all --check` et `git diff --check` passent. Aucun test
désactivé ni attendu remplacé par le refus observé ; ce commit de baseline
ne prétend pas réparer le guichet.

Validation indépendante supplémentaire, sous env_clear/umask 077 et
watchdog 600 s : core --lib **39/39**, 1,11 s ; transport --lib **248/248**,
5,54 s, un micro-banc historique ignoré et un test Node/UI explicitement
exclu (`TEMOIN_vocabulaire_vue_et_ecriture_ne_divergent_pas`, à découpler
en T010). Premier passage transport : 11 PolicyPathNotCanonical à cause de
l'alias macOS /tmp ; seul TMPDIR canonicalisé en /private/tmp les referme,
sans modification de code. Aucun fournisseur réel ni processus résiduel
de ces deux validations. Ces résultats ne valent pas gate global T034.

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
