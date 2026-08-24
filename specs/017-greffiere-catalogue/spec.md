# Spec 017 — Maicie greffière du catalogue

**Branche** : `session-17-greffiere-catalogue` | **Créée** : 2026-08-23
**Statut** : Proposition de conception
**Origine** : engagement « Maicie greffière du catalogue » de
`specs/011-maicie-orchestration/exigences-coordination-v2.md`, amendé par
`specs/011-maicie-orchestration/revue-adverse-boucle-amelioration-2026-08-23.md`.

## Intention produit

Maicie tient le **journal du dû** avec la même rigueur que le journal du fait :
elle consigne les constats déclarés, les relie à leur mission et reflète les
clôtures d'objectifs attestées. Elle rend la liste lisible et retrouvable, mais
ne décide jamais ce qui doit être travaillé ensuite.

Cette fonctionnalité transforme une discipline fragile en comportement produit.
Elle répond à trois incidents attestés : métriques collectées puis perdues faute
de commit, annotations d'état oubliées à deux reprises, et clôtures de remèdes
retardées à trois reprises. L'erreur d'homonymie du lot C au greffe impose par
ailleurs que tout ciblage d'objectif utilise un identifiant, jamais un texte.

## Scénarios utilisateur et validation

### US1 — Consigner un constat fermé (P1)

Une personne ou une mission autorisée ajoute un constat au catalogue du projet
avec son texte verbatim, sa sévérité explicitement déclarée et sa mission
source. Maicie le refuse si une clé obligatoire, une valeur d'énumération ou un
identifiant de source est invalide.

**Pourquoi P1** : sans format fermé et appendable, les constats restent de la
prose susceptible d'être perdue ou oubliée.

**Test indépendant** : migrer un catalogue-prose d'exemple, y ajouter un
constat, puis rouvrir le fichier et vérifier l'entrée exacte et son identifiant
stable.

**Scénarios d'acceptation** :

1. **Étant donné** un catalogue v1 migré, **quand** un constat `add` conforme
   est soumis, **alors** une seule entrée appendable et versionnée est ajoutée
   sans modifier les artefacts de planification de l'hôte.
2. **Étant donné** un constat avec sévérité inconnue, texte absent ou source
   sans identifiant, **quand** il est soumis, **alors** Maicie le refuse sans
   modifier le catalogue.
3. **Étant donné** un rejet ou une coupure après l'append d'un constat,
   **quand** le même `id` et les mêmes octets canoniques sont rejoués,
   **alors** Maicie répond de façon idempotente sans ajouter de seconde ligne.
4. **Étant donné** deux objectifs aux libellés semblables, **quand** un constat
   les référence, **alors** seul l'identifiant explicite est accepté ; aucun
   rapprochement textuel n'est tenté.

---

### US1bis — Transcrire un fait couvert (P1)

Lorsqu'un **type de fait couvert** par le contrat de correspondance ci-dessous
est observé (gate raté, verdict de revue `AMENDER`), Maicie append un `add`
dont la sévérité est **dérivée** de la table versionnée — pas jugée à
l'écriture. Tout fait hors table part en `pending_qualification`.

