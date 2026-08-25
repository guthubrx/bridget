# Maicie v3

Maicie est un compagnon CLI de coordination construit au-dessus des contrats
publics de Bridget. Elle transforme une instruction explicite en objectif et en
délégation durables, sans devenir un moteur de workflow ni un superviseur de
processus.

## Démarrer

Compiler Bridget et Maicie depuis la racine du dépôt :

```bash
cargo build --manifest-path /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/Cargo.toml --release -p bridget-daemon -p maicie
```

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget daemon
```

Dans un autre terminal, créer un fichier de configuration. Tous les chemins
doivent être absolus et la base Maicie doit rester distincte de `bridget.db`.
Les exemples suivants supposent que ce fichier est enregistré dans
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.json` :

```json
{
  "version": 1,
  "bridget_socket": "/Users/moi/.cache/bridget/bridget.sock",
  "database_path": "/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.sqlite3",
  "durations": {
    "short_secs": 30,
    "normal_secs": 300,
    "long_secs": 3600
  },
  "status_capture_budget_ms": 250,
  "profiles": [{
    "id": "reviewer",
    "agent_name": "reviewer",
    "agent_type": "codex",
    "model": "modele-epingle",
    "effort": "high",
    "display_name": "Relecture",
    "tags": ["review"],
    "personality_ref": "/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/profiles/reviewer.md",
    "tools": ["bridget_send"],
    "spawn_order_ref": "/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/agents/reviewer"
  }]
}
```

Les commandes sont des invocations courtes. Chacune charge la configuration et
ouvre la SQLite privée. Les commandes qui contactent Bridget réconcilient les
outboxes non terminales avant leur action ; les commandes d'objectif restent
strictement locales. Toutes rendent ensuite la main :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/maicie delegate \
  --config /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.json \
  --goal "Relire le risque de reprise" \
  --to reviewer \
  --duration normale \
  --idempotency-key delegation-review-1 \
  --json

