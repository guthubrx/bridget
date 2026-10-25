# Archives techniques de l’adaptateur T3

Document d’origine : [specs/169-priorite-bridget/validation/converge169.md](../../../../../../../specs/169-priorite-bridget/validation/converge169.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Tâches avant et après la phase ROOT : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/tasks.md. SHA-256 identique : `234d6596b7451839ec7e701f2bb33d7455e41aa8afdee82a187eeb7f7b599535`. Aucune édition des tâches ou du code n'a été faite pour ce reçu.
- Matrice de preuves lue par ROOT : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/tests169.md. Son SHA au moment de la décision est `6e18347f34b24e94181e74a505db3ce07b72dae1642e3a411a4b48cb6563c206`. Après clôture, seule une note attribuée à ROOT y complète le contrôle des archives ; elle ne change aucun résultat ni réserve.
- Reçu owner : /Users/moi/.cache/bridget-delivery169/source-snapshot/developpement169.md, SHA-256 `5615751c6e7f9b3bb958ec5db0894dc851cbf14174175586df007173e70563ec`. Source distante : /home/moi/.local/state/bridget-delivery169/developpement169.md.
- Source de travail : /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget, base `76780a20bb22b9390eab2389eaf0db87e1349e7e`. Copie Mac des35 fichiers attestée identique : /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget.
- Preuves téléchargées et archivées par ROOT : /Users/moi/.cache/bridget-delivery169/source-snapshot/preuves. ROOT atteste127 membres et126 SHA contrôlés ; SHA-256 de l'archive `eb1afc827ebc7a8985fd57f6d5ced241f4b2edc2228d0808f46567bfab0f921a`. La différence d'index Git de9 à10 chiffres du patch Mac ne change aucun des35 SHA sources.

<a id="extrait-2"></a>

## Extrait 2

| Repère | Produit lu par ROOT | Test ou preuve existante |
|---|---|---|
| R01 — Schéma | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/contracts/src/settings.ts:1176 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/contracts/src/settings.test.ts |
| R02 — Réglages fiables | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/serverSettings.ts:890 ; lignes1274 et1428 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/serverSettings.test.ts:1908 ; lignes1923,1980 et1994 |
| R03 — Contexte par tour | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-core/src/server/ProviderAdapter.ts:396 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/ProviderSessionManager.ts:394 ; ligne1834 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/ProviderSessionManager.test.ts:4611 |
| R04 — Consignes communes | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-core/src/server/orchestrationInstructions.ts:47 ; lignes61,67,132,171 et190 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-core/src/server/orchestrationInstructions.test.ts:89 |
| R05 — Codex | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:6341 ; cache ligne1844 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/CodexDeveloperInstructions.ts | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/CodexDeveloperInstructions.test.ts |
| R06 — Claude | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:1447 ; lignes7888 et7962 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.test.ts:160 ; ligne2448 |
| R07 — ACP | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-acp/src/server/adapter.ts:6771 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/AcpAdapterV2.test.ts:766 |
| R08 — OpenCode1/2 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-opencode/src/server/adapter.ts:3260 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-opencode/src/server/v2/adapter.ts:3294 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-opencode/src/server/adapter.test.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/OpenCode2AdapterV2.test.ts |
| R09 — Cursor | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-cursor/src/server/adapter.ts:2104 ; ligne2200 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-cursor/src/server/adapter.test.ts:39 |
| R10 — Pi | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-pi/src/server/adapter.ts:624 ; lignes1268,1676,2405 et2551 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-pi/src/server/mcpExtensionSource.ts:279 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-pi/src/server/commands.ts | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-pi/src/server/adapter.test.ts:3273 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-pi/src/server/mcpExtensionSource.test.ts:402 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-pi/src/server/commands.test.ts |
| R11 — Réponse HTTP | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/http.ts:169 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/boundedSnapshotTransport.test.ts:290 |
| R12 — Suivi149 inchangé | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/bridget/BridgetLineage.ts:101 | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/bridget/BridgetLineage149.test.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-core/src/server/mcpSession149.test.ts |
| R13 — Montages et gardes inchangés | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/Drivers/CodexMcp.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/Drivers/ClaudeMcp.ts | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/Drivers/CodexMcp.test.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/provider/Drivers/ClaudeMcp.test.ts |

<a id="extrait-3"></a>

## Extrait 3

| Exigence | Code lu | Test ou preuve retenue | Limite |
|---|---|---|---|
| FR001 | R01,R02 :booléen fichier, défautfalse, exclu du patch public. | R01,R02 ; T001,T005. | Aucun réglage visuel ajouté ; environnement isolé. |
| FR002 | R01,R02 :absence/OFF fiables, ON explicite distinct. | R01,R02 ; T001,T005. | ON du processus cible reste à constater enJ4–J5. |
| FR003 | R02 :cache réglages/confiance, refus avant secret/write si non fiable. | R02 :JSON/I/O, ancienOFF, fichier byte-identique et réparation ; T001,T005. | Intégration fichier isolée ; aucun OFF silencieux après erreur. |
| FR004 | R03–R10 :préférence portée par input à chaque tour, y compris sans MCP/attachement réutilisé. | R03 :cinq tours fichier/watch et builder ; matrices R04–R10 ; T002–T005,T007–T008. | Aucun tour en cours réécrit ; aucune obéissance de modèle réel déduite. |
| FR005 | R04,R07–R10 :exception de fil T3, outils hors délégation, slash natives. | R04,R09,R10 ; T003,T006–T008 ; nom Pi réservé refusé au chemin utilisateur. | Le canal privé Pi est fermé ; autres slash conservées en fixtures. |
| FR006 | R04 :contraintes explicites gardées, axes restants automatiques. | R04 ; T003,T006 ; contrat168 réutilisé. | Texte/transport ; aucun nouveau choix contraint par modèle réel169. |
| FR007 | R03–R10 :aucun second moteur ni algorithme de flotte. | Lecture ROOT des35 deltas ; T003,T006. | Le choix et l'admission Bridget restent hérités ; pas une nouvelle exécution native. |
| FR008 | R03,R05,R06,R13 :identité stable, gardes et opt-outs conservés. | R03,R13 ; T002,T004–T006 ; montages/révocations réutilisés. | La préférence non autorisante ne certifie pas la santé future du daemon. |
| FR009 | R04–R10 :blocked conservé sans MCP, erreur explicite et aucun nouveau secours d'hôte. | Matrices R04–R10 et lecture du delta ; T003–T004,T007–T009. | Consigne et absence de dispatch hôte prouvées ; pas de verrou natif ni d'obéissance absolue. |
| FR010 | R04,R12 ; mécanismes149/163/168 inchangés. | Références de suivi/rejeu conservées, lecture ROOT sans nouveau dispatch ; T009–T010. | Un mock T3 n'a pas exécuté d'admission réelle Bridget ; réemploi décrit dans la matrice de preuves. |
| FR011 | R03,R12 :aucune migration ou relance ajoutée ; task_status/task_cancel T3 conservés. | Transitions R03 et régressions R12 ; T005,T010. | Reprise de la cible et suivi des moteurs initiaux réservés àJ5. |
| FR012 | R12 :projection Lineage inchangée. | Banc Lineage149,23 PASS au registre ; T010. | Régression héritée ; aucune nouvelle preuve visuelle nécessaire pour ce delta technique. |
| FR013 | R12,R13 ; moteur Bridget autonome inchangé. | Régression149 et inspection du périmètre ; T006,T010. | Aucun parcours standalone réel169 refait. |
| FR014 | R03–R10 :raccords Codex/Claude et états fidèles ACP/OpenCode1/2/Cursor/Pi. | Matrices de chaque raccord ; T002–T004,T007–T008. | Sans montage Bridget, ON est blocked. Pi réel absent du PATH ; contrat/fixtures seuls. |
| FR015 | R01–R04,R10 :source fichier unique, input existant, commande privée bornée. | Contrats, fiabilité et canal Pi ; lecture des35 deltas ; T001–T003,T006,T008,T011. | Aucun second état durable, dépendance, service ou protocole Bridget/public ajouté. |
| FR016 | R03–R11 :préférence/consignes et correction causale du gros fil. | R11 :sujet authentifié Bridget seul, champs utiles et autres sujets intacts ; T012,T011. | HTTP isolé ; simulation33 868 206→6 372 704 octets distincte de la cible. |
| FR017 | R11 prépare la reprise ; procédure de livraison prévue. | T012 et jalonsJ2–J6. | **Ouverte.** Installation, activation, code chargé et reprise réelle non attestés par Converge. |

<a id="extrait-4"></a>

## Extrait 4

| Critère | Code et preuve rapprochés | Limite et état |
|---|---|---|
| SC001 | R01–R10 :absence/OFF/ON/blocked, transmission de chaque raccord ; OFF historique byte-identique au registre. | Couvert en contrat/fixture/transport ; aucun modèle réel déduit. |
| SC002 | R02,R03 :lecture non fiable explicite, refus avant secrets/write, fichier conservé. | Couvert par I/O et cache isolés. |
| SC003 | R04–R06 et T009 préparent l'outil automatique et les comptes séparés Bridget/delegate_task/fournisseur. | **Ouvert enJ5.** Résultat natif corrélé et zéro double lancement exigent la recette réelle. |
| SC004 | R03,R04,R13 :contraintes, gardes et identité stable. | Couvert au niveau texte/montage/fixture ; aucune admission réelle169 revendiquée. |
| SC005 | R04–R10,R13 :absence/refus explicites, aucun nouveau secours d'hôte ; réemploi149/163/168. | Contrat/fixture/inspection ; la limite d'admission réelle de FR010 demeure. |
| SC006 | R02,R03,R12 :OFF→ON→OFF, suivi initial et lectures sans nouveau lancement. | Intégration isolée et régression149 ; reprise de la cible réservée àJ5. |
| SC007 | R04,R07–R10 :fil T3 explicite, outils hors délégation et slash natives. | Contrat/fixtures ; nom privé Pi réservé et refusé au chemin utilisateur. |
| SC008 | R01–R11 et lecture ROOT des35 fichiers :aucune nouvelle UI, politique durable, dépendance, service ou protocole Bridget/public. | Inspection du périmètre ; commande Pi privée dans le canal existant explicitement comptée. |
| SC009 | R11 :handler authentifié et banc HTTP existant ; plan de reprise. | **Ouvert enJ5.** Code chargé, ON, nouveaux contextes, conversations et suivi réels restent à prouver. |

<a id="extrait-5"></a>

## Extrait 5

Le serveur types reste RC1 :75 erreurs TS historiques, delta0. ROOT a comparé les75 blocs complets et les trouve identiques. Les warnings de types passent de1 à2 :nouveau `TS377033 multipleEffectProvide` dans /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/ProviderSessionManager.test.ts:4712. Ce warning de fixture est soumis à la contre-revue. Il ne fait pas partie des12 warnings lint, dont la comparaison avec la base est identique. Aucun succès global du dépôt n'est annoncé.

<a id="extrait-6"></a>

## Extrait 6

| Correction relue par ROOT | Produit et preuve utiles | Limite |
|---|---|---|
| T013 — Reset OFF fidèle | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:774 ; banc /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts:3290. Compositeur /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-core/src/server/orchestrationInstructions.ts:67 ; lignes147 et171. Oracles /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-core/src/server/orchestrationInstructions.test.ts:99 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/AcpAdapterV2.test.ts:766 ; /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/packages/provider-cursor/src/server/adapter.test.ts:99. | Reset explicite sans MCP et après reprise à état inconnu ; slash natives et avancement après envoi conservés. Frontière fournisseur en fixture. |
| T014 — Préparation avant offer | /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:826 pour le retour physique ; lignes1030,1100 et1180 pour layer, file propre et seconde candidate. Clé de query ligne7976, builder8048, runner8075. Banc /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.test.ts:165 ; lignes364,388,418 et2731 pour la réutilisation. | Vrai runner et vrai préparateur derrière SDK simulé ; aucun prompt utilisateur à la candidate retirée, Query.return attendu, travail de fond conservé. Aucune réponse de modèle réelle. |

<a id="extrait-7"></a>

## Extrait 7

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.
