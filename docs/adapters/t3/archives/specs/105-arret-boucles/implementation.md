# Archives techniques de l’adaptateur T3

Document d’origine : [specs/105-arret-boucles/implementation.md](../../../../../../specs/105-arret-boucles/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le pont T3 créait une attente et promettait un relais pour chaque message, même
`reply=false`. Chaque réponse pouvait donc devenir la demande suivante.
Le dispatch respecte désormais le contrat. L'état durable porte la preuve
`reply_requested` ; les anciens états attendent une demande attestée par le daemon.
Les anciennes attentes non attestées ne déclenchent plus de lecture HTTP ou de
pagination pour corrélation. Elles restent conservées ; une absence dans la liste
bornée du daemon n'est pas une preuve autorisant leur suppression.

<a id="extrait-2"></a>

## Extrait 2

dans un terminal, les relais gérés étaient déjà protégés. Le prompt interactif disait
néanmoins « Réponds TOUJOURS », Codex affichait sa consigne de réponse même sans
demande, Claude stream JSON ne projetait que le corps. Ces consignes sont alignées ;
Claude réutilise le contrat géré ACP. L'enveloppe terminal commune couvre PTY et tmux.

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

```sh
cargo test -p bridget-core --lib --release -- --test-threads=1
cargo test -p bridget-daemon --lib --release t3code::tests -- --test-threads=1
cargo test -p bridget-daemon --lib --release wrapper::prompt_tests -- --test-threads=1
cargo test -p bridget-daemon --lib --release wrapper::delegated_runtime_tests -- --test-threads=1
cargo test -p bridget-transport --lib --release spec105 -- --test-threads=1
cargo test -p bridget-transport --lib --release consigne -- --test-threads=1
cargo test -p bridget-transport --lib --release acp::tests::prompt_preserves_the_body_byte_for_byte -- --exact
```

<a id="extrait-5"></a>

## Extrait 5

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

<a id="extrait-6"></a>

## Extrait 6

Installation par copie adjacente, comparaison puis renommage atomique.
SHA256 installé : `c185fe267e610d3ba60a4e6fd4567cb965134549943a5746d8d706c9ab9eba5f`.
Seuls le daemon et le pont ont reçu un SIGTERM individuel après contrôle du PID ;
launchd les a relancés : 18471 → 84375 et 19624 → 85300.
Daemon en ligne, pont running, 12 fils T3 joignables, aucune nouvelle conversation
sollicitée pour le contrôle. T3 application et serveur gardent les mêmes PID et
dates de naissance. Trois processus Codex et Cursor attestés inchangés ; un autre
Codex et son MCP ont disparu pendant la fenêtre, cause non attestée, aucun signal
fournisseur envoyé. Ne pas annoncer que tous les PID fournisseurs sont inchangés.
