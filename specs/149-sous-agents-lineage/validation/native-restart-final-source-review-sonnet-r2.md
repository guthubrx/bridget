# Revue de sources finale - redémarrage natif - session 149 - Sonnet 5.5 - ronde r2

Date : 2026-10-10. Relecteur : sous-agent de revue indépendant (Sonnet 5.5).
Cette revue lit les sources seulement. Elle n'édite que ce rapport. Elle n'exécute ni Cargo, ni test, ni build, ni service, ni Git. Elle ne lance aucun modèle. Elle n'affirme aucun comportement d'exécution.

## 1. Verdict

| Niveau | Verdict |
|---|---|
| Sources : O4.6 (délai natif commun), E2 (queued à propriétaire perdu), F3 (clôture CAS avant `cancelled`), alias Codex hors namespace | **SOURCE_ONLY_APPROVE** - zéro finding bloquant |
| Documents (héritage, discovery explicite, compat148, O1) | **Cohérents** avec le code lu. Aucune contradiction matérielle. |
| Delta T3 `BridgetTaskJournal.tsx` (libellé `journal_unavailable`) | **Approuvé** par lecture. SHA conforme. |
| Preuve d'exécution | **Non faite ici.** Voir section 6. |

Ce verdict couvre l'état de disque de la section 2. Une nouvelle modification de production rouvre cette revue pour les fichiers touchés.

## 2. Code exact relu

Racine : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`.

| Fichier (sous `crates/bridget-daemon/src/`) | Modifié à | Digest SHA-256 (16 premiers caractères) |
|---|---|---|
| `wrapper.rs` | 19:19 | `49f3a304d9504fe6` |
| `daemon.rs` | 19:58 | `6baee3bad12b89e5` |
| `managed_process.rs` | 20:00 | `dde9324a45e6c5e2` |
| `daemon/native_delegation.rs` | 20:00 | `7343c6df46ed89fc` |

T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/BridgetTaskJournal.tsx` = `4abac314791a3ad77d8d63e399ec5c9c3e6188df1ebaff13aef426ae90195ff5` (conforme au brief). Le test `BridgetTaskJournal149.test.tsx` fait `69e67222...0a90`.