/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/maicie status --config /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.json --json
```

Réutiliser la même clé d'idempotence avec le même contenu rejoue le même
résultat. Réutiliser cette clé avec un contenu différent est refusé.

## Migration de schéma — consentement explicite

L'ouverture d'une base Maicie **ne migre plus** un schéma déjà versionné.
Créer une base neuve reste autorisé sans flag : `user_version = 0` **et**
aucun objet utilisateur dans `sqlite_master` (créer n'est pas migrer). Une
base peuplée dont on remet `user_version` à 0 est refusée comme une
migration — ce n'est pas une base neuve. Un schéma trop récent pour le
binaire reste refusé (fail-closed, inchangé).

Quand la base porte un schéma **antérieur** au binaire, l'ouverture refuse
avec un message parlant et **n'écrit rien** (`user_version` inchangé). Le
seul consentement documenté est :

```bash
maicie migrate --config /chemin/absolu/vers/maicie.json
```

Équivalent sur toute commande qui ouvre la base (flag global, avant ou après
le verbe) :

```bash
maicie --migrate status --config /chemin/absolu/vers/maicie.json
maicie plage list --migrate --config /chemin/absolu/vers/maicie.json
```

Codes de sortie : refus de migration = erreur `store` (exit 6), comme les
autres erreurs SQLite / schéma. Usage invalide (flag dupliqué, `migrate`
sans `--config`) = exit 2.

**Limite assumée** : ce consentement tue la migration *silencieuse* ; il
n'est pas une garde anti-production. Un binaire de branche invoqué avec
`--migrate` (ou `maicie migrate`) contre la config de production migre
encore. La règle sociale du chantier (« un binaire de branche ne touche
jamais la config/base de production ») reste la seule barrière ; une garde
technique (allowlist de chemins, variable d'environnement) est un chantier
séparé, hors de ce lot.

## Arrêt

Maicie ne maintient aucun service résident, timer ou processus enfant. Une
invocation peut être interrompue avec `Ctrl-C` ; les décisions et outboxes déjà
commitées restent dans sa SQLite et seront réconciliées à l'invocation suivante.
Le daemon et les équipiers restent sous la responsabilité de Bridget
(`bridget stop <nom>` pour un équipier géré).

Si Bridget utilise un registre utilisateur `agents.json`, ce fichier doit être
régulier, non symbolique et avoir le mode `0600` ou plus restrictif ; le daemon
refuse de démarrer avec un registre plus permissif.

## Routines (schéma v15)

Une routine est un gabarit de `delegate` + un calendrier évalué à chaque
relève (~60 s). Elle **délègue**, n'approuve jamais. Surface CLI :

| Action | Rôle |
|---|---|
| `routine propose` | Propose un gabarit (`--goal`, `--participant`, `--period-secs`, `--suite`, `--depends-on`, `--references`) ; état `proposed` |
| `routine approve` | Approbation locale ADR 011 : refus pré-écran si gabarit altéré, puis écran des six champs scellés + confirmation `oui` |
| `routine list` | Liste les routines et occurrences ouvertes / récentes |
| `routine show --id` | Détail d'une routine (`open_occurrence`, `recent_differee`, …) |
| `routine pause --id` | Suspend sans rattraper les buckets de pause à la reprise |
| `routine resume --id` | Reprend au bucket courant (pas de rattrapage de la pause) |

Schéma SQLite `user_version = 15` : tables `routines` / `routine_occurrences`.
La migration v15 n'est pas dans ce lot (`--migrate` ailleurs).
L'adoption d'un mandat orphelin exige une délégation **vivante**
(`state NOT IN (annulee, terminee)`) : un mandat terminal n'est jamais
ressuscité en `ouverte/mandat_adopte`. Une `ouverte` qui pointerait déjà
vers un cadavre est rétractée en `sautee/mandat_plus_vivant` en tête de
tick. **Choix** : sans attestation vivante, la routine redélègue même si
l'objectif précédent reste ouvert — le calendrier ne doit pas geler en
silence sur un mandat mort ; le traitement propre d'`Annulee` / jamais-
retour reste la dette hors lot.

### Dette assumée (hors lot) — formulation mesurée

Si une délégation est **annulée** sans clôture d'objectif, l'occurrence reste
`ouverte`, les relèves suivantes produisent des `differee`, **aucun nouveau
mandat ne part**, et `routine list` continue d'afficher `active`. En termes
mesurés : **la routine cesse définitivement de tourner, en silence, pendant
que la surface affirme qu'elle est vivante.** Remède futur (~1 j) : états
`Annulee` / jamais-retour. Banc de référence : mesure relec5 (dix relèves,
contrôle positif objectif clos).

## Guichet Maicie

La session 015 ajoute le contrat du guichet Maicie : Bridget peut tenir une
boîte aux lettres durable pour la cible de service `maicie`, sans annoncer un
processus Maicie vivant et sans recopier l'état métier dans `bridget.db`.
Maicie reste pull-only : au début d'une commande locale, elle relève sous budget
absolu les demandes non terminales, puis rend la main.

La capacité `maicie_guichet` est la borne de protocole des opérations sensibles
du guichet : relève `GuichetClaimNext`/`GuichetClaim`, réponse `GuichetReply`
et événements `RequestLifecycleEvent`. Un nom déclaré, y compris
`from: "maicie"`, n'ouvre jamais ces droits. C'est une limite coopérative v1,
pas une identité opposable : un processus hostile du même compte local n'est
pas authentifié cryptographiquement par cette capacité.

Le modèle de relève est FIFO et borné. `GuichetClaimNext` sélectionne au plus
une demande relivable selon `deposited_sequence ASC`; le client répète cette
relève seulement tant que son échéance globale le permet. Il n'existe pas de
polling, de worker caché ou de boucle résidente dans cette version. Une boucle
`maicie serve` visible, avec arrêt propre, est une évolution v2 à spécifier
séparément.

Les seules opérations admises par le contrat v1 sont `delivery_report`,
`mission_status` et `deadline_question`. Les refus sont fermes :
`service_role_required`, `capability_required`, `reserved_target_required`,
`declared_sender_mismatch`, `unsupported_version`, `frame_too_large`,
`invalid_envelope`, `canonical_bytes_mismatch`, `request_already_terminal`,
`idempotency_expired`, `claim_stale` et `transition_invalid`. Une opération
d'approbation, un texte libre, un champ inconnu ou une enveloppe divergente est
refusé sans créer d'objectif, de délégation, d'approbation ou de `SpawnOrder`.

Le dépôt producteur est idempotent sur
`(issuer_scope, service_request, request_id)`. Il faut donc rejouer les mêmes
octets, le même `issued_at`, le même `issuer_scope` et le même `request_id` ;
une divergence canonique est refusée et `idempotency_expired` est terminal. La
relève est atomique et FIFO : un claim retourne un propriétaire, un token, une
génération et un bail. Seul ce quatuor encore courant peut produire
`GuichetReply`; un détenteur devenu périmé reçoit `claim_stale` sans mutation.

Le scénario réel G1504 a validé la chaîne complète en 925 ms (commit
`69ad00d`) : wrapper ACP réel, dépôt alors que Maicie est absente, relève
pull-only par une commande, greffe SQLite unique, réponse corrélée, demande
Bridget `answered`, événement durable relevé et retry sans doublon. Pour les
commandes producteur et le harnais reproductible, voir
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/specs/015-guichet-maicie/quickstart.md`.

## Deux autorités, jamais une vérité fusionnée

