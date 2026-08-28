# Spécification 059 — Mesure vérification / production

**Statut** : Implémentée, prête pour revue

**Branche** : `session-059-mesure-verification-production`

**Base gelée** : `394c0f5c8d352a606c85db7232bb1ad4fa79446e`

**Objectif Maicie** : `924ccb3c-2ac1-49e7-85a9-746d32d36a11`

**Délégation** : `f3ca9807-bdf2-4be9-88bb-bbeef803a86e`

**Message** : `78cba682-d9e5-434c-b4fb-c3078606118e`

## Problème

Le premier rapport vérification / production classe les buts par mots-clés.
Sur la fenêtre humaine de 169 objectifs, il place 92 objectifs en « mixte » et
19 en « autre », puis calcule son ratio sur les 58 restants. La couverture
effective n'est donc que de 34,32 %. Ce ratio détecte correctement une forte
activité de vérification, mais son échelle n'est pas opposable.

Le corpus historique n'apporte pas la causalité qui permettrait de réparer ce
classement automatiquement. Le constat `6e51af3` établit notamment que la
provenance est absente historiquement et trompeuse sur les deux objectifs
neufs qui la portent. Le champ `origin` n'est donc pas une entrée du présent
calcul.

Une seconde distorsion existe indépendamment du classement : découper une
revue en cinq charges multiplie le numérateur par cinq. Un ratio par objectifs
seul confond ainsi le volume de travail et la granularité choisie par le
référent.

## Décision

La ronde passive publie deux unités et leur couverture :

1. **objectifs** : chaque objectif compte une fois dans sa classe ;
2. **racines de lot** : une racine compte au plus une fois dans chaque classe,
   quel que soit le nombre de charges qui en descendent.

Le rapport inclut un manifeste complet `objective_id -> classe` pour la
population observée. Chaque entrée rend également sa racine, la base du
classement et le condensat du but lu. Une entrée sans annotation explicite est
`indeterminate` ; elle n'est jamais inférée depuis des mots-clés.

Les classes fermées sont :

- `production` : livre directement une capacité demandée ;
- `verification` : n'existerait pas sans l'évaluation d'un artefact déjà
  produit, y compris une charge ou un amendement né de cette évaluation ;
- `instruction` : établit un fait sans mutation autorisée du produit ;
- `repair` : rétablit un défaut déjà intégré ;
- `indeterminate` : les données durables ne permettent pas de trancher.

Un objectif qui observe puis modifie un lot reste `verification`. L'effet de
la tâche ne change pas sa cause. Le seau « mixte » disparaît donc du contrat,
sans répartir arbitrairement son poids entre deux classes.

## Ratios publiés

Pour chacune des deux unités, le rapport expose les comptes bruts et deux
dénominateurs, sans choisir à la place de l'humain le statut d'une réparation :

- `verification / production` ;
- `verification / (production + repair)`.

Un ratio ponctuel n'est `available` que si la couverture de classement de son
unité vaut 100 % et si son dénominateur est non nul. Sinon le rapport rend
`unavailable` avec la raison, les comptes connus, les indéterminés et la
couverture. Il ne calcule jamais un pourcentage sur les seuls classés en le
présentant comme celui de la population.

La valeur cible et le choix du dénominateur appartiennent à l'humain. Cette
session n'en fixe aucun.

## Racine et invariance au découpage

`root_id` identifie la capacité ou le lot dont l'objectif dépend causalement.
Le lot de production et ses revues, charges et amendements partagent cette
racine. Une racine peut donc appartenir à la fois aux ensembles `production`
et `verification`.

L'agrégat de racines est une cardinalité d'ensembles : ajouter un second
objectif `verification` sous une racine qui en porte déjà un augmente le
compte par objectifs et le facteur d'expansion, mais ne change ni l'ensemble
des racines vérifiées ni le ratio par racines. C'est la contre-épreuve exigée
contre le découpage artificiel.

Une racine manquante rend la couverture de racine incomplète. Elle n'est ni
fabriquée depuis le texte du but, ni remplacée par l'identifiant de l'objectif,
car ce dernier geste réintroduirait exactement le biais de découpage.

## Population et reproductibilité

Chaque publication nomme :

- `as_of`, la borne d'observation ;
- le genre de fenêtre, ses deux bornes, leurs inclusivités et le fuseau ;
- le cardinal de population et le SHA-256 des identifiants ordonnés ;
- la version du classificateur et le SHA-256 du manifeste effectif ;
- les couvertures de classement et de racine ;
- les comptes par objectifs et par racines.

