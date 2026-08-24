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

## Mise à jour 015 — guichet durable Bridget

La session 015 ajoute un guichet Maicie sans changer le modèle compagnon. Le
destinataire de service `maicie` devient joignable dans Bridget même lorsque le
binaire Maicie n'est pas lancé, mais cette joignabilité reste une boîte aux
lettres de transport : Bridget ne possède ni objectif, ni délégation, ni
décision Maicie.

La capacité `maicie_guichet` est la borne d'autorisation du guichet. Elle
contrôle la relève `GuichetClaimNext`/`GuichetClaim`, les réponses
`GuichetReply` et les `RequestLifecycleEvent`. Un nom déclaré, y compris
`from: "maicie"`, n'accorde jamais ces droits. Ce choix applique la limite C5 :
dans le modèle local coopératif v1, l'identité reste déclarative et non
opposable à un processus hostile du même compte. La capacité ferme la surface
protocolaire sans prétendre fournir une authentification cryptographique.

Le guichet est relevé en pull-only à l'ouverture d'une commande Maicie. Bridget
retourne au plus une demande relivable par `GuichetClaimNext`, dans l'ordre
FIFO durable, puis Maicie traite sous budget absolu et rend la main. Il n'y a
ni polling, ni worker caché, ni boucle résidente dans cette décision. Une
boucle `maicie serve` visible est explicitement une évolution v2, à spécifier
avec son arrêt, sa présence et ses règles d'exploitation.

La matrice v1 est fermée : `delivery_report`, `mission_status` et
`deadline_question`. Les refus sont explicites et sans mutation, notamment pour
texte libre, opération inconnue, champ inconnu, cible non réservée, capacité
absente, divergence canonique, claim périmé, terminal déjà atteint et
expiration d'idempotence. Aucune route guichet, Bridget ou MCP ne peut proposer
ou consommer une approbation distante ; l'approbation de profil reste une
frappe TTY humaine locale.

Un dépôt est identifié de façon durable par
`(issuer_scope, service_request, request_id)` et ses octets canoniques. Le
rejeu conserve cette enveloppe et l'`issued_at` d'origine ; une divergence est
refusée, et `idempotency_expired` est terminal. La relève FIFO durable délivre
au plus une demande avec un propriétaire, un token, une génération et un bail.
`GuichetReply` et les événements corrélés sont conditionnés au détenteur
courant : une ancienne génération ne peut pas finaliser la demande.

Le gate G1504 a éprouvé la chaîne livrée avec un wrapper ACP réel : dépôt alors
que Maicie est absente, relève pull-only, greffe unique dans SQLite, réponse
corrélée, demande Bridget `answered`, événement durable relevé, puis rejeu
sans doublon. Le parcours a mesuré 925 ms dans le commit `69ad00d`. Cette
preuve ne transforme pas le guichet en runtime : elle confirme que l'identité
de service durable et le compagnon CLI restent découplés.

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

La session 009 réserve toutefois `SpawnOrder` et `CancelRequest` au rôle
wrapper ; le rôle client négocié ne les admet pas. Maicie ouvre donc une
connexion fraîche déclarée comme wrapper pour chaque émission ou reprise
d'activation. Cette compatibilité ne constitue pas une autorisation : dans le
modèle local coopératif v1, tout processus du même compte capable d'ouvrir ce
rôle peut émettre le même ordre. L'approbation locale Maicie est une preuve de
gouvernance durable dans Maicie, mais elle n'est pas opposable côté Bridget.
Cette limite est assumée en v1 et n'est pas masquée par le digest.

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
- aucun daemon Maicie résident ; la boucle `maicie serve` est une évolution v2
  séparée, pas un effet secondaire de `status` ou de la relève guichet ;
- aucun lancement, arrêt ou redémarrage d'agent par Maicie ;
- les permissions ACP sont affichées comme décisions automatiques déjà prises
  par Bridget, jamais comme demandes humaines en attente ;
- l'activation ne dispose pas encore d'un budget global unique couvrant toute
  la passe de lookup, connexion, replay et accusé ; les bornes locales peuvent
  donc se cumuler, contrairement à la voie de délégation ;
- modèle local coopératif mono-utilisateur, sans frontière d'autorisation
  hostile entre processus du même compte.
- capacité `maicie_guichet` bornant le guichet, sans droit implicite par nom
  déclaré et sans identité cryptographique opposable en v1.

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