| Autorité | Ce qu'elle possède |
|---|---|
| SQLite Maicie | objectifs, délégations, décisions et remises locales durables |
| Bridget | présence, livraison, demandes suivies et snapshot de transport |

Une outbox Maicie décrit ce qu'elle a durablement préparé ou appris d'une issue.
Un snapshot Bridget décrit ce que le transport a observé, avec sa source et sa
fraîcheur. Une coupure, un `Gap` ou un `End` rend le transport incomplet ; cela
ne clôt jamais un objectif et ne réécrit pas l'état métier.

## Outboxes transactionnelles et reprise

La délégation et son enveloppe filaire complète sont écrites dans la même
transaction avant toute I/O. Après une coupure, une ligne `prepared` ou
`outcome_unknown` commence par consulter l'issue durable de son `message_id`.
Sans issue et sous l'horizon contractuel, Maicie rejoue les octets persistés
avec le même identifiant. `IdempotencyExpired` est terminal : aucune nouvelle
émission implicite n'est créée.

L'activation d'un profil suit la même discipline avec
`ActivationOutbox(command_id, spawn_order_bytes)`. Le protocole 009 n'expose
pas de `SpawnLookup` séparé : rejouer exactement le `SpawnOrder` avec le même
`command_id` constitue le lookup idempotent. Après acceptation, Maicie compare
le digest de définition renvoyé par `SpawnAccepted` au hash épinglé dans
l'approbation. Elle ne relit jamais `agents.json` pour refaire cette preuve.

Limite v1 : le protocole 009 n'admet `SpawnOrder` et `CancelRequest` que sur
une connexion déclarée avec le rôle wrapper, pas sur le rôle client négocié.
Maicie utilise donc une connexion wrapper fraîche pour ces ordres. Ce rôle
n'est pas une frontière d'autorisation : un autre processus local peut émettre
le même `SpawnOrder`. L'approbation humaine durable prouve la décision dans
Maicie, mais Bridget ne sait pas encore l'imposer. Une future capacité client
`spawn`, négociée sur une session Bridget dédiée, devra déplacer cette garde
du côté transport.

## Limites assumées

- comportement déterministe, sans LLM, interprétation libre ni sélection
  sémantique cachée ;
- sélection explicite ou égalité stricte de tags déclarés ;
- aucun `Child`, `Command::spawn`, arrêt ou redémarrage de processus dans
  Maicie : tout lancement passe par `SpawnOrder` Bridget ;
- `approve` est une frappe humaine locale exclusivement, dans le modèle de
  confiance mono-utilisateur ; cette capacité n'est exposée ni à Bridget ni à
  MCP ;
- les permissions ACP sont des faits déjà auto-décidés par Bridget, jamais une
  attente d'approbation humaine dans Maicie ;
- la passe d'activation n'a pas encore de budget global partagé entre lookup,
  connexion, replay et accusé ; ses délais locaux peuvent se cumuler ;
- aucune lecture de `bridget.db`, du journal ACP ou d'un fichier interne
  Bridget : seul le protocole public est consommé.
- le guichet 015 utilise `maicie_guichet` comme borne de capacité, mais cette
  borne reste coopérative v1 et non opposable entre processus locaux hostiles ;
- `bridget guichet deposer` expose les trois dépôts fermés aux wrappers
  enregistrés ; la relève reste volontairement interne à l'ouverture bornée
  d'une commande Maicie, sans CLI de claim séparé ;
- la boucle résidente `maicie serve` est explicitement v2.

Les scénarios de validation sont décrits dans
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/specs/011-maicie-orchestration/quickstart.md`
et, pour le guichet 015, dans
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/specs/015-guichet-maicie/quickstart.md`.

## Journal du dû (greffière, session 017)

Le catalogue v1 est un journal append-only déclaré par `catalogue_path` dans
la configuration. `registre list` en est la seule vue humaine d'autorité.

- **Migration** : `registre migrer --depuis <prose.jsonl>` conserve chaque
  texte verbatim en `pending_qualification` ; aucune sévérité ni source n'est
  inventée.
- **Qualification** : humaine uniquement (`registre qualifier`), champs fermés
  déclarés.
- **Interdits** : écriture vers plans/tasks/issues hôtes ; message libre depuis
  le catalogue ; score, déduplication automatique, état `planned`, runtime
  résident, adaptateur hôte.
- **Limite v1** : la transition `open → delivered` exige un lien d'arbitrage
  `(constat_id, objective_id)` posé à la délégation et une clôture d'objectif
  attestée ; chaque commande `registre` réconcilie ces faits depuis le store,
  sans boucle résidente ni polling.

Guide opératoire : `specs/017-greffiere-catalogue/quickstart.md`.
