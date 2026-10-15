# Développement 168 — lot B

Date : 2026-10-15.

## Publication partielle

T002 a publié les contrats fermés de transport et Lineage : AutoDelegateV2,
workload, expected_base_commit, routing_catalogue_v1, CandidateProbeV2,
HostResourceSnapshotV1, IDs c001..c128 et LineageDecisionV4.
Les chemins v1, IDs c01..c08 et Lineage v1/v2/v3 sont conservés.

T013/T016/T024 ont ajouté `host_resources.rs` et le parcours worker de
CandidateProbeV2. Le snapshot de la requête a été retiré : le worker conserve
son collecteur et retourne son propre relevé une seule fois par lot. La
capacité compilation reste null tant que C ne transmet pas la valeur déclarée
du registre. La connexion est cherchée dans `routing_candidates()` sur
machine, connection_id, moteur, type, modèle et effort avant le probe lecture
seule ; l'absence refuse sans Prepare, slot, worktree ni fournisseur.
Le SHA attendu est transmis au champ `WorkerPrepare.base_commit` existant et
réemploie ainsi les préconditions Git et `verify_base_commit` existants.

## Contrôles réalisés

- `rustfmt --edition 2024` ciblé sur les fichiers sous ownership B ; succès.
- `git diff --check` ; succès.

Cargo, tests, build et services ne sont pas exécutés : ils restent interdits
sans permis explicite ROOT. Les oracles ajoutés ne constituent donc pas une
preuve PASS.

## Suite

T032 a reçu des oracles de sérialisation/forme dans protocol et Lineage ; les
scénarios fixture complets et les preuves d'exécution restent à produire par
ROOT avec le permis Cargo. Un oracle source168 vérifie les bornes128/v1 dans
`remote_worker_162.rs`. La campagne future ciblée est :
`cargo test -p bridget-daemon --test remote_worker_162` suivie du filtre
transport168, exclusivement avec le permis ROOT.
Le module `host_resources` doit être enregistré dans `daemon/src/lib.rs` par C,
son propriétaire exclusif. Aucun Prepare, worktree, slot ou fournisseur n'est
créé par cette publication.