**Pourquoi P1** : le journal du dû doit se remplir sans frappe du référent
lorsque la décision est déjà prise ailleurs (définition d'un gate, verdict
explicite d'un relecteur).

**Scénarios d'acceptation** :

1. **Étant donné** un gate identifié qui échoue, **quand** le fait
   `gate_failed` est consigné, **alors** un `add` de sévérité `blocker` et
   source `gate`/`failed=true` est appendé (ou no-op idempotent).
2. **Étant donné** un verdict de revue `AMENDER` identifié, **quand** le fait
   `review_amender` est consigné, **alors** un `add` de sévérité `major` et
   source `review` est appendé.
3. **Étant donné** un type de fait absent de la table, **quand** une
   transcription est demandée, **alors** aucune sévérité n'est inventée : le
   texte part en `pending_qualification`.

---

### US2 — Constater automatiquement la clôture (P1)

Lorsqu'un objectif explicitement lié par une délégation d'arbitrage est clôturé
de façon attestée, Maicie ajoute la transition automatique correspondante au
constat lié. Elle la déclenche par événement attesté ou, après une perte de
transport, par réconciliation en lecture de l'état durable de cet objectif.
Elle ne reconstruit pas
une clôture depuis une échéance, une absence de message ou une interprétation.

**Pourquoi P1** : les remèdes livrés ne doivent plus rester affichés comme dus
par oubli manuel.

**Test indépendant** : fermer un objectif identifié, rouvrir le catalogue et
vérifier la transition unique ; rejouer la clôture ne crée pas de seconde
transition.

**Scénarios d'acceptation** :

1. **Étant donné** un constat ouvert dont une délégation d'arbitrage journalise
   le couple `(constat_id, objective_id)`, **quand** cet
   objectif est clôturé durablement, **alors** son état dérivé devient livré
   par une transition automatique journalisée.
2. **Étant donné** une clôture rejouée, **quand** Maicie la reçoit à nouveau,
   **alors** le journal et la vue restent inchangés.
3. **Étant donné** un constat sans objectif clôturé attesté, **quand** une
   horloge avance ou qu'une mission devient silencieuse, **alors** son état ne
   change pas.
4. **Étant donné** un événement de clôture perdu, **quand** la réconciliation
   relit l'état durable de l'objectif identifié, **alors** elle dépose la même
   transition unique sans inférer une clôture depuis le silence.

---

### US3 — Consulter sans jugement ni prescription (P2)

Une personne consulte `registre list` et obtient une vue déterministe des
constats ouverts, triée exclusivement sur des champs déclarés. Le pied de page
répète les décomptes nécessaires aux jalons ; aucun élément n'est transformé en
tâche, en plan ou en ordre de travail.

**Pourquoi P2** : le catalogue doit rester découvrable sans devenir un Jira
automatique ni dépendre de la mémoire d'une session.

**Test indépendant** : permuter l'ordre physique de plusieurs entrées valides
et vérifier que la vue et son pied de page restent identiques.

**Scénarios d'acceptation** :

1. **Étant donné** des constats ouverts avec sévérités, récurrences et sources
   de gate déclarées, **quand** `registre list` est demandé, **alors** leur
   ordre est déterministe et explicable par ces seuls champs.
2. **Étant donné** une vue de jalon, **quand** elle est rendue, **alors** son
   pied de page affiche exactement `N` ouverts, `M` récurrents, `K` liés à un
   gate raté et `P` en attente de qualification.
3. **Étant donné** un constat ouvert, **quand** la vue est consultée, **alors**
   aucune écriture n'est faite dans `tasks.md`, un plan, une issue ou un autre
   artefact hôte.

---

### US4 — Encadrer les arbitrages qui déclenchent du travail (P2)

Quand un humain décide qu'un constat mérite un remède, Maicie n'envoie jamais
un message libre à un agent : l'action passe par une délégation durable. La
consultation et le tri restent eux-mêmes sans effet.

**Pourquoi P2** : les jonctions hors greffe ont montré qu'un message nu perd la
traçabilité et peut contourner le cycle de coordination.

**Test indépendant** : tenter de lancer un travail depuis un constat via une
voie de message libre ; vérifier le refus, puis vérifier qu'une délégation
explicite conserve corrélation et reçu durable.

## Cas limites

- Un catalogue-prose historique ambigu est migré en conservant son texte
  verbatim et un statut de migration explicite ; aucune déduplication ou
  interprétation automatique n'est autorisée.
- `recurrence_of` peut référencer seulement un identifiant de constat existant ;
  il ne peut pas être déduit du texte ni créer une boucle.
- Une transition de clôture visant un objectif absent ou déjà appliquée est
  refusée ou idempotente sans créer d'état intermédiaire `planifié`.
- Un objectif ne peut jamais être retrouvé par son titre, son corps ou une
  correspondance approximative.
- Les données non reconnues par le format fermé sont refusées avant append ;
  les futurs formats exigent une version nouvelle et une migration explicite.

## Exigences

### Fonctionnelles

- **FR-1701 — Format canonique fermé** : Maicie DOIT définir un catalogue v1
  versionné et machine-appendable. Ce fichier est un journal canonique ;
  `registre list` est sa seule vue humaine d'autorité. Les entrées
  acceptées sont des enregistrements fermés, versionnés et append-only ; tout
  champ ou type inconnu est refusé.
- **FR-1702 — Migration conservatrice** : Maicie DOIT migrer le
  catalogue-prose existant vers le format v1 sans supprimer le texte source ni
  inventer une sévérité, une récurrence, une source ou un état. Une entrée
  historique incomplète reste en attente d'une qualification humaine explicite
  et ne devient jamais un constat actif par défaut.
- **FR-1703 — Constat add** : `constat add` DOIT exiger `id`, `date`,
  `mission_source`, `severity`, `recurrence_of` optionnel et `text` verbatim.
  `date` est un horodatage ISO-8601/RFC 3339 avec fuseau explicite et la
  sévérité est l'énumération fermée `blocker|major|minor|info`. La sévérité
  est soit **déclarée** par l'émetteur humain, soit **dérivée** par
  transcription du contrat de correspondance (FR-1711) pour un type de fait
  couvert : ce n'est pas un jugement formé à l'écriture. Hors table, aucune
  sévérité n'est inventée. Un `id` déjà présent est un no-op idempotent si les
  octets canoniques sont identiques, sinon un refus sans mutation : il ne peut
  jamais produire une seconde entrée.
- **FR-1711 — Correspondance fait→sévérité** : Maicie DOIT appliquer
  exclusivement la table versionnée ci-dessous. Elle est totale sur les types
  qu'elle couvre (aucune exception « selon le contexte »). Tout fait hors
  table DOIT partir en `pending_qualification`. Classer par importance,
  calculer une priorité ou décider qu'un constat mérite l'attention reste
  interdit : la frontière passe entre **transcrire** une décision prise
  ailleurs et **former** un avis.
- **FR-1704 — Références exactes** : une mission source, une récurrence et une
  clôture d'objectif DOIVENT utiliser des identifiants stables. Le système NE
  DOIT effectuer aucune recherche par texte, similarité ou homonymie.
- **FR-1705 — Transition attestée** : la clôture durable d'un objectif DOIT
  ajouter atomiquement ou idempotemment la transition de constat associée. Le
  seul lien admissible est le fait déclaré `(constat_id, objective_id)` porté
  et journalisé par une délégation d'arbitrage. La transition est déclenchée
  par un événement de clôture ou par la lecture de l'état durable attesté du
  même `objective_id`, jamais par une horloge, une absence ou un texte. Les
  seuls états v1 sont `open` et `delivered`; aucun état `planned` n'existe.
- **FR-1706 — Vue de lecture** : `registre list` DOIT être une fonction pure
  du catalogue : elle trie les constats ouverts sur les valeurs déclarées
  (sévérité, marqueur de récurrence, source de gate, date, identifiant) et ne
  modifie aucune donnée.
- **FR-1707 — Aucun pilotage d'hôte** : Maicie NE DOIT écrire ni dans les plans,
  ni dans `tasks.md`, ni dans les issues, ni dans les artefacts de workflow de
  l'hôte. La conversion en tâche reste un acte humain extérieur au v1.
- **FR-1708 — Délégation obligatoire** : tout arbitrage qui lance du travail
  DOIT créer une délégation durable ; aucune API, commande ou automatisation
  de catalogue ne DOIT envoyer un message nu à un agent.
- **FR-1709 — Découvrabilité** : la skill Maicie DOIT prescrire `registre list`
  au début d'une session et avant une proposition de suite ; les sorties de
  jalon et le rituel de clôture DOIVENT rendre le décompte déterministe et la
  vue triée.
- **FR-1710 — Append atomique** : l'ajout et l'idempotence DOIVENT être
  décidés sous le même verrou exclusif du journal déclaré, puis la ligne entière
  est appendée et synchronisée avant de rendre succès. Un remplacement global
  par fichier temporaire est interdit, car il pourrait écraser un append
  concurrent.

### Entités clés

- **Constat** : fait appendé, identifié, daté, lié à une mission source et
  portant texte verbatim, sévérité déclarée et éventuelle récurrence.
- **Mission source** : fait d'origine typé (mission, incident, review ou gate)
  avec identifiant stable ; un gate raté est un fait de source, non un score.
- **Fait couvert** : type de fait listé dans le contrat de correspondance
  FR-1711 ; sa sévérité est dérivée par transcription, jamais jugée à
  l'écriture.
- **Transition de constat** : événement appendé qui dérive l'état `delivered`
  d'un constat à partir de la clôture durable d'un objectif identifié.
- **Lien d'arbitrage** : fait durable de délégation portant le couple exact
  `(constat_id, objective_id)` qui autorise, et seul autorise, une transition.
- **Vue de registre** : projection en lecture seule du journal canonique et de
  ses champs explicitement déclarés.

## Contrat de correspondance fait → sévérité (FR-1711)

Versionnée avec la spec 017. Toute modification de cette table est un
changement de sémantique du journal : elle repasse par revue hostile.

Règle générale : **le défaut est l'attente, jamais l'invention.** Si un type
de fait n'apparaît pas ici, ou si l'appartenance à une case est douteuse, le
texte part en `pending_qualification`.

| Type de fait (id machine) | Source journal | Sévérité dérivée | Justification (une ligne) |
|---|---|---|---|
| `gate_failed` | `mission_source.kind=gate`, `failed=true` | `blocker` | Échouer un gate **est** la définition d'un blocage ; ce n'est pas une évaluation. |
| `review_amender` | `mission_source.kind=review` | `major` | Le verdict `AMENDER` est déjà une décision du relecteur ; le journal la transcrit. |

Hors table (exemples non couverts, non exhaustifs) : gate vert, verdict
`APPROVE`, incident libre, mission close sans lien d'arbitrage, prose
ambiguë — tous → `pending_qualification` ou refus, **jamais** une sévérité
improvisée.

Identité d'émission automatique : `id = "{type}:{source_id}"` (ex.
`gate_failed:G1701`). Le rejeu du même fait est un no-op idempotent.

