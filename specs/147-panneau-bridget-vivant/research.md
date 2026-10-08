# Recherche et décisions — SPEC147

Date : 2026-10-08. Recherche ciblée de conception. Aucun résultat d'exécution147 revendiqué.

## Sources et méthode

Le principal a conduit les explorations Bridget, runtime T3 et UI. Leurs preuves sont réutilisées ici. Un contrôle documentaire ciblé a confirmé la garde humaine, la distinction Done/Replayed, la résolution serveur, la subscription runtime et les reconstructions du store. Aucun deuxième balayage complet n'a été réalisé.

Baselines lues : /Users/moi/.speckit/research/07-frontend-design-systems.md et /Users/moi/.speckit/research/08-testing-quality.md. Elles orientent vers accessibilité, cycle de vie et tests de comportement. Elles ne prouvent pas un abonnement existant dans le panneau.

Sources primaires consultées le 2026-10-08 :

- [W3C WAI — On Focus](https://www.w3.org/WAI/WCAG21/Understanding/on-focus.html) : une prise de focus ne doit pas provoquer de changement de contexte. Décision locale : ne pas déplacer le focus lors d'une revalidation et conserver les contrôles inchangés. Cette page ne prouve pas une conformité complète de T3.
- [web.dev — Back/forward cache](https://web.dev/articles/bfcache?hl=en) : une page peut être suspendue puis restaurée. Décision locale : libérer le suivi quand la page n'est plus active, puis revalider à la reprise ; ne pas supposer que la connexion a livré tous les événements.
- [Testing Library — Guiding Principles](https://testing-library.com/docs/guiding-principles/) : les tests utiles ressemblent à l'usage du logiciel. Décision locale : vérifier sélection, contenu, copie et focus avec les contrôles visibles, pas seulement un compteur de mocks.
- [The Practical Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html) : répartir tests rapides, intégration et parcours complets. Décision locale : tester le transport et la garde aux frontières, puis un parcours réel isolé ; éviter une recette exclusivement simulée.

## Décision R147-01 — Abonnement humain séparé

Décision : ajouter `human_thread_watch_v1`. Ses événements sans contenu invalident les lectures humaines146.

Pourquoi : observations et sollicitations des agents peuvent produire du bruit et des réveils. La consultation humaine doit rester lecture seule et conserver une autorité distincte.

Alternatives : polling de contenu rejeté par FR147-03 ; observation `turn_ended` rejetée car ce n'est pas un changement du fil ; abonnement agent rejeté pour ses effets métier ; nouveau serveur de notifications rejeté car IPC et RPC existent.

Charge future : un contrat fermé et une garde commune sont inspectables. Aucun journal ou worker de notification supplémentaire.

## Décision R147-02 — Reprise par resync, pas par journal durable

Décision : inscription avant `ready`, génération propre à chaque connexion, séquence volatile croissante, invalidation bornée et rattrapage par les lectures146. Le signal porte seulement version, génération, séquence et statut ; aucun UUID de fil. Le daemon filtre les changements selon l'accès attesté, puis la vue relit liste et détail en groupe.

Pourquoi : aucun compteur de changements durable n'est nécessaire pour savoir qu'une vue doit être relue. Une coupure ou une saturation peut perdre des signaux, mais `ready`/`resync` imposent la revalidation de l'état actuel.

Alternatives : journal SQLite d'événements rejeté pour migration et maintenance inutiles ; conserver silencieusement une file non bornée rejeté ; lire avant inscription sans rattrapage rejeté pour fenêtre de perte.

Charge future : le mainteneur n'a qu'à vérifier l'ordre inscription/ready, les bornes et les lectures existantes. La mémoire du bus n'est jamais la source de vérité des messages.

## Décision R147-03 — Signaler uniquement une mutation réelle

Décision : préserver Done/Replayed au passage transactionnel avant `tx_result`. Déclencher après commit Create/Post/Close ; supersedes est un Post. Un renommage d'annuaire réellement modifié peut invalider les vues concernées.

Pourquoi : `tx_result` projette aujourd'hui Done et Replayed en même résultat JSON. Le test générique mutatif exclut Create. Un signal basé uniquement sur ces projections manquerait des créations ou inventerait des changements au rejeu.

Alternatives : émettre sur chaque requête rejeté pour bruit ; observer les fichiers SQLite rejeté car imprécis et lié au stockage ; nouveau CRUD de titre/membres rejeté car hors demande.

Charge future : le signal décrit l'effet committé, pas le nom supposé d'une commande. Les mêmes tests vérifient le chemin réel et son rejeu.

## Décision R147-04 — Étendre le lecteur T3 et ses flux natifs

Décision : `BridgetReader.watch` résout le contexte comme `read`. Le flux enfant utilise la ressource Effect `ChildProcessSpawner`, puis `bridget.watch` réutilise RPC, autorisation de lecture et supervisor existants. `idleTtlMs: 0` est explicite.

Pourquoi : `ProcessRunner.run` collecte une sortie finie ; un watch est long. Le défaut de rétention cinq minutes du runtime ne respecte pas fermeture → zéro suivi.

Alternatives : interval React rejeté ; nouveau serveur WebSocket rejeté ; détacher un processus CLI sans scope rejeté pour fuite ; chemin racine fourni par navigateur rejeté pour autorité non attestée.

Charge future : transport WebSocket repris par supervisor natif, sans protocole de reconnexion parallèle. La convergence autorisée ajoute une reprise technique bornée du stream CLI si le WebSocket reste sain : Schedule existant, initiale + trois reprises maximum, uniquement unavailable/command_failed/timeout. Refus métier et invalid_output restent terminaux. La CLI reste testable sans T3, le service est testable aux frontières et le runtime conserve ses conventions.

## Décision R147-05 — Références de sélection dans le store natif

Décision : `bridgetSelectionByContextKey`, hors `ThreadRightPanelState`, avec clé JSON environment/project/conversation et UUID seul.

Pourquoi : la sélection locale du panneau est perdue au démontage. `upsertSurface` reconstruit l'état de surface et `updateThread` peut retirer un état fermé ; y cacher le choix le rendrait fragile.

Alternatives : singleton du dernier UUID rejeté pour mélange A/B ; persister des corps rejeté pour contenu périmé et confidentialité ; nouveau store dédié rejeté car le store du panneau porte déjà la navigation ; recherche dans toutes les pages rejetée au profit de `show` autorisé.

Charge future : une table de références validées et deux actions ciblées suffisent. Les snapshots anciens n'ont aucune migration de données métier à effectuer.

## Décision R147-06 — Revalidation sans défaire la lecture

Décision : versions locales de lecture, un in-flight et un dirty par contexte, lignes stables et staging commun S des pages autorisées. Revalider les remplacements présents dans les pages déjà consultées. Si la tête50 ne rejoint plus l'ancre ancienne, lire les pages intermédiaires avant publication atomique.

Pourquoi : relire toute la première page en remplaçant le panneau ferait perdre pages, détails, focus et texte sélectionné. Comparer seulement le UUID de conversation n'empêche pas une ancienne réponse A d'être acceptée après A → B → A.

Alternatives : reset intégral à chaque signal rejeté ; ajouter les nouveautés sans réconcilier superseded_by_seq rejeté pour vieille consigne présentée courante ; stocker toute la base localement rejeté.

Charge future : les versions et clés explicites restent testables. La réconciliation coûte O(nouveautés + pages consultées) ; aucune I/O par message et aucun parcours de toute la base.

Convergence observée : ready premier sur le wire peut être coalescé avec changed avant le premier rendu de l'atome. Le runtime garde event/readyGeneration/subscriptionId ; un visitId local isole chaque visite. Les deux champs d'entrée RPC restent threadId/projectId. Cette précision ne crée ni persistance de corps ni nouveau transport. Corrections et preuves finales en cours, non déclarées achevées.

## Décision R147-07 — Piles et tests propres aux dépôts

Décision : Rust et Vitest/Testing Library/Effect, avec scénario intégré isolé. Les standards génériques Next.js, Axios et Pytest ne remplacent pas les outils réels de T3/Bridget.

Pourquoi : ces prescriptions visent d'autres projets. Ajouter leurs runtimes ou restructurer T3 ne testerait pas le transport effectivement livré.

Alternatives : générer une suite Python parallèle rejeté ; recette uniquement mocks rejetée ; recette sur conversations actives rejetée pour risque inutile.

Charge future : tests colocalisés et nommés147, commandes adaptées aux scripts existants ; limites explicites et preuves RED/GREEN conservées.

## Limites de recherche

Aucun second fournisseur adverse joignable n'a été trouvé dans l'annuaire du même projet ; seul le fil bdget Codex est attesté. Ne pas contacter des agents d'un autre projet pour contourner cette limite.

Les sources externes éclairent des principes. Les décisions de transport et d'autorité sont déduites du code local. Elles ne constituent pas une garantie d'implémentation ou un résultat de performance mesuré147.

Les décisions sont proposées au gate du plan. Le canal humain constitue la décision structurante à reprendre dans l'ADR de livraison. Aucun point marqué NEEDS CLARIFICATION ne reste dans ce dossier.

## Extension US6 — constats et décision validée

Lecture ciblée owner documentaire : require_uuid dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:493 appelle canonical_uuid ; celui-ci exige l'égalité exacte minuscule à:350. thread_members dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs:1519 ne reconnaît que cette forme stricte et envoie les autres valeurs au résolveur. ACK vérifie encore séparément canonical_uuid sur receipt à threads.rs:1027. Ces faits viennent de rg/sed réels, pas d'une hypothèse sur le daemon actif.

Décision main : étendre require_uuid pub(crate), comparer la seule forme hyphénée36 eq_ignore_ascii_case et retourner minuscule avant hash/SQL ; réutiliser dans thread_members et ACK avec ReceiptInvalid conservé. canonical_uuid humain reste strict. Aucun nouveau helper/dependency/migration. Gate UUID approuvé avant implémentation. Les reçus RED2PASS/8FAIL puis GREEN26PASS/1ignored et régressions sont reçus ; T031–T034 validées sur GO du principal.

## Extension US7 — sources et plan validés

Le principal a consulté le09/10 les sources primaires [Claude Code MCP](https://code.claude.com/docs/en/mcp) et [Agent SDK MCP](https://code.claude.com/docs/en/agent-sdk/mcp). Elles distinguent configuration utilisateur ~/.claude.json, configuration projet .mcp.json, commande/arguments stdio et découverte des outils. La découverte d'un outil n'atteste pas à elle seule une identité Bridget.

Sources de test consultées par le principal : [Integration Test](https://martinfowler.com/bliki/IntegrationTest.html) et [Testing Library](https://testing-library.com/docs/guiding-principles/). Elles justifient deux preuves distinctes : inspection d'options SDK de lancement en fixture et exercice d'une vraie façade MCP privée. Aucun résultat de catalogue runtime actif ni E2E modèle n'est déduit d'un test de configuration.

État courant : cible regional-wrkr-1/claude_glm/.claude-glm et absence de MCP Bridget effectif vérifiées en lecture seule. Le principal a validé le plan SDK natif status→setMcpServers avant sessions.set/ready/prompt, les extractions partagées de chemins et d'exécutable, l'opt-out et les homonymes. Les références externes ne prouvent pas cette configuration locale ; le reçu ci-dessous la corrobore. Les tests SDK sans modèle et la vraie autorité MCP privée sont des preuves distinctes encore en cours.

### Diagnostic local vérifié — reçu assaini du principal/RPC

Cause vérifiée en lecture seule par le principal/RPC : agent regional-wrkr-1, thread1288a673-d16e-473a-80dc-def0b2f776c8, adapter ClaudeAgent, provider claude_glm, modèle glm-5.3-flash. Home effectif /Users/moi/.claude-glm, exécutable gclaude, launchArgs sans flags. Les noms MCP home/projet sont vides. La commande `CLAUDE_CONFIG_DIR=/Users/moi/.claude-glm /Users/moi/.local/bin/claude mcp get bridget`, cwd /Users/moi/Nextcloud/10.Scripts/69.opus2D, sort RC1 et confirme l'absence de Bridget. Sortie assainie64 octets, raw non conservé. T3 injecte seulement t3-code ; ce constat ne signifie pas que GLM est interdit.

Owner documentaire n'a pas rejoué la commande. Aucun secret ZAI, contenu brut de configuration, modèle lancé ou configuration active modifiée. Le diagnostic ne prouve pas le succès d'une correction ni le catalogue d'un modèle actif.

Portée UUID US6 : seulement les actions et l'outil de fils partagés create/post/read/ack/history/show/close et leurs références UUID thread/operation/membres/targets/reçus. Aucun changement des IDs opaques send ou d'autres outils, de l'acteur d'autorité, des noms/préfixes UUID partiels, de canonical_uuid humain ni des curseurs fermés.

## US8 — décisions vérifiées et réutilisations

Findings compact_t3 réutilisés, plan et gate5/5 approuvés par le principal avant code : AddMembers/add_members, ThreadMembersV1/thread_members_v1, CLI --member répétable. ensure_schema/store existant contient CHECK des opérations et version2 ; migration cibléev3 conserve résultats/index/reçus. Union projet complète puis recheck transactionnel ; NoChange sans clé engagée ; ajout silencieux séparé des dispatch_wakes.

Tests existants à réutiliser, pas nouveau harness : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/tests/spec102_threads_test.rs (CLI/MCP) ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/store/threads.rs, test spec136_v1_schema_migration_preserves_legacy_and_is_repeatable ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs, test spec147_cli_real_watch_mutations_replay_idle_and_cancel_ten_cycles. Ces références établissent le plan, pas encore les résultats US8. Le seul writer daemon membership attend libération AUTH T040.
