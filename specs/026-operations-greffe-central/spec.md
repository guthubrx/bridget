# Session 026 — Opérations fédérées du greffe central

**Branche** : `session-026-operations-greffe-central`

**Base fonctionnelle** : session 021, tête `2623772`

**Dépendance de schéma** : v17 (021) → v18 (refus local rc1) → v19 (026)
**Statut** : seconde tranche urgente — pilotage fédéré du greffe central

## Pourquoi cette tranche existe

Le guichet est l'unique porte de vérité entre les agents fédérés et le greffe
Maicie central. Ajouter un accès direct MCP ou ouvrir une SQLite chez l'appelant
créerait une seconde vérité.

Le chemin local refusait déjà un but qui cite un objectif connu sans classer
cette citation. Le chemin fédéré ne pouvait pas même déposer `delegate`. Cette
tranche admet donc l'enveloppe, applique la même règle déterministe et produit
un refus terminal durable avant toute mutation métier.

Elle ne prétend pas encore déléguer à distance. Une demande cohérente reçoit
`operation_not_available`, elle aussi persistée : un dépôt n'est jamais vendu
comme un effet applicatif.

La seconde tranche conserve cette porte unique et rend applicatives trois
mutations strictement bornées : `delegate`, `registre_add` et
`objective_close`. Leur consultation terminale est un quatrième verbe séparé :
un dépôt `queued` ou un transport interrompu n'est jamais présenté comme un
effet métier réussi.

## Propriétés livrées

### P2601 — Une seule règle, deux appelants

Le traitement guichet appelle
`citation::unclassified_known_citations`. Il ne recopie ni la grammaire UUID,
ni une heuristique de prose, ni le rapprochement avec une branche ou un SHA.

La contradiction bloquante est exactement :

- `suite=aucune` ;
- le `goal` cite l'UUID canonique d'un objectif présent dans le store central ;
- cet UUID est absent de `depends_on` et `references`.

Le motif exact est `suite_none_with_unclassified_citation`.

### P2602 — Limite déclarée, sans faux positif

Une branche ou un SHA seuls ne sont pas des UUID d'objectif connus. Ils ne
déclenchent donc pas le motif de contradiction. Cette limite couvre
volontairement moins de cas que la prose observée, car un refus injuste serait
plus dangereux qu'un signal manquant.

Le témoin de limite exige alors le motif exact `operation_not_available`. Il
n'existe aucun LLM, score lexical ou liste parallèle de mots causaux.

### P2603 — Refus durable avant réponse

Le refus est écrit dans `guichet_refusal_receptions` avant que ses octets soient
rendus. Un rejeu du même claim relit exactement les mêmes octets et ne crée pas
une seconde ligne. Aucune table d'objectif, de délégation ou d'outbox n'est
mutée par cette tranche.

`queued`, `in_flight` et `outcome_unknown` attestent seulement un dépôt. Seul
le reçu terminal `refused` est conclusif.

### P2604 — Vocabulaire fermé dans le code, refus conservable en SQL

`ServiceRequestOperation`, `OperationGuichet` et `MotifRefusGreffe` restent des
enums fermés. Une source Rust unique engendre `ALL`, `as_sql()` et le parseur
exact pour les deux vocabulaires Maicie.

La migration v19 reconstruit seulement `guichet_refusal_receptions` et retire
ses deux listes `CHECK(operation IN ...)` et `CHECK(reason IN ...)`. Elle garde
les contraintes structurelles. Cela permet de conserver le nom d'une tentative
refusée sans en faire une variante autorisée. Une chaîne inconnue injectée en
SQL échoue `StoreError::Corrupt` à sa première lecture Rust.

### P2605 — Aucun saut de migration

v19 ne s'applique qu'après la vraie v18 de rc1. Son préflight transactionnel :

- crée sous savepoint un objectif et un refus local sentinelles ;
- prouve que l'insertion v18 fonctionne ;
- prouve que les triggers refusent `UPDATE` et `DELETE` ;
- exécute `ROLLBACK TO` puis `RELEASE` sur le chemin succès comme échec ;
- compare les nombres de lignes avant et après.

Une base seulement estampillée v18, sans ce DDL, est refusée sans mutation. Une
base estampillée v19 qui porte encore le CHECK v18 est également refusée.

### P2606 — Même service métier, quel que soit l'appelant

Les commandes locales et les claims fédérés appellent le même service Maicie
pour déléguer, ajouter au registre et clore un objectif. Ce service ouvre la
configuration centrale, la même base SQLite et le même journal déclarés par le
greffe ; aucun chemin, URI de base ou journal n'est accepté depuis la charge.

Les gardes, l'idempotence et les transactions restent celles des chemins
locaux. Le guichet ne recopie pas la sélection des candidats, le parseur du
journal ni la clôture d'objectif.

### P2607 — Quatre verbes, résultats terminaux seulement

Le client MCP expose exactement `maicie_delegate`, `maicie_registre_add`,
`maicie_objective_close` et `maicie_request_status`. Les trois premiers
déposent une demande canonique et rendent seulement `queued` ou
`outcome_unknown` tant que le maître n'a pas persisté de reçu terminal.

