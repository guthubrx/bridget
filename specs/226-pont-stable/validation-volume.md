# Lectures de gros historiques — préparation du chargement

État au 2 novembre 2026 : correction du serveur de l’application hôte développée,
intégrée et poussée. Paquet signé vérifié, mais application active encore ancienne.
La livraison de ce complément reste incomplète avant son chargement réel.
Aucune purge et aucune nouvelle commande de nettoyage développée.

## Source et correction

Dépôt de l’adaptateur hôte : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local.
Commit 7a024e9b866c46be6491059a1218b985df19f37d, intégré dans local/main-20261009
et poussé sur le distant fork. Branche fix/large-thread-command-reads également poussée.
La modification préexistante de pnpm-lock.yaml est conservée, sans ajout au commit.

Les lectures de types d’éléments et de nœuds filtrent par métadonnées avant de
charger les corps. L’ordre ne recherche que les identifiants de messages retenus.
Les messages utilisateur utilisent l’index partiel existant. La lecture fenêtrée
garde son index des nœuds actifs. Les prédicats et l’ordre des résultats restent
identiques ; le tri temporaire multi-types subsiste sur les résultats sélectionnés.

## Preuves et limites

Copie SQLite cohérente privée de 25 766 764 544 octets ; base active ouverte
uniquement en lecture. Aucun historique réel supprimé, compacté ou réécrit.
Mesures SQL, mêmes lignes et empreintes avant/après : éléments 11,52 → 1,26 s ;
nœuds 8,65 → 0,91 s ; messages utilisateur 5,20 → 0,526 s ; ordre ciblé 3,94 s → 5 ms.
Ce sont des mesures isolées sur disque externe, pas une promesse de temps complet.
Le vrai composant lit correctement cette copie ; le dernier temps complet est
10,69 s sous charge de construction et d’entrées-sorties. Mesure active à refaire
après chargement, sans rejouer une demande métier dont l’issue reste inconnue.

Migration unique : 230,85 s, trois index totalisant 313 208 832 octets, pris dans
les pages libres de la copie. Prévoir plusieurs minutes au premier démarrage.
Le runner enveloppe ses migrations dans une transaction ; une interruption ne
laisse pas de migration partiellement validée. Les noms/prédicats des index
partiels forcés sont présents dans les migrations 55/60 et ShellIndexes.
L’ancien manifeste accepte les identifiants futurs avec avertissement, sans
retirer les index ; retour applicatif additif, aucune suppression SQL requise.

512 insertions représentatives, 256 éléments et 256 nœuds, annulées après mesure :
+0,86 ms médian. 512 mises à jour de statut/corps, également annulées : +0,34 ms.
Cinq échantillons de chaque côté, premières lectures froides incluses dans les
reçus. Ces mesures ne qualifient ni le débit global ni le coût de synchronisation
d’une charge durable. Aucune autre limite de capacité n’est augmentée.

52 tests ciblés ProjectionStore/Orchestrator et huit tests de migrations acquis ;
format et lint des quatre fichiers corrigés passent. Le typage serveur rencontre
13 erreurs hors des fichiers corrigés ; aucune réussite globale de typage annoncée.
Deux revues indépendantes lues, acquittées et closes ; vérifications conditionnelles
levées par lecture du runner transactionnel, des migrations et des plans testés.
Aucune nouvelle campagne générale ni troisième ronde de revue.

## Paquet et reprise

Version locale : 0.1.0-local.20261102.volume. Les sources serveur/bureau ont été
assemblées. Le conditionnement standard rencontre un patch node-pty devenu
inapplicable lors de sa résolution de dépendances. Le paquet local conserve donc
le runtime installé et ses 95 fichiers natifs dépaquetés, vérifiés identiques,
avec les nouveaux assemblages. Intégrité ASAR et signature ad hoc renouvelées.
Démarrage réel du serveur empaqueté dans un dossier APFS privé vide : migration 65
et trois index vérifiés, processus proprement arrêté. Signature, empreinte de
l’archive et extraction du paquet vérifiées. Cela ne vaut pas activation du bureau.

Installateur natif préparé :
/Users/moi/.local/share/t3-local-build/releases/20261102-volume-7a024e9b/installer.command
Point d’entrée existant : /Users/moi/.local/bin/t3-local-build --install.
Il refuse l’application ouverte ou une image source différente, conserve l’ancienne
application, remplace atomiquement le paquet, vérifie la signature puis rouvre
l’application. Aucun accès de cet installateur à la base utilisateur.

Fermer complètement l’application hôte est nécessaire avant ce remplacement.
Le fil courant et les autres travaux sont conservés ; aucune fermeture forcée
ni arrêt d’un autre travail n’est effectué pour installer le candidat.
Après lancement : attendre la migration, reprendre le même fil, vérifier image,
migration et délai réel par une lecture ou une remise technique distincte.

Preuves privées et copie volumineuse conservées dans :
/Volumes/8TB2/50-repos-archives/11.Repositories/t3code/.worktrees/006-large-thread-command-reads/.t3/qualification.
Le worktree intégré reste conservé pour cette copie et la qualification à terminer.
Le candidat signé et la version active restent conservés jusqu’au chargement.
