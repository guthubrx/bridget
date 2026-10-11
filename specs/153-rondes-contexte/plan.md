# Plan - Session153 rondes multiples et contexte transmis

**Spec** : `specs/153-rondes-contexte/spec.md`
**Branche** : `session-153-rondes-contexte`
**Worktree** : `.worktrees/150-socle-delegation` (exécution séquentielle des sessions 150 à 158, sans commit automatique)
**Date** : 2026-10-11

## Résumé

Trois changements dans le moteur de délégation native, sans nouvel état, sans
nouvelle dépendance et avec une seule table d'index (missions passées) :

1. un champ `brief` sur `bridget_delegate` : un dossier de passation validé par le
   validateur existant, placé avant la consigne (R9) ;
2. un champ `keep_alive_secs` : le nettoyage de l'enfant attend une fin explicite
   ou un délai d'inactivité (R1, R8) ;
3. un outil `bridget_task_continue` : ouvre la ronde suivante sur la même tâche
   et le même enfant (R3, R4, R5, R10).

## Décisions architecturales

| Décision | Raison | Alternative rejetée |
|---|---|---|
| Une tâche, plusieurs rondes ; `mission` change à chaque ronde | garde l'enfant, la lignée, les index par `child` et l'idempotence du résultat | tâche liée par `continues_task_id` : casse `for_child` et le nettoyage |
| Aucun nouvel état : enfant inactif = `result_available` + `idle_deadline_at` | liste de 9 états fermée par le contrat 149 | état `idle` : change le contrat T3 |
| Fin explicite = `bridget_task_cancel` et arrêt humain | un outil de moins ; même sens pour l'agent : « finir avec cet enfant » | outil `bridget_task_release` : surface en plus |
| `brief` = brouillon de `bridget_handoff`, rendu par `validate_and_render` | zéro validateur neuf ; mêmes bornes (16 Kio) et refus | historique du parent : Bridget ne le détient pas ; message handoff séparé : ni atomique, ni ordonné |
| Plafond de 8 rondes, garde de 60 à 7 200 s | borne le contexte de l'enfant et la taille du JSON de la tâche | pas de plafond : contexte non borné |
| Une ronde échouée termine la tâche | réutilise le nettoyage 149 ; la reprise est le sujet de 152 | nouvel essai automatique : hors périmètre |
| Rejeu d'une ronde par `rounds[] = {round, request_id, sha256}` | 8 entrées au plus ; la `request_id` d'une ronde ne vit pas dans l'index unique `(owner, request_id)` des délégations | table dédiée : surface en plus |
| Droits revérifiés à chaque ronde par une fonction extraite de `spawn_task` | deux usages d'un contrôle de sécurité | copier les lignes : divergence future |
| Droits actuels : recalcul de la politique de l'enfant avec l'attestation actuelle, refus si elle diffère de la politique figée | une réduction de droits sans révocation ne doit pas laisser l'enfant continuer | se limiter aux révocations et aux sources : ne voit pas la réduction (contre-revue Codex, objection 3) |
| Index `native_delegation_past_missions(mission, task_id)` | absorber une réponse tardive en O(log n) ; balayer `json_each` de 4096 tâches par message serait O(n) (Art. XVIII) | tableau JSON dans la tâche : lecture en O(n) |
| Preuve d'accusé par ronde (`unacked_rounds`) | ne perd aucun résultat indéterminé (invariant 150) sans bloquer le parent, qui continue souvent dans le tour où il lit | exiger l'accusé avant la ronde : le message n'est remis qu'à la fin du tour du parent, la ronde serait impossible dans le même tour |

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | `Delegate.keep_alive_secs`, `Delegate.brief` (optionnels, `skip_serializing_if`) ; variante `Continue` |
| `crates/bridget-daemon/src/delegation.rs` | champs `round`, `idle_deadline_at`, `round_body`, `rounds`, `guarded`, `unacked_rounds` ; `round_no()`, `result_message_id()`, `holds_slot()` ; table `native_delegation_past_missions` et `past_mission_task()` |
| `crates/bridget-daemon/src/delegation_lineage.rs` | `begin_round` (transaction : contrôle d'échéance, mission passée, `unacked_rounds`, `completed_at` remis à vide) ; `human_cancel` lève la garde d'une tâche inactive ; suppression des missions passées avec l'arbre purgé (150) |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | admission (`brief`, `keep_alive_secs`), `Continue`, `tick` (pose de la garde avec `result_sent`, expiration, gardes de nettoyage, corps de mission), `capture_reply` (missions passées), `Cancel`, `cancel_tree` (phase sous verrou), `prepare_restart`, `result_message_id`, `recheck_task_authority` (révocations, sources, politique actuelle), recopie des accusés par ronde (150), catalogue, `task_view`, `drives` |
| `crates/bridget-daemon/src/delegation_mcp.rs` | outil `bridget_task_continue`, champs de `bridget_delegate` |
| `crates/bridget-daemon/src/mcp.rs` | dispatch, liste des noms et test du catalogue (`:2608`) |
| `crates/bridget-daemon/src/wrapper.rs` | `BRIDGET_SAFE_MCP_TOOLS` (20 vers 21), listes d'approbation Codex |
| `crates/bridget-daemon/tests/claude_native_permissions_test.rs` | liste d'outils (`:400`) |
| `crates/bridget-daemon/tests/native_delegation_e2e.rs` | scénario trois rondes |
| tests des mêmes modules | tests `native153_*` |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | rondes, brief, refus |
| `docs/decisions/153-rondes-et-contexte-delegation-native.md` | ADR |

Aucun nouveau module. `handoff.rs` n'est pas modifié : sa fonction
`validate_and_render` est déjà `pub(crate)`.

## Mécanique

### Cycle d'une tâche avec garde

```
queued - starting - mission_pending - working - waiting_for_children
   - result_available (ronde 1 remise, idle_deadline_at posé)
        |-- bridget_task_continue --> mission_pending (ronde 2) - working - ...
        |-- bridget_task_cancel / arrêt humain / expiration / parent fini / redémarrage
        |        --> garde levée --> nettoyage 149 (cancel_tree) --> cleanup_done
```

### Invariants

- **I1** : la garde se pose et se lève seulement sous le verrou d'état. Une ronde
  s'ouvre seulement sous ce verrou, garde posée. `tick` n'ajoute une tâche aux
  nettoyages que garde levée. Aucune ronde ne s'ouvre donc sur un enfant en cours
  d'arrêt.
- **I2** : une tâche tient une place (16 par instance, 128 au total) si elle est
  non terminale OU si elle a une garde (`guarded`) et que son nettoyage n'est pas
  confirmé (`cleanup_done` faux). Lever la garde ne libère pas la place.
- **I3** : une tâche portant une garde n'est jamais purgeable. Un arbre n'est
  purgeable que si le résultat de chaque ronde est accusé (`delivery_acked` de la
  ronde courante ET `unacked_rounds` vide).
- **I6** : la garde se pose dans la même sauvegarde que `result_sent = true`, dans
  `tick` seulement : l'attente et la lecture d'état de SPEC-151 sont des lectures
  pures. Code local à `tick` (un seul appelant, pas de fonction partagée).
- **I7** : une ronde ne s'ouvre que si `now < idle_deadline_at`, contrôlé à
  l'admission et refait dans `begin_round` avec une heure relue juste avant
  l'écriture (horloge injectable pour les tests).
- **I8** : une fin explicite (`guard_released`) est durable et interdit à `tick` de
  poser la garde, y compris si le résultat n'est pas encore envoyé.
- **I4** : une ronde ne modifie ni `request`, ni `definition`, ni `cwd`, ni
  `permission_snapshot`, ni `child`.
- **I5** : les champs propres à une ronde (`result`, `result_sent`,
  `delivery_acked`, `error`, `mission_deadline_at`, `completed_at`, champs
  ajoutés par 151 et 152) sont remis à zéro par `begin_round` et par lui seul.

### Ordre des refus de `bridget_task_continue`

`identity_unavailable`, `invalid_request`, `brief_invalid`, `task_unavailable`,
rejeu (`envelope_mismatch` ou tâche rendue), `referent_paused`, révocations et
grants (refus existants), `keep_alive_not_enabled`, `result_delivery_pending` (résultat prêt, pas encore envoyé : réessayable), `keep_alive_ended` (garde
levée, tâche terminale, enfant absent ou `now >= idle_deadline_at`) ou
`round_in_progress`, `round_limit`, recontrôle des sources
(`settings_revision_changed`), recontrôle de la politique actuelle
(`permission_not_inherited`, `permission_attestation_unavailable`), puis
`begin_round` (qui refait le contrôle d'échéance).

### Corps de mission

Avec brief : `<corps rendu du dossier>\n\n--- Consigne ---\n<consigne>`.
Sans brief : la consigne seule. Le corps de la ronde courante est stocké dans
`round_body` ; `tick` l'envoie (idempotent par la `mission` de la ronde).

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 153 prévu (T024) |
| VIII Forensic | OK | défauts prouvés par lecture (R1 à R5) ; tests rouges avant correction |
| XV Test-before-next | Adapté | pas de commit automatique ; une tâche cochée après son test vert |
| XVI Worktree | OK | worktree 150 partagé par le pipeline |
| XVIII Complexité | OK | R13 : O(log n + r), r ≤ 8 ; garde en O(p) dans un parcours existant |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Minimalisme (Article XIX)

- Un seul outil neuf. Fin explicite portée par l'outil d'annulation existant.
- Aucun état, aucun module, aucune dépendance (`sha2` existe déjà). Une seule
  table d'index, imposée par l'Art. XVIII (réponses tardives).
- Validateur de brief réutilisé tel quel.
- Une extraction justifiée : `recheck_task_authority` (deux usages).
- Rejetés : délai propre à une ronde, extrait d'historique, reprise après
  redémarrage, nouvel essai de ronde, état `idle`, table des rondes.

Potentiel minimalisme : environ 10 lignes de `spawn_task` déplacées, aucune
suppression nette.

### Responsabilité future (Article XX)

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- Charge future ajoutée : 4 champs de tâche, 1 outil, 7 refus. Justifiée : sans
  eux, le cas « revue, corrections, revue » reste impossible.
- Charge supprimée : le parent n'a plus besoin de recoller le contexte à la main
  à chaque ronde.
- Non vérifiable ici : le lecteur T3 devant une tâche qui repart de
  `result_available`. Parade : mutation visible normale (relecture complète).

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | enfant arrêté après résultat malgré `keep_alive_secs` ; `brief` ignoré ; outil absent |
| Unitaire | bornes, enveloppes, rejeu par ronde, refus ordonnés, garde posée/levée, expiration, annulation d'un enfant inactif, redémarrage, nettoyage du parent, place tenue, purge refusée, identifiant de résultat par ronde, `completed_at` remis à vide |
| Intégration | `native_delegation_e2e` : revue, corrections, revue avec un faux enfant ; même `child_instance` ; trois résultats, trois identifiants |
| Compatibilité | octets canoniques sans nouveau champ ; relecture de payloads 149 et 150 ; filtres `native148`, `native149`, `native150`, `delegation149`, `spec_103` |
| Recette réelle | mémoire de l'enfant entre deux rondes (environnement isolé) ; sinon consigné « non vérifié » |
| Suite complète | `--workspace --no-fail-fast` avant clôture |

## Risques

| Risque | Parade |
|---|---|
| Lecteur T3 mal à l'aise avec `result_available` vers `mission_pending` | mutation visible normale ; test de relecture ; consigné non vérifié côté T3 |
| 151 ou 152 ajoutent des champs par ronde que `begin_round` oublie | T001 liste les champs ; I5 ; test qui compare tous les champs de `Task` à une liste |
| L'identifiant de résultat par ronde casse la preuve d'accusé de 150 | une fonction unique `result_message_id` ; T016 modifie aussi la lecture de phase de 150 |
| Contexte de l'enfant qui grossit | plafond de 8 rondes ; brief delta ; documentation |
| Biais d'un enfant qui garde sa mémoire | documentation : contre-revue indépendante = nouvelle délégation |
| Délai d'inactivité non gelé pendant une pause | assumé : 152 ne gèle que les tâches non terminales ; aucun résultat perdu ; documenté |
| `prepare_restart` réécrit par 152 | T011(d) s'applique au code livré par 152 (lecture en T001) |
| Attente 151 sur une tâche inactive rend aussitôt le résultat de la ronde précédente | `round` ajouté à la vue et aux entrées d'attente (T018) ; sémantique documentée |
| `bridget_task_continue` juste après une attente : le résultat n'est pas encore envoyé par `tick` (1 s) | refus réessayable `result_delivery_pending`, documenté, testé (T013) |
| Redémarrage du daemon perd la mémoire de l'enfant | documenté ; reprise = 152 |
| Annulation reçue avant l'envoi du résultat, puis garde posée par `tick` | `guard_released` durable ; `tick` ne pose pas la garde ; tests agent et humain (T008) |
| Posture effective utilisée à la place de la posture demandée : politique Codex modifiée | recalcul avec la posture demandée (`native_permissions.rs:208-213`) ; tests posture absente, parents natifs, conversions (T014) |
| Politique figée et recalculée jamais identiques à tort (faux refus) | test : ronde immédiate sans changement admise ; si l'égalité du JSON est instable, comparer les champs de droits seuls (T014) |
| Corps de mission 80 Kio plus lourd | plafond de corps injecté 120 000 ; test de borne |
| Fuite d'information via le brief (journal lisible plus largement) | avertissements du dossier rappelés dans l'outil et la documentation |

## Livraison

Code sur disque dans le worktree, sans commit automatique ni relance de service.
Diff cumulé enregistré dans `validation/diff-153.patch`.
