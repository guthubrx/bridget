# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/research.md](../../../../../../specs/169-priorite-bridget/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les faits T3 viennent des deux explorations coordonnées, complétées par une lecture ciblée du chargeur de réglages et du registre MCP. Les repères complets sont conservés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/reuse-audit.md. La présence de code n'atteste ni un montage actif ni une authentification future.

<a id="extrait-2"></a>

## Extrait 2

| Sujet | Décision et raison | Alternative examinée | Effet pour le mainteneur |
|---|---|---|---|
| Configuration | Un champ `preferBridgetDelegation` dans `ServerSettings` seulement, exclu du patch public. Défaut `false`, absence OFF, ON explicite par fichier. | Second fichier, politique par projet ou mutation publique supplémentaire. | Un seul fichier à comprendre et à modifier. |
| Lecture invalide | Garder un cache réglages/confiance et exposer `settingsFileTrusted`. Lecture non fiable : `blocked`, même après OFF. Refuser `updateAndPersistSettings` avant secrets/écriture ; fichier byte-identique. | Déduire OFF des defaults, sentinelle JSON ou écrasement du fichier invalide. | L'échec reste distinct du choix humain et ne détruit pas son réglage. |
| Mode effectif | Calcul O(1) : fiabilité, préférence puis montage réel. Le mode a trois valeurs fermées, `off`, `prefer`, `blocked`. | Nouveau service de politiques ou sonde réseau par tour. | La règle tient dans une petite table vérifiable. |
| Identité et transport | Réutiliser le contexte existant `ProviderAdapterV2TurnInput`, enrichi d'un champ optionnel non autorisant. Le manager le renseigne avant chaque `runtime.startTurn`, y compris sans MCP. | Mutation/reconstruction de configuration MCP, faux credential ou map éphémère. | Protège les identités et évite un ON perdu quand MCP est désactivé, sans registre supplémentaire. |
| Consignes | Composer les paragraphes de délégation dans le module partagé ; conserver les autres consignes T3. | Ajouter ON après les consignes natives contradictoires ou dupliquer le prompt par fournisseur. | Un seul texte métier à relire, avec des preuves de non-contradiction. |
| Claude | Employer `snapshot: false` pour l'append dynamique et la clé de contexte. Respecter le travail de fond : attente explicite, aucun tour au mode périmé. Coût cache/thinking non chiffré, à distinguer d'une preuve LLM réelle. | Modifier seulement `queryPolicyKey` ou tuer une query active. | La mise à jour reste réelle et préserve le travail en cours. |
| Raccords sans montage | ACP, OpenCode, Cursor et Pi annoncent `blocked` avec ON. Pi réutilise commande privée et `ctx.ui.setStatus` : reçu fermé corrélé après affectation, plus ACK, avant le premier vrai prompt même OFF. | Inventer un montage, croire l'ACK seul ou redémarrer Pi par changement. | Le reçu protège de l'erreur avalée par le SDK, sans nouveau canal. |
| Disponibilité | Chercher `bridget_auto_delegate` une seule fois par découverte bornée si le catalogue est paresseux, puis bloquer si absent. Aucun HTTP de santé ajouté ; un refus reste un refus. | Assimiler un montage ou un ancien catalogue à une disponibilité garantie. | Pas de boucle réseau ni de promesse fausse. |
| Suivi et livraison | Lineage149 inchangé ; recette et activation contrôlée avec retour arrière préparé. Aucun mécanisme de relance persistant. | Nouveau suivi ou installation seule présentée comme activation. | Les preuves distinguent le code livré du code chargé. |
| Reprise du gros fil | Dans le handler HTTP existant, le principal authentifié Bridget reçoit `visibleTurnItems: []` ; champs consommateurs intégraux et autres sujets inchangés. Réutiliser le banc `withHttp` existant. | Augmenter le plafond Rust, tronquer les messages utiles ou ajouter un endpoint. | Corrige le prérequis causal de reprise sans nouveau contrat ni helper produit. |

<a id="extrait-3"></a>

## Extrait 3

Les questions de périmètre sont arbitrées : absence OFF, ON explicite, préférence non autorisante, mise à jour avant chaque tour et exception T3 demandée. Le portage sans MCP est résolu par le contexte existant : /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/ProviderAdapter.ts:396 et /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1823. Aucun registre, map, faux credential ou mutation MCP n'est retenu. Le passage dans la compatibilité legacy fait partie des preuves d'implémentation.

<a id="extrait-4"></a>

## Extrait 4

L'option Claude est attestée dans /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/node_modules/@anthropic-ai/claude-agent-sdk/sdk.d.ts:2257. Le comportement documenté se poursuit jusqu'à la ligne2284. L'implémentation doit encore prouver l'append réel, le reçu Pi corrélé et leur parcours. Les tests ciblés et la recette activée restent des gates distinctes. Aucun gain chiffré ni obéissance absolue du modèle n'est promis.

<a id="extrait-5"></a>

## Extrait 5

Sources vérifiées ROOT : /Users/moi/.cache/bridget-delivery169/diagnostic-http-limit169.md, SHA-256 `25df7f44115ddcb43cb38e89a1ba49214bf1eaafc3292b2c5d3309506334b434` ; handler /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/http.ts:166 ; banc réel /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/boundedSnapshotTransport.test.ts:235. Aucun Rust, RPC, endpoint ou contrat public supplémentaire n'est requis.

<a id="extrait-6"></a>

## Extrait 6

ROOT ne confirme pas C1 sur le SDK installé0.3.276, SHA b1607967e0dfb39a0db45f143d3b57c7a85f2f6eaa77d6bee7d8184a9830a9f3. Preuve /Users/moi/.cache/bridget-delivery169/review169-sdk-c1.md : sessionStore absent de T3, flush miroir inactif, lecteur libéré et course d'attente de sortie de2 s. close réemploie le même nettoyage ; le timeout+close ne force rien et pourrait rouvrir trop tôt. Aucun changement ni faux test return infini retenu. La sortie physique d'un processus figé n'est pas prouvée.

<a id="extrait-7"></a>

## Extrait 7

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.
