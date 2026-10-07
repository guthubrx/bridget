# Découverte bornée

Le scope comprend quatre fichiers Rust :message.rs, communication.rs, daemon.rs, t3code.rs ; deux fichiers production T3 MessagesTimeline.logic.ts et MessagesTimeline.tsx ; leurs deux fichiers de tests ; un asset SVG. Aucun manifeste de dépendance modifié, nouveau service, API ou migration.

Les points chauds du diff sont l'enrichissement à la remise, la projection d'enveloppe et les commandes de disclosure. Le titre client n'a pas autorité. Store.thread_show conserve l'appartenance ; le titre appartient seulement à la remise, pas au canon/journal. Le parser visuel n'atteste pas l'origine.

Tests et outils ciblés :269 frontend et67 Rust distincts PASS. Lint baseline22 warnings identiques, types/build/fmt et cargo check --workspace --all-targets --offline PASS. Recette Browser T3 isolée :16 cartes desktop,12 mobile320px par iframe même origine, copie brute7357 exacte, corps lisible7154 et ancrage623stable. Les limites de charge/recyclage sont dans manifest.json, results.json et scoring.md.

Duplication :JSCPD sur les fichiers de contexte Rust avec max-size5mb :35416 lignes,41 clones/734 lignes. Intersection avec lignes du diff :zéro. Deux sources production web :6936 lignes,zéro clone. Pas de baseline ; aucune conclusion fondée sur isNew/newClones.
