# Audit de réutilisation de l'existant - Programme 260 (assainissement)

Date : 10 novembre 2026. Statut : `PASS`. Méthode : le projet n'a pas de primitive `speckit-audit-existing` (ni modèles, ni scripts). Protocole manuel : extraire les éléments neufs du plan, chercher leurs équivalents avec `rg`, lire les consignes projet.

## Decision

`PASS`. Aucune duplication évidente. Deux abstractions neuves (`effective_uid()`, `RecoverPoison`) n'ont pas d'équivalent existant. Tous les autres éléments réutilisent un mécanisme présent.

## Synthese

| Élément du plan | Existant trouvé | Décision |
|-----------------|-----------------|----------|
| Cibles `make fmt`, `fmt-check`, `clippy`, `lint` | `Makefile` existe, sans ces cibles (`rg '^(lint|fmt|clippy)' Makefile` : rien) | Ajouter au `Makefile` existant. Pas de nouveau script. |
| Appel Cargo | `scripts/build.py` (liste blanche `build test check clippy`) | Réutiliser. Aucune modification de la liste. |
| Lints du workspace | `rg '\[workspace.lints' Cargo.toml` : rien | Créer la section dans le `Cargo.toml` racine. |
| `.github/workflows/rust.yml`, `rust-audit.yml` | Seul `repository-hygiene.yml` existe | Nouveaux fichiers. Même modèle de sécurité, même empreinte `actions/checkout`. |
| Contrôle de secrets et de documents | `scripts/check-secrets.sh`, `scripts/check-docs.py` | Réutilisés tels quels en livraison. |
| `effective_uid()` | `rg 'fn effective_uid\|fn current_uid'` : rien. 76 lectures `geteuid` réparties dans 31 fichiers | Créer une fonction dans `bridget-transport/src/fsutil.rs` (module existant). 76 usages : seuil de 3 dépassé. |
| `RecoverPoison` | `rg 'RecoverPoison\|POISON_RECOVERIES'` : rien. Plus de 600 sites `unwrap_or_else(\|x\| x.into_inner())` | Créer un trait et un compteur. Plus de 600 usages. |
| Compteur de connexions | `Metrics` avec `active_connections`, `increment_connections`, `decrement_connections` existe, jamais appelé | Câbler l'existant. Aucun nouveau type. |
| Module `daemon/wrapper_dispatch.rs` | Motif `#[path]` + `mod x;` déjà utilisé (`native_workspace154_tests.rs`) | Même motif. Un seul nouveau module, pour les 37 branches. |
| Externalisation des tests | `rg -ln 'extraire\|externalis' scripts` : rien | Un outil neuf `outils/extraire_module_tests.py`, dans le dossier de la spec, pas dans `scripts/`. Usage sur 16 fichiers : justifié. |
| `.git-blame-ignore-revs` | Absent | Créer au lot 10 (un fichier, une ligne). |
| ADR | `docs/decisions/` existe (modèle `251-*.md`) | Suivre le modèle. |
| Lot 15 (signature) | Aucune signature de release existante | Proposition seulement (D8). Rien à créer. |
| Écrans Android | `MainActivity.kt` (1154 lignes) | Découper. Fichiers neufs dans le même paquet. |

## Reutilisations correctement identifiees

- `scripts/build.py` pour tout Cargo (consigne AGENTS.md « Builds »).
- `repository-hygiene.yml` comme modèle de sécurité du workflow.
- Motif `#[path]` des tests déjà externalisés.
- `Metrics` existant pour les connexions.
- `fsutil.rs` existant pour la fonction d'identité.

## Existant potentiellement pertinent non mentionne

| Existant | Constat | Traitement |
|----------|---------|------------|
| 257 dossiers dans `specs/` | Reçus de session déjà rangés par dossier | Le lot 3 déplace les reçus de `AGENTS.md` vers `specs/recus-de-session.md` (un fichier, pas un par session). |
| 40 scripts dans `scripts/` | Seuls 4 sont vivants (R6.8) | Le lot 13 les vérifie sans les exécuter. |
| `scripts/tests/` | Tests Python de l'installateur | Réutilisés en livraison. Pas de nouveaux tests de scripts. |

## Duplications evidentes

Aucune.

## Memoires et regles applicables

- `AGENTS.md` du projet : Builds (Cargo par `scripts/build.py`), Livraison (activation), Fin de livraison (Git, nettoyage), Communications attentives (pas de bulletin général).
- Mémoire projet : tests Bridget avec `umask 077` et environnement privé (d'où `ENVT` dans les règles) ; dépôt public (pas d'adresse, pas de jeton).
- Constitution : XV (écart justifié dans `plan.md`), XIX (deux abstractions seulement), XX (pas de régénération), XXII (mesure avant optimisation).
- Interdits du programme : R6 (dix interdits).

## Specs livrees applicables

| Spec | Lien |
|------|------|
| 258 `conversation-complete` | Modèles de forme des artefacts (`reuse-audit.md`, `analyze.md`, `tasks.md`). |
| 251 `communications-attentives` | Modèle d'ADR. |
| 259 `guidage-preflight` | Dernier état livré : le programme 260 ne change pas ses comportements. |

## Journal de recherche

| Recherche | Résultat |
|-----------|----------|
| `rg 'fn effective_uid\|fn current_uid' crates` | aucune fonction |
| `rg 'RecoverPoison\|POISON_RECOVERIES\|undocumented_unsafe' crates Cargo.toml` | aucun résultat |
| `rg '\[workspace.lints' Cargo.toml` | aucun résultat |
| `rg '^(lint\|fmt\|clippy)' Makefile` | aucun résultat |
| `ls .git-blame-ignore-revs` | absent |
| `ls docs/decisions` | 251-*.md et README.md ; aucun fichier 260 |
| `rg -c geteuid crates --glob '*.rs'` | 31 fichiers concernés |
| Lecture de `.github/workflows/repository-hygiene.yml`, `rust-toolchain.toml` | modèle de sécurité et version 1.92.0 repris |

## Arbitrages

Aucun arbitrage utilisateur requis.

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
