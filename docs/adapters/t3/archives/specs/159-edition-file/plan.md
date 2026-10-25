# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/plan.md](../../../../../../specs/159-edition-file/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Lot A (159a), livré 14/14 sur53863bd** : phase `held`, rang durable, libération et balayage, lecture
  exacte, migration v7. Réglage `held_queue` par type d'agent, **fermé par défaut**.
- **Lot B (159b)** : `cancel`, `reorder`, `promote` sur les messages retenus, avec
  reçu complet, plus `bridget_queue` et la CLI.
- Hors lots : fils T3 (159c), `edit`.

<a id="extrait-2"></a>

## Extrait 2

| Sujet | Choix | Alternative rejetée | Raison |
|---|---|---|---|
| Où vit la file | base du daemon (`send_deliveries`, phase `held`) | file mémoire du porteur | une édition devient une transaction, sans saga (R2, R3) |
| Réutiliser `execution_queue` | non | oui | aucun consommateur, autre magasin, pas de lien à la saga (R14) |
| Portée | types d'agents avec `held_queue`, fermé par défaut | tous les agents d'emblée | mesurer avant d'étendre (R14) |
| Fils T3 | hors lot | émettre `TurnState`, déplacer le regroupement | regroupement 114 et file de 64 propres au pont (R14, R16) |
| Libération | réservation de tour durable (`turn_gate`), repos attesté, revérification en transaction | « aucune remise `dispatching` » seule | l'accusé ouvre un trou avant `TurnState true` ; un refus envoie un faux repos (R5) |
| Signal de tour | véridique et réattesté, `turn_attest` | signal historique avant159A | Anciennes ancres6224/6666 : refus en false avant le lotA. Code livré : DeliveryRejected à `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/crates/bridget-daemon/src/wrapper.rs:6715`, signal réel à6723–6724 ; oracles busy11540/idle11596. `busy_since` reste legacy ; `busy_period_start` est conservé par true. |
| Rattachement | `recipient_agent` durable, adoption à l'enregistrement | `recipient_instance_id` seul | présences vides après un redémarrage du daemon (R5) |
| Pause | `HeldRelease` effet autonome différé | libération libre | collision avec la reprise de pause (ADR 027) |
| Déclencheurs | repos attesté, reconnexion, fin d'exécution, levée de pause, tick | fin de tour seule | aucun ne prouve seul le repos : conditions revérifiées |
| Édition | transaction unique + reçu | saga répartie | 11 défauts de la première conception |
| Résultat d'édition | toujours final | `outcome_unknown` | aucun aller-retour avec le porteur |
| Promotion | libération immédiate en guidage + marqueur `steer-or-queue`, garanties réduites | annuler puis renvoyer | pas de perte avant la remise ; file ordinaire pleine et erreur ambiguë de Codex restent des cas déclarés (R9, R17) |
| Porteur ancien | chemin actuel sans rétention faute de `turn_attest` ; `promote` du lot B exige `queue_fallback` | exiger partout | la rétention exige le signal véridique déclaré ; les capacités anciennes restent fermées |
| Rang | queue_rank INTEGER PRIMARY KEY AUTOINCREMENT ; delivery_id UNIQUE NOT NULL | rang recyclé après purge ou compteur métier parallèle | FR008 durable ; INSERT sans rang, anciens rowid copiés ; surcoût SQLite justifié (R6) |
| Migration A livrée | v7 à sens unique + précontrôle relevé + sauvegarde éprouvée dans159A | double écriture | `CHECK` de phase ; échec fermé voulu (R6) |
| Reçu d'édition | `queue_edits`, même transaction que l'effet | table de réservation | effet et reçu atomiques |
| `edit` | reporté | implémenter | enveloppe idempotente, attribution (R15) |
| Outil | `bridget_queue` + `action` | outils séparés | un outil, déjà créé en 156 |

<a id="extrait-3"></a>

## Extrait 3

