# Journal de réalisation — 089

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
