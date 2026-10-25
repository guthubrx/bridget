# Livraison 180 — 25 octobre 2026

Statut : livré avec conservations justifiées. Version active : 0.6.1.

## Résultat livré

La flèche normale enregistre le message. Pendant le travail du destinataire, sa file durable apparaît dans un volet repliable au-dessus de la saisie. Chaque message retenu peut être modifié, annulé ou transmis en guidant le tour actif. L’ancien lien préalable est retiré. Les actions incertaines restent visibles et se rejouent avec la même clé. Une remise commencée n’est jamais présentée comme annulée.

Source compilée : 21a16c374d71319155b13728261d62535dedfacf, intégrée et poussée sur main. Les documents de clôture sont dans un commit distinct. Aucun tag ni release GitHub 0.6.1 publié par cette livraison ; le déploiement des trois plateformes est direct. La dernière release GitHub reste 0.6.0.

## Installation et santé

| Composant | Observation après installation finale |
|---|---|
| Core Cartae | PID2153657, actif, build180-21a16c374d71 |
| Worker Cartae | PID2156728, actif, même exécutable final |
| Mobile Cartae | PID2153921, actif, même exécutable final |
| Connecteur Mac | PID46375, chargé depuis le répertoire180-21a16c374d71 |
| Pixel | Bridget0.6.1, versionCode6001, association conservée |
| T3 | PID650/975 et dates de naissance conservés, aucune relance |

Mac signé : SHA-256 1f011009873b342bbb3b8ce065d7879d131cc237e3def3c4194333586f24f1af.
Linux : SHA-256 4d12c54d189f92a1a59f277c9b7bf5821ce83da659bf3b18b4cb79d5d5eb32fa.
APK signé : SHA-256 9b6d9b10c01fe8fd8717b9f4bef4e85cfefebd3d82505d36e3bc7f2edcfabf2f.
Certificat Android inchangé : 0c43c621254d9a6e0dbe148466f453f1aaba1a62bf23bd7332c61086f16a6588.

L’APK reconstruit depuis le commit final est byte-identique au candidat installé et testé. Le paquet installé a été relu par USB et son empreinte correspond. Aucune nouvelle installation inutile. Les binaires natifs définitifs ont été reconstruits depuis le commit final, puis installés hors staging. Le binaire Mac du staging a été signé avant copie ; son empreinte correspond à l’installation.

Bases bridget.db et mobile-devices.db : quick_check=ok, zéro violation de clé étrangère. Core et mobile sont stables. Le worker a repris six fois sa connexion pendant la barrière de maintenance, puis son PID et son compteur sont restés stables après libération. Les surveillants de mise à jour restent actifs. Leur référence de retour désigne désormais la version0.6.0 vérifiée sur chaque machine ; l’état antérieur est archivé.

## Activation contrôlée

La tâche historique e5959986-e426-422d-956f-e035dbcf89be reste cancelling, guarded=false et mission_issued_at=null. Son payload conserve l’empreinte 650b6528636b25cc3954f9e1907abca0b185d4af564f77f96192c6e2a020b3f3. Aucun état de tâche corrigé artificiellement. La préparation automatique reste soumise au refus update_missions_active_or_unconfirmed jusqu’à réconciliation.

La maintenance manuelle réemploie la procédure179 : contrôle des admissions et commandes worker, sauvegardes, arrêt propre du mobile/core, vérification des cgroups sans enfant de mission, arrêt worker, barrière privée bornée180s, liens stables commutés, reprise et contrôle de santé Mac/Cartae, libération de la barrière. Aucun fournisseur ni T3 arrêté. Les reçus et sauvegardes de l’installation candidate restent distincts de ceux de l’installation finale.

## Vérifications et preuves

Rust : lot mobile77PASS/1ignored, guidage6PASS ; six tests180 finaux PASS. Android JVM20PASS/0échec, lintRelease et assembleRelease réussis. Pixel : QueueStoreTest instrumenté1PASS ; paquet de test distinct ensuite désinstallé. Revue indépendante terminée : référence et limites dans revue.md. Aucune suite globale ni qualification globale déduite.

