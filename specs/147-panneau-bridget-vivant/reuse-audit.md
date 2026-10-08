# Audit de reutilisation de l'existant — SPEC147 panneau Bridget vivant

## Decision

Statut: PASS premier lot et gates de conception UUID US6 / MCP US7. Extension membres US8 : plan accepté, gate détaillé en préparation.
Date: 2026-10-08
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant

Conclusion courte: le plan étend l'autorité humaine, les lectures146, le transport RPC et le store natif. Le contrat de suivi humain et son bus d'invalidation borné sont les seules responsabilités nouvelles nécessaires. Les abonnements qui réveillent les agents sont volontairement exclus. PASS vaut pour la conception avant tasks, pas pour le code ou l'installation.

## Synthese

| Metrique | Valeur |
| --- | ---: |
| Items extraits du plan | 18 |
| Items audites | 18 |
| Reutilisations deja prevues | 16 |
| Existants potentiellement pertinents non mentionnés | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 2 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
| --- | --- | --- | --- |
| 1. Garde humaine commune | `handle_human_thread_view` | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:11790 | Extraire la garde, pas une deuxième politique d'identité ou projet. |
| 2. Contrat HumanThreadWatchV1 | Contrats humains distincts des requêtes agent | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-transport/src/protocol.rs:189 et2159 | Nouveau contrat nécessaire, aucun équivalent de suivi humain au socle. Pas d'ouverture globale de ThreadRequest au Client. |
| 3. Résultat de mutation réelle | Done/Replayed/Refused transactionnels | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:717 | Étendre la projection pour préserver la preuve de commit. Create manque au classement mutatif générique : daemon.rs12624. |
| 4. Invalidation nom d'annuaire | `rename_display_name` et résolution des noms | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:15561 | Ajouter signal seulement quand le nom a réellement changé ; pas de CRUD titres/membres. |
| 5. Bus humain borné | Aucun bus de vue humain existant ; observations proches mais mutatives | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-transport/src/protocol.rs | Créer seulement la mémoire de suivi bornée, sans journal durable. Réutiliser les observations agent serait une fausse équivalence. |
| 6. Client nu et CLI watch | `human_thread_view` et chemin inspect | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/communication/client.rs:147 | Étendre la négociation et le décodage, pas Register/autostart/credentials. |
| 7. Service Reader.watch | `BridgetReader.read` et contexte via snapshots | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts:74 | Même résolution de conversation/projet et exécutable. Aucun chemin d'autorité browser. |
| 8. Ressource enfant de flux | Effect ChildProcessSpawner ; ProcessRunner pour sorties finies | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/processRunner.ts | Réutiliser la ressource de processus existante pour streaming scoped ; garder ProcessRunner unary pour read. |
| 9. Schéma et RPC watch | BridgetReadInput et WsBridgetReadRpc | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/bridget.ts et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/rpc.ts:998 | Ajouter un tagstream et un schéma fermé, pas un autre serveur ou format de lecture de corps. |
| 10. Autorisation RPC | AuthOrchestrationReadScope | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/auth/RpcAuthorization.ts | Même scope lecture que read ; aucune autorité agent ajoutée. |
| 11. Observation WS | observeRpcStream | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/ws.ts:709 | Étendre la table de handlers native ; pas de deuxième WebSocket. |
| 12. Subscription et reconnexion runtime | createEnvironmentRpcSubscriptionAtomFamily + supervisor | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/runtime.ts:651 et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/connection/supervisor.ts | idleTtl0 explicite ; ready neuf à reprise. Pas de polling métier. |
| 13. Table de sélection contextuelle | Persistance et actions useRightPanelStore | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/rightPanelStore.ts:274 et293 | Table de UUID distincte de ThreadRightPanelState reconstruit/nettoyé. Pas nouveau store. |
| 14. Restauration hors première page | show/history_recent autorisés146 | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:285 | Retrouver directement la référence ; pas inventaire de toutes les pages. |
| 15. Revalidation snapshot atomique | Projection history_recent et to_seq commun | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/store/threads.rs:718 et2034 | Nouveau snapshot S puis reconstruction du segment chargé ; une simple fusion de snapshots différents masquerait un remplacement. |
| 16. Gestes de lecture et générations | Panel146, dépliages/détails et générations de requête | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx | Étendre la réconciliation sans remontage des lignes inchangées, préserver focus/copietexte et rendre A→B→A sûr. |
| 17. Ready attesté malgré rendu coalescé | Factory de subscription native, Stream.scan et état volatil dans orchestration | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts:18–58 | Étendre le mémo event/readyGeneration/subscriptionId et la clé visitId locale, sans store générique, champ wire ni nouveau canal. Code ciblé lu ; RED/GREEN, recette native et gate final validés. |
| 18. CLI rompu sous WebSocket sain | Portée de stream Reader + Schedule Effect ; supervisor WS conservé | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/connection/supervisor.ts | Étendre le stream existant par initiale + trois reprises techniques maximum. Aucun retry métier ni timer de corps. Convergence autorisée du principal, correction/preuves en cours. |

