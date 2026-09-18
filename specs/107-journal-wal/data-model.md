# Modèle 107

Aucune table, colonne ni type ajouté. Propriété de fichier : `journal_mode = wal` dans l'en-tête de
`bridget.db` (persistant). Fichiers auxiliaires gérés par SQLite : `bridget.db-wal` (journal d'écriture
anticipée), `bridget.db-shm` (index partagé), mêmes droits que la base, supprimés proprement à la fermeture
de la dernière connexion. Invariant vérifié à l'ouverture : la valeur rendue par le pragma est `wal`
(ou `memory` pour une base en mémoire).