La recette Pixel emploie un destinataire natif contrôlé, sans modèle. Trois messages : édition puis annulation sans remise ; interruption reçue une seule fois avec steer_current ; message normal remis au retour au repos pendant que l’application était arrêtée. Le témoin contient deux IDs uniques, la base un cancelled et deux acked. Aucun message de test dans les vrais fils utilisateurs. Le guidage Codex/T3 est vérifié par les tests ciblés et la lecture de l’adaptateur, pas par une nouvelle interruption d’un vrai tour utilisateur. Les filtres/favoris ont été remis et Bridget laissé sur le fil courant.

Preuves : diaporama vérifié,13étapes,7clics relevés. HTML autonome338660octets inclus dans le même commit que le correctif. Avant de l’ancien lien non capturé ; cette limite est affichée. Les captures originales, manifestes, journaux et témoins natifs restent hors Git. Les22fichiers de rapports Android sont archivés et relus byte à byte ; archive SHA-256 096706673c6ae918b2af0db57d7153dbda8c87c029b670baf4b9fc01e08c691f. Le contrôle de secrets du commit source n’a trouvé aucun secret.

## Nettoyage effectué

- Mac : worktree temporaire de compilation retiré sans force. Les23fichiers modifiés/non suivis ont été comparés à main ; seul l’ancien avancement des tâches différait, puis a été archivé. Aucun code distinct perdu. Trois copies d’APK de validation supprimées :35372280octets logiques.
- Cartae : quatre anciennes installations174/177/178/180-candidate retirées après un inventaire privilégié complet des fichiers ouverts et contrôle des références de service. Version finale et retour0.6.0 conservés. Les artefacts Gradle/Android180 devenus inutiles ont été retirés après archivage vérifié des rapports. Total retiré406726549octets logiques, binaires et artefacts Android compris.
- Le serveur HTTP temporaire des preuves a été arrêté proprement ; port41780 libéré. Le destinataire de recette était déjà fermé proprement.
- Entretien Cargo effectué avant/après compilations puis commande installed exécutée sur les deux machines. Cartae conserve8,03Gio d’intermédiaires reconnus, sous le budget10Gio ; Mac environ1,88Gio. Aucun gain physique déduit des tailles logiques.

## Conservations précises

- Mac : le nettoyage du staging installé a refusé une observation globale incomplète. Un processus d’un autre projet, PID53789 au contrôle, a un répertoire courant et deux descripteurs révoqués. Il n’a pas été arrêté. Les intermédiaires Rust restent conservés, sans contournement du nettoyeur.
- Mac : les installations178 et180-candidate restent chargées par des clients MCP. Elles sont conservées en plus de l’actif et du retour0.6.0. Aucun rechargement des clients existants revendiqué. Les quatre anciennes sauvegardes Git non qualifiées restent protégées.
- Cartae : le contrôle initial a permis le retrait des quatre binaires et des artefacts Android. Le contrôle suivant du cache Rust est devenu incomplet à cause de descripteurs /proc disparus pendant la lecture, notamment dans dockerd. Aucun autre processus arrêté pour obtenir un nettoyage. Les intermédiaires Rust180, son worktree et sa branche sont conservés pour ce motif, propres sur les sources finales.
- Les worktrees177 et178 restent liés à des tâches ouvertes (21 et1). Le worktree de contre-revue177 n’a pas de clôture qualifiée. Le worktree179 garde des fichiers ignorés et reste protégé par le contrôle incomplet. Leurs branches intégrées restent attachées aux worktrees. Aucun travail non intégré ni preuve supprimé.
- Aucun branchement de travail distant180 n’a été créé. Les références distantes sont actualisées ; le contrôle Git final est archivé hors Git pour distinguer le commit documentaire du commit source et éviter une auto-référence circulaire.

Reçus privés : /Users/moi/.cache/bridget-mobile180 et /home/moi/.cache/bridget-mobile180. Le contrôle final des main propres et alignés est dans git-final180.json sur les deux machines.
