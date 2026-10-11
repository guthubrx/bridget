# Session153 - Rondes multiples et contexte transmis

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 153-rondes-contexte
Titre: Rondes multiples sur un même enfant et brief de contexte structuré
Statut: Implemented
Priorité: P1
Tâches: 29/29
Tests:40 natifs +1 E2E PASS ;179 références lib PASS sur binaire final ;16 intégrations maintenues par preuve/impact. Workspace R1 :2145 PASS/10 FAIL/72 ignorés avant correctifs ciblés, aucun GREEN global. Mémoire réelle non vérifiée.

Résumé:
- Contexte: Un enfant natif est arrêté dès que son résultat est remis (`cancel_tree` au nettoyage). Une revue suivie de corrections puis d'une nouvelle revue oblige à créer un nouvel enfant, sans mémoire. L'enfant ne reçoit qu'une consigne en texte libre.
- Objectif: Permettre à un parent de garder un enfant vivant pour plusieurs rondes, avec une fin explicite et un délai d'inactivité borné. Permettre au parent de transmettre un brief structuré et borné.
- Dépendances: SPEC-150 (rétention, pause, délai), SPEC-151 (résultat structuré, attente), SPEC-152 (reprise), SPEC-148, SPEC-149, SPEC-103 (dossier de passation)

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-153-rondes-contexte`
**Created**: 2026-10-11
**Status**: Implemented
**Priority**: P1
**Dependencies**: SPEC-150, SPEC-151, SPEC-152

Date : 2026-10-11. Accord utilisateur : feuille de route « Bridget cinq étoiles »
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`). Critère
visé : « Contexte et rondes », note 2 vers 5 (`specs/150-socle-delegation/research-t3-bridget.md`).

## Besoin et périmètre

Un parent qui délègue une revue veut souvent trois rondes : revue, corrections,
nouvelle revue. Aujourd'hui, Bridget ne le permet pas bien. Deux défauts.

**Défaut 1 - L'enfant meurt avec son résultat.** Quand le résultat est remis au
parent, le nettoyage arrête l'enfant et ses descendants
(`crates/bridget-daemon/src/daemon/native_delegation.rs:1079`, `:1283`, puis
`cancel_tree` `:1380-1480`). La ronde suivante exige une nouvelle délégation.
Le nouvel enfant ne se souvient de rien. Une seule mission existe par tâche : la
clé unique `$.mission` (`crates/bridget-daemon/src/delegation.rs:77`) et la
consigne unique `request.task`.

**Défaut 2 - Contexte en texte libre.** L'enfant reçoit `task` (64 Kio) et une
carte de reprise du dépôt (`crates/bridget-daemon/src/wrapper.rs:303-392`). Rien
ne structure le contexte (objectif, constats, décisions, questions, références).
Un dossier de passation validé existe déjà (`bridget_handoff`, SPEC-103,
`crates/bridget-daemon/src/handoff.rs:460`, JSON borné à 16 Kio). Il n'est pas
relié à la délégation : il part dans un message séparé.

**Doctrine reprise de T3.** Une tâche par ronde, une clé distincte par ronde,
tout le contexte dans le brief. T3 relance un enfant sans mémoire à chaque ronde.

**Ce que Bridget fait mieux.** L'enfant reste le même processus : il garde sa
mémoire. Le brief d'une ronde suivante ne transporte que le changement. Chaque
ronde a sa clé de rejeu, son échéance et son résultat. Les limites 16 enfants,
profondeur 8 et rétention 150 restent en vigueur. Le droit de l'enfant est
revérifié à chaque ronde.

**Hors périmètre** : voir la section dédiée.

## Scénarios utilisateur

### US1 - Brief structuré à la délégation (P1)

Un parent appelle `bridget_delegate` avec un champ `brief` : un brouillon de
dossier de passation (objectif, résumé, résultats, décisions, questions,
prochain pas, références, limites). Bridget le valide avec le même validateur
que `bridget_handoff` et le place avant la consigne dans la mission. Un brief
invalide ou trop gros (plus de 16 Kio rendu) est refusé avec `brief_invalid`,
sans tâche créée, sans troncature. Sans `brief`, la mission est identique à 150.

### US2 - Garder l'enfant vivant et le terminer (P1)

