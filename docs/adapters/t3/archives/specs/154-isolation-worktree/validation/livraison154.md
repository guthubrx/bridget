# Archives techniques de l’adaptateur T3

Document d’origine : [specs/154-isolation-worktree/validation/livraison154.md](../../../../../../../specs/154-isolation-worktree/validation/livraison154.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Activation constatée le 12 octobre : daemon 67985 depuis 01:25:36 CEST et
pont 68081 depuis 01:25:39 CEST ; statut RC0, build 806d0984fe45 chargé.
T3 reste inchangé : PID 60116/60228, naissances du 11 octobre à 01:13:35/36.
Aucune relance T3.

<a id="extrait-2"></a>

## Extrait 2

Complément à 01:27 CEST : 18 TMPDIR attestés, aucun fichier ouvert. Sept racines
vides retirées ; 74 937 515 octets logiques de fichiers hors base retirés. Toutes
les bases synthétiques et leurs fichiers SQLite auxiliaires restent conservés,
conformément à la règle « aucune suppression DB ». Aucun gain physique revendiqué.
Bases de production et conversations T3 intactes. Sauvegarde du binaire 153 :
/Users/moi/.cache/bridget-delivery154.l50umk90/rollback153-bridget
