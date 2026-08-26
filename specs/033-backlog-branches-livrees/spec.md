# Spécification 033 — Backlog des branches livrées non fusionnées

**Statut** : Prête en relecture

**Base gelée du lot** : `52b831b791b44382bfe0b90715fd7caf37acdcea`

**Objectif** : `efe2e90c-6a3f-4412-aa64-c0c5a2928497`

## Problème mesuré

La ronde `bridget-idle` classe les agents mais ne regarde aucune branche. Elle
a tourné toutes les sept minutes pendant douze heures sans signaler dix-sept
branches distantes non fusionnées. Un lot muni d'un verdict favorable a dormi
de 16 h 27 à 04 h 12 ; cinq lots avaient déjà dormi une journée entière, dont
un que le référent allait redéléguer onze heures après sa correction.

Ce défaut n'est pas une pénurie de relecteurs. Pendant la mesure, onze agents
étiquetés relecture étaient vivants. L'inventaire ponctuel séparait environ six
à sept lots jamais routés, trois à quatre attendant le référent et cinq avec un
travail réel en cours. Le lot 033 corrige une cécité de l'instrument ; il ne
route, ne juge et ne fusionne rien.

## Propriété

À chaque exécution, la même sortie que la partition des agents expose une vue
Git locale de chaque référence `origin/*` dont la tête n'est pas ancêtre du
`origin/main` courant :

- âge du commit de tête ;
- SHA de la base commune présente dans l'historique de main, ou base périmée
  lorsqu'aucun ancêtre commun n'existe ;
- état `sans conflit textuel` ou `en conflit` ;
- blocage Git certain (`en conflit`, `base périmée, à rebaser`) ou blocage
  métier `indéterminé`.

Le libellé `sans conflit textuel` ne signifie jamais « mergeable » et
n'atteste ni compilation ni tests de la composition.

## Frontière du verdict

Le diagnostic sur copies a mesuré, dans la source Maicie accessible sous
Linux, `objectives=0`, `delegations=0` et `guichet_receptions=0`. La base
Bridget accessible porte dix-sept demandes guichet, toutes refusées, et zéro
`review_verdict` structuré. Les verdicts de la nuit n'existent que dans le
catalogue en prose ou dans des messages.

Le script ne cherche donc jamais un verdict par mot-clé, nom de branche ou
SHA. Une telle heuristique contredirait la limite branche/SHA de la spec 026 et
mentirait parfois. La sortie porte littéralement :

`BACKLOG BRANCHES INDISPONIBLE (greffe sans etat exploitable)`

La vue Git reste rendue comme partielle. Pour une branche sans blocage Git, la
dernière colonne vaut `indetermine — greffe sans etat exploitable`, jamais
`attend un relecteur` ni `attend le referent`.

## Scénarios

### US1 — Voir une branche non fusionnée

Une référence distante locale dont la tête n'est pas dans main apparaît avec
son âge, sa base, son état de conflit textuel et son blocage connu.

### US2 — Ne plus voir une branche fusionnée

Une référence distante conservée après fusion est exclue dès que sa tête
devient ancêtre du `origin/main` observé pendant l'exécution.

### US3 — Dire l'incomplétude

La sortie rappelle à chaque exécution que les refs sont locales sans fetch,
que l'âge est celui du commit de tête uniquement et que l'existence d'une ref
distante ne prouve pas une livraison métier.

### US4 — Échouer bruyamment et rapidement

Une commande Git en échec, un dépôt invalide ou le dépassement du budget total
rend `BACKLOG BRANCHES INDISPONIBLE (<motif>)`. Une liste partielle n'est jamais
présentée comme complète.

## Exigences fonctionnelles

- **FR-3301** : `origin/main` est résolu à chaque invocation ; le SHA de la
  base du développement n'est jamais figé dans l'instrument.
- **FR-3302** : aucune commande `fetch`, `merge`, `push`, création de branche
  ou écriture dans le dépôt observé n'est autorisée.
- **FR-3303** : seules les références distantes déjà présentes sous
  `refs/remotes/origin/` sont observées ; `origin/HEAD` et `origin/main` sont
  exclus.
- **FR-3304** : une branche est fusionnée si sa tête est ancêtre de main ; elle
  est absente du backlog dans ce cas.
- **FR-3305** : l'âge est `maintenant - committerdate(tête)`, borné à zéro.
- **FR-3306** : la base affichée est le `merge-base` avec main ; l'absence
  d'ancêtre commun donne `base périmée`.
- **FR-3307** : `merge-tree --write-tree` est exécuté avec un répertoire
  d'objets temporaire hors dépôt et l'object store réel en lecture seule comme
  alternative.
- **FR-3308** : le budget Git est global à l'analyse, configurable et égal à
  cinq secondes par défaut ; son dépassement est visible.
- **FR-3309** : le texte et le JSON distinguent vue Git disponible et blocage
  métier indisponible ; aucun zéro silencieux n'est produit.
- **FR-3310** : les trois limites — refs locales sans fetch, âge du seul commit
  de tête, ref distante non équivalente à une livraison — figurent dans chaque
  sortie.
- **FR-3311** : le comportement historique de partition des agents reste
  inchangé lorsque la vue branches est disponible ou indisponible.

## Critères de succès

- **SC-3301** : dans un dépôt jetable, une branche distante non fusionnée est
  vue avec sa référence, sa tête et son âge exacts avant tout oracle d'absence.
- **SC-3302** : dans le même dépôt, une branche distante fusionnée et encore
  présente est absente du backlog.
- **SC-3303** : deux modifications incompatibles sont libellées `en conflit` ;
  un cas propre est seulement `sans conflit textuel`.
- **SC-3304** : une histoire sans ancêtre commun est libellée `base périmée`.
- **SC-3305** : un faux Git qui dépasse le budget produit
  `BACKLOG BRANCHES INDISPONIBLE` dans la borne annoncée.
- **SC-3306** : les refs, l'index et les objets du dépôt jetable sont
  identiques avant et après l'analyse.
- **SC-3307** : supprimer le filtre des têtes déjà ancêtres de main tue
  l'oracle nommé de présence puis d'absence.

## Hors périmètre explicite

- Création ou inscription d'un verdict structuré, routage automatique,
  correction des jurys zombies et choix d'un régime de revue.
- Détection d'une branche poussée mais jamais récupérée localement.
- Âge réel d'une livraison avant rebase : seul le commit de tête est daté.
- Détection d'un contenu déjà intégré par squash ou cherry-pick sans relation
  d'ascendance.
- Compilation, tests ou sûreté sémantique de la composition.
