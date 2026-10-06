# SPEC138 — Façades de communication

Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet.

## Contrat implémenté

- CLI `who` et `agents` utilisent l'annuaire local par défaut. `--global`
  demande le diagnostic global. La sortie JSON d'`agents` reste une liste
  aplatie avec `communication_project` et `project_relation`.
- MCP `bridget_who.scope` accepte seulement `same_project` ou `global`.
- Un auxiliaire hérite du projet du propriétaire vivant. `--project-root`
  n'écrase jamais cette identité. Le client autonome négocié peut annoncer
  sa propre racine Git explicite, sans créer un faux agent.
- Sans racine explicite, le client de fond reste inconnu. Le cwd du processus
  ou le dépôt dotfiles ne devient pas silencieusement le projet d'un run.
- `--cross-project-reason` et le champ MCP `cross_project_reason` portent le
  choix volontaire. La validation commune refuse null, vide, contrôles et
  plus de 512 octets UTF-8 après normalisation. Le texte du message ne change pas.
- Le motif des fils est porté par `ThreadRequest`, hors `ThreadAction`.
  Handoff et journal transmettent le même champ au transport existant.
- Un daemon ancien qui ne négocie pas `CommunicationProjectsV1` ne reçoit
  aucun message avec ce nouveau motif. Aucun repli global n'est effectué.
- Le résultat caller contient `project_warnings`, distinct du corps livré.
  La CLI produit `AVERTISSEMENT PROJET: ` suivi d'un JSON `ProjectWarning`.
  Agent Loop peut filtrer cette structure sans afficher le stderr libre.

## Réutilisation et périmètre

La recherche préalable des annuaires, des négociations, des résolveurs et des
envois a retenu `DaemonConnection`, `authenticate_auxiliary`,
`registered_connection`, `send_idempotent_to_daemon_at`, `execute_who`,
`execute_send`, `HandoffTransport` et `JournalRequest`. Les nouveaux petits
helpers ne créent aucun moteur, service, dépendance ou registre parallèle.

Le harnais réel réutilise `start_real_daemon`, `owner_connection` et
`caller_identity` de la suite CLI existante. Leur visibilité interne sous
`cfg(test)` évite un second daemon de tests. Le scénario local partagé entre
la recette ordinaire et la variante opt-in évite de copier ce harnais.

L'annonce propriétaire wrapper/T3 est détaillée dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/transport-facades.md.

## RED exécuté et limites

La première vague RED commune a exécuté trois tests de ce périmètre avant
leur implémentation : options CLI global/racine, motif MCP transmis, et
handoff strict avec corps inchangé. Résultat commun : 0 PASS, 5 FAIL,
1021 filtrés, exit 101. Deux autres tests concernaient le noyau.

La capture brute de cette première vague n'a pas été enregistrée dans un log.
Le reçu outil et les assertions sont tracés sans inventer un log dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/core.md.

Les autres tests ont été écrits avant leur code mais leur RED séparé était
bloqué par l'intégration des nouveaux types. Le principal a autorisé la
suite sans supprimer ces tests. Aucun RED supplémentaire exécuté n'est revendiqué.

## GREEN déjà exécuté

Isolation : umask 077, BRIDGET_HOME et TMPDIR dans un dossier privé court,
CARGO_INCREMENTAL=0, target isolé
/Volumes/8TB2/01-workflow/bridget-138-cargo.HjVvbG.

Commande : `/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec138_ -- --nocapture`.
Première vague intégrée : 20 PASS, 1 FAIL. Tous les tests façades et les cinq
tests wrapper/T3 passent. Le seul échec est une fixture auxiliaire du noyau
avec un issuer_scope plus court que sa borne contractuelle. Le noyau a depuis
corrigé cette fixture ; cette vague ne revendique pas un PASS global.
Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-green-138.log.

Suites historiques exécutées après la correction de notre fixture ancien
serveur, avant l'ajout du scénario réel :

- `cargo test -p bridget-daemon --lib mcp::tests` : 51 PASS.
  Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-mcp.log.
- `cargo test -p bridget-daemon --lib cli::` : 80 PASS.
  Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-cli.log.
- `cargo test -p bridget-daemon --lib communication::client::` : 14 PASS.
  Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-client.log.

Formatage et `git diff --check` limités aux huit fichiers possédés : PASS.
La recette réelle ajoutée n'est pas encore exécutée au moment de cette fiche.
Une première compilation T019 a détecté et fait corriger quatre erreurs de
types dans notre nouvelle fixture réelle, sans résultat RED comportemental.

