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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/reuse-audit.md#extrait-1).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/reuse-audit.md#extrait-2).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/reuse-audit.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/reuse-audit.md#extrait-4).

- [x] Responsabilités existantes et consommateurs réels identifiés.
- [x] Extractions partagées justifiées ; aucun wrapper neutre, endpoint, table ou dépendance ajouté au plan.
- [x] Barrière native et cleanup timeout/tardy explicitement retenus.
- [x] Homonymes, opt-out, identité héritée et permissions conservés ; métadonnées MCP fermées et Skills inchangé.
- [x] Six groupes RED SDK/configuration et preuve MCP OS privée distincte définis dans T035–T041.

Gate US7 PASS5/5 de conception par décision du principal. Les23 responsabilités incluent18 initiales,1 UUID et4 MCP. Les tests SDK finaux US7 restent ouverts. Les preuves natives MCP et refresh ci-dessous sont séparées, aucun catalogue d'agent actif ou E2E modèle annoncé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/reuse-audit.md#extrait-5).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/reuse-audit.md#extrait-6).

- [x] Panneau et wrappers/styles hérités inspectés avant modification par Web/principal.
- [x] Tokens sélection et contraste clair réutilisés, pas rebrand ou CSS/API parallèle.
- [x] Détails déplacés frère du bouton ; ScrollArea/messagekeys/copie conservés.
- [x] Choix horspage/filtre épinglé unique mêmeparent/key seulement sur détail autorisé ; masque/refus sans fuite.
- [x] Cinq groupes RED comportementaux et recette isolée prévus ; aucune opération d'agent ou nouveau stockage.

Plan US9 accepté par le principal, gate5/5 de conception documentaire.27 responsabilités retenues ; aucune réussite visuelle ou test anticipée. Main autorise les tâches US9 après ce gate, pas livraison/install/restart.
