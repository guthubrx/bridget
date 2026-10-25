# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/reuse-audit.md](../../../../../../specs/169-priorite-bridget/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| A1 — « un booléen […] dans les réglages existants » | `ServerSettings` seulement ; champ exclu du patch public. | /Users/moi/11.Repositories/t3code-local/packages/contracts/src/settings.ts:1139 ; /Users/moi/11.Repositories/t3code-local/packages/contracts/src/settings.ts:1476 | Réglage par fichier ; un patch d'un autre champ conserve sa valeur. Aucun second réglage de même rôle. |
| A2 — « un fait de lecture fiable » | Service, chargeur, cache cohérent et writer existants. | /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:256 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:772 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:1250 | Garder réglages/confiance ensemble. Refuser `updateAndPersistSettings` sur cache non fiable avant secrets/écriture ; fichier byte-identique. Ni nouveau service ni sentinelle JSON. |
| A3 — « actualisée avant chaque tour » | Décorateur du démarrage de chaque tour dans le gestionnaire existant. | /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1792 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1823 | La préparation MCP aux lignes356/485 ne suffit pas pour un fil attaché. A4 porte le résultat de cette lecture vers les adaptateurs. |
| A4 — « contexte par tour existant » | `ProviderAdapterV2TurnInput` et appel commun à `runtime.startTurn`. | /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/ProviderAdapter.ts:396 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1823 | Champ optionnel non autorisant, même sans MCP. Vérifier sa conservation vers les drivers legacy ; ni map ni mutation de credential. |
| A5 — « un seul endroit pour composer » | Compositeurs communs d'orchestration. | /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/orchestrationInstructions.ts:3 ; /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/orchestrationInstructions.ts:63 ; /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/orchestrationInstructions.ts:72 | Remplacer les seuls paragraphes concernés ; pas de prompts complets copiés. |
| A6 — « contexte additionnel » Codex | Montage MCP et contexte développeur existants. | /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/Drivers/CodexMcp.ts:29 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/CodexDeveloperInstructions.ts:197 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:772 | Pas de transport ou de montage inventé. |
| A7 — « append SDK et clé de réutilisation » Claude | Montage Claude et option SDK réelle. | /Users/moi/11.Repositories/t3code-local/apps/server/src/provider/Drivers/ClaudeMcp.ts:186 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:1443 ; /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/node_modules/@anthropic-ai/claude-agent-sdk/sdk.d.ts:2257 | Réutiliser `snapshot: false` ; preuve d'exécution requise sans compaction. |
| A8 — « comparaison d'état ACP » | État d'injection et adaptateur existants. | /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/orchestrationInstructions.ts:105 ; /Users/moi/11.Repositories/t3code-local/packages/provider-acp/src/server/adapter.ts:678 ; /Users/moi/11.Repositories/t3code-local/packages/provider-acp/src/server/adapter.ts:6766 | Inclure le mode ; garder les commandes natives. |
| A9 — « deux variantes OpenCode » | Deux adaptateurs actuels. | /Users/moi/11.Repositories/t3code-local/packages/provider-opencode/src/server/adapter.ts:976 ; /Users/moi/11.Repositories/t3code-local/packages/provider-opencode/src/server/v2/adapter.ts:3295 | Même état blocked avec ON, sans nouveau raccord. |
| A10 — « premier contexte […] après passage ON » Cursor | Injection existante du contexte. | /Users/moi/11.Repositories/t3code-local/packages/provider-cursor/src/server/adapter.ts:2128 | Ne pas garder une consigne périmée après le premier tour. |
| A11 — « source existante […] before_agent_start » Pi | Extension, requêtes et canal `ctx.ui.setStatus` actuels. | /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/mcpExtensionSource.ts:32 ; /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/mcpExtensionSource.ts:364 ; /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/adapter.ts:2460 | Le hook statique existe ; A12 ajoute sa commande privée avec reçu fermé corrélé, pas un canal nouveau. |
| A13 — « projection149 » | Service et banc de régression Lineage. | /Users/moi/11.Repositories/t3code-local/apps/server/src/bridget/BridgetLineage.ts:101 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/bridget/BridgetLineage149.test.ts | Aucun nouveau suivi ni changement de rendu. |
| A14 — « circuit T3 existant » | Build desktop et paquet existants. | /Users/moi/11.Repositories/t3code-local/package.json:25 ; /Users/moi/11.Repositories/t3code-local/package.json:40 | Version169, signature et activation contrôlée ; pas de nouveau lanceur persistant. |
| A15 — « prérequis causal de reprise du gros fil » | Handler HTTP `threadSnapshot`, principal `EnvironmentAuthenticatedPrincipal` et banc `withHttp` existants. | /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/http.ts:166 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/boundedSnapshotTransport.test.ts:235 | Pour le sujet authentifié Bridget seul : `visibleTurnItems: []`, champs consommateurs intégraux, autres sujets inchangés. Aucun helper produit, Rust, nouveau endpoint, RPC ou contrat public. |

<a id="extrait-2"></a>

## Extrait 2

