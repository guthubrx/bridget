# Recherche et décision 136 (ADR)

## Preuves locales

64 IDs distincts, cx-coordinator → 3D-collision, 2026-10-05 21:36 à 03:20.
batch_envelope recopie les body ; ThreadNotify::Targets([]) est déjà silencieux.
Store threads: historique immuable, reçu snapshot et alerte par membre.

## Décision

Étendre dépôt/projection du fil102. Classe et remplacement explicites, même
auteur/audience. Messages directs restent pour urgences autonomes/demandes suivies.
Pas de nouvelle table de missions, résumé automatique ou suppression.
Alternatives rejetées: regex (sens incertain), dernier message seul (perte des
blocages), résumé IA (coût/interprétation), nouveau bus (doublon102).
Anciennes données libres conservées telles quelles. Pas de propagation d'un
statut de réussite : la lecture reste une réception, pas une acceptation.

## Sources et limites

Baselines charge cognitive, gestion des connaissances, tests consultées.
Recherche primaire 2026-10-06, aucune donnée du projet transmise.
https://www.microsoft.com/en-us/research/publication/the-impact-of-generative-ai-on-critical-thinking-self-reported-reductions-in-cognitive-effort-and-confidence-effects-from-a-survey-of-knowledge-workers/
Étude CHI2025 sur319 travailleurs, perceptions de pensée critique, pas ce pont.
https://arxiv.org/abs/2501.02684 : pratiques développeur et charge cognitive.
Décision: inférence d'ingénierie locale, aucun gain de productivité chiffré déduit.
Ni RAG ni moteur vectoriel nécessaires ; leur ajout serait hors périmètre.

## Inconnues résolues

Projection de replay évaluée au snapshot, pas à la tête actuelle.
Conflit de remplacement: unicité et validation transactionnelle.
Audience: ensembles effectifs, pas mode all/targets.
Ancien serveur: nouveaux champs refusés. Ancien dépôt: omission des métadonnées.
