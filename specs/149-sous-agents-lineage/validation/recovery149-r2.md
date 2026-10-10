# Scénarios dégradés natifs 149 - T039 - ronde r2 (reprise après F1)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production, aucun modèle.
Rapport r1 archivé : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149-r1.md`. Résultats r1 archivés : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r1/`.

## Verdict : PARTIAL

- **F1 est corrigé** (R9.2.a à R9.2.e : 5/5 PASS) sur le binaire immuable r5 `ec6b18b5d468`.
- Les 23 contrôles qui passaient en r1 passent toujours (R1 à R8, R6, R7, R9.1, R9.1b). R9.2 (FAIL en r1) est remplacé par R9.2.a à R9.2.e (5/5 PASS). 9 contrôles s'ajoutent : R8.4, R9.3, R9.4, R9.4.b à R9.4.e, R9.5, R9.6.
- **Un nouvel écart F2 (moyen)** est apparu en rejouant la reprise avec une racine qui attend un descendant (R9.4.b et R9.4.d en FAIL, cause prouvée par R9.4.c). T039 n'est pas cochable tant que le principal n'a pas tranché F2.

| Total | PASS | FAIL | Couche |
|---|---|---|---|
| 37 contrôles | 35 | 2 (R9.4.b, R9.4.d) | 35 RÉEL + 2 SIMULÉ (diagnostic sur la base privée de la fixture) |

Résultat brut : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/recovery149.json`. Journal : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/recovery149.stdout.txt`. Aucun processus résiduel après nettoyage.

## F1 corrigé : preuve avant / après

Même scénario R9.2 : mission `SLOW_149:9000` en vol, VRAI SIGTERM du daemon (PID individuel, vérifié), relance du daemon dans la même fixture.

| Mesure | r1 (binaire 823e8a5f) | r2 (binaire r5 ec6b18b5) |
|---|---|---|
| Tours portant la mission | 2 | **1** |
| PID fournisseurs distincts | 2 | **1** (celui d'avant l'arrêt) |
| Lancements fournisseur (avant → après) | 12 → 13 | **12 → 12** (inchangé à t0, 7 s, 15 s) |
| État de la tâche | `failed: unreachable` environ 2 s après la relance | `failed: unreachable` **dès le démarrage du daemon**, avant que le parent se reconnecte |
| Notices d'échec au parent | 1 | 1 (`failed : unreachable`) |
| Rejeu de la requête interrompue | non testé | même `task_id`, statut `failed`, aucun lancement |

Le correctif Sol (`prepare_restart`) fait échouer `starting`, `mission_pending` et `working` avant toute reprise. La flotte managée n'a plus d'enfant natif à relancer.

## Écarts et observations

| Id | Gravité | Sujet |
|---|---|---|
| F2 | moyenne | Racine en `waiting_for_children` bloquée après SIGTERM + relance ; résultat retenu jamais remis (détail ci-dessous) |
| O4 | faible | Le fournisseur en vol survit à l'arrêt du daemon (confirmé, mesuré) |
| O1 | faible | Sens exact de G-P-07(b) « lecture et rejeu inchangés » (trois voies observées, proposition de clarification) |
| O6 | info | `queued` non engagé n'a pas pu être exposé en réel après SIGTERM (6/6 tentatives en `starting`) |
| O7 | info | Un fournisseur est lancé pendant l'arrêt (admission coupée) et reste sans mission |

### F2 - racine retenue bloquée après redémarrage (R9.4, R9.4.b, R9.4.c, R9.4.d)

Scénario réel : le parent A délègue une racine. Son enfant répond, puis la racine attend un petit-enfant encore actif (`NESTED_149[SLOW_149:20000]`). On envoie SIGTERM au daemon et on le relance.

