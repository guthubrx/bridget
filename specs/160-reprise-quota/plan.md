# Plan - Session160 reprise après une limite d'usage

**Spec** : `specs/160-reprise-quota/spec.md`
**Branche** : `session-160-reprise-quota`
**Worktree** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota`
**Date** : 2026-10-11
**Préalables livrés (hypothèses)** : 150, 152 (révisée ronde 2), 157. La porte
T001 les prouve sur le code livré.

## Résumé

Six corrections, une par exigence d'entrée issue de la contre-revue de la 157,
sur un mécanisme : une mission opt-in qui touche un quota attend l'heure de
retour, puis reprend une seule fois.

1. compteur de rondes durable (`quota_waits`, liste sans retrait) (R3) ;
2. reconstruction ciblée d'une exécution `failed`, autorisée par une
   qualification durable (R4, R5) ;
3. identité de relance reliée à la continuation, ordre d'écriture entre les deux
   magasins, huit frontières a–h de crash, remise reprise à l'état trouvé (R5, R10) ;
4. cause quota distincte du compteur de pannes de 152 (R6) ;
5. gel cumulé de la pause et de l'attente par l'union des intervalles (R7) ;
6. gestionnaire de `DeliveryRejected` qui sort sans effet, avec une décision en
   attente de 5 s (R8, R9).

## Contexte technique

- Rust 2024, workspace à 3 crates. Travail dans `bridget-daemon` et
  `bridget-transport`.
- Stockage : un champ de liste et un champ optionnel dans le payload JSON de la
  tâche, une table SQLite nouvelle dans le magasin d'exécutions.
- Tests : recette `umask 077` (`quickstart.md`).
- Aucune nouvelle dépendance. Aucun nouveau module Rust.

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | `Delegate.resume_on_limit` (`Option<bool>`), `DeliveryRejected.execution_id` et `generation` (optionnels) |
| `crates/bridget-daemon/src/wrapper.rs` | capture des deux champs de `DeliveryRejected` depuis `bindings` AVANT toute transition terminale qui retire la liaison |
| `crates/bridget-daemon/src/delegation.rs` | champs `quota_waits`, `quota_hold`, types `QuotaWait`, `QuotaHold` |
| `crates/bridget-daemon/src/execution_store.rs` | table `execution_quota_resumes`, `qualify_quota_failure`, `reconstruct_quota_failed` |
| `crates/bridget-daemon/src/referent_control.rs` | `pause_intervals` (extrait de `pause_overlap`), `frozen_overlap` |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | `qualify_quota`, sortie à cinq valeurs de `rejected()`, ordre fixe de `tick`, chemin B et décision en attente, relance, réservation de chaîne (`may_resume`), exceptions de `prepare_restart`, réconciliation de `cancel_tree`, vue |
| `crates/bridget-daemon/src/daemon.rs` | gestionnaire `DeliveryRejected` (sortie anticipée), `deliver_reconstructed` (extrait de `schedule_execution_recovery`, date d'émission figée, reprise à l'état de la remise) |
| module `provider_limits` de la 157 | `attested_exhausted_since` |
| `crates/bridget-daemon/src/delegation_mcp.rs` | schéma `resume_on_limit`, catalogue `features` |
| tests des mêmes modules, `native_delegation_permissions149_tests.rs` | tests `native160_*`, test qui fige les clés |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | fonction, bornes, limites |
| `docs/decisions/160-reprise-sur-limite-d-usage.md` | ADR |

## Décisions architecturales

- **D-A** Compteur = longueur d'une liste sans retrait. Il ne peut que sur-compter
  (R3).
- **D-B** Reconstruction ciblée d'un `failed`, par identifiant exact, autorisée
  par une ligne de qualification. La reprise ordinaire reste inchangée (R4).
- **D-C** Deux magasins, ordre d'écriture : qualification, entrée de tâche,
  `relaunching`, reconstruction, solde. Chaque frontière est idempotente (R5).
- **D-D** Cause distincte : continuation `usage_limit_reset`, `restart_resumes`
  jamais touché, `prepare_restart` ne reconstruit pas une tâche en attente (R6).
- **D-E** Gel = longueur de l'union des intervalles de pause et d'attente.
  Aucune écriture d'état de gel (R7).
- **D-F** `rejected()` rend `Pass`, `PauseInterruption`, `QuotaHold`,
  `NativeFailed` ou `StaleIgnored`. Le gestionnaire sort pour toute sortie autre
  que `Pass`. Pour TOUT échec d'une tâche opt-in, le seul avis est l'avis durable
  existant : jamais de message direct (R8).
- **D-G** Décision en attente de 5 s pour les seules tâches opt-in. Redémarrage :
  échec normal (R9).
- **D-H** La relance n'utilise pas la continuation gouvernée. Son autorisation est
  la qualification plus l'entrée de tâche (R10).
- **D-I** Bornes : 6 h, 2 rondes, retour futur, fait postérieur à la dernière
  attente (R11).
- **D-J** Aucune substitution : définition épinglée réutilisée (R12).
- **D-K** Remise de la relance reprise à l'état trouvé, date d'émission figée,
  solde seulement à `acked`. `indeterminate` ou `orphaned` : `native_relaunch_failed`
  (le daemon met en quarantaine toute remise native retrouvée `dispatching`). Remise non reprenable : échec explicite
  `native_relaunch_failed` plutôt que réparation (R5, R10, Art. XIX).
- **D-L** Gel quota borné à `reset_at + 30 s`, distinct du gel de pause (R7).
- **D-M** Chaîne réservée à la relance de quota tant que l'entrée est ouverte
  (R10).
- **D-N** Ordre fixe dans `tick`, hold inhibe expiration et rappels (R9).
- **D-P** Corrélation des refus par l'exécution et la génération, par une extension
  compatible de la trame `DeliveryRejected` ; sans corrélation après une relance,
  échec explicite `native_refusal_uncorrelated` (R15).
- **D-O** Annulation pendant `relaunching` : réconciliation ordonnée, sinon
  `cancelling` avec erreur nommée (R16).

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 160 prévue (T029) |
| VIII Forensic | OK | exemple réel cherché (T002), tests rouges avant code (T003 à T008) |
| XV Test-before-next | Adapté | pas de commit automatique ; chaque tâche testée avant d'être cochée |
| XVI Worktree | OK | worktree160 isolé, exécution séquentielle |
| XVIII Complexité | OK | voir ci-dessous |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Complexité (Article XVIII)

- Qualification : recherche par type O(log A), puis parcours des F fenêtres (F borné par157). Les écritures SQLite par clé coûtent O(log N).
- Feuille de chaîne : O(c (log N + log c)) selon la primitive livrée ; ne pas confondre une clé indexée et un accès O(1).
- Gel : coût de lecture du journal de pause H, puis tri et union O((P + Q) log(P + Q)), espace O(P + Q), Q au plus2. P désigne les intervalles de pause lus ; H inclut les lignes SQL parcourues. Calcul seulement quand le résultat sert.
- Décision en attente : un contrôle O(1) par tick et par tâche opt-in concernée.

### Minimalisme (Article XIX)

- Ni module, ni dépendance, ni CLI, ni état Lineage nouveau.
- Une table (nécessaire : seule une ligne d'exécution autorise la
  reconstruction d'un `failed`), deux champs de tâche.
- Extractions à deux usages : `deliver_reconstructed`, `pause_intervals`.
- Réutilisés : 152 (`latest_in_chain`, garde de pause, journal des pauses,
  échéance effective, `prepare_restart`), 157 (`provider_limits`), `canonical`.
- Rejetés : champ d'état de suspension, minuterie en mémoire, compteur entier
  séparé de la liste, avis informatif au parent, gouvernance de continuation,
  nouveau mécanisme de relance de l'enfant, changement de fournisseur.

Potentiel minimalisme : aucune ligne existante supprimée. Le code ajouté est
borné par les six exigences d'entrée.

### Responsabilité future (Article XX)

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- T002 : refus de quota non vu en réel. La lecture du code confirme qu’un parent `failed` peut être réservé par la gouvernance sous ses autres gardes ; cela ne prouve pas sa reconstruction.
- La relance est opt-in, bornée, silencieuse : le parent voit `quota_wait`.

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | T003 à T008, T032, T033 : gestionnaire complet, reconstruction d'un `failed`, gel cumulé, redémarrage en attente, compteur, huit frontières a–h de crash, concurrence avec la reprise ordinaire, hold et horloges, refus ancien |
| Unitaire | `qualify_quota` en table, `frozen_overlap`, `qualify_quota_failure`, `reconstruct_quota_failed`, `attested_exhausted_since`, sérialisation sans le champ |
| Intégration | deux chemins × deux ordres, décision à 5 s, relance unique, 10 échecs de même retour, 2 redémarrages après 2 rondes, annulation, pause, enfant absent |
| Non-régression | filtres `native148`, `native149`, `native150`, `native152`, `native157`, `delegation149`, `spec_087` |
| Suite complète | recette `--workspace --no-fail-fast` avant clôture |

## Risques

| Risque | Parade |
|---|---|
| Primitive de 152 ou 157 absente ou différente | porte T001 ; arrêt et mise à jour du contrat |
| Chemin d'échec d'un vrai quota différent de A et B | T002 ; décision commune `qualify_quota` appelée par tout chemin d'échec connu ; limite notée |
| Hold de 5 s trop court ou trop long | constante unique, test aux deux ordres ; valeur non mesurée en réel |
| Crash entre les deux magasins | ordre d'écriture, idempotence par identité et remise reprise à l'état trouvé, tests des huit frontières a–h |
| Faux positif : un échec sans rapport pendant un quota épuisé | opt-in, fait postérieur à l'envoi, borne de 2 rondes ; dommage possible : répétition d’effets ; opt-in et bornes ne prouvent ni causalité ni exactly-once |
| La gouvernance exige un parent actif | décision D-H, T002 la relit ; le budget de 158 devra compter ces relances |
| Gel de l'enfant seul : le parent peut expirer | limite documentée ; annulation en cascade de 152 |
| La reprise ordinaire de 152 concurrence la relance | chaîne réservée (D-M), tests de reconnexion aux frontières c à f |
| Refus ancien fait échouer la mission relancée, ou refus courant ignoré | corrélation par exécution et génération (D-P) ; sans corrélation, échec explicite ; compatibilité ancien daemon et ancien wrapper testée (T036) |
| Message de relance jamais envoyé avec une attente soldée | solde seulement à `acked` ; incertain = échec explicite (D-K) |
| Double avis d'échec pour une tâche opt-in | sortie `NativeFailed`, test par cas (T015, T037) |
| Remise de relance non reprenable | échec explicite nommé avec avis durable (D-K) |
| Collision d'édition dans `native_delegation.rs` et `daemon.rs` | exécution séquentielle |

## Livraison

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/160-reprise-quota/plan.md#extrait-1).

## Réponses retenues à R1

FR003 est une politique de corrélation temporelle, pas une preuve que le quota cause cet échec. Un fait admissible antérieur à l’échec peut qualifier une panne sans rapport et faire répéter des effets de la mission. L’opt-in, les bornes6h/2rondes et l’idempotence de la relance ne garantissent ni causalité ni exactly-once des effets externes. Aucune fenêtre W60 ou test supplémentaire n’est ajouté.

La corrélation FR006b précède toute classification FR006c/quota. Après une relance, des champs absents donnent donc `native_refusal_uncorrelated`, même pour un refus synchrone. Une réponse vide termine l’Execution en `completed`, tandis que la Task demeure `working` jusqu’au refus : elle échoue avec l’erreur d’origine si le refus est corrélé. Les Task déjà `result_available` ou `cancelled` sont préservées ; une Task déjà `failed` ne reçoit pas de nouvel avis.

Lecture160 conservatrice : `attested_exhausted_since(type,since,now,past_since:Option<i64>)` garde la preuve enregistrée par157 et son admission inchangée. Observation strictement après `since`, puis reset futur OU (`past_since=hold.since` et observation>=past_since). Pendant le hold, filtrer chaque fenêtre par `observed_at <= past_since + 5` avant toute agrégation de reset/observation. Hors résolution du hold, past_since=None ; ancienne observation expirée=NoFact. Le cas fait observé pendant le hold puis expiré à la décision donne `Past{reset_at}`/usage_limit. Une fenêtre rejetée à son enregistrement n’est jamais inventée.

Toute sortie d’échec depuis `relaunching` réconcilie la feuille et la remise avant Task.failed/cleanup, en réemployant `close_lost_execution` et T024 : cible child vérifiée, CAS de la feuille vers `unreachable`, remise dispatching mise en quarantaine. Une erreur de clôture se propage ; l’entrée reste `relaunching` pour réessayer le settlement. Aucun nouvel arrêt physique. Les résultats reçus pour Task.failed sont déjà ignorés par capture_reply (ACK transport seulement). Cela ne garantit pas l’unicité des effets externes.

`AlreadyResumed(id)` reste inchangé : relire snapshot, message et continuation existants pour observed_at initial. Aucun nouveau DTO ni upsert de continuation. Le test dix échecs au même retour prouve seulement≤2relances (Past peut arrêter plus tôt) ; RoundsDone est vérifié séparément par les oracles déjà prévus T014/T022. T029 reprend la limite Codex et non-vu réel.
