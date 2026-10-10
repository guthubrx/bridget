# T039 - Scénarios dégradés 149 (daemon natif réel, hôte MCP T3 réel, enfants fermés)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), ronde natif/réseau r1.
Aucun code de production modifié. Aucun commit. Aucune case cochée.

## Verdict

**CHANGES_REQUIRED sur un seul point (R9.2). T039 n'est pas cochable en l'état.**

- 23 contrôles PASS, 1 FAIL sur 24.
- Les six scénarios US6 sont tous exécutés pour de vrai. Cinq passent sans réserve : retry, cancel, parent extérieur, identité inconnue, refus fournisseur.
- Le sixième, **reprise**, passe pour les missions terminées (R9.1, R9.1b) et **échoue pour une mission en vol** (R9.2).
- Panne T3 après admission, nested actif avec résultat retenu, retry après changement de droits : PASS.
- Les sources de production Rust ont changé après la compilation du binaire testé (`daemon.rs`, `native_delegation.rs`, `wrapper.rs`). Il faut rejouer sur le prochain binaire avant de conclure.

## Constat F1 (R9.2) - mission en vol et redémarrage du daemon

**Observé à chaque passage** (3 passages complets de `recovery149.ts`, 3 de `server149.ts`, 2 explorations manuelles) :

1. Une mission native est en cours (`SLOW_149:9000`, enfant vivant). Le daemon reçoit SIGTERM puis est relancé.
2. Moins d'une seconde après le début de l'arrêt (relance du daemon comprise, +0,44 s), le daemon **relance un fournisseur enfant** (nouveau PID) et lui envoie d'abord la carte de reprise (1342 caractères), puis **la mission d'origine une seconde fois** (772 caractères, +0,60 s).
3. Environ 2 s après la relance, la même tâche passe `failed` avec l'erreur `unreachable` (horodatage à la seconde). Le parent reçoit une seule notification d'échec.
4. Résultat : la mission a été présentée à **deux fournisseurs** (2 PID distincts, 2 tours portant `NONCE_i1`) alors que la tâche est déclarée échouée. Lancements d'enfants : 12 avant, 13 après.
5. Dans `server149.ts`, deux missions en vol (WAIT) donnent 4 -> 6 lancements après la relance, avec ces tâches en `failed: unreachable`.

Contraintes violées si le comportement est voulu comme tel :

- `spec.md` US6 « Reprise » : « sans dupliquer l'enfant ni relancer la mission ».
- `plan.md` : « Les reprises gardent la tâche, le modèle, la politique et la clé de remise ».

