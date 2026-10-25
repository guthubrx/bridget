# Archives techniques de l’adaptateur T3

Document d’origine : [specs/167-coordination-cli-autonome/spec.md](../../../../../../specs/167-coordination-cli-autonome/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

L'utilisateur doit pouvoir employer un agent Bridget comme coordinateur autonome depuis son terminal. T3 peut enrichir cette expérience, mais il ne doit pas être nécessaire pour créer, suivre, continuer ou arrêter une tâche.

<a id="extrait-2"></a>

## Extrait 2

Comme utilisateur de scripts et d'autres interfaces, je veux une sortie stable et exploitable par machine afin d'intégrer Bridget à mon environnement sans dépendre de T3.

<a id="extrait-3"></a>

## Extrait 3

**Test indépendant** : chaque commande du nouveau parcours fournit un format machine stable, conserve les identifiants durables et fonctionne lorsque T3 est arrêté.

<a id="extrait-4"></a>

## Extrait 4

1. Étant donné une commande de coordination, quand le format machine est demandé, alors la sortie contient des champs stables et aucun texte décoratif parasite.
2. Étant donné une nouvelle tentative avec la même clé et les mêmes données, quand l'issue précédente était incertaine, alors Bridget ne crée pas une seconde tâche.
3. Étant donné que T3 est absent, quand le daemon Bridget et le pilote requis sont disponibles, alors le parcours complet reste utilisable.
4. Étant donné une tâche créée par MCP, quand le terminal la consulte, alors les identifiants et les états ont le même sens.

<a id="extrait-5"></a>

## Extrait 5

- Reproduire l'interface graphique, les worktrees ou le graphe d'exécution de T3.
- Ajouter un planificateur de graphes ou un langage de workflows.
- Ajouter un nouveau fournisseur ou modifier l'authentification d'un fournisseur.
- Modifier les décisions d'admission, les quotas ou les droits existants.
- Créer une interface plein écran dans le terminal.
- Déduire qu'une fin de tour constitue une réussite de mission.
- Déployer ou configurer automatiquement T3.
