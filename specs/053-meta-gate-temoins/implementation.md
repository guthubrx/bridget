# Implémentation 053 — Méta-gate des témoins de présence

## Objet mesuré

- Plateforme : Linux x86_64.
- Base gelée : `52173dc7d886e30d980332734e99e815bd092309`.
- Branche : `session-053-meta-gate-temoins`, publiée vide avant écriture.
- Rust/Cargo : 1.92.0, imposé par `rust-toolchain.toml`.

## Mesure initiale

`cargo test -p bridget-daemon --lib --no-run` termine avant le comptage. Le
listing réel de la base contient 576 tests, dont 109 sous le préfixe exact
`daemon::presence_tests::`.

## Cycle rouge puis vert

Le banc a d'abord été écrit contre un gate volontairement permissif. Son
univers annoncé contenait trois scénarios. Résultat brut :

```text
la sélection amputée a été acceptée: daemon::presence_tests::TEMOIN_send_idempotent_reply_false_pose_deadline_codex_sans_agents_json
session-053 result: 1 passed / 2 failed / 0 ignored
```

Le contrôle complet passait déjà. Les deux rouges prouvaient séparément que le
stub acceptait une famille amputée et une sélection de cardinal zéro.

Le gate implémenté relance deux listings du même binaire. Il extrait uniquement
les lignes `: test`, refuse les deux sources muettes, puis compare les noms par
une table associative en O(N). Aucun manifeste de noms n'est recopié.

Après correction, sans changer le banc :

```text
univers session-053 (3 scénarios): sélection amputée, sélection complète, sélection vide
REFUS méta-gate 053: sélection incomplète: présents=108/109 manquants=1 univers_sélection=575
MANQUANT daemon::presence_tests::TEMOIN_send_idempotent_reply_false_pose_deadline_codex_sans_agents_json
session-053 selection_amputee_est_refusee ... ok
méta-gate 053: sélection complète: présents=109/109 univers_sélection=576
session-053 selection_complete_est_acceptee ... ok
REFUS méta-gate 053: sélection inobservable: 0 test listé; témoins attendus=109
session-053 selection_vide_est_inobservable ... ok
session-053 result: 3 passed / 0 failed / 0 ignored
```

## Mutant sélectif et restauration

Le mutant force uniquement `missing_count=0` après la comparaison. La
sélection amputée redevient faussement complète à 575 tests, tandis que le
contrôle complet et le refus de la sélection vide gardent leur issue. Résultat
brut : **2 passés / 1 échoué / 0 ignoré** ; le test mort est
`selection_amputee_est_refusee`.

Après restauration, le condensat du gate revient exactement à
`b9ec2a1f28f3ccc062ad39dad8e364c6a59f89de7846e586ffc7787caa7b9fca`,
puis le banc rend de nouveau **3/0/0**.

## Gates

- `bash -n` sur le gate et son banc : vert ;
- `git diff --check` : vert ;
- `cargo test -p bridget-daemon --lib --no-run` : vert avec quatre
  avertissements préexistants hors du hunk du lot ;
- `cargo clippy -p bridget-daemon --lib -- -D warnings` : rouge sur sept
  diagnostics de `bridget-transport`, aucun fichier concerné par le delta ;
- le même clippy avec `--no-deps` reste rouge sur dix diagnostics déjà présents
  dans le daemon, aucun sur le hunk ajouté ;
- `cargo fmt --all --check` : rouge sur la base et la tête, statut 1 et la même
  liste de 74 fichiers ; cette dérive globale n'est pas née du lot ;
- ShellCheck est indisponible sur la machine ; aucun outil n'a été installé.

## Minimalisme et limites

Le gate ne lance pas les 109 témoins : il vérifie que la sélection annoncée les
contient. Leur exécution et leur verdict restent un tir distinct, ce qui évite
de confondre complétude de l'univers et réussite des tests. La commande de banc
est exposée par `make test-review-witnesses` et le commentaire 050 pointe vers
elle.

Aucune dépendance ni structure de données persistante n'est ajoutée. Le
potentiel de suppression à comportement constant après relecture est estimé à
zéro ligne : les deux scripts séparent le gate livré du harnais qui prouve ses
issues opposées.

## Livraison

Le commit fonctionnel `290b0c671e4360b7f91d22bc25417283aec17528`
a été poussé. Sa valeur locale, la référence distante suivie et
`git ls-remote` étaient identiques. Au dernier contrôle, `main` avait déjà
avancé à `00833fcbcbde396298a09a9dcc8b27ecd0dbf540` ; aucune mesure du lot ne lui
est attribuée. L'objet compilé et éprouvé reste la branche issue de la base
gelée `52173dc7d886e30d980332734e99e815bd092309`.

## Non mesuré

- exécution des 109 tests de présence, volontairement distincte du listing ;
- macOS ;
- ShellCheck ;
- intégration à un service CI, absent du dépôt ;
- interception d'une commande Cargo exécutée en dehors du méta-gate.
