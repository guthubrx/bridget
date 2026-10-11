# Contre-revue adverse - Session159

## Ronde 1 (plan « édition répartie chez les porteurs »)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Question** : sûreté d'une saga de retrait, de promotion et de déplacement répartie entre le daemon et les porteurs de file.
- **Verdict reçu** : BLOCKED (11 points, dont 4 critiques). La matrice de droits auteur / parent / humain est jugée cohérente.
- **Diagnostic de l'architecte** : les 11 points ont une racine commune. La file vit dans la mémoire de chaque porteur, donc toute édition est une saga entre deux processus. T3 évite cela parce que sa file est durable côté serveur. D'où l'étude d'une file tenue par le daemon (`research.md` R14).
- **Vérification** : chaque point est relu dans le checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`, non modifié).

| # | Point | Vérifiée comment | Sort | Raison |
|---|---|---|---|---|
| 1 | crash entre retrait mémoire et tombe : annulation perdue | lu le plan d'origine (retrait puis tombe, deux écritures dans deux processus) | **sans objet** | plus de retrait chez le porteur : un message retenu est une ligne de la base du daemon ; l'annulation est une transaction unique |
| 2 | erreur d'écriture après installation de la tombe : réinjection d'une remise `cancelled` (`fsutil.rs:87`) | même cause | **sans objet** | plus de tombe |
| 3 | idempotence porteur de `promote` / `reorder` non conçue | même cause | **sans objet** | les éditions sont des transactions du daemon ; le reçu `queue_edits` est écrit dans la même transaction que l'effet |
| 4 | promotion sans retour en file garanti (`wrapper.rs:973` rend `indeterminate` ; `codex_app_server.rs:1724` remet en file sur tout `Err`) | lu `wrapper.rs:757-776`, `claude_stream_json.rs:310-317`, `acp.rs:620-628`, `codex_app_server.rs:1700-1770` | **retenu, fermé par conception** | Claude et ACP rejettent un guidage. La promotion libère le message avec le marqueur `steer-or-queue` : un transport qui ne peut pas guider l'enfile en file ordinaire au lieu de le rejeter (T018). Un porteur qui n'a pas déclaré `queue_fallback` ne reçoit pas de promotion (`steer_unavailable`, le message reste retenu) |
| 5 | rang durable divergent de la file réelle ([A,B], A déplacé, C arrive : [B,A,C] chez le porteur, [B,C,A] au daemon) ; borne « au plus 2 » fausse | raisonnement vérifié sur `send_delivery.rs:86-140` | **sans objet** | une seule copie de l'ordre, dans la base du daemon. Le déplacement re-numérote les seuls messages retenus du destinataire dans une transaction ; une arrivée concurrente se sérialise derrière elle |
| 6 | éditions rattachées à l'ancienne instance après réaffectation (`daemon.rs:6311`) | lu `send_delivery.rs:214-245` : la réaffectation ne touche que `dispatching` | **retenu, fermé** | `reassign_dispatching_deliveries` et `orphan_dispatching_for_instance` couvrent aussi la phase `held` (T004). Les éditions visent un `delivery_id`, pas une instance |
| 7 | capacité négociée par le client, pas par le porteur (`protocol.rs:3190`) | lu `daemon.rs:16150-16200` (capacités client) | **retenu, fermé** | le hold et les éditions ne demandent rien au porteur. Seule la promotion dépend du porteur : il déclare `queue_fallback` à l'enregistrement (T018). La capacité `QueueEditV1` reste une capacité client |
| 8 | application de `cancel` non transactionnelle (suivi, transition d'exécution, helpers à transaction propre) | lu `daemon.rs:3058-3071` : `Store`, `IdempotencyStore`, `ExecutionStore`, `DelegationStore` ouvrent le même fichier par quatre connexions distinctes | **fermé** | les messages retenus n'ont pas d'exécution liée (l'admission d'exécution ne concerne que `TriggerTurn` / `InterruptAndStart`, jamais retenus). Retrait, résultat terminal, notice d'auteur, fermeture du suivi de réponse et reçu sont écrits par la seule connexion de `IdempotencyStore`, qui écrit déjà `requests` et le ledger dans `begin_send_delivery` (`send_delivery.rs:109-140`). Vérifié en T002(5) |
| 9 | migration v7 : précontrôle plafonné à 6 (`store_schema.rs:4`) ; retour au binaire v6 impossible | lu `store_schema.rs:1-45` : `MAX_IDEMPOTENCY_SCHEMA_VERSION = 6`, un schéma supérieur est refusé « aucune migration » | **retenu, fermé** | T003 relève la constante à 7 dans la même modification, avec test du précontrôle. Le retour arrière est impossible par conception (échec fermé) : sauvegarde `.backup` obligatoire avant la première ouverture, procédure de retour = restauration, écrite dans l'ADR |
| 10 | snapshot : ordre `messages` / `steer` inversé, `engaged` non attesté (`codex_app_server.rs:1694`) | lu `codex_app_server.rs:1176-1200` | **sans objet** | plus de snapshot du porteur. `list` lit la base : `held` (retenu, éditable) et `released` (libéré au porteur par le daemon : fait que le daemon connaît) |
| 11 | « zéro perte » non démontré ; redémarrage du porteur exclu ; T018 accepte `indeterminate` | lu le tableau de pannes d'origine | **fermé pour les messages retenus** | un message retenu survit à tout redémarrage (il n'est que dans la base). Le tableau de pannes (T009, T021) vérifie : retenu, libéré (`dispatching`) ou `cancelled`, jamais perdu, jamais doublé. `indeterminate` ne peut plus résulter d'une édition ; il reste possible après libération, comportement existant hors édition |

## Évaluation de la piste « file tenue par le daemon » (Art. XIX)

Détail et preuves : `research.md` R14. Décision : **oui, mais en deux lots
livrables séparément et sans changer le défaut**.

- 159a : file retenue par le daemon, ouverte par un réglage par type d'agent (défaut : fermé). Aucune édition.
- 159b : `cancel`, `reorder`, `promote` sur les messages retenus, une transaction chacun.
- Hors lots : cibles T3 (le pont n'envoie pas `TurnState` et possède le regroupement de la session 114, `t3code.rs:44`, `:3126`, `:3150`). Option 159c : confier la file des fils T3 à la file durable de T3 (`queued-run.*`).

Raison du découpage : la refonte du chemin de remise par défaut touche la reprise,
les échéances, le suivi de réponse et l'ordre des messages ; elle doit pouvoir être
mesurée sur un type d'agent avant d'être étendue.

## Statut final du plan

Points 1, 2, 3, 5, 10 sans objet. Points 4, 6, 7, 8, 9, 11 fermés par une tâche
nommée. Restent à vérifier en T002 : fiabilité du signal de fin de tour de chaque
porteur géré, écriture du suivi de réponse dans la transaction d'`IdempotencyStore`,
tick existant pour le balayage, regroupement éventuel des porteurs gérés.
Nouveau risque nommé : un signal de fin de tour perdu retient les messages jusqu'à
2730 s (45 min + 30 s de grâce, `registry.rs:18`, `daemon.rs:478`) ; déclencheurs
supplémentaires et réglage par type d'agent le bornent. Pas de ronde 2 demandée
avant relecture du nouveau plan.

## Ronde 2 (nouvelle conception « file retenue par le daemon »)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : points 1, 2, 3, 5, 10 de la ronde 1 sans objet pour `held` ; point 7 cohérent ; points 4, 6, 8, 11 partiellement ouverts ; point 9 bien conçu mais tests seulement prévus. Exclusion des fils T3 jugée cohérente (regroupement 114, remise Claude/Cursor). Garde 156 requise pour toute promotion. 10 points nouveaux.
- **Principe** : chaque point est une hypothèse, relue dans le checkout principal. Quand une promesse était fausse, elle a été réduite plutôt que laissée en l'état.

| # | Point | Vérifiée comment | Retenue | Raison et sort |
|---|---|---|---|---|
| R2-1 | « Un message par tour » non garanti : un refus de remise émet `TurnState false` sans fin de tour ; l'absence de `dispatching` ne réserve pas le tour après l'accusé | lu `wrapper.rs:6205-6232` : le gestionnaire de `DeliveryRejected` envoie `TurnState{in_progress:false}` sans condition ; lu `send_delivery.rs:700+` : après l'accusé la ligne quitte `dispatching` | **oui, critique** | signal de tour véridique et réattesté (FR-023, T006) ; réservation de tour durable `turn_gate` levée par `TurnState true`, sortie sans accusé, ou accusé plus 30 s sans occupation (FR-024, T007) ; instances sans `turn_attest` jamais retenues. Tests : refus de guidage en plein tour, accusé puis faux signal terminal, déclencheurs concurrents (SC-012) |
| R2-2 | Signal perdu : la présence reste `busy` ; chaque `true` remet le départ à zéro | lu `daemon.rs:3473-3491` (`busy_since = Instant::now()` à chaque `true`) et `:519-528` (obsolescence mesurée depuis `busy_since`) | **oui** | réattestation au plus toutes les 30 s ; `busy_period_start` non remis à zéro par un `true` répété ; deux cas distincts : wrapper vivant (prochaine réattestation) et wrapper muet (sortie forcée, au plus 2730 s depuis `busy_period_start`). SC-005 corrigé (FR-025) |
| R2-3 | Double redémarrage : présences vides, retenus sans lien d'exécution introuvables | lu `daemon.rs:3150` (`presences: HashMap::new()`) et `:6295-6325` (la migration des remises suppose de retrouver l'ancienne instance dans `presences`) | **oui** | colonne durable `recipient_agent` (résolue à l'admission) et `adopt_held_for_agent` à chaque enregistrement, sans passer par `presences` (FR-007, T003, T004). Test : daemon puis porteur redémarrés avec une nouvelle instance (SC-015). Reste vrai pour les remises `dispatching` existantes : comportement actuel, non modifié |
| R2-4 | « Zéro perte » après libération faux : `Seen` avant injection | lu `receipt_store.rs:140-165` (`Seen` persisté puis `Inject`), `wrapper.rs:757-776` (rejeu d'un `Seen` rend `Indeterminate`) | **oui, critique** | promesse **réduite** : ni perte ni doublon pour un message encore retenu ; après la libération, garanties de toute remise, avec indétermination explicite. Point d'arrêt « entre `Seen` et l'injection » ajouté au tableau de pannes (T010). Reprise durable du porteur écartée : elle changerait la sémantique d'injection de toutes les remises (R17). SC-002 réécrit |
| R2-5 | `queue_fallback` ne garantit pas une promotion sans rejet : file ordinaire pleine ; Codex remet le guidage sur toute erreur, même après application | lu `claude_stream_json.rs:318-324` (`file Claude pleine`), `codex_app_server.rs:1700-1740` (`state.steer.push_front(message)` sur tout `Err`) | **oui** | promesse **réduite** (FR-016, SC-009) : pas de perte avant la libération ; file ordinaire pleine = rejet explicite ; erreur ambiguë de Codex = doublon possible. Tests T020 les établissent et la documentation les déclare. Plus d'« acceptation durable » promise |
| R2-6 | `release_next_held` ne vérifie pas la clôture ni l'échéance du suivi | lu la conception d'origine (`research.md` R5 : seule la saga était vérifiée) | **oui** | la transaction de libération et celle de promotion relisent la demande suivie (ouverte, échéance) et l'échéance de saga (FR-004, FR-016). Tests : clôture juste avant (SC-014) |
| R2-7 | Clore en SQL ne clôt pas `pending_replies` en mémoire ; rappels sans relire SQL | lu `daemon.rs:6880-6912` (boucle des relances sur `pending_replies`), `:18125-18160` (annulation qui fait `retain` en mémoire), `:3150` (liste en mémoire) | **oui** | après chaque commit de retrait, retrait de l'entrée mémoire ; ensemble `held_ids` : aucun rappel palier 1 ou 2 pour une demande retenue (FR-006, T008, SC-007) |
| R2-8 | Pause : classification de la libération absente (ADR 027) | lu `docs/decisions/027-…md` décisions 2 et amendement : la pause interrompt les tours et mémorise leur reprise ; les remises acceptées restent admises | **oui** | choix et justification : un message retenu n'a été remis à aucun porteur, sa libération démarre du travail pour un agent que la pause vient d'interrompre, et entrerait en collision avec la reprise mémorisée. Effet autonome `HeldRelease` différé en pause ; à la levée, reprise d'abord, libération ensuite (FR-026, T007, SC-013). Différence assumée avec la file mémoire actuelle des transports, qui poursuit |
| R2-9 | Migration : restauration non vérifiée | lu `store_schema.rs:1-45` (échec fermé) ; la procédure d'origine ne contenait que des tests de migration | **oui** | tâche T011 : répétition isolée de la restauration avec l'ancien binaire, comparaison des données, pièce `validation/restauration-v7.md`, porte de la livraison du lot A (FR-010, SC-006) |
| R2-10 | R8 admet B,A,C ou B,C,A, T017 exigeait B,A,C | lu `research.md` R8 et `tasks.md` d'origine | **oui** | T019 teste les deux ordres de validation ; l'ordre persistant est l'ordre de validation (FR-008, FR-015, SC-008) |

## Effets sur le plan

- Tâches : 27 au lieu de 25 (lot A : 13, lot B : 14). Ajouts : T006 (signal de tour véridique et réattesté) et T011 (répétition de la restauration). La réservation de tour, l'adoption par agent, la vérification du suivi en transaction, la classification de la pause et le suivi mémoire sont intégrés aux tâches existantes (T003, T004, T007, T008).
- Nouvelle condition de rétention : l'instance doit avoir déclaré `turn_attest`. La phrase « la rétention n'exige rien du porteur » de la première version de la conception était fausse ; elle est retirée.
- Garanties réduites : SC-002, SC-005, SC-009 réécrits ; R17 ajoutée.
- Nouvelle différence de comportement, déclarée : en pause, aucun message retenu n'est libéré.
- Questions utilisateur : inchangées (3).

## Statut final du plan

Les 10 points sont retenus et intégrés. Restent à vérifier en T002 : source de l'état réel de la session dans le wrapper et cadence du battement (1), colonnes de `requests` (8), contenu de `control_pause_interruptions` et points de levée de la pause (9), résolution de l'identifiant d'agent (10). Aucune promesse n'excède ce que les tests de T007, T010, T011, T020 et T021 établissent. Pas de ronde 3 demandée.
