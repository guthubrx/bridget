# Journal 116 — Réponses fiables, identités stables, entretien automatique

- **Base** : main `9cbd770e` — **Date** : 2026-09-24 — **Statut** : Implemented, livré le 2026-09-24 07:57

## A. Appariement des réponses (`t3code.rs`)

### Mesures
- `horizon-terra-captation` (Codex) : 38 messages, 38 tours selon T3, 34 tours écrits. Les messages de
  08:06:32 et 08:25:53 le 20/09 ont chacun eu un tour muet (`01a0bdda`, `01a0bdeb`) ; deux demandes
  voisines ont été abandonnées.
- `opus_city_ai` (Claude) : 58 messages, 139 tours écrits, 320 points de contrôle — réveils
  d'arrière-plan à la fin des sous-agents, et messages tapés pendant un tour.
- Points de contrôle T3 écartés comme décompte : absents sur 5 fils sur 20 (hors Git), non un pour
  un sur 9. Hypothèse abandonnée avant d'écrire du code.
- `latestTurn.requestedAt` égale le `createdAt` d'un message unique sur 3 fils Codex et 1 fil Claude ;
  aucun message pour un réveil d'arrière-plan. C'est la preuve retenue.

### Correction
- `record_turn_origin` relève l'origine du dernier tour à chaque lecture (avant, seulement si le
  journal d'observation était à jour) ; `prune_turn_origins` borne la table sans élaguer une origine
  qu'une demande attend encore.
- `correlate` consulte d'abord l'origine prouvée ; `proven_turn_answer` rend le texte de ce seul
  tour, `Waiting` s'il est ouvert ou en écriture, `Silent` s'il est clos sans texte. Le rang reste en
  repli, inchangé : les tests historiques passent sans modification.
- `Correlation::Silent` n'est engagé qu'après SETTLE_ATTEMPTS lectures (relecture critique : T3
  projette de façon asynchrone ; un texte arrivé tard l'emporte). Annonce attribuée à Bridget.

## B. Inventaire des fichiers ouverts (`runtime.rs`)
Vérifié : `lsof -p vivant,disparu` rend l'inventaire complet du vivant et sort en code 1.
`only_vanished` n'excuse l'échec que si au moins un processus manque et que tous les manquants sont
morts (`kill(pid, 0)` → ESRCH) ; toute autre cause reste un échec.

## C. Entretien automatique
- `settle_expired_dispatching` : remises expirées encore `dispatching` → `indeterminate`.
- `purge_expired_sends` : envois seuls, 30 jours après expiration ; `purge_expired`, jamais appelée
  en production, aurait emporté `spawn_commands` en cascade.
- `purge_stale_identity_files` : marqueurs de processus morts ou recyclés ; noms et preuves de plus de
  7 jours ni référencés ni connectés. Réutilise `read_marker_file`. Une preuve d'instance connectée
  est gardée même sans marqueur (fil T3 au repos) : la supprimer aurait recréé la panne de la 115.
- `rotate_log_if_large` / `service_logs` : copie puis troncature au-delà de 20 Mio, une génération
  `.1` ; journaux ouverts en mode ajout par launchd (vérifié `lsof +fg` : AP).
- `run_maintenance`, fil `entretien` : 2 min après le démarrage puis toutes les heures ; verrou tenu
  pour les deux requêtes SQL seulement ; sortie sur arrêt par `sleep_until_shutdown`.
- `prune_merged_worktrees` (`scripts/build.py`) : fusionné dans main, propre (non suivis compris),
  dernier commit de plus de 24 h, aucun processus dont le répertoire courant s'y trouve (un `lsof`,
  échec = rien retiré) ; branche conservée. Racine prise sur le dépôt principal. Simulation sur le vrai
  dépôt : exactement les 21 attendus, les 3 sales, 2 non fusionnés et 2 récents épargnés.

## D. Diagnostic (`cli.rs`)
`unattested_hint` explique un refus d'identité (code et remède de la résolution) pour `send` et
`reply`, sans maquiller un autre refus.

