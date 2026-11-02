# Validation et diagnostic

Incident : requête opus2d-pietons-r56-capacite-index-revue-20261102-02, tâche
7024f34c-22df-4b6a-b16f-8b37907351b8, enfant
33b7e243-efa4-4734-ba74-35a5c1e79238, worker SYS.
Lecture du maître : starting, launch_issued_at1793642118, délai1793642418,
commande7024f34c, instance d9ce3451-e872-4a0a-9999-38baf0c4e19a, génération1890,
mission_issued_at null, résultat nul. Aucun champ de cette tâche réécrit.

Worker PID1528907 inchangé : dossier préparé au commit demandé, reçu de
préparation présent, prepare_digest2083ff2dd851f353efe470ae32399403abf554d38c13e44ce9788ce08f9c4e2d.
Aucun reçu de commande dans le magasin exact. Le code conserve ces reçus
terminaux au moins sept jours ; aucun fichier de journal ou processus corrélé
observé par les lectures ciblées. Aucun lancement métier attesté.
La cause précise de l’absence de confirmation reste inconnue. Les refus de
sondes contemporains ne sont pas attribués à ce démarrage sans preuve.
L’absence seule de reçu ne ferme pas une commande potentiellement en transit ;
aucun Fence, Cancel ou nouveau Start effectué pour qualifier cet incident.

Défaut confirmé : task_view n’expliquait pas les deux codes remote_start
WorkerUnavailable/StaleConnection. Le delta porte seulement la projection
d’erreur : portée, clé, machine connue, contrôles et faits non observés.
Le modèle de tâche, les délais, l’admission et les droits restent identiques.

Deux tests ciblés Mac passent en0,13s. Admission/relecture stable, reçu durable
inchangé, aucune remise ni nouvelle tâche ; erreur historique séparée après
remise ou résultat, famille ResourceLimit distincte, terminal sans faux arrêt.
Premier essai Mac : fixtures refusées pour socket trop longue sous le TMPDIR
système. Cet essai ne qualifie pas le correctif. Relance avec TMPDIR=/private/tmp.
Contrôle de non-régression de la confirmation Release : un test passe en0,07s.
Trois tests distincts acquis ; aucun autre test relancé. La version du paquet
est ensuite portée à0.7.6 sans autre changement du comportement testé.

Preuves privées : /Users/moi/.cache/bridget227-start-guidance.
Revue, assemblages, chargement et relecture native du propriétaire : à compléter.