Ajout US5 : l'item16 étend aussi la ligne auteur/date déjà présente avec notify/targets et detail.members déjà autorisés. Aucun nouvel item de transport/service/store/annuaire. Zéro API ou appel supplémentaire, corps et copie exacts conservés. La présentation indique une sollicitation, jamais une livraison confirmée. Tests et QA de T027–T030 restent à prouver.

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
| --- | --- | --- | --- |
| Aucun | Observations agent et sollicitations de fil déjà examinées | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/research.md | Équivalence rejetée explicitement : réveils/notifications métier incompatibles avec la consultation silencieuse. Aucun arbitrage utilisateur requis. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
| --- | --- | --- | --- |
| Aucun | Aucun contrat de suivi humain déjà disponible | Inventaire délégué consolidé et preuves des18 items ci-dessus | Conserver les extensions ciblées ; ne pas créer service, store, DB ou framework parallèles. |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
| --- | --- | --- |
| /Users/moi/.speckit/constitution.md | III/XV/XVI : workflow, preuve avant clôture, isolation | Plan avant tasks ; RED/GREEN, worktrees147 et autorisation de livraison distincte. |
| /Users/moi/.speckit/constitution.md | XVIII/XIX/XX : complexité, minimalisme, responsabilité future | Dispatch et files bornés ; index de fusion, zéro nouvelle dépendance/table ; limites documentées. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/.specify/memory/constitution.md | Pont vers constitution globale | Source globale lue, pas une règle métier Cartae importée par défaut. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/.specify/memory/standards.md et user-layer.json | Patrimoine utilisateur et scripts de publication | Les templates/scripts projet absents sont documentés ; aucun outillage régénéré. |
| /Users/moi/.speckit/ref/standards-frontend.md et standards-tests.md | Typage, tests de comportement et accessibilité | Piles Next.js/Pytest génériques inadaptées : utiliser React/Effect/Vitest/Rust des dépôts ; conserver les principes de qualité. |
| /Users/moi/.speckit/ref/code-quality-details.md | Contrats, validation frontière, transaction et erreurs | Signal après commit, schémas fermés, erreur codée, ressource annulable et pas d'erreur brute. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/AGENTS.md | Langue, chemins, sécurité Git et périmètre | Français ; chemins absolus ; pas de commit/push/install/restart par l'agent documentaire, travaux d'autrui conservés. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
| --- | --- | --- |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/145-bridget-thread-viewer/plan.md | Lecture humaine distincte de l'agent, contexte attesté et projection bornée | Autorité et service existants réutilisés, aucune notification agent au panneau. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/146-bridget-panel-lisible/spec.md | Ordre récent, snapshots, copie exacte, détails et sélection native | Style conservé ; nouvelles invalidations doivent reconstruire les pages cohérentes sans perdre les gestes de lecture. L'ancien refresh manuel reste disponible. |

## Journal de recherche

| Requete | Portee | Resultat |
| --- | --- | --- |
| `rg --files` puis recherche par nom/responsabilité des gardes, mutations, clients et capabilities | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates | Passage global conduit par exploration coordonnée du principal, puis contrôles ciblés ; garde humaine, transaction et absence du nouveau watch confirmées. |
| Recherche des noms Reader, RPC, subscription, supervisor, ChildProcessSpawner et rightPanelStore | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps et packages | Passage global des explorateurs réutilisé ; points d'extension natifs identifiés, aucun second transport/store requis. |
| `sed` ciblés guard11790, client147, tx_result717, Reader74, runtime651, store274/293 et snapshot718/2034 | Fichiers absolus cités dans le tableau des items | Contrôles documentaires de preuves critiques ; la borne snapshot masque un remplacement futur, d'où staging atomique commun. |
| Lecture manifests, spec146, plan145 et références globales | Les deux worktrees et /Users/moi/.speckit | Piles réelles, scripts et patterns conservés ; divergence Next.js/Pytest justifiée. |
| Sources W3C, web.dev, Testing Library et Fowler | Pages publiques listées dans research.md | Principes focus/reprise/tests ; aucune source externe utilisée comme preuve du code local. |

