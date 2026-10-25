# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/recovery149-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/recovery149-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- 23 contrôles PASS, 1 FAIL sur 24.
- Les six scénarios US6 sont tous exécutés pour de vrai. Cinq passent sans réserve : retry, cancel, parent extérieur, identité inconnue, refus fournisseur.
- Le sixième, **reprise**, passe pour les missions terminées (R9.1, R9.1b) et **échoue pour une mission en vol** (R9.2).
- Panne T3 après admission, nested actif avec résultat retenu, retry après changement de droits : PASS.
- Les sources de production Rust ont changé après la compilation du binaire testé (`daemon.rs`, `native_delegation.rs`, `wrapper.rs`). Il faut rejouer sur le prochain binaire avant de conclure.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

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
| R6.1 | PASS | T3 en panne après admission : la mission admise (enfant déjà lancé) va à son terme par le parcours natif Bridget, résultat remis UNE fois à A, PID enfant sans relance | `{"state": "result_available", "deliveries": 1, "prompts": 1, "t1pid": 12439, "startsAfter": 10}` |
| R6.2 | PASS | T3 en panne : l'annulation reste native (CLI Lineage) -> tâche cancelled et PID de l'enfant réellement terminé ; la lecture native reste servie | `{"cancel": {"status": "cancelling", "task_id": "e0f5308c-741e-4e71-91a2-60805310b511", "version": 1}, "state": "cancelled", "t2pid": 12454, "alive": false, "readResult": "fixture149-answer:t1"}` |
| R6.3 | PASS | T3 en panne : le montage MCP T3 est fermé (status et nouvelle admission refusés t3_session_unavailable), zéro ligne et zéro lancement ajoutés, aucun repli PID | `{"status": "t3_session_unavailable", "delegate": "t3_session_unavailable", "t3Prompts": 0}` |
| R6.4 | PASS | T3 revenu : l'ancien credential reste refusé (registre perdu) ; avec un credential neuf + fait, le rejeu de la requête admise avant la panne rend la MÊME tâche, sans relance | `{"stale": "t3_session_unavailable", "sameTask": true, "sameMessage": true, "prompts": 1}` |
| R9.1 | PASS | reprise du daemon, missions terminées : mêmes task_id/child/message, mêmes états (result_available, cancelled, failed), aucun nouveau lancement, aucun fournisseur survivant à l'arrêt | `{"statesEqual": true, "startsBefore": 11, "startsAfter": 11, "providersAfterDaemonStop": 0}` |
| R9.1b | PASS | reprise du daemon : aucune remise déjà accusée (ACK) n'est rejouée au parent reconnecté | `{"deliveriesAfterReconnect": 0, "replayedAfterAck": 0, "plain": 0}` |
| R9.2 | FAIL | mission EN VOL coupée par l'arrêt du daemon : l'état est explicite (failed + erreur) ET la mission n'est pas exécutée une seconde fois par un enfant relancé (1 seul tour portant la mission) | `{"taskState": "failed", "error": "unreachable", "missionPrompts": 2, "distinctProviderPids": 2, "startsBefore": 12, "startsAfter": 13, "parentNotices": 1}` |

<a id="extrait-4"></a>

## Extrait 4

- Lancements d'enfants (PID distincts, fichier d'évidence du fournisseur) : 13 sur tout le passage. Tours : 16. Réponses : 11.
- Lignes durables : 12. États finaux : result_available, result_available, result_available, cancelled, cancelled, failed, result_available, result_available, result_available, cancelled, result_available, failed.
- Les tours dépassent les lancements car un enfant intermédiaire reçoit un tour par résultat de son petit-enfant (observé, voulu par la conception).
- Retry : 20 rejeux, 1 seul lancement, 1 seul tour, 1 seule remise corrélée.
- Cancel : 2 PID (racine + descendant) terminés, 0 fournisseur résiduel, 0 remise de résultat.
- Panne T3 : l'enfant déjà lancé finit par le parcours natif Bridget ; 1 remise ; l'annulation native termine le PID de l'enfant en attente.
- Reprise terminée : 11 lancements avant et après la relance, 0 remise rejouée après ACK, 0 fournisseur survivant à l'arrêt du daemon.
- Reprise en vol (R9.2) : 12 -> 13 lancements, 2 tours portant la mission, 1 notification d'échec.
- Arrêt final : SIGTERM sur un PID vérifié, 0 processus résiduel.

<a id="extrait-5"></a>

## Extrait 5

- **O3.** Pendant la panne de T3, un parent ne peut plus lire sa tâche par MCP (`t3_session_unavailable`). Seules la lecture et l'annulation natives (CLI Lineage) restent disponibles. La remise du résultat au wrapper continue.
- **O4.** L'arrêt du daemon ne signale pas les enfants. Un enfant occupé (fixture en `sleep`) a survécu un court moment à l'arrêt (R9.2 : 1 fournisseur vivant juste après SIGTERM, 0 à 7 s). Dépend de la fixture.
- **O5.** Ce qui est SIMULÉ ici : wrapper du fil T3, publication du fait de permissions, fournisseur enfant. Ce qui est RÉEL : daemon, MCP Rust, hôte MCP T3, CLI Lineage, comptages.
