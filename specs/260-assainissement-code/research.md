# Recherche 260 - choix et alternatives écartées

Date : 10 novembre 2026. Les mesures sont dans `releves.md`. Ce fichier donne les choix et leurs raisons.
Format : Choix, Pourquoi, Écarté.

## C1 - Preuve de comportement constant par comparaison de listes

- Choix : comparer la liste triée des tests en échec à une ligne de base figée. Pour un déplacement, comparer aussi la liste triée des noms de tests.
- Pourquoi : la suite a 40 échecs hérités sur Mac (`releves.md` section 1). « Tout est vert » est donc inutilisable comme critère. Une liste identique est un critère simple qu'un agent moins capable vérifie par `diff`.
- Écarté : corriger d'abord les 40 échecs (hors périmètre, cause par cause inconnue) ; compter les succès (un test perdu et un test nouveau se compensent).

## C2 - Un lot = une branche courte intégrée à `main`

- Choix : 17 lots indépendants, un worktree par lot, intégration à la fin de chaque lot.
- Pourquoi : `main` avance chaque jour (sessions 250 à 259 en dix jours). Une branche longue produit des conflits sur `daemon.rs`. Un lot court se réaligne sans peine.
- Écarté : une seule branche « 260 » pour tout le programme ; un seul grand commit par thème.

## C3 - Filet avant tout changement (lot 1 en premier)

- Choix : commande locale `make lint`, lints du workspace, contrôle distant, avant les découpages.
- Pourquoi : sans contrôle automatique, chaque lot suivant peut réintroduire ce que le lot précédent a retiré.
- Écarté : poser le filet à la fin (les 385 avertissements reviendraient pendant le programme).

## C4 - Dette structurelle d'analyse placée en `allow` au niveau du workspace

- Choix : `too_many_arguments`, `type_complexity`, `result_large_err`, `large_enum_variant` passent en `allow` dans `[workspace.lints.clippy]`, avec un commentaire qui renvoie au lot 9.
- Pourquoi : 78 signalements qui demandent une refonte de signatures ou de types sérialisés. Les corriger change des interfaces, parfois le protocole (interdit R6).
- Écarté : 78 `#[allow]` locaux (bruit) ; refonte immédiate (risque).

## C5 - Tests en observation avant d'être bloquants

- Choix : le contrôle distant des tests tourne en `continue-on-error` jusqu'à la décision D16. Trois exécutions Linux fixent la ligne de base (T021).
- Pourquoi : la ligne de base Linux est inconnue. Des tests sont sensibles à la charge (`releves.md` section 15).
- Écarté : tests bloquants dès le premier jour (blocage de toutes les livraisons sur un faux échec).

## C6 - Externalisation des tests par un outil déterministe

- Choix : `outils/extraire_module_tests.py` déplace un module de tests en ligne vers un fichier voisin, avec `#[path]`.
- Pourquoi : 16 fichiers, plus de 110 000 lignes à déplacer. Un déplacement à la main par un agent se trompe d'accolade. L'outil refuse les cas ambigus au lieu de deviner.
- Écarté : dossier `tests/` d'intégration (les tests accèdent à des éléments privés par `use super::*`) ; édition manuelle.

## C7 - Aiguillage : une fonction par branche, même module, même signature de retour

- Choix : les 37 branches de 30 lignes ou plus deviennent des fonctions `on_xxx` dans `daemon/wrapper_dispatch.rs`. Une branche par commit.
- Pourquoi : déplacement pur, vérifiable par compilation. Le `match` reste le sommaire.
- Écarté : un trait « handler » par message (abstraction sans besoin, article XIX) ; table de fonctions (perd la vérification d'exhaustivité du `match`).

## C8 - `unsafe` : une fonction partagée, puis des commentaires

- Choix : `effective_uid()` dans `bridget-transport/src/fsutil.rs` remplace 76 lectures. Les autres blocs reçoivent un commentaire `// SAFETY:`. Le lint `undocumented_unsafe_blocks` ferme le lot.
- Pourquoi : 76 usages justifient une fonction. Les autres appels (`kill`, `flock`, `fcntl`, `setsid`) ont des contextes différents.
- Écarté : dépendance `nix` ou `rustix` (nouvelle dépendance, interdit R6) ; une enveloppe par appel système.

## C9 - Concurrence : mesurer, ne rien changer

- Choix : trait `RecoverPoison` (consigne et compte, puis poursuit comme aujourd'hui), compteur de connexions câblé, durée du tour mesurée avec seuil.
- Pourquoi : FR14 interdit de changer la poursuite du service. L'article XXII interdit d'optimiser sans mesure.
- Écarté : arrêt du service sur verrou empoisonné ; plafond de connexions (D6) ; regroupement des requêtes du tour (D7) ; compteurs dans `get_status` (protocole, interdit).

## C10 - Erreurs : constante partagée avant type

- Choix : niveau 1, une constante pour chaque texte produit par Bridget et comparé ailleurs. Texte externe : commentaire qui nomme la source. Niveau 2 (types) : seulement sur liste validée (D13).
- Pourquoi : 375 comparaisons. Une constante supprime le risque de divergence sans changer une signature.
- Écarté : `thiserror` partout (dépendance, refonte) ; ne rien faire.

## C11 - Format en un seul commit tardif

- Choix : `cargo fmt --all` en un commit isolé au lot 10, SHA dans `.git-blame-ignore-revs`.
- Pourquoi : 120 fichiers touchés. Avant les découpages, ce commit ferait entrer en conflit tous les lots ouverts.
- Écarté : format fichier par fichier (historique illisible) ; format en premier.

## C12 - Stockage : inventaire et ADR proposé, aucune unification

- Choix : le lot 12 documente les trois mécanismes de version et les modes de journal. Il propose un ADR. Il ne modifie rien.
- Pourquoi : un changement de schéma est un format durable (R6). L'unification demande une décision humaine.

## C13 - Contrôles distants : actions épinglées par empreinte

- Choix : `actions/checkout` épinglé par SHA, `permissions: contents: read`, `persist-credentials: false`, outils d'audit installés à version fixe.
- Pourquoi : dépôt public. Même règle que le workflow existant `repository-hygiene.yml`.
- Écarté : actions tierces de mise en cache ou d'installation (surface d'attaque, article XIX).

## Inconnues restantes

| Inconnue | Tâche qui la lève |
|----------|-------------------|
| Ligne de base des tests sur Linux | T021 |
| Recette de test complète sur un worktree neuf | T003 |
| Fichiers de contrôle distant jamais exécutés | T010, T020 |
| Durée réelle du tour de délégation | lot 7, première mesure |
| Nombre exact de comparaisons de texte interne (lot 8) | lot 8, classement |
