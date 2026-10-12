# T011 — suppression historique par entretien au démarrage

Les cinq lignes absentes sont3answered et2timed_out. Toutes satisfont state !=open et completed_at <cutoff. Les57 profils restants ne changent pas ; aucune ligne neuve. Les identifiants sont hachés et aucun corps privé n’est lu.

Store::purge_older_than_days supprime ces lignes au démarrage du daemon. Le défaut de référence est le vieillissement de la sauvegarde : la rétention est7jours. Le code de purge est byte-identique au source main158B. La migration T003 ne supprime pas ces tracked_requests.

Cutoff réel : floor(SystemTime secondes)-604800. L’instant du prélèvement n’est pas journalisé. La naissance du PID et son TERM bornent cutoff dans[1791214539,1791214540]. Les cinq lignes passent le prédicat même à la borne basse. Aucun cutoff exact non observé n’est inventé.

Recette minimale : ROOT crée une nouvelle sauvegarde readonly v6 séparée, avec SHA/date ; l’ancienne référence et le FAIL restent historiques. Reprendre le script existant avec cette référence, sans modifier ni nettoyer les données. Comparer encore toutes les lignes. Le prochain completed_at connu par ROOT franchit7jours dans environ680s à son instant de lecture : exécuter rapidement, et refuser une référence devenue périmée. Aucun nouveau drapeau daemon de rétention n’existe ; aucun outil/helper/option neuf nécessaire.

Preuves détaillées : /Users/moi/.cache/bridget-restoration159A-preparation-20261012/T011-maintenance-diagnostic/diagnostic.json
Aucun PASS restauration neuf annoncé. Aucune production ouverte par cet owner.