Le parent délègue avec `keep_alive_secs`. Quand le résultat de la ronde 1 est
remis, l'enfant reste connecté. La tâche reste en `result_available` et indique
un `idle_deadline_at`. L'enfant inactif compte dans les limites actives. Le
parent peut terminer l'enfant à tout moment avec `bridget_task_cancel`. Un humain
peut le faire avec l'arrêt Lineage. Sans action, l'enfant est arrêté à
`idle_deadline_at`. La fin du parent, un redémarrage du daemon ou une
annulation de la lignée l'arrêtent aussi.

### US3 - Nouvelle ronde sur le même enfant (P1)

Le parent appelle `bridget_task_continue` avec `task_id`, une **nouvelle**
`request_id`, une consigne et un `brief` facultatif. Si l'enfant est inactif,
Bridget ouvre la ronde 2 : même tâche, même enfant, nouvelle mission, nouvelle
échéance. L'enfant répond avec sa mémoire de la ronde 1. Le résultat de la
ronde 2 est remis comme celui de la ronde 1, avec son propre identifiant de
message. Rejouer la même clé ne crée pas de ronde. Changer la consigne avec la
même clé donne `envelope_mismatch`. Une tâche en cours de ronde refuse
(`round_in_progress`). Une tâche terminée ou libérée refuse (`keep_alive_ended`).

### US4 - Limites, pause et rétention restent vraies (P1)

Un enfant inactif compte dans 16 enfants actifs par instance et 128 tâches
actives. Une ronde ne crée aucun enfant : elle ne dépasse jamais ces limites.
La profondeur 8 ne change pas. Pendant une pause du référent, une nouvelle
ronde est refusée (`referent_paused`) ; le rejeu d'une ronde admise reste
possible. La purge 150 ne supprime jamais un arbre qui contient un enfant
inactif vivant. Le plafond est de 8 rondes par tâche (`round_limit`).

### US5 - Aucun changement pour l'existant (P1)

Un appel sans `keep_alive_secs` ni `brief` produit les mêmes octets canoniques
et le même comportement qu'en 150 : l'enfant est arrêté après son résultat. Les
tâches créées avant la mise à jour se relisent sans erreur. La vue Lineage T3
garde ses neuf états.

## Exigences fonctionnelles

**Brief**

- **FR-001** : `bridget_delegate` et `bridget_task_continue` acceptent `brief`
  (objet). Le brief suit le schéma du brouillon de `bridget_handoff`.
- **FR-002** : Le brief est validé et rendu par `handoff::validate_and_render`
  (mêmes champs, mêmes bornes, mêmes refus). Le corps rendu fait au plus 16 384
  octets. Aucune troncature.
- **FR-003** : Un brief invalide rend `brief_invalid` avec le champ et la
  raison. Aucune tâche, aucune ronde, aucune clé consommée.
- **FR-004** : Le corps de la mission est : corps rendu du brief, ligne de
  séparation, consigne. Sans brief, le corps est la consigne seule, comme en 150.
- **FR-005** : Le brief fait partie de l'enveloppe idempotente.
- **FR-006** : Le brief reste une déclaration du parent. Bridget ne lit aucune
  source citée. Les avertissements du dossier (`sources_not_verified`, journal
  lisible plus largement que le destinataire) sont rappelés dans la
  documentation de l'outil.

**Garde de l'enfant**

- **FR-007** : `bridget_delegate` accepte `keep_alive_secs`, entier de 60 à
  7 200. Hors bornes : `keep_alive_out_of_range`, aucune tâche. Absent : aucune
  garde.
- **FR-008** : Le champ fait partie de l'enveloppe idempotente.
- **FR-009** : Avec garde, la remise du résultat ne déclenche pas le nettoyage.
  La tâche reste `result_available`. Elle porte `idle_deadline_at` = instant de
  la remise + `keep_alive_secs`. La garde se pose dans la même sauvegarde que `result_sent = true`, c'est-à-dire
  quand `tick` a envoyé le message de résultat (SPEC-151 : l'attente et la lecture
  d'état sont des lectures pures et n'écrivent jamais la remise).
- **FR-010** : Une tâche avec garde tient sa place (16 par instance, 128 au
  total) tant que son enfant n'est pas arrêté de façon confirmée : enfant inactif
  ET nettoyage non terminé (`cleanup_done` faux). Lever la garde ne libère pas la
  place. Seul l'arrêt confirmé la libère.
