# Tâches 090 — Implemented

## Correctif reprise par nom du 07/09 — installé

- [x] T021 Mesurer le Codex installé : scan par défaut lent, searchTerm seul 46,993 s, catalogue sans réparation via useStateDbOnly 364 fils/1,811 s. Conserver les preuves externes et cadrer le delta minimal.
- [x] T022 Oracle rouge au collecteur : un test exécuté, échec attendu None vs true avant correction ; deux pages de même nom doivent toutes être conservées avec le drapeau. Le silence du scan est attesté par la sonde réelle, pas simulé par un délai de test. Le vrai parcours 090 est rejoué en T023.
- [x] T023 Ajouter le drapeau API seul, puis tests ciblés, recette native, revue du delta ; fmt/clippy et consolidation finale, release installé sans redémarrage de daemon/agents. Trois tests catalogue PASS ; recettes natives nom/menu/ambiguïté PASS, nom aussi sur release. Workspace : 1252 PASS / 0 FAIL / 47 ignorés. Release installé, empreinte e092f3044f48e87fdcc84b1b687ffada6815a80fc1e82f004755a72a3845a1d3. Preuves dans verification-reprise-catalogue.md.

## Phase 1 — Cadrage et sonde

- [x] T001 Vérifier le contrat réel de fil partagé et d'approbation dans specs/090-codex-interactif/probe_shared_session.py et research.md ; même threadId, aucun prompt caché, décision du second client respectée.
- [x] T002 Auditer la réutilisation, les bornes et les permissions dans specs/090-codex-interactif/reuse-audit.md ; gate PASS, aucune duplication non arbitrée.

## Phase 2 — Frontière native commune

- [x] T003 Déplacer connect_nonblocking de crates/bridget-daemon/src/communication/client.rs vers crates/bridget-transport/src/jsonl.rs sans changement de contrat ; rejouer tests de connexion/délais existants.
- [x] T004 Ajouter crates/bridget-transport/src/codex_socket.rs et la dépendance framing minimale ; tester bytes atypiques, limite, EOF, handshake et écriture pendant lecture inactive avant branchement du pilote.

## Phase 3 — US1, session interactive joignable

- [x] T005 [US1] Étendre crates/bridget-transport/src/codex_app_server.rs pour posséder un app-server Unix, nommer le fil et réutiliser reader/worker ; stdio géré inchangé, identité = PID réel du serveur.
- [x] T006 [US1] Ajouter crates/bridget-daemon/src/codex_interactive.rs et raccorder wrapper.rs : options natives explicites, terminal requis, présence non-tmux, TUI sur le même fil après journal/identité prêts ; tests parsing et refus sans TTY.
- [x] T007 [US1] Prouver demande/réponse liée et retry avec daemon/CLI/MCP réels dans crates/bridget-daemon/tests/codex_interactive_090_test.rs ; une remise, ledger et identité cohérents, namespace isolé.

## Phase 4 — US2, tour humain et permissions

- [x] T008 [US2] Observer les tours TUI dans crates/bridget-transport/src/codex_app_server.rs : journal complet, état attesté, aucune collision avec worker interagent ; tests tour externe puis message Bridget, raw exact et navigation explicitement traitée.
- [x] T009 [US2] Tester puis garantir dans le reader interactif que permission/élicitation n'obtiennent aucune réponse automatique ; vraie requête native décidée par la TUI dans codex_interactive_090_test.rs.

## Phase 5 — US3, fin et pannes

- [x] T010 [US3] Borner arrêt TUI/serveur, EOF et fermeture terminal dans codex_interactive.rs et codex_app_server.rs ; zéro enfant de test survivant et socket possédée nettoyée, sous 10 s.
- [x] T011 [US3] Rejouer coupure/reconnexion du daemon depuis codex_interactive_090_test.rs ; identité et threadId inchangés, même tracker, aucun thread/start supplémentaire ; parcours SSH configurable conservé.

## Phase 6 — Recette, convergence et audit

- [x] T012 Exécuter la vraie TUI sous PTY et un échange fournisseur sous abonnement existant si disponible dans codex_interactive_090_test.rs ; aucun fallback API payant. Consigner tout blocage exact dans implementation.md.
- [x] T013 Aligner README.md, README.en.md, skills/bridget/SKILL.md et docs/communication-installation.md sur le comportement vérifié, distinguer interactif/spawn persistant/attach.
- [x] T014 Exécuter cargo fmt --all --check, cargo test --workspace et cargo clippy --workspace --all-targets -- -D warnings, puis convergence/audit dans specs/090-codex-interactif/implementation.md ; aucune case fermée sans preuve.

## Dépendances et exécution

## Amendement demandé le 2026-09-06

- [x] T015 Alias `--yolo`, `resume <UUID>` et `--name` : tests positifs et refus ; aucun bypass implicite ni nom confondu avec l'identité.
- [x] T016 Réutiliser reprise native et renommage partagé ; vraie TUI/daemon, historique et titre préservés, permissions explicites attestées, UUID/nom cohérents.
- [x] T017 Rejouer les tests pertinents des deux crates, les recettes natives de non-régression, fmt/clippy ; documenter la correction vérifiée. L'adoption du binaire suit le commit, avec reçu et sauvegarde externes.

## Ordre historique

T001→T002→T003→T004→T005→T006→T007 ; T008/T009 prolongent le même pilote
séquentiellement ; T010/T011 après assemblage ; T012→T013→T014 en clôture.
Pas de couloir d'écriture parallèle sur wrapper/pilote. Les lectures/revues peuvent
être indépendantes. Les tâches de test rédigent d'abord l'oracle puis le fix utile.
US1 seule n'est PAS la livraison : toutes les phases sont requises.

## Correctif sous-agents internes — arbitrage utilisateur du 2026-09-06

- [x] T018 Dans crates/bridget-transport/src/codex_app_server.rs, test discriminant au lecteur réel : création puis reprise/statut d'un autre fil pendant un tour parent, traitement du prochain événement parent et reader toujours vivant ; zéro corrélation/journal/permission étrangère ; EOF réel terminal. Supprimer la fermeture déduite de ces seules notifications, sans ajouter de gestion d'enfants à Bridget.
- [x] T019 Recette isolée utilisant le vrai app-server et le raccord interactif : chargement d'un second fil sans fermeture du parent ; conservation du fil initial et de son adressage. Aucun lancement de sous-agent dans la session de production avant adoption du correctif.
- [x] T020 Aligner README FR/EN, skill et contrat ; tests ciblés, non-régression du raccord natif, fmt/clippy ; rebuild et adoption explicitement autorisés par le mandat utilisateur courant, sans redémarrage de la flotte. Distinguer preuve automatisée et recette humaine après reprise. Preuve : implementation.md, dont la recette permission optionnelle rouge également sur l'ancien binaire (avant injection EOF), distincte de ces deux recettes natives vertes.
