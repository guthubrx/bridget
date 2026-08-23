# ADR 003 — Maicie est un compagnon d'orchestration hors processus

- **Statut** : Accepté ; MVP implémenté, activation de profils en finition
- **Date** : 2026-08-22
- **Mise à jour** : 2026-08-23

## Contexte

Bridget prouve en usage réel l'intérêt de la communication directe entre
agents : présence, messages, demandes suivies, réponses et reprise restent
fluides. L'ancien Maicie centralisait au contraire une mission dans un
superviseur, un pipeline et des états de tâches qui enfermaient l'utilisateur
dans une exécution prédéfinie.

Le besoin est de coordonner volontairement un objectif sans transformer
Bridget en moteur de workflow ni attribuer à ACP un statut métier universel.

## Décision

Maicie v3 est un **compagnon CLI hors processus** distribué dans
`plugins/maicie/`, avec sa propre SQLite. Chaque invocation charge une
configuration explicite, exécute une commande puis rend la main. Les voies qui
contactent Bridget réconcilient auparavant leurs outboxes ; les actions
strictement locales n'ouvrent aucune connexion. Maicie n'est ni chargée par le
daemon Bridget, ni un service résident obligatoire.

Elle utilise exclusivement les interfaces locales publiques de Bridget pour :

- lire l'annuaire et la disponibilité ;
- envoyer une délégation idempotente avec un identifiant client durable ;
- observer les faits ACP par l'abonnement public session 008 ;
- demander une activation par le `SpawnOrder` public session 009.

Bridget conserve la vérité du transport : connexion, présence, livraison,
demande suivie, timeout et snapshot. Maicie conserve la vérité de coordination :
objectif, participants, délégations, décisions et outboxes. Il n'existe ni base
SQLite partagée, ni import de modules internes Bridget, ni accès de Maicie à
`bridget.db`.

Une conversation reste libre. Une intention ne devient un objectif coordonné
que par une commande explicite. Les messages directs ne modifient jamais un
objectif Maicie par effet de bord.

## Durabilité et réconciliation

### Délégations

La création d'une délégation et de son enveloppe filaire complète partage une
transaction SQLite. Le `message_id`, la cible, le corps, le délai et l'horizon
existent donc avant toute I/O Bridget.

Au redémarrage, toute ligne non terminale commence par un lookup de l'issue
durable. En l'absence d'issue et sous l'horizon négocié, Maicie rejoue les
octets exacts avec le même identifiant. `IdempotencyExpired` devient un refus
terminal et n'autorise jamais un nouvel envoi implicite.

### Activations de profils

L'approbation est mono-usage, expirante et scellée avec
`actor=local_human`. Elle représente une frappe locale dans le modèle de
confiance mono-utilisateur ; aucune commande Bridget ou MCP ne peut la produire.
La transaction d'approbation persiste simultanément
`ActivationOutbox(command_id, spawn_order_bytes)`.

Le contrat 009 ne comporte pas de `SpawnLookup`. Le replay exact du
`SpawnOrder`, avec le même `command_id`, est le lookup idempotent et retourne
l'issue durable. Après acceptation, Maicie compare le digest de définition
résolue de `SpawnAccepted` au hash épinglé dans l'approbation. Elle ne relit
jamais le registre courant pour calculer ou confirmer cette preuve.

## Arrêt et responsabilité des processus

Maicie ne possède aucun processus enfant, groupe de processus, timer actif ou
boucle de relance. Interrompre une commande ne détruit pas les transactions
déjà commitées ; elles seront réconciliées au prochain appel. L'arrêt, la
reprise et la persistance des équipiers appartiennent exclusivement au daemon
Bridget.

## Frontière d'état : deux vérités explicites

Une remise locale durable exprime ce que Maicie a préparé et appris d'une
issue. Un snapshot de transport exprime ce que Bridget ou l'abonnement ACP a
observé, avec source, séquence et fraîcheur. Aucun des deux ne remplace l'autre.

En particulier, un snapshot absent, périmé, `Gap`, `End` ou `unavailable` ne
ferme pas un objectif, ne transforme pas une délégation en succès ou échec et
ne réécrit pas une outbox. La progression métier exige une issue durable ou
une décision explicite.

## Limites

- aucune sélection par LLM, interprétation sémantique ou échelle implicite de
  compétence ; la cible est explicite ou issue d'une égalité stricte de tags ;
- aucun DAG, scheduler, cron, GUI ou TUI dans ce périmètre ;
- aucun lancement, arrêt ou redémarrage d'agent par Maicie ;
- les permissions ACP sont affichées comme décisions automatiques déjà prises
  par Bridget, jamais comme demandes humaines en attente ;
- modèle local coopératif mono-utilisateur, sans frontière d'autorisation
  hostile entre processus du même compte.

## Conséquences

### Positives

- Bridget reste un transport petit, réutilisable et multifournisseur.
- Maicie peut évoluer ou être remplacée sans migrer l'état de livraison
  Bridget.
- Les crashs entre commit et accusé ne créent ni délégation ni activation en
  double.
- Un futur affichage peut consommer les mêmes sorties CLI/JSON sans devenir
  source de vérité.

### Coûts acceptés

- Deux propriétaires imposent un contrat public et une corrélation explicite.
- La reprise est une saga idempotente, pas une transaction distribuée.
- L'absence de déduction sémantique demande des décisions utilisateur
  explicites là où un orchestrateur opaque aurait deviné.

## Alternatives écartées

- **Réintégrer Maicie dans le daemon Bridget** : couplage fort, cycle de
  publication commun et transport alourdi.
- **Reprendre le superviseur historique** : pipeline rigide, cycle de
  worktrees et fournisseur imposés.
- **Partager `bridget.db`** : double autorité et migrations couplées.
- **Adopter DSH ou T3 Code comme interface canonique** : modèle de sessions et
  source d'état supplémentaires.
