# Rapport Analyze: SPEC-065

**Date**: 2026-08-30
**Passage**: 7 après implémentation et validation
**Verdict**: SPEC-065 implémentée dans un worktree isolé depuis `main`
`04fd9c6`. La validation propre au lot est verte. La validation workspace
complète reste non verte sur des défauts de baseline hors SPEC-065, consignés
ci-dessous.

## Findings traités

| ID | Catégorie | Sévérité initiale | Correction appliquée |
|---|---|---:|---|
| A1 | Ambiguïté | HIGH | L'autorité de chaque champ est explicitée entre ProjectIdentity et ProjectBinding. |
| C1 | Couverture | HIGH | Les états partiels et le crash entre les deux stores ont été ajoutés à US1 et T016. |
| S1 | Sécurité | HIGH | Canonicalisation, préfixes autorisés, racines trop larges et symlinks sont couverts. |
| M1 | Minimalisme | MEDIUM | Docker, secrets, plugins et mémoire sont explicitement exclus. |
| I1 | Incohérence | MEDIUM | `domain` reste une métadonnée et ne remplace jamais `project_id`. |
| C2 | Couverture | MEDIUM | Non-destruction et empreinte avant/après sont couvertes par T019 et T023. |
| RC7-C1 | Concurrence | CRITICAL | pending_binding, issue de collision convergente et oracle alias/symlink concurrent ajoutés au modèle, contrat et T004-T017. |
| RC7-R2-L1 | Cycle de vie | LOW | Rebind conserve explicitement les agents actifs sur leur ancienne génération sans arrêt implicite. |
| RC8-C1 | Précondition | CRITICAL | Le worktree documentaire en retard de 18 commits est exclu comme base d'implémentation; nouveau worktree et nouvel audit obligatoires. |
| RC8-S1 | Sécurité | HIGH | `ProjectRootPolicy` possède maintenant une source, un propriétaire, des modes, des racines interdites et un comportement fail-closed. |
| RC8-I1 | Intégration | HIGH | ProjectReference est mappée vers toutes les surfaces durables SPEC-064, snapshots, projections et curseurs compris. |
| RC8-C2 | Contrat | HIGH | Une variante locale Maicie vers Bridget, négociée et authentifiée, remplace toute réutilisation ambiguë de `ServiceRequest`. |
| RC8-M1 | Migration | MEDIUM | `review_project` dispose d'un parcours dry-run/confirm idempotent et d'oracles interdisant toute migration implicite. |
| IR-F01 | Couverture d'audit | MEDIUM | T009, T014, T019, T021 et T023 produisent, rejouent et prouvent exactement un `ProjectAuditEvent` par mutation réussie. |
| IR-F02 | Traçabilité | LOW | Les preuves du reuse-audit référencent désormais les chemins et symboles stables, rejoués contre `main` à `d589b24`. |
| IR-F03 | Processus SpecKit | LOW | Décision utilisateur: l'absence de `.specify` est non bloquante et ne déclenche ni installation ni mise à jour de SpecKit. |
| IR-F04 | Intégration SPEC-068 | HIGH | Les liaisons, événements et incidents runtime délégués conservent le même `ProjectReference` dans le store, les trames, les rejeux et les acquittements. |
| I-065-1 | Migration | HIGH | La migration d'idempotence projet est v6, car v5 est déjà prise par les liens de livraison. Un test démarre sur une base v5 et prouve l'ajout sans collision. |
| I-065-2 | Chemins | HIGH | Un cwd de projet est accepté seulement s'il est dans la racine liée ou s'il partage le Git common dir d'un worktree lié. Un voisin est refusé avant le provider. |
| I-065-3 | Baseline | MEDIUM | `cargo clippy --workspace --all-targets -- -D warnings` échoue sur deux diagnostics préexistants de `bridget-transport`, hors fichiers changés par SPEC-065. |
| I-065-4 | Baseline | MEDIUM | `cargo test --workspace -- --test-threads=1` échoue dans trois scénarios MCP de `managed_parity_test`, hors surfaces SPEC-065. |

## Couverture

| Inventaire | Total | Couverts par tâches |
|---|---:|---:|
| Exigences fonctionnelles | 30 | 30 |
| Exigences non fonctionnelles | 7 | 7 |
| Critères nécessitant du travail | 11 | 11 |
| User stories | 3 | 3 |
| Tâches | 30 | 30 mappées |

Couverture: 100 %.

## Alignement constitutionnel

- Aucun conflit avec un MUST de la constitution.
- Migration additive, worktree isolé et rollback documentés.
- Le statut de SPEC-065 est documenté comme terminé, sans modifier la couche
  utilisateur SpecKit ni ses fichiers de synchronisation.
- Potentiel minimalisme: environ 0 ligne suppressible dans les artefacts sans
  perdre une exigence ou une preuve; le futur code devra refaire ce contrôle.

## Gates appliquées pendant l'implémentation

- Worktree créé depuis `main` `04fd9c6`, sans modification de `main`.
- Reuse-audit rejoué, sans nouveau registre concurrent ni nouvelle dépendance.
- Politique de racines uniquement de fixture, avec canonicalisation, symlinks,
  propriétaires et modes testés.
- Aucun déploiement, redémarrage de service ou écriture de production.

## Métriques Analyze

- Ambiguïtés restantes: 0
- Duplications restantes: 0
- Findings CRITICAL: 0
- Findings HIGH ouverts dans SPEC-065: 0
- Findings MEDIUM de baseline: 2
- Tâches non mappées: 0

La relecture indépendante a confirmé la cohérence globale et révélé un manque
de tâche productive pour l'audit. Ce manque est fermé: register, rebind et
disable sont audités transactionnellement, avec preuve de rejeu. La propagation
de `ProjectReference` est compatible avec les données historiques et couvre les
surfaces durables prévues. La seule réserve avant intégration est extérieure au
lot: assainir ou accepter explicitement les trois tests MCP de baseline et les
deux diagnostics Clippy, puis rejouer la validation sur la tête `main` de
l'intégration. L'absence de `.specify` est respectée et n'a déclenché aucune
mise à jour.
