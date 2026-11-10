# Analyze 260 - 10 novembre 2026

Primitive appliquée : lecture croisée spec / plan / tâches / guides, protocole manuel (le projet n'a ni modèles `.specify/templates/`, ni scripts, ni hooks). Corrections autonomes autorisées par `my.specify-all`. Pas de hook obligatoire dans le projet Bridget. Deuxième lecture après corrections ci-dessous.

## Constats

| ID | Niveau | Constat | Traitement |
|----|--------|---------|------------|
| A1 | HIGH | `decisions.md` renvoyait D7 à T106 et D13 à T114. Dans `tasks.md`, T106 découpe `spawn_reader` et T114 est la porte D9. | Corrigé : D7 renvoie à T090, D13 à T097 (colonne « Défaut » et colonne « Bloque »). Relu. |
| A2 | MEDIUM | `quickstart.md` et `validation/README.md` disaient que `base-tests.txt` pouvait être commité, alors que `validation/.gitignore` exclut `*-tests.txt`. `git add` aurait échoué. | Corrigé : `base-tests.txt` n'est pas commité ; les listes d'échecs et de noms le sont. |
| A3 | MEDIUM | Les noms de tests contiennent le chemin du module. Le lot 11 renomme 18 modules : `C-NOMS` et `C-ECHECS` auraient donné des écarts. | Corrigé : le guide du lot 11 (T119, T124) prépare des lignes de base transformées par `validation/lot11-renommages.txt`. |
| A4 | MEDIUM | La porte de lot du modèle de données exigeait `C-NOMS` identique, alors que les lots 6 et 7 ajoutent volontairement des tests (+1, +3). | Corrigé : `data-model.md` et les guides 6 et 7 attendent exactement les lignes ajoutées et actualisent la base. |
| A5 | MEDIUM | `outils/longues_fonctions.py` relève 49 fonctions de 300 lignes ou plus (relevé initial : 43) et rate 4 des 9 fonctions de 15 paramètres. | Accepté : c'est une mesure de contrôle (dit dans l'en-tête et le guide). Les 9 fonctions sont nommées dans le guide du lot 9 et se cherchent par nom. |
| A6 | MEDIUM | `#[path]` : la résolution du chemin n'est pas compilée dans ce tour (aucun Cargo). | Vérifié par lecture : le motif existant `daemon/native_delegation.rs` + `native_workspace154_tests.rs` dans le même dossier confirme la règle de l'outil. Le premier `C-CHECK` réel est T037. Risque résiduel nommé. |
| A7 | MEDIUM | La contre-revue par un agent d'un autre fournisseur n'a pas eu lieu (aucun agent connecté). | Consigné dans `adversarial-review-indisponible.md`. Aucune équivalence prétendue. |
| A8 | LOW | Les numéros de ligne de `releves.md` datent du commit `065f54f0` et du début du lot 4. | Les guides cherchent par symbole. Dit dans `tasks.md` et `plan.md`. |
| A9 | LOW | Les workflows `rust.yml` et `rust-audit.yml` ne sont pas exécutés. | Marqué « non exécuté » dans `contracts/controles-ci.md` ; T010 et T020 les valident. |
| A10 | LOW | SC14 (un agent moins capable exécute un lot sans question hors portes) n'est vérifiable qu'au premier lot réel. | Nommé comme critère à mesurer à l'exécution du lot 0 ; non prouvé par ce tour. |
| A11 | LOW | Disque à 10 Gio libres pendant la rédaction, sous le seuil de 12 Gio de l'implémenteur. | Signalé dans le rapport. T002 le contrôle avant toute compilation. |

## Couverture

| Exigence | Tâches |
|----------|--------|
| FR01 | T008, T019 |
| FR02 | T010, T019 |
| FR03 | T010, T021, T024 |
| FR04 | T020 |
| FR05 | T009 à T019 |
| FR06 | T025, T028 |
| FR07 | T026, T027 |
| FR08 | T031 à T035 |
| FR09 | T032, T034, T035 |
| FR10 | T036 à T053 |
| FR11 | T054 à T068 |
| FR12 | T069 à T080 |
| FR13 | T069 à T072 |
| FR14 | T081 à T087 |
| FR15 | T088 |
| FR16 | T089, T090 |
| FR17 | T092 à T098 |
| FR18 | T100 à T104 |
| FR19 | T105 à T113 |
| FR20 | T114 à T118 |
| FR21 | T119 à T124 |
| FR22 | T125 à T127 |
| FR23 | T128 à T137 |
| FR24 | T138 à T140 |
| FR25 | T141, T142 |
| FR26 | T143 à T150 |
| FR27 | T151 à T154 |
| FR28 | règle R6 et portes de chaque lot |
| FR29 | T003 et portes de lot (`C-ECHECS`) |
| FR30 | `tasks.md` et 17 guides |
| FR31 | `decisions.md` et section « Dépendances » de `tasks.md` |
| FR32 | T001 et format du journal dans `contracts/regles-implementeur.md` |

Critères : SC01 = T010, T019 ; SC02 = T019 ; SC03 = T026, T030 ; SC04 = T034, T035 ; SC05 = T053 ; SC06 = T067 ; SC07 = T079 ; SC08 = T072 ; SC09 = T113 ; SC10 = T118 ; SC11 = T140 ; SC12 = portes de lot ; SC13 = T156 ; SC14 = voir A10.

32 exigences couvertes sur 32. 157 tâches numérotées sans saut de T001 à T157, conformes à la table de `plan.md`. 14 critères de réussite sur 14 reliés à une tâche. Deuxième lecture : aucun CRITICAL restant ; un seul risque résiduel de compilation (A6) et un critère non prouvable à froid (A10).

Articles XIX et XX : deux abstractions neuves seulement (`effective_uid()` : 76 usages ; `RecoverPoison` : plus de 800 sites mesurés), plus la garde de connexion (1 usage, justifiée par les sorties anticipées) et les structures d'arguments (9, justifiées par FR18). Aucune dépendance ajoutée. Aucun texte généré en masse : déplacements et remplacements ciblés, chaque tâche a une vérification.

Décision : PASS des artefacts. Ce verdict ne prouve ni code, ni tests, ni livraison.
