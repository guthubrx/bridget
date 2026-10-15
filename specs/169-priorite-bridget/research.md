# Recherche SPEC169 — Décisions bornées au raccord existant

Date : 2026-10-15. Portée : choix de conception et lecture de qualifications bornées déjà réalisées. Le responsable documentaire n'exécute aucun scénario.

## Sources et limites

Baselines lues : /Users/moi/.speckit/research/01-ai-agents-agentic-ai.md et /Users/moi/.speckit/research/04-architectures-patterns.md. Elles orientent vers les sources primaires et la vérification des limites. Aucun chiffre de marché ni gain de productivité n'est repris pour justifier ce lot.

Le blog officiel MCP précise que l'hôte décide comment il injecte les instructions. Il recommande de vérifier le client réel. Il indique aussi qu'une instruction textuelle ne garantit pas un comportement et ne remplace pas les règles de sécurité déterministes. Cette session limite donc sa promesse aux consignes injectées et aux parcours vérifiés. [Source MCP, publiée le 3 novembre 2025](https://blog.modelcontextprotocol.io/posts/2025-11-03-using-server-instructions/), consultée le 15 octobre 2026.

Anthropic recommande de commencer par une solution simple et d'ajouter de la complexité seulement si le besoin l'exige. L'article date de décembre2024 et signale que son paysage d'outils a évolué. Nous retenons ce principe, sans en déduire une compatibilité actuelle de SDK ou l'adoption d'un framework. [Source Anthropic](https://www.anthropic.com/engineering/building-effective-agents), consultée le 15 octobre 2026.

Les faits T3 viennent des deux explorations coordonnées, complétées par une lecture ciblée du chargeur de réglages et du registre MCP. Les repères complets sont conservés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/reuse-audit.md. La présence de code n'atteste ni un montage actif ni une authentification future.

## Décisions

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

## Inconnues traitées et limites conservées

Les questions de périmètre sont arbitrées : absence OFF, ON explicite, préférence non autorisante, mise à jour avant chaque tour et exception T3 demandée. Le portage sans MCP est résolu par le contexte existant : /Users/moi/11.Repositories/t3code-local/packages/provider-core/src/server/ProviderAdapter.ts:396 et /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/ProviderSessionManager.ts:1823. Aucun registre, map, faux credential ou mutation MCP n'est retenu. Le passage dans la compatibilité legacy fait partie des preuves d'implémentation.

L'option Claude est attestée dans /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/node_modules/@anthropic-ai/claude-agent-sdk/sdk.d.ts:2257. Le comportement documenté se poursuit jusqu'à la ligne2284. L'implémentation doit encore prouver l'append réel, le reçu Pi corrélé et leur parcours. Les tests ciblés et la recette activée restent des gates distinctes. Aucun gain chiffré ni obéissance absolue du modèle n'est promis.

Le SDK Pi attend le handler d'extension, mais convertit aussi son exception en commande traitée. Son mode RPC peut alors répondre success. Un ACK seul ne prouve pas le changement de mode. Ce constat justifie le reçu privé réemployant `ctx.ui.setStatus`, émis après affectation et corrélé à la tentative. [Source de session Pi](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/src/core/agent-session.ts), lignes1946–1950 et2027–2037 ; [source RPC Pi](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/src/modes/rpc/rpc-mode.ts), lignes368–387. Sources consultées le 15 octobre2026 ; leur branche principale reste mobile, à rapprocher du SDK réellement installé dans la preuve d'implémentation.

Le binaire Pi est hors PATH selon le constat transmis par ROOT. Il n'y a donc aucune recette Pi réelle attestée. Les tests de contrat/fixtures restent prévus ; l'installation d'un fournisseur supplémentaire ne fait pas partie du lot.

## Complément causal issu d'Analyze

ROOT a observé `champ response limit` sur ce long fil : réponse complète33 868 206 octets pour un plafond16Mio, dont environ27,5Mo de `visibleTurnItems` redondants non consommés par Bridget. La simulation qui vide ce seul champ conserve6 372 704 octets. Ce constat justifie T012 pour FR017/SC009 ; il ne prouve pas un correctif exécuté.

Sources vérifiées ROOT : /Users/moi/.cache/bridget-delivery169/diagnostic-http-limit169.md, SHA-256 `25df7f44115ddcb43cb38e89a1ba49214bf1eaafc3292b2c5d3309506334b434` ; handler /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/http.ts:166 ; banc réel /Users/moi/11.Repositories/t3code-local/apps/server/src/orchestration-v2/boundedSnapshotTransport.test.ts:235. Aucun Rust, RPC, endpoint ou contrat public supplémentaire n'est requis.

## Qualification Cartae T014 — Identité après initialisation sans prompt

Preuve existante lue par SSH, par lecture du seul JSON : /home/moi/.local/state/bridget-delivery169/qualification-t014/result.json. SHA-256 des octets reçus : `c0747f06ddfbb910c0b28cbeee636b43fc3e47179b475b1cde00ee82e45b41c0`. Le reçu identifie le SDK Claude0.3.276 et la CLI réellement installée2.1.251, au chemin /home/moi/.local/share/claude/versions/2.1.251.

Le cycle qualifié par ROOT est :nouvelle session avec `sessionId`, attente de `Query.return()`, puis seconde initialisation avec le même `sessionId`. Le JSON montre les deux initialisations réussies, chacune fermée avec sortie0. Il ne contient que des frames de contrôle `initialize` ; `userFrames` vaut0. Aucune frame utilisateur ou modèle et aucune conversation JSONL ne sont observées. Des fichiers de configuration et d'état de processus temporaires sont listés ; leur présence n'est pas une conversation persistée.

Dans le scénario témoin, la première initialisation avec `sessionId` réussit. La seconde avec `resume` sur cet ID sans conversation échoue :`No conversation found`, sortie1. ROOT retient donc l'identité d'entrée inchangée pour la réinitialisation préparatoire de T014 :une entrée neuve reste `sessionId` ; une reprise existante conserve `resume`/`resumeSessionAt`. Cette qualification ne constitue pas un test positif de reprise d'une conversation existante.

Ce constat Cartae est limité à la paire SDK0.3.276/CLI2.1.251 et à une initialisation sans prompt. `allTrackedChildrenStopped` vauttrue. ROOT atteste qu'aucun endpoint, authentification ou session de production n'a été touché. Le responsable documentaire n'a relancé ni CLI, ni qualification, ni service. Cette preuve seule ne valide ni le montage Claude réel du correctif, ni sa première offre de prompt, ni T014 dans son ensemble ; l'acceptation R2 est distincte ci-dessous.

## Complément Mac T014 — Seconde paire réellement qualifiée

Reçu local lu et empreinte vérifiée : /Users/moi/.cache/bridget-delivery169/qualification-t014-mac-2.1.300/result.json, SHA-256 `6f5d22cd4bc72b4d926193e68cf463b12c54cccc037127464011c18d4dbcd009`. Le SDK reste0.3.276 ; la CLI réelle est2.1.300, installée dans /Users/moi/.local/share/claude/versions/2.1.300. ROOT rapporte un RC0 de la qualification ; le JSON mesure1 782 ms et0 frame utilisateur.

Après attente de `Query.return()`, deux initialisations neuves avec le même `sessionId` réussissent, sans conversation JSONL. Le témoin `resume` sur un ID sans conversation est refusé avec `No conversation found` et sortie1 attendue. Les quatre PID terminent naturellement, sans signal ni erreur de fermeture ; `allTrackedChildrenStopped` vauttrue. Seules des frames de contrôle sont observées, sans frame utilisateur ou modèle.

ROOT précise le lancement par /usr/bin/env avec l'option `-i`, sans identifiants d'authentification, avec HOME, configuration et répertoire de travail privés. Aucun modèle ou service de production n'est sollicité. Cette lecture n'exécute aucun scénario supplémentaire.

La preuve couvre maintenant les deux paires présentes :SDK0.3.276/CLI2.1.251 sur Cartae et SDK0.3.276/CLI2.1.300 sur Mac. Elle n'est pas généralisée aux autres versions ni à une reprise existante positive. La décision d'identité reste inchangée :entrée neuve avec `sessionId`, reprise existante avec `resume`/`resumeSessionAt`. Aucun PASS du correctif, montage réel ou première offre de prompt n'est déduit de ces qualifications sans prompt.

## Corrections R2 acceptées par ROOT

ROOT accepte T013/T014 après lecture des9 deltas et vérification des preuves R2. Reçu : /Users/moi/.cache/bridget-delivery169/source-snapshot-r2/developpement169.md. Le reset OFF utilise le compositeur existant pour ACP/Cursor et le contexte Codex par tour, même après reprise à cache vide. Le runner Claude prépare le montage réel avant publication de la candidate ; si nécessaire, il attend le `Query.return()` physique avant une unique candidate blocked, avec file et identité d'entrée propres. Le SDK lu distingue ce retour attendu d'un `close` synchrone.

Les536 cas affectés R2 sont finalement PASS après reprises ; le vrai runner et le vrai préparateur sont exercés derrière une frontière SDK simulée. Ces cas ne sont pas ajoutés aux bilans R1 recouvrants. Les qualifications natives précédentes restent bornées à leurs versions et à une initialisation sans prompt. ROOT a ensuite clos le passage2 CONVERGED avec0 tâche ajoutée ; contre-revue R2 reçue et arbitrée ci-dessous ; audit source clos et jalons réels encore à compléter. Aucune obéissance ou réponse de modèle n'est déduite de l'acceptation de l'implémentation.

## Contre-revue R2 — Lecture SDK et exception OFF

ROOT ne confirme pas C1 sur le SDK installé0.3.276, SHA b1607967e0dfb39a0db45f143d3b57c7a85f2f6eaa77d6bee7d8184a9830a9f3. Preuve /Users/moi/.cache/bridget-delivery169/review169-sdk-c1.md : sessionStore absent de T3, flush miroir inactif, lecteur libéré et course d'attente de sortie de2 s. close réemploie le même nettoyage ; le timeout+close ne force rien et pourrait rouvrir trop tôt. Aucun changement ni faux test return infini retenu. La sortie physique d'un processus figé n'est pas prouvée.

C2 reste documenté : politique absence/OFF habituelle, enveloppe sans MCP différente même au premier tour inconnu. Ordinal1 n'exclut pas import/reprise. Seule la constante OFF avec MCP est prouvée byte-identique,6 986 octets : /Users/moi/.cache/bridget-delivery169/source-snapshot-r2/preuves/R2/historical-OFF-check.json. Contexte Codex identique dédupliqué, coût restant non mesuré et87 transcriptions adaptées sans87 défauts produit. Pas de branche/cache supplémentaire.

Cache blocked jusqu'à réouverture/changement de clé, visibilité UI du message de divergence non prouvée, reprise existante positive non qualifiée, Pi absent et serveur types non GREEN restent des limites. Les qualifications SDK/CLI couvrent seulement les sorties normales des deux paires installées. P2 et J1 restent clos ; J2 est partiel et les gates réels restent ouverts.

## Checkpoint documentaire après clôture J1

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.
