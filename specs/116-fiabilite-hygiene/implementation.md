# Journal 116 — Réponses fiables, identités stables, entretien automatique

- **Base** : main `9cbd770e` — **Date** : 2026-09-24 — **Statut** : Implemented, livré le 2026-09-24 07:57

## A. Appariement des réponses (`adaptateur de bureau.rs`)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/116-fiabilite-hygiene/implementation.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/116-fiabilite-hygiene/implementation.md#extrait-2).

## B. Inventaire des fichiers ouverts (`runtime.rs`)
Vérifié : `lsof -p vivant,disparu` rend l'inventaire complet du vivant et sort en code 1.
`only_vanished` n'excuse l'échec que si au moins un processus manque et que tous les manquants sont
morts (`kill(pid, 0)` → ESRCH) ; toute autre cause reste un échec.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/116-fiabilite-hygiene/implementation.md#extrait-3).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/116-fiabilite-hygiene/implementation.md#extrait-4).
