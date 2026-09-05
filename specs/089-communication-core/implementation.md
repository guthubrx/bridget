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
