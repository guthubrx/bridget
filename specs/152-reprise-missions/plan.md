# Plan - Session152 reprise des missions natives

**Spec** : `specs/152-reprise-missions/spec.md`
**Branche** : `session-152-reprise-missions`
**Worktree** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions`
**Date** : 2026-10-11
**Préalables livrés (hypothèses H1, H2)** : sessions 150 et 151.

## Résumé

Cinq corrections dans le moteur de délégation native. Elles partagent un seul
mécanisme : reprendre une mission avec son identifiant d'origine, en suivant la
chaîne d'exécutions de l'ADR 027.

1. Pause : suspendre au lieu d'échouer, reprendre quand l'exécution est
   `interrupted` (R1, R2, R3).
2. Horloges : geler l'échéance de mission, le suivi de demande et le lancement
   (R4, R5, R11).
3. Panne : reprendre au lieu de passer `failed/unreachable`, avec une borne (R6).
4. Annulation en cascade : classer d'abord, arrêter ensuite, ne rejouer que les
   échecs (R7, R9).
5. Reçus de refus définitifs (R8).

## Contexte technique

- Rust 2024, crate `bridget-daemon` (et `bridget-transport` si le contrat de
  statut change : non prévu).
- Stockage : payload JSON de la tâche (champs `serde(default)`), une table SQLite
  nouvelle pour les reçus de refus. Pas de nouvelle dépendance.
- Tests : recette `umask 077` (voir `quickstart.md`).

## Décisions architecturales

| # | Décision | Raison |
|---|---|---|
| D1 | La suspension est un fait dérivé (pause active et état gelé), pas un état ni un champ | Lineage 149 inchangé (R10) |
| D2 | Garde de pause en tête de `tick`, avant l'échéance | Objection 4 (R4) |
| D3 | `AutonomousEffect::NativeMissionDispatch` partage le bras de garde de `NativeDelegation` | Garde unique ADR 027 ; admission et dispatch sont deux effets nommés, sans logique dupliquée |
| D4 | `rejected()` rend `Result<bool, String>` : `Ok(true)` protège l'interruption de pause, `Ok(false)` garde le refus ordinaire ; une erreur de lecture/persistance est propagée | Objection 1 (R1), contre-analyse M2 |
| D5 | L'état `interrupted` de pause n'échoue pas la tâche ; une lecture de pause illisible diffère le tick | Chemin B (R1), contre-analyse M2 |
| D6 | Reprise pilotée par `tick`, exige `interrupted` | Objection 2 (R2) |
| D7 | `latest_in_chain` remplace `execution-{mission}` à 4 endroits | Objection 3 (R3) |
| D8 | `frozen_secs` = `pause_overlap` depuis la ligne durable de la demande | Idempotence (R4, R23) |
| D9 | `launch_issued_at` posé et sauvegardé avant la première soumission, sans saga existante | R26 ; une saga déjà soumise garde son enveloppe |
| D10 | `prepare_restart` garde la tâche si identité, échéance, borne OK ; seuls neuf refus métier nommés deviennent un échec, les erreurs opaques sont propagées | Défaut 1 (R6), contre-analyse M3 |
| D11 | `native_child_env` extrait de `spawn_task` ; contrôles de révocation et de sources réutilisés avant chaque reprise | Env natif à la reprise ; l'extraction du bloc env seule n'autorise aucun lancement (R6, R28) |
| D12 | `classify_stop` séparé de `prepare_managed_stop` | Vérifier d'abord (R7) |
| D13 | Table `native_delegation_refusals`, liste fermée de cinq codes | Reçus de refus (R8, R18) |
| D14 | Deux chemins de reprise avec filiation ; après identité Fleet attestée, Connected retente `schedule_execution_recovery` sur l'instance courante, la connexion primaire et l'enfant libre | Objection 1 (R13, R24), course Register/Fleet observée |
| D15 | `restart_resumes` incrémenté à la décision de reprise (sur-compte seulement) | Objection 2 (R14, R22) |
| D16 | Échéance effective lue depuis le journal des pauses dans `prepare_restart` ; `interrupting` réconcilié en `interrupted` | R15 remplacée par R25 ; aucun `settle_suspension` |
| D17 | `rejected()` rend `Ok(true)` pour `pause_interruption` ; le gestionnaire garde le suivi et se tait sur `Ok(true)` ou `Err`, avant ses mutations | Objection 4 (R16), contre-analyse M2 |
| D18 | Dernière exécution = feuille de plus grande `generation` | Objection 5 (R17) |
| D19 | Gel du suivi depuis la demande durable ; `mission_issued_at` garde l'enveloppe d'envoi. L'échéance brute vient seulement de `get_request(mission).created_at` ; absence/erreur diffère, sans date fabriquée | Objection 6 (R19, R23), contre-analyse M4 |
| D21 | Temps de pause = fonction du journal `control_events` (`pause_overlap`) ; index temporel dans le schéma existant ; échéance effective sans réécriture | R25, R28 ; coût O(log N + P), N événements durables, P événements de la fenêtre |
| D22 | `restart_identities` non fatal pour une tâche irréconciliable ; vérifier aussi son marqueur avant publication de l'identité | R24, R28 ; la réconciliation lit les marqueurs même hors de la table native |
| D20 | Annulation : contrôle avant toute mutation, tick sur les racines seulement | Objection 7 (R20) |

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-daemon/src/delegation.rs` | champs `launch_issued_at`, `mission_issued_at`, `restart_resumes`, `cancel_stopped` ; table et lecture des reçus de refus |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | garde de pause dans `tick`, `rejected()`, `latest_in_chain`, `may_resume`, `prepare_restart`, `native_child_env`, `cancel_tree`, `task_view`, reçus de refus à l'admission |
| `crates/bridget-daemon/src/daemon.rs` | garde de reprise (`:4251`), `reserve_managed_recoveries`, `PendingReply.frozen_secs`, `collect_reminder_actions`, `restore_pending_for_agent`, `classify_stop`, appel de reprise depuis `tick` |
| `crates/bridget-daemon/src/execution_store.rs` | `latest_in_chain` (génération), `abandon_pause_interruption` |
| `crates/bridget-daemon/src/referent_control.rs` | variante `NativeMissionDispatch`, bras partagé ; `pause_overlap` et index temporel dans le schéma existant |
| `crates/bridget-daemon/src/fleet.rs` | rendre `ownership_from_desired` accessible au crate pour la reprise connectée ; réutiliser sa logique, sans copie |
| `crates/bridget-daemon/src/daemon.rs` (entretien) | purge des reçus de refus de plus de 30 jours dans le passage de la 150 |
| tests des modules, `tests/native_delegation_e2e.rs` | tests `native152_*` |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | pause, reprise, refus nouveaux |
| `docs/decisions/152-reprise-missions-natives.md` | ADR des décisions D1 à D22 |