- **FR-011** : Quand `idle_deadline_at` est atteint, la garde est levée et le
  nettoyage existant arrête l'enfant et sa descendance. L'état et le résultat
  restent `result_available`.
- **FR-012** : `bridget_task_cancel` sur une tâche avec garde en `result_available`
  lève la garde (fin explicite), **que le résultat ait déjà été envoyé ou non**.
  L'arrêt humain Lineage fait de même. Le statut reste `result_available`. Sur une
  tâche en cours de ronde, l'annulation existante s'applique.
- **FR-033** : La fin explicite laisse une preuve durable (`guard_released`). Tant
  qu'elle est posée, `tick` ne pose jamais de garde et aucune ronde ne s'ouvre. Une
  annulation reçue avant l'envoi du résultat ne peut donc pas être annulée en
  silence par la pose de la garde qui suit.
- **FR-013** : Le nettoyage de la tâche parente arrête aussi ses enfants
  inactifs et lève leur garde.
- **FR-014** : Au redémarrage du daemon, toute garde est levée : l'enfant est
  arrêté par la réconciliation et ne peut plus recevoir de ronde.

**Rondes**

- **FR-015** : `bridget_task_continue(task_id, request_id, task, brief?)` ouvre
  la ronde suivante sur une tâche inactive du même parent. La consigne fait 1 à
  65 536 octets, comme à la délégation.
- **FR-016** : L'ouverture est atomique : nouvelle `mission`, résultat effacé,
  `result_sent`, `delivery_acked`, erreur et échéance remis à zéro, état
  `mission_pending`, `completed_at` remis à vide, `idle_deadline_at` vide.
  L'ancienne mission et, si son résultat n'est pas accusé, le numéro de sa ronde
  sont conservés dans la même transaction (FR-029, FR-030).
- **FR-017** : Chaque ronde utilise une mission distincte et une échéance propre
  (délai de la tâche, SPEC-150). Le résultat de la ronde N est remis avec un
  identifiant de message propre à la ronde.
- **FR-018** : Rejouer une `request_id` déjà admise pour cette tâche et la même
  enveloppe rend la tâche, sans nouvelle ronde, même pendant une pause. Une
  autre enveloppe rend `envelope_mismatch`.
- **FR-019** : Refus nommés, sans nouvelle ronde, mission ni clé consommée : `round_in_progress` (ronde en
  cours dont l’enfant reste attesté), `keep_alive_ended` (garde levée, tâche terminale ou enfant absent),
  `keep_alive_not_enabled` (tâche créée sans `keep_alive_secs`),
  `result_delivery_pending` (résultat prêt mais pas encore envoyé par `tick` :
  **refus réessayable**, attendre environ une seconde), `round_limit`
  (8 rondes atteintes), `referent_paused`, `task_unavailable` (autre parent ou
  tâche purgée). Les refus et les rejeux ne modifient pas la tâche. Seule exception :
  si la tâche appartient au demandeur et que sa garde a expiré ou que son enfant
  est absent, `keep_alive_ended` confirme durablement cette fin (`guard_released`
  vrai, `idle_deadline_at` vide), sans changer le résultat ni l'état métier.
  L’absence attestée prime sur `round_in_progress`, même si la ronde est
  encore active : elle confirme durablement la fin de garde.
- **FR-020** : Avant d'ouvrir une ronde, Bridget revérifie les droits du parent
  (révocations, grants, instance) et les sources de la politique figée de
  l'enfant, comme au lancement. Une source changée refuse la ronde avec le refus
  existant (`settings_revision_changed`).
- **FR-031** : Bridget revérifie aussi les droits **actuels** du parent : il
  relit l'attestation actuelle (ou le grant applicable pour une tâche sans
  preuve) et recalcule la politique que l'enfant recevrait aujourd'hui. Si la
  politique figée de l'enfant n'est pas identique, la ronde est refusée
  (`permission_not_inherited`, ou `permission_attestation_unavailable` sans
  attestation). Une réduction des droits sans révocation explicite suffit. Le
  recalcul reprend les entrées de la délégation : cible, dossier figé, et posture
  **demandée** à l'origine (y compris absente). Il n'utilise jamais la posture
  effective, qui changerait la politique Codex. Aucun champ de sécurité n'est
  retiré pour faire passer l'égalité.