## Recette interdépôts opt-in

Test ignoré volontairement :
`mcp::tests::spec138_real_agent_loop_cli_daemon_scoped_replay`.
Il exige les chemins absolus `BRIDGET_SPEC138_AGENT_LOOP_SCRIPT` et
`BRIDGET_SPEC138_BIN`, sans défaut vers le script ou le binaire de production.

Il importe Agent Loop sans lancer de heartbeat ni de LaunchAgent. Il utilise
un daemon, deux projets Git, une socket, BRIDGET_HOME et TMPDIR privés. Le cwd
est un autre projet que la racine déclarée. Il vérifie annuaire local/global,
refus interprojets sans motif, remise exacte avec motif, warning structuré,
rejeu du même reçu sans second dépôt et absence de faux agent de fond.

Le chemin binaire transmis au script est `AGENT_LOOP_BRIDGET_BIN`, pas
`BRIDGET_BIN`. Une exécution réelle reste nécessaire avant de revendiquer PASS.

Aucun commit, installation, message de production, déploiement ou appel fournisseur.

## État final des façades

Les huit fichiers de production sont gelés. Le contrat/API ci-dessus reste
inchangé. La lecture de `ControlExecutionResult` et `ExecutionControlRefusal`
dans ce périmètre n'a trouvé aucune façade SteerCurrent existante. Aucune
commande de contrôle nouvelle n'est créée. Le noyau et le store couvrent T019.

L'index T3 a été corrigé sur demande du principal : une HashMap locale est
construite une fois pendant la lecture des projets. Les identifiants dupliqués
restent inconnus. Le coût ajouté est O(P+T), et non O(P×T). Les tests de
référence manquante et ambiguë restent présents et passent.

La recette réelle a d'abord montré deux erreurs de fixture, pas deux défauts
de production : le refus MCP expose le statut catégorisé
`cross_project_reason_required`, et l'envoi idempotent remet
`DeliverIdempotent`, non `Deliver`. La seconde réponse a été relue en données
runtime exactes avant la correction de l'assertion. Les logs des échecs sont
conservés pour ne pas inventer une réussite initiale.

Validation finale exécutée :

- `/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec138_` :
  25 PASS, 0 FAIL, 1 ignoré volontairement.
  Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-final-all-138.log.
- `/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib mcp::tests` :
  52 PASS, 0 FAIL, 1 ignoré volontairement.
  Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-final-mcp.log.
- `/Users/moi/.cargo/bin/cargo build -p bridget-daemon --bin bridget` : PASS.
  Binaire isolé : /Volumes/8TB2/01-workflow/bridget-138-cargo.HjVvbG/debug/bridget.
  Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-debug-build.log.
- `/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec138_real_agent_loop_cli_daemon_scoped_replay -- --ignored --nocapture` :
  1 PASS, 0 FAIL. La recette ignorée ordinaire a donc bien été exécutée.
  Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-loop-real.log.

Variables explicites de la recette :

```text
BRIDGET_SPEC138_AGENT_LOOP_SCRIPT=/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py
BRIDGET_SPEC138_BIN=/Volumes/8TB2/01-workflow/bridget-138-cargo.HjVvbG/debug/bridget
CARGO_TARGET_DIR=/Volumes/8TB2/01-workflow/bridget-138-cargo.HjVvbG
CARGO_INCREMENTAL=0
```

Le test fixe en plus BRIDGET_HOME, BRIDGET_SOCKET et TMPDIR au namespace du
harnais privé et désactive les caches Python. Il retire l'identité T3 héritée
via le véritable `bridget_background_environment` du script.

La recette passe par `cmd_attach_agent` : sans motif, refus interprojets et
tous les fichiers du run restent byte-identiques ; avec motif, la session
enregistre le mandat exact agent/rôle. `bridget_scope_context` transforme ce
mandat en contexte de l'envoi réel. Le contrôle SQL trouve trois remises au
total : MCP, CLI et Agent Loop. Les refus n'ajoutent rien au ledger. Le rejeu
du reçu Agent Loop ne dépose pas une quatrième remise. Le nombre d'agents
reste deux : aucun équipier éphémère n'a été fabriqué.

Les comptes partiels précédents sont conservés comme journal, pas comme
validation finale. Validation globale workspace/clippy/release et audits de
convergence restent sous la responsabilité du principal. Aucun PASS global,
commit, fusion ou déploiement n'est revendiqué par ce rapport.

## Migration des diagnostics globaux des harnais historiques

