# Journal 260 - mise en oeuvre

Statut du programme : In Progress. Règles : `contracts/regles-implementeur.md`. Tâches : `tasks.md`.

## Lots

| Lot | État | Remarque |
|-----|------|----------|
| 0 | partiel | T001, T002, T006 faits ; T003 à T005 en attente (Cargo, voir T002) |

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
