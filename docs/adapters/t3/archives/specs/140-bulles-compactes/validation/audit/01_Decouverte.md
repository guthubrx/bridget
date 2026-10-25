# Archives techniques de l’adaptateur T3

Document d’origine : [specs/140-bulles-compactes/validation/audit/01_Decouverte.md](../../../../../../../../specs/140-bulles-compactes/validation/audit/01_Decouverte.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le scope comprend quatre fichiers Rust :message.rs, communication.rs, daemon.rs, t3code.rs ; deux fichiers production T3 MessagesTimeline.logic.ts et MessagesTimeline.tsx ; leurs deux fichiers de tests ; un asset SVG. Aucun manifeste de dépendance modifié, nouveau service, API ou migration.

<a id="extrait-2"></a>

## Extrait 2

Tests et outils ciblés :269 frontend et67 Rust distincts PASS. Lint baseline22 warnings identiques, types/build/fmt et cargo check --workspace --all-targets --offline PASS. Recette Browser T3 isolée :16 cartes desktop,12 mobile320px par iframe même origine, copie brute7357 exacte, corps lisible7154 et ancrage623stable. Les limites de charge/recyclage sont dans manifest.json, results.json et scoring.md.