Aucun nouveau module Rust.

La lecture fraîche de la base `cd2bb015755fde7af26cf87fdb8f4e658172e237`
est consignée dans `reuse-audit.md` et `research.md` R27. Les repères de ligne
anciens restent historiques. `agent_link_by_id` existe dans `IdempotencyStore` :
T043 réutilise cette lecture, sans nouvelle requête ni façade sans règle métier.
L'alignement durable `align_request_deadline` livré par 150 est conservé ; le gel
152 n'ajoute aucune réécriture relative de `tracked_requests.deadline_at`.
T018 calcule aussi l'échéance métier native effective dans `wait_request` 151 ;
le budget `wait_timeout` du demandeur reste inchangé. T017 applique la même
échéance effective à l'admission d'un descendant ; la date brute exposée reste
la date persistée. T021 ajoute `suspended` dans `current_task_view`, et garde
`task_view` pur.

T050 crée dans le schéma existant un index temporel sur `(at, id)` pour les
événements `pause_on`/`pause_off`. La recherche du prédécesseur et de la fenêtre
utilise cet index ; `id` départage les événements d'une même seconde. L'index
est assuré après les migrations existantes. Aucun journal ni outil nouveau.

## Étapes d'implémentation

1. Baseline et tests rouges (T001 à T007, T038 à T040, T047 à T049) : pause chemin A complet,
   chemin B, ordre inverse, horloges, lancement après pause, panne (enfant
   connecté, deux redémarrages, imbriquée), panne pendant pause, gel mixte,
   annulation multi-ticks, borne avant l'envoi, identité aux frontières de crash,
   pause enregistrée avant le premier tick.
