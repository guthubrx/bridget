# 227 — Guidage du démarrage distant non confirmé

Correction autorisée après le signalement piétons du 2 novembre 2026.
Pour remote_start:WorkerUnavailable ou StaleConnection, le statut indique
l'absence de confirmation du démarrage, la portée limitée à cette tâche,
les références connues et la vérification native manquante.
Il ne déduit ni panne réseau, ni compte invalide, ni arrêt physique.
Il indique explicitement si la remise métier est enregistrée.
Il interdit une nouvelle mission automatique et conserve la clé d'origine.
Un résultat ultérieur ou une remise enregistrée ne restent pas présentés
comme une panne actuelle sur la seule erreur historique de lancement.
Aucune modification du lancement, des droits, délais ou gardes persistantes.
Le correctif doit être chargé et vérifié sur les hôtes concernés.
