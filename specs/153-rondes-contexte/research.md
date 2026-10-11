# Recherche - Session153 rondes multiples et contexte transmis

Date : 2026-10-11. Les constats R1–R9 et leurs numéros de lignes ci-dessous
sont historiques (préparation avant livraison152). Source initiale : lecture du checkout principal non modifié
`/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`, état 149) et des
documents de la session 150. Les lignes citées sont celles de cette base. Le code
du worktree 150 n'a pas été lu : les éléments livrés par 150 sont cités par leur
spec (`specs/150-socle-delegation/`) et seront relus en T001.

## R1 - Le cycle de vie actuel arrête l'enfant dès la remise du résultat

- **Constat** : quand `result_available` et `result_sent` sont vrais et que
  `cleanup_done` est faux, `tick` ajoute la tâche aux nettoyages
  (`native_delegation.rs:1079`, `:1283`). `cancel_tree` arrête l'enfant, puis
  pose `cleanup_done = true` (`:1380-1480`, pose à `:1475`).
- **Conséquence** : garder l'enfant vivant demande de retarder ce nettoyage.
  Deux gardes suffisent (`:1079`, `:1283`).
- **Décision** : une garde `idle_deadline_at` dans la tâche. Tant qu'elle est
  posée, `tick` ne nettoie pas.

## R2 - Aucun nouvel état possible

- **Constat** : `task_entry` refuse tout état hors de neuf valeurs
  (`delegation_lineage.rs:35`) et le contrat 149 dit « un statut inconnu ferme le
  contrat ».
- **Décision** : l'enfant inactif est `result_available` avec `idle_deadline_at`.
  La ronde suivante repasse la tâche en `mission_pending` puis `working`, états
  déjà connus.
- **Risque** : la projection montre une tâche qui quitte un état « terminal ».
  Non vérifiable dans ce dépôt (lecteur T3). Parade : mutation visible normale,
  donc relecture du lecteur ; test T020.

## R3 - Une mission par tâche : la ronde remplace la mission

- **Constat** : l'index unique `native_delegations_mission` porte sur
  `$.mission` (`delegation.rs:77`). `for_mission` (`delegation.rs:195`) retrouve la
  tâche par la mission corrélée. `capture_reply` s'en sert
  (`native_delegation.rs:771-808`). La consigne vient de `request.task`
  (`:1134`).
- **Décision** : une ronde = un nouvel UUID dans `task.mission`. L'ancienne
  mission quitte l'index (elle n'est plus la valeur courante). Une réponse tardive
  à l'ancienne mission ne retrouve plus la tâche : elle est ignorée comme un
  message ordinaire. La consigne de la ronde courante est portée par
  `round_body`.
- **Correction (contre-revue, objection 2)** : l'affirmation « ignorée comme un
  message ordinaire » est fausse et dangereuse. Quand `capture_reply` rend `false`,
  le message part au dispatch ordinaire et réveille le parent, hors du mécanisme
  de résultat (`daemon.rs:8724` puis dispatch ; `daemon.rs:17562-17578`). Avant 153,
  une réponse tardive à une mission de tâche existante était absorbée (`Ok(true)`,
  `native_delegation.rs:771-808`). Décision : index `native_delegation_past_missions`
  (R14).
- **Alternative rejetée** : une nouvelle tâche liée par `continues_task_id`.
  Elle casse les invariants indexés par `child` (`for_child`,
  `delegation_lineage.rs:121`) et la lignée.

## R4 - L'identifiant du résultat doit changer à chaque ronde

- **Constat** : le résultat part avec l'identifiant fixe
  `native-result-<task_id>` (`native_delegation.rs:1216`), vérifié à
  `authorize_retained_result` (`:680`). L'envoi est idempotent par
  `(issuer_scope(task_id), message.id)`. Un second résultat avec le même
  identifiant serait un rejeu muet.
- **Décision** : ronde 1 garde `native-result-<task_id>` (octets 149). Ronde N à
  partir de 2 : `native-result-<task_id>-r<N>`. Une fonction unique
  `result_message_id(task)` sert aux trois lieux : `tick`, l'autorisation de
  résultat retenu, et la lecture de phase `acked` de SPEC-150 (T010 de 150).
- **Avis d'échec** : inchangé. Un échec termine la tâche : un seul avis par tâche.

## R5 - Les dates de la tâche sont collantes

- **Constat** : `stamp` conserve `completed_at` et `started_at` une fois posés
  (`delegation_lineage.rs:56-64`). Une sauvegarde ordinaire ne peut donc pas
  remettre `completed_at` à vide.
- **Conséquence** : l'âge d'un arbre pour la purge 150 (max de `completed_at`)
  serait faux après une ronde 2.