2. Fondations (T008 à T011, T050) : champs de tâche, variante de garde, fonctions de
   magasin, `frozen_secs`, `pause_overlap`. T009 et T050 partagent le même fichier
   et sont attribuées au même propriétaire.
3. US1 et US2 (T012 à T021, T044, T045) : pause, reprise, gel des horloges.
4. US3 (T022 à T025, T041 à T043) : reprise après panne.
5. US4 (T026 à T028, T046) : annulation en cascade.
6. US5 (T029 à T031) : reçus de refus.
7. Finitions (T032 à T037) : compatibilité, docs, ADR, suite complète, relecture.

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 152 prévu (T035) |
| VIII Forensic | OK | tests rouges qui reproduisent les chemins avant correction |
| XV Test-before-next | Adapté au pipeline autorisé | chaque tâche testée avant cochage ; commits, livraison et nettoyage à la fin de chaque session ; seuls les tests prévus sont exécutés |
| XVI Worktree | OK | worktree dédié 152 ; livraison et nettoyage avant 153, puis jusqu'à 160 |
| XVIII Complexité | OK : commentaires et minimalisme relus, T034 validée après R5 | `latest_in_chain` O(c × (log n + log c)), espace O(c), c maillons sur n exécutions/liens ; ORDER BY/LIMIT 1 garde une seule feuille, coût O(c). `may_resume` O(B + F log(F + 1) + log L + c × (log C + log E + log(c + 1)) + log P), avec B octets de flotte, F entrées, L liens, C continuations, E exécutions, P interruptions ; génération décalée : O(Bs + S log(S + 1)), S sagas/Bs octets ; hors latences système. `pause_overlap` O(log N + P) avec index temporel ; q demandes donnent O(q × (log N + P)). `classify_stop` O(1) supervisé ou O(b) pour un marqueur de b octets. Précontrôle/index locaux de `cancel_tree` O(m + f + p + n log n), hors descendants/liens/marqueurs ; sauvegardes cumulées O(n²) octets. Aucun banc ni optimisation revendiqués ; le Python EXPLAIN n’est pas la SQLite bundled Rust |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Minimalisme (Article XIX)

- Réutilisés : reprise ADR 027 (`reconstruct_active_for_agent`), chaîne de
  continuations, `prepare_recovery`, garde unique, entretien de la 150.
- Pas de nouvel état, pas de nouveau module, pas de nouvelle dépendance.
- Deux extractions de fonctions existantes (`native_child_env`, `classify_stop`).
  Chacune a deux usages et supprime une duplication de logique d'effet.
  Leur contrainte architecturale est précise : même identité et mêmes permissions
  aux deux chemins de lancement ; classement sans effet avant une annulation.
- Rejetés : état `suspended` (R10), décalage relatif des échéances suivies pour
  gérer la pause (R25, R27),
  retrait de `deadline_at` du canon de flotte (R5), nouveau mécanisme de reprise
  des missions.
- Reçus de refus limités aux refus qui dépendent de l'état du monde (R8).