## Arbitrages

| Sujet | Decision | Justification | Date |
| --- | --- | --- | --- |
| Signal humain versus observations agent | Créer nouveau contrat minimal, étendre garde/transport existants | Les observations réveillent les agents ; cette vue ne doit pas le faire. | 2026-10-08 |
| Signal global versus UUID de fil | version/generation/seq/status seulement, choix du principal | Moins de fuite et moins de logique ; relecture liste+détail groupée et bornée. | 2026-10-08 |
| Staging commun versus fusion d'anciens snapshots | Nouveau S, relecture segment consulté, publication atomique | to_seq ancien masque superseded_by_seq récent ; test page2/page3 prévu. | 2026-10-08 |
| Stockage sélection | Étendre store natif par table de références distincte | Éviter perte lors reconstruction/cleanup de surface, sans persister contenu. | 2026-10-08 |
| ProcessRunner versus ChildProcessSpawner | read reste unary ; watch scoped streaming | Timeout/output collecté de ProcessRunner incompatibles avec un abonnement long. | 2026-10-08 |
| Fallback scripts/templates | Outline manuel, aucune génération d'outillage | Fichiers officiels absents ; ne pas inventer une exécution réussie. | 2026-10-08 |
| Ready côté wire versus DOM | État sticky natif et visitId local | Un rendu peut absorber ready ; conserver sa preuve sans ajouter de champ wire. | 2026-10-08 |
| Coupure WS versus coupure CLI | Supervisor WS natif et Schedule de stream borné | Un WebSocket sain ne détecte pas une rupture CLI ; trois reprises techniques maximum, pas de polling. | 2026-10-08 |
| Tête50 versus grand intervalle nouveau | Pages146 intermédiaires jusqu'à l'ancre ancienne, puis staging commun S | Rattraper les nouveautés sans trou ni lecture de toute la base. | 2026-10-08 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree.
- [x] Chaque item extrait du plan a une ligne d'audit :18/18. Gate initial16/16 complété par deux extensions natives lors de convergence autorisée.
- [x] Les regles projet applicables ont ete lues. Les explorations coordonnées sont attribuées, pas revendiquées comme un deuxième audit complet.
- [x] Les specs existantes proches ont ete verifiees :145/146.
- [x] Le plan.md a ete refactore ou les divergences sont justifiees : signal global retenu, snapshots cohérents et piles natives.

Gate documentaire5/5 PASS du premier lot conservé ; ses30 tâches et convergence sont validées. Les gates UUID et MCP ci-dessous sont validés par le principal avant leurs implémentations. Une validation de conception ne vaut pas réussite des nouveaux tests.

## Gate d'extension UUID US6 — REUTILISER, validé par le principal

| Responsabilité supplémentaire | Existant vérifié | Décision |
|---|---|---|
|19. UUID de fil partagé normalisé avant idempotence/SQL | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:493 require_uuid et:350 canonical_uuid ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs:1519 thread_members ; ACK threads.rs:1027 | Étendre require_uuid pub(crate), réutiliser parser uuid et tests colocalisés. Garder canonical_uuid humain strict et noms/préfixes CLI inchangés. Aucun autre helper, dependency ou migration. |

Recherche ciblée réelle : rg des noms require_uuid/thread_members/canonical_uuid dans threads.rs et cli.rs, puis sed des blocs350/493/1519. Le principal a validé le plan UUID avant implémentation. Cet item porte une responsabilité partagée par Create/Post/Close/ACK/membres/targets ; elle n'est pas un wrapper supplémentaire.

