# Plan d'implémentation - SPEC-081 Flotte globale et sources Desktop

**Branche** : `session-081-flotte-globale-sources-ui`  
**Worktree** : `/home/moi/bridget-referent/.worktrees/session-081-flotte-globale-sources-ui`  
**Base** : `origin/main` 29c81424b28d2607d871a0feed67e3941717e2f9  
**Statut** : Planifié, implémentation non commencée

## Résumé

Bridget Desktop devient la coque de navigation multi-sources. Il lit un instantané borné de chaque tunnel actif, réduit les données à une projection sans secret ni chemin, puis affiche Sources, filtres, tri et flotte dans la page locale. L'ouverture d'une carte conserve un unique panneau enfant placé à droite, chargé avec le tunnel de l'agent choisi. Le relais distant passe en mode compact et n'affiche plus ses colonnes Projet et Agents.

Le daemon reste l'autorité de chaque source. Desktop reste l'autorité de ses préférences locales. Le relais reste l'autorité de la conversation.

## Surfaces et responsabilités

| Surface | Responsabilité |
|---|---|
| `apps/bridget-desktop/src-tauri/src/connection.rs` | Construire les chemins authentifiés de lecture et l'URL compacte d'un agent, exclusivement dans Rust. |
| `apps/bridget-desktop/src-tauri/src/connection.rs` | Découvrir dynamiquement le relais local par la commande constante `bridget ui endpoint --json`, valider son endpoint et le garder uniquement en mémoire sous le libellé « Cet ordinateur ». |
| Nouveau `apps/bridget-desktop/src-tauri/src/fleet.rs` | Désérialiser les réponses privées, éliminer chemins et jetons, joindre projets et agents, produire les DTO testables. |
| `apps/bridget-desktop/src-tauri/src/lib.rs` | Capturer sessions sans verrou pendant le réseau, consommer le transport existant, exposer `fleet_snapshot`, ouvrir la conversation dans sa source et réserver la zone enfant. |
| `apps/bridget-desktop/src-tauri/src/preferences_store.rs` | Ajouter épingles, exclusions de pré-épinglage et critères de tri locaux versionnés. |
| `apps/bridget-desktop/ui/index.html`, `app.js`, `fleet-app.js`, `fleet-presentation.js`, `fleet-desktop.css` | Rendre Sources, chips de filtre et de tri, liste d'agents, épingles et gestionnaire de serveurs accessible. |
| `crates/bridget-daemon/assets/ui/app.js`, `theme.css` | Respecter `desktop_shell=1` sans ajouter de capacité Tauri. |
| Tests Desktop et daemon | Prouver origine, projection sans données sensibles, tri, épingle et géométrie. |

## Séquence test-first

1. Écrire les témoins Rust de projection et les témoins JavaScript purs des filtres, clés composées, tri et épingle.
2. Découvrir le relais local sans profil persistant, puis créer `DesktopFleetSnapshotV1` dans `fleet.rs`.
3. Ajouter les chemins de relais, le snapshot multi-sources et les commandes Tauri, sans exposer jeton, URL, détail SSH ou chemin canonique.
4. Faire évoluer les préférences locales par ajout compatible.
5. Recomposer la coque Desktop visible : Sources à gauche, flotte au centre-gauche, conversation enfant à droite.
6. Ouvrir un agent avec `desktop_shell=1` et `agent`, sans dépasser un panneau enfant.
7. Ajouter le mode compact du relais avec ses tests.
8. Lancer tests ciblés, formatage, clippy et workspace.
9. Rejouer audit de réutilisation, Analyze, convergence et contre-revue adverse.

## Invariants

- Une action sur `A` est toujours routée via la source `S` de la clé `S:A`.
- Aucun jeton, URL de relais, détail SSH ou `canonical_path` ne traverse `fleet_snapshot`.
- Aucun mutex de session ou de profil n'est tenu pendant le réseau.
- « Cet ordinateur » n'apparaît que si son endpoint local versionné est valide et que sa lecture de snapshot réussit. Son jeton reste en mémoire Rust.
- Une source inaccessible n'affecte ni la visibilité ni les actions des autres sources.
- Une carte homonyme affiche toujours source et projet textuellement.
- Le pré-épinglage n'existe que pour `agent_link.role == "coordinator"`, jamais par nom.
- Les épingles, filtres et tris ne modifient jamais un serveur.
- La WebView distante n'obtient ni capacité Tauri ni accès aux autres sources.
- La projection est O(s + a + p), le tri O(a log a).

## Constitution Check

| Contrôle | Verdict | Justification |
|---|---|---|
| Worktree isolé | PASS | Branche et worktree dédiés depuis `origin/main`. |
| Réutilisation avant création | PASS | Sessions, transport, snapshot, panneau isolé et store atomique sont consommés. |
| État durable minimal | PASS | Métier sur les sources, seulement préférences d'affichage locales. |
| Sécurité fail-closed | PASS | Source hors ligne visible mais non actionnable, secret confiné à Rust. |
| Complexité | PASS | Projection bornée, sans parcours quadratique. |
| Compatibilité | PASS sous test | Une source non conforme devient indisponible, sans fallback inventé. |
| Déploiement | Hors périmètre | Arrêt explicite avant livraison demandé. |

## Risques

| Risque | Réduction |
|---|---|
| `/v1/projects` retourne `canonical_path`. | DTO privé puis projection publique réduite, avec test d'absence. |
| Homonymes sur deux serveurs. | Clé composite et `source_id` obligatoire lors de l'ouverture. |
| Le relais local n'est pas lancé ou Bridget n'est pas installé. | La source est absente, sans configuration manuelle ni faux statut connecté. |
| Parent Tauri masqué par l'enfant. | Enfant décalé à droite, test de géométrie et coque de largeur constante. |
| Coordinateurs historiques sans rôle. | Pas de pré-épinglage spéculatif, épingle manuelle locale. |
| Serveur incompatible avec mode compact. | Échec isolé par source, vérification manuelle avant livraison. |

## Décision

L'architecture est consignée dans `docs/decisions/022-coque-desktop-flotte-globale.md`.