La fenêtre humaine historique reste une baseline gelée, distincte d'une
fenêtre glissante ou d'un jour civil :

- début exclusif : `1787788534` ;
- fin inclusive : `1787874934` ;
- ordre : `cree_at`, puis `id` ;
- 169 identifiants, 6253 octets avec un LF final ;
- SHA-256 :
  `e2a279624ecd4fbd9ca7f8f97effa45777c15b6b3b9574f3d43a9d90298d5a55`.

La publication courante utilise une fenêtre glissante de 24 heures terminée
à `as_of`. Les deux fenêtres portent des noms distincts ; leurs résultats ne
sont jamais comparés comme s'ils décrivaient la même population.

## Publication continue et passivité

La mesure rejoint le JSON archivé par `bridget-ronde.py`, déjà exécuté par le
timer de ronde. Elle lit la copie SQLite jetable créée par la ronde ; elle ne
réconcilie aucune outbox, ne modifie aucune ligne et n'émet aucun message.

La publication est atomique avec l'archive existante : un rapport incomplet
n'est jamais installé comme dernier constat. Une source illisible produit un
état `unavailable`, jamais une population vide ni un ratio nul.

## Scénarios d'acceptation

### S059-1 — Corpus humain reproductible

**Étant donné** la base réelle et la fenêtre gelée,
**quand** la mesure ordonne puis condense ses identifiants,
**alors** elle rend 169, 6253 octets et le SHA-256 attendu.

### S059-2 — Couverture honnête

**Étant donné** des objectifs non annotés,
**quand** la mesure publie la fenêtre,
**alors** ils figurent dans le manifeste comme `indeterminate`, la couverture
les compte et le ratio ponctuel reste `unavailable`.

### S059-3 — Classement réfutable

**Étant donné** une annotation explicite,
**quand** le but durable est lu,
**alors** le rapport expose l'identifiant, la classe, la racine, la justification
et le condensat du but ; un tiers peut désigner exactement l'entrée contestée.

### S059-4 — Découpage sans déplacement du ratio racine

**Étant donné** une production et plusieurs vérifications réelles de la même
racine,
**quand** une vérification supplémentaire est comptée sous cette racine,
**alors** le compte d'objectifs augmente mais le compte de racines vérifiées et
le ratio par racines restent identiques.

Le mutant remplace la racine par `objective_id` pendant l'agrégation. Il doit
mourir à l'assertion finale du scénario, après lecture réussie des données
réelles.

### S059-5 — Dégradation explicite

**Étant donné** une base, un payload ou une borne illisible,
**quand** la ronde s'exécute,
**alors** la section mesure vaut `unavailable` avec une raison et les autres
constats de ronde restent publiables.

## Exigences fonctionnelles

- **FR-5901** : toute population publiée DOIT porter ses bornes, son fuseau,
  son cardinal et son empreinte reproductible.
- **FR-5902** : le manifeste effectif DOIT contenir chaque identifiant de la
  population, y compris sous la classe `indeterminate`.
- **FR-5903** : aucune classe historique NE DOIT être inférée lexicalement ou
  depuis `origin`.
- **FR-5904** : la couverture de classement et la couverture de racine DOIVENT
  être publiées séparément.
- **FR-5905** : un ratio incomplet ou à dénominateur nul DOIT être
  `unavailable`, jamais zéro ni calculé sur le seul sous-ensemble connu.
- **FR-5906** : les comptes par objectifs et par racines DOIVENT être publiés
  ensemble avec le facteur d'expansion.
- **FR-5907** : une racine DOIT compter au plus une fois par classe.
- **FR-5908** : la baseline humaine et la fenêtre glissante courante DOIVENT
  rester deux populations nommées et distinctes.
- **FR-5909** : l'instrument DOIT lire une copie SQLite et NE DOIT provoquer ni
  écriture d'autorité, ni décision, ni envoi.
- **FR-5910** : aucune valeur cible NE DOIT être encodée par cette session.

## Bornes explicites

- Le corpus historique n'est pas reclassé en base et reste octet-stable.
- Une annotation humaine du manifeste peut être fausse ; elle est rendue
  réfutable, pas transformée en vérité cryptographique.
- Tant que la couverture n'est pas complète, le résultat honnête peut être
  l'indisponibilité d'un ratio ponctuel. C'est un résultat mesuré.
- Le coût en temps, tours ou jetons n'est pas déduit des durées calendaires :
  aucune attestation d'effort exploitable n'existe pour la baseline.
- La session ne modifie ni `plugins/maicie/src/main.rs`, ni
  `plugins/maicie/src/lib.rs`, ni les autres fichiers portés par la session 058.