Le workspace complet a révélé un RED réel : six scénarios097 attendaient la
présence enregistrée jusqu'à leur timeout. Le daemon et le faux fournisseur
étaient prêts. Le harnais demandait `agents --json` depuis un client autonome
sans projet : la nouvelle portée locale rendait correctement une liste vide.
La fixture097 recrée BRIDGET_HOME et BRIDGET_SOCKET dans un environnement
neuf ; elle n'hérite pas du namespace du lanceur. Elle utilise le vrai daemon,
donc aucune annonce ProjectFact n'est lue par un faux interlocuteur.

Preuve RED conservée :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/workspace-final-v2.log.
La dernière traceback peut montrer `--global`, car Python relit la ligne
source après la modification du fichier sur disque. Ce n'est pas une deuxième
exécution chargée avec le correctif. Aucun RED supplémentaire n'est inventé.

Les observations de présence et d'absence globales ont reçu leur portée
explicite. Aucune découverte locale ni test d'argument SURPLUS n'a été modifié.
Les assertions d'absence ne peuvent donc pas passer artificiellement grâce
à une vue locale inconnue vide. Fichiers de tests concernés :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/fixtures/claude_interactive_097.py : sept appels d'observation.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/fixtures/codex_interactive_090.py : neuf appels d'observation, y compris les assertions d'absence.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/core_089_federation_test.rs : trois sondes who et un annuaire agents. Les recettes SSH restent opt-in.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/core_089_native_test.rs : deux lectures who qui vérifient transport, modèle, effort et limites.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/core_089_status_test.rs : diagnostics globaux explicites. Pour who/agents, l'inventaire préalable est valide, cap138 est négociée et DirectoryScoped(Global) est réellement reçu. Registered étranger suivi d'une liste plausible provoque le refus sans stdout. Le chemin status legacy reste inchangé.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/core_089_isolation_test.rs : portée diagnostic explicite et vraie preuve propriétaire pour MCP. Le helper support/idempotent.rs reçoit le credential réel ; le propriétaire reste connecté pendant MCP. L'UUID attendu est visible et le nombre d'identités ne change pas pendant la lecture. Aucun assouplissement d'autorité daemon.

La première vague ciblée conserve ses trois échecs de fixture observés : une
autre lecture who native restait locale ; l'assertion status attendait un
libellé différent du message fermé réel ; isolation ne créait qu'un fichier
de nom, sans credential propriétaire. Ces données réelles ont motivé les
corrections de test autorisées. Aucun code de production n'a changé.

Validation du script Python097 seul, binaire debug déjà compilé, sans Cargo :

```text
/usr/bin/python3 /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/fixtures/claude_interactive_097.py /Volumes/8TB2/01-workflow/bridget-138-cargo.HjVvbG/debug/bridget --basic
```

PASS : présence PTY, remise/ACK, journal humain et assistant, restauration du
terminal et sortie0. La commande est exécutée depuis la racine absolue du
worktree inscrite au début du rapport.
Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-fixture097-green.log.

Première vague Cargo ciblée avec `--no-fail-fast`, session33569 :
0978 PASS/1 ignoré,0901 PASS/15 ignorés, puis trois échecs de fixture décrits
ci-dessus. Log conservé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-fixtures-global-green.log.

Vague finale Cargo ciblée avec `--no-fail-fast`, session25365, exit0 :
isolation8 PASS/1 ignoré, native2 PASS/3 ignorés, status2 PASS/1 ignoré,
fédération0 PASS/5 ignorés. Aucun test SSH ni fournisseur payant lancé.
Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-fixtures-global-final.log.

Les six fichiers de tests sont à nouveau gelés. Le target Cargo est rendu
au principal pour le workspace complet, clippy et release. Cette migration
de harnais ne revient pas sur la portée locale par défaut du produit.

## Deux oracles historiques complémentaires après workspace V3

Le workspace V3 `--no-fail-fast` a montré deux RED réels supplémentaires,
conservés dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/workspace-final-v3.log.

- Ligne1355 : le faux pair de concurrence attendait ListAgents directement.
  La façade actuelle utilise son auxiliaire attesté et négocie cap138 avant
  DirectoryScoped. Le fake attend désormais le credential exact fourni au
  vrai processus MCP, puis ClientHello138 et DirectoryScoped(SameProject).
- Ligne1542 : le lecteur des exemples de skill supposait un objet par bloc
  JSON. Un bloc contenant plusieurs objets provoquait `trailing characters`.
  Le lecteur utilise maintenant le flux de valeurs serde_json. Il ne filtre
  aucun exemple et exige les six objets publiés.

