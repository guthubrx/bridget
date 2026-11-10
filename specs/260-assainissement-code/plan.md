# Plan 260 - Assainissement du code et du dépôt

Date : 10 novembre 2026. Spécification : `spec.md`. Mesures : `releves.md`. Décisions réservées : `decisions.md`.

## Résumé

Le programme corrige ce que l'analyse du 10 novembre 2026 a constaté. Il ne change aucune fonction du produit.
Il est découpé en 17 lots indépendants et 157 tâches. Un agent moins capable exécute les tâches. Il suit `tasks.md`, puis le guide du lot.
Ce tour produit seulement les artefacts. Aucune tâche n'est exécutée.

## Contexte technique

- Langage : Rust 1.92.0 (`rust-toolchain.toml`), workspace Cargo de 3 crates, version 0.8.26. Kotlin pour Android. Python et shell pour les scripts.
- Stockage : SQLite. Aucun changement de schéma dans ce programme.
- Tests : `make test` ou `python3 scripts/build.py cargo test ...`. Jamais Cargo en direct.
- Plateformes : macOS ARM64 et Linux. La ligne de base des tests dépend de la machine.
- Contraintes : disque presque plein (arrêt sous 12 Gio) ; dépôt public ; un seul lot ouvert à la fois.
- Volume : 408 922 lignes de Rust, 3813 tests, 247 blocs `unsafe`, 385 avertissements d'analyse.

## Principe directeur

Chaque lot prouve qu'il ne change pas le comportement. La preuve est une comparaison de listes :

1. la liste triée des tests en échec est identique à la ligne de base ;
2. pour un déplacement de code, la liste triée des noms de tests est identique.

Trois lots ajoutent une observation (lot 7) ou une constante partagée (lot 8). Ils le disent dans leur guide et mesurent avant et après.

## Les 17 lots

| Lot | Titre | Tâches | Guide | Code exécutable modifié |
|-----|-------|--------|-------|-------------------------|
| 0 | Fondations | T001-T007 | `quickstart.md` | non |
| 1 | Filet automatique | T008-T024 | `guides/lot-01-filet-automatique.md` | oui, à partir de T011 |
| 2 | Dépôt sans résidus | T025-T030 | `guides/lot-02-depot-sans-residus.md` | non |
| 3 | Consignes allégées | T031-T035 | `guides/lot-03-consignes-allegees.md` | non |
| 4 | Tests hors des fichiers de production | T036-T053 | `guides/lot-04-tests-externalises.md` | non (tests déplacés) |
| 5 | Aiguillage découpé | T054-T068 | `guides/lot-05-aiguillage.md` | oui |
| 6 | `unsafe` justifiés | T069-T080 | `guides/lot-06-unsafe.md` | oui (T069, T070) |
| 7 | Concurrence observable | T081-T091 | `guides/lot-07-concurrence.md` | oui |
| 8 | Erreurs sans texte fragile | T092-T098 | `guides/lot-08-erreurs.md` | oui |
| 9 | Fonctions lisibles | T099-T113 | `guides/lot-09-fonctions.md` | oui |
| 10 | Format uniforme | T114-T118 | `guides/lot-10-format.md` | oui (forme seulement) |
| 11 | Modules nommés par rôle | T119-T124 | `guides/lot-11-modules.md` | oui (noms seulement) |
| 12 | Stockage documenté | T125-T127 | `guides/lot-12-stockage.md` | non |
| 13 | Scripts contrôlés | T128-T137 | `guides/lot-13-scripts.md` | scripts seulement |
| 14 | Tests ignorés expliqués | T138-T140 | `guides/lot-14-tests-ignores.md` | non |
| 15 | Proposition de signature | T141-T142 | `guides/lot-15-signature.md` | non |
| 16 | Écrans Android séparés | T143-T150 | `guides/lot-16-android.md` | oui (Android) |
| 17 | Code mort | T151-T154 | `guides/lot-17-code-mort.md` | oui |
| - | Clôture | T155-T157 | `quickstart.md` | non |

## Ordre d'exécution

