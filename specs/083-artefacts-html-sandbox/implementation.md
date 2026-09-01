# Preuves d'implémentation - SPEC-083

## Correctif de publication HTML - 2026-09-01

- Observation de production : malgré le serveur MCP déclaré, un agent Codex
  pouvait répondre par un bloc HTML brut et affirmer à tort que la sandbox
  était indisponible.
- Correction : chaque tour Codex géré reçoit désormais une consigne explicite
  et persistante rappelant `bridget_publish_artifact`, `kind: html` et
  l'interdiction du faux refus avant l'appel de l'outil.
- Preuve ciblée : `cargo test -p bridget-transport --lib
  codex_app_server::tests::consigne_outil_bridget_exige_la_publication_html_avant_un_refus -- --exact`.

## Validation runtime finale - 2026-09-01

- Observation de production : le Desktop pouvait conserver un panneau attaché
  à un tunnel SSH local mort. Le serveur publiait alors bien l'artefact, mais
  la conversation affichait un relais illisible et ne recevait pas le rendu.
- Correction Desktop : la perte de tunnel passe désormais par une reconnexion
  automatique, recrée le tunnel et retargete la WebView existante vers son
  nouveau port local. Une génération de connexion empêche qu'un ancien watcher
  remplace une session reconnectée plus récente.
- Corrections renderer : la projection de snapshot référence désormais le
  normaliseur disponible pour les références publiées et le prédicat de
  demandes fournisseur est défini dans la portée partagée. Le corps JSON d'un
  ticket sandbox conserve la référence canonique, distincte de la forme URL.
- Correction sandbox : l'hôte transmet explicitement sa racine au factory,
  permettant la création de l'iframe sans accéder à une variable hors portée.
- Preuves : `node --test crates/bridget-daemon/assets/ui/app.js` (129 verts),
  `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml` (60
  verts), puis test navigateur local sur le relais distant : deux détails,
  deux tickets et deux lectures sandbox répondent 200, avec deux iframes
  créées sans erreur JavaScript. La même publication apparaît ensuite dans la
  fenêtre Bridget installée, avec son cadre sandbox, Agrandir, Enregistrer et
  Exporter JSON.

## Correctif du canal dynamique Codex - 2026-09-01

- Cause observée : le catalogue MCP était bien chargé par Codex, mais le moteur
  pouvait demander `bridget_publish_artifact` par la requête app-server
  `item/tool/call`. Bridget la traitait comme inconnue, écrivait
  `unsupported_provider_request`, puis Jim terminait sans publication.
- Correction : ce canal dynamique ne reconnaît que
  `bridget_publish_artifact` et le délègue au même validateur, à la même
  identité d'agent et au même reçu que MCP stdio. Les autres outils restent
  explicitement refusés.
- Preuve ciblée : `cargo test -p bridget-transport --lib
  codex_app_server::tests::outil_dynamique -- --nocapture`.

## Correctif - portée de conversation sans projet - 2026-09-01

- Une conversation de test sans projet actif recevait correctement la demande
  mais refusait ensuite la publication avec `project_context_unavailable`.
- Bridget crée désormais une portée privée stable `conversation-agent:<agent>`
  depuis l'identité déjà authentifiée du wrapper. Le navigateur ne fournit
  jamais cette valeur. Dès qu'un projet actif existe, sa portée attestée reste
  prioritaire.

## Correctif - élicitation MCP de publication - 2026-09-01

- Le protocole Codex peut demander une élicitation MCP avant l'appel de
  publication. Bridget accepte uniquement son formulaire vide, issu de son
  propre serveur, et répond `decline` à toute demande de donnée, URL ou serveur
  tiers. Cela ne contourne aucune autorisation et ne journalise ni le texte de
  la demande ni une éventuelle valeur sensible.
- Preuve ciblée : `cargo test -p bridget-transport --lib
  elicitation_mcp_vide_de_bridget_est_acceptee_et_un_formulaire_est_refuse
  -- --nocapture`.

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
- Les agents Codex gérés reçoivent la même déclaration MCP statique que le
  client Codex, injectée avant la sous-commande `app-server`. Ce n'est pas un
  faux équivalent CLI : le catalogue réel du fournisseur expose alors
  `bridget_publish_artifact`, y compris `kind: html`, et peut créer la
  publication que le renderer sandboxé affichera inline.
- Le renderer HTML n'a aucun accès réseau, cookie, secret, fichier local ou IPC. Une source ne peut être ouverte dans Browser qu'après un clic opérateur.

## Commandes vertes

- `/Users/moi/.cargo/bin/cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --lib --tests`
- `/Users/moi/.cargo/bin/cargo check --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --target aarch64-apple-darwin`
- `cd /private/tmp/bridget-project-nav.JNWHqE/crates/bridget-daemon/assets/ui && npm test`
- `/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test artifact_service_test --test artifact_lifecycle_test --test artifact_publication_test --test artifact_store_test --test artifact_policy_test`
- `/Users/moi/.cargo/bin/cargo fmt --check`
- `git diff --check`
- `/Users/moi/.cargo/bin/cargo test -p bridget-daemon spawn_gere_injecte_mcp_identite_et_path`
- `/Users/moi/.cargo/bin/cargo test -p bridget-daemon mcp::tests::publication_html_est_annoncee_au_moteur_comme_un_artefact_sandboxe --lib`
- Sondage protocolaire Codex app-server : `mcpServerStatus/list` retourne le
  serveur `bridget` et l'outil `bridget_publish_artifact` avec le contrat HTML.

Les suites Desktop ont validé le contrat sandbox, les préférences migrées, le panneau Browser unique, les capabilities sans privilège et l'absence de secrets ou IPC dans les surfaces non fiables. La compilation macOS dédiée a vérifié les APIs WebView Tauri utilisées par la coque, y compris le profil et l'effacement des données.

## Limite connue de la suite globale

La suite globale du daemon contient des tests historiques qui ne sont pas exécutables depuis un worktree temporaire long : certains dépassent la limite macOS `SUN_LEN` des sockets Unix et d'autres utilisent des identifiants de test désormais invalides. Les suites dédiées aux deux spécifications sont vertes et ne masquent pas ces échecs préexistants.
