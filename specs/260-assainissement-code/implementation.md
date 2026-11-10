# Journal 260 - mise en oeuvre

Statut du programme : In Progress. Règles : `contracts/regles-implementeur.md`. Tâches : `tasks.md`.

## Lots

| Lot | État | Remarque |
|-----|------|----------|
| 0 | partiel | T001, T002, T006 faits ; T003 à T005 en attente (Cargo, voir T002) |
| 2 | intégré | T025 à T030 faits |
| 3 | intégré | T031 à T035 faits |
| 12 | intégré | T125 à T127 faits |
| 13 | intégré | T128 à T136 faits ; T137 partiel (job CI différé) |
| 15 | intégré | T141, T142 faits |

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

### Lot 3 - Consignes allégées (T031 à T035)
- Statut : fait
- Commits : copie du bloc (`289c9327`), remontée des consignes (`7cc2326b`), retrait du bloc (`d87deac6`) sur la branche du lot.
- Fichiers : `AGENTS.md` (126 112 -> 10 144 octets), `specs/recus-de-session.md` (nouveau, 117 229 octets), `validation/lot03-constat.txt`.
- Vérifications : bornes du bloc D=112 et F=1087 relevées ; bloc copié octet pour octet (`diff` vide, 975 lignes) ; retrait par motif : 975 lignes retirées, 0 ajoutée par le motif ; taille 10 144 octets (moins de 20 480, SC04). `check-docs.py` : PASS. `gitleaks` ciblé : aucune fuite.
- Écarts : les huit consignes vivantes ont été écrites avec leur source. Le repère du registre Mac est `2b4f409e...f9d32c9` (lu dans le reçu 256, copié depuis le fichier, pas de mémoire). Pour le Python 3.9.6, le reçu 224 est cité.
- Non vérifié : si d'autres consignes vivantes existent dans les reçus au-delà des huit demandées par le guide (le guide n'en demandait pas davantage).

### Bilan lot 3
Pas d'activation : documentation seulement.

### Lot 12 - Stockage documenté (T125 à T127)
- Statut : fait
- Commits : `6348d671` (inventaire), `bd37ae63` (ADR proposé) sur la branche du lot.
- Fichiers : `specs/260-assainissement-code/validation/lot12-inventaire-stockage.md`, `docs/decisions/260-stockage-versions-schema.md` (statut Proposé).
- Vérifications : relevé par `rg` hors fichiers de tests ; `python3 scripts/check-docs.py` PASS ; `git diff origin/main --stat -- crates` vide (aucun code touché).
- Écarts : l'inventaire contient des cellules « non lu » (comportement face à une base plus récente pour trois mécanismes ; modes de journal de six magasins). Elles ne sont pas inventées.
- Non vérifié : ce qui est marqué « non lu » dans l'inventaire.

### Bilan lot 12
Pas d'activation : documentation seulement. Décision à prendre par François : ADR `260-stockage-versions-schema.md`.

### Lot 15 - Proposition de signature (T141 et T142)
- Statut : fait
- Commit : `5a92f1b4` sur la branche du lot.
- Fichiers : `specs/260-assainissement-code/validation/lot15-proposition-signature.md`.
- Vérifications : `check-docs.py` PASS ; `git diff origin/main --stat -- . ':!specs'` vide (rien hors `specs/`) ; D8 reste `en attente` (non modifiée).
- Écarts : la procédure de publication n'est pas lisible dans le dépôt (le script `publier.py` cité par la mémoire de la séance est absent). Écrit « non lue » dans la proposition.
- Non vérifié : la faisabilité de l'option C sur le flux de publication actuel.

### Bilan lot 15
Pas d'activation : documentation seulement. Décision D8 demandée à François.

### Lot 13 - Scripts contrôlés (T128 à T136 faits, T137 partiel)
- Statut : T128 à T136 faits ; T137 partiel (cible locale faite, job CI différé)
- Commits : sur la branche du lot : `ruff.toml`, imports inutiles et chaînage des exceptions, avertissements de boucle, exceptions de sécurité justifiées et documentation de l'`exec`, commentaires `set -e`, cible `lint-scripts`.
- Fichiers : `ruff.toml` (nouveau), `Makefile`, `scripts/*.py` (prepare-project, publish-android, real_hook_profile171, sentinel, update, tests/...), 4 scripts shell (commentaires), `validation/lot13-*.txt`.
- Vérifications : `ruff check scripts` : 137 signalements au départ (`lot13-ruff-avant.txt`) -> 0 ; `make lint-scripts` code 0 ; `bash -n scripts/*.sh` code 0 ; `py_compile` des fichiers modifiés OK (dossiers `__pycache__` supprimés) ; `scripts/tests` : `test_install` 44 tests OK, `test_sentinel` 14 OK, `test_publish_android` 7 OK, `test_prepare_project_view` 3 OK ; `test_updates` : 16 échecs sur 26, **identiques sur `origin/main`** (chemin `/private/var` contre `/var` sous macOS), donc hérités.
- Écarts :
  - Le guide attendait 6 scripts sans `set -e`. Lecture : `install-k1.sh` et `registre-nature-reclasser.sh` ont déjà `set -euo pipefail` ; `federate-ssh.sh` l'active dans `federation_main()` (le fichier est sourcé). Restent 4 scripts, tous documentés par un commentaire. **T135 adapté** : les deux scripts de test ne reçoivent pas `set -eu`, car ils comptent leurs échecs (variables `passed`/`failed`) ; un `set -e` changerait leur comportement. Ajout du seul commentaire.
  - B905 : avec `target-version = "py39"` ruff ne le signale plus (rien à faire).
  - Le fichier `scripts/sentinel.py` est embarqué dans le démon par `include_str!` : le commentaire d'invariant ne change pas le comportement mais change les octets du binaire à la prochaine compilation.
  - D11 `en attente` : action par défaut appliquée (mécanisme `exec` conservé, commentaire d'invariant et `# noqa: S102`).
- Non vérifié : le job CI `scripts` de T137 (`rust.yml` n'existe pas avant le lot 1) ; aucun script vivant n'a été exécuté.

### Bilan lot 13
Activation en attente : `scripts/sentinel.py` est embarqué dans le binaire du démon (octets modifiés, comportement identique). Les autres scripts sont distribués tels quels.