- [x] Existant central identifié et réutilisé.
- [x] Aucun helper ou validateur humain dupliqué.
- [x] UUID canonique avant hash/SQL, reçu ACK cohérent et erreur ReceiptInvalid conservée.
- [x] Noms/préfixes et corps inchangés ; garde humaine stricte conservée.
- [x] Tests comportementaux et non-régression colocalisés, aucun modèle/production.

Gate US6 PASS5/5 de conception. Les reçus UUID RED2PASS/8FAIL puis GREEN26PASS/1ignored et non-régressions sont reçus ; le principal autorise T031–T034 cochées. La façade MCP UUID a une frontière d'authentification simulée et ne prouve pas l'autorité US7.

## Gate d'extension MCP US7 — validé avant code

La cible regional-wrkr-1/claude_glm/.claude-glm possède une configuration effective sans Bridget vérifiée ; T3 injecte seulement t3-code. Le principal a validé le plan concret et les trois amendements de revue : barrière avant sessions.set/ready/prompt ; timeout d'une promesse non annulable ferme candidate/consumer et interdit un montage tardif ; métadonnées MCP fail-closed distinguent NotFound/I/O sans changer Skills fail-open. Les explorations RPC fournissent les responsabilités et consommateurs réels, sans refaire une exploration globale.

| Responsabilité supplémentaire | Existant vérifié et décision |
|---|---|
|20. Chemins des réglages Claude | Extraire le calcul existant de /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeSkills.ts vers ClaudeSettingsPaths.ts dans le même dossier. Consommateurs Skills/MCP ; exception architecture<=3 approuvée car le calcul de scope sécurise les deux usages. Aucune nouvelle politique Skills. |
|21. Résolution de l'exécutable Bridget | Extraire la résolution de /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts vers BridgetExecutable.ts dans le même dossier. Trois usages read/watch/MCP, mêmes priorités et rejet cmd/bat ; aucune dépendance ou service nouveau. |
|22. Politique MCP native de sécurité | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.ts fournit createQuery et le raccord SDK. ClaudeMcp.ts dans provider/Drivers porte une vraie politique Effect : status, homonymes, opt-out, identité héritée, permissions et barrière. Pas de classe enveloppe neutre ; t3-code est conservé. |
|23. Preuve d'autorité MCP et qualification refresh | Réutiliser spec145_human_view_tests de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs : façade OS native3PASS reçue, daemon gelé/libéré à membership. Complément approuvé avant code dans tests spec101_identity de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/t3code_identity.rs : refresh réel, DBT3 privée et PID/birth/lineage natifs ; tuple live/HTTP synthétique. Aucun code fonctionnel/helper neuf, mcp.rs ou modèle. Le stub UUID ne prouve pas cette autorité et le complément ne prouve pas T3+SDK+modèle complet. |

- [x] Responsabilités existantes et consommateurs réels identifiés.
- [x] Extractions partagées justifiées ; aucun wrapper neutre, endpoint, table ou dépendance ajouté au plan.
- [x] Barrière native et cleanup timeout/tardy explicitement retenus.
- [x] Homonymes, opt-out, identité héritée et permissions conservés ; métadonnées MCP fermées et Skills inchangé.
- [x] Six groupes RED SDK/configuration et preuve MCP OS privée distincte définis dans T035–T041.

Gate US7 PASS5/5 de conception par décision du principal. Les23 responsabilités incluent18 initiales,1 UUID et4 MCP. Les tests SDK finaux US7 restent ouverts. Les preuves natives MCP et refresh ci-dessous sont séparées, aucun catalogue d'agent actif ou E2E modèle annoncé.

Amendement conservateur principal/RPC : pour cwd enfant, la politique MCP inspecte les métadonnées root/.claude/settings.json et projects[root] de .claude.json afin de ne pas contourner un refus d'auto-montage T3. Ceci n'élargit pas les paths/comportements Skills. Le getter CLI privé prouve root/.mcp.json scope project seulement ; son RC0 sous disabled ne révèle pas l'état d'activation. Réutiliser le calcul root existant, pas un nouveau chargeur générique.

Amendement lifetime T050 accepté avant code : fermeture candidate/consumer du ClaudeAdapter existant sous propriétaire local unique jusqu'à sessions.set. Aucun framework/helper neutre. RED prepare réussi puis close ancienne query en échec, candidate fermée/noready ; tests flags isolés automaticMcp:true/envprivé sans identité héritée. Mêmes FR21/22, pas responsabilité ou fonctionnalité nouvelle ;26 items conservés.

