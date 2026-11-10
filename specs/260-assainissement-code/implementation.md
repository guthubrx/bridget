# Journal 260 - mise en oeuvre

Statut du programme : In Progress. Règles : `contracts/regles-implementeur.md`. Tâches : `tasks.md`.

## Lots

| Lot | État | Remarque |
|-----|------|----------|
| 0 | partiel | T001, T002, T006 faits ; T003 à T005 en attente (Cargo, voir T002) |
| 2 | intégré | T025 à T030 faits |

## Journal

### T001 - Journal et règles
- Statut : fait
- Commit : voir l'historique du lot 0
- Fichiers : `specs/260-assainissement-code/implementation.md`
- Vérifications : règles et `decisions.md` lus depuis `origin/main`. Toutes les portes sont `en attente`.
- Écarts : aucun
- Non vérifié : rien

### T002 - Outils et disque
- Statut : fait (constat), avec réserve bloquante pour Cargo
- Fichiers : aucun
- Vérifications : `rg` 15.2.0, `python3` 3.12.12, `gh` 2.103.2, `ruff` 0.14.13, `git` 2.56.0, `gitleaks` 8.30.1, `scripts/build.py` présent. Absents : `shellcheck`, `actionlint`, `cargo-audit`.
- Écarts : `C-DISQUE` (`df -g /`) donne 14, 12 puis 11 à 12 Gio libres sur la durée de la séance, valeur qui bouge avec l'activité de la machine. Le seuil de la règle R4 est 12 Gio.
- Non vérifié : combien de Gio une première compilation complète consomme (le dossier `target` du dépôt principal pèse 4,8 Mio).

### T006 - Ligne de base du format
- Statut : fait
- Fichiers : `specs/260-assainissement-code/validation/base-format.txt`
- Vérifications : `cargo fmt --all -- --check` (ne compile pas) -> 120 fichiers signalés. Aucun fichier source modifié.
- Écarts : chemins ramenés en relatif.
- Non vérifié : rien

### T003, T004, T005 - Lignes de base des tests et de l'analyse
- Statut : différé (disque)
- Raison : ces tâches compilent tout le workspace. Le disque libre (11 à 14 Gio) est au seuil d'arrêt de R4 (12 Gio). Décision demandée à François : relever l'autorisation ou libérer de la place.

## Idées

### Lot 2 - Dépôt sans résidus (T025 à T030)
- Statut : fait (T025 à T030)
- Commits : `git log --oneline origin/main..` du lot : archivage des 15 résidus, `.gitignore` (ignorer les résidus), `.gitignore` (4 exceptions D3), inventaire.
- Fichiers : `specs/archives/residus-racine-20261110/` (15 fichiers déplacés par `git mv`), `.gitignore`, `validation/lot02-constat.txt`, `validation/lot02-gros-fichiers.txt`.
- Vérifications : constat de départ 345 fichiers suivis mais ignorés, 15 résidus de racine. Après : 0 résidu suivi à la racine, 15 dans l'archive ; `git ls-files -ci --exclude-standard` reste à 345 (D3 : on documente, on ne retire pas). Aucun code ou script ne lit les résidus (`rg` hors `specs/` : aucune ligne). `gitleaks` ciblé sur les commits du lot : aucune fuite. `python3 scripts/check-docs.py` : PASS (32 pages, 522 liens, 13 paires).
- Écarts : D3 et D5 `en attente` : actions par défaut appliquées (ne rien retirer, archiver sans supprimer). Le contrôle `scripts/check-secrets.sh` complet (historique entier) n'a pas été relancé : il a été arrêté par le système lors de la séance précédente ; remplacé par un scan ciblé.
- Gros fichiers (propositions, rien déplacé) : 11 fichiers de plus de 1 Mio. `daemon.rs` et `native_delegation.rs` (code vivant) : à alléger par les lots 4 et 5. Les deux fichiers `preuves.html`, trois `.patch` de `specs/` et `audits/`, `docs/adapters/t3/...` : à garder (preuves historiques) ou à sortir de Git sur décision de François.
- Non vérifié : rien d'exécutable n'est touché.

### Bilan lot 2
Pas d'activation : aucun fichier exécutable modifié.
