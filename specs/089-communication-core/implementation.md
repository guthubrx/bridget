# Journal de réalisation — 089

## 2026-09-05 — P0 vérifiée et baseline T005 partielle

T002 : gel complet de familles, 17 fichiers, référence produit dfa2134 et
capture bd1cbe0 épinglées séparément. Vérificateur --self-test --require-complete
exit 0, six mutants refusés. T003 : 264 fichiers historiques classés, zéro
oublié/doublon, 12 critères reliés aux scénarios Gherkin. T004 : 14 frontières
de confiance. T006 : revue indépendante PASS sur la stratégie, sans transformer
les gates non exécutées en succès.

| Commande réelle | Résultat et durée |
|---|---|
| PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline --workspace --no-run | Compilation de toutes les cibles, exit 0, 32,12 s ; aucun test lancé par cette commande |
| PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline -p bridget-core --lib | 39/39, 1,10 s |
| env -i PATH=/usr/bin:/bin TMPDIR=<racine privée> HOME=<racine privée> XDG_DATA_HOME=<racine privée> BRIDGET_ARTIFACT_ROOT=<racine privée>/artifacts target/debug/deps/bridget_daemon-b7902f0f20bc17cc 'store::tests::' --test-threads=4 | 43/43, 0,95 s ; inclut receipt_store et artifact_blob_store par filtre, aucun daemon lancé |
| même environnement fermé, filtre 'idempotency::tests::' | 39/39, 0,20 s |
| PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline -p bridget-daemon --test mission_boundary_test | Rouge PRÉEXISTANT confirmé : « Maicie ne doit être disponible que pour les fixtures de test » ; le manifeste produit dépend de Maicie. Le test est conservé, T009 doit fermer ce défaut. |

Les racines des deux lots SQLite sont créées par mktemp -d /tmp/bg089-store.XXXXXX
et /tmp/bg089-idem.XXXXXX ; aucune variable BRIDGET_AGENT ni home de production
n'est héritée. Les tests utilisent uniquement leurs DB de fixture.

L'audit a trouvé que le daemon historique peut ramasser le TMPDIR partagé même
avec HOME isolé. Le plan avance donc T007 avant les bancs de daemon de T005 :
isoler avant d'exécuter, pas un skip de gate. La baseline totale, les fournisseurs
réels et SSH restent non validés. L'utilisateur a autorisé les revues/validations
et le travail complet ; aucune bascule de la flotte n'est nécessaire ni engagée.

## 2026-09-05 — Matérialisation du codec de référence (suite T002)

42 trames de cinq familles passent dans le codec de production inchangé de
dfa2134 : envoi/reply/idempotence (sept issues), annuaire/ledger non vide,
spawn/stop (cinq issues), attach et claim/lease/réponse guichet. Les entrées
sont des scénarios de caractérisation ; les sorties sont une capture du codec,
pas des chaînes devinées. `core_089_wire_test` relit ces sorties indépendantes
et compare leur émission octet pour octet. Le générateur est ignoré par défaut
et ne réécrit jamais les attentes. Cela protège le fil, pas encore le canon SQL
ni la livraison réelle, réservés aux gates T014/T018.

Deux lignes natives sont extraites des faux fournisseurs historiques :
codex_app_server.rs:3303 et claude_stream_json.rs:1467 au commit source.
Les tests de session consomment les vrais flux de ces sous-processus et
comparent raw/source/origine aux fixtures, sans normalisation des espaces.
Ce n'est pas une recette auprès des comptes fournisseurs réels.

Commandes : `cargo test --offline -p bridget-transport --test core_089_wire_test`
(1 réussi, 1 générateur ignoré, 0,00 s) et `cargo test --offline -p
bridget-transport --lib session_native_ -- --test-threads=2` (2 réussis,
0,04 s ; faux fournisseurs, TMPDIR=/tmp/bg089-native.QwxMC2).
Le codec protocol.rs et le modèle core/message.rs sont identiques à dfa2134
(`git diff dfa2134 --` sur ces deux sources : vide).

Self-review : aucune dépendance ni DTO produit ajouté ; golden externe nécessaire
car les anciens round-trip se comparaient principalement à eux-mêmes. Les
attentes incluent corps UTF-8, espaces, corrélation et limites déclarées. La
preuve runtime du daemon reste distincte. Seuls les deux oracles de tests
natifs changent dans les sources, pas les pilotes.

