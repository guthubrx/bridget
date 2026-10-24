# Livraison mobile r10b — 24 octobre 2026

Statut : bilan historique r10b. Le complément courant r10c est dans r10c.md :
APK mis à jour, annuaire compact et détails des outils ; recette finale en attente
du déverrouillage du Pixel.
Le périmètre complet de la session 177 reste ouvert. Cette livraison ne déclare
ni la session terminée ni une validation globale de Bridget.

## Corrections regroupées

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

## Contrôles effectués

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

Une suite étendue antérieure r8 a donné 2231 PASS, 18 FAIL et 23 ignorés ; les
18 échecs ne sont pas classés comme hérités. Aucun succès global ni audit V15
complet n’est déduit des contrôles ciblés.

## Distribution et activité

Sources de départ communes : main `7ff749de884b6601f90f5ad6c28f2a879ec54bb1`.
Le présent commit contient les corrections et leur preuve partielle. Le manifeste
privé des sources relie les builds aux empreintes exactes de l’arbre de travail.
Les correctifs natifs intégrés par bridge1016 sont conservés.

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

## Preuves et tâches conservées

Diaporama : `preuves-corrections/2026-10-24-mobile-conversation/preuves.html`.
Captures réelles de cette conversation et du clavier, antérieures à r10b.
Couverture partielle des clics. Les outils repliés de la version finale ne sont
pas présentés comme vérifiés sur le téléphone. Originaux et manifestes hors Git.

À terminer dès que le Pixel redevient disponible :

1. Installer l’APK signé par mise à jour sans effacer les données.
2. Vérifier les groupes d’outils, le dépliage, les noms et le suivi continu.
3. Vérifier un message normal et « Transmettre maintenant » avec le même reçu.
4. Vérifier la notification réelle en arrière-plan et l’ouverture authentifiée.
5. Qualifier Wi-Fi/4G, fermeture/reprise, fichiers et actions de tâches.
6. Qualifier le fonctionnement depuis le serveur avec Mac éteint puis rallumé.
7. Terminer la correspondance exigences/preuves et la distribution complète.

Aucune de ces tâches n’est remplacée par un test de composant.

## Nettoyage effectué

La commande native `installed` a contrôlé les installations byte-identiques.
Mac : 0,48 Gio logiques d’intermédiaires retirés ; certains fichiers protégés
restent conservés. Cartae : observation des processus incomplète, cache de build
conservé sans forcer sa suppression. Aucun nouveau cache de compilation copié.

Cinq anciennes copies installées de binaires non utilisés ont été retirées après
contrôle des processus et services (deux Mac, trois Linux). La version active
r10b, le retour r10a et les workers R8 en cours sont conservés. Les bases, preuves,
clés et exécutables encore chargés ne sont pas supprimés. Le dépôt Cartae ne
comporte plus de worktree mobile secondaire ; la branche courante reste main.
