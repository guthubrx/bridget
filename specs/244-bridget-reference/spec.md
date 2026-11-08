# Session 244 — Bridget de référence : déléguer sans solliciter l'utilisateur

Statut : corrections prioritaires et exécutables livrés en0.8.10/R5 ; qualification étendue partielle, minuteur non activé. Ouverte le 8 novembre 2026, à la demande de François.

## Besoin

François ne doit plus être sollicité pour des refus de Bridget. Les délégations
doivent fonctionner pour les anciens projets, les projets actuels et les
nouveaux projets, sur toutes les machines. Elles doivent aussi fonctionner sans
T3. Un refus restant doit être compris et traité par l'agent qui le reçoit.

## Constat à l'origine (session 243)

Quatre défauts indépendants ont bloqué les délégations pendant une semaine :
une qualification Codex qui plantait sans message, deux anciens mappings
incomplets, une première vue de projet neuf refusée, des messages de refus
faux ou vagues. Aucun test ne refaisait le parcours réel d'un agent.

## Exigences

Les quatre axes se renforcent : la sentinelle (E1) découvre les écarts, la
réparation (E2) corrige les écarts connus, les erreurs gardées (E3) donnent la
cause des autres, et la règle des refus (E4) rend chaque refus actionnable.

### E1 — Sentinelle : le parcours réel, chaque jour et après chaque changement

- E1.1 Une mission minimale réelle part vers chaque machine active, au moins une
  fois par jour, après chaque activation de Bridget et après chaque nouvelle
  version d'un programme d'agent détectée.
- E1.2 Le parent de cette mission est une session d'agent gérée par Bridget,
  sans T3. Elle emprunte le même parcours d'admission, de préparation de projet,
  de projection des droits, d'exécution et de remise qu'un agent réel.
- E1.3 La sentinelle couvre au moins : un projet existant en worktree, un
  projet existant en vue de commit, un projet neuf créé pour l'occasion, chaque
  moteur déclaré (Claude, Codex) sur chaque machine où il est actif.
- E1.4 Coût borné : mission en lecture, modèle le plus léger, réponse fixe,
  aucune mission si la machine est saturée (consigné, pas une panne).
- E1.5 Résultat durable et lisible : dernier passage, par machine et moteur,
  succès ou code de refus exact, horodatage. Lisible par CLI et par l'interface
  mobile.

### E2 — Réparation automatique des écarts connus

- E2.1 Un mapping existant auquel manque une racine additionnelle déjà approuvée
  pour la même source est complété automatiquement, sur toutes ses copies,
  sous verrou, avec sauvegarde et révision incrémentée.
- E2.2 Une nouvelle version d'un programme d'agent dont la qualification échoue
  est requalifiée selon la cadence existante ; l'échec persistant porte sa cause.
- E2.3 Une réparation n'élargit jamais un droit : elle reproduit une autorisation
  déjà accordée, ou elle refuse et signale.

### E3 — Aucune erreur muette

- E3.1 Tout programme lancé par Bridget pour un contrôle (qualification,
  sonde, préparation) garde une sortie d'erreur bornée, sans secret, rattachée
  à la cause publiée.
- E3.2 Un contrôle qui ne peut pas aboutir ne reste jamais « en attente » sans
  limite : après un délai borné, il devient un échec nommé.

### E4 — Règle des refus

- E4.1 Chaque code de refus émis a un message qui dit, en une phrase, ce que
  l'agent doit faire, et si une décision humaine est nécessaire.
- E4.2 Un test vérifie que tout code de refus de la liste fermée a ce guidage.
- E4.3 La sentinelle signale tout refus sans guidage comme un défaut.

### E5 — Alerte seulement si nécessaire

- E5.1 Si la sentinelle échoue et que la réparation ne suffit pas, le mainteneur
  Bridget est prévenu par Bridget, une fois par incident, avec la cause.
- E5.2 François n'est prévenu que si une décision humaine est nécessaire.

### E6 — Aucun recours à T3 pour rattraper Bridget

Constat de François : depuis une semaine, quand Bridget était en panne, il s'en
sortait grâce aux fils T3 entre agents. Sans T3, ces pannes seraient bloquantes.

- E6.1 La sentinelle couvre aussi la communication entre agents : message
  direct, réponse attendue, fil partagé, notification de résultat, entre
  machines différentes, avec des agents lancés par Bridget sans T3.
- E6.2 Une panne d'un composant Bridget (daemon, worker, pont, lien entre
  machines) est détectée et réparée automatiquement par Bridget : redémarrage
  natif, reprise du lien, reprise des messages en attente, sans perte ni doublon.
- E6.3 Pendant une panne, un agent reçoit un statut clair (quoi, depuis quand,
  quand réessayer) au lieu d'un échec opaque ; ses messages restent en file et
  partent au retour du service.
- E6.4 Chaque correctif livré est vérifié par la sentinelle sans T3 avant
  d'annoncer la livraison.

## Critères de réussite

- Sur une semaine, aucune sollicitation de François due à un refus de Bridget.
- Sur une semaine, aucun besoin de passer par T3 pour débloquer des agents.
- Chaque régression de parcours est vue par la sentinelle avant un agent.
- Les quatre défauts de la session 243, réintroduits en test, sont détectés :
  par la sentinelle (E1), réparés (E2) ou nommés (E3, E4).

## Hors périmètre

- Supprimer les protections : refus de capacité sur machine imposée (décision
  de François du 8 novembre 2026), droits, gardes de dossier.
- Remplacer T3 : la sentinelle prouve seulement que Bridget n'en dépend pas.