Constaté :
1. À la relance, la racine reste `waiting_for_children` avec son résultat déjà capturé. Le petit-enfant en vol devient `failed: unreachable`. Aucun nouvel enfant, aucun nouveau tour (R9.4 PASS, conforme au correctif).
2. Pendant 40 s, la racine ne quitte pas `waiting_for_children`. Le parent ne reçoit ni résultat, ni échec. `bridget_task_status` rend `waiting_for_children` sans résultat (R9.4.b FAIL).
3. Seule l'annulation par le parent la termine. R9.4.e le prouve sur une 2e racine bloquée de la même façon et non mutée : `cancelled`, zéro fournisseur vivant, aucun nouveau tour. Le résultat retenu est alors perdu.

Cause (prouvée par données, pas supposée) :
- La base de la fixture montre l'exécution du petit-enfant restée `running / provider_accepted` alors que la tâche est `failed / unreachable`.
- `descendants_busy` (`crates/bridget-daemon/src/daemon/native_delegation.rs`) compte un descendant comme occupé tant que `recoverable_execution_ids_for_agent` (`crates/bridget-daemon/src/execution_store.rs`) renvoie une exécution `running`.
- Diagnostic (protocole forensic, mutation de la base PRIVÉE de la fixture, R9.4.c SIMULÉ) : en passant cette exécution à `failed`, la racine passe `result_available` avec son résultat intact en moins de 10 s. La cause est donc bien l'exécution laissée `running` par la reprise.
- Deuxième point (R9.4.d FAIL, SIMULÉ) : même débloquée, la racine reste `result_sent = false` et le parent ne reçoit aucune remise pendant 4 s de plus. Je n'ai pas isolé la cause de cette seconde étape (hypothèse non vérifiée : l'envoi se fait au nom de l'enfant, qui n'est plus connecté).

Même cause pour l'exécution de la mission R9.2 : sa tâche est `failed` mais son exécution reste `running` dans la base. Le correctif r5 empêche sa reprise (R9.5 : aucune mission portée par deux tours sur 4 relances), mais l'enregistrement reste incohérent.

