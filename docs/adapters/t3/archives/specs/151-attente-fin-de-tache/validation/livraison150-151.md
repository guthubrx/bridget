# Archives techniques de l’adaptateur T3

Document d’origine : [specs/151-attente-fin-de-tache/validation/livraison150-151.md](../../../../../../../specs/151-attente-fin-de-tache/validation/livraison150-151.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les PID et leurs dates de naissance sont inchangés : daemon 30562 depuis 02:17:41 CEST, pont 30974 depuis 02:17:48 CEST, T3 60116 depuis 01:13:35 CEST et serveur T3 60228 depuis 01:13:36 CEST.
Aucun service ni T3 n'a été relancé. L'installation du fichier n'active pas le nouveau code dans ces processus.

<a id="extrait-2"></a>

## Extrait 2

Le principal a retiré les worktrees 149/150/151 sans force. Il a d'abord contrôlé leur propreté, leur ascendance, leur push et l'absence de processus ouverts.
Les trois branches locales ont été retirées. Aucune branche distante correspondante n'existait.
Le constructeur a retiré 69,42 Gio logiques de cache : 6,80 Gio dans main et 62,62 Gio dans le worktree 150. Le retrait du worktree 150 a ensuite retiré son reliquat de 1,50 Gio.
Ces volumes sont logiques. APFS et l'activité concurrente empêchent d'attribuer un gain physique à ces suppressions.
Espace libre final mesuré par le principal : 40013060 Kio, soit environ 38,16 Gio.
Les bases de données, la production T3, les sauvegardes 149, le stage T3 en attente et les liens symboliques étrangers sont conservés.
