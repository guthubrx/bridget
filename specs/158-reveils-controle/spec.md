# Session158 - Contrôle humain, sécurité de l'appelant et réveils

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 158-reveils-controle
Titre: Vue globale, arrêt global et portées MCP (158a) ; réveils et rejeu (158b, spec seule)
Statut: Spécifiée (révisée après contre-revue Codex du 2026-10-11)
Priorité: P1 (158a), P2 (158b)
Tâches: 158a générées (22) ; 158b à générer après livraison de 158a
Tests: définis en tasks.md (filtres `control158_*`, `mcp158_*`)

Résumé:
- Contexte: Bridget n'offre ni vue globale des délégations sans fil T3, ni arrêt global. Les outils MCP ne déclarent aucune portée. Aucun réveil programmé n'existe et les faits d'observation manqués ne sont pas rejoués.
- Objectif: Contrôle humain de 3 à 4 et Sécurité de l'appelant de 4 à 5 (158a) ; préparer Réveil de 3 à 5 (158b). Pas d'interface graphique, pas de service HTTP.
- Dépendances: SPEC-150 (admission, pause, délai), SPEC-151 à 157 (outils MCP à classer), ADR 027, ADR 011

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓ (158a)
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-158-reveils-controle` (worktree isolé158 ; ancien worktree150 historique).
**Created**: 2026-10-11
**Status**: Spécifiée
**Priority**: P1
**Dependencies**: SPEC-150 à 157 ; ADR 027, ADR 011

Feuille de route : `docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`.
Comparaison : `specs/150-socle-delegation/research-t3-bridget.md`.
Contre-revue : `adversarial-review-codex.md` (10 objections vérifiées).
Base requise : livraison157 terminée et vérifiée par ROOT avant le GO158A.
Le reçu canonique de livraison157 et ses empreintes seront renseignés à ce GO ;
aucun SHA157 livré n'est déduit de cette préparation. Les preuves anciennes
(base `1f2c661d`) restent historiques. Revalider les fonctions sur cette base,
puis renseigner la branche et la racine du worktree158A.

## Découpage 158a / 158b

| Partie | Contenu | Critères | État |
|---|---|---|---|
| **158a** | Vue globale, arrêt global, portées MCP et plafond | Contrôle humain 3→4, Sécurité 4→5 | spec + plan + tâches complets |
| **158b** | Réveils programmés, déclencheur local `bridget wake`, rejeu des faits manqués, secret | Réveil 3→5 | spec seule |

**Budget de tours : retiré de 158a** (décision après la contre-revue). Aucun tour
attesté, dédupliqué et agrégé sur toute la tâche n'existe pour Claude ET Codex
(preuves dans `adversarial-review-codex.md`, points 6 à 8 et 10). Le budget de
temps existe déjà (`reply_timeout`, 150). Voir question 3.

158b dépend de 158a pour la table des portées et pour la garde commune
d'admission (un réveil est un effet autonome).

## Besoin et périmètre (158a)

Bridget reste un bus entre agents. Le contrôle humain passe par la CLI sur la
socket Unix, avec un terminal interactif (ADR 027).

**Défaut 1 - Vue des délégations liée à T3.** `bridget lineage` exige
`--t3-thread` et `--project-root` (`cli_lineage.rs:19-20`). Le daemon rattache
ensuite la requête à un fil T3 (`daemon/native_lineage.rs`, `authorize`). Un
humain hors de T3 ne voit rien et ne peut pas lister toutes les tâches.

**Défaut 2 - Pas d'arrêt global.** La pause (ADR 027) suspend l'autonomie. Elle
ne refuse pas durablement les délégations et ne se confond pas avec un arrêt
d'urgence. T3 n'a pas d'arrêt global non plus.

**Défaut 3 - Aucune portée déclarée par outil MCP.** Le catalogue est une liste
de valeurs `json!` (`mcp.rs:1891-2235`), l'exécution un `match` sur le nom
(`mcp.rs:470-598`). Un enfant en posture `discovery` appelle les mêmes outils
qu'un parent. T3 déclare des portées par outil (`McpToolAccess.ts`).

## Scénarios utilisateur (158a)

### US1 - Voir toutes les délégations sans T3 (P1)

L'humain tape `bridget control delegations`. Il voit les tâches de tous les
parents : propriétaire, enfant, état, âge, échéance restante. Il filtre sur les
actives. Il annule une tâche par son identifiant. Aucun fil T3 n'est demandé.
Un client agent qui négocie la capacité de contrôle est refusé.

### US2 - Arrêter toutes les délégations (P1)

L'humain tape `bridget control halt-delegations --reason "..."`. Toute nouvelle
délégation est refusée (`delegations_halted`). Avec `--cancel-active`, les
tâches actives sont aussi annulées en cascade. `resume-delegations` lève
l'arrêt. Un agent ne peut ni poser ni lever l'arrêt. L'arrêt et la pause se
composent dans une seule décision d'admission.

### US3 - Portées déclarées pour chaque outil MCP (P1)

Chaque outil MCP a une portée déclarée dans une table unique, par action pour
les outils à actions. Un test exhaustif échoue si un outil n'a pas de portée ou
si une portée désigne un outil absent. Un enfant en posture `discovery` ne peut
appeler que les actions de lecture et de son propre état. Il lit un fil et
confirme sa lecture (`ack`). Il n'écrit pas dans un fil. Tout appel hors
plafond est refusé avec `capability_denied`, sans effet de bord.

## Exigences fonctionnelles (158a)

**Vue globale**

- **FR-001** : `bridget control delegations [--state active|all] [--limit N] [--json]` liste les tâches natives de tous les propriétaires, de la plus récente à la plus ancienne, sans fil T3. Défaut : `--state active`, limite 50, maximum 100.
- **FR-002** : Chaque ligne donne : `task_id`, propriétaire, enfant, état, âge, échéance restante, `parent_task_id`.
- **FR-003** : La lecture exige la négociation `ControlStateV1` ET un acteur humain attesté (`control_client_actor`). Pour une requête correctement formée : négociation absente → `negotiation_required` ; capacité absente → `capability_not_negotiated` ; sinon acteur humain absent → `human_actor_required`. Raison : `ControlStateV1` est accordée à tout client demandeur. Réemployer les gardes de négociation puis `control_client_actor`, jamais un alias `from=humain`.
- **FR-004** : `bridget control delegations cancel <task_id>` annule par le chemin humain existant (`human_cancel`). Terminal interactif et acteur humain exigés.
- **FR-005** : La liste ne montre jamais le corps de la mission ni le résultat.

**Arrêt global**

- **FR-006** : `halt-delegations` pose un état persistant d'arrêt. Seul un principal humain attesté peut le poser ou le lever, avec terminal interactif.
- **FR-007** : La décision d'admission d'une délégation native est unique et vit dans `referent_control.rs`. Elle compose l'état de pause et l'état d'arrêt. Les quatre combinaisons sont testées. Si l'arrêt est posé, le motif est `delegations_halted` (prioritaire sur `pause`).
- **FR-008** : Pendant l'arrêt, `bridget_delegate` et une nouvelle ronde `bridget_task_continue` avec une nouvelle clé rendent `delegations_halted`, sans tâche créée et sans clé consommée. Le rejeu d'une clé admise précède l'admission et rend la vue courante de la tâche existante, sans mutation. Pour Delegate et Continue, `suspended` reflète la pause courante ; ce n'est pas une réponse immuable. Seuls les reçus de `DelegationHaltSet` rendent une réponse historique. `bridget_task_status`, `bridget_capabilities` et les annulations restent disponibles.
- **FR-009** : `--cancel-active` annule, à la première application de la commande seulement, chaque arbre actif existant à cet instant, par `human_cancel`. Sans ce drapeau, aucune tâche active n'est touchée. Un rejeu (même `command_id`) n'ouvre aucun parcours d'annulation : il rend la réponse enregistrée. Séquence A (arrêt + annulation), B (reprise), C (nouvelle tâche), rejeu de A : C reste active, avant et après redémarrage. L'annulation est au plus une fois : si le daemon tombe entre l'enregistrement et la fin de l'annulation, l'humain relance `delegations cancel`. Un refus `store_unavailable` peut survenir avant ou après le commit : l'arrêt ou les annulations ont pu être appliqués. Vérifier la vue courante, puis annuler les tâches restantes par `delegations cancel` ; le rejeu du reçu n'émet aucune nouvelle annulation.
- **FR-010** : La pose et la levée sont idempotentes par `command_id`, avec une table de reçus bornée (256 commandes), distincte de l'état. Le reçu contient la réponse rendue : état historique et `cancelled`, nombre d'arbres actifs visés par la demande d'annulation, figé avant le commit. Ce nombre ne prouve pas un arrêt physique confirmé. Séquence A (arrêt), B (reprise), rejeu de A : l'état reste « repris » et la réponse est celle de A, marquée rejouée. Vrai aussi après redémarrage du daemon.
- **FR-011** : `bridget control status` affiche l'arrêt, son auteur, sa date et son motif. Si les données d'arrêt sont indisponibles, la commande sort avec le code 1, même si elle a déjà affiché l'état de contrôle087 ; cet affichage partiel ne vaut pas succès.
- **FR-012** : L'arrêt est indépendant de la pause. Aucune ne lève l'autre.

**Portées MCP**

- **FR-020** : Une table unique associe chaque outil MCP à une portée de base : `reads`, `writes_own_state`, `sends`, `writes_threads`, `delegates`, `publishes`. Pour les outils à actions, des listes d'actions abaissent la portée à `reads` ou à `writes_own_state`. La portée peut dépendre d'un argument de cible (`to`, cible d'un réveil).
- **FR-021** : Aucun outil MCP n'a de portée « contrôle humain ». Cette portée n'existe pas dans le type. Un test de surface vérifie qu'aucun outil ne peut poser l'arrêt.
- **FR-022** : Classement par action requis pour `bridget_thread` (lecture : `list`, `show`, `history` ; état propre : `read`, `ack` ; écriture : `create`, `add_members`, `post`, `close`), `bridget_events` (lecture : `types`, `list` ; état propre : `sub`, `unsub`), `bridget_handoff` (`preview` en lecture), `bridget_ledger` (lecture). `bridget_journal` a pour base `sends` (il partage l'extrait par message quand `to` est présent, `mcp.rs:549-557`) et est abaissé à `reads` quand `to` est absent. Son `readOnlyHint` est faux.
- **FR-023** : Un test exhaustif vérifie : chaque outil de `tools()` a une entrée ; chaque entrée désigne un outil présent ; `readOnlyHint` vaut vrai si et seulement si la portée de BASE est `reads` (la base est la portée la plus haute de l'outil ; un abaissement par action ou argument ne change pas l'indice).
- **FR-024** : Plafond « lecture seule » : un appelant dont une tâche native a une posture effective `discovery` (`effective_posture`, sinon la posture demandée) ne peut appeler que les portées `reads` et `writes_own_state`. Le plafond dure tant que l'enfant n'est pas nettoyé (`cleanup_done`), y compris entre deux rondes. Une posture omise suit `effective_posture`. Le MCP obtient la décision par la requête interne `CallerScope {}` dans l'enveloppe native existante, avec une identité inscrite. Le daemon utilise son index d'enfant et rend un booléen explicite. Aucune lecture de DB côté client. Toute panne RPC, réponse invalide ou erreur du magasin refuse sans effet chaque appel avec un effet ou une lecture du daemon ; aucun repli vers le plafond standard. Exception pure locale : `bridget_handoff` avec `action=preview` réemploie le parseur strict et le rendu existants avant le RPC. Il ne lit aucune DB, ne crée aucun identifiant et ne transporte rien. Sa portée `reads` est permise pour toutes les postures ; il n'a donc pas besoin de lire la tâche ni son plafond. Cette exception ne s'étend à aucun autre appel de lecture dépendant de la socket.
- **FR-025** : Les outils ajoutés par les sessions 151 à 157 sont classés avant la clôture de 158a. Le test FR-023 le garantit.
- **FR-026** : La lecture d'un fil par un enfant `discovery` suit la séquence lecture, `ack`, page suivante. Elle ne rejoue jamais la même page.
- **FR-029** : Politique explicite du plafond `discovery`, décidée ici et non laissée à l'implémentation. Permis : lecture, état propre, `bridget_read_artifact`, lecture de fil avec `ack`, et la réponse finale automatique au parent (chemin du wrapper, `wrapper.rs:6184`, hors outils MCP). Refusés (ruptures voulues, documentées dans la skill) : `post`, `create`, `add_members`, `close` de fil ; `bridget_journal` avec `to` (partage) ; `bridget_publish_artifact` ; `bridget_send`, `bridget_cancel`, `bridget_handoff` en envoi ; toute délégation. Raison : `discovery` est une posture de lecture seule ; son résultat part par la réponse automatique. Un besoin contraire (publier un artefact depuis `discovery`) demande une nouvelle spec, pas un élargissement en cours d'implémentation. Tests : la réponse automatique et la lecture d'artefact fonctionnent ; la publication et le partage sont refusés sans effet.

**Compatibilité**

- **FR-027** : 158A ne change pas le schéma ni les octets canoniques de `bridget_delegate` de la base157 livrée, avec ses ajouts151–157, dont `fallback` et `substitution`. Hors arrêt posé, un appel sans ajout158 garde le comportement de cette base. Un appelant sans tâche `discovery` garde tous ses outils.
- **FR-028** : Le nouveau refus (`delegations_halted`), la CLI et les portées sont documentés dans `docs/delegation-native.md` et dans la skill Bridget. L'ADR 158 note l'extension de la garde de l'ADR 027.

## Critères de succès (158a)

- **SC-001** : Avec 3 propriétaires et 10 tâches, `control delegations` liste les 10 sans paramètre T3. Un client agent négocié est refusé.
- **SC-002** : 100 % des délégations avec nouvelle clé pendant l'arrêt sont refusées `delegations_halted`, 0 tâche créée.
- **SC-003** : Avec `--cancel-active`, 0 tâche active ne reste après l'annulation en cascade, dans le scénario de test. Cette mesure du scénario exclut un crash, un refus de stockage ou un échec d’arrêt physique ; le reçu cancelled ne confirme pas ces arrêts.
- **SC-004** : Aucun outil MCP ne peut poser ou lever l'arrêt.
- **SC-005** : La séquence A, B, rejeu de A laisse l'état « repris », avant et après redémarrage.
- **SC-006** : Le test exhaustif de la table échoue si un outil est ajouté sans portée (outil factice dans le test).
- **SC-007** : Un enfant `discovery` voit 100 % des appels d'écriture refusés, 0 effet de bord, et lit un fil complet par la séquence lecture, `ack`.
- **SC-008** : Hors arrêt posé, un appel sans ajout158 garde le comportement de la base157 livrée. Les filtres de non-régression existants148–150 sont comparés à la baseline fraîche158 ; aucun succès global n'est déduit de cette préparation.

## Entités et hypothèses (158a)

**Entités** : État d'arrêt (une ligne) ; Reçu de commande d'arrêt (borné) ; Portée d'outil ; Plafond d'appelant (calculé).

**Hypothèses**

- Le plafond vient de la tâche native de l'appelant, retrouvée par nom d'enfant. Le plan mesure ce chemin avant le code (T017).
- `issuer_scope` est déclaré par le client. L'attestation humaine garde la borne de l'ADR 011 : elle arrête un agent qui passe par ses outils, pas un processus hostile du même compte.
- Le temps d'une tâche est borné par `reply_timeout` (150). Les tokens et les tours ne sont pas bornés par 158a.
- La mention « budgets de tokens ou de coût (session158) » de la spec 150 est levée par cette spec (voir ADR 158).
- Tests : `umask 077`, environnement privé, racine courte. Livraison Git/disque et nettoyage par ROOT après les gates du pipeline utilisateur ; activation Bridget seulement sous la condition déjà autorisée de zéro mission non terminale. Aucune relanceT3.

## Questions pour l'utilisateur

1. **Arrêt global : annuler les tâches actives par défaut ?** Défaut : non. `--cancel-active` est explicite. Raison : l'annulation détruit du travail.
2. **Plafond lecture seule pour les enfants `discovery` dès 158a ?** Défaut : oui, selon la politique FR-029. Ruptures : pas de `post`, pas de partage de journal, pas de publication d'artefact. Le résultat part par la réponse automatique.
3. **Budget de tours : le traiter dans une session propre ?** Défaut : oui, après un compteur de tours attesté pour Claude et Codex (identifiant de tour, adaptateur Codex, agrégation de toutes les rondes). 158a ne livre pas de borne trompeuse.

## Hors périmètre (158a)

- Budget de tours et de tokens (question 3).
- Réveils, `bridget wake`, rejeu des faits, secrets : 158b.
- Suspension des missions pendant une pause (152).
- Authentification forte, OAuth, jetons par session (T3 D) : borne ADR 011 inchangée.
- Interface graphique, service web.
- Historique d'audit de l'arrêt au-delà de l'état, des reçus bornés et du journal du daemon.

## Base158A vérifiée par le principal

Date : 2026-10-12. Branche : session-158-reveils-controle. Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle. Base Git : bd37e927eee0b28f058df9d6a6992e38665a6ad0. Sources157 : 8f21dfd7b553e822fcabe0a0e7eabc6741a8e45b. Reçu canonique157 : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/validation/livraison157.md. SHA256 : 6ac3d08948f39d6999b0a5c425235a70a1ad7d99d433866887abbdf52a86b705. Intégration, push, activation et nettoyage157 vérifiés. Les déclarations de préparation ci-dessus restent historiques ; cette vérification lève leur prérequis de livraison.


---

# Partie 158b - Réveils, déclencheur local, rejeu, secret (spec seule)

Pas de plan détaillé ni de tâches. 158b sera planifiée après 158a.

## Besoin (158b)

Un agent ne peut pas se faire réveiller à heure fixe. Aucun planificateur
n'existe dans le daemon. Les abonnements `bridget_events` ne rejouent rien : la
reprise annonce « faits survenus pendant la coupure perdus, non rejoués »
(`observation.rs`, `owner_returned`). Quand l'abonné est absent, en ne pas
déranger, ou que la file de 64 est pleine, la notification est perdue et
comptée (`daemon.rs:1470-1495`, `observation_lost`).

## Scénarios (158b)

### US5 - Réveil programmé par un agent (P2)

Un agent programme un réveil récurrent (`interval`, minimum 60 s) ou à heure
fixe (`HH:MM`, jours de semaine), lié à lui-même, à un autre agent ou à un fil.
À l'échéance, la cible reçoit un message système avec le libellé. Une heure
fixe manquée de plus de 10 minutes est ignorée et reprogrammée. Un intervalle en
retard rattrape une seule fois (règles de T3, `scheduledTasks/Schedule.ts`).

### US6 - Déclencheur externe local (P2)

Un script (launchd, cron, ssh) tape `bridget wake <agent|fil> [--label T] [--id K]`
par la socket Unix. Limite : 60 appels par minute. Déduplication par `--id`.
Aucun port réseau.

### US7 - Rejeu des faits manqués (P2)

Quand une notification d'observation ne peut pas partir, le fait est gardé
dans une table bornée. Au retour de l'abonné, un seul message résume les faits
manqués. `bridget_events action=missed` les relit. Un fait que le daemon n'a
jamais vu n'est pas reconstruit. La lacune est datée et annoncée.

### US8 - Demande de secret sans exposition (P3, reportée)

L'humain saisit le secret hors de l'agent (terminal interactif, sans écho).
L'agent reçoit une référence à usage unique, limitée au projet, 24 h (modèle T3,
`secrets/SecretRequests.ts`). **Condition préalable** : un consommateur côté
Bridget qui prend la référence sans la montrer à l'agent. Il n'existe pas. Cette
histoire reste spécifiée et non planifiée.

## Exigences (158b, aperçu)

- **FR-101** : table `wake_schedules` (propriétaire, cible, type, intervalle ou heure, libellé ≤ 2048 octets, prochaine échéance). Limites : 16 par propriétaire, 128 au total.
- **FR-102** : un réveil est un effet autonome. Il passe par la décision commune d'admission de `referent_control.rs` (nouvelle variante `ScheduledWake`). Pendant la pause, il est différé, puis rattrapé selon US5.
- **FR-103** : identifiant d'occurrence déterministe `wake:<schedule>:<échéance>`. Un redémarrage ne double pas un réveil remis.
- **FR-104** : outil MCP `bridget_wake` avec portée dépendante de la cible. Lister et annuler ses propres réveils, ou programmer un réveil vers soi : `writes_own_state`. Programmer vers un autre agent ou un fil : `sends`, avec vérification des droits de projet (comme `bridget_send`) et de l'appartenance au fil AVANT l'admission. Un agent ne gère que ses réveils. L'humain peut tous les lister et les annuler.
- **FR-105** : `bridget wake` (CLI) réveille seulement un agent ou un fil connu du registre.
- **FR-106** : table `observation_missed` bornée (64 par abonné, 7 jours). Un résumé par retour d'abonné.
- **FR-107** : toute lacune de couverture est annoncée avec ses dates, jamais comblée.

## Webhook HTTP : hors périmètre, justification

Un webhook demande un port TCP et une vérification HMAC exposée. Bridget garde
la règle : socket Unix seulement, aucun HTTP public. Un écouteur, même sur
`127.0.0.1`, ajoute une surface réseau, un secret HMAC à stocker et un risque de
requête locale forgée. `bridget wake` couvre le besoin : un relais externe
reçoit le webhook et appelle la commande locale. Bridget ne livre pas ce relais.

## Risques 158b

- Fuseau horaire des heures fixes : fuseau du daemon, annoncé.
- Réveil d'un agent hors ligne : report borné, puis `missed`, jamais de relance de processus.
- Occurrences manquées pendant une pause : règle de rattrapage de US5, pas reprise de mission (152).
