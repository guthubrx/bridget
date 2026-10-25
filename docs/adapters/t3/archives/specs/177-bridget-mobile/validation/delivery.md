# Archives techniques de l’adaptateur T3

Document d’origine : [specs/177-bridget-mobile/validation/delivery.md](../../../../../../../specs/177-bridget-mobile/validation/delivery.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Conversation compacte, fin de grand message et clavier, historique récent,
  noms courts de projets sans `.git` et distinction des projets homonymes.
- Suivi continu qui ne se présente plus en reconnexion à chaque attente vide.
  Relecture bornée du reçu après envoi ; aucun renvoi automatique de POST ni
  changement de clé après une réponse perdue.
- Action « Transmettre maintenant » avec autorité limitée au message du téléphone.
  Le fournisseur peut recevoir l’instruction pendant le travail. Sinon, la même
  instruction attend en file ; l’application n’invente pas de nouvelle mission.
  Un guidage immédiat peut interrompre un outil ; aucun redémarrage n’est demandé.
- Outils regroupés sous « Outils utilisés », repliés avec chevrons et texte discret.
  Une identité d’outil stable ne conserve que son état le plus récent dans la vue.
  L’application lit les événements natifs Bridget. Elle ne contacte pas T3.
- Après le redémarrage concurrent du service, un destinataire momentanément absent
  conserve le message local et reçoit une explication lisible. Le simple texte
  « vérification affichage » refusé à ce moment n’a pas été réinjecté sous une
  nouvelle clé. Sa remise n’est pas revendiquée.

<a id="extrait-2"></a>

## Extrait 2

- Lot natif `mobile177` : 63 PASS, 1 recette d’origine ignorée par défaut.
- Régression guidage T3 : 6 PASS ; transport promotion : 8 PASS.
- Android : 10 tests JVM PASS (conversation 5, projets 3, HTTPS 2), lintRelease
  et assembleRelease PASS. Mise à jour finale physique r10b non exécutée.
- Builds release optimisés Mac et Linux PASS. Signature ad hoc Mac vérifiée.
- Deux revues indépendantes lues et acquittées. Corrections retenues : état
  orphelin terminal, imports JUnit, déduplication des états d’outil et information
  sur l’interruption possible d’outils. Pas de troisième campagne de revue.
- Documentation : 29 pages, 438 liens locaux et 12 paires linguistiques PASS.
- HTTPS public : réponse 401 sans autorisation, certificat TLS valide.
- Firebase : OAuth HTTP 200 et validation seule FCM HTTP 200 pour un appareil
  enregistré. Aucun contenu métier ni notification réelle transmis par ce contrôle.
  Le compte privé est en 0600 ; aucun jeton ou secret n’est inclus ici.

<a id="extrait-3"></a>

## Extrait 3

- Binaires `177-r10b-opt` installés et actifs sur Mac et Cartae.
- Core et backend mobile Cartae actifs ; connecteur Mac actif avec 22 fils.
- Application T3 PID 16798 et workers R8 conservés sans redémarrage.
- APK 0.5.0 signé, certificat courant SHA-256
  `0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588`.
- APK final SHA-256
  `756e2144bb6fdb66859c051a05d9930188bc30adba5de6f4e455124f4cd1a6ef`.
- Le Pixel n’est plus visible en USB à la clôture de ce lot. L’APK r10b est
  préparé mais n’est pas déclaré installé. L’association existante est conservée.
- Pas de publication GitHub 0.5.0 présentée comme recette complète.
