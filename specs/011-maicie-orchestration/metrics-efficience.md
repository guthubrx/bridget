# Métriques d'efficience — avant/après Maicie + contrat idempotent

**Regravé le 2026-08-23 (soir)** (la première version du 23 n'avait jamais été
commitée — leçon : toute métrique se grave le jour même, rituel de clôture).

## Sources

- **Avant** (journée du 2026-08-22, coordination manuelle pré-Maicie,
  pré-contrat idempotent) : relevé du référent, reconstitué — ordres de
  grandeur, non re-vérifiables (le fichier d'origine a été perdu).
- **Après** (2026-08-23 soir) : greffe Maicie (`maicie status`),
  historique git, ledger — chaque chiffre re-calculable à la commande.

## Coordination

| Métrique | Avant (22/08) | Après (23/08) |
|---|---|---|
| Relances manuelles du référent | ~526 / jour | ~0 (rappels automatiques du daemon ; les envois du référent sont du contenu, plus du harcèlement) |
| Messages de coordination | ~495 / jour | ~120 sur la journée, tous porteurs (mandats, verdicts, arbitrages) |
| Sweeps / polling | ~50 / jour | 0 (réveil par événements exclusivement) |
| Sleeps bloquants | fréquents (`sleep 180 && send`) | 0 |
| Missions journalisées | 0 (mémoire de conversation) | 30 objectifs au greffe (26 clos, 4 actifs), motifs complets |
| Cycle médian d'une mission | non mesurable | **20 min** (min 3, max 46) |

## Production (journée du 23/08, ~13 h, vs journée type avant)

| Livré 23/08 | Volume |
|---|---|
| Session 011 (Maicie complète) | 26 tâches, revue hostile finale MERGEABLE |
| Session 014 (observabilité) | 7 tâches, **conception→merge en une soirée** |
| Bloc C micro + D24 | 5 tâches (dont clôture de la spec 001, la doyenne) |
| Session 015 (guichet) | conception + contrat gelés (5 tours, 8 P1), 3 lots en chantier |
| Constats de review trouvés/fermés | ~35, zéro régression aux merges |
| Gate MVP (remise/statut/clôture réels) | stable 1,5-1,8 s |

## Tokens

Non mesurables par mission à ce jour — aucune télémétrie tokens/mission
(candidat : métrologie du greffe, le registre a déjà les horodatages).
Réductions structurelles constatées qualitativement : mandats autoportants
(zéro re-explication de contexte), zéro boucle de polling, reviews ciblées
fichier:ligne, enquêtes sur logs sans consommation modèle (revendiqué et
constaté sur l'enquête MCP). Proxy disponible : quota hebdomadaire codex.

## Lecture honnête

Le gain dominant n'est pas la vitesse brute d'une tâche — c'est la
**suppression du travail de harcèlement** (526→0 relances) et la
**qualité forcée par structure** : chaque livraison vérifiée sur pièces,
chaque review liée, chaque décision au greffe. Le coût : des cycles de
review plus nombreux (5 tours sur le contrat 015) — payés en minutes,
économisés en semaines.

## Mesure du 2026-08-24 midi — références pour la mesure quotidienne

Méthode : git (commits, merges, numstat sans merges) + tokens du référent
depuis les journaux de session (lignes `usage`, fenêtres locales = UTC+2).
Les ratios par unité produite sont la métrique retenue par l'utilisateur.

| Fenêtre | Commits | Fusions | Lignes+ | Tours réf. | Tokens sortie | tok/commit | tok/fusion | tok/ligne |
|---|---|---|---|---|---|---|---|---|
| A : 23→24 midi (Maicie greffier) | 266 | 38 | 71 015 | 4 202 | 2 679 614 | 10 073 | 70 516 | 37,7 |
| B : 22→23 midi (avant) | 213 | 10 | 53 588 | 2 407 | 1 637 948 | 7 689 | 163 794 | 30,6 |

Lecture : +31 % par commit et +23 % par ligne (le coût de la coordination),
mais **−57 % par fusion** — l'unité livrée-relue-intégrée coûte moitié moins.
La fenêtre A portait 5 h 30 de panne (22h27→03h57). Mesure suivante planifiée
le 25/08 à 12h03 (fenêtre C, coordination AUTONOME attendue en service) —
hypothèse : les ratios du référent baissent, les relances et transcriptions
étant reprises par Maicie.

### Raffinement du 24/08 (analyse livrée après clôture, intégrée)

Corrections à retenir sur la mesure de midi :
- **Le surcroît de lignes de A est de la DOC, pas du code** : production
  A +27 629 contre B +34 467 — la fenêtre Maicie a écrit MOINS de code de
  production et TROIS FOIS plus de documentation (+36 778 contre +8 997).
  Le « +33 % de lignes » brut était trompeur.
- Panne mesurée au trou de commits : 5,50 h exactement (22:27:57→03:57:56).
- Tokens « facturables » (entrée+sortie+création de cache, hors lecture) :
  A ≈ 21,2 M, B ≈ 15,9 M. **Par fusion : 0,68 M contre 1,45 M — le ÷2 du
  coût par livraison intégrée est confirmé par la seconde méthode.**
- Taux de réponse des demandes : 78 % (A) contre 36 % (B) — la coordination
  répond, l'avant laissait mourir 6 demandes sur 10 en timeout.
- Le « ×139 missions » est un artefact de naissance d'outil (première
  délégation à 11:55, 4 min avant la frontière) — à ne plus citer comme
  rendement.
- Tri de sessions fiable = par `cwd` et branche, JAMAIS par mots-clés
  (la liste d'outils injectée contient h3/horizon/ltx partout).

---

## Fenêtre C — 24/08 12:00 → 25/08 12:00 CEST

Mesure exécutée par le référent (aucun agent Cursor libre : les onze étaient mandatés).

### Production (git, toutes branches et worktrees, SHA uniques, date auteur)

| | |
|---|---|
| Commits uniques | **275** |
| Fusions | **41** |
| Non-merge | **234** |
| Lignes non-merge — **prod** | **22 701** |
| Lignes non-merge — test | 6 631 |
| Lignes non-merge — doc | 5 075 |
| **Total lignes** | **34 407** |

### Indisponibilité (mesurée, non déclarée)

**Plus grand trou de commits : 299 minutes** — 24/08 21:53 → 25/08 02:53.
Cause connue et documentée : le shell système a été écrasé par un agent qui « réparait » un `ENOENT` mal lu ; toute exécution était impossible. 255 commits horodatés dans la fenêtre.

### Greffe

Objectifs créés dans C : **111**. Objectifs clos dans C : **91** (via `mis_a_jour_at`, état `clos`).
État global du greffe en fin de fenêtre : 271 clos, 29 en coordination, 1 à évaluer.
*Aucun ratio vs B n'est cité : artefact de naissance d'outil, retiré du protocole.*

### Ledger

| | |
|---|---|
| Messages | **1 486** |
| Demandes suivies | **77** |
| `answered` | 67 |
| `timed_out` | 6 |
| `cancelled` | 4 |
| **Taux de réponse** | **87 %** |

Référence : A = 78 %, B = 36 %. **C est le meilleur des trois**, et dépasse A de 9 points.

### Tokens du référent (tri par `cwd`, jamais par mots-clés)

70 fichiers retenus (`cwd` sous `10.Scripts/bridget`), 27 sessions avec usage dans la fenêtre.

| | |
|---|---|
| input | 22 200 |
| output | 13 151 132 |
| cache_creation | 35 300 353 |
| **Facturable** | **48 473 685** |
| cache_read (à part) | 4 236 696 806 |

### Ratios

| Métrique | C | B | A |
|---|---|---|---|
| **Facturable / fusion** | **1,18 M** | 1,45 M | 0,68 M |
| Facturable / commit non-merge | 0,207 M | — | — |
| Facturable / ligne de prod | 2 135 | — | — |

**C vs B : −18 %. C vs A : +74 %.**

### Hypothèse testée — la coordination autonome de Maicie réduit-elle les tokens du référent par unité livrée ?

**Partiellement confirmé, et l'attribution reste incertaine.**

Le coût par fusion baisse de 18 % entre B et C, et le taux de réponse aux demandes passe de 36 % à 87 %. Ce second chiffre est le plus solide : il mesure directement la coordination, et non le volume produit.

**Mais l'attribution à Maicie seule n'est pas établie**, pour trois raisons mesurées :
- une **indisponibilité de 299 minutes** ampute la fenêtre d'un cinquième, sans que la production s'effondre — ce qui déforme tous les ratios ;
- **huit agents distants** ont été mis en service dans la seconde moitié de la fenêtre, déportant des compilations dont le coût en tokens n'est pas comptabilisé ici ;
- le référent a passé une part importante de la fenêtre à **corriger ses propres constats** — quatre rectifications de faits qu'il avait portés à tort au registre. Ce coût est bien dans les tokens, mais il n'est pas un coût de coordination.

**Non mesurable en l'état** : la part du gain due à Maicie contre celle due au déport et à la discipline de mesure adoptée dans la fenêtre.

### Limites déclarées

- **Tokens Codex et Cursor absents** : seuls ceux du référent sont mesurables. Avec vingt-six agents actifs dont huit distants, la consommation réelle du chantier est très supérieure et non quantifiée.
- **Qualité non chiffrable.** Verdicts de revue de la fenêtre, depuis le greffe : **7 Blocker, 32 Major, 11 Minor, 2 Info**, et **47 constats portés le 25/08**. Quatre branches jugées caduques et archivées par tag après vérification indépendante. Trois trailers d'auteur interdits détectés, dont **26 occurrences sur `main`** — découverte d'un juré, non résolue.
- **Non-portabilité découverte** : le gate de référence (`--features test-support`) ne compile pas sur Linux, ce qui limite ce que les jurés distants peuvent mesurer. Constat porté.
