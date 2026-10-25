# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/adversarial-review-claude.md](../../../../../../specs/169-priorite-bridget/adversarial-review-claude.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Résultat reçu par ROOT : **APPROVE_WITH_CHANGES**. Relecteur : Claude Sonnet5.5. Source relue : T3, base `76780a20`. Aucun test n'a été exécuté par le relecteur. Il a aussi écrit un plan privé hors des sources ; ce document n'est pas une preuve d'implémentation. Les constats et arbitrages ci-dessous ont été vérifiés ou fixés par ROOT, puis intégrés aux artefacts169.

<a id="extrait-2"></a>

## Extrait 2

| Sujet | Constat et preuve | Décision intégrée |
|---|---|---|
| B1 — Lecture avant chaque tour | La préparation MCP peut s'arrêter tôt et ne se refait pas pour un fil attaché : /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:471 et /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1311. | Décorer chaque `runtime.startTurn` aux points1792/1823 avec le contexte par tour existant. |
| B2 — Confiance perdue dans les defaults/cache | Le chargeur possède un fait de décodage distinct des valeurs : /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:772. | Cache cohérent réglages/confiance. Erreur => `blocked`, même après OFF ; aucun faux OFF ni sentinelle JSON. |
| B3 — Écrasement depuis les defaults | Le writer peut repartir des defaults : /Users/moi/11.Repositories/t3code-local/apps/server/src/serverSettings.ts:1250, comportement vérifié ROOT jusqu'à la ligne1279. | Refuser `updateAndPersistSettings` sur cache non fiable, avant secrets/écriture. Fichier byte-identique ; retour de confiance par réparation valide relue ou écriture depuis un état fiable. |
| Consignes contradictoires | Les parcours existants comportent six injonctions concernées, dont revue, scheduler et repli ACP. | Composition conditionnelle commune ; aucune priorité Bridget ajoutée après une injonction T3 contraire. Garder l'exception T3 demandée et les outils hors délégation. |
| Découverte MCP paresseuse | Montage ou ancien catalogue ne garantit pas l'outil ni l'authentification future. | Une découverte d'outil existante et bornée avant de conclure à l'absence ; aucune sonde HTTP de santé par tour. |
| Oracle de recette ambigu | `delegate_task` appartient à T3 ; les fournisseurs possèdent aussi leurs outils natifs de sous-agent. | SC003 sépare ces deux compteurs de l'appel automatique Bridget. Les deux premiers doivent valoir zéro dans la recette concernée. |
| Claude garde l'append | Option réelle du SDK : /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/node_modules/@anthropic-ai/claude-agent-sdk/sdk.d.ts:2257. | `snapshot: false` et clé cohérente ; preuve après reprise sans compaction. Garder le refus en cas de travail de fond : attente explicite, aucun travail tué ou mode périmé. Coût cache/thinking non chiffré, preuve d'options distincte d'un vrai modèle. |
| Pi réinjecte un texte statique | Hook existant : /Users/moi/11.Repositories/t3code-local/packages/provider-pi/src/server/mcpExtensionSource.ts:364. | Extension initialement bloquée/inconnue ; commande privée fermée avant premier vrai prompt même OFF, découverte dans ce processus et cachée au catalogue utilisateur. |
| Pi ACK trompeur | Le SDK attend le handler, mais avale ses erreurs ; le RPC peut répondre success. Sources primaires relues dans la recherche169. | Après affectation, reçu privé fermé via `ctx.ui.setStatus`, mode et nonce de tentative validés, intercepté/masqué. Reçu courant corrélé ET ACK requis. Ancien reçu, erreur ou absence => aucun vrai prompt ni avancement. |

<a id="extrait-3"></a>

## Extrait 3

| Proposition | Décision et raison |
|---|---|
| Préférence seulement dans la configuration MCP, mutation ou map latérale. | Rejetée. Le contexte existant /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/ProviderAdapter.ts:396 porte déjà tous les tours. Il fonctionne sans MCP et évite une identité factice ou un registre supplémentaire. |
| Ajouter le booléen à `ServerSettingsPatch`. | Rejeté par minimalisme : le besoin porte sur un réglage par fichier. Le champ reste lu et conservé lors des autres patches. |
| Croire `success: true` comme preuve d'application Pi. | Rejeté après lecture du SDK officiel. Le reçu corrélé complète le même canal existant sans endpoint nouveau. |
| Bloquer techniquement tous les outils natifs T3. | Hors périmètre. Les consignes restent des consignes ; les gardes Bridget portent les droits et l'admission. |

