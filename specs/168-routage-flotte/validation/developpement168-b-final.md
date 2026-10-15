# SPEC168 — lot B — reçu source final

Date : 2026-10-15. Périmètre T013/T016/T024/T032. Travail effectué exclusivement sur Cartae par SSH dans `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte`. Ce reçu décrit du code et des scénarios source. Aucun succès d'exécution n'est déduit.

## Interface publiée et coordination

ROOT a autorisé l'extension additive suivante dans `WorkerRequestOperation` :

```rust
PrepareV2 {
    prepare: WorkerPrepare,
    workload: AutoDelegateWorkload,
    connection_id: String,
    engine: String,
    expected_definition_digest: String,
    expected_permission_digest: String,
}
```

`workload` est obligatoire. `WorkerPrepare` v1 est inchangé. L'identité de commande reste une préparation : instance vide et génération zéro. Les digests sont des SHA-256 et `engine` vaut `codex` ou `claude`. La connexion doit avoir négocié `WorkerCapability::CandidateProbeV2`. Les liens163 n'autorisent ni le probe168 ni PrepareV2.

La signature de mesure reste :

```rust
HostResourceCollector::collect(&mut self, now: i64, compilation_slots: Option<u8>) -> HostResourceSnapshotV1
```

Le résultat envoyé est exactement `WrapperToDaemon::CandidateProbeResultV2 { result: CandidateProbeV2Result }`. ROOT et C ont reçu ces interfaces avant leur raccord. Le contrôle maître de l'époque, du temps de réception et du TTL reste sous responsabilité C. Aucun fichier C/A/D/E ni `lineage_protocol.rs` n'a été édité par ce lot.

## Réalisations

- Linux : CPU cumulés `/proc/stat` borné, sans doubler guest/guest_nice ; RAM disponible `/proc/meminfo` borné. macOS : statistiques CPU et VM Mach, mémoire physique par `sysctlbyname`, taille de page native. Les ports Mach sont libérés. Charge par `getloadavg`, normalisée au nombre de CPU. Aucun processus, thread, benchmark ou dépendance ajouté.
- CPU : premier relevé inconnu, delta seulement entre deux lectures cohérentes ; échec, recul du compteur, recul de l'horloge ou nouvelle époque coupent la série. RAM et charge invalides restent inconnues. La RAM Mac compte les pages libres et inactives sans additionner des catégories qui se recouvrent.
- Capacité compilation : uniquement la valeur déclarée `RoutingMachinePolicy.compilation_capacity` (0..10), jamais une estimation CPU/RAM. L'absence reste inconnue.
- Collecteur conservé dans le worker, rafraîchi par le tick existant à cadence10s et une fois par lot de probe168. Reset lors de chaque acceptation d'époque. Cache de lecture fermé sur temps futurs/périmés et TTL invalide.
- Probe168 : jusqu'à128 options, index de registre local par tuple machine/connexion/moteur/type/modèle ; un relevé hôte pour tout le lot. Réemploi du contrôle lecture seule existant. Aucun Prepare, reçu, worktree, token, slot, bootstrap ou fournisseur pendant le classement.
- PrepareV2 : nouvelle lecture unique du registre ; connexion exacte, machine, moteur, type, modèle et effort relus ; droits et digests frais contrôlés avant worktree/token. `agent_type` seul ne remplace jamais une connexion révoquée.
- La sélection opaque est retenue dans `PreparedRuntime` et, avec champ privé optionnel compatible, dans le reçu worker existant. Start/Release et la reconstruction des commandes retenues peuvent donc revalider cette même sélection sans saga supplémentaire. Les reçus historiques sans champ restent lisibles.
- Git168 : remote attendu et SHA source exact conservés. `WorkerPrepare.base_commit` sert à Shared et à la base future `base_ref` de Worktree ; l'absence de SHA, le SHA différent ou une base divergente refusent avant effets.
- Shared général hors Git : autorisé uniquement par les mappings explicites existants et leurs contrôles source_host/source_root/remote_root/chemins protégés/droits. L'absence de dépôt est établie par les marqueurs de fichiers et ancêtres bornés. Erreur I/O, marqueur partiel ou lien symbolique ambigu refuse. Aucun stderr/exitcode Git ne vaut preuve hors Git. SHA fourni hors Git, compilation hors Git et Worktree hors Git refusent. Le chemin162/163 conserve son comportement Git historique.