## E. Identifiant de build figé (`build.rs`), trouvé à la livraison
Le daemon relancé annonçait `853f29034c2d`, commit de la session 105 du 17/09, alors que le binaire
venait d'être reconstruit au commit `81623fab`. La sortie Cargo du script de compilation listait des
chemins du worktree 105 : `build.rs` lisait `CARGO_MANIFEST_DIR` par `env!`, donc à SA compilation.
Compilé une fois depuis ce worktree dans le répertoire de compilation principal, puis réutilisé (Cargo
n'inclut pas le chemin d'un membre de l'espace de travail dans son empreinte), il interrogeait depuis
le Git figé de ce worktree. L'alerte de daemon périmé comparait donc deux valeurs toujours égales.
Correction : lecture à l'exécution (`std::env::var_os`). Vérifié : le binaire porte le commit réel.
Limite connue : construire un worktree dans le répertoire de compilation principal fait surveiller
les fichiers Git de ce worktree jusqu'à la construction suivante depuis le dépôt principal.

## F. Fixtures de test abandonnées (`tests/support/idempotent.rs`)
`test_root` créait `/tmp/bid-<id>` sans jamais l'effacer : 156 répertoires (1,3 Go) après une
matinée de recettes, cause principale du disque plein. Chaque racine est désormais inscrite et
effacée à la sortie du binaire de test (`atexit`), quand ses tests ont rendu leurs processus. Aucun
test ne relance son propre binaire en utilisant ces racines : pas de suppression prématurée.

## Écarté
Profils d'agents (313) : seul registre du nom d'un identifiant passé ; ne bloquent plus de nom (110).

## Incident pendant la recette
La deuxième recette complète a échoué sur « No space left on device » : disque tombé à 1,1 Gio.
Cause principale : mes propres répertoires de compilation (`/tmp/b115` 4,4 Go, `/tmp/b116-bis`) et
156 fixtures `/tmp/bid-*` (1,3 Go) laissées par les tests. Retirés : 10 Gio libres. Effet en
production : une seule écriture ratée, le statut informatif du pont à 07:42.

## Vérifications
- 13 tests Rust `spec116_*` et 4 tests Python, verts. Tests historiques d'appariement inchangés.
- fmt OK ; clippy `-D warnings` OK.
- Première recette complète (avant le correctif du silence différé) : 1545 réussis, 0 échec.
- Recette finale, sur l'état final : **1543 réussis, 3 échecs, 52 ignorés** sur 80 binaires. Les
  trois échecs sont dans le seul `search_104_test` (s22, s24, s25-s26) : « réponse daemon :
  WouldBlock (os error 35) », délai de lecture de socket dépassé, avec une charge moyenne de la
  machine à 55 due à d'autres processus. Relancé seul trois fois : **23 réussis, 0 échec** à chaque
  fois. La recherche n'est pas touchée par cette session. Tests Python : 26 réussis.

- Recette complète après le correctif des fixtures : **1545 réussis, 1 échec, 52 ignorés** ;
  **0 fixture restante** dans /tmp, contre une cinquantaine par recette avant. L'échec,
  `claude_gere_avec_bypass_ecrit_et_relit_un_fichier`, est préexistant et lié à la charge : il échoue
  aussi sur main sans ce correctif (1 sur 6 à une charge moyenne de 67), jamais à charge 15 (10 sur 10
  en parallèle, 8 sur 8 seul). Le pair reçoit la fin de connexion avant la réponse, en 1,3 s : piste
  ouverte d'un wrapper Claude géré qui se termine avant de relayer sa réponse sous charge extrême.

## Livraison
- 07:52 : fusion, construction ; le script retire de lui-même les 21 worktrees fusionnés.
- 07:52 puis 07:57 : relance du daemon et du pont (la seconde pour l'identifiant de build corrigé).
  `bridget status` annonce `259b85793d50`, commit réel ; 26 agents, 22 fils T3.
- 07:54:53 : premier passage d'entretien — 33 remises en vol passées en sort inconnu ; 55 marqueurs,
  49 noms, 2 preuves retirés. `spawn_commands` intact (449).
- Identités vivantes vérifiées après ce passage : les 10 marqueurs restants désignent tous un agent
  vivant ; `requests` et la sonde d'envoi aboutissent depuis le fil `bdget`.
- 52 fils ont déjà des origines de tour relevées par le nouvel appariement.
