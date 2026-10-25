# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/reuse-audit.md](../../../../../../specs/147-panneau-bridget-vivant/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

| Requete | Portee | Resultat |
| --- | --- | --- |
| `rg --files` puis recherche par nom/responsabilité des gardes, mutations, clients et capabilities | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates | Passage global conduit par exploration coordonnée du principal, puis contrôles ciblés ; garde humaine, transaction et absence du nouveau watch confirmées. |
| Recherche des noms Reader, RPC, subscription, supervisor, ChildProcessSpawner et rightPanelStore | /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps et packages | Passage global des explorateurs réutilisé ; points d'extension natifs identifiés, aucun second transport/store requis. |
| `sed` ciblés guard11790, client147, tx_result717, Reader74, runtime651, store274/293 et snapshot718/2034 | Fichiers absolus cités dans le tableau des items | Contrôles documentaires de preuves critiques ; la borne snapshot masque un remplacement futur, d'où staging atomique commun. |
| Lecture manifests, spec146, plan145 et références globales | Les deux worktrees et /Users/moi/.speckit | Piles réelles, scripts et patterns conservés ; divergence Next.js/Pytest justifiée. |
| Sources W3C, web.dev, Testing Library et Fowler | Pages publiques listées dans research.md | Principes focus/reprise/tests ; aucune source externe utilisée comme preuve du code local. |

<a id="extrait-3"></a>

## Extrait 3

La cible regional-wrkr-1/claude_glm/.claude-glm possède une configuration effective sans Bridget vérifiée ; T3 injecte seulement t3-code. Le principal a validé le plan concret et les trois amendements de revue : barrière avant sessions.set/ready/prompt ; timeout d'une promesse non annulable ferme candidate/consumer et interdit un montage tardif ; métadonnées MCP fail-closed distinguent NotFound/I/O sans changer Skills fail-open. Les explorations RPC fournissent les responsabilités et consommateurs réels, sans refaire une exploration globale.

<a id="extrait-4"></a>

## Extrait 4

| Responsabilité supplémentaire | Existant vérifié et décision |
|---|---|
|20. Chemins des réglages Claude | Extraire le calcul existant de /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeSkills.ts vers ClaudeSettingsPaths.ts dans le même dossier. Consommateurs Skills/MCP ; exception architecture<=3 approuvée car le calcul de scope sécurise les deux usages. Aucune nouvelle politique Skills. |
|21. Résolution de l'exécutable Bridget | Extraire la résolution de /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts vers BridgetExecutable.ts dans le même dossier. Trois usages read/watch/MCP, mêmes priorités et rejet cmd/bat ; aucune dépendance ou service nouveau. |
|22. Politique MCP native de sécurité | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.ts fournit createQuery et le raccord SDK. ClaudeMcp.ts dans provider/Drivers porte une vraie politique Effect : status, homonymes, opt-out, identité héritée, permissions et barrière. Pas de classe enveloppe neutre ; t3-code est conservé. |
|23. Preuve d'autorité MCP et qualification refresh | Réutiliser spec145_human_view_tests de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs : façade OS native3PASS reçue, daemon gelé/libéré à membership. Complément approuvé avant code dans tests spec101_identity de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/t3code_identity.rs : refresh réel, DBT3 privée et PID/birth/lineage natifs ; tuple live/HTTP synthétique. Aucun code fonctionnel/helper neuf, mcp.rs ou modèle. Le stub UUID ne prouve pas cette autorité et le complément ne prouve pas T3+SDK+modèle complet. |

<a id="extrait-5"></a>

## Extrait 5

Amendement conservateur principal/RPC : pour cwd enfant, la politique MCP inspecte les métadonnées root/.claude/settings.json et projects[root] de .claude.json afin de ne pas contourner un refus d'auto-montage T3. Ceci n'élargit pas les paths/comportements Skills. Le getter CLI privé prouve root/.mcp.json scope project seulement ; son RC0 sous disabled ne révèle pas l'état d'activation. Réutiliser le calcul root existant, pas un nouveau chargeur générique.

<a id="extrait-6"></a>

## Extrait 6

| Responsabilité | Existant et décision |
|---|---|
|27. Choix natif et détails sans répétition | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx : réutiliser liste, détail autorisé, details/ScrollArea/messages seq et tests BridgetPanel.test.tsx. Main relit Sidebar1449/indexcss1081/PreviewPanelShell133 : bg-sidebar/row-active/foreground, ringinset et état accessible natifs, pas nouveau composant/API/store/deps/helper. |
