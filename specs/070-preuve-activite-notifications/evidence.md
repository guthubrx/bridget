# Preuves d'exécution - SPEC-070

**Date** : 30 août 2026
**Portée** : worktree isolé, aucune livraison ni redémarrage.

## Commandes validées

| Commande | Résultat |
|---|---|
| `node --test crates/bridget-daemon/assets/ui/app.js` | 71 tests réussis |
| `/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib -- --quiet` | 651 réussis, 7 ignorés |
| `/home/moi/.cargo/bin/cargo test -p bridget-transport --lib -- --quiet` | 218 réussis, 1 ignoré |
| `/home/moi/.cargo/bin/cargo fmt --check` | réussi |
| `git diff --check` | réussi |

## Couverture apportée

- Une acceptation ou un `prompt_dispatched` seul ne crée aucune activité.
- Un acte fournisseur produit une ligne compacte sûre, retirée au terminal.
- Une réponse suivie conserve l'échéance fournisseur configurée, au-delà de 60 s.
- Une échéance fournisseur écrit `turn/interrupt` puis produit un terminal d'échec unique.
- Une erreur terminale remplace ou met à jour la bulle optimiste du bon message, même sans corps répété par le journal, et expose une référence de journal dépliable.
- Une notification exige message suivi, terminal attesté, permission accordée, arrière-plan et absence de doublon.

## Vérifications non réalisées

- Vérification navigateur automatisée : l'outil de navigation a échoué avant chargement de page, car Chromium Playwright manque à l'emplacement `/Users/moi/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell`.
- Vérification manuelle de permission de notification et de clic : elle demande un navigateur interactif, après livraison de la branche.
- Notification navigateur lorsque la page est fermée : hors périmètre de cette SPEC, sans Push API ni service worker.
