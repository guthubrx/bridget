# Recherche — Maicie v3

**Date** : 2026-08-22  
**Portée** : choix de conception uniquement ; aucune dépendance ni code ajoutés.

## R-001 — Deux propriétaires d'état, aucun partage SQLite

**Décision** : Bridget possède livraison, présence, timeout et demande suivie ;
Maicie possède objectif, délégation, profil et décision. Chaque couche a sa
SQLite. Maicie n'importe aucun crate interne et ne lit jamais `bridget.db`.

**Rationale** : une base commune ou le superviseur historique recréerait des
sources de vérité concurrentes. Le contrat local versionné est le seul couplage
accepté.

## R-002 — Outbox durable avec identifiant client idempotent

**Décision** : Maicie génère `message_id`, écrit la délégation et son outbox
`prepared` avant toute I/O, puis appelle Bridget avec ce même id. Ack perdu ou
coupure devient `outcome_unknown` ; la reprise consulte Bridget ou réessaie le
même id, que Bridget déduplique.

**Rationale** : persister seulement un identifiant retourné par Bridget ouvre
une fenêtre de crash après acceptation et avant persistance, donc un doublon.
Cette table supplémentaire est justifiée par une garantie vérifiable.

## R-003 — Conversation libre, MVP déterministe

**Décision** : un message ne crée un objectif qu'après intention explicite ou
confirmation. La sélection est une cible explicite ou l'unique égalité de tags;
sinon Maicie propose des candidats. `summarize` agrège factuellement les
réponses corrélées. Aucun LLM, interprétation de texte libre ou heuristique de
compétence dans le MVP.

**Rationale** : cela conserve la fluidité du dogfooding sans réintroduire le
pipeline et les décisions opaques de Maicie historique.

## R-004 — ACP 008 : snapshots factuels, pas état métier

**Décision** : Maicie consomme seulement l'abonnement public session 008, avec
`subscription_id`, `seq`, `Gap`, `End` et fraîcheur. Les snapshots sont dérivés
et ne produisent jamais seuls une transition de coordination. Une permission
ACP est montrée avec la décision automatique déjà émise par le registre 007,
jamais comme attente humaine.

**Rationale** : ACP n'impose pas de vocabulaire `bloqué` ou `result_ready`.
La session 007 journalise l'activité, mais la session 008 est le contrat public
fiable requis pour ne pas lire de fichier ou socket interne.

**Source** : [ACP Overview](https://github.com/agentclientprotocol/agent-client-protocol/blob/main/docs/protocol/v2/overview.mdx), consulté le 2026-08-22.

## R-005 — Bridget est l'unique horloge active

**Décision** : `courte`/`normale`/`longue` sont traduites vers le timeout
Bridget. Maicie les affiche mais n'a ni timer durable, cron, relance, borne ou
décision automatique à échéance.

**Rationale** : une seconde horloge ferait revenir un scheduler caché, en
concurrence avec le cycle de demande Bridget.

## R-006 — Profils approuvés, SpawnOrder 009 seulement

**Décision** : un profil est une identité durable à tags déclarés. Maicie crée
une approbation locale mono-usage liée à objectif, profil, hash/version, hash et
périmètre de contexte, paramètres, acteur `local_human` et expiration. Après
revalidation atomique, elle émet uniquement le contrat public SpawnOrder de
session 009. Aucun `Child`, `Command::spawn`, arrêt ou reconnexion dans Maicie.

**Rationale** : un spawn local dans Maicie lui ferait récupérer le superviseur
historique par morceaux et introduirait une faille TOCTOU.

## R-007 — DSH et T3 Code ne sont pas le noyau

**Décision** : DSH et T3 Code restent des références. Aucune dépendance,
interface ou source d'état externe n'est ajoutée. GUI et TUI sont des specs
ultérieures consommatrices du contrat CLI/JSON.

**Sources** : [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) et [T3 Code](https://github.com/pingdotgg/t3code), consultés le 2026-08-22.

## R-008 — ActivationOutbox est une seconde saga, non une transaction distribuée

**Décision** : l'approbation passe à `dispatching` dans la même transaction que
la persistance de `ActivationOutbox(command_id, spawn_order_bytes)`. Après
crash, lookup/replay du même command_id précède toute émission ; l'approbation
ne devient consommée qu'après issue durable. Le contexte envoyé après Connected
utilise l'OutboxDélégation existante et les bytes/hash approuvés.

**Rationale** : consommer avant SpawnOrder perdrait une activation au crash ;
envoyer avant consommer permettrait un double lancement. Une transaction
distribuée fictive ne résout aucun de ces cas.

## Risques couverts

| Risque | Réponse |
|---|---|
| doublon après crash | outbox préalable, même id, déduplication et test Ack perdu |
| carcan | aucun DAG, scheduler, LLM ou tâche implicite |
| ACP surinterprété | Subscribe 008, séquence/fraîcheur/Gap, aucun statut métier |
| permission erronée | historique auto-décidé, pas d'attente humaine fictive |
| superviseur caché | SpawnOrder 009 et approbation TOCTOU, zéro spawn Maicie |
| activation perdue/doublée | ActivationOutbox command_id, lookup/replay et crash barriers |
