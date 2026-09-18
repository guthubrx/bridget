# Journal WAL pour la base du daemon

Date : 2026-09-18. Statut : Accepté (session 107).

## Contexte

`bridget.db` était en mode journal `delete` (constaté en production). Chaque validation d'écriture
prenait un verrou exclusif sur le fichier ; les lecteurs concurrents (recherche 104 avec attente de
100 ms, relecture d'artefacts, sondes) recevaient `SQLITE_BUSY` pendant les validations. Mesuré en
test : 96 747 lectures refusées sur 100 001 pendant 50 validations (mode `delete`).

## Décision

`Store::open`, après le contrôle de schéma et avant l'initialisation, relit le mode journal, pose
`PRAGMA journal_mode=WAL` si nécessaire (réessai borné à 2 s : SQLite n'invoque pas le gestionnaire
d'attente sur cette transition) et vérifie la valeur rendue (`wal`, ou `memory` pour une
base en mémoire) ; sinon l'ouverture échoue explicitement. Le mode est une propriété persistante du
fichier : les autres ouvreurs du même fichier (idempotence, exécution, profils, artefacts, lecteurs en
lecture seule) l'héritent sans modification. `synchronous` reste au défaut `FULL` : aucune relaxation de
la durabilité promise par l'envoi idempotent 099.

## Conséquences

Positives : lecteurs jamais bloqués par une validation (0 `SQLITE_BUSY` mesuré sur le même banc) ;
écrivains toujours sérialisés par `busy_timeout` ; aucune migration de données ; conversion d'une base
existante à la première ouverture, idempotente.
Négatives : deux fichiers auxiliaires (`-wal`, `-shm`) à côté de la base tant que le daemon tourne ;
une copie à chaud de `bridget.db` seul est incomplète (documenté : `docs/demarrage-a-froid.md`,
`README.md`, exclusion `*.db-*` dans `scripts/deploy-remote.sh`) ; `BEGIN EXCLUSIVE` ne bloque plus
les lecteurs (le test 104 S14 provoque désormais le refus via `locking_mode=EXCLUSIVE`).

## Hors périmètre

`synchronous=NORMAL`, `wal_autocheckpoint`, `mmap`, sauvegarde en ligne : à mesurer et spécifier
séparément.
