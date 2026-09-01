# Preuves d'implémentation - SPEC-083

**Date** : 2026-08-31
**Branche** : `082-artifact-publication`

## Parcours couverts

- Les artefacts HTML restent des publications canoniques et immuables de Bridget. Le rendu inline passe par un iframe opaque `sandbox="allow-scripts"` avec CSP fermée, hauteur maximum de 1 200 px et protocole de messages fermé.
- L'état d'interaction est limité à 128 Kio, ne peut pas contenir d'URL et son enregistrement crée explicitement une version enfant. L'original n'est jamais modifié.
- Le Browser est un unique WebView droit à profil `Ce Mac` séparé : URLs HTTPS et publications locales seulement, aucune capability Tauri, suppression explicite des données WebView et conservation des artefacts canoniques.
- Les onglets Browser, Artefacts, Fichiers, Liens et Activité réutilisent les données attestées de Bridget. La portée reste le projet actif jusqu'à l'action explicite de recherche globale.
- Le schéma MCP `bridget_publish_artifact` expose `kind: html` et explique le
  payload sandboxé. Il interdit explicitement à l'agent de recopier le HTML
  dans son message après publication : le renderer inline reste l'unique vue
  exécutée.
- Le renderer HTML n'a aucun accès réseau, cookie, secret, fichier local ou IPC. Une source ne peut être ouverte dans Browser qu'après un clic opérateur.

## Commandes vertes

- `/Users/moi/.cargo/bin/cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --lib --tests`
- `/Users/moi/.cargo/bin/cargo check --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --target aarch64-apple-darwin`
- `cd /private/tmp/bridget-project-nav.JNWHqE/crates/bridget-daemon/assets/ui && npm test`
- `/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test artifact_service_test --test artifact_lifecycle_test --test artifact_publication_test --test artifact_store_test --test artifact_policy_test`
- `/Users/moi/.cargo/bin/cargo fmt --check`
- `git diff --check`

Les suites Desktop ont validé le contrat sandbox, les préférences migrées, le panneau Browser unique, les capabilities sans privilège et l'absence de secrets ou IPC dans les surfaces non fiables. La compilation macOS dédiée a vérifié les APIs WebView Tauri utilisées par la coque, y compris le profil et l'effacement des données.

## Limite connue de la suite globale

La suite globale du daemon contient des tests historiques qui ne sont pas exécutables depuis un worktree temporaire long : certains dépassent la limite macOS `SUN_LEN` des sockets Unix et d'autres utilisent des identifiants de test désormais invalides. Les suites dédiées aux deux spécifications sont vertes et ne masquent pas ces échecs préexistants.