Portée UUID US6 : seulement les actions et l'outil de fils partagés create/post/read/ack/history/show/close et leurs références UUID thread/operation/membres/targets/reçus. Aucun changement des IDs opaques send ou d'autres outils, de l'acteur d'autorité, des noms/préfixes UUID partiels, de canonical_uuid humain ni des curseurs fermés.

## Gate US8 — réutilisation du contrat de fils, plan accepté

| Responsabilité supplémentaire | Réutilisation vérifiée / décision |
|---|---|
|24. Action et négociation d'ajout | ThreadAction protocol.rs2647, communication/client.rs thread_request122–147 et daemon.rs ThreadRequest13943/allowlist15176 : étendre enums/capacité/allowlist existants, hello unique avec projects, garde capacité attestée ; pas nouvelle API autonome. |
|25. Transaction et idempotence | threads.rs handle_with_change573/require_uuid493/digest/replay ; store/threads.rs ensure_schema303/CHECK352 : union complète validée et recheck transactionnel, migration cibléev2→v3, NoChange sans reçu ni clé engagée. Réutiliser tables/reçus/index, aucun nouveau journal. |
|26. Façades et preuve silencieuse | cli.rs thread_members/parser1541, mcp.rs schema484, fixtures privées existantes et watch humain147 : ajout dans CLI/MCP, all futur/targets passés, historique complet, cursor0, wakepending non dispatché et signal après commit seulement. Pas nouveau harness, composeur ou timer. |

Racine absolue de ces sources : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates. Findings compact_t3 et vérification principale réutilisés, pas exploration globale répétée. Harness CLI/MCP existant : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/tests/spec102_threads_test.rs ; fixture migration136 et watch147 colocalisées dans store/threads.rs et daemon.rs, pas nouveau harness. Les docs locales docs/reference-communication.md, skills/bridget/SKILL.md et skills/bridget/references/commandes.md sont mises à jour par le propriétaire membership seulement, sans publier les compétences globales.

- [x] Responsabilités existantes et champs/tables proches identifiés par recherche ciblée.
- [x] Extensions minimales, sans dépendance, wrapper, nouveau harness ou base parallèle.
- [x] Créateur/ouvert/16 et union autorisée revalidés transactionnellement ; migration préserve reçus/index.
- [x] Hello combiné auxiliaire/capacité attestée et exclusion dispatch_wakes explicitement requis.
- [x] Tests RED comportementaux et négociation réelle définis dans T042–T048, suivi de GREEN/régressions/contre-revue.

Gate documentaire US8 PASS5/5 accepté par le principal après deux passes Analyze, aucun résultat produit anticipé.26 responsabilités retenues. Capacité figée ClientCapability::ThreadMembersV1 / thread_members_v1 ; sérialisation et migrationsv1/v2→v3, seconde ouverture/rollback/reçus exacts dans T043/T047. Le gate ne vaut pas autorisation de déployer ou de publier des compétences globales.

## Gate US9 — REUTILISER panneau et tokens natifs, avant tâches/code

| Responsabilité | Existant et décision |
|---|---|
|27. Choix natif et détails sans répétition | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx : réutiliser liste, détail autorisé, details/ScrollArea/messages seq et tests BridgetPanel.test.tsx. Main relit Sidebar1449/indexcss1081/PreviewPanelShell133 : bg-sidebar/row-active/foreground, ringinset et état accessible natifs, pas nouveau composant/API/store/deps/helper. |

- [x] Panneau et wrappers/styles hérités inspectés avant modification par Web/principal.
- [x] Tokens sélection et contraste clair réutilisés, pas rebrand ou CSS/API parallèle.
- [x] Détails déplacés frère du bouton ; ScrollArea/messagekeys/copie conservés.
- [x] Choix horspage/filtre épinglé unique mêmeparent/key seulement sur détail autorisé ; masque/refus sans fuite.
- [x] Cinq groupes RED comportementaux et recette isolée prévus ; aucune opération d'agent ou nouveau stockage.

Plan US9 accepté par le principal, gate5/5 de conception documentaire.27 responsabilités retenues ; aucune réussite visuelle ou test anticipée. Main autorise les tâches US9 après ce gate, pas livraison/install/restart.
