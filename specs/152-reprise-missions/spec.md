# Session152 - Reprise des missions natives après panne et après pause

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 152-reprise-missions
Titre: Reprise des missions natives après panne et après pause
Statut: Spécifiée
Priorité: P1
Tâches: à générer (voir tasks.md)
Tests: définis en planification (filtre `native152_*`)

Résumé:
- Contexte: Un redémarrage du daemon fait passer toute mission native engagée à `failed/unreachable`. Une pause du référent la fait échouer aussi, par deux chemins. Les horloges de la mission continuent pendant la pause. L'annulation en cascade n'est pas vérifiée avant d'agir.
- Objectif: Une mission engagée reprend après une panne et après une pause, avec son identifiant d'origine, sans lancer ni remettre de travail pendant la pause.
- Dépendances: SPEC-148, SPEC-149, SPEC-150 (livrée), SPEC-151 (livrée, hypothèse H1), ADR 027, ADR 007

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-150-socle-delegation` (exécution séquentielle 150 à 158 dans un seul worktree, sans commit automatique)
**Created**: 2026-10-11
**Status**: Spécifiée
**Priority**: P1
**Dependencies**: SPEC-148, SPEC-149, SPEC-150, SPEC-151

Critères de la feuille de route visés : Résilience 2 vers 4 (la 157 complète à 5),
Suivi et annulation vers 5. Voir
`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`.

## Besoin et périmètre

Le moteur de délégation native tourne dans le daemon Bridget. Une mission est
confiée à un enfant. Le daemon suit son état, son échéance et son résultat. Cinq
défauts rendent cette mission fragile.

**Défaut 1 - Une panne tue la mission.** Au démarrage, `prepare_restart` passe
chaque tâche `starting`, `mission_pending` ou `working` à `failed` avec l'erreur
`unreachable` (`native_delegation.rs:922-952`). Il clôt aussi l'exécution
(`close_lost_execution`, `:900-920`) et arrête l'entrée de flotte. La reprise
ordinaire des exécutions ignore les missions natives (`daemon.rs:4251-4258`).

**Défaut 2 - Une pause tue la mission, par deux chemins.**
- Chemin A : l'interruption de pause fait envoyer `DeliveryRejected`
  (`wrapper.rs:6202-6207`). Le daemon appelle `rejected()` (`daemon.rs:18078-18081`),
  qui force `failed` (`native_delegation.rs:810-828`).
- Chemin B : `tick` lit l'exécution `interrupted` et passe la tâche à `failed`
  (`native_delegation.rs:1193-1203`).

**Défaut 3 - La reprise de pause ne sait pas reprendre une mission.**
La garde de reprise ne reconnaît que `execution-{mission}` (`daemon.rs:4251-4258`).
Elle bloque donc toute reprise. Elle ne suit pas non plus la chaîne
d'exécutions créée par une reprise (`execution_store.rs:1298-1440`). Le wrapper
envoie `TurnState(false)` avant `ExecutionStateChanged(interrupted)`
(`wrapper.rs:6141,6182`). Une reprise trop rapide voit donc encore
`interrupting`.

**Défaut 4 - Les horloges ne s'arrêtent pas.** Trois horloges courent pendant
la pause :
- l'échéance de mission : `tick` la vérifie avant tout (`:1013-1023`) ;
- le suivi de la demande de mission : il a ses propres paliers
  (`daemon.rs:6828-7012`), et le palier 3 reste émis pendant la pause ;
- l'échéance de lancement : l'enveloppe de lancement fige `issued_at` et
  `deadline_at` (`fleet.rs:1662-1696`, `native_delegation.rs:534-668`). Un
  lancement soumis après une longue pause serait refusé comme expiré.

**Défaut 5 - Annulation et refus fragiles.** L'annulation en cascade arrête les
membres un par un sans savoir d'abord si tous sont arrêtables
(`native_delegation.rs:1380-1486`). Un nouvel essai rejoue tous les arrêts. Un
refus de délégation n'est pas gardé : le rejeu d'une clé réévalue l'état du
monde et peut changer de réponse (`:396-533`).

Cette session corrige ces cinq défauts avec le mécanisme de reprise de l'ADR 027
(`control_pause_interruptions.resume_execution_id`, `execution_store.rs:1422`),
sans nouveau moteur de reprise.

**Hors périmètre** : reprise sur quota et remplacement de fournisseur (157) ;
boîte de fin de tâche et attente synchrone (151) ; rondes et enfant réutilisable
(153) ; isolation par worktree (154) ; demandes de l'enfant (155) ; guidage
(156) ; reprise des tours des agents non natifs (déjà couverte par la 087) ;
nouvel état visible dans la vue Lineage T3.

## Scénarios utilisateur

### US1 - Une pause ne fait plus échouer les missions en cours (P1)

Un humain met le référent en pause pendant qu'une mission tourne. La mission est
suspendue, pas échouée. Elle garde son identifiant. L'humain lève la pause. La
mission reprend et rend son résultat au parent. Les trois séquences d'événements
sont couvertes : refus de remise d'abord, état `interrupted` d'abord, et
`TurnState(false)` avant `interrupted`.

### US2 - Aucun lancement ni aucune remise de mission pendant la pause (P1)

Pendant la pause, aucune tâche en attente n'est lancée. Aucune mission n'est
remise à un enfant. Les horloges de la mission, du suivi de demande et du
lancement sont gelées. Après la levée, le temps restant est celui qu'il y avait
avant la pause. Un lancement en attente réussit, même après une longue pause, et
sa clé de rejeu garde la même enveloppe.

### US3 - Une mission engagée survit à un redémarrage du daemon (P1)

Le daemon redémarre pendant qu'une mission tourne. L'enfant est relancé. La
mission reprend avec son identifiant d'origine. Le nombre de reprises est borné.
Au-delà de la borne, ou si l'échéance est dépassée, la mission échoue avec une
erreur nommée.

### US4 - L'annulation en cascade est vérifiée d'abord (P2)

Un parent annule une tâche qui a des descendants. Le daemon vérifie d'abord que
chaque membre peut être arrêté. S'il en trouve un qui ne le peut pas, il n'arrête
rien. Sinon, il arrête les membres, les plus profonds d'abord. Un nouvel essai
ne rejoue que les arrêts qui ont échoué.

### US5 - Les refus définitifs se rejouent à l'identique (P2)

Un parent rejoue une clé dont la délégation a été refusée de façon définitive
(par exemple `cwd_outside_parent_project`). Il reçoit le même refus. Un refus
transitoire (`referent_paused`, `task_limit`, `retention_saturated`) n'est jamais
gardé : la clé reste libre.

### US6 - Aucun changement pour l'existant (P1)

Les appels, les contrats MCP et CLI, et la vue Lineage T3 restent compatibles.
Les tâches créées avant la mise à jour se relisent sans migration.

## Exigences fonctionnelles

**Suspension et reprise après pause**

- **FR-001** : Pendant la pause, `tick` ne fait passer aucune tâche non terminale
  à `failed` à cause de la pause. Ce contrôle précède la vérification
  d'échéance.
- **FR-002** : Un refus de remise `tour annulé par le pilote` n'est pas un échec
  de mission quand l'exécution visée a une ligne de pause non reprise. Le
  gestionnaire général de `DeliveryRejected` reçoit cette qualification. Il
  conserve alors la demande suivie et n'envoie aucun avis d'échec au parent
  (`daemon.rs:18095-18125`).
- **FR-003** : Un état d'exécution `interrupted` dû à une pause n'est pas un
  échec de mission.
- **FR-004** : La reprise d'une mission après pause attend l'état d'exécution
  `interrupted`, jamais `interrupting`. Elle est pilotée par `tick`, pas
  seulement par l'événement de levée.
- **FR-005** : La reprise garde l'identifiant de mission d'origine. Le message
  repris est le message d'origine. La tâche suit la dernière exécution de la
  chaîne de continuations, pas seulement `execution-{mission}`.
- **FR-006** : La garde de reprise ordinaire reconnaît toute exécution de la
  chaîne d'une mission native. Elle ne reprend que si la tâche est reprenable
  (FR-005, FR-017).
- **FR-007** : Une annulation reçue pendant la pause est exécutée. Elle consomme
  la ligne de pause correspondante. Une levée de pause ne ressuscite pas une
  mission annulée.

**Gel des horloges**

- **FR-008** : Pendant la pause, ni le lancement d'une tâche `queued`, ni la
  remise d'une mission `mission_pending` ne sont faits. La garde unique de
  l'ADR 027 les classe comme effet autonome.
- **FR-009** : Pendant la pause, l'échéance de mission n'expire pas. L'échéance
  effective vaut l'échéance enregistrée plus la durée des pauses écoulées depuis
  l'admission de la mission. Cette durée est reconstruite depuis le journal
  durable des pauses (`control_events`), pas depuis un état écrit par `tick`. Elle
  reste exacte après un crash survenu entre l'enregistrement de la pause et le
  premier tick.
- **FR-010** : Pendant la pause, le suivi de demande d'une mission native n'émet
  ni palier ni expiration. Son temps écoulé exclut les pauses survenues depuis la
  date d'admission de la demande, lue dans la ligne durable de la demande suivie
  (pas dans l'état `working` de la tâche, qui peut être écrit plus tard). Une
  pause subie avant l'envoi de la mission ne retire rien. Le calcul survit à une
  reconnexion, à un redémarrage et à un crash entre l'admission de l'envoi et la
  sauvegarde de la tâche. L'échéance du suivi part de la date d'envoi de la
  mission (`mission_issued_at`).
- **FR-011** : La date d'émission de l'ordre de lancement (`launch_issued_at`)
  est posée à la première soumission du lancement et sauvegardée avant elle. Une
  tâche `queued` retenue par la pause est donc lancée avec une date récente. Une
  saga déjà soumise n'est jamais modifiée.
- **FR-012** : Les tâches en cours de lancement au début de la pause finissent
  leur lancement. Leur délai de 300 s court. La limite est documentée.

**Reprise après panne**

- **FR-013** : Au redémarrage, une tâche `starting`, `mission_pending` ou
  `working` n'est plus passée à `failed` si : son identité de flotte est vérifiée,
  son échéance effective n'est pas dépassée, et son compteur de redémarrages
  survécus (`restart_resumes`) est inférieur à la borne (2). Le compteur est
  incrémenté et sauvegardé dans la décision de reprise, avant toute reprise, pour
  les états `starting`, `mission_pending` et `working`.
- **FR-014** : L'enfant est relancé par le chemin de reprise de la flotte, avec
  son environnement natif complet (mission, politique, sources). Les deux
  chemins sont couverts : un lancement encore en cours (candidat de reprise) et
  un enfant déjà connecté (entrée de flotte `Running`, `daemon.rs:3995-4020`).
- **FR-015** : Une mission reprise après panne est reconstruite par la reprise
  ordinaire des exécutions. Le compteur ne dépend d'aucune exécution : il compte
  aussi les relances d'un enfant `starting` ou `mission_pending`, qui n'ont pas
  encore d'exécution. Un crash ne peut que sur-compter, jamais sous-compter.
- **FR-027** : Après une reprise, la filiation native est préservée : lien de
  flotte, `delegation_id`, rôle `native_delegate`, identité de commande. Tous les
  contrôles liés à `task_id` (`parent_fact`, admission des descendants,
  `restart_identities`) restent vrais après un, puis deux redémarrages, y compris
  pour une mission imbriquée. La commande courante est dérivée du lien de flotte
  attesté, pas stockée dans la tâche.
- **FR-029** : Aucun état incohérent d'identité ne peut empêcher le daemon de
  démarrer. Si la commande, l'instance ou le lien d'une tâche ne se réconcilient
  pas depuis le lien attesté, cette tâche échoue avec
  `native_restart_identity_mismatch` et le démarrage continue. Choix assumé
  (Art. XIX) : échec nommé d'une tâche plutôt qu'une réparation complexe.
- **FR-028** : Une panne pendant une pause ne crée ni `mission_reply_timeout`
  ni attente infinie. Le gel est appliqué dès la décision de reprise au
  démarrage. Une exécution `interrupting` dont le processus d'origine a disparu
  est réconciliée en `interrupted`.
- **FR-016** : Si une condition de FR-013 manque, la tâche échoue comme
  aujourd'hui (`unreachable`), ou avec `mission_reply_timeout` si l'échéance est
  dépassée, ou avec `native_resume_exhausted` si la borne est atteinte.
- **FR-017** : Une tâche `queued` avec un propriétaire perdu échoue comme en 150.
  Une tâche `waiting_for_children`, `result_available` ou terminale est
  inchangée par la décision de reprise ; un test de conservation le prouve pour
  chacun de ces états.

**Annulation en cascade**

- **FR-018** : Avant toute mutation (y compris le marquage `cancelling` des
  descendants) et avant le premier arrêt, le daemon classe chaque membre de
  l'arbre (supervisé, marqueur, déjà arrêté, introuvable, non géré) sans effet
  de bord.
- **FR-019** : Si un membre est non géré ou non arrêtable, aucun arrêt n'est
  exécuté et aucun descendant n'est marqué. La tâche racine reste `cancelling`
  avec l'erreur `native_cancel_unverifiable:<enfant>`. Le contrôle est refait au
  tick suivant, depuis la racine seulement. Un descendant ne devient jamais une
  racine d'annulation indépendante tant que sa racine est en cours.
- **FR-020** : Les arrêts réussis sont enregistrés dans la tâche racine
  (`cancel_stopped`). Un nouvel essai, ou une reprise après un crash entre deux
  arrêts, ne rejoue que les arrêts absents de cette liste.

**Reçus de refus**

- **FR-021** : Un refus définitif de `bridget_delegate` est enregistré avec la
  clé de rejeu, l'enveloppe et le code. Le rejeu avec la même enveloppe rend le
  même refus. Une autre enveloppe rend `envelope_mismatch`.
- **FR-022** : La liste des refus définitifs est fermée à cinq codes :
  `cwd_outside_parent_project`, `cwd_unavailable`, `model_unavailable`,
  `effort_unavailable`, `agent_type_unknown`. Le texte dynamique du registre
  (`registry.rs:157`) est ramené au code `agent_type_unknown` ; son texte n'est
  jamais enregistré. Tout autre code, et tout texte libre, n'est jamais
  enregistré.
- **FR-023** : Les reçus de refus expirent après 30 jours, dans le passage
  d'entretien de la 150.

**Visibilité et compatibilité**

- **FR-024** : `task_view` ajoute `suspended` (booléen) et `resume_attempts`. Il
  garde l'état `working`. La vue Lineage T3 ne change pas.
- **FR-025** : Les tâches enregistrées avant la mise à jour se relisent avec les
  valeurs par défaut des champs nouveaux.
- **FR-026** : Les refus et erreurs nouveaux sont documentés dans
  `docs/delegation-native.md` et la skill Bridget.

## Critères de succès

- **SC-001** : Sur les trois séquences de l'US1, 100 % des missions en cours à
  la pause sont non échouées pendant la pause et rendent un résultat après la
  levée, avec le même identifiant de mission. Aujourd'hui : 0 %.
- **SC-002** : Pendant une pause de 300 s simulée, 0 lancement et 0 remise de
  mission sont faits.
- **SC-003** : Une mission avec `reply_timeout` de 120 s, pausée 300 s puis
  levée, a un temps restant égal à celui d'avant la pause (écart au plus 2 s) et
  n'expire pas pendant la pause.
- **SC-004** : Un lancement retenu 600 s par la pause réussit à la levée. Le
  rejeu de sa clé ne produit pas `spawn_envelope_mismatch`.
- **SC-005** : Après un redémarrage, une mission `working` n'est pas `failed`,
  rend un résultat une seule fois, et sa 3e reprise échoue avec
  `native_resume_exhausted`. Deux redémarrages successifs du magasin gardent la
  filiation et les contrôles liés à `task_id`, y compris pour une mission
  imbriquée. Un crash avant ou après la reconstruction ne perd ni ne double
  jamais le compte.
- **SC-009** : Une panne pendant une pause ne produit ni `mission_reply_timeout`
  ni attente d'un terminal absent, même si le crash a lieu avant le premier tick
  qui suit l'enregistrement de la pause : la mission est reprise après la levée, dans
  les deux cas (pause encore active au démarrage, pause levée pendant l'arrêt).
- **SC-006** : Avec un membre non arrêtable sur trois, 0 arrêt est exécuté. Après
  correction, le nouvel essai exécute exactement les arrêts non encore réussis.
- **SC-007** : 100 % des refus définitifs listés se rejouent à l'identique ; 0
  refus transitoire est enregistré.
- **SC-008** : Tests de non-régression 148, 149 et 150 verts ; aucun test passe de
  vert à rouge.

## Entités et hypothèses

**Entités**

- **Mission suspendue** : tâche `working` ou en lancement, arrêtée par la pause,
  (pause active et état `queued`, `starting`, `mission_pending` ou `working`).
- **Chaîne d'exécutions** : exécution d'origine et ses continuations
  (`execution_continuations`).
- **Ligne de pause** : ligne de `control_pause_interruptions` d'une exécution
  interrompue.
- **Reçu de refus** : trace d'un refus définitif de délégation.
- **Compteur de redémarrages survécus** (`restart_resumes`) : nombre de
  redémarrages du daemon qu'une mission a traversés. Incrémenté à la décision de
  reprise, avant la reprise.
- **Journal des pauses** : suite des événements `pause_on` et `pause_off` de
  `control_events`. Source unique du temps de pause.

**Hypothèses**

- **H1** : La session 151 est livrée. La boîte de fin de tâche reprend l'envoi du
  résultat et de l'avis d'échec. La 152 ne change que les transitions avant l'état
  terminal. La minuterie d'une attente synchrone appartient à l'appelant : elle
  n'est pas gelée par la pause. Une attente sur une tâche suspendue rend
  `suspended: true` (FR-024).
- **H2** : La session 150 est livrée : `reply_timeout` par tâche, `delivery_acked`,
  pierres tombales, entretien à 120 s, refus `referent_paused`.
- **H3** : L'envoi du résultat et de l'avis d'échec reste admis pendant la pause,
  comme le prévoit l'ADR 027 pour les remises déjà acceptées.
- **H4** : La reprise d'un enfant déjà connecté crée aujourd'hui une commande
  `recovery-<uuid>` sans filiation (`daemon.rs:3995-4020`). Elle doit passer la
  filiation d'origine (`ownership_from_desired`, `fleet.rs:1649`) et rendre les
  contrôles liés à `task_id` vrais pour la nouvelle commande. Le test rouge T004
  le prouve avant le code.
- **H5** : Les tests suivent la règle du projet : `umask 077`, environnement
  privé, racine temporaire courte.
- **H6** : La livraison suit les sessions 147 à 151 : fichiers sur disque, sans
  relance automatique des services.

## Questions pour l'utilisateur

1. L'échéance de mission court-elle pendant un arrêt du daemon (hors pause) ?
   **Défaut : oui.** Seule la pause gèle les horloges. Une mission reprise dont
   l'échéance est passée échoue avec `mission_reply_timeout`.
2. Combien de reprises après panne par mission ? **Défaut : 2.** Les pauses ne
   comptent pas.
3. La vue Lineage T3 doit-elle montrer un état `suspended` ? **Défaut : non.**
   L'état reste `working`. Seule la lecture de statut MCP et CLI ajoute le
   booléen `suspended`.
