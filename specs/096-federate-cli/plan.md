# Plan 096 — Commande federate intégrée

Branche session-096-federate-cli, 2026-09-07. Source : spec.md.

## Résumé
Intégrer l'accès utilisateur au mécanisme095 autonome, sans seconde implémentation des services. Le binaire embarque le script canonique à la compilation ; un point d'entrée CLI adapte URL/options et découverte à ses fonctions existantes. Les données utilisateur ne deviennent jamais du code shell.

## Contexte technique
Rust existant, Bash3.2+, macOS/Linux, dépendances actuelles seulement. Configuration et reçus095 conservés. Un module étroit dans crates/bridget-daemon/src/federate.rs raccordé à cli.rs/lib.rs ; scripts/federate-ssh.sh reste l'autorité de gestion native. Découverte bornée et parcours linéaire des installations.

## Gates constitution
Session096 validée ; worktree dédié contenant la source cumulative094+095, sans modifier ces worktrees durant le développement. Pas de commit opportuniste des WIP antérieurs. Scripts officiels de setup/prérequis absents : artefacts rédigés suivant les modèles lus. Synchronisation projet seule effectuée, aucune publication globale SpecKit. Mémoire projet et commande mem absentes, observations consignées ici.

La couche nouvelle porte uniquement ergonomie et embarquement autonome. Aucun framework, crate réseau, registre parallèle ou gestionnaire natif supplémentaire. Tests ciblés avant revue, fmt/clippy workspace et binaire release après convergence. Ne pas redémarrer daemon/tunnel actifs pour démontrer la réutilisation.

## Organisation
- Root : artefacts, revue, intégration et recette finale.
- Auteur : module CLI/script canonique et tests096, docs FR/EN/skill sur ownership explicite.
- Reviewer : lecture indépendante des frontières, erreurs, autonomie et maintien de095.

## Validation
Tests unitaires du parseur/adaptateur et véritable exécution du binaire isolé avec gestionnaires/SSH doublés aux seules frontières. Tests095/089 de script, inventaire CLI094, paquet autonome. Recette non destructive sur Cartae : commande demandée, statut, PID/reçus inchangés et who distant. Aucun retrait du tunnel de production.

## Livraison
Construire à partir de ce snapshot cumulatif, publier binaires Mac/Linux avec sauvegarde des fichiers remplacés, sans reload des processus déjà vivants. Documenter précisément ce qui reste chargé en ancienne version.