Potentiel minimalisme : environ 40 lignes supprimées (branche `failed` de
`prepare_restart`, mise à `failed` des chemins A et B, double collecte d'état
d'arrêt).

### Responsabilité future (Article XX)

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- Le risque principal est l'environnement natif d'un enfant relancé (H4). Un
  test rouge le prouve avant le code.
- Non vérifiable dans ce dépôt : le comportement réel de T3 sur un fil `working`
  pendant la pause. La vue reste `working` (question 3).

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | mission sous pause : refus de remise (A), état `interrupted` (B), ordre `TurnState(false)` avant `interrupted` ; échéance qui expire en pause ; lancement après pause de 600 s ; redémarrage avec mission `working` ; annulation avec membre non arrêtable |
| Unitaire | `latest_in_chain`, `may_resume`, calcul d'échéance effective idempotent, `pause_overlap`, `frozen_secs`, `classify_stop`, reçus de refus (liste fermée) |
| Intégration | pause puis levée avec résultat rendu une fois ; redémarrage puis résultat ; 3e redémarrage |
| Non-régression | filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087` |
| Suite complète | recette `--workspace --no-fail-fast` avant clôture |

## Risques

| Risque | Parade |
|---|---|
| L'enfant relancé n'a pas l'environnement natif, les autorisations encore valides ou la filiation (H4, R13) | tests T004 existants sur les deux chemins ; contrôles avant reprise, `native_child_env`, `ownership_from_desired`, commande dérivée du lien |
| Lien de flotte refusé pour un parent d'instance disparue (mission imbriquée) | test rouge T004 (d), puis reprise avec filiation attestée exigée par FR-027/SC-005 ; ne pas remplacer ce contrat par un échec toléré |
| Panne pendant une pause : échéance jugée brute, `interrupting` sans terminal | `effective_deadline` au démarrage depuis le journal, réconciliation `interrupting` (T041, T042, T050) |
| Descendants annulés un par un malgré le refus global | contrôle avant mutation, tick sur les racines (T027, T046) |
| Reprise avant l'état `interrupted` | reprise pilotée par `tick`, exige `interrupted` ; test de l'ordre inverse |
| Décalage d'échéance perdu ou doublé par un crash | aucune écriture : échéance effective = enregistrée + `pause_overlap` (T017, T050) |
| Compteur de reprises sous-compté par un crash | incrément sauvegardé avant la reprise, au même endroit (T023, T025) |
| Daemon qui ne démarre pas sur une commande, instance ou lien natifs incohérents dans un marqueur lisible | tâche `failed` nommée, démarrage continue (T043) ; les erreurs de cible étrangère, CAS ou stockage restent bloquantes |
| Second refus fatal dans la réconciliation des marqueurs | T043/T048 vérifient l’identité du marqueur natif lisible avant publication ; les marqueurs corrompus/illisibles gardent le refus sûr existant hors FR-029 |
| Cleanup d'annulation qui écrase une erreur terminale | T027 conserve l'erreur et le résultat déjà acquis pendant l'arrêt et le cleanup |
| Saga de lancement qui expire pendant une longue pause | limite documentée (FR-012) ; test qui prouve le comportement |
| Boucle de reprise sur une mission qui tue le daemon | borne de 2 reprises ; échec `native_resume_exhausted` |
| Mission annulée ressuscitée par la levée | `may_resume` refuse ; `abandon_pause_interruption` ; test dédié |
| Refus transitoire figé par erreur | liste fermée ; test qui vérifie que chaque code transitoire n'écrit rien |
| Conflits d'édition sur `native_delegation.rs` | un propriétaire par fichier ; 151 déjà livrée, 153 ne démarre qu'après livraison et nettoyage de 152 |

## Livraison

Après les gates de validation : commits, livraison et nettoyage de la session
152, puis exécution séquentielle des sessions 153 à 160, avec leur propre
livraison et nettoyage. Le principal coordonne Git, les fichiers installés et
l'activation de production. La référence historique 147 à 151 installait les
fichiers sans relance ; elle n'autorise pas à interrompre les autres fils T3.
Le diff 152 est enregistré dans
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/diff-152.patch`.

FR-029 ne promet pas la réparation d’un fichier de marqueur corrompu ou
illisible. `load` rend une erreur et la réconciliation ne supprime un marqueur
qu’après preuve de disparition du groupe ou de PID recyclé. Réutiliser ces
garanties pour les incohérences natives lisibles ; aucun nouveau mécanisme de
réparation des marqueurs ordinaires.

Les incohérences de cible d'exécution étrangère et les erreurs de CAS ou de
stockage restent des refus sûrs bloquants. Elles ne sont pas converties en
`native_restart_identity_mismatch` pour contourner les contrôles149.
Après les arrêts confirmés, une erreur de clôture CAS peut laisser des
clôtures métier partielles ; elle ne relance pas la boucle d'arrêts.

Le cas de demande durable absente après admission ou en `working` sans
échéance a été confirmé dans la source de R5. Le correctif posé réutilise
`close_lost_execution` avec ses gardes avant la sauvegarde de l'échec
`request_unavailable`. Le même chemin consomme la pause liée pour cette raison,
après les contrôles existants, sans nouvelle continuation. Une erreur de lecture SQL, de cible ou de CAS reporte
sans mutation de tâche dans ces branches ; le chemin avant admission reste
inchangé. Aucune validation après ce patch n'est encore acquise.