## Critères mesurables

- **SC-1701** : une migration de corpus prose produit un catalogue v1 lisible,
  dont 100 % des entrées sont ré-ouvrables et dont aucun texte historique n'est
  perdu ou réécrit.
- **SC-1702** : 100 % des tentatives comprenant un type, un champ, une sévérité
  ou une référence inconnue sont refusées sans modifier le catalogue.
- **SC-1703** : une clôture d'objectif identifié produit exactement une
  transition `delivered`; son rejeu produit zéro transition supplémentaire.
- **SC-1704** : deux catalogues équivalents avec ordres physiques différents
  donnent une vue `registre list` octet pour octet identique, ainsi qu'un pied
  de page exact `N/M/K/P`.
- **SC-1705** : les tests attestent qu'aucune commande de la greffière n'écrit
  dans les artefacts de planification de l'hôte ni n'émet de message libre ;
  tout lancement de travail observé correspond à une délégation durable.
- **SC-1706** : un test de démarrage de session et un test de proposition de
  suite vérifient que la skill prescrit `registre list` dans les deux cas.
- **SC-1707** : un test de rituel de clôture vérifie que la première opération
  consulte la vue triée et que le pied de page expose `N/M/K/P`.
- **SC-1708** : un `add` rejoué avec le même identifiant et les mêmes octets
  canoniques crée une seule entrée ; le même identifiant avec des octets
  divergents est refusé sans append.
