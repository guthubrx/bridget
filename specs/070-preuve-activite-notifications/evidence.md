# Preuves d'exécution - SPEC-070

**Date** : 30 août 2026
**Portée** : worktree isolé, puis livraison du seul relais UI. Le daemon et les agents ne sont pas redémarrés.

## Commandes validées

| Commande | Résultat |
|---|---|
| `node --test crates/bridget-daemon/assets/ui/app.js` | 72 tests réussis |
| `/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib -- --quiet` | 651 réussis, 7 ignorés |
| `/home/moi/.cargo/bin/cargo test -p bridget-transport --lib -- --quiet` | 218 réussis, 1 ignoré |
| `/home/moi/.cargo/bin/cargo fmt --check` | réussi |
| `git diff --check` | réussi |
| `/home/moi/.cargo/bin/cargo build --release -p bridget-daemon` | binaire de livraison construit |
| service `bridget-ui.service` + lecture de `/app.js` | actif, asset courant vérifié |

## Couverture apportée

- Une acceptation ou un `prompt_dispatched` seul ne crée aucune activité.
- Une remise seule affiche trois points animés, sans bouille ni libellé de
  travail ; le premier acte fournisseur ou le terminal les retire.
- Un acte fournisseur produit une ligne compacte sûre, retirée au terminal.
- Le port UI 17888 est désormais tenu par `bridget-ui.service`, lancé sur le binaire courant ; le daemon actif est resté inchangé.
- Une réponse suivie conserve l'échéance fournisseur configurée, au-delà de 60 s.
- Une échéance fournisseur écrit `turn/interrupt` puis produit un terminal d'échec unique.
- Une erreur terminale remplace ou met à jour la bulle optimiste du bon message, même sans corps répété par le journal, et expose une référence de journal dépliable.
- Une notification exige message suivi, terminal attesté, permission accordée, arrière-plan et absence de doublon.

## Vérifications non réalisées

- Vérification navigateur automatisée : l'outil de navigation a échoué avant chargement de page, car Chromium Playwright manque à l'emplacement `/Users/moi/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell`.
- Vérification manuelle de permission de notification et de clic : elle demande un navigateur interactif, après livraison de la branche.
- Notification navigateur lorsque la page est fermée : hors périmètre de cette SPEC, sans Push API ni service worker.

## Correctif flux vivant - 30 août 2026

| Commande | Résultat |
|---|---|
| `node --test crates/bridget-daemon/assets/ui/app.js` | 75 tests réussis |
| `/home/moi/.cargo/bin/cargo fmt --check` | réussi |
| `/home/moi/.cargo/bin/cargo build -p bridget-daemon --bin bridget` | réussi, prérequis des tests de présence |
| `/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib -- --quiet` | 651 réussis, 7 ignorés, après construction du binaire de test |
| `/home/moi/.cargo/bin/cargo test -p bridget-transport --lib -- --quiet` | réussi avant la compilation de livraison |
| projection lecture seule du journal `1aecc26942514` | 7 commandes et 7 autorisations accordées projetées chronologiquement |

La première exécution complète du test daemon a échoué sur huit tests de
présence, avant toute exécution de leur logique, avec « binaire bridget de test
absent ». La cause a été vérifiée dans `daemon.rs:17009` puis corrigée dans
l'environnement de test par la construction debug du binaire. Le même test
ciblé, puis la batterie complète, passent ensuite sans modification Rust.

Le correctif rend les événements liés à un envoi local pendant le rattrapage
initial avec un temporisateur de rendu de 50 ms côté interface. Les événements
historiques sans relation avec cet envoi restent groupés : l'ouverture conserve
donc sa protection contre la reconstruction répétée d'un historique dense.

Le flux vivant affiche : texte progressif dans sa bulle, outils au fil de leurs
événements journalisés avec le détail réellement journalisé, puis autorisation demandée, accordée ou refusée. Il ne
fabrique pas de fin ou de sortie d'outil si le fournisseur ne les journalise
pas.


Le flux vivant est désormais compact par défaut. Il affiche la dernière commande réellement journalisée, avec son autorisation corrélée si elle existe ; le bouton « Voir les N actes » révèle l’historique complet et reste ouvert pendant les événements suivants. Les détails techniques sont atténués sans être masqués.


## Correctif de remise distincte - 30 août 2026

| Commande | Résultat |
|---|---|
| `node --test crates/bridget-daemon/assets/ui/app.js` | 76 tests réussis, dont le témoin transport -> `prompt_dispatched` -> activité réelle |
| `git diff --check` | réussi |

La zone de remise utilise les événements bruts du journal uniquement pour
reconnaître `prompt_dispatched` corrélé au `message_id` humain. Cette preuve
suffit à afficher `Remis au fournisseur`, jamais une activité. La projection
existante reste la seule source de l’état actif : un `turn_start` ou un
`prompt_dispatched` seul ne crée toujours aucun acte visuel de travail.

La mise en page réutilise les avatars existants, avec une taille réduite et un
texte atténué. Son conteneur a la même largeur que le timeline, de sorte que les
points, la bouille de remise et les activités réelles partagent le rail gauche
des bulles agent.


## Livraison du relais UI - 30 août 2026

- `main` contient `170785a fix(ui): Distinguer remise et activité`.
- Le binaire release de `bridget-daemon` a été reconstruit depuis `main`, puis
  installé dans `/home/moi/.local/bin/bridget`.
- `bridget-ui.service` a été redémarré et est actif ; les assets servis sur le
  port local 17888 contiennent `deliveryVisualState` et
  `delivery-activity__receipt`.
- Le daemon métier et les agents actifs n’ont pas été redémarrés.


## Correctif de séquence texte et outils - 30 août 2026

| Commande | Résultat |
|---|---|
| `node --test crates/bridget-daemon/assets/ui/app.js` | 77 tests réussis, dont le témoin texte -> actions -> texte |
| `git diff --check` | réussi |

Le projecteur UI forme des segments contigus de texte et des lots contigus de travail. Tant que ce lot est terminal dans un tour actif, il demeure dans la
zone vivante. Lorsque le fournisseur émet un fragment texte ultérieur, le lot devient un
élément chronologique repliable du fil, entre les deux bulles. Il ne reste donc
pas en double sous le compositeur.

Le résumé terminal ne répète plus les actes déjà projetés dans le fil. Les
détails techniques du lot sont placés sous leur libellé et conservent leurs
retours à la ligne réels.


## Livraison de la séquence texte et outils - 30 août 2026

- `main` contient `3364d90 fix(ui): Ordonner texte et actions`.
- Le binaire release a été reconstruit depuis `main`, installé dans
  `/home/moi/.local/bin/bridget`, puis `bridget-ui.service` a été redémarré.
- Le service est actif et les assets locaux exposent `renderActivityBatch` et
  `timeline-action-batch`. Le daemon métier et les agents actifs restent
  inchangés.
