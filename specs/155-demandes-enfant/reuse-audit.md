# Audit de réutilisation de l'existant - 155-demandes-enfant

## Décision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/155-demandes-enfant

Conclusion courte: Le plan réutilise la boîte humaine, la garde de pause, l'avis
système, la trame de contrôle et l'état d'exécution `waiting_*`. Il crée un fichier
(`daemon/native_child_requests.rs`), une table et trois trames. Chaque création est
justifiée plus bas. Aucune duplication évidente.

## Synthèse

| Métrique | Valeur |
|---|---:|
| Items extraits du plan | 18 |
| Items audités | 18 |
| Réutilisations déjà prévues | 12 |
| Existants potentiellement pertinents | 4 |
| Duplications évidentes | 0 |
| Règles et mémoires applicables | 6 |
| Specs existantes applicables | 6 |

## Réutilisations correctement identifiées

| Item du plan | Existant réutilisé | Preuve | Commentaire |
|---|---|---|---|
| Route humaine | `daemon_deposit_human_inbox` (rendra son résultat), `deposit_and_notify`, `get`, `resolve`, `ack`, `close_self` | `crates/bridget-daemon/src/daemon.rs:9412`, `human_inbox.rs:99,189,237,308,333` | type `intervention_required` existant ; lecture par identifiant exact |
| Garde de pause | `admit_autonomous_effect` + variante de 150 | `crates/bridget-daemon/src/referent_control.rs:125` | aucune nouvelle garde |
| Avis au parent | `send_as` et message système idempotent | `native_delegation.rs:716,1040-1066` | patron de `native-failure-<task>` |
| Contrôle du propriétaire | `recover_owner`, `can_own_task`, contrôle d'identité de `Status`/`Cancel` | `native_delegation.rs:136,172` | même règle pour `Respond` |
| Droits figés du parent | `permission_snapshot.parent` | `crates/bridget-transport/src/native_permissions.rs:23-31` | aucune relecture du parent vivant |
| Envoi vers l'enfant | patron `ControlExecutionDispatch` | `daemon.rs:9195-9240`, `wrapper.rs:4485` | route agent puis connexion |
| États d'attente | `waiting_approval`, `waiting_user_input` | `execution_store.rs:954`, `daemon.rs:3584` | aucun état de tâche nouveau |
| Transition d'exécution | `publish_execution_transition` | `wrapper.rs:5506` | sans `publish_approval_wait` (limite 3) |
| Tick 1 s | `native_delegation::tick` | `daemon.rs:4700` | expiration et renvoi |
| Sauvegarde de politique | `CHILD_POLICY_ENV` et patron d'environnement du lancement | `native_delegation.rs:~585` | une variable d'environnement voisine |
| Liste fermée des outils du parent natif | `BRIDGET_SAFE_MCP_TOOLS` | `wrapper.rs:5035-5057,5061,5265` | ajout précis de `bridget_task_respond` |

## Existant potentiellement pertinent non mentionné

| Item du plan | Existant proche | Preuve | Décision |
|---|---|---|---|
| Observation `permission_required` | transition publiée après coup | `wrapper.rs:5741` | Non réutilisable : après coup, sans attente ; la nouvelle trame la remplace pour les enfants `forward` |
| Réponse automatique | `approval_response`, `mcp_elicitation_response` | `codex_app_server.rs:2195,2317` | Conservés tels quels pour le mode `deny` |
| Refus des invites Claude | `can_use_tool` refusé | `claude_stream_json.rs:1084-1100` | Hors périmètre (question 3) |
| `permission_response` ACP | choix par `permissions` | `acp.rs:1995` | Hors périmètre (ACP) |

## Duplications évidentes

| Item proposé | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Créations justifiées

