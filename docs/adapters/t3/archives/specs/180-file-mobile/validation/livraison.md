# Archives techniques de l’adaptateur T3

Document d’origine : [specs/180-file-mobile/validation/livraison.md](../../../../../../../specs/180-file-mobile/validation/livraison.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Composant | Observation après installation finale |
|---|---|
| Core Cartae | PID2153657, actif, build180-21a16c374d71 |
| Worker Cartae | PID2156728, actif, même exécutable final |
| Mobile Cartae | PID2153921, actif, même exécutable final |
| Connecteur Mac | PID46375, chargé depuis le répertoire180-21a16c374d71 |
| Pixel | Bridget0.6.1, versionCode6001, association conservée |
| T3 | PID650/975 et dates de naissance conservés, aucune relance |

<a id="extrait-2"></a>

## Extrait 2

La maintenance manuelle réemploie la procédure179 : contrôle des admissions et commandes worker, sauvegardes, arrêt propre du mobile/core, vérification des cgroups sans enfant de mission, arrêt worker, barrière privée bornée180s, liens stables commutés, reprise et contrôle de santé Mac/Cartae, libération de la barrière. Aucun fournisseur ni T3 arrêté. Les reçus et sauvegardes de l’installation candidate restent distincts de ceux de l’installation finale.

<a id="extrait-3"></a>

## Extrait 3

La recette Pixel emploie un destinataire natif contrôlé, sans modèle. Trois messages : édition puis annulation sans remise ; interruption reçue une seule fois avec steer_current ; message normal remis au retour au repos pendant que l’application était arrêtée. Le témoin contient deux IDs uniques, la base un cancelled et deux acked. Aucun message de test dans les vrais fils utilisateurs. Le guidage Codex/T3 est vérifié par les tests ciblés et la lecture de l’adaptateur, pas par une nouvelle interruption d’un vrai tour utilisateur. Les filtres/favoris ont été remis et Bridget laissé sur le fil courant.
