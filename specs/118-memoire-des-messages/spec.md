# Spécification 118 — Un grand fil T3 n'est recopié qu'une fois

## Fiche synthèse
Spec: 118-memoire-des-messages | Statut: Implemented | Priorité: P0 | Date: 2026-09-25
Branche: session-118-memoire-des-messages | Correctif, suite de la 117.

## Problème observé
Le 25/09 dès 00:13 UTC, le coordinateur `sol_city_ai` reçoit une paire de notifications toutes les
6 secondes : « surveillance interrompue : source indisponible », puis « couverture modifiée ». La
source est le fil `opus-city-glmkrkr4` (agent `ea69b65b`).

Données : ce fil compte 1 363 messages ; le pont n'en retient que 1 000 comme déjà recopiés, en
oubliant les plus anciens. À chaque lecture, les oubliés paraissent nouveaux et sont recopiés, ce qui
en fait oublier d'autres : 146 644 lignes en un jour pour 1 363 messages (jusqu'à 131 copies
chacun), 336 Mo de journal, file d'écriture saturée en permanence. Le pont annonçait en outre la
source prête avant de vérifier son journal, puis indisponible : deux notifications par lecture.

## Exigences
- **FR-001** : un message encore montré par T3 n'est jamais oublié ; au-delà de la borne, seul ce
  que T3 ne montre plus est oublié.
- **FR-002** : chaque message d'un fil, quelle que soit sa taille, est recopié une seule fois.
- **FR-003** : un journal en retard produit une annonce d'indisponibilité, sans alternance ; une
  lacune reste signalée sur une source déclarée.

## Critères de succès
- **SC-001** : un fil de SEEN_BOUND + 50 messages, lu quatre fois, a exactement une copie par message.
- **SC-002** : deux lectures d'un journal en retard produisent une seule annonce, vide.
- **SC-003** : en production, plus de notification en boucle vers `sol_city_ai`, le journal de
  `ea69b65b` cesse de grossir.