Baseline qualité : Clippy a aussi révélé trois erreurs préexistantes côté
transport (variante ACP trop volumineuse, Default dérivable, format constant).
Corrections ciblées : message terminal dans Box, restitué intact à l'adaptateur ;
Default dérivé identique et literal JSON de test. Le test structurel borne la
taille de l'événement et vérifie le message restitué. `cargo clippy --offline
-p bridget-transport --all-targets -- -D warnings` passe ; `acp::tests::`
passe 41 tests, 1 ancien micro-banc ignoré, 4,09 s. Ces corrections suivent la
capture initiale ; elles ne modifient aucun octet filaire. Le contrôle global
fmt ne signalait que les deux include_bytes nouveaux, désormais formatés.

## 2026-09-05 — Préparation uniquement

Base : dfa2134dcfe2a2522e3ae77d93561e6ae72556b3, main de l'ancien dépôt. Clone indépendant créé par :

```sh
git clone --no-local --no-hardlinks --single-branch --branch main /Users/moi/Nextcloud/10.Scripts/bridget /Users/moi/Nextcloud/10.Scripts/XX.bridget
```

Résultat : exit 0, 0,5 s observée. L'origine locale a ensuite été retirée du nouveau clone pour empêcher un push accidentel vers l'ancien dépôt. Aucune identité Git modifiée.

Worktree dédié : `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core`, branche `session-089-communication-core`, créée par le hook SpecKit git-feature. Synchronisation SpecKit limitée au nouveau projet ; aucun adaptateur global modifié. La préparation officielle plan/tasks résout correctement ce dossier via feature.json.

`cargo metadata --no-deps --format-version 1 --offline` : exit 0. Quatre membres encore présents, dont Maicie ; la dépendance directe maicie du daemon est constatée. **Le clone n'est pas encore le noyau extrait.**

Le dépôt original présentait les modifications suivantes avant et après préparation : Cargo.toml de l'application desktop modifié ; répertoires .claude/.gstack et fichier watch_20260825-084049 non suivis. Rien de ce WIP n'a été copié ni modifié. Aucun daemon, wrapper, client de production, tunnel ou outil de communication inter-agent n'a été lancé.

## Registre des preuves à fournir

Vérifications de préparation : IDs T001–T036 uniques et ordonnés, aucun placeholder de template restant, prérequis plan/tasks reconnus, métadonnées Cargo lues hors ligne. Le vérificateur SpecKit refuse le préfixe Git `session-` sans sélection explicite ; il passe avec `SPECIFY_FEATURE=089-communication-core`, sans patch des scripts officiels. Ce résultat porte sur les artefacts, pas sur le logiciel.

SC-08901..SC-08912 : **NON EXÉCUTÉS**. Aucun test Rust, gate fournisseur, crash-test ni scénario SSH réalisé lors de cette préparation. Les vérifications documentaires et Git ne valent pas non-régression de l'extraction.

Chaque future entrée doit contenir : commit, commande exacte, espace de test, résultat, durée, oracle et éventuel mutant ; liste séparée des gates non exécutés. Les résultats préexistants des anciens chantiers ne sont pas réattribués à 089.

## 2026-09-05 — T001 achevée, T002 partielle

L'utilisateur a autorisé la poursuite de l'inventaire et des tests avec les deux cases préalables ouvertes. La revue reste obligatoire avant suppression. Aucun code de production ni données utilisateur modifiés.

T001 : baseline.md couvre les 45 modules du daemon et 22 modules core/transport, les sept sources de schéma SQL, les commandes du dispatch et les résolutions de chemins. La comparaison des tables aux trois lib.rs ne laisse aucun module racine sans disposition. La lecture a rectifié le propriétaire de DaemonConfig (daemon.rs, pas runtime.rs) et identifié un mélange fixture de forme/contrat filaire Codex ; aucune correction opportuniste du pilote.

Première tranche T002 : 13 fixtures historiques, manifeste et vérificateur local. Toutes les familles restantes sont nommées ; **T002 n'est pas cochée**. Cette livraison partielle empêche précisément de déclarer un corpus complet à partir des seules trames faciles à copier.

| Commande exécutée dans le worktree 089 | Résultat | Durée observée |
|---|---|---|
| PATH=/Users/moi/.cargo/bin:$PATH cargo test --offline -p bridget-transport --lib protocol:: -- --test-threads=4 | 65 réussis, 0 échec, 0 ignoré, 184 filtrés ; aucun processus fournisseur/daemon | Compilation 9,73 s ; tests 0,03 s |
| sh -n scripts/verify-089-contracts.sh | Syntaxe shell valide | Incluse dans le lot de vérification <1 s |
| sh scripts/verify-089-contracts.sh --self-test | 13 fichiers égaux à Git ; 6 mutants refusés | <1 s |
| sh scripts/verify-089-contracts.sh --require-complete | Exit 1 attendu : six familles manquantes | <1 s |

### Self-review XIX/XX

- Nécessité : protéger les bytes et distinguer manque de fixture de régression fonctionnelle avant extraction.
- Choix : un seul vérificateur local, Git + bibliothèque standard Python déjà requise par l'outillage ; aucune dépendance du produit ajoutée, aucun nouveau framework.
- Hypothèse : le commit source est accessible dans le clone indépendant ; contrôlé par Git. Un fichier de fixture historique n'est pas nécessairement un protocole valide en production.
- Vérifié : lecture des coutures, couverture des modules, hash + comparaison aux objets, échecs discriminants et tests de protocole.
- Non vérifié : suite complète, crashs, pilotes réels et SSH ; aucun SC de livraison déclaré clos.
- Complexité évitée : aucun parseur Rust maison pour fabriquer des fixtures depuis les sources ; aucun nouveau DTO, daemon ou magasin de données.
- Charge de maintenance : un manifeste relie chaque fixture à son origine ; une seule liste fermée des familles oblige à rendre visibles les lacunes.