Fichiers de tests seuls adaptés :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/core_089_concurrency_test.rs.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/core_089_skill_test.rs.

L'oracle de concurrence conserve les huit connexions ouvertes, avec les huit
requêtes d'annuaire réellement reçues et aucune réponse finale anticipée.
Le neuvième appel doit être busy sans neuvième socket. Après libération,
les huit résultats corrélés aux identifiants1 à8 doivent être présents.

L'oracle skill exécute les deux annuaires, l'envoi interprojets avec motif,
la demande suivie, la réponse liée et le ledger. Deux dépôts Git réels prouvent
le groupe local et le destinataire extérieur. Le hostname de registration
matche celui du daemon isolé : le helper historique fixe un autre hostname,
donc son constructeur n'est pas adapté à cette preuve. Client,
save_fixture_credential, McpProcess et deliver_and_retry restent réutilisés.
Aucun nouveau harnais ou helper n'est créé.

Les trois reçus d'envoi sont ACKés puis rejoués à l'identique. La réponse
doit clôturer la demande en answered. Le ledger doit compter exactement trois
messages, chacun une seule fois, et la CLI doit afficher les trois corps.
Une attente de deux messages ou l'omission du reçu interprojets n'est pas
utilisée pour fabriquer un succès. Le warning cross_project est vérifié.

Les deux fichiers sont gelés après rustfmt et diffcheck PASS. Aucun Cargo
n'est lancé pendant le build release réservé au principal. Leur exécution
GREEN reste à la charge du workspace V4 ; aucun PASS de ces deux oracles
n'est revendiqué ici avant ce résultat. Aucun code de production modifié.

## Oracle de parité historique : rappel attesté après le corpus

Le workspace V4 conserve un RED réel dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/workspace-final-v4.log,
lignes1689–1714. Le même target avait passé ses sept tests dans V3.
La matrice a reçu vingt événements au lieu de seize : les quatre tours métier
étaient complets, suivis de quatre événements pour un rappel Bridget.

Les données de la fixture restée sur disque prouvent la corrélation :
/private/tmp/bg909-matrix-1-4367-41f50358-ab58-40ef-8082-ce735a765358/state/sessions/208fe66b-0000-4000-8000-023e208fe66b/2026-10-06.jsonl.
Le tour17–20 porte message_id=face6667686d4, from=bridget, reply=false,
et le corps exact du rappel #7ac94938. Le message7ac949384a6d4 est QUEUE-SLOW.
La base existante, ouverte en lecture seule, atteste dans
guichet_coordination_events ce couple request_id/reminder_message_id,
generation1 et le destinataire exact. Les quatre demandes sont answered.
QUEUE-SLOW a reçu reminder_deferred à18:38:22, puis answered à18:38:23 ;
le rappel effectivement remis est attesté à18:38:23.

La fixture ne lit pas who/agents via la CLI ; elle utilise ListAgents brut.
Le seul changement138 antérieur de cette matrice était le pattern Ack {..}.
Le wrapper libère busy avant l'envoi de sa réponse. La surveillance peut donc
remettre le rappel différé dans cette fenêtre. collect_journal retirait
uniquement bridget-reprise et comptait le rappel comme un cinquième tour métier.
Aucune perte du corpus ni suppression de relance n'est démontrée.

Recheck sans Cargo du binaire V4 inchangé : session88363, exit0,
1 PASS/6 filtrés,35,94s. Cette réussite isolée confirme l'intermittence ;
elle ne remplace pas le RED V4. Log :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/managed-parity-v4-recheck.log.

Adaptation autorisée TEST ONLY dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/managed_parity_test.rs.
Le journal métier conserve les quatre IDs exacts, le sender réel, reply=true,
les quatre réponses exactes et toutes les assertions answered/deferred.
Seuls les rappels dont le reçu durable vise une de ces demandes et ce
destinataire exact peuvent être séparés. Leur turn_start doit avoir from=bridget,
reply=false et le corps exact du palier1 ou2, avec sender et ID corrélés.
Le journal système reste conservé à part et exige les quatre événements
ordonnés, reasoning available:false et terminal end_turn.
Une origine, un ID, un corps ou un reçu non conforme reste dans l'oracle
métier et fait échouer le test. Aucun filtre général de préfixe ou d'auteur.

rustfmt et diffcheck PASS. Aucun code de production changé. Le test adapté
est gelé ; son GREEN compilé reste à établir après la libération du target
Cargo réservé au clippy principal.