## Scénarios source ajoutés — non exécutés

Dans `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/host_resources.rs` :

- `spec168_cpu_initial_delta_inconnu_apres_erreur_et_reset_epoch` : première mesure, delta, nombre de collectes, valeurs inconnues, échec, reset d'époque, recul d'horloge, overflow, TTL.
- `spec168_linux_guest_ram_load_et_bornes_native` : guest non doublé, erreurs de lecture/parse, RAM et charge bornées.
- `spec168_darwin_cpu_et_ram_utilisent_les_primitives_natives` : scénario spécifique macOS, non exécuté ici.

Dans `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs` :

- `spec168_prepare_v2_contrat_ferme_et_prepare_v1_inchange` : forme v1 conservée, roundtrip v2, digests/connexion/moteur/identité de préparation invalides, champ inconnu et workload absent refusés.

Dans la fixture réelle `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs` :

- `worker168_probe128_connexion_revoquee_prepare_et_base_epinglee` :128 vrais candidats, snapshot unique initial, capacité déclarée, zéro effet ; révocation de connexion malgré un autre compte du même type encore actif ; changement réel de définition ; connexion/moteur/modèle/effort/digests/SHA invalides ; Worktree positif sur `base_ref` antérieure à HEAD ; révocation après Prepare refuse aussi Start avant slot/bootstrap/fournisseur.
- `worker168_reconnexion_epoch_et_capacite163_restent_distinctes` : reconnexion du même worker, ancienne époque refusée, nouveau CPU initial inconnu ; probe163 conservé ; probe168 et PrepareV2 refusés sur un lien négocié163 seulement.
- `worker168_shared_hors_git_general_sha_et_erreur_ambigue` : Shared général sans Git accepté, SHA/compilation/Worktree refusés hors Git, marqueur `.git` symbolique ambigu refusé.

Les scénarios réemploient le relay/worker/provider synthétiques existants. Aucun nouveau framework. Le petit oracle de forme128 antérieur ne remplace pas ces scénarios.

## Contrôles réellement effectués

- `/home/moi/.cargo/bin/rustfmt --edition 2024` sur les quatre sources détenues : RC0.
- `git diff --check --` sur les quatre sources : RC0.
- Un premier appel `rustfmt` sans chemin absolu a retourné127 (`command not found`) dans le PATH SSH ; aucun fichier n'avait été traité. L'appel absolu ci-dessus a ensuite réussi.
- Aucun Cargo, rustc, exécution de test, build, installation, Git en écriture, service, auth, DB ou registre de production.

Le formatage contrôle la syntaxe, pas le typage ou les résultats. Les scénarios source ne sont pas présentés comme PASS. Le code macOS doit encore être compilé et exécuté sur macOS. Les oracles de lot/epoch requièrent une vraie exécution des processus privés de fixture ; leurs effets n'ont pas été observés durant ce mandat.

## Commandes futures pour ROOT — non exécutées par B

Depuis `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte`, avec le Git>=2.44 réel prévu par la fixture et le cache partagé réservé :

```sh
export PATH=/home/moi/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target
export CARGO_BUILD_JOBS=2
cargo test -p bridget-daemon --lib host_resources::tests:: -- --test-threads=2
cargo test -p bridget-transport --lib spec168_prepare_v2_contrat_ferme_et_prepare_v1_inchange -- --test-threads=2
cargo test -p bridget-daemon --test native_delegation_e2e worker168_ -- --test-threads=2
cargo test -p bridget-daemon --test native_delegation_e2e worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement -- --test-threads=2
cargo check -p bridget-daemon --tests
```

