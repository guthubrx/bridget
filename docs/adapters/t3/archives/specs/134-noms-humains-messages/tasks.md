# Archives techniques de l’adaptateur T3

Document d’origine : [specs/134-noms-humains-messages/tasks.md](../../../../../../specs/134-noms-humains-messages/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- [x] T003 [US2] Ajouter le test rouge `spec134` puis remplacer l’accès direct à l’expéditeur par `sender_label()` dans `crates/bridget-daemon/src/t3code.rs`

<a id="extrait-2"></a>

## Extrait 2

- [x] T005 Exécuter les tests ciblés, `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo build --locked --release -p bridget-daemon`, puis consigner les preuves dans `specs/134-noms-humains-messages/implementation.md`
- [x] T006 Faire la self-review Article XIX/XX, la contre-revue adverse post-implémentation, la convergence des artefacts et l’audit final ; résultat observable dans `specs/134-noms-humains-messages/implementation.md` et `specs/134-noms-humains-messages/validation/results.json`
- [x] T007 Fusionner dans `main`, construire depuis la racine principale, sauvegarder le binaire actif, déployer, relancer uniquement `com.bridget.daemon` et `com.bridget.t3`, vérifier le build-id et un message réel, pousser vers `https://github.com/guthubrx/bridget.git`, puis nettoyer le worktree et la branche

<a id="extrait-3"></a>

## Extrait 3

- [x] T008 Reproduire les 1 000 noms provisoires occupés sur les schémas actuel et historique ; vérifier l'échec avant correction.
- [x] T009 Supprimer le plafond arbitraire dans l'allocation existante, borner la recherche par le nombre de profils et rendre les échecs d'enregistrement visibles ; vérifier réparation, renommage T3 et absence de régression.
- [x] T010 Livrer le correctif autorisé de SPEC-134 avec sauvegarde, vérifier les noms des fils connectés et un message réel, puis consigner les limites.

<a id="extrait-4"></a>

## Extrait 4

Le premier incrément utile est T002 : les transports qui utilisent déjà
`sender_label()` deviennent lisibles. T003 ferme l’écart des lots. T004 répare la
cause qui empêche certains titres T3 d’atteindre le message. Aucun incrément
n’ajoute une dépendance, une table ou un service.