- **Décision** : une méthode de magasin dédiée `begin_round` réécrit la charge
  utile avec `completed_at = None`, dans une transaction avec l'incrément de
  projection (même schéma que `human_cancel`, `delegation_lineage.rs:225-250`).

## R6 - Les gardes de limites comptent les tâches actives

- **Constat** : l'admission 149 compte `!task.terminal()`
  (`native_delegation.rs:441-451`). `terminal()` est vrai pour `result_available`
  (`delegation.rs:60-65`). Un enfant inactif n'est donc pas compté.
- **Décision** : un prédicat « tient une place » = non terminale OU (garde prévue
  et nettoyage non confirmé). Lever la garde ne libère pas la place : un enfant
  vivant dont l'arrêt a échoué reste compté (contre-revue, objection 6 ;
  `cancel_tree` laisse `cleanup_done` faux quand un arrêt échoue,
  `native_delegation.rs:1444-1476`).
  SPEC-150 remplace ce comptage par des requêtes indexées (T013 de 150) : 153 y
  ajoute la clause garde posée. Une ronde n'ajoute aucune tâche : elle ne
  dépasse jamais 16 enfants par instance ni 128 au total.
- **Profondeur** : inchangée. La ronde ne crée pas de lignée.

## R7 - Interaction avec la purge 150

- **Constat** (spec 150, R2) : un arbre est purgeable si chaque tâche est
  terminale et `cleanup_done`, et si aucun sous-agent n'est connecté ou relançable.
- **Conséquence** : un enfant inactif a `cleanup_done = false` et une présence
  connectée. L'arbre n'est donc pas purgeable. La purge de borne ne peut pas
  tuer un enfant gardé. Une garde ne peut pas saturer la rétention : elle compte
  dans les 128 tâches actives.
- **Décision** : aucune règle neuve dans la purge, mais un test explicite et
  une condition de ceinture (`idle_deadline_at.is_none()`) dans le prédicat.
- **Remise de la ronde précédente (corrigé, contre-revue objection 1)** : effacer
  `delivery_acked` à la ronde N+1 oubliait un résultat N indéterminé dès que N+1
  était accusé, ce qui viole l'invariant de 150 (spec 150, R2). Décision : preuve
  par ronde. `begin_round` ajoute N à `unacked_rounds` si `delivery_acked` est
  faux. La recopie de 150 retire les rondes accusées. La purge exige
  `unacked_rounds` vide. Exiger l'accusé avant la ronde est rejeté : le message
  de résultat n'est remis au parent qu'à la fin de son tour, donc le parent ne
  pourrait pas enchaîner les rondes dans un même tour.

## R8 - Arrêt de l'enfant : où lever la garde

- **Constat** : `cancel_tree` collecte les noms sous le verrou, puis arrête hors
  verrou, puis marque `cleanup_done` (`native_delegation.rs:1380-1480`).
  `descendants_busy` ignore une tâche terminale
  (`native_delegation.rs:1297-1300`). `prepare_restart` arrête toutes les
  entrées désirées `Running` (`:947-949`).
- **Décisions** :
  - la garde d'un descendant est levée dans la phase sous verrou de
    `cancel_tree` (sinon une ronde pourrait s'ouvrir pendant l'arrêt) ;
  - `prepare_restart` lève toute garde (l'enfant est arrêté par la
    réconciliation, il ne peut plus recevoir de mission) ;
  - `Cancel` (`:518-530`) lève la garde d'une tâche inactive au lieu de ne rien
    faire ; `human_cancel` (`delegation_lineage.rs:235`, `drives = !terminal`) fait
    de même ;
  - un enfant inactif ne bloque pas le résultat de son parent : il est terminal.
    Le nettoyage du parent l'arrête. Un parent qui veut garder l'enfant doit
    avoir fini avec lui avant sa propre fin.
