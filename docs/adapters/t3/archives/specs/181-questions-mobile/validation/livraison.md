# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/validation/livraison.md](../../../../../../../specs/181-questions-mobile/validation/livraison.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le brouillon reste sur son appareil. Aucun partage des réponses en cours de rédaction. Aucun protocole ni serveur T3 requis pour le parcours natif. Les sessions terminal capables d’un callback structuré et l’outil MCP bridget_ask_user sont pris en charge. Une TUI opaque exige cet outil MCP ; sa boîte native n’est pas capturée par simulation de touches. Les nouveaux lancements utilisent0.6.2. Les anciens clientsMCP déjà chargés ne sont pas déclarés rechargés ; aucun redémarrage de T3 ou de fournisseur.

<a id="extrait-2"></a>

## Extrait 2

- Dernier lotRust181 :19PASS. Transports3PASS ; HTTP13PASS ; scopesMCP4PASS ; contratT3 19PASS/1ignored. Régression155 :62PASS/1FAIL initial puis cas de corrélation isolé corrigé1PASS. Aucune FULL ni succès global inventé.
- JVM23PASS, lintAndroid et assemblages concernés PASS ; buildsrelease Linux/macOS/Android PASS. Un espace final dans le test vocal a été retiré avant commit ; diff staged vérifié.
- Émulateur : choix+complément, multiples, fermeture forcée/brouillon, réponse terminal, FCM réel, bouton de notification, texte RemoteInput, coupure et reprise mêmecommande, redémarrage du daemon puis réponse confirmée.
- Saturation :32reçusT3 conservés après rejeu ; notification après onze demandes actives désormais présentée et sa réponse confirmée ; seconde lecture d’un tombstone purgé ne réécrit plus les données. Assertions avant/après atteintes.
- Reconnaissance vocale réelle **non validée** : injection audio officielle a fait crasher QEMUAndroid16. Retour vocal contrôlé : texte éditable, aucun envoi automatique. Android15 sert aux autres essais ; aucun enregistrement du micro du Mac.
- AdaptateurT3 : worker réel et serveurHTTP/WS contractuel contrôlé. Pas de question réellement générée dans l’application T3 active revendiquée.
- Contre-revue plan : api_error. Code : demandeR1 refusée pour références sans host, corrigée ; R2 d3a4073d-bc4c-449b-b31a-3e047ab65a18 a dépassé180 s. Aucun verdict indépendant reçu. Pas de boucle de délégation, aucun APPROVE inventé.

<a id="extrait-3"></a>

## Extrait 3

| Composant | Vérification |
|---|---|
| Cartae | Noyau2600399, worker2600652, mobile2600610 ;0.6.2 chargée, exécutables installés et empreintes vérifiés |
| Mac | Connecteur26001 ;0.6.2 chargée, signature vérifiée |
| T3 | PID650/975 et dates de naissance inchangés ; aucune relance |
| Bases | quick_check ok sur les deux bases ; tâche historiquee595 inchangée par empreinte |
| Retour |0.6.1 vérifiée sur Mac/Cartae, politique rollback mise à jour et persistée après rechargement des seuls services de mise à jour |
| Android | APKrelease signé, même certificat,6002/version0.6.2 installé sur émulateur ; APKdebug et compagnon de test retirés |

<a id="extrait-4"></a>

## Extrait 4

Installations : /Users/moi/.local/lib/bridget-communication/181-dd1410f43dc2/bin/bridget et /home/moi/.local/lib/bridget-communication/181-dd1410f43dc2/bin/bridget . Versions de retour : /Users/moi/.local/lib/bridget-communication/180-21a16c374d71/bin/bridget et /home/moi/.local/lib/bridget-communication/180-21a16c374d71/bin/bridget . L’ancienne politique avait été réécrite par le watcher vivant ; arrêt normal/relecture de son état/rechargement corrigent ce problème de procédure, sans relancer T3 ni modifier du code fonctionnel.