`maicie_request_status` relit ce reçu auprès du daemon maître. Seul ce résultat
terminal peut annoncer `created`, `appended`, `closed`, `selection_required`
ou `refused`, avec les identifiants durables correspondants.

### P2608 — Identité de connexion centralisée, mais encore déclarative

Le serveur MCP enregistre auprès du daemon le nom et l'instance déjà résolus au
démarrage. Le daemon conserve cette identité avec la connexion ; les charges de
mutation ne répètent aucun jeton ni principal. Déplacer le couple de chaque
requête vers un `Register` unique réduit les sources d'identité, mais ne
l'authentifie pas.

Le point d'appel du service reçoit un principal injecté par le daemon. Un champ
`from` filaire reste une déclaration à comparer, jamais une source d'autorité.
Une politique centrale fermée borne ensuite ce principal aux actions
`delegate`, `registre_add` et `objective_close`, avec refus par défaut.

Cette politique suppose toutefois un appelant non hostile sous le même compte.
`crates/bridget-transport/src/protocol.rs:854-882` reçoit aujourd'hui `name` et
`instance_id` du client dans `Register` ;
`crates/bridget-daemon/src/daemon.rs:6206-6258` transmet ces valeurs, puis
`crates/bridget-daemon/src/daemon.rs:3443-3465` et
`crates/bridget-daemon/src/daemon.rs:3630-3632` les écrivent sans vérifier la
filiation du processus pair. La preuve de filiation de
`crates/bridget-daemon/src/mcp_identity.rs:143-186` est exécutée côté processus
MCP et redevient donc une déclaration sur le fil.

Sur le déploiement mesuré le 27 août 2026, le répertoire `agent-pids` est en
`0770` et ses marqueurs en `0660`, tous sous le même compte `moi:moi` : aucune
frontière de privilège ne sépare les agents locaux. Un client local parlant le
protocole brut peut donc se déclarer sous un autre nom. Cette borne cessera
d'être acceptable dès que des comptes différents partageront le daemon ;
l'identité devra alors être établie côté serveur, notamment en tenant compte des
agents distants dont le processus pair visible est celui du tunnel.

### P2609 — Compatibilité et frontière humaine inchangées

Les extensions filaires sont additives et gardent les anciennes trames
décodables. `profile_approve`, `routine_approve` et toute commande arbitraire
restent absentes des outils et des opérations autorisées : ces actions restent
réservées à la frappe humaine.

## Scénarios d'acceptation

1. Un objectif central existe. Un claim `delegate`, `suite=aucune`, cite son
   UUID dans `goal` sans relation structurée : reçu terminal exact, une ligne
   durable, puis rejeu octet-identique.
2. Le même texte ne contient qu'un nom de branche et un SHA : aucun motif de
   contradiction n'est inventé ; le reçu exact dit que l'opération applicative
   n'est pas encore disponible.
3. Une valeur d'opération inconnue est injectée directement dans la table
   privée : la première lecture échoue fermée.
4. Une vraie v18 migre vers v19 en conservant les octets historiques ; une
   fausse v18 et une fausse v19 ne modifient ni schéma ni numéro.
5. Une copie privée v14 emprunte réellement v17, v18 puis v19 ; sa source reste
   octet-identique. Le bootstrap vide est exercé séparément.
6. Un MCP enregistré sous son identité résolue dépose chacune des trois
   mutations ; aucun premier retour ne prétend que l'effet est appliqué.
7. La relève Maicie applique chaque mutation par le même service que le CLI,
   persiste son reçu terminal, puis le statut rend l'issue et ses identifiants.
8. Un dépôt interrompu avant résultat reste `outcome_unknown`; sa consultation
   ultérieure relit le reçu du maître sans ouvrir de base sur l'appelant.
9. Les charges ne peuvent fournir ni chemin de base, ni chemin de journal, ni
   principal d'autorisation.
10. Les témoins d'autorisation déclarent explicitement qu'ils prouvent le
    traitement du principal enregistré, pas l'authenticité de `Register` face à
    un client local capable de parler le protocole brut.

## Frontière de sécurité inchangée

`profile_approve` et `routine_approve` restent absentes de
`ServiceRequestOperation`. L'approbation est une frappe humaine protégée par
l'ADR 011.

Leur transformation en refus fédérés durables portant le nom tenté appartient
à la seconde tranche. Les deux témoins existent mais sont explicitement
ignorés jusque-là ; cette première livraison ne présente pas cette propriété
comme acquise.

## Travail explicitement reporté

- persister les tentatives `profile_approve` et `routine_approve` avant refus ;
- établir côté daemon l'identité de `Register` contre la filiation réelle du
  processus pair, y compris derrière un tunnel distant ;
- interdire à `Rename` de cibler la connexion d'un autre appelant ;
- exposer toute autre commande Maicie.

Ces éléments ne doivent jamais ouvrir une base locale ni créer une seconde
porte d'écriture.

## Hors périmètre

- interpréter les branches, SHA ou la prose par heuristique ;
- modifier la règle locale portée par rc1 ;
- reprendre le variant composé de refus livré par 021 ;
- exposer l'approbation de profils ou routines ;
- toucher la base Maicie active pour un test de migration.