- **Invariant I1** (verrou d'état unique) : la garde est levée sous le verrou
  avant que la tâche soit ajoutée aux nettoyages. Une ronde ne s'ouvre que sous le
  verrou et seulement si la garde est posée. Les deux ne se croisent donc jamais.

## R9 - Brief : réutiliser le dossier de passation, pas l'historique

- **Constat** : `handoff::validate_and_render` valide un brouillon strict
  (champs inconnus et `null` refusés, 12 éléments par liste, 16 références,
  corps rendu ≤ 16 384 octets, aucune troncature) et rend un corps déterministe
  (`handoff.rs:460-493`, bornes `:14-24`). Les références peuvent citer un fichier, une URL, un message, une
  tranche de journal, un fil ou un artefact (`handoff.rs:84-120`). Aucune source
  n'est lue (`handoff.rs:9-10`). Le schéma MCP existe (`mcp.rs:1977-2005`).
- **Constat** : `bridget_handoff send` envoie un message séparé
  (`mcp.rs:512-545`). Il n'est ni atomique avec la création de la tâche, ni
  ordonné avant la mission.
- **Décision** : `brief` = le même objet que `draft`. Même validateur, mêmes
  refus. Le corps rendu est préfixé à la consigne dans le corps de mission. Aucun
  nouveau validateur, aucune nouvelle structure.
- **Extrait de l'historique du parent : rejeté.** Le fournisseur (Codex,
  Claude) détient sa conversation. Bridget n'en a pas copie. Le parent peut
  citer une tranche de journal ou de fil par une référence `journal` ou `thread`.
  `bridget_journal` rend déjà des extraits sourcés bornés (64 Kio).
- **Taille** : brief 16 Kio + consigne 64 Kio = 80 Kio au plus. Le plafond de
  corps injecté par un wrapper est 120 000 caractères
  (`wrapper.rs:300`) : marge de 40 %. Test de borne en T006.

## R10 - Vérification des droits à chaque ronde

- **Constat** : `spawn_task` revérifie les révocations et les sources de la
  politique figée avant de lancer (`native_delegation.rs:535-553`). Une ronde
  n'appelle pas `spawn_task`.
- **Risque** : un grant humain retiré ou un fichier de politique modifié entre
  deux rondes laisserait l'enfant travailler avec l'ancienne politique.
- **Décision actualisée152** : `native_launch_registry` existe désormais,
  appelé par lancement et reprise. Continue réutilise cette fonction pour
  révocations et sources. La comparaison de politique actuelle reste locale
  à Continue, avec `parent_fact` et `child_policy` ; aucun helper extrait neuf.
  Comparaison complète, posture demandée conservée, refus existants.

## R11 - Pause

- **Constat** : SPEC-150 ajoute `AutonomousEffect::NativeDelegation` et refuse
  `referent_paused` à l'admission d'une nouvelle clé. `Continuation`
  (`referent_control.rs:113`) et `ProjectRound` (`:109`) désignent d'autres puits
  (continuation d'exécution, ronde de projet). Rien à voir avec les rondes de
  tâche.
- **Décision** : `Continue` utilise `NativeDelegation`. Le rejeu précède la
  garde de pause, comme en 150.
- **Limite** : le délai d'inactivité n'est pas gelé pendant une pause en 153.
  Raison actualisée152 : le journal de pause (`pause_overlap`) détermine le
  gel des échéances métier non terminales à la lecture, sans réécrire les dates
  brutes. `suspended` est dérivé ; `suspended_at` n'existe pas. Une tâche
  inactive est terminale. Un enfant
  peut donc expirer pendant une longue pause. Aucun résultat n'est perdu (déjà
  remis). Le parent peut déléguer de nouveau. Geler aussi cette garde demanderait
  de mesurer la durée de pause pour des tâches terminales : rejeté (Art. XIX).
- **Reprise 152** : une ronde N >= 2 en `working` est reprise par la même
  mécanique que la ronde 1, car chaque ronde a son exécution `execution-<mission>`
  et sa chaîne (`latest_in_chain`). Une tâche inactive n'est pas reprise après
  redémarrage (FR-013 de 152 ne vise que `starting`, `mission_pending`, `working`) :
  153 lève sa garde (FR-014).

## R12 - Doctrine T3 et écart