- **FR-021** : L'échec d'une ronde (échéance, refus de remise, perte de l'enfant)
  termine la tâche en `failed`. Le nettoyage arrête l'enfant. Aucun nouvel essai
  automatique (SPEC-152 traite la reprise).
- **FR-022** : Une ronde ne crée ni ne change de posture, de dossier ni de
  modèle. Ces champs restent ceux de la délégation.

**Catalogue, lecture, compatibilité**

- **FR-023** : `bridget_capabilities` annonce `keep_alive_min_secs` (60),
  `keep_alive_max_secs` (7 200), `max_rounds` (8) et `brief_max_bytes` (16 384).
- **FR-024** : La lecture d'état (`bridget_task_status`) ajoute `round`,
  `idle_deadline_at` et `continuable` (booléen). `continuable` n'est vrai qu'après
  l'envoi du résultat par `tick` (cadence d'une seconde). Aucun nouvel état n'est créé.
- **FR-025** : Les payloads 149 et 150 se relisent sans erreur. Un appel sans
  nouveau champ garde ses octets canoniques.
- **FR-026** : La purge 150 ne supprime pas un arbre dont une tâche porte une
  garde ou dont un enfant est vivant.
- **FR-029** : Le résultat non accusé d'une ronde reste protégé de la purge après
  l'ouverture des rondes suivantes. Chaque ronde garde sa propre preuve d'accusé.
  Un arbre n'est purgeable que si toutes les rondes sont accusées.
- **FR-030** : Une réponse authentifiée à une mission d'une ronde passée est
  absorbée sans effet sur la tâche et sans réveil du parent, sur les deux
  chemins d'envoi (idempotent et ordinaire). Elle n'est jamais remise comme
  message ordinaire.
- **FR-032** : L'ouverture d'une ronde est refusée (`keep_alive_ended`) si
  l'instant courant est égal ou postérieur à `idle_deadline_at`. Le contrôle est
  fait sous verrou à l'admission et refait dans la transaction d'ouverture avec
  une heure **relue juste avant l'écriture** (les revérifications de droits
  précédentes peuvent durer), sans attendre le tick.

## Critères de succès

- **SC-001** : Avec garde, l'enfant est encore connecté 60 s après la remise du
  résultat de la ronde 1. Sans garde, il est arrêté comme en 150.
- **SC-002** : La ronde 2 s'exécute sur la même instance d'enfant que la ronde 1
  (même `child_instance`), sans nouveau lancement. Un fait donné en ronde 1 et
  absent du brief de la ronde 2 est restitué par l'enfant (recette réelle).
- **SC-003** : Une `request_id` rejouée 10 fois produit une seule ronde et une
  seule remise de mission.
- **SC-004** : Avec 16 enfants inactifs pour une instance, une 17e délégation est
  refusée (`task_limit`). Les rondes sur ces 16 enfants réussissent.
- **SC-005** : 0 arbre purgé tant qu'un enfant inactif vit, sur l'ensemble des
  scénarios.
- **SC-006** : À `idle_deadline_at`, l'enfant est arrêté au tick suivant (cadence
  d'une seconde, `crates/bridget-daemon/src/daemon.rs:4700`).
- **SC-007** : 100 % des rondes demandées avec une nouvelle clé pendant une pause
  sont refusées avec `referent_paused`.
- **SC-008** : Un brief valide produit le même corps rendu que
  `bridget_handoff` en aperçu. Un brief de plus de 16 384 octets est refusé,
  0 tâche créée.
- **SC-009** : Un appel sans nouveau champ produit la même tâche qu'en 150 (tests
  `native148`, `native149`, `native150` verts).
- **SC-010** : Le résultat de la ronde 2 est remis exactement une fois, avec un
  identifiant distinct de celui de la ronde 1.
- **SC-011** : Résultat de la ronde N indéterminé, ronde N+1 accusée : l'arbre
  n'est pas purgé. Il l'est après l'accusé de N.