<a id="extrait-4"></a>

## Extrait 4

| Objection | Qualification ROOT | Décision et preuve attendue |
|---|---|---|
| O1 — OFF Codex sans MCP | Retenue. Le contexte OFF était omis ; l'absence de clé ne prouve pas le reset d'un ancien texte ON/blocked. | T013 :reset explicite dans les injections existantes, oracle ON/blocked→OFF sans MCP. Références du gel : /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:6356 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/CodexDeveloperInstructions.ts. |
| O1 — Variante après reprise | Retenue sur ACP et Cursor :un cache vide ou inconnu ne prouve pas l'absence d'un ancien texte dans le contexte repris. OpenCode1/2 réémettent leur système complet ; ils ne sont pas affectés. | T013 :reset après reprise, slash natives conservées, cache avancé après envoi réussi. Références du gel : /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-acp/src/server/adapter.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-cursor/src/server/adapter.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-core/src/server/orchestrationInstructions.ts. |
| O3 — Credential Claude pris pour un montage | Retenue après lecture ROOT :la préparation peut donner `disabled`, `explicit_config`, `inherited_identity`, `metadata_unavailable` ou `foreign_server`. La présence du credential ne vaut pas `mounted`. Aucun gain de droit n'est établi, mais la consigne est inexacte. | T014 :projeter le fait réel avant le premier offer au SDK, dans le cycle Query/préparation existant ; zéro offre `prefer` si non monté. Garder le travail de fond et son refus. Références du gel : /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:7968 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/Drivers/ClaudeMcp.ts. |
| O2 — Normalisation du chemin Pi | Hypothèse conservée, correction realpath rejetée sans preuve réelle. Pi n'est pas dans le PATH ; aucune dérive de chemin n'est observée. | Garder la garde fermée et la réserve Pi réel. Ne pas ajouter de tâche, dépendance, installation ou correctif spéculatif. |
| TS377033 — Fixture | LOW conservé, non causal. ROOT confirme une seule instance sous portée gérée ; pas de rupture de cycle de vie établie. | Aucune correction produit requise. Conserver2 warnings de types contre1 sur la base, séparément des75 erreurs historiques et des12 warnings lint identiques. Lieu : /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/ProviderSessionManager.test.ts:4712. |

<a id="extrait-5"></a>

## Extrait 5

| Objection | Réponse vérifiée ROOT | Décision |
|---|---|---|
| C1 — Query.return supposé sans borne | SDK0.3.276 lu et SHA confirmé. Pas de sessionStore T3 donc pas de flush miroir actif ; lecteur libéré et attente de sortie en course avec2 s. close réutilise exactement cleanupPromise. Preuve /Users/moi/.cache/bridget-delivery169/review169-sdk-c1.md. | Non confirmé. Pas de timeout+close qui ne force rien et pourrait rouvrir trop tôt ; pas de faux test return infini. Sortie physique absolue d'un processus figé non prouvée. |
| C2 — OFF sans MCP ajoute une enveloppe | runOrdinal1 ne prouve pas une conversation native neuve après import/reprise. L'état inconnu peut porter ON ; le reset conservateur reste utile. Codex reçoit t3_code_orchestration et t3_code_runtime. | Comportement conservé, politique habituelle mais enveloppe sans MCP différente. Seuls6 986 octets de la constante OFF avec MCP sont byte-identiques. Pas de branche/cache supplémentaire ni de passage3. |
| Coût et87 transcriptions | Codex déduplique le contexte identique ; coût de contexte/prompt restant non mesuré. |87 transcriptions adaptées, pas87 bugs produit ; réserve explicite. |
| Query blocked en cache | Elle peut rester blocked jusqu'à réouverture ou changement de clé/processus, même si le montage redevient possible. | Limite fermée conservée, sans rafraîchissement automatique promis. |
| Reprise et message UI | Reprise Claude existante positive et visibilité UI du message de divergence non prouvées. Qualifications natives sur les deux paires installées, sorties normales seulement. | Aucun crash figé ou résultat de modèle déduit. |
| Pi, types et fixture | Pi réel absent ;75 erreurs TS historiques identiques, serveur non GREEN ; TS377033 LOW non causal conservé. | Réserves maintenues, aucun gate réel fermé. |

<a id="extrait-6"></a>

## Extrait 6

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.