Décision demandée au principal (je n'ai modifié aucun code de production) :
- Option A : `prepare_restart` règle aussi l'exécution des tâches qu'il fait échouer (état terminal), et la racine retenue est soit remise au parent, soit déclarée `failed` avec une erreur explicite.
- Option B : accepter la racine « durable » et exiger une notice au parent. Dans ce cas, il faut au moins documenter que seul `cancel` sort de cet état.
Je recommande A : elle garde la phrase « sans dupliquer l'enfant ni relancer la mission » et évite une mission silencieusement bloquée.

### O4 - le fournisseur survit à l'arrêt du daemon (mesuré)

Pendant R9.2, l'arbre de processus avant l'arrêt est : daemon → `bridget managed-wrapper fixture-codex-149` (groupe de processus propre) → fournisseur Python (groupe de processus PROPRE, différent de celui du wrapper). Après le SIGTERM du daemon, le fournisseur de la mission en vol reste vivant avec `ppid 1` pendant 6,9 s environ, jusqu'à la fin naturelle de son tour (`SLOW_149:9000`), puis il disparaît sans avoir consigné de réponse (cause probable : écriture vers un tube fermé, non vérifiée). Il n'a jamais répondu à personne : un seul tour, aucune réponse enregistrée.
- Risque réel : un vrai Codex orphelin pourrait continuer à agir (écrire dans le projet en `full-access`) alors que la tâche est déjà `failed: unreachable`. La durée dépend du tour en cours.
- Limite de ce que j'ai prouvé : sous launchd, la configuration de la tâche peut tuer le groupe du daemon ; je ne l'ai pas vérifiée. La fixture n'utilise pas launchd.
- Le groupe propre du fournisseur est la raison visible pour laquelle l'arrêt du groupe du wrapper ne l'atteint pas.

### O1 - G-P-07(b) : comportement exact observé après retrait du fait (R8.4)

Contrat : `permissions.md` lignes 29 et 40-46 (une preuve d'identité fraîche autorise l'accès sans remplacer le snapshot ; un fait invalide, périmé, contradictoire ou révoqué produit un refus, jamais v1) et G-P-07(b) (« lecture et rejeu inchangés »).

Comportement observé (fait retiré en fin de run, puis session tournée) :

| Voie | Rejeu de `bridget_delegate` | `bridget_task_status` | `bridget_who` |
|---|---|---|---|
| Montage MCP privé de l'ancien credential | `t3_session_unavailable` | `t3_session_unavailable` | `t3_session_unavailable` |
| Credential neuf qui n'a jamais eu de fait (enveloppe v1, règle (a)) | `ok` (tâche `result_available`) | `ok` | `ok` |
| Lecture humaine Lineage native (CLI, gardes 147) | sans objet | `show` : résultat intact `result_available` | sans objet |

Côté T3, `bridget_session` rend le refus nommé `permission_attestation_unavailable`, jamais l'enveloppe v1 (S5.1 de `interop149.md`). Côté Rust, le montage MCP privé échoue alors EN BLOC avec `t3_session_unavailable` : volontaire, fermé par défaut. Aucun retour à v1.

Lecture : le texte « lecture et rejeu inchangés » est plus large que le comportement. Il est vrai pour les voies humaine/native et pour un credential neuf. Il est faux pour le montage MCP de l'ancien credential.

Proposition de clarification normative (pour la revue finale des sources, sans nouvelle demande à l'utilisateur) :
> G-P-07(b) : le montage MCP privé d'un credential dont le fait a été retiré, révoqué ou tourné est fermé en bloc (`t3_session_unavailable`, y compris `bridget_task_status`, `bridget_task_cancel`, `bridget_who` et le rejeu), jamais v1. « Lecture et rejeu inchangés » s'entend de la lecture humaine Lineage native (gardes 147), de l'annulation native, et d'un credential neuf sans fait (v1, identité/status/cancel/lecture 148). Aucun contournement de la production n'est requis.

### O6 - `queued` n'a pas pu être exposé après SIGTERM

J'ai admis une délégation puis envoyé SIGTERM aussitôt, 6 fois (puis 3 fois dans la version finale). La base montre `starting` à chaque fois, jamais `queued`. Le cas `queued` non engagé reste couvert par le test unitaire `native149_apres_redemarrage_le_tick_admet_le_queue` seulement ; il n'est pas prouvé en réel. Autre résultat réel (R9.3) : une admission coupée en `starting` devient `failed: unreachable`, aucune mission envoyée, aucun enfant relancé, une notice d'échec.

### O7 - lancement pendant l'arrêt

Dans R9.3, un fournisseur est lancé pendant l'arrêt du daemon (`startsDuringStop = 1`, 0 prompt). Il ne reçoit jamais de mission et n'est jamais relancé. C'est cohérent avec « aucune mission portée deux fois », mais cela laisse un processus lancé sans tâche vivante.

## Autres vérifications demandées

- **R9.1 / R9.1b** (missions terminées, ACK) : PASS. Aucune remise accusée n'est rejouée. L'ACK conserve les 19 chiffres exacts de `delivery_generation`.
- **R7** : une racine avec petit-enfant actif n'expose ni résultat ni remise ; une seule remise corrélée quand le petit-enfant finit ; le résultat du petit-enfant n'est pas remis au parent.
- **Annulation de deux PID** (R2.2) : racine et descendant `cancelled`, les deux PID réellement terminés.
- **R6 (panne T3 indépendante)** : mission admise terminée sans T3, annulation native servie, montage MCP fermé, rejeu après retour de T3 sans relance.
- **R9.6 (mort du fournisseur natif, daemon vivant)** : SIGTERM sur UN PID vérifié. Le wrapper ne le relance pas : lancements inchangés (20 → 20), 1 seul tour, 0 fournisseur vivant, tâche `failed` avec l'erreur « stdout Codex fermé pendant le tour » et une notice d'échec.
- **R9.5** : sur toute la recette (plusieurs SIGTERM + relances), chaque nonce a exactement un tour fournisseur.

## Résultats détaillés (37 contrôles)

| Id | Résultat | Couche | Contrôle | Observé |
|---|---|---|---|---|
| R1.1 | PASS | REEL | retry : 20 rejeux (10 en vol, 10 après la fin) -> même task_id, même enfant, même message_id ; 1 lancement, 1 tour, 1 remise corrélée | `{"replays": 20, "sameAll": true, "starts": 1, "prompts": 1, "deliveries": 1, "rows": 1}` |
| R1.2 | PASS | REEL | même request_id avec une autre enveloppe -> envelope_mismatch, aucune mutation, aucun lancement | `{"payload": {"text": "envelope_mismatch"}}` |
| R1.3 | PASS | REEL | Lineage montre un seul enfant pour ce request_id (pas de doublon) | `{"tasks": 1}` |
| R7.1 | PASS | REEL | nested actif : le tour de l'enfant est fini mais la racine n'expose ni statut result_available ni résultat tant que le petit-enfant travaille ; aucune remise au parent | `{"rootStatus": "waiting_for_children", "rootResult": null, "nestedState": "working", "deliveriesDuring": 0, "starts": 3}` |
| R7.2 | PASS | REEL | la lignée est durable : parent_task_id et propriétaire racine stables (la racine appartient à A, le petit-enfant à l'enfant) | `{"rootOwnerOfNested": "74786b56", "ownerOfNested": "6f676277"}` |
| R7.3 | PASS | REEL | quand le petit-enfant a fini : la racine devient result_available, UNE seule remise corrélée à A ; le résultat du petit-enfant n'est pas remis à A | `{"root": "result_available", "nested": "result_available", "deliveries": 1, "leaked": false}` |
| R2.1 | PASS | REEL | un autre fil (B) ne peut ni annuler ni lire la tâche de A : task_unavailable, tâche et processus inchangés | `{"cancel": {"text": "task_unavailable"}, "status": {"text": "task_unavailable"}, "providers": 4}` |
| R2.2 | PASS | REEL | cancel de la racine : racine + descendant actif cancelled, PID des deux enfants réellement terminés, zéro fournisseur résiduel, aucune remise de résultat | `{"states": ["cancelled", "cancelled"], "pids": [[17211, false], [17240, false]], "liveProviders": 0}` |
| R2.3 | PASS | REEL | cancel rejoué : idempotent (même état cancelled, aucune erreur, aucun nouveau lancement) | `{"status": "cancelled", "starts": 5}` |
| R4.1 | PASS | REEL | identité inconnue : token forgé, absence totale d'identité, fil sans binding -> refus explicites ; aucune identité voisine empruntée, aucune ligne, aucun lancement | `{"bogus": ["t3_session_unavailable", "t3_session_unavailable"], "noIdentity": ["identity_not_found", "identity_not_found"], "lineage": "binding_unavailable"}` |
| R3.1 | PASS | REEL | parent extérieur : le fil B voit zéro tâche de A (liste, show), compteurs et base inchangés | `{"listBTasks": 0, "show": "task_unavailable"}` |
| R5.1 | PASS | REEL | refus fournisseur : tâche failed avec erreur explicite, 1 seul lancement, 1 seul tour (pas de reprise ni de substitution de modèle/fournisseur), 1 notification d'échec, Lineage failed | `{"state": "failed", "error": "livraison échouée: erreur Codex (code -32000; référence sha256:66bb1dfcdec778b0b928b8b230dfa24ce84519b01e7d8b321935f82f5", "prompts": 1, "notices": 1, "lineageStatus": "failed", "lineageModel": "fixture-model-149"}` |
| R5.2 | PASS | REEL | le rejeu de la requête refusée rend l'échec mémorisé sans nouveau lancement | `{"replayStatus": "failed", "prompts": 1}` |
| R5.3 | PASS | REEL | modèle ou fournisseur absent du registre : refus nommé à l'admission, aucun lancement, aucune substitution | `{"model": "model_unavailable", "agent": "type d'agent inconnu 'agent-inconnu' dans /Users/moi/.cache/bridget149-native-interop.s8Jk1F/state/agents.json. Types disponibles : claude, codex, cursor, fixture-claude-149, fixture-codex-149, gemini, project-discovery-claude, project-discovery-codex, projec` |
| R8.1 | PASS | REEL | droits réduits (parent devient readOnly) : le rejeu de l'ancienne requête rend la même tâche et le même résultat, sans nouveau lancement ; son snapshot reste dangerFullAccess | `{"sameTask": true, "prompts": 1, "sandbox": "dangerFullAccess", "snapshotUnchanged": true, "revision": 2}` |
| R8.2 | PASS | REEL | droits réduits : une NOUVELLE requête development est refusée (permission_not_inherited) avant tout lancement ; une requête héritée reçoit le sandbox readOnly (jamais l'ancien full-access) | `{"development": "permission_not_inherited", "g2Prompts": 0, "g3Sandbox": "readOnly"}` |
| R8.3 | PASS | REEL | fait retiré, puis session tournée : aucun rejeu n'entraîne de lancement ni de nouvelle remise ; l'ancien credential est refusé ; sans fait (v1) la lecture/le rejeu de l'identité 148 ne relance rien | `{"tombstone": {"tombstone": "t3_session_unavailable", "oldCredentialAfterRotation": "t3_session_unavailable", "newCredentialWithoutFact_replay": "ok:result_available", "newCredentialWithoutFact_status": "ok:result_available"}, "startedBefore": 7, "startedAfter": 8, "g1Prompts": 1}` |
| R8.4 | PASS | REEL | O1 (comportement exact, G-P-07(b)) : fait retiré puis session tournée - (1) le montage MCP privé de l'ancien credential est fermé EN BLOC (rejeu, status ET identité : t3_session_unavailable, jamais v1) ; (2) un credential neuf qui n'a jamais eu de fait reçoit v1 et lit/rejoue la tâche admise (148) ; (3) la lecture humaine Lineage native (gardes147) lit la tâche admise, résultat intact | `{"mcpMountWithRetiredFact": {"delegateReplay": "t3_session_unavailable", "taskStatus": "t3_session_unavailable", "who": "t3_session_unavailable"}, "freshCredentialNeverHadFact": {"replay": "ok:result_available", "status": "ok:result_available"}, "humanNativeLineageCli": {"showResult": "fixture149-an` |
| R6.1 | PASS | REEL | T3 en panne après admission : la mission admise (enfant déjà lancé) va à son terme sans T3, résultat remis UNE fois à A, PID enfant sans relance | `{"state": "result_available", "deliveries": 1, "prompts": 1, "t1pid": 21437, "startsAfter": 10}` |
| R6.2 | PASS | REEL | T3 en panne : l'annulation reste native (CLI Lineage) -> tâche cancelled et PID de l'enfant réellement terminé ; la lecture native reste servie | `{"cancel": {"status": "cancelling", "task_id": "b63fc8d3-3cc5-436d-a45b-87914ad30c5a", "version": 1}, "state": "cancelled", "t2pid": 21448, "alive": false, "readResult": "fixture149-answer:t1"}` |
| R6.3 | PASS | REEL | T3 en panne : le montage MCP T3 est fermé (status et nouvelle admission refusés t3_session_unavailable), zéro ligne et zéro lancement ajoutés, aucun repli PID | `{"status": "t3_session_unavailable", "delegate": "t3_session_unavailable", "t3Prompts": 0}` |
| R6.4 | PASS | REEL | T3 revenu : l'ancien credential reste refusé (registre perdu) ; avec un credential neuf + fait, le rejeu de la requête admise avant la panne rend la MÊME tâche, sans relance | `{"stale": "t3_session_unavailable", "sameTask": true, "sameMessage": true, "prompts": 1}` |
| R9.1 | PASS | REEL | reprise du daemon, missions terminées : mêmes task_id/child/message, mêmes états (result_available, cancelled, failed), aucun nouveau lancement, aucun fournisseur survivant à l'arrêt | `{"statesEqual": true, "startsBefore": 11, "startsAfter": 11, "providersAfterDaemonStop": 0}` |
| R9.1b | PASS | REEL | reprise du daemon : aucune remise déjà accusée (ACK) n'est rejouée au parent reconnecté | `{"deliveriesAfterReconnect": 0, "replayedAfterAck": 0, "plain": 0}` |
| R9.2.a | PASS | REEL | mission EN VOL, vrai SIGTERM du daemon puis relance : la tâche est failed/unreachable DÈS le démarrage (avant la reconnexion du parent), sans attente de 2 s ni reprise | `{"before": "working", "early": "failed", "earlyError": "unreachable", "earlyStarts": 12, "startsBefore": 12}` |
| R9.2.b | PASS | REEL | la mission n'est portée que par UN tour et UN PID : 1 prompt NONCE_i1, 1 PID distinct (celui d'avant la coupure, mort), aucun fournisseur vivant à 7 s ni à 15 s (un survivant immédiat est tracé dans survivorsAfterDaemonStop) | `{"missionPrompts": 1, "distinctPids": 1, "samePidAsBeforeStop": true, "pidAliveNow": false, "providers": [1, 0, 0], "survivors": [{"pid": 24270, "firstMs": 67, "lastMs": 6872, "ppid": 1, "command": "/Applications/Xcode.app/Contents/Developer/Library/Frameworks/Python3.framework/Versions/3", "isInfli` |
| R9.2.c | PASS | REEL | starts avant = après : aucun nouvel enfant (compteur de lancements fournisseur inchangé à t0, 7 s et 15 s) ; aucun événement started/prompt après la relance | `{"startsBefore": 12, "startsT7": 12, "startsFinal": 12, "eventsAfterRestart": 0, "rows": 12, "rowsBefore": 12}` |
| R9.2.d | PASS | REEL | UNE notice d'échec failed/unreachable remise au parent reconnecté (pas de résultat, pas de rejeu), état final failed + erreur explicite | `{"state": "failed", "error": "unreachable", "notices": 1, "noticeHead": "Délégation native cbfc37f8-2ff2-449c-858f-964a6b6bb893 échouée : unreachable"}` |
| R9.2.e | PASS | REEL | le rejeu de la requête interrompue rend la MÊME tâche failed, sans nouveau lancement (pas de substitution silencieuse) | `{"replayTask": true, "replayStatus": "failed", "status": "failed", "startsFinal": 12, "prompts": 1}` |
| R9.3 | PASS | REEL | admission coupée par SIGTERM avant toute mission (état durable `starting`, 0 prompt) : après relance la tâche est failed/unreachable, 0 mission envoyée (0 prompt), 0 enfant relancé, 1 notice d'échec ; `queued` seul n'a pas pu être exposé en réel | `{"attempts": [{"exit": "starting", "post": "failed", "err": "unreachable", "prompts": 0, "startsDelta": 1, "notices": 1}, {"exit": "starting", "post": "failed", "err": "unreachable", "prompts": 0, "startsDelta": 1, "notices": 1}, {"exit": "starting", "post": "failed", "err": "unreachable", "prompts"` |
| R9.4 | PASS | REEL | racine en attente de son petit-enfant, vrai SIGTERM + relance : l'attente de la racine reste durable (waiting_for_children), le petit-enfant en vol devient failed/unreachable, AUCUN nouvel enfant ni nouveau tour (m1 et m1g : 1 prompt chacun) | `{"before": {"root": "waiting_for_children", "nested": "working", "starts": 19}, "early": {"root": "waiting_for_children", "rootError": null, "nested": "failed", "nestedError": "unreachable"}, "late": {"root": "waiting_for_children", "rootError": null, "nested": "failed", "nestedError": "unreachable"` |
| R9.4.b | **FAIL** | REEL | OBSERVATION : la racine retenue (résultat déjà capturé) sort de waiting_for_children (result_available ou failed) dans les 40 s après relance, une fois son seul descendant failed/unreachable ; sinon le parent n'a ni résultat ni échec | `{"waitedMs": 40062, "rawState": "waiting_for_children", "hasRetainedResult": true, "resultSent": false, "mcpStatus": "waiting_for_children", "mcpResultPresent": false, "lineageStatus": "ok", "noticesToParent": 0}` |
| R9.4.c | PASS | SIMULE | DIAGNOSTIC (mutation de la base privée) : cause prouvée - quand l'exécution `running` du petit-enfant mort passe à failed, la racine quitte waiting_for_children (result_available, résultat retenu intact) ; c'est donc l'exécution laissée `running` par la relance qui bloque l'attente | `{"nestedExecutionBefore": [{"execution_id": "execution-367c35be-8733-4701-a5a1-657c0d0dbbf2", "state": "running", "reason": "provider_accepted"}], "agentLinks": [{"n": 19}], "afterMutation": {"resultSentFlag": false, "stateAfter4s": "result_available", "rootState": "result_available", "rootResult": ` |
| R9.4.d | **FAIL** | SIMULE | DIAGNOSTIC : une fois la racine débloquée, son résultat retenu est remis UNE fois au parent (result_sent vrai, 1 remise corrélée) | `{"resultSentFlag": false, "stateAfter4s": "result_available", "rootState": "result_available", "rootResult": "fixture149-answer:m1", "deliveriesToParent": 0}` |
| R9.4.e | PASS | REEL | la 2e racine, bloquée de la même façon et non mutée, ne se termine que par le cancel du parent : cancelled, zéro fournisseur vivant, aucun nouveau tour (m2 et m2g : 1 prompt chacun) - le résultat retenu est perdu | `{"stateBefore": "waiting_for_children", "isError": false, "status": "cancelling", "rawStateAfter": "cancelled", "providersAlive": 0, "prompts": {"m2": 1, "m2g": 1}}` |
| R9.6 | PASS | REEL | mort du fournisseur natif en vol (SIGTERM d'UN PID vérifié, daemon vivant) : aucune relance par la flotte ni le wrapper (starts inchangé, 1 seul tour, 0 fournisseur vivant), la tâche devient terminale (failed) avec erreur explicite et UNE notice d'échec | `{"pid": 39872, "parentOfProviderIsWrapper": "non précisé", "diedAfter3s": true, "state": "failed", "error": "stdout Codex fermé pendant le tour", "startsBefore": 20, "startsAfter": 20, "prompts": 1, "liveProvidersAfter": 0, "notices": ["Délégation native 1bca52af-2d49-4910-9c0c-b20528334865 échouée ` |
| R9.5 | PASS | REEL | sur tout le rejeu (plusieurs SIGTERM + relances du daemon) chaque mission (nonce) a été portée par au plus UN tour fournisseur : aucune mission exécutée deux fois | `{"nonces": 17, "duplicated": [], "daemonStarts": 7}` |

## Limites et non prouvé

- Fournisseurs simulés (faux serveur Codex fermé) : un vrai Codex peut se comporter autrement à l'arrêt (voir O4).
- `queued` non engagé en réel (O6).
- Pas de launchd, pas de reprise de session de modèle, pas de PTY (T037/T038).
- Le binaire r6 n'est pas rejoué.
- Les contrôles R9.4.c et R9.4.d mutent la base PRIVÉE de la fixture pour un diagnostic. Ils ne décrivent pas un comportement produit.
