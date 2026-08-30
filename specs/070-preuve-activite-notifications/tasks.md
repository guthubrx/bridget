# Tâches - SPEC-070

## Fondations et régression du délai - P1

- [x] T001 - Ajouter les témoins Rust qui distinguent suivi de réponse et
  échéance fournisseur dans `crates/bridget-daemon/src/daemon.rs`. Preuve : une
  demande humaine avec réponse conserve l'échéance Codex configurée au-delà de
  60 secondes.
- [x] T002 - Corriger la pose de `deadline_at` dans
  `crates/bridget-daemon/src/daemon.rs` sans modifier le suivi métier de la
  réponse. Preuve : le témoin T001 passe et l'interruption explicite conserve
  son délai court.
- [x] T003 - Ajouter le témoin transport dans
  `crates/bridget-transport/src/codex_app_server.rs` pour une échéance réelle.
  Preuve : `turn/interrupt` est écrit avant un unique terminal d'échec.
- [x] T004 - Corriger `wait_for_turn` pour interrompre proprement l'exécution à
  son échéance fournisseur. Preuve : T003 passe, sans régression des témoins
  de steering SPEC-063.

## Activité réelle et erreur corrélée - P1

- [x] T005 - Ajouter les témoins Node de projection d'un tour actif dans
  `crates/bridget-daemon/assets/ui/app.js`. Preuve : aucun événement de remise
  seul ne crée d'activité ; un acte fournisseur crée une activité et un
  terminal la clôt.
- [x] T006 - Rendre la présence compacte de l'agent au-dessus du compositeur
  dans `index.html`, `theme.css` et `app.js`. Preuve : bouille existante et
  dernier libellé sûr s'affichent seulement pendant un travail prouvé.
- [x] T007 - Corriger l'état local de remise et l'erreur terminale corrélée
  dans `app.js`. Preuve : `prompt_dispatched` ne prétend plus que l'agent
  travaille ; l'erreur durable est reliée au message concerné et détaillable.

- [x] T012 - Afficher trois points animés pendant la seule remise, dans
  index.html, theme.css et app.js, puis les retirer au premier acte réel ou au
  terminal corrélé. Preuve : témoin Node de corrélation et asset livré par le
  relais UI courant.

## Notification et navigation - P2

- [x] T008 - Ajouter les témoins Node de permission, déduplication et ciblage
  de notification dans `app.js`. Preuve : aucun terminal absent, premier plan,
  refus de permission ou doublon ne notifie.
- [x] T009 - Ajouter le contrôle explicite de notification et le clic vers la
  cible `agent + message_id` dans `index.html`, `theme.css` et `app.js`.
  Preuve : une réponse ou erreur réelle notifie et ramène au bon message ; la
  puce interne reste le repli.

## Vérification et documentation

- [x] T010 - Exécuter les tests Node de l'asset, les tests Rust ciblés daemon
  et transport, puis consigner les résultats dans `spec.md`. Preuve : toutes
  les commandes passent ou chaque blocage est nommé.
- [x] T011 - Mettre à jour les critères de validation de `spec.md`, réaliser
  une auto-revue minimalisme et préparer la convergence. Preuve : chaque
  exigence possède une preuve code ou test.
