# Archives techniques de l’adaptateur T3

Document d’origine : [specs/102-fils-inter-agents/quickstart.md](../../../../../../specs/102-fils-inter-agents/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La branche102 part de main1738a072 et ne contient pas les changements non commitées
de101. Leur emplacement connu est :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/101-abonnements-t3

<a id="extrait-2"></a>

## Extrait 2

T001 impose de relever git status et worktree list, retrouver le commit intégrant
101 s'il existe désormais, vérifier ses contrats (preuve MCP T3, observations),
puis documenter la base utilisée. Ne jamais faire git reset/checkout destructif,
copier les fichiers101, ni committer le travail d'un autre agent. Si101 reste
non intégré, demander à son propriétaire/utilisateur comment intégrer avant
d'éditer les fichiers communs. Le binaire actuellement installé n'est pas une
preuve d'intégration Git.

<a id="extrait-3"></a>

## Extrait 3

Les tests fonctionnels ne nécessitent aucun appel de modèle payant : fake
provider au bord du système, vraie DB/socket locale. Une recette avec agents
réels et un déploiement demandent une autorisation nouvelle et des preuves de
version/capacité chargée ; on ne relance pas T3 ni ses agents pour installer la skill.