- **SC-012** : Une réponse tardive à la mission de la ronde N, reçue pendant la
  ronde N+1, ne réveille pas le parent et ne modifie pas la tâche (deux chemins
  d'envoi, et rejeu).
- **SC-013** : Politique du parent réduite sans révocation : 100 % des rondes
  refusées. Politique inchangée : la ronde est admise.
- **SC-014** : `bridget_task_continue` à l'instant exact `idle_deadline_at`, avant
  le tick : refusé (`keep_alive_ended`), l'enfant n'est pas prolongé.
- **SC-016** : Annulation (agent et humaine) reçue pendant `result_available` avec
  `result_sent` faux : après l'envoi du résultat par `tick`, aucune garde n'est
  posée, l'enfant est arrêté et aucune ronde ne s'ouvre.
- **SC-017** : Échéance franchie pendant les revérifications d'une ronde : la
  ronde est refusée (`keep_alive_ended`). Posture demandée absente, parents
  natifs, conversions Codex et Claude : la ronde immédiate sans changement est
  admise.
- **SC-015** : Expiration ou annulation avec arrêt refusé : la place reste tenue,
  une nouvelle admission au plafond est refusée. Un `bridget_task_continue` émis
  entre la fin de ronde et l'envoi du résultat par `tick` rend
  `result_delivery_pending` (réessayable) et réussit après l'envoi.

## Entités et hypothèses

**Entités**

- **Tâche native** : identité et filiation conservées (SPEC-150). Sept champs s'ajoutent : `round`, `idle_deadline_at`, `round_body`, `rounds`, `guarded`, `guard_released` et `unacked_rounds`.
- **Ronde** : un cycle consigne, travail de l'enfant, résultat. Une tâche en a 1
  à 8. La ronde courante est portée par la tâche.
- **Garde** : droit de garder l'enfant vivant entre deux rondes, borné par
  `keep_alive_secs`.
- **Mission passée** : mission d'une ronde terminée, conservée (index) pour
  absorber les réponses tardives.
- **Enfant inactif** : tâche `result_available` avec `idle_deadline_at`.
- **Brief** : brouillon de dossier de passation v1, rendu en tête de mission.

**Hypothèses**

- SPEC-150 est livrée : `reply_timeout`, `delivery_acked`, purge par arbre,
  `AutonomousEffect::NativeDelegation`, comptages actifs indexés.
- SPEC-151 est livrée : résultat structuré, attente d'une tâche et boîte de
  fin. SPEC-153 s'y branche par la ronde courante. Les points de couplage sont
  vérifiés en T001.
- SPEC-152 est livrée : reprise d'une mission et chaîne d'exécutions lue par
  `latest_in_chain`. Le journal de pause et `pause_overlap` déterminent le gel
  des échéances métier sans modifier les dates brutes. Le statut `suspended`
  est dérivé ; aucun champ `suspended_at` n'existe. Une tâche inactive est
  terminale : son délai d'inactivité n'est pas gelé par une pause. Son résultat
  reste protégé quand l'enfant se termine pendant une pause.
- L'enfant garde sa mémoire parce que son processus fournisseur reste vivant.
  La taille de son contexte croît à chaque ronde. Le plafond de 8 rondes limite
  ce risque.
- Une revue indépendante (sans biais de la ronde précédente) reste un cas pour
  une nouvelle délégation sans garde.
- Les tests suivent la recette `umask 077` de la session 150.
- Le principal livre chaque session, puis nettoie son worktree. Il distingue
  les sources Git, les fichiers installés et le code actif. L'activation est
  coordonnée pour préserver les autres fils T3.

## Hors périmètre

- Extrait de l'historique du parent (le fournisseur garde cet historique, pas
  Bridget). Le parent cite des tranches de journal ou de fil dans les
  références du brief.
- Délai de réponse propre à une ronde (le délai de la tâche s'applique).
- Fork de contexte entre fournisseurs et fusion de retour (`merge_back` de T3).
- Reprise d'un enfant inactif après redémarrage (SPEC-152).
- Nouvel essai automatique d'une ronde échouée (SPEC-152).
- Attente synchrone d'une ronde (SPEC-151 : la ronde réutilise son mécanisme).
- Isolation de l'espace de travail (SPEC-154).
- Questions de l'enfant au parent pendant une ronde (SPEC-155).

## Choix retenus dans le contrat autorisé

1. `keep_alive_secs` de 60 à 7 200, sans valeur par défaut : absence = pas de garde.
2. Plafond de huit rondes par tâche.
3. Brief limité au dossier de passation. Aucun extrait automatique de l'historique
   du parent, que Bridget ne détient pas.