Sur macOS, après synchronisation des mêmes sources et avec le cache local choisi par ROOT : `cargo test -p bridget-daemon --lib host_resources::tests:: -- --test-threads=2`. Ne pas réemployer le chemin de cache Linux sur Mac.

## Empreintes finales SHA-256

- `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/host_resources.rs` : `efdee93fedd247fe5b3d0c9a56d6907918839c474f8a55396394b091d8915449`
- `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs` : `8432ef5796b5b011196fdcce5420c95191cbf224df0cf2db41827f441d02ca56`
- `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs` : `c41e4ef1ca1acc2e6b40a3073ade554a867c7136ab494d000fba1013765d9301`
- `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs` : `ac7f9d1b37644ef6e98a6e4fd679099b687af74aba82291265cdc46af0915f4b`

Les fichiers de protocole/probe et du worker existaient déjà modifiés par le B précédent. Le reçu couvre leur état consolidé final ; il n'attribue pas tout le diff depuis HEAD à cette reprise. Les cases de tâches, le document central et la livraison restent à ROOT/docowner.

## Complément après première exécution ROOT

ROOT a signalé neuf tests transport168 PASS et un avertissement introduit par une fixture morte. Retrait limité au `let snapshot = HostResourceSnapshotV1 { ... }` inutilisé dans `routing168_protocol_tests::routing168_ids_et_probe_v2_sont_fermes`. Aucun nom, assertion ou comportement modifié. Aucune commande Cargo exécutée par B. ROOT exécutera séparément le filtre `routing168`, qui n'était pas sélectionné par `spec168`, sans rejouer les tests D inchangés.

## Correction de fixture après preuves ROOT R2/R3

ROOT a exécuté les scénarios worker168. R1 a refusé le Git2.43 du PATH runner. ROOT a corrigé uniquement son environnement vers Git2.56. R2 a validé le probe128/révocation/base et la reconnexion/capacité163 ; le scénario hors Git a échoué sans publier son verdict. La régression worker163 nommée a passé selon ROOT. Ces résultats appartiennent à ROOT et ne sont pas des exécutions de B.

B a ajouté seulement le verdict complet dans l'assertion du scénario hors Git. La fixture diagnostique portait le SHA `a5979c11c1192c1dc3eaf2634e124522e92fdabf6762832eecdc947b17097150`. ROOT a compilé puis rejoué ce seul scénario avec source gelée. Le log `/home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-diagnostic-r3.log` atteste `[Ineligible { candidate_id: "c001", reason: MappingUnavailable }]`.

Lecture runtime complémentaire de la fixture R2 conservée : `/tmp/n148-0f5deeaffb/project` existe et son ancêtre `/tmp/n148-0f5deeaffb/.git` est un répertoire mode0700. Le support initialise le dépôt parent. Le scénario supprimait seulement `project/.git` : son cwd restait dans un Git parent, donc le refus sans SHA était correct.

Après GO ROOT et fin de son replay, B a ajouté uniquement le retrait du dépôt parent synthétique `f.fixture.root.join(".git")` dans la préparation de CE scénario. Le retrait du dépôt imbriqué et les assertions restent présents. Aucun contrôle produit n'a changé. L'assertion diagnostique est conservée. Le correctif est source seulement, sans rejeu B.

Contrôles finaux B : `rustfmt --edition 2024 --check` sur la fixture et `git diff --check` RC0. Aucun Cargo/test/build chez B. Les SHA de host_resources, remote_worker et protocol restent inchangés. Fixture finale : `c41e4ef1ca1acc2e6b40a3073ade554a867c7136ab494d000fba1013765d9301`.

Rejeu futur ROOT uniquement : `cargo test -p bridget-daemon --test native_delegation_e2e remote_worker_162::worker168_shared_hors_git_general_sha_et_erreur_ambigue -- --exact --test-threads=2`. Les deux autres scénarios worker168 et worker163 inchangés ne nécessitent pas de répétition pour cette correction de fixture.