Aucun nouveau module Rust. Aucun changement du pont T3 (`t3code.rs`). Aucune
trame daemon vers porteur.

<a id="extrait-4"></a>

## Extrait 4

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | A et B réalisées, publication après gates ROOT | ADR159 initiale A dans T012 livrée ; extension B du même fichier dans T025 : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/decisions/159-file-retenue-et-edition.md |
| VIII Forensic | OK | défauts prouvés par lecture ; tests rouges d'abord |
| XV Test-before-next | Adapté | pas de commit automatique ; tâche cochée après test vert |
| XVI Worktree | OK | worktree partagé |
| XVIII Complexité | OK | R13 |
| XIX Minimalisme | OK | lot A fermé par défaut, deux lots, `edit` et T3 reportés, pas de saga d'édition, pas de nouveau module |
| XX Vertus LLM | OK | chaque décision a sa preuve `fichier:ligne` ; coûts listés |
| XXII Optimisation | OK | latence mesurée avant et après (T001, T010), pas supposée |

<a id="extrait-5"></a>

## Extrait 5

- Réutilisés : `send_deliveries` et sa saga, `schedule_idempotent_delivery_recovery`, `defer_idempotent_delivery`, `orphan_emitter_notices`, `release_stale_busy_turns`, `TurnState`, migration par reconstruction (v4), `guidance.rs` et `bridget_queue` (156), `test_sync::checkpoint`.
- Créés : une phase `held`, une phase `cancelled`, une colonne `queue_rank`, un réglage de registre, une table `queue_edits`, deux trames client, un champ optionnel de `Register`.
- Écartés : tombe durable chez le porteur, saga d'édition, snapshot du porteur, trames daemon vers porteur, `execution_queue`, `edit`, extension aux fils T3.

<a id="extrait-6"></a>

## Extrait 6

| Niveau | Contenu |
|---|---|
| Rouge d'abord | un envoi vers un agent `busy` est poussé tout de suite ; `bridget_queue` refuse les actions d'écriture ; `DROP TABLE` supprime les liens d'exécution ; avant le lotA, le précontrôle ancien refuse7 ; codeA livré accepte7 et refuse8. Le RED du lotB porte les nouvelles éditions |
| Migration | `held159_migration_*` |
| Rétention et libération | `held159_hold_*`, `held159_rank_*`, `held159_release_*`, `held159_sweep_*`, `held159_list_*` |
| Pannes | `held159_crash_*`, `edit159_cancel_*` avec `test_sync::checkpoint` |
| Édition | `edit159_receipt_store_*`, `edit159_rights_*`, `edit159_cancel_*`, `edit159_reorder_*`, `edit159_promote_*` |
| Transports | `edit159_promote_transport_*` |
| Bout en bout | `held159_e2e.rs`, `edit159_e2e.rs` |
| Mesure | latence fin de tour vers début du message suivant, avant et après |
| Non-régression | `spec_087`, `native148`, `native149`, `native150`, `guidance156`, `t3code`, `mcp::`, transport |
| Suite complète | recette `--workspace --no-fail-fast` |

<a id="extrait-7"></a>

## Extrait 7

Livraison par ROOT dans le pipeline152–160 déjà autorisé : commit, intégration,
push, build signé, installation et activation conditionnelle de Bridget, puis
nettoyage. T011/SC006 reste préalable : sauvegarde cohérente et restauration
isolée avec le vrai ancien binaire158B installé, empreinté et conservé avant
reconstruction159. Contrôler zéro mission native non terminale avant activation.
T3 reste sans relance explicite. `held_queue` reste fermé.
Diffs enregistrés par lot (`validation/diff-159a.patch`, `validation/diff-159b.patch`).
Le lotA a déjà livré la migrationv7, même réglage fermé. Ses preuves de sauvegarde et restauration restent historiques. Le lotB ne relance pas cette migration : `queue_edits` est créé par `CREATE TABLE IF NOT EXISTS` sur la connexion existante, sansv8 ni copie/reconstruction de `send_deliveries`.