| Source | Regle | Impact sur le plan |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/AGENTS.md | Français, chemins absolus et isolation de session. | Documents français dans le worktree169 ; pas d'édition du travail d'autrui. |
| /Users/moi/.speckit/constitution.md | XIX — réutilisation et périmètre. | Un champ de réglage et des compositeurs existants. |
| /Users/moi/.speckit/constitution.md | XVIII et XX — complexité, responsabilité future. | Résolution O(1), frontière d'identité explicite, preuves ciblées. |
| /Users/moi/11.Repositories/t3code-local/AGENTS.md | Contrôles ciblés ; contrôles globaux interdits. | Pas de FULL ni de boucle de lint/types de tout le dépôt. |
| /Users/moi/11.Repositories/t3code-local/AGENTS.md | État réel protégé et pas de plan interne committé dans T3. | Bases conservées ; artefacts SpecKit dans Bridget. |
| /Users/moi/.speckit/ref/speckit-workflow.md | Statuts et champ Tests canoniques. | Draft tant que les gates de conception ne sont pas closes ; aucun succès inventé. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/AGENTS.md | Auteur humain seul et services protégés. | Aucun trailer interdit ; opérations de livraison par ROOT avec identités réelles. |
| Instruction utilisateur prioritaire du 15 octobre 2026, relayée par ROOT | Installation et activation déjà autorisées. | Retirer l'ancienne exclusion de relance ; retour arrière prêt, aucun `launchctl submit`. |

<a id="extrait-3"></a>

## Extrait 3

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/149-sous-agents-lineage/spec.md | Enfants natifs, projection unique, droits hérités. | Réutilisation, aucune nouvelle conversation en doublon. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/163-decision-admission/spec.md | Contraintes, refus et décision avant admission. | Aucune autorité de choix déplacée vers T3. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/164-conversation-t3-distante/spec.md | Lecture de journal et de résultat native. | Présentation inchangée ; régression existante. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/166-rattachement-bridget-reprise/spec.md | Attestation courante par fil, reprise et absence de faux secours. | Préférence sans rotation d'identité ; incident de relance pris en compte. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/168-routage-flotte/spec.md | Automatique, contraintes, reçus et non-duplication. | Réutiliser l'outil automatique et son schéma, sans nouveau routage. |

<a id="extrait-4"></a>

## Extrait 4

| Requete | Portee | Resultat |
|---|---|---|
| Deux explorations coordonnées transmises par ROOT, base T3 `76780a20`. | Réglages, montage, adaptateurs, consignes et suivi existants. | Passage large réutilisé ; aucun nouveau passage large lancé. Références complètes dans A1–A14. |
| Recherche des titres, exigences et plans des specs149/163/164/166/168. | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs | Cinq socles applicables lus et retenus. |
| Recherche ciblée `settingsFileTrusted` et lecture du chargeur. | /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:772 | Fait existant ; un défaut après décodage invalide ne prouve pas OFF. |
| Lecture de la comparaison des propriétaires de permissions. | /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/mcpSession.ts:153 | Remplacer l'objet efface les faits ; identité à préserver. |
| Constat ROOT du contexte commun et de sa transmission à tous les tours. | /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/ProviderAdapter.ts:396 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1823 | Porteur réutilisable trouvé ; les sorties MCP anticipées aux lignes471/1311 ne doivent pas conditionner le champ. |
| Recherche `t3-bridget-policy`, `lastAcknowledgedDelegationPolicy`, `registerCommand` et responsabilité de mise à jour. | /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/mcpExtensionSource.ts ; /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/adapter.ts | Aucun équivalent exact trouvé sur la source de référence ; hook et flags existants seulement. |
| Contrôle des requêtes Pi et de leur catalogue, constat ROOT. | /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/rpc.ts:95 ; /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/rpc.ts:305 ; /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/commands.ts:46 ; /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/mcpInjection.ts:36 | Réutiliser commande et statut d'extension. L'erreur avalée par le SDK impose reçu corrélé après affectation + ACK, jamais ACK seul. Sources primaires tracées dans la recherche. |
| Diagnostic empirique et recherche précise du handler/banc, vérifiés ROOT. | /Users/moi/.cache/bridget-delivery169/diagnostic-http-limit169.md ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/http.ts:166 ; /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/boundedSnapshotTransport.test.ts:235 | Réponse33 868 206octets >16Mio ; simulation sans le champ redondant :6 372 704octets. Banc HTTP réel réutilisable, aucun nouvel oracle ou helper nécessaire. |

<a id="extrait-5"></a>

## Extrait 5

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| A12 — `/t3-bridget-policy` | CRÉER une commande interne dans l'extension Pi existante, accord explicite ROOT. | Aucun équivalent. Enum fermé et nonce court validé ; handler synchrone, découverte `get_commands`, filtre utilisateur. Après affectation : reçu fermé du mode par `ctx.ui.setStatus`, corrélé à la tentative, intercepté/masqué, ET ACK corrélé avant premier vrai prompt même OFF. Ancien reçu ou ACK seul ne suffit pas. Aucun RPC Bridget/public, endpoint, service ou fichier. | 2026-10-15 |
| Append Claude figé par le SDK | RÉUTILISER `snapshot: false` et la clé de contexte existante. | La clé seule ne prouve pas l'actualisation après reprise. La preuve reste à exécuter. | 2026-10-15 |
| A4 — mode sans MCP | RÉUTILISER le contexte par tour existant, accord ROOT après constat worker. | `undefined` ne fait pas perdre ON. Le champ est indépendant de MCP ; aucune map, mutation de session ou identité factice. | 2026-10-15 |
| A15 — réponse complète pour Bridget | RÉUTILISER le handler et le banc HTTP existants, correction causale retenue par ROOT. | La reprise autorisée échoue au plafond. Vider seulement le champ de présentation redondant pour le sujet authentifié Bridget ; ne pas relever le plafond ni couper les champs consommateurs. | 2026-10-15 |
