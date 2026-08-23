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