| Création | Pourquoi pas l'existant |
|---|---|
| Table `native_child_requests` | aucune table ne porte une demande de fournisseur avec route, délai et issue ; `human_inbox` ne porte pas le détail ni la réponse du parent |
| Fichier `daemon/native_child_requests.rs` | `native_delegation.rs` fait 2714 lignes ; la logique est séparée et testable |
| Trames `ChildRequest*` | aucune trame ne transporte une demande de fournisseur du wrapper vers le daemon ; `ExecutionStateChanged` ne porte pas de contenu |
| Outil `bridget_task_respond` | aucun outil ne répond à un enfant ; `bridget_send` n'est pas corrélé à une demande de fournisseur et ne porte pas le plafond |
| `classify_request` | `child_policy` calcule une politique, pas un verdict demande contre droits |
| Lecture d'item par identifiant exact + recherche par clé exacte | `pending_decisions` est affamée par 200 anciens items (`human_inbox.rs:275-306`) ; aucune recherche par clé n'existe |

## Mémoires et règles applicables

| Source | Règle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | bornes 4 / 32 / 512, requêtes indexées |
| `~/.speckit/constitution.md` XIX | minimalisme | un fichier neuf, type de boîte réutilisé |
| `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` | garde unique de pause | réponse du parent gardée |
| `docs/decisions/149-permissions-heritees-et-lineage-native.md` | droits figés, pas de dialogue | changement explicite par ADR 155 |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart et gates |
| `AGENTS.md` (checkpoints 149) | livraison sans relance | aucune relance de service |

## Specs livrées applicables

| Spec | Pattern déjà établi | Impact |
|---|---|---|
| 148-identite-delegation | contrat `bridget_delegate`, rejeu par `request_id`, contrôle du propriétaire | `child_requests` dans l'enveloppe |
| 149-sous-agents-lineage | snapshot figé, états fermés | plafond sur le snapshot, aucun état nouveau |
| 150-socle-delegation | `mission_deadline_at`, garde de pause native, purge d'arbre | plafond de délai, garde, suppression des lignes |
| 151 (à venir) | attente et boîte de fin | T031 conditionnelle |
| 087 (pause) | interruption des tours | demandes `cancelled` |
| 089 (tests) | recette `umask 077` | gates de test |

## Journal de recherche

| Requête | Portée | Résultat |
|---|---|---|
| `grep -rn requestUserInput` | `specs`, `docs`, `crates` | aucun résultat : forme à lire en T002 |
| `grep -rn "permission_required"` | `crates` | observation après coup seulement (`wrapper.rs:5741`) |
| `grep -rn "ManagedWaitState::"` | `crates` | consommé par le wrapper, émis par aucun pilote |
| `grep -rn "HumanInboxProducer::Daemon"` | `crates` | aucun consommateur de décisions `Daemon` |
| `grep -n "MAX_APPROVAL_REQUESTS_PER_EXECUTION"` | `wrapper.rs` | limite 3 (`:5700`) |
| `grep -rn "can_use_tool"` | `crates` | refus Claude (`claude_stream_json.rs:1084`) |

## Arbitrages

| Sujet | Décision | Justification | Date |
|---|---|---|---|
| Type de boîte | réutiliser `intervention_required` | évite la migration du `CHECK` | 2026-10-11 |
| Attente dans le wrapper | nouvelle trame, pas `publish_approval_wait` | limite 3 incompatible avec FR-007 | 2026-10-11 |
| Fichier neuf | créer `native_child_requests.rs` | fichier hôte de 2714 lignes | 2026-10-11 |
| Défaut du paramètre | `deny` | contrat 149 inchangé | 2026-10-11 |
| Relève des décisions | lecture par identifiant exact, pas de requête par préfixe | famine à 200 items (contre-revue, objection 9) | 2026-10-11 |
| Route d'approbation du parent | retirée | politique de l'enfant égale à celle du parent ; injection (contre-revue, objections 3 et 8) | 2026-10-11 |
| Enfants Claude | hors périmètre | rouvre le lancement 149 | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication évidente non arbitrée
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les règles projet applicables ont été lues
- [x] Les specs existantes proches ont été vérifiées
- [x] Le plan.md a été refactoré ou les divergences sont justifiées
