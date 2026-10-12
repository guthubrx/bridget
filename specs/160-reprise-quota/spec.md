# Session160 - Reprise d'une mission après une limite d'usage

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 160-reprise-quota
Titre: Reprise d'une mission native après une limite d'usage du fournisseur
Statut: Implemented
Priorité: P1
Tâches: 37/37 vérifiées ; audit final, Converge final et livraison restent des gates distinctes
Tests: 59 noms ciblés PASS cumulés ; T027 276 PASS, 0 FAIL, 2 ignored. FULL unique historique : 2683 PASS, 6 FAIL, 76 ignored (RC101) ; sept PASS enfants exclus. Six oracles corrigés puis replays exacts 1+5 PASS ; aucune seconde suite ni GREEN global.

Résumé:
- Contexte: quand le quota d'un fournisseur est épuisé au milieu d'une mission native, la mission échoue pour de bon et le parent reçoit un avis d'échec. T3 reprend un fil limité à l'heure de réinitialisation. Bridget ne le fait pas. Une première conception de cette reprise (session 157) a été retirée après deux rondes de contre-revue : six défauts, tous dans la reprise.
- Objectif: une mission lancée avec `resume_on_limit` attend l'heure de réinitialisation du quota, sans avis d'échec, puis reprend avec son identifiant d'origine. L'attente est bornée (6 h au plus, 2 rondes), sans boucle, et le fournisseur ne change jamais.
- Dépendances: SPEC-150, SPEC-152 (livrée, primitives prouvées en T001), SPEC-157 (livrée : mémoire de limites par fenêtre, épuisement attesté)

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-160-reprise-quota` (worktree isolé, sans commit automatique ; livraison Git par ROOT après les gates)
**Created**: 2026-10-11
**Status**: Implemented
**Priority**: P1
**Dependencies**: SPEC-150, SPEC-152, SPEC-157

Date : 2026-10-11. Accord utilisateur : être au moins aussi bon que T3, qui
reprend une mission après un quota épuisé. Cette session est la « session
future » annoncée dans `specs/157-catalogue-quotas/spec.md`. Critère visé :
Résilience 4 vers 5 (voir `docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`).

Les lignes de code citées viennent de la base `1f2c661d` (avant la session 150).
Les sessions 150, 152 et 157 les déplacent. Les tâches citent les fonctions.

## Besoin et périmètre

Un parent délègue une mission à un enfant. Le fournisseur de l'enfant atteint sa
limite d'usage (fenêtre de 5 heures, par exemple). La mission échoue. Le parent
reçoit un avis d'échec. L'humain doit relancer à la main après la
réinitialisation.

**Ce que fait T3.** `limitRecoveryCommand` relance le fil limité quand l'heure de
retour est atteinte. Il exige une erreur de classe `usage_limit`, une heure de
retour postérieure à la fin du run (sinon « un retour déjà passé ne peut pas
démarrer une boucle »), et une identité `fil:run:retour`. Tout est dérivé de
faits persistés : aucune minuterie n'est à restaurer
(`apps/server/src/orchestration-v2/UsageLimitRecoveryWorker.ts`). La reprise est
un réglage global, T3 ne change jamais de fournisseur.

**Ce que fait Bridget aujourd'hui.** Rien. Après la 152, une pause et une panne
ne font plus échouer la mission, mais un quota épuisé la fait toujours échouer
par deux chemins :
- chemin A : le fournisseur refuse la remise. Le wrapper publie la transition
  d'exécution `failed`, puis `DeliveryRejected`
  (`crates/bridget-daemon/src/wrapper.rs:6218-6262`). Le daemon appelle
  `native_delegation::rejected()`, qui force `failed`
  (`native_delegation.rs:810`). Le gestionnaire général retire ensuite le suivi
  de la demande et envoie au parent « Échec de livraison de la demande »
  (`crates/bridget-daemon/src/daemon.rs:18078-18125`) ;
- chemin B : `tick` lit l'exécution `failed` et passe la tâche à `failed`
  (`native_delegation.rs:1197`).

**Pourquoi une première conception a été retirée.** La contre-revue de la 157 a
trouvé six défauts. Ce sont les exigences d'entrée de cette session :

1. **Compteur durable.** Le compteur de rondes ne doit pas disparaître quand
   l'attente est soldée. Sinon, la borne de rondes n'existe pas.
2. **Reconstruction ciblée d'une exécution `failed`.** La reprise ordinaire exclut
   l'état `failed` (`execution_store.rs:1327` pour la sélection, `:1378` et
   `:1395` pour les mises à jour). Il faut une reconstruction ciblée, autorisée
   seulement pour une reprise quota qualifiée.
3. **Identité exacte de relance.** L'identité doit être enregistrée avec la
   continuation, dans la même transaction. Un rejeu après un crash solde
   l'attente sans seconde relance.
4. **Cause quota distincte.** La cause doit être qualifiée durablement et exclue
   du compteur de pannes de 152 (`restart_resumes`, borne 2).
5. **Gel cumulé sans double comptage.** Une pause et une attente de quota qui se
   chevauchent ne retirent le temps commun qu'une fois.
6. **Gestionnaire de `DeliveryRejected`.** Il doit sortir avant de supprimer le
   suivi et de notifier le parent, comme 152 le fait pour la pause.

Cette session traite ces six exigences. Chacune a un test rouge (T003 à T007).

**Ce qui est dans la session.**
- Un paramètre `resume_on_limit` de `bridget_delegate` (opt-in par tâche).
- La qualification d'un échec de quota, avec une attente de décision de 5 s au
  plus.
- L'attente durable, la relance à l'heure de retour, les bornes.
- Le gel cumulé des horloges, la reconstruction ciblée, la vue d'état.
- Une extension compatible de la trame `DeliveryRejected` (deux champs
  optionnels) pour corréler un refus à son exécution.

**Hors périmètre** :
- changement de fournisseur en cours de mission (jamais implicite) ;
- persistance de la mémoire des limites après redémarrage (dette de la 157) ;
- avis informatif au parent au début de l'attente (question 3) ;
- propagation du gel aux tâches ancêtres (limite notée) ;
- quota partagé entre fournisseurs d'un même compte (limite de la 157) ;
- budgets de tokens ou de coût (session158) ;
- reprise après panne ou pause (152, livrée).

## Scénarios utilisateur

### US1 - Une mission opt-in n'échoue pas sur un quota (P1)

Un parent délègue avec `resume_on_limit: true`. Le fournisseur de l'enfant
atteint sa limite. Le parent ne reçoit aucun avis d'échec. Son suivi de la
demande reste en place. La lecture de statut montre l'attente et l'heure de
reprise. Cela vaut pour les deux chemins d'échec et pour les deux ordres
d'arrivée du fait de quota et de l'échec. Si aucun fait de quota ne vient dans
les 5 s, la mission échoue avec son erreur d'origine, et le parent reçoit l'avis d'échec
durable habituel.

### US2 - La mission reprend à l'heure de retour, une seule fois (P1)

À l'heure de réinitialisation plus 30 s, la mission reprend avec son identifiant
d'origine, chez le même fournisseur. L'identité de relance est enregistrée avec
la continuation. Un crash à n'importe quelle étape ne produit ni seconde relance
ni attente orpheline. La mission rend son résultat une seule fois.

### US3 - Les bornes empêchent toute boucle (P1)

Une tâche attend 6 h au plus et fait au plus 2 rondes. Le troisième échec de
quota, ou un retour à plus de 6 h, ou un retour déjà passé, donne un échec
normal `usage_limit` avec l'heure de retour. Les reprises après panne de 152
gardent toute leur borne : après 2 rondes de quota, 2 redémarrages du daemon sont
encore survécus.

### US4 - Les horloges sont gelées sans double comptage (P1)

Pendant l'attente, l'échéance de la mission et le suivi de la demande sont
gelés. Si une pause du référent chevauche l'attente, le temps commun n'est retiré
qu'une fois. L'annulation pendant l'attente fonctionne. Un redémarrage du daemon
pendant l'attente garde la tâche et ne reconstruit pas l'enfant avant l'heure.

### US5 - Aucun changement sans opt-in (P1)

Un appel sans `resume_on_limit` produit les mêmes octets canoniques et le même
comportement qu'avant. La vue Lineage T3 ne change pas : la tâche reste
`working`. Le catalogue annonce la fonction dans `features`.

## Exigences fonctionnelles

**Opt-in et qualification**

- **FR-001** : `bridget_delegate` accepte `resume_on_limit` (booléen optionnel).
  Absent, la fonction est inactive. Le champ fait partie de l'enveloppe
  idempotente, et « absent » et « `false` explicite » sont deux enveloppes
  différentes (`envelope_mismatch` au rejeu). Sans le champ, rien ne change.
- **FR-002** : Seule une tâche `working` avec `resume_on_limit` peut entrer en
  attente. Les causes de pause (règles de 152) passent avant. Le refus de remise
  d'un pilote interrompu et l'état `interrupted` ne sont jamais une limite.
- **FR-003** : Un échec est qualifié « limite d'usage » si une fenêtre épuisée et
  attestée (157, FR-008) est connue pour le type d'agent de la tâche, observée
  après l'envoi de la mission (ou après la fin de la dernière attente), avec une
  heure de retour dans `(maintenant, maintenant + 6 h]`, et si la tâche a fait
  moins de 2 rondes. Une limite Codex globale sans fenêtre attestée ne qualifie
  jamais.

FR003 est une politique de corrélation temporelle, pas une preuve que le quota cause cet échec. Un fait admissible antérieur à l’échec peut qualifier une panne sans rapport et faire répéter des effets de la mission. L’opt-in, les bornes6h/2rondes et l’idempotence de la relance ne garantissent ni causalité ni exactly-once des effets externes. Aucune fenêtre W60 ou test supplémentaire n’est ajouté.

- **FR-004** : Le fait de quota et l'échec n'arrivent pas dans un ordre garanti.
  Si aucun fait ne qualifie à l'instant de l'échec, la tâche `working` garde une
  décision en attente (`quota_hold`). C'est une politique bornée à 5 s, pas une
  garantie d'ordre. Un fait qui qualifie pendant ce délai lance l'attente, qui
  commence à l'instant de l'échec. Sinon, la tâche échoue avec son erreur
  d'origine. Un fait qui arrive après les 5 s ne change rien : l'échec explicite
  est le comportement prévu.
La borne porte sur l’observation du fait : `observed_at <= quota_hold.since + 5`. Un tick retardé peut résoudre le hold plus tard, mais ne rend jamais admissible un fait observé après cette borne. La classification emploie `now` réel au tick de décision, pas `hold.since+5` comme horloge fictive. La borne observée s’applique à chaque fenêtre avant sélection : observed_at<=since+5. Un résultat différent quand now change est prévu ; un fait observé à10:42 pour un hold commencé à10:40 dépasse5s et est exclu.

- **FR-004b** : Ordre fixe dans `tick`, pour une tâche donnée : annulation, puis
  résolution du `quota_hold`, puis saut de pause de 152, puis expiration de
  l'échéance, puis relance. La résolution du hold passe donc avant la pause. Pendant
  le hold, l'échéance n'expire pas et le suivi de la demande n'émet ni palier ni
  expiration. Le hold se résout sans dépendre du parent (déconnecté ou non) ni de
  la pause : passer à l'attente est une écriture d'état, passer à l'échec garde
  l'avis durable de l'article FR-006.
- **FR-005** : Si l'épuisement est attesté mais la reprise impossible (retour à
  plus de 6 h, retour passé admissible seulement pour un fait frais observé pendant le hold, 2 rondes faites), la tâche échoue avec l'erreur
  `usage_limit` et l'heure de retour dans `limit_reset_at`.


Lecture160 conservatrice : `attested_exhausted_since(type,since,now,past_since:Option<i64>)` garde la preuve enregistrée par157 et son admission inchangée. Observation strictement après `since`, puis reset futur OU (`past_since=hold.since` et observation>=past_since). Hors résolution du hold, past_since=None ; ancienne observation expirée=NoFact. Le cas fait observé pendant le hold puis expiré à la décision donne `Past{reset_at}`/usage_limit. Une fenêtre rejetée à son enregistrement n’est jamais inventée.

**Gestionnaire et silence**

- **FR-006** : Pour TOUTE tâche opt-in, le gestionnaire général de
  `DeliveryRejected` sort sans rien faire d'autre que retirer le suivi quand
  l'échec est décidé : plus de message direct « Échec de livraison de la
  demande », pour aucun échec (quota, réponse vide, refus immédiat, refus tardif
  après la fin d'un hold). Le seul avis est l'avis d'échec durable existant (clé
  stable `native-failure-<task_id>`, remise au parent reconnecté,
  `native_delegation.rs:1048`). Pendant le hold et l'attente, le gestionnaire
  sort sans retirer le suivi. Les tâches sans `resume_on_limit` gardent leur
  comportement exact (message direct et avis durable).
- **FR-006b** : Corrélation des refus. `DeliveryRejected` est étendu de deux
  champs optionnels, `execution_id` et `generation`, remplis par le wrapper
  depuis sa liaison d'exécution quand elle existe. Pour une tâche opt-in, le
  daemon compare ces champs à la feuille de la chaîne (identifiant et
  génération). Même mission, autre exécution : refus ancien, sans effet. Champs
  absents (ancien wrapper, ou liaison absente) : si la tâche n'a jamais été
  relancée, le refus est traité comme courant ; si elle a été relancée au moins
  une fois, la corrélation est impossible et la tâche échoue explicitement avec
  `native_refusal_uncorrelated` (jamais ignoré). Un ancien daemon ignore les
  champs ajoutés.
- **FR-006c** : Seul un refus courant dont la feuille de chaîne est `failed`
  peut qualifier une limite. Le refus d'une injection synchrone (aucune
  transition d'exécution) et la réponse vide (transition `completed` puis refus)
  ne sont jamais repris : la tâche échoue avec son erreur d'origine. Limite
  assumée, à revoir si T002 montre qu'un vrai refus de quota passe par là.

La corrélation FR006b précède toute classification FR006c/quota. Après une relance, des champs absents donnent donc `native_refusal_uncorrelated`, même pour un refus synchrone. Une réponse vide termine l’Execution en `completed`, tandis que la Task demeure `working` jusqu’au refus : elle échoue avec l’erreur d’origine si le refus est corrélé. Les Task déjà `result_available` ou `cancelled` sont préservées ; une Task déjà `failed` ne reçoit pas de nouvel avis.

- **FR-007** : Pendant la décision en attente et pendant l'attente, aucun avis
  d'échec n'est envoyé et la tâche n'apparaît pas dans les échecs à notifier.

**Attente durable, compteur, cause**

- **FR-008** : Chaque attente est une entrée de `quota_waits`, liste de la tâche
  où l'on ajoute sans jamais retirer. Le nombre de rondes est la longueur de la
  liste. Il survit à la fin de l'attente et au redémarrage.
- **FR-009** : La qualification est enregistrée sur l'exécution échouée avant
  l'entrée de la tâche (deux magasins, dans cet ordre). Si Task.save échoue après cette qualification, réutiliser sa même identité et son reset pour finir le même intent Task au retry ; aucun calcul d'une identité nouvelle. Une qualification sans TaskQuotaWait ne peut jamais reconstruire. Une entrée sans qualification ne peut pas relancer. Si le stockage est indisponible, conserver le checkpoint, propager l'erreur et réessayer ; ne jamais annoncer une clôture de hold non écrite.
- **FR-010** : La cause quota est distincte de la panne : la continuation porte
  la raison `usage_limit_reset`, et `restart_resumes` de 152 n'est jamais
  incrémenté par une relance de quota.

**Relance**

- **FR-011** : À l'heure de retour plus 30 s, sous la garde de pause
  (`NativeMissionDispatch` de 152), la tâche `working` non annulée, dont l'enfant
  est connecté, est relancée. La relance suit des étapes durables dans cet
  ordre : l'entrée passe à `relaunching` (tâche) ; la reconstruction est faite
  (exécution) ; la remise du message est réservée puis gravée ; l'entrée n'est soldée
  (tâche) qu'après lecture de la phase `acked`. Une continuation créée ou une
  remise gravée `dispatching` ne vaut pas accusé : l'entrée reste `relaunching`
  et chaque tick relit la remise existante. Le chemin waiting doit atteindre retour+30 ; le chemin déjà relaunching est repris à chaque tick sans nouvelle condition horaire. Les gardes de pause, annulation et échéance restent prioritaires.
- **FR-011b** : La remise est déterministe. La date d'émission du message
  canonique est figée (date d'observation de la continuation), pas `now`. États
  lus et reprise : réservation absente (réserver, graver) ; réservée sans remise
  (graver) ; remise `dispatching` (attendre, l'entrée reste `relaunching`) ;
  remise `acked` (solder) ; remise `indeterminate` ou `orphaned` : échec
  explicite `native_relaunch_failed` avec l'avis durable. Le solde n'a lieu qu'à
  `acked`. Raison : le daemon met en quarantaine (`indeterminate`, état
  absorbant) toute remise native retrouvée `dispatching` après une
  reconnexion (`daemon.rs:4195`, `send_delivery.rs:294`) ; un crash entre la
  gravure et l'émission peut donc faire que le message ne parte jamais. Soit le
  message est remis une fois, soit la tâche échoue explicitement. Une remise
  non reprenable (enveloppe différente) échoue de même.

Toute sortie d’échec depuis `relaunching` réconcilie la feuille et la remise avant Task.failed/cleanup, en réemployant `close_lost_execution` et T024 : cible child vérifiée, CAS de la feuille vers `unreachable`, remise dispatching mise en quarantaine. Une erreur de clôture se propage ; l’entrée reste `relaunching` pour réessayer le settlement. Aucun nouvel arrêt physique. Les résultats reçus pour Task.failed sont déjà ignorés par capture_reply (ACK transport seulement). Cela ne garantit pas l’unicité des effets externes.

- **FR-011c** : Une chaîne d'exécutions dont l'entrée de quota est ouverte est
  réservée à la relance de quota. La reprise ordinaire (`may_resume`,
  `schedule_execution_recovery`) la refuse, y compris après la création de la
  continuation et avant le solde.
- **FR-012** : L'identité de relance est `limit-resume:<task_id>:<ronde>:<retour>`.
  Elle est enregistrée avec la continuation dans la même transaction. Un rejeu
  rend la continuation existante sans en créer une seconde.
- **FR-013** : La reconstruction ciblée accepte une exécution `failed` seulement
  si elle est qualifiée (FR-009), si elle est la feuille de sa chaîne et si son
  identité n'a pas encore servi. La reprise ordinaire des exécutions continue
  d'exclure `failed`.
- **FR-014** : Si l'enfant n'est pas connecté à l'heure, la relance est réessayée
  à chaque tick. Le gel de l'attente ne dépend pas de la connexion : il s'arrête
  à l'heure de retour plus 30 s. Passé ce point, l'échéance effective court de
  nouveau et la tâche échoue avec `mission_reply_timeout` à son échéance.
- **FR-015** : La mission relancée garde sa définition épinglée : même
  `agent_type`, même modèle. Le champ `fallback` n'est jamais réévalué. Aucune
  substitution de fournisseur.

**Gel et interactions**

- **FR-016** : Le gel d'une tâche est l'union de ses pauses et de ses attentes de
  quota. Un intervalle d'attente va du début de l'attente au solde, borné à
  l'heure de retour plus 30 s. Le gel de pause reste distinct et illimité tant
  que la pause dure. L'échéance effective, le suivi de la demande (paliers, expiration,
  `frozen_secs`) et la décision de reprise de 152 utilisent cette union. Un
  intervalle commun n'est compté qu'une fois.
- **FR-017** : L'annulation pendant un `quota_hold` ferme le hold, sans relance, par T024. L'annulation pendant l'attente est exécutée, ferme l'entrée et
  empêche toute relance. Pendant `relaunching`, elle réconcilie d'abord la
  continuation et la remise : elle relit l'identifiant de l'exécution reprise,
  clôt la feuille de la chaîne, marque la remise non livrée (indéterminée) si
  elle est en cours, puis ferme l'entrée. Si cette réconciliation échoue, la
  tâche reste `cancelling` avec `native_cancel_unverifiable:quota_relaunch`
  (mécanisme de 152) au lieu d'être déclarée annulée. Pendant une pause, la relance est différée, l'attente
  reste ouverte.
- **FR-018** : Un redémarrage du daemon pendant l'attente garde la tâche
  `working`, ne reconstruit pas l'enfant avant l'heure, n'incrémente pas
  `restart_resumes` et applique l'échéance effective. Un redémarrage pendant la
  décision en attente la résout en échec normal, même si un fait avait été vu dans la mémoire volatile avant le redémarrage ; cette mémoire n’est pas une autorisation durable de reprise.

**Compatibilité et documentation**

- **FR-019** : Les champs de la tâche et de la vue sont additifs. La projection
  Lineage ne change pas.
- **FR-020** : Le catalogue annonce `features.resume_on_limit`,
  `features.resume_max_wait_secs` et `features.resume_max_rounds`. Les nouveaux champs, refus et erreurs sont
  documentés dans `docs/delegation-native.md` et la skill Bridget.

## Critères de succès

- **SC-001** : Pour les 2 chemins d'échec × les 2 ordres d'arrivée, une mission
  opt-in qui touche un quota : 0 avis d'échec, suivi conservé, mission reprise,
  résultat rendu une seule fois avec le même identifiant de mission.
- **SC-002** : Sans fait de quota, l'échec explicite arrive au plus 5 s plus un
  tick après l'échec, une seule fois, avec l'avis d'échec durable. Pendant ces
  5 s, l'échéance n'expire pas et aucun rappel n'est émis, même avec une
  échéance imminente, une pause qui commence, ou un parent déconnecté.
- **SC-003** : Après 10 échecs de quota consécutifs avec la même heure de retour,
  il y a au plus 2 relances, puis `usage_limit`. 0 boucle.
- **SC-004** : Aux huit frontières a–h de T008 : (a) qualification sans TaskWait ; (b) waiting ; (c) relaunching sans reconstruction ; (d) continuation sans réservation ; (e) réservation sans remise ; (f) dispatching gravé sans émission ; (g) acked sans solde ; (h) orphaned. Zéro seconde continuation et zéro attente orpheline. Le message d’origine est remis exactement une fois, ou la Task échoue avec native_relaunch_failed et avis durable. Seule une remise acked permet le solde ; une remise perdue/non reprenable donne cet échec explicite. Un message jamais envoyé avec attente soldée est interdit. Cette idempotence ne garantit pas l’unicité des effets externes.

- **SC-005** : Après2rondes de quota, les pannes ordinaires APRÈS relance utilisent le compteur `restart_resumes` de152 : deux redémarrages survécus, puis `native_resume_exhausted`. Les redémarrages pendant `waiting` ne l’incrémentent pas.
- **SC-006** : Pause de 600 s dans une attente de 1800 s : l'échéance effective
  augmente de 1800 s, pas de 2400 s (à 2 s près). Le suivi de la demande suit la
  même mesure.
- **SC-007** : 0 exécution `failed` reconstruite hors d'une reprise quota
  qualifiée.
- **SC-008** : 0 substitution de fournisseur : la cible de la tâche relancée est
  identique à celle d'avant.
- **SC-009** : Sans `resume_on_limit` : mêmes octets canoniques, tests de
  non-régression 148 à 157 verts.
- **SC-010** : Enfant absent après l'heure de retour plus 30 s : la tâche échoue
  avec `mission_reply_timeout` à l'échéance (l'échéance n'avance plus).
- **SC-011** : Un refus ancien (autre exécution ou autre génération que la
  feuille) reçu après le solde d'une relance : 0 effet, même quand la nouvelle
  feuille est déjà `failed`. Un refus sans corrélation reçu après une relance :
  échec explicite `native_refusal_uncorrelated`, jamais ignoré. Injection
  synchrone refusée et réponse vide avec DeliveryRejected courant corrélé : erreur d'origine sans reprise. Champs absents : erreur d'origine seulement si la Task n'a jamais été relancée ; sinon native_refusal_uncorrelated prioritaire. Si aucun DeliveryRejected n'arrive après une réponse vide, la Task attend le timeout de mission existant ; aucun refus n'est inventé.
- **SC-013** : Pour tout échec d'une tâche opt-in, le parent reçoit exactement un
  avis (l'avis durable) et aucun message direct, y compris pour un refus
  immédiat (`TooFar`, `Past`, `RoundsDone`) et pour un refus tardif après la fin
  d'un hold. Les tâches non opt-in gardent leur comportement.
- **SC-012** : Pendant la reprise de quota, la reprise ordinaire de 152 ne
  produit jamais de seconde continuation, y compris à la reconnexion de l'enfant
  après un crash.

## Entités et hypothèses

**Entités**

- **Attente de quota** (`QuotaWait`) : entrée durable de la tâche : début, heure
  de retour, ronde, identité, phase.
- **Décision en attente** (`QuotaHold`) : fait durable de la tâche qui garde un
  échec non encore classé, 5 s au plus.
- **Qualification** : ligne de l'exécution échouée qui autorise sa reconstruction
  ciblée.
- **Ronde** : une attente suivie d'une relance.

**Hypothèses**

- Les sessions 150, 152 et 157 sont livrées. La porte T001 prouve chaque
  primitive utilisée sur le code livré. Sinon, la session s'arrête.
- Limite fournisseur : une limite Codex globale sans fenêtre attestée ne qualifie pas ; T002 n’a vu aucun refus de quota réel. Déclenchement réel Codex non prouvé, sans gate fournisseur supplémentaire ; limite reprise dans ADR T029.
- Un vrai refus de quota est visible par l'un des deux chemins A ou B, et le
  fait de quota est enregistré par la 157. Cela n'a pas été vu en réel. T002
  cherche un exemple dans le journal. Sans exemple, la validation le dit.
- L'enfant d'une mission en attente reste connecté (agent persistant). S'il
  disparaît, FR-014 s'applique.
- Le wrapper connaît l'exécution et la génération du message refusé dans la
  plupart des chemins (liaison `bindings`). Pour l'injection synchrone refusée,
  la liaison peut manquer : T036 le vérifie. Sans corrélation, une tâche relancée
  échoue explicitement (FR-006b).
- Quand l'enfant est connecté et libre, la remise de la mission reprise passe par
  la même voie que la reprise ordinaire (queue de `schedule_execution_recovery`,
  extraite en T019).
- Le gel d'un enfant ne gèle pas la tâche de son parent : si le parent expire
  avant l'enfant, l'annulation en cascade de 152 s'applique (limite notée).
- Les tests suivent la règle du projet : `umask 077`, environnement privé,
  racine temporaire courte (voir `quickstart.md`).
- Base livrée : `6907df9ba15b58df05c2041631005beb57a1bed6` (159B). Livraison Git/disque/activation Bridget par ROOT après les gates et absence de mission non terminale ; pas de redémarrage T3. Les anciennes bases et lignes sont des références historiques à revalider par nom.

## Questions pour l'utilisateur

1. **Bornes d'attente : 6 h au plus et 2 rondes par tâche ?** Défaut retenu :
   oui. Six heures couvrent la fenêtre de 5 h. Une fenêtre hebdomadaire est donc
   refusée (échec `usage_limit` avec l'heure de retour). Deux rondes évitent toute
   boucle de coût.
2. **Enfant absent à l'heure de retour : réessayer jusqu'à l'échéance effective ?**
   Défaut retenu : oui, puis `mission_reply_timeout`. Aucun nouveau mécanisme de
   relance de l'enfant n'est créé.
3. **Avis informatif au parent au début de l'attente ?** Défaut retenu : non. Le
   statut (`quota_wait`) suffit, et la règle « jamais d'avis d'échec pendant
   l'attente » reste simple. L'avis pourra venir avec la boîte de fin de tâche
   (151).

Priorité SC011 : après une relance, un refus sans corrélation donne native_refusal_uncorrelated avant toute règle d’erreur d’origine. L’erreur d’origine du refus synchrone/réponse vide ne vaut que pour un refus courant corrélé, ou une tâche jamais relancée.
