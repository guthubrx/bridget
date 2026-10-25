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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/recovery149-r2.md#extrait-1).

Lecture : le texte « lecture et rejeu inchangés » est plus large que le comportement. Il est vrai pour les voies humaine/native et pour un credential neuf. Il est faux pour le montage MCP de l'ancien credential.

Proposition de clarification normative (pour la revue finale des sources, sans nouvelle demande à l'utilisateur) :
> G-P-07(b) : le montage MCP privé d'un credential dont le fait a été retiré, révoqué ou tourné est fermé en bloc (`t3_session_unavailable`, y compris `bridget_task_status`, `bridget_task_cancel`, `bridget_who` et le rejeu), jamais v1. « Lecture et rejeu inchangés » s'entend de la lecture humaine Lineage native (gardes 147), de l'annulation native, et d'un credential neuf sans fait (v1, identité/status/cancel/lecture 148). Aucun contournement de la production n'est requis.

### O6 - `queued` n'a pas pu être exposé après SIGTERM

J'ai admis une délégation puis envoyé SIGTERM aussitôt, 6 fois (puis 3 fois dans la version finale). La base montre `starting` à chaque fois, jamais `queued`. Le cas `queued` non engagé reste couvert par le test unitaire `native149_apres_redemarrage_le_tick_admet_le_queue` seulement ; il n'est pas prouvé en réel. Autre résultat réel (R9.3) : une admission coupée en `starting` devient `failed: unreachable`, aucune mission envoyée, aucun enfant relancé, une notice d'échec.

### O7 - lancement pendant l'arrêt

Dans R9.3, un fournisseur est lancé pendant l'arrêt du daemon (`startsDuringStop = 1`, 0 prompt). Il ne reçoit jamais de mission et n'est jamais relancé. C'est cohérent avec « aucune mission portée deux fois », mais cela laisse un processus lancé sans tâche vivante.

## Autres vérifications demandées

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/recovery149-r2.md#extrait-2).

## Résultats détaillés (37 contrôles)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/recovery149-r2.md#extrait-3).

## Limites et non prouvé

- Fournisseurs simulés (faux serveur Codex fermé) : un vrai Codex peut se comporter autrement à l'arrêt (voir O4).
- `queued` non engagé en réel (O6).
- Pas de launchd, pas de reprise de session de modèle, pas de PTY (T037/T038).
- Le binaire r6 n'est pas rejoué.
- Les contrôles R9.4.c et R9.4.d mutent la base PRIVÉE de la fixture pour un diagnostic. Ils ne décrivent pas un comportement produit.
