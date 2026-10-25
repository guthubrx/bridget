# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/163-decision-admission-sous-autorite-maitre.md](../../../../../decisions/163-decision-admission-sous-autorite-maitre.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le développement Rust, les compilations et les tests Linux de 163 sont réalisés sur Cartae. Utiliser un checkout de session et un namespace privé avec umask 077, TMPDIR court et fixtures isolées. Ne jamais toucher le worker actif 471321, son socket ni son état pour ces essais. Le défaut de rattachement du connecteur est hors fonctionnalité 163. ROOT rapporte son rétablissement opérationnel : limite du seul pont passée de 4096 à 8192 et pont seul relancé ; daemon et T3 protégés. Le checkout distant de session existe, branche session-163-decision-admission, base dcfd9bc29140457b1fa016c276f5a48933415a5c propre selon ROOT. Le rédacteur n’a exécuté aucun SSH ; ce checkpoint ne prouve pas encore le lancement de DEV_CARTAE163.

<a id="extrait-2"></a>

## Extrait 2

Les modifications T3, les tests propres à macOS, la signature et l’installation Mac restent sur le Mac. La recette du serveur de décision part du maître Mac : un succès Linux ne vaut pas validation du chemin réseau ni du comportement macOS. Les builds Linux et Mac doivent attester le même commit source et les empreintes des sources utilisées. Aucun redémarrage de T3.