Deux mécanismes se contredisent. Le plan d'exécution marque l'exécution `unreachable` donc la tâche `failed` (`native_delegation.rs`, lecture du snapshot d'exécution). La flotte managée, elle, relance la commande figée avec une carte de reprise (comportement voulu par le test `daemon.rs` « reprise persistante »). Aucun des deux n'annule l'autre.

Risque : une mission qui écrit un fichier est exécutée deux fois alors que le parent lit un échec.
Limites de ce constat : le fournisseur est un faux serveur Codex ; un vrai Codex pourrait traiter la carte de reprise autrement. Le binaire est antérieur aux trois fichiers modifiés depuis.
Décision attendue du principal : (a) la tâche reste vivante et reprend, ou (b) la tâche échoue ET aucun enfant n'est relancé ni ne reçoit la mission.

## Les six scénarios US6 et les extras

| Scénario | Contrôles | Résultat |
|---|---|---|
| Retry | R1.1, R1.2, R1.3 | PASS |
| Cancel | R2.1, R2.2, R2.3 | PASS |
| Reprise | R9.1, R9.1b PASS ; R9.2 FAIL | PARTIEL |
| Parent extérieur | R3.1, R2.1 | PASS |
| Identité inconnue | R4.1 | PASS |
| Refus fournisseur | R5.1, R5.2, R5.3 | PASS |
| Panne T3 après admission (daemon autonome) | R6.1 à R6.4 | PASS |
| Nested actif, résultat retenu | R7.1 à R7.3 | PASS |
| Retry après changement de droits | R8.1 à R8.3 | PASS |

## Détail des contrôles

| Id | Résultat | Ce qui est prouvé | Valeur observée |
|---|---|---|---|
| R1.1 | PASS | retry : 20 rejeux (10 en vol, 10 après la fin) -> même task_id, même enfant, même message_id ; 1 lancement, 1 tour, 1 remise corrélée | `{"replays": 20, "sameAll": true, "starts": 1, "prompts": 1, "deliveries": 1, "rows": 1}` |
| R1.2 | PASS | même request_id avec une autre enveloppe -> envelope_mismatch, aucune mutation, aucun lancement | `{"payload": {"text": "envelope_mismatch"}}` |
| R1.3 | PASS | Lineage montre un seul enfant pour ce request_id (pas de doublon) | `{"tasks": 1}` |
| R7.1 | PASS | nested actif : le tour de l'enfant est fini mais la racine n'expose ni statut result_available ni résultat tant que le petit-enfant travaille ; aucune remise au parent | `{"rootStatus": "waiting_for_children", "rootResult": null, "nestedState": "working", "deliveriesDuring": 0, "starts": 3}` |
| R7.2 | PASS | la lignée est durable : parent_task_id et propriétaire racine stables (la racine appartient à A, le petit-enfant à l'enfant) | `{"rootOwnerOfNested": "74786b56", "ownerOfNested": "516e9168"}` |
| R7.3 | PASS | quand le petit-enfant a fini : la racine devient result_available, UNE seule remise corrélée à A ; le résultat du petit-enfant n'est pas remis à A | `{"root": "result_available", "nested": "result_available", "deliveries": 1, "leaked": false}` |
| R2.1 | PASS | un autre fil (B) ne peut ni annuler ni lire la tâche de A : task_unavailable, tâche et processus inchangés | `{"cancel": {"text": "task_unavailable"}, "status": {"text": "task_unavailable"}, "providers": 4}` |
| R2.2 | PASS | cancel de la racine : racine + descendant actif cancelled, PID des deux enfants réellement terminés, zéro fournisseur résiduel, aucune remise de résultat | `{"states": ["cancelled", "cancelled"], "pids": [[9750, false], [9801, false]], "liveProviders": 0}` |
| R2.3 | PASS | cancel rejoué : idempotent (même état cancelled, aucune erreur, aucun nouveau lancement) | `{"status": "cancelled", "starts": 5}` |
| R4.1 | PASS | identité inconnue : token forgé, absence totale d'identité, fil sans binding -> refus explicites ; aucune identité voisine empruntée, aucune ligne, aucun lancement | `{"bogus": ["t3_session_unavailable", "t3_session_unavailable"], "noIdentity": ["identity_not_found", "identity_not_found"], "lineage": "binding_unavailable"}` |
| R3.1 | PASS | parent extérieur : le fil B voit zéro tâche de A (liste, show), compteurs et base inchangés | `{"listBTasks": 0, "show": "task_unavailable"}` |
| R5.1 | PASS | refus fournisseur : tâche failed avec erreur explicite, 1 seul lancement, 1 seul tour (pas de reprise ni de substitution de modèle/fournisseur), 1 notification d'échec, Lineage failed | `{"state": "failed", "error": "livraison échouée: erreur Codex (code -32000; référence sha256:66bb1dfcdec778b0b928b8b230dfa24ce84519b01e7d8b321935f82f5", "prompts": 1, "notices": 1, "lineageStatus": "failed", "lineageModel": "fixtu…` |
| R5.2 | PASS | le rejeu de la requête refusée rend l'échec mémorisé sans nouveau lancement | `{"replayStatus": "failed", "prompts": 1}` |
| R5.3 | PASS | modèle ou fournisseur absent du registre : refus nommé à l'admission, aucun lancement, aucune substitution | `{"model": "model_unavailable", "agent": "type d'agent inconnu 'agent-inconnu' dans /Users/moi/.cache/bridget149-native-interop.KLsVHW/state/agents.json. Types disponibles : claude, codex, cursor, fixture-claude-149, fixture-codex-…` |
| R8.1 | PASS | droits réduits (parent devient readOnly) : le rejeu de l'ancienne requête rend la même tâche et le même résultat, sans nouveau lancement ; son snapshot reste dangerFullAccess | `{"sameTask": true, "prompts": 1, "sandbox": "dangerFullAccess", "snapshotUnchanged": true, "revision": 2}` |
| R8.2 | PASS | droits réduits : une NOUVELLE requête development est refusée (permission_not_inherited) avant tout lancement ; une requête héritée reçoit le sandbox readOnly (jamais l'ancien full-access) | `{"development": "permission_not_inherited", "g2Prompts": 0, "g3Sandbox": "readOnly"}` |
| R8.3 | PASS | fait retiré, puis session tournée : aucun rejeu n'entraîne de lancement ni de nouvelle remise ; l'ancien credential est refusé ; sans fait (v1) la lecture/le rejeu de l'identité 148 ne relance rien | `{"tombstone": {"tombstone": "t3_session_unavailable", "oldCredentialAfterRotation": "t3_session_unavailable", "newCredentialWithoutFact_replay": "ok:result_available", "newCredentialWithoutFact_status": "ok:result_available"}, "st…` |
| R6.1 | PASS | T3 en panne après admission : la mission admise (enfant déjà lancé) va à son terme sans T3, résultat remis UNE fois à A, PID enfant sans relance | `{"state": "result_available", "deliveries": 1, "prompts": 1, "t1pid": 12439, "startsAfter": 10}` |
| R6.2 | PASS | T3 en panne : l'annulation reste native (CLI Lineage) -> tâche cancelled et PID de l'enfant réellement terminé ; la lecture native reste servie | `{"cancel": {"status": "cancelling", "task_id": "e0f5308c-741e-4e71-91a2-60805310b511", "version": 1}, "state": "cancelled", "t2pid": 12454, "alive": false, "readResult": "fixture149-answer:t1"}` |
| R6.3 | PASS | T3 en panne : le montage MCP T3 est fermé (status et nouvelle admission refusés t3_session_unavailable), zéro ligne et zéro lancement ajoutés, aucun repli PID | `{"status": "t3_session_unavailable", "delegate": "t3_session_unavailable", "t3Prompts": 0}` |
| R6.4 | PASS | T3 revenu : l'ancien credential reste refusé (registre perdu) ; avec un credential neuf + fait, le rejeu de la requête admise avant la panne rend la MÊME tâche, sans relance | `{"stale": "t3_session_unavailable", "sameTask": true, "sameMessage": true, "prompts": 1}` |
| R9.1 | PASS | reprise du daemon, missions terminées : mêmes task_id/child/message, mêmes états (result_available, cancelled, failed), aucun nouveau lancement, aucun fournisseur survivant à l'arrêt | `{"statesEqual": true, "startsBefore": 11, "startsAfter": 11, "providersAfterDaemonStop": 0}` |
| R9.1b | PASS | reprise du daemon : aucune remise déjà accusée (ACK) n'est rejouée au parent reconnecté | `{"deliveriesAfterReconnect": 0, "replayedAfterAck": 0, "plain": 0}` |
| R9.2 | FAIL | mission EN VOL coupée par l'arrêt du daemon : l'état est explicite (failed + erreur) ET la mission n'est pas exécutée une seconde fois par un enfant relancé (1 seul tour portant la mission) | `{"taskState": "failed", "error": "unreachable", "missionPrompts": 2, "distinctProviderPids": 2, "startsBefore": 12, "startsAfter": 13, "parentNotices": 1}` |

## Comptage réel des exécutions, processus et remises

- Lancements d'enfants (PID distincts, fichier d'évidence du fournisseur) : 13 sur tout le passage. Tours : 16. Réponses : 11.
- Lignes durables : 12. États finaux : result_available, result_available, result_available, cancelled, cancelled, failed, result_available, result_available, result_available, cancelled, result_available, failed.
- Les tours dépassent les lancements car un enfant intermédiaire reçoit un tour par résultat de son petit-enfant (observé, voulu par la conception).
- Retry : 20 rejeux, 1 seul lancement, 1 seul tour, 1 seule remise corrélée.
- Cancel : 2 PID (racine + descendant) terminés, 0 fournisseur résiduel, 0 remise de résultat.
- Panne T3 : l'enfant déjà lancé finit sans T3 ; 1 remise ; l'annulation native termine le PID de l'enfant en attente.
- Reprise terminée : 11 lancements avant et après la relance, 0 remise rejouée après ACK, 0 fournisseur survivant à l'arrêt du daemon.
- Reprise en vol (R9.2) : 12 -> 13 lancements, 2 tours portant la mission, 1 notification d'échec.
- Arrêt final : SIGTERM sur un PID vérifié, 0 processus résiduel.

Mesures de reprise en vol (relevé brut) :

```json
[{"dt": 0.44, "event": "started", "pid": 14845}, {"dt": 0.44, "event": "thread_start", "pid": 14845}, {"dt": 0.58, "event": "prompt", "pid": 14845, "nonce": "none", "turn": 1, "chars": 1342}, {"dt": 0.58, "event": "answered", "pid": 14845, "nonce": "none"}, {"dt": 0.6, "event": "prompt", "pid": 14845, "nonce": "i1", "turn": 2, "chars": 772}]
{"state": "failed", "error": "unreachable", "updated_at": 1791638224, "completed_at": 1791638224, "started_at": 1791638220, "created_at": 1791638220, "restartAtEpoch": 1791638222}
```

## Droits après perte de session (relevé de R8.3)

```json
{"tombstone": "t3_session_unavailable", "oldCredentialAfterRotation": "t3_session_unavailable", "newCredentialWithoutFact_replay": "ok:result_available", "newCredentialWithoutFact_status": "ok:result_available"}
```

Lecture : le credential tombstoné et l'ancien credential sont refusés. Un credential neuf SANS fait (v1) rend l'état mémorisé de la requête déjà admise (rejeu et status) sans rien relancer. Une nouvelle admission avec v1 reste refusée (interop149.md S4.3).

## Observations secondaires

- **O3.** Pendant la panne de T3, un parent ne peut plus lire sa tâche par MCP (`t3_session_unavailable`). Seules la lecture et l'annulation natives (CLI Lineage) restent disponibles. La remise du résultat au wrapper continue.
- **O4.** L'arrêt du daemon ne signale pas les enfants. Un enfant occupé (fixture en `sleep`) a survécu un court moment à l'arrêt (R9.2 : 1 fournisseur vivant juste après SIGTERM, 0 à 7 s). Dépend de la fixture.
- **O5.** Ce qui est SIMULÉ ici : wrapper du fil T3, publication du fait de permissions, fournisseur enfant. Ce qui est RÉEL : daemon, MCP Rust, hôte MCP T3, CLI Lineage, comptages.

## Non couvert

Modèle réel, voie standalone Claude/GLM, observer PTY, `can_use_tool` corrélé, SIGKILL du daemon, coque desktop et UI. Voir `interop149.md`, section « Ce qui n'est PAS prouvé ici ».

Résultats bruts : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/recovery149.json` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/recovery149.stdout.txt`.
Commande : `node recovery149.ts` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes` (voir `README.md`).
