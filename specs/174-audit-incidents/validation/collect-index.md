# Validation : collecte indexée (LIFE-COLLECT, audit 174)

## Changement

- Fichier : `crates/bridget-daemon/src/remote_worker.rs`, fonction `WorkerStore::collect`.
- Base : `0dcf930a`.
- Les deux recherches `any` (une par slot, une par commande) sont remplacées par deux ensembles `HashSet` construits une fois par appel.
- Clé d'identité complète : `command_id`, `task_id`, `instance_id`, `generation`.
- Complexité des recherches : O(slots × commandes) devient O(slots + commandes).
- Conditions, retraits, verrous, fsync, rétention et journaux sont inchangés.

## Mesure

- Fixture synthétique : 1024 commandes terminales et 3072 slots, soit 4096 entrées (`MAX_WORKER_STORE_ENTRIES`).
- Aucun slot n'atteint la rétention : aucun retrait, donc la mesure isole la lecture et les recherches.
- Médiane sur 5 appels :

| Mesure | Avant | Après |
|---|---|---|
| `collect` | 1,637 s | 1,584 s |
| `inventory` seul | 0,74 s | 0,74 s |

- `collect` appelle `inventory` deux fois. Les deux lectures disque pèsent environ 1,48 s sur 1,6 s.
- La partie quadratique pesait environ 0,15 s. Le gain mesuré est d'environ 0,05 s.
- Le bench temporaire n'est pas commité.

## Tests

- `cargo test -p bridget-daemon --lib worker1` : 35 tests passés, 0 échec.
- Nouveau test `worker174_collect_distingue_terminal_actif_et_reference` :
  - commande terminale expirée sans slot : retirée ;
  - voisine de même tâche et même instance, avec slot en attente : conservée ;
  - commande active expirée : conservée ;
  - commande terminale expirée encore référencée par un slot : conservée.
- Le test passe sur la base et après changement.

## Différences avec adaptateur de bureau

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/collect-index.md#extrait-1).

## Limites

- `command_path` ne repose que sur `command_id`. Deux générations d'un même identifiant ne peuvent pas coexister. Sur un store valide, l'index par identité complète donne le même résultat que l'ancienne comparaison.
- Une seule fixture et une seule machine. Pas de test de charge réel.
- Le vrai coût restant est le double `inventory` de `collect`. Le réduire modifierait la relecture après retraits. Ce point reste hors périmètre.