- **SC-1709** : un événement de clôture absent suivi d'une réconciliation sur
  l'objectif identifié produit exactement la même unique transition.
- **SC-1710** : deux writers concurrents ajoutent deux lignes complètes et
  distinctes au catalogue déclaré ; aucune entrée n'est perdue ni tronquée.
- **SC-1711** : un fait `gate_failed` produit un `add` `blocker` ; un fait
  `review_amender` produit un `add` `major` ; un type hors table produit un
  `pending_qualification` sans sévérité ; le rejeu du même `id` dérivé est un
  no-op.

## Hors périmètre

- Déduplication ou rapprochement automatique de constats.
- Score composite, classement par jugement, comparaison inter-projets ou
  interprétation par LLM.
- Écriture dans les plans, tasks, issues ou backlog de l'hôte.
- État `planned` et toute automatisation qui transforme un constat en tâche.
- Runtime résident, GUI et adaptateurs Jira/GitHub ; ils restent des suites
  possibles après les routines du bloc F.

## Hypothèses

- Chaque projet hôte déclare son unique catalogue v1 dans la configuration
  Maicie et le versionne avec son dépôt.
- La fermeture d'objectif durable est accessible au composant Maicie comme fait
  attesté, notamment via les événements Bridget/guichet déjà prévus.
- Les humains gardent l'autorité sur la récurrence, la promotion d'un constat
  hors table, et les arbitrages inter-projets. La sévérité des faits **couverts**
  est fixée par le contrat FR-1711 ; hors table, elle reste humaine via
  qualification.
