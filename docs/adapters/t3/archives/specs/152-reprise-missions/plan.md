# Archives techniques de l’adaptateur T3

Document d’origine : [specs/152-reprise-missions/plan.md](../../../../../../specs/152-reprise-missions/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- Le risque principal est l'environnement natif d'un enfant relancé (H4). Un
  test rouge le prouve avant le code.
- Non vérifiable dans ce dépôt : le comportement réel de T3 sur un fil `working`
  pendant la pause. La vue reste `working` (question 3).

<a id="extrait-2"></a>

## Extrait 2

Après les gates de validation : commits, livraison et nettoyage de la session
152, puis exécution séquentielle des sessions 153 à 160, avec leur propre
livraison et nettoyage. Le principal coordonne Git, les fichiers installés et
l'activation de production. La référence historique 147 à 151 installait les
fichiers sans relance ; elle n'autorise pas à interrompre les autres fils T3.
Le diff 152 est enregistré dans
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/diff-152.patch`.
