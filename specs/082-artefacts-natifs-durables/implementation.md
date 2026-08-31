# Preuves d'implémentation - SPEC-082

**Date** : 2026-08-31
**Branche** : `082-artifact-publication`

## Parcours vérifiés

- Publication MCP à contrat fermé, reçus idempotents, provenance attestée et
  rattachement projet/tour.
- Rendu natif local de graphique, KPI, tableau, chronologie, image et fichier,
  avec export, manifeste, sources et erreurs non muettes.
- Magasin canonique versionné, restauration contrôlée, épinglage, cache Desktop
  local évictible et recherche strictement bornée au projet par défaut.
- Référence de conversation immuable : elle conserve la version réellement
  publiée et indique explicitement lorsqu'une version plus récente existe.
- Collecte externe désactivée par défaut, HTTPS seulement, destination et
  redirections revalidées, taille bornée.
- Annulation coopérative avant écriture atomique : aucune version partielle
  n'est créée et le reçu de récupération propose une reprise à l'agent.

## Commandes vertes

```text
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test artifact_lifecycle_test --test artifact_store_test --test artifact_publication_test
/Users/moi/.cargo/bin/cargo test -p bridget-daemon relais_artefact_pagine_exporte --lib
/Users/moi/.cargo/bin/cargo test -p bridget-daemon artifact_fetch --lib
cd /private/tmp/bridget-project-nav.JNWHqE/crates/bridget-daemon/assets/ui && npm test
/Users/moi/.cargo/bin/cargo fmt --check
git diff --check
```

Les suites dédiées sont vertes : publication, isolation projet, liste/recherche,
partage, restauration, cache, annulation, collecte, relay et renderer.

## Limite de suite globale constatée

`cargo test -p bridget-daemon` ne peut pas servir de preuve globale dans cet
environnement. Avec le répertoire temporaire macOS normal, plusieurs fixtures
dépassent la longueur des sockets Unix. Avec `TMPDIR=/tmp`, cette classe diminue
mais restent des fixtures hors SPEC-082 qui supposent littéralement `/tmp` au
lieu de `/private/tmp`, ou des identifiants désormais rejetés par les règles
générales. Ces échecs sont hors artefacts et ne sont pas modifiés dans cette
spécification.