- **T3** (`research-t3-bridget.md`, A5 et A9) : l'enfant reçoit seulement le
  prompt ; une tâche par ronde, tout le contexte dans le brief, clé distincte par
  ronde, jamais de relance par envoi dans le fil enfant. T3 transfère du contexte
  par fork ou par résumé portable (budget 16 000 jetons, références vers
  l'historique omis).
- **Reprise** : clé distincte par ronde ; brief structuré ; jamais de relance
  hors de l'outil de ronde.
- **Amélioration** : mémoire conservée (même processus) ; brief delta ; ronde
  idempotente et plafonnée ; droits revérifiés ; fin explicite et délai borné.
- **Limite honnête** : un enfant qui garde sa mémoire garde aussi ses biais. Pour
  une contre-revue indépendante, la bonne pratique reste une nouvelle délégation
  sans garde. La documentation le dit.
- **Baseline research (Art. IX)** : fonctionnalité de délégation, pas d'IA
  générative nouvelle. Consultation des notes de la comparaison T3/Bridget faite.
  Pas de recherche web : sans objet pour un mécanisme interne.

## R14 - Contre-revue Codex (ronde 1) : points retenus

Voir `adversarial-review-codex.md`. Effets sur la recherche :

- **Droits actuels** : `parent_fact` relit l'attestation actuelle du parent
  (`native_delegation.rs:38-66`) et `child_policy` calcule la politique de
  l'enfant (`native_permissions.rs:206`). `spawn_task` ne compare rien à une
  politique vivante, car l'enfant naît à cet instant. Une ronde, elle, réutilise
  un enfant dont la politique est figée : le recalcul et la comparaison sont
  nécessaires (FR-031).
- **Échéance** : `tick` lève la garde échue, mais l'admission d'une ronde peut
  précéder le tick d'une seconde au plus (`daemon.rs:4700`). Le contrôle
  `now < idle_deadline_at` est donc refait à l'admission et dans `begin_round`
  (FR-032).
- **Attente 151 (réaligné sur 151 révisée)** : l'attente, la lecture d'état et
  l'attente de requête sont des lectures pures. Elles n'écrivent jamais
  `result_sent`, `delivery_acked` ni le nettoyage. `tick` est le seul écrivain de
  la remise. La garde se pose donc avec `result_sent` dans `tick` (code local).
  Conséquence : `continuable` devient vrai jusqu'à 1 s après la fin de ronde.
  Un `bridget_task_continue` émis aussitôt après une attente peut arriver avant
  l'envoi : refus réessayable `result_delivery_pending`.
- **Heure périmée (ronde 2, point 1)** : l'admission lit `now`, puis les
  revérifications de sources lisent des fichiers (`native_delegation.rs:551`). Une
  échéance peut être franchie entre les deux. `begin_round` relit l'heure.
- **Annulation avant l'envoi (ronde 2, point 2)** : `Cancel` ne fait rien sur
  `result_available` (`native_delegation.rs:524`) et `human_cancel` ne pilote pas
  une tâche terminale (`delegation_lineage.rs:235`). Avec une garde, `tick` poserait
  ensuite `idle_deadline_at` jusqu'à 2 h malgré l'annulation. `guard_released`
  rend la fin durable et couvre les deux phases.
- **Posture (ronde 2, point 3)** : `child_policy` traite `posture == Some(Discovery)`
  comme lecture seule sans réseau pour Codex vers Codex
  (`native_permissions.rs:208-213`). L'admission et `spawn_task` passent la posture
  demandée (`*posture`, ou `None` avec un snapshot). Le recalcul doit faire de même,
  pas utiliser la posture effective.
- **Index des missions passées** : lecture O(log n) par clé primaire. Un balayage
  `json_each` des tâches serait O(n) par message reçu (Art. XVIII).

## R13 - Complexité (Art. XVIII)

- `Continue` : une lecture par clé primaire O(log n) + un balayage des rondes
  déjà jouées, r ≤ 8 : O(log n + r).
- `capture_reply` sur mission passée : une lecture par clé primaire, O(log n).
- `tick` : la garde ajoute une comparaison par tâche en attente, déjà
  parcourue : O(p), p = tâches en attente (≤ 4096).
- Pas de boucle imbriquée sur une même collection. Pas de N+1.


## R15 — Réemploi frais après livraison152 (base ee03ebdc)

Passage unique de lecture du code WT153, avec cartes exactes dans
`validation/couplage.md` et arbitrages dans `reuse-audit.md`.
`deserialize_present_optional` préserve brief:null ; `validate_and_render`
est public(crate). L'index de quota garde son nom et son préfixe state : migration
Immediate de l'ancienne définition reconnue, pas d'index concurrent ni scan des
terminaux. `native_launch_registry` existe ; comparaison complète de politique
actuelle locale à Continue. Schéma handoff Value réutilisé aux deux adaptateurs.

`begin_round` est nécessaire car `save_projected` conserve les dates terminales
et l'accusé151. Remettre uniquement les champs de ronde à zéro ; conserver
`restart_resumes` (plafond2 vie entière), `launch_issued_at` et `started_at`.
Ronde1 garde l'identifiant historique ; les quatre consommateurs de résultat
doivent consulter la ronde courante, y compris la boîte151. ACK151 de lecture et
ACK150 de livraison restent deux preuves distinctes.

La reprise152 des rondes actives, les refus d'identité/CAS/stockage et les sources
attestées sont préservés. `pause_overlap` gèle l'échéance métier, pas la minuterie
de l'appelant151 ni la garde terminale153. Coûts de corps O(B) et recontrôles de
politique/registre/sources à compter séparément ; O(log n + r) ne décrit que les
lectures et le rejeu local, r≤7. Coût quota O(log n + a), a≤128, total150 séparé.
Aucune performance mesurée ni implémentation153 déclarée par cette lecture.
