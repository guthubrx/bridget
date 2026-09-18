# Spécification 107 — Journal WAL pour la base du daemon

## Fiche synthèse

Spec: 107-journal-wal
Statut: Implemented (2026-09-18, diff non commité)
Priorité: P2
Tâches: 9/9 (100%)
Tests: 4 unitaires spec107 + S14 (104) adapté + régressions 089 (concurrence, migration) ; recette release 1513/0
Date: 2026-09-18
Branche: session-107-journal-wal
Dépendances : 104 (lecteurs en lecture seule concurrents), 099 (envoi idempotent : durabilité des remises),
101/102 (écritures fréquentes : observations, fils). Aucune dépendance de code nouvelle.

## Problème

La base du daemon (`bridget.db`) est en mode journal `delete` (constaté en production le 2026-09-18).
Dans ce mode, chaque validation d'écriture prend un verrou exclusif sur le fichier : les lecteurs
concurrents (recherche 104, relecture d'artefacts, sondes en lecture seule) reçoivent `SQLITE_BUSY` si la
validation dure plus que leur attente (100 ms pour 104). La recette complète 104 l'a observé : une
page de recherche a reçu `storage_unavailable` pendant les 200 messages témoins. Le contrat 104
prescrit le rejeu, mais la cause est évitable.

## Objectif

Les lecteurs ne sont plus bloqués par les validations d'écriture du daemon, sans changer les garanties
de durabilité ni le format des données, et sans nouvelle dépendance, table ou service.

## Scénarios utilisateur

- **US1 (P1) — Lire pendant qu'on écrit.** Un agent cherche dans ses échanges pendant qu'un autre envoie
  des messages : la recherche répond sans `storage_unavailable` dû au verrou d'écriture.
- **US2 (P1) — Base existante.** Un daemon relancé sur une base historique en mode `delete` bascule en
  WAL à l'ouverture, une fois, sans perte ni migration de données ; une relance ultérieure reste en WAL.
- **US3 (P2) — Fichiers d'état.** Les fichiers auxiliaires `bridget.db-wal` et `bridget.db-shm` ont les
  mêmes droits privés que la base ; les opérateurs savent qu'une copie de `bridget.db` seul est incomplète
  tant que le daemon tourne.

## Exigences fonctionnelles

- **FR-001** : à l'ouverture de la base par le daemon, le mode journal effectif est `wal` (vérifié, pas
  supposé) ; si SQLite refuse la bascule, l'ouverture échoue avec une erreur explicite plutôt que de
  continuer silencieusement en `delete`.
- **FR-002** : une base ouverte en mémoire (tests) n'est pas concernée par cette exigence.
- **FR-003** : une base existante en `delete` est convertie à l'ouverture sans perte : mêmes lignes avant et
  après, mêmes schémas, réouverture idempotente.
- **FR-004** : pendant une transaction d'écriture en cours et pendant sa validation, une connexion en lecture
  seule (104) lit sans `SQLITE_BUSY`.
- **FR-005** : les garanties de durabilité restent celles du réglage `synchronous` par défaut (`FULL`) :
  aucune relaxation dans cette spec.
- **FR-006** : les connexions en lecture seule existantes (préflight de schéma, lecteurs 104, artefacts)
  fonctionnent sur une base WAL, y compris juste après conversion.
- **FR-007** : les fichiers `-wal` et `-shm` créés par le daemon sont privés (0600, même propriétaire).
- **FR-008** : la contention écrivain-écrivain conserve son comportement (attente bornée puis erreur),
  les tests existants qui la simulent restent verts.
- **FR-009** : les tests qui simulaient un blocage des lecteurs par un verrou d'écriture (104 S14) sont
  adaptés pour provoquer un vrai `SQLITE_BUSY` en WAL, sans affaiblir leur assertion.
- **FR-010** : la documentation d'exploitation mentionne les fichiers auxiliaires et la règle de copie.

## Critères de succès

- **SC-001** : sur une base fixture, un écrivain tient une transaction avec écriture non validée puis la valide
  pendant qu'un lecteur en lecture seule lit en boucle : 0 `SQLITE_BUSY` côté lecteur (100 itérations).
- **SC-002** : conversion d'une base `delete` de 10 000 lignes : lignes identiques, `journal_mode = wal`
  après deux réouvertures.
- **SC-003** : recette 104 complète (`search_104_test`) verte, dont S14 reformulé, et le banc S25 sans rejeu
  `storage_unavailable` (rejeux = 0 en release, noté dans la mesure).
- **SC-004** : recette release complète du workspace verte.
- **SC-005** : `bridget.db-wal` et `bridget.db-shm` du daemon isolé ont le mode 0600.

## Hors périmètre

`synchronous=NORMAL`, `wal_autocheckpoint` personnalisé, `mmap`, second fichier, migration de schéma,
sauvegarde en ligne (`VACUUM INTO`) : à mesurer et spécifier séparément si un besoin apparaît.

## Checklist de validité

- [x] Besoin utilisateur explicite (idée capturée en fin de 104).
- [x] Exigences mesurables, sans détail d'implémentation.
- [x] Dépendances déclarées.
- [x] Hors périmètre écrit.