| Vague | Lots, dans cet ordre | Raison |
|-------|----------------------|--------|
| A | 0, puis 1 | Le filet doit exister avant tout changement de code. |
| B | 2, 3, 12, 14, 15, 13, 17 | Peu de risque. Les lots 2, 3, 12, 14, 15 n'attendent pas la décision D1. |
| C | 4, 5, 6, 9, 16 | Découpages mécaniques. Le lot 4 passe avant le lot 5 : il allège `daemon.rs`. Le lot 5 passe avant le lot 9. |
| D | 7, 8 | Ajouts mesurés. |
| E | 10, puis 11 | Grands commits mécaniques. Ils attendent D9 et D10 et un dépôt calme. |

Un lot bloqué par une porte ne bloque pas les autres. L'implémenteur passe au lot suivant de la liste.

## Structure des artefacts

```text
specs/260-assainissement-code/
  spec.md  plan.md  research.md  data-model.md  quickstart.md  tasks.md
  decisions.md  releves.md  reuse-audit.md  analyze.md
  contracts/regles-implementeur.md   règles R1 à R8, commandes nommées
  contracts/controles-ci.md          contenu attendu des contrôles distants
  guides/lot-NN-*.md                 un guide par lot
  outils/extraire_module_tests.py    outil du lot 4
  validation/                        preuves de l'implémenteur (fichiers .txt)
  implementation.md                  journal, créé par T001
docs/decisions/260-assainissement-progressif.md   ADR
```

## Contrôle de constitution

| Article | Verdict | Justification |
|---------|---------|---------------|
| I langue | conforme | Artefacts en français. |
| III cycle SpecKit | conforme | Spec, plan, tâches. L'implémentation est exclue de ce tour par la demande. |
| VII ADR | conforme | `docs/decisions/260-assainissement-progressif.md`. Le lot 12 produit un second ADR, au statut « proposé ». |
| XV tests avant suite | écart justifié | « Zéro échec dans tout le projet » est impossible aujourd'hui : 40 échecs hérités sur Mac. Le programme fige cette liste et interdit de l'allonger. La réduire est hors périmètre. |
| XVI worktree | conforme | Un lot = une branche et un worktree (R1). |
| XVIII complexité | conforme | Le lot 7 mesure la boucle à requêtes par élément. Il ne la regroupe pas sans mesure ni décision D7. |
| XIX minimalisme | conforme | Deux abstractions seulement : `effective_uid()` (76 usages) et `RecoverPoison` (plus de 600 usages). Aucune dépendance Rust ajoutée. |
| XX responsabilité | conforme | Chaque tâche a une vérification. Aucune régénération de fichier : déplacements et remplacements ciblés. |
| XXII optimisation | conforme | Aucune optimisation sans mesure avant et après (lot 7). |

## Risques principaux

| Risque | Parade |
|--------|--------|
| Un déplacement perd ou renomme un test. | Comparaison de la liste triée des noms de tests (commande `C-NOMS`). |
| Un test sensible à la charge échoue par hasard. | Rejeu seul, trois fois, avant de conclure (liste dans `releves.md` section 15). |
| `main` avance pendant un lot long. | Réalignement avant intégration. Conflit sur fichier déplacé : arrêt (R4). |
| Le disque se remplit. | Un seul worktree de lot. Arrêt sous 12 Gio. |
| L'outil de build retire un worktree sans commit. | Committer avant le premier Cargo (R1). |
| Une chaîne durable change par remplacement de masse. | Contrôle `git diff -U0` sur les littéraux (lots 8 et 11). |
| Une version non activée est annoncée « livrée ». | Règle R8 et décision D1. |

## Limites de ce plan

- Les numéros de ligne de `releves.md` datent du commit `065f54f0`. Chaque tâche cherche par symbole.
- La recette de test et les fichiers de contrôle distant ne sont pas exécutés dans ce tour. T003, T010 et T020 les valident.
- La contre-revue par un autre agent n'a pas eu lieu : aucun agent n'était connecté. Voir `adversarial-review-indisponible.md`.
