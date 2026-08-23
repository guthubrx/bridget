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
3. **Étant donné** deux objectifs aux libellés semblables, **quand** un constat
   les référence, **alors** seul l'identifiant explicite est accepté ; aucun
   rapprochement textuel n'est tenté.

---

### US2 — Constater automatiquement la clôture (P1)

Lorsqu'un objectif référencé est clôturé de façon attestée, Maicie ajoute la
transition automatique correspondante au constat lié. Elle ne reconstruit pas
une clôture depuis une échéance, une absence de message ou une interprétation.

**Pourquoi P1** : les remèdes livrés ne doivent plus rester affichés comme dus
par oubli manuel.

**Test indépendant** : fermer un objectif identifié, rouvrir le catalogue et
vérifier la transition unique ; rejouer la clôture ne crée pas de seconde
transition.

**Scénarios d'acceptation** :

1. **Étant donné** un constat ouvert lié à `objective_id`, **quand** cet
   objectif est clôturé durablement, **alors** son état dérivé devient livré
   par une transition automatique journalisée.
2. **Étant donné** une clôture rejouée, **quand** Maicie la reçoit à nouveau,
   **alors** le journal et la vue restent inchangés.
3. **Étant donné** un constat sans objectif clôturé attesté, **quand** une
   horloge avance ou qu'une mission devient silencieuse, **alors** son état ne
   change pas.

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
   pied de page affiche exactement `N` ouverts, `M` récurrents et `K` liés à un
   gate raté.
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
  versionné, lisible dans le dépôt hôte et machine-appendable. Les entrées
  acceptées sont des enregistrements fermés, versionnés et append-only ; tout
  champ ou type inconnu est refusé.
- **FR-1702 — Migration conservatrice** : Maicie DOIT migrer le
  catalogue-prose existant vers le format v1 sans supprimer le texte source ni
  inventer une sévérité, une récurrence, une source ou un état. Une entrée
  historique incomplète reste en attente d'une qualification humaine explicite
  et ne devient jamais un constat actif par défaut.
- **FR-1703 — Constat add** : `constat add` DOIT exiger `id`, `date`,
  `mission_source`, `severity`, `recurrence_of` optionnel et `text` verbatim.
  La sévérité est une énumération fermée déclarée par l'émetteur ; Maicie ne la
  calcule jamais.
- **FR-1704 — Références exactes** : une mission source, une récurrence et une
  clôture d'objectif DOIVENT utiliser des identifiants stables. Le système NE
  DOIT effectuer aucune recherche par texte, similarité ou homonymie.
- **FR-1705 — Transition attestée** : la clôture durable d'un objectif DOIT
  ajouter atomiquement ou idempotemment la transition de constat associée. Les
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

### Entités clés

- **Constat** : fait appendé, identifié, daté, lié à une mission source et
  portant texte verbatim, sévérité déclarée et éventuelle récurrence.
- **Mission source** : fait d'origine typé (mission, incident, review ou gate)
  avec identifiant stable ; un gate raté est un fait de source, non un score.
- **Transition de constat** : événement appendé qui dérive l'état `delivered`
  d'un constat à partir de la clôture durable d'un objectif identifié.
- **Vue de registre** : projection en lecture seule du journal canonique et de
  ses champs explicitement déclarés.

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
  de page exact `N/M/K`.
- **SC-1705** : les tests attestent qu'aucune commande de la greffière n'écrit
  dans les artefacts de planification de l'hôte ni n'émet de message libre ;
  tout lancement de travail observé correspond à une délégation durable.

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
- Les humains gardent l'autorité sur la sévérité, la récurrence, la promotion
  d'un constat et les arbitrages inter-projets.