Lecture faite : les quatre sources ci-dessus (zones citées), `desired_state.rs` (`stop_non_persistent_running`), `fleet.rs` (`open`, `recover_commands`), `execution_store.rs` (`transition_if_current`), `delegation.rs`, `delegation_lineage.rs` (`parent_lineage`), `environment.rs` (préflight), `codex_socket.rs` et `codex_app_server.rs` (garde de socket), puis `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `contracts/permissions.md`, `test-strategy.md`, `native-real-smoke-sonnet-r6.md`.

## 3. Revue point par point

### 3.1 O4.6 - délai natif commun (`managed_process.rs`, `daemon.rs`)

| Point | Conclusion | Preuve |
|---|---|---|
| Source des identités | Conforme. Seul le daemon les fournit. Aucun nom ou PID venu d'un RPC. | `daemon.rs:4547-4557` appelle `native_delegation::restart_identities` sous le flock et après le préflight. Le `DaemonState` est créé avant (`:4551`) et réutilisé après. Il n'y a qu'un seul `DaemonState::new` hors tests. |
| Aucune socket ni supervision avant la réconciliation | Conforme. | `DaemonState::new` ouvre des stores SQLite et lit le registre. Il lance un seul thread inerte (`observation_output`, file d'attente). La socket est liée en `:4581`, le superviseur créé après. `FleetSupervisor::open` ne dépend d'aucun processus vivant (`fleet.rs:405-438` : état désiré et commandes en base). |
| Même dossier de marqueurs | Conforme. | `DaemonState` : `db_path.parent()/managed` (`daemon.rs:3119-3124`). Réconciliation : `state_root.join("managed")` (`:4556`). `state_root` = `db_path.parent()` (`:4482`). Les deux chemins sont identiques. Sans cela, tout wrapper natif serait traité en groupe ordinaire (SIGKILL à 500 ms). |
| Preuve complète avant la branche native | Conforme. | `native_delegation.rs:830-871` exige : marqueur présent ; requête `Delegate` ; `child_instance` ; entrée désirée du même nom ; `command_id == task_id` ; même `agent_type`, même `cwd` ; **définition figée égale** ; lien SQL de flotte (`agent_link_for_child`) ; parent du lien = propriétaire d'origine ou courant ; `delegation_id == task_id` ; rôle `native_delegate` ; lien persisté identique au lien SQL. Puis `managed_process.rs:1240-1248` recoupe `instance_id`, `command_id`, `generation` du marqueur. La naissance PID est lue deux fois (première boucle, puis `:1286` juste avant le signal). |
| Contradiction ou collision | Fermé, sans signal. | Toute incohérence rend `Err` avant le premier `kill` du lot : `restart_identities` (`?` en `daemon.rs:4554`) et le contrôle du marqueur (`managed_process.rs:1240-1248`, première boucle). Aucun groupe, natif ou ordinaire, n'est signalé. |
| Signal étranger | Aucun. | Un marqueur sans identité native prouvée reste dans `ordinary_groups` (`:1274`). Un PID recyclé (`Ok(_) => continue`, `:1286`) ne reçoit aucun signal. `signal_group` garde `group_exists` et traite `ESRCH` (`:514-528`). |
| Délai commun | Conforme. | `native_deadline` est calculé une fois (`:1280`). Tous les SIGTERM natifs partent avant toute attente. Ensuite chaque attente lit `deadline - now` : le lot coûte 8 s au plus, pas N x 8 s. `wait_group_gone` teste le groupe au moins une fois même avec un délai nul (`managed_process.rs:530-541`). |
| Aucun SIGKILL natif | Conforme. | La boucle native (`:1322-1340`) ne fait qu'attendre. Une expiration rend `Err("arrêt coopératif natif incomplet")` et **conserve le marqueur** (le `remove` est après l'attente). Le daemon refuse de démarrer, sans relance du fournisseur. |
| Délai ordinaire intact | Conforme. | `timeout / 2` puis SIGKILL puis `timeout - timeout/2` (`:1307-1320`). `MANAGED_STOP_FORCED_GRACE` = 1 s et `MANAGED_STOP_POLL` = 20 ms (`daemon.rs:462-463`). L'appel sans identité natives passe `BTreeMap::new()` et `Duration::ZERO` (`:1186-1191`) : même comportement qu'avant. |

### 3.2 E2 - `queued` à propriétaire natif perdu (`native_delegation.rs:877-896`, `:930-939`)

| Point | Conclusion | Preuve |
|---|---|---|
| Fermer seulement le cas prouvé | Conforme. | `queued_owner_lost` rend `false` sans `parent_task_id` (`:878`). Un `queued` externe (T3 ou autonome) n'a pas de parent natif : `parent_task_id` n'est posé que si `for_child(&owner)` trouve une tâche (`:460-477`). Il garde la reprise normale. |
| Preuve de filiation | Conforme. | Il faut : parent lisible ; `parent.child == task.owner` ; `instance == owner_instance == origin_owner_instance` ; même racine ; lien de flotte de l'instance du parent ; son parent dans {origine, courant} du parent ; `delegation_id` du parent ; rôle `native_delegate` (`:879-893`). |
| Perte définitive | Conforme. | `prepare_restart` ne s'exécute qu'au démarrage (seul appelant : `daemon.rs:3892`, via `reserve_managed_recoveries` en `:4628`). Les enfants natifs sont exclus de toute reprise. Le propriétaire ne revient donc jamais. |
| Effet sur la racine | Conforme. | Le `queued` passe `failed/unreachable`, donc terminal. `descendants_busy` ne le compte plus (`:1298-1300`). Une racine `waiting_for_children` qui a son résultat sort de l'attente (`:1091-1121`) et le résultat reste livrable une fois (clé idempotente inchangée). |
| Ordre sûr à la reprise | Conforme. | CAS d'exécution d'abord, `save` ensuite (`:937-938`). Un arrêt entre les deux se rejoue sans rouvrir un état terminal. |
| Contradiction de filiation | **Fermée, mais bloque le démarrage.** | Parent absent, instance absente ou lien incohérent rendent `Err` (`:879-893`). La tâche n'est pas modifiée. Mais l'`Err` remonte jusqu'à `daemon.rs:4628` et le daemon refuse de démarrer. Voir O-1. |

### 3.3 F3 - clôture CAS avant `cancelled` (`native_delegation.rs:900-918`, `:1453-1480`)

| Point | Conclusion | Preuve |
|---|---|---|
| CAS exact | Conforme. | `close_lost_execution` lit `execution_control_target`, refuse un agent cible différent (`native_execution_mismatch`), puis appelle `transition_if_current` avec état, révision et génération lus (`:910-911`). La requête SQL répète ces trois conditions dans le `WHERE` (`execution_store.rs:1265-1275`). Aucune mise à jour aveugle. |
| Terminal jamais rouvert | Conforme. | Seuls `queued`, `starting`, `running`, `waiting_approval`, `waiting_user_input` et `interrupting` sont fermés (`:908-909`). `completed`, `failed`, `unreachable`, `interrupted` sont laissés tels quels. |
| Après arrêt confirmé | Conforme. | Le bloc de clôture est sous `if success` (`:1453`). `success` exige `Stopped`, `StoppedForced` ou `NotFound` pour chaque nom (`:1441-1452`). |
| Avant `taskcancelled` | Conforme. | La clôture (`:1462`) précède le passage `cancelling -> cancelled` (`:1466-1467`). |
| Erreur fermée | Conforme. | Un refus de CAS (`native_execution_changed`) ou un agent différent fait `continue` : la tâche reste `cancelling`, `cleanup_done` n'est pas posé, le prochain tick rejoue. Aucun état n'est inventé. |
| Redémarrage | Conforme. | `prepare_restart` ferme aussi les lignes des tâches `cancelling` et `cancelled` (`:940-944`), après la réconciliation des groupes. `descendants_busy` accepte alors l'exécution close (`:1348-1361`). |

### 3.4 Alias Codex hors du namespace (`wrapper.rs:431-480`, `:4057-4061`, `:4761-4774`)

| Point | Conclusion | Preuve |
|---|---|---|
| Emplacement privé | Conforme. | Dossier `0700` créé par `DirBuilder` non récursif (donc échec si le nom existe, aucun suivi de lien), sous `canonicalize("/tmp")`, nom UUID aléatoire. Il est recontrôlé (`validate_private_directory_if_present`). Chemin court : `/private/tmp/bridget-codex-<32 hex>/s.sock`, sous la limite `sun_path`. |
| Indépendant de l'environnement | Conforme. | Ni `TMPDIR` ni `BRIDGET_HOME` ne servent. Aucun détournement possible du point d'écoute fournisseur par l'environnement du modèle. |
| Garde du transport conservée | Conforme. | `validate_private_path` (`codex_socket.rs:21-48`) exige : longueur valide, parent dossier propre à l'utilisateur sans accès groupe ni autres, chemin cible absent. Elle s'applique avant `--listen` (`codex_app_server.rs:362-365`). Le préflight du namespace (`environment.rs:240-280`) refuse toujours tout symlink sous la racine d'état. Il est inchangé, donc un alias étranger dans le namespace reste refusé. |
| Durée de vie | Conforme. | `codex_endpoint` est déclaré avant `transport` : à la sortie, le transport est libéré d'abord, le dossier ensuite. Le transport retire l'alias après arrêt confirmé (`stop_interactive_server`, `codex_app_server.rs:24-35`). Si l'alias subsiste, la fonction rend une erreur et `Drop` conserve le dossier (`wrapper.rs:4761-4774`, `:459-476`). |
| Nettoyage | Conforme. | `Drop` retire le dossier seulement si l'alias est absent, si le dossier est un dossier, appartient à l'utilisateur, a la même identité `(dev, ino)` qu'à la création et n'a aucun droit groupe ou autres. Il utilise `remove_dir` (jamais récursif). |
| Chemin non interactif | Inchangé. | Le repli `state_root/c-<instance>.sock` (`:4059-4061`) n'est lu que dans la branche interactive. Les wrappers natifs managés n'en créent pas. |

### 3.5 Rappel : points déjà approuvés en r1, relus seulement pour interférence

La capacité privée de résultat retenu, l'autorité avant idempotence, le CAS de `working`/`failed-unreachable`, le gestionnaire SIGTERM par drapeau et l'arrêt du fournisseur par le wrapper n'ont pas changé de façon visible dans les zones relues. Aucune interférence trouvée avec les quatre deltas. La revue T3 existante n'est pas refaite.

## 4. Findings

**Aucun finding bloquant ou majeur.**

### Écarts optionnels, non bloquants

Chaque écart donne le déclencheur concret et la preuve dans les sources.

**O-1 - Une incohérence durable ferme le démarrage du daemon, sans procédure d'exploitation écrite**

- Déclencheur : une tâche native avec marqueur dont l'entrée désirée, le lien de flotte ou la définition ne correspond plus (`native_restart_identity_mismatch`) ; un marqueur de mission natif d'identité différente ; un `queued` imbriqué dont le parent est absent ou contradictoire (`parent_task_unavailable`, `native_owner_identity_mismatch`) ; une clôture CAS refusée au démarrage (`native_execution_changed`, `native_execution_mismatch`).
- Preuve : `daemon.rs:4554` et `:4628` propagent avec `?`. `grep` sur `docs/`, `skills/`, `contracts/` et `recovery149.md` ne trouve aucun de ces codes.
- Effet : choix fermé voulu. Aucun fournisseur en double et aucun état rouvert. Le coût est un daemon qui ne démarre pas tant que l'opérateur n'a pas corrigé ou retiré la donnée. Point à confirmer : la consigne « préserver la filiation contradictoire » est respectée au sens « tâche non modifiée », pas au sens « le daemon démarre quand même ».
- Probabilité : faible. Les tâches ne sont jamais supprimées (`delegation.rs`, aucun `DELETE` de production), donc un parent absent exige une corruption.
- Suggestion : une phrase d'exploitation dans `docs/delegation-native.md` (liste des codes et geste de reprise). Pas de changement de code.

**O-2 - Le délai natif de 8 s court depuis le premier SIGTERM, il est consommé par les groupes ordinaires périmés**

- Déclencheur : plusieurs anciens groupes ordinaires et un wrapper natif lent au démarrage. Chaque groupe ordinaire peut attendre jusqu'à 1 s (`managed_process.rs:1292-1320`) avant la boucle native.
- Effet : la marge réelle du wrapper natif baisse d'autant. Avec peu de groupes ordinaires (cas normal), l'effet est nul. En cas d'expiration, le comportement est le refus fermé de la section 3.1, pas un SIGKILL.
- Choix défendable : la date de départ est celle du signal. Aucun changement requis.

**O-3 - Une annulation humaine laisse l'exécution en `unreachable`, avec le motif `native_cancelled`**

- Déclencheur : `bridget_task_cancel` sur une tâche dont l'exécution est active (`native_delegation.rs:1462`).
- Effet : l'état d'exécution dit « injoignable » pour une annulation voulue. `descendants_busy` accepte déjà `interrupted` (`:1357-1359`). Seule la lecture d'un état d'exécution brut est concernée. Le statut de la tâche (`cancelled`) reste exact.
- Non bloquant : le motif porte la cause, et aucun lecteur lu ne dépend de l'état d'exécution pour le statut de la tâche.

**O-4 - Fuite mineure et orphelin résiduel hors périmètre**

- Un wrapper interactif tué par SIGKILL laisse le dossier `/tmp/bridget-codex-*` (0700, vide ou avec l'alias). Aucun balayeur ne le traite. L'effet est un dossier privé, sans droit ni donnée.
- Le cas E4 de r1 reste ouvert : un wrapper natif tué par SIGKILL laisse son fournisseur vivre jusqu'à la fin naturelle de son tour. La réconciliation ne le voit pas, car elle ne suit que le groupe du wrapper.

**O-5 - Verrou d'état gardé pendant les CAS d'annulation**

- `cancel_tree` appelle `close_lost_execution` sous `state.lock()` (`:1454-1462`). Le CAS ouvre une transaction SQLite avec un délai maximal de 2 s. Borné et cohérent avec les `save` voisins, également sous verrou. À noter pour l'Article XVIII, pas à corriger.

### Observations d'information

- `restart_identities` parcourt toutes les tâches en O(T) lectures de marqueur. T est borné à 4096 (`task_limit`). Coût de démarrage uniquement.
- `descendants_busy` est borné à 4096 instances (`descendant_limit`). E1 de r1 reste vrai : une erreur de lecture fait échouer une racine qui a déjà son résultat. Non réouvert.

## 5. Documents et O1

Relus : `docs/delegation-native.md` (lignes 24-27, 74-122, 124-139), `skills/bridget/SKILL.md` (lignes 40-62), `contracts/permissions.md` (lignes 36-52, 436-475), `test-strategy.md` (S149-32).

| Exigence du brief | Conclusion | Preuve |
|---|---|---|
| Héritage par défaut, sans grant humain | Conforme au code. | Doc : posture absente = héritage, « ne demande aucun grant ». Code : avec un fait parent, `child_policy` ne produit jamais plus que le fait (`native_permissions.rs:206-277`) et aucune lecture de grant n'a lieu. |
| `discovery` explicite = lecture seule | Conforme. | Doc l.81. Code : `readonly = posture == Some(Discovery)` (`native_permissions.rs:209`), effet `Discovery` (`:274`). |
| Compat148 sans fait : seulement `discovery` explicite, selon l'ancien grant | Conforme. | Doc l.81-83 et SKILL l.48-50. Code : sans fait, `Discovery` explicite passe par `authorize_cwd` (grant) ; toute autre posture rend `permission_attestation_unavailable` (`native_delegation.rs:429-432`). Pas de repli silencieux. |
| O1 : credential frais sans fait jamais vs credential retiré | Clarifié, cohérent. | `contracts/permissions.md:36-47` et `:441-466` (branche (b)(i)/(ii)) reprennent la formulation de r1. `test-strategy.md` S149-32(b) est aligné. L'annulation humaine native est indépendante du credential T3 (b)(i). |

Seul écart de précision, non matériel : `docs/delegation-native.md` l.134 dit « Une session sans fait de droits garde l'identité seule (version 1) ». La phrase précédente exclut déjà les credentials retirés, révoqués ou tournés. Écrire « une session qui n'a jamais eu de fait de droits » alignerait le mot à mot sur le contrat. À reprendre par le principal si souhaité.

Le contrat l.36-38 annonce « Enveloppe exacte : » puis une phrase avant le bloc JSON. Cette mise en page vient d'avant cette ronde. Elle n'est pas matérielle.

## 6. Delta T3 - `BridgetTaskJournal.tsx`

- `failureMessage` (`BridgetTaskJournal.tsx:39-48`) : le sujet est « Journal de la tâche indisponible » seulement si `code === "journal_unavailable"`. Toute autre erreur garde « Bridget indisponible ». Le code entre parenthèses et la phrase « Le contenu déjà lu est conservé. » sont inchangés. Le code lu vient de `error.code` avec repli `unavailable`.
- Le code existe côté Rust (`attach.rs:291` : `code: "journal_unavailable"`) et dans le contrat de transport (`lineage_contract_test.rs:133`).
- Trois appels utilisent `failureMessage` (`:151`, `:244`, `:304`). Le libellé est commun à la lecture du journal, à l'abonnement et à l'arrêt. Le sujet « Journal de la tâche » est exact pour le code `journal_unavailable` quel que soit l'appelant.
- Le test (`BridgetTaskJournal149.test.tsx:432-462`) vérifie le libellé exact, l'absence de « Bridget indisponible », la conservation de l'historique (« ligne 7 »), les contrôles (`Reconnecter`, `Lire le début`) et l'absence du bouton « Arrêter » pour une tâche terminale. Ces assertions sont des spécifications lues. Elles n'ont pas été exécutées ici.
- Le rendu vient d'un test fixture, pas d'une nouvelle recette. Aucune recette globale n'est déduite.
- Note sur `ui-journal-local-error-haiku-r1.md` : l'écart « baseline web 0 » contre 10 web / 16 server / 0 NEW est documentaire. Aucun changement de source n'est demandé par cette revue.

## 7. Ce qui reste pour l'exécution (ne pas déduire d'un PASS de lecture)

1. **R9 / réseau r4** : redémarrage réel du daemon avec un vrai wrapper natif Codex vivant (branche native de 8 s, plus de SIGKILL à 500 ms). Constat attendu : fournisseur et commande arrêtés, marqueur retiré, tâche `failed/unreachable`, une seule remise du résultat retenu.
2. **Redémarrage réel r7** : parent Codex TUI créé par le binaire actuel, alias sous `/private/tmp/bridget-codex-*`, aucun symlink sous `BRIDGET_HOME`, démarrage du daemon sans déblocage manuel. Contrôle du nettoyage du dossier après la sortie normale.
3. **E2** : le cas n'est pas atteint avec un arrêt SIGTERM normal. Seul un test d'état durable simulé le couvre (propriétaire natif perdu, `queued` imbriqué, racine avec résultat capturé). Un `queued` externe doit rester inchangé dans le même test.
4. **F3** : un test à état d'exécution `starting` puis annulation, avec constat que la ligne est fermée **avant** `cancelled`, et qu'une exécution `completed` n'est pas rouverte.
5. **Compilation** : aucune compilation faite ici. Le testeur r9 la possède. Les types et emprunts des fonctions relues ont été vérifiés par lecture seulement (`ConditionalTransition`, `ManagedIdentity`, `BTreeMap`, `Instant`).

## 8. Limites de cette revue

- Aucune compilation, aucun test, aucun processus lancé.
- Aucun fournisseur réel. Le temps d'arrêt réel d'un Codex ou d'un Claude sous le délai de 8 s reste à mesurer (`native-real-smoke-sonnet-r6.md` donne 1,32 s au plus sur un cas, pour l'arrêt du daemon, pas pour la réconciliation au démarrage).
- Les rapports de recette précédents décrivent des binaires antérieurs aux deltas de 19:19 à 20:00. Ils ne prouvent pas le code relu ici.
- Aucun secret n'est copié dans ce rapport.
