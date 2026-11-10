# Lot 14 - Tests ignorés expliqués (T138-T140)

But : chaque test marqué `#[ignore]` dit pourquoi. 66 tests sont ignorés, 8 sans motif (`releves.md` section 12). Un lint empêche le retour en arrière.
Branche : `session-260-lot14-ignores`. Worktree : `.worktrees/260-lot14-ignores`. Aucun comportement ne change : un motif est un texte. Aucune activation.
**Règle D14 : ne pas exécuter ces tests et ne pas les réactiver.** Ils sont ignorés parce qu'ils exigent un fournisseur réel, du temps ou un service. Seul le motif s'ajoute.

## T138 - Inventaire

```sh
rg -n '#\[ignore\]' crates --glob '*.rs' | tee "$V/lot14-sans-motif.txt"
wc -l "$V/lot14-sans-motif.txt"          # attendu : 8 lignes (peut varier de 1 ou 2 après les lots 4 et 5)
rg -c '#\[ignore' crates --glob '*.rs' | awk -F: '{s+=$2} END{print s}'    # total, environ 66
```

Lieux attendus : `workspace_commit_view_tests.rs` ; `daemon.rs` (ou ses fichiers de tests déplacés au lot 4) ; `managed_process.rs` (4). Si le lot 4 est intégré, les lignes de `daemon.rs` sont dans les nouveaux fichiers `daemon_*_tests.rs` : la commande `rg` les trouve toujours.
Les `#[ignore = "..."]` existants ne sont pas touchés. Commit : `docs(specs): Inventorier les tests ignores sans motif`.

## T139 - Écrire le motif

Pour chaque test sans motif :

1. Lire le test (30 lignes au plus) et ses commentaires voisins. Chercher la cause : `rg -n 'Command::new|TcpStream|sleep|claude|codex|HOME|real|reel' <fichier>` autour du test.
2. Écrire le motif : `#[ignore = "<cause courte en français>"]`. Exemples valides : `exige un fournisseur Claude réel`, `exige un processus enfant réel et du temps`, `dépend de l'horloge du système`.
3. **Cause non établie après lecture** : `#[ignore = "motif non établi (programme 260) : à qualifier"]`. Ne jamais inventer une cause.
4. Ne pas toucher au corps du test, à son nom, ni à ses attributs voisins.
Vérifier : `C-CHECK`. Contrôle : `git diff -U0 | rg '^[-+]' | rg -v 'ignore|^(\+\+\+|---)'` ne montre aucune ligne. Commit : `docs(tests): Ajouter le motif des tests ignores`.

## T140 - Lint permanent et porte du lot

1. `Cargo.toml` (racine), `[workspace.lints.clippy]` : ajouter `ignore_without_reason = "warn"`. Si le lot 1 n'est pas intégré, voir T009 de son guide pour créer la section.
2. `C-CLIPPY` : aucun avertissement `ignore_without_reason` (critère SC11 : 0 test ignoré sans motif).
3. Contre-épreuve : ajouter temporairement `#[test] #[ignore] fn essai() {}` dans un fichier de test, lancer `C-CLIPPY`, constater l'avertissement, retirer.
4. `C-NOMS` : aucune différence. Compter les tests ignorés : le total reste identique (66 ou la valeur de T138). `C-TESTS` et `C-ECHECS` ne sont pas requis (aucun test exécuté ne change), mais lancer `C-CRATE` pour le crate touché par le lint.
5. Bilan dans `implementation.md` : liste des 8 motifs écrits, nombre de motifs « non établi ». Fin de lot (`quickstart.md`).
Commit : `chore(lint): Exiger un motif sur chaque test ignore`.

## Arrêts propres à ce lot

- Un test ignoré semble pouvoir tourner : ne pas le lancer. Le noter dans « Idées ».
- Plus de 5 tests sans cause établie : écrire quand même le motif de repli, puis le dire dans le bilan.
