# Bilan — Session 105

17 septembre 2026. Correction implémentée et vérifiée en environnement isolé.
Statut : déployée le 17 septembre 2026 après autorisation explicite. Validation ciblée et recette binaire réussies ; suite exhaustive non exécutée.

## Cause et correction

Le pont T3 créait une attente et promettait un relais pour chaque message, même
`reply=false`. Chaque réponse pouvait donc devenir la demande suivante.
Le dispatch respecte désormais le contrat. L'état durable porte la preuve
`reply_requested` ; les anciens états attendent une demande attestée par le daemon.
Les anciennes attentes non attestées ne déclenchent plus de lecture HTTP ou de
pagination pour corrélation. Elles restent conservées ; une absence dans la liste
bornée du daemon n'est pas une preuve autorisant leur suppression.

Hors T3, les relais gérés étaient déjà protégés. Le prompt interactif disait
néanmoins « Réponds TOUJOURS », Codex affichait sa consigne de réponse même sans
demande, Claude stream JSON ne projetait que le corps. Ces consignes sont alignées ;
Claude réutilise le contrat géré ACP. L'enveloppe terminal commune couvre PTY et tmux.

Pas de filtre lexical, de nouveau verrou, de dépendance ou de mécanisme métier.
Une nouvelle question explicite reste possible. Une réponse à une demande reste
transmise même si elle se limite à « OK ». Les envois volontaires par outil ne
sont pas bloqués ; leur pertinence reste une consigne aux agents.

## Preuves

- Reproduction initiale T3 : 0/3 tests passent avant correction (enveloppe,
  attente sans demande, réponse ancienne prête).
- Reproduction projections : les 3 nouveaux tests ACP/Claude/Codex échouent avant
  correction ; le test d'enveloppe terminal échoue également.
- Les premiers tests socket ont été ajustés après observation réelle de
  `{"type":"JournalReady"}` : compter les seules trames `Send`, pas les trames de service.
- 11 tests nommés spec105 réussis ; ils sont compris dans les groupes ci-dessous.
- Cœur : **39/39**.
- T3 : **40/40**.
- Prompts et reprise wrapper : **8/8**.
- Relais gérés et événements délégués : **4/4**, dont une matrice
  ACP/Claude/Codex × mode automatique/interactif × demande/non-demande × trois
  réponses, soit **36 cas** dans un test.
- Nouvelles projections transport : **3/3**, dont lecture de la véritable
  trame Claude sur stdin/stdout d'un processus `/bin/cat`.
- Consignes privées, modes de réponse et conservation HTML : **5/5**.
- Conservation octet pour octet du corps ACP : **1/1**.
- Total distinct : **100 tests réussis, aucun échec final**.
- `cargo clippy -p bridget-core -p bridget-transport -p bridget-daemon --lib --release -- -D warnings` : succès.
- `cargo fmt --all -- --check` et `git diff --check` : succès.
- Validation de la skill par quick_validate : succès.
- Relecture indépendante en lecture seule des six fichiers et de la skill :
  aucun défaut bloquant. Le signalement d'une pagination inutile des anciennes
  attentes a été traité et testé.

## Reproduction des vérifications

Répertoire : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/105-arret-boucles

Utiliser /Users/moi/.cargo/bin/cargo avec
`CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target` et
`CARGO_INCREMENTAL=0`. Les tests daemon sont exécutés avec `--test-threads=1`.

Commandes cargo, profil release, tests de bibliothèque uniquement :

```sh
cargo test -p bridget-core --lib --release -- --test-threads=1
cargo test -p bridget-daemon --lib --release t3code::tests -- --test-threads=1
cargo test -p bridget-daemon --lib --release wrapper::prompt_tests -- --test-threads=1
cargo test -p bridget-daemon --lib --release wrapper::delegated_runtime_tests -- --test-threads=1
cargo test -p bridget-transport --lib --release spec105 -- --test-threads=1
cargo test -p bridget-transport --lib --release consigne -- --test-threads=1
cargo test -p bridget-transport --lib --release acp::tests::prompt_preserves_the_body_byte_for_byte -- --exact
```

Après compilation des trois bibliothèques de test avec Cargo, les groupes élargis
ont été exécutés directement via leurs binaires de test release, sans recompilation.

## Limites et intégration

Les limites ci-dessous décrivent la première phase avant l'accord d'installation.
Le déploiement réalisé ensuite est consigné en fin de document.

- La suite workspace exhaustive n'a pas été lancée : certains bancs existants
  arrêtent des sous-processus par SIGKILL ou par groupe de processus, interdits
  par les instructions de cette session. Aucun de ces chemins n'a été invoqué.
- Aucun vrai fournisseur n'a été lancé ; aucune conversation réelle sollicitée.
- Aucun daemon/pont/agent relancé, aucun commit ni installation globale de skill.
- Les services déjà vivants et leur ancienne version restent inchangés.
- Le worktree contient les changements non commités de la session 101 en socle.
  Ne pas attribuer ces changements à la 105 ni déployer un build main sans eux.
  La comparaison finale du diff source101 avec le diff initial confirme qu'il
  n'a pas été modifié. L'arbre principal n'a pas été modifié par cette session.
- Les six fichiers de code propres à la correction : daemon t3code et wrapper,
  core envelope, transports ACP/Codex app-server/Claude stream JSON.
- Minimalisme : réutilisation du prompt ACP, aucun nouveau service ou paquet.
  L'état supplémentaire est nécessaire pour distinguer après redémarrage une
  demande attestée d'une attente historique créée par le bug ; l'essentiel des
  ajouts est constitué de tests. Aucune abstraction sans usage réel.

## Adoption autorisée — 17 septembre 2026, 06:43 CEST

Accord utilisateur : « install et relance ». Compilation séparée par clone APFS
du cache, sans remplacer le binaire actif pendant la validation. Recette réelle
isolée supplémentaire : `spec101_real_daemon_journal_share_collision_and_restart`,
1/1 réussie en 6,43 s sur le binaire final (soit 101 tests distincts validés).

Installation par copie adjacente, comparaison puis renommage atomique.
SHA256 installé : `c185fe267e610d3ba60a4e6fd4567cb965134549943a5746d8d706c9ab9eba5f`.
Seuls le daemon et le pont ont reçu un SIGTERM individuel après contrôle du PID ;
launchd les a relancés : 18471 → 84375 et 19624 → 85300.
Daemon en ligne, pont running, 12 fils T3 joignables, aucune nouvelle conversation
sollicitée pour le contrôle. T3 application et serveur gardent les mêmes PID et
dates de naissance. Trois processus Codex et Cursor attestés inchangés ; un autre
Codex et son MCP ont disparu pendant la fenêtre, cause non attestée, aucun signal
fournisseur envoyé. Ne pas annoncer que tous les PID fournisseurs sont inchangés.

Les quatre attentes historiques sans contrat restent conservées sans relais.
Le conflit de nom wiki préexistait et n'a pas été modifié.
La skill et sa référence sont installées dans la source principale déjà pointée
par Codex/Claude/Agents ; leurs autres modifications préexistantes sont préservées.
Les wrappers/MCP existants ne sont pas relancés : hors T3, un wrapper déjà vivant
prendra les nouvelles consignes embarquées lors de son prochain lancement.

Sauvegarde privée (ancien binaire, base avec quick_check=ok, état pont, plists,
skill, sources compilées et procédure de retour arrière) :
/Users/moi/.cache/bridget-adoptions/105-20260917.bWasBb/receipt.md

Aucun commit ni merge. Le disque ne dispose plus que d'environ 0,8 Gio libres ;
aucun nettoyage de fichiers utilisateur effectué.
