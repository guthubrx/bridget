# Livraison et nettoyage du 10 novembre 2026

La demande de livraison concerne le travail de cette conversation. Le correctif 259 était déjà intégré et poussé : source exécutable 406466cb, clôture documentaire b3949c93. Au début du contrôle, main était propre et aligné au distant sur ca1e203c. Aucun changement exécutable supplémentaire de ce fil n'attendait une fusion. Les diagnostics ultérieurs restent dans leurs dossiers privés de preuves ; ils n'ont modifié ni code ni mission métier.

## Retraits effectués

Quinze anciennes images Mac, sources intégrées et empreintes vérifiées, n'avaient plus de processus utilisateur selon les lectures lsof complètes. Elles ont été retirées : 243-tests-r5, 243-tests-r6, 244-reference-r5, 246-quota-accord-r2, 247-coordination-r1, 248-messages-en-cours-r2, 249-guidage-brief-r1, 250-resultat-natif-r1, 251-attention-r1, 251-attention-r2, 252-guidance-r1, 253-native-project-r1, 253-native-project-r2, 254-local-preparation-r2 et 254-local-preparation-r3. Total logique : 551 800 752 octets.

Une copie APK de transfert de 3 967 548 octets a aussi été retirée. Son empreinte est identique à celle du fichier conservé dans les archives publiques qualifiées. Total des retraits de ce passage : 555 768 300 octets logiques. Ce total ne mesure pas l'espace physique libéré sur APFS.

La référence distante périmée github/session-259-guidage-preflight a été purgée par fetch --prune. La branche distante réelle et le worktree Mac de ce lot avaient déjà été retirés lors de la livraison initiale. Aucune autre branche distante de travail n'est publiée.

## Entretien et conservations

L'entretien natif scripts/build.py clean a été exécuté sur Mac, SYS et Cartae. Aucun cache supplémentaire n'était éligible : Mac 9,06 Gio, SYS 5,54 Gio, Cartae 0,00 Gio dans les périmètres reconnus. Le cache Mac a augmenté depuis le relevé de clôture 259, pendant les travaux suivants ; il reste sous le budget de 10 Gio. Aucun target n'a été copié ou vidé globalement. Le nettoyeur conserve les worktrees quand son observation des fichiers ouverts est incomplète.

Les dossiers suivants restent conservés :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/260-lot00-fondations : travail distinct, commit 2316a179 non intégré et fichiers ignorés. Sa branche session-260-lot00b-fondations est conservée. Aucun travail de la session 260 n'a été fusionné ou effacé par cette livraison.
- /home/moi/.local/share/bridget-projects/bridget/repository/.worktrees/259-guidage-preflight : cinq processus du compte restent inobservables. Aucun usage direct trouvé ne prouve pas l'absence d'usage.
- /home/moi/10.Scripts/64.bridget/.worktrees/259-android : 68 processus restent inobservables au nouveau relevé.
- Les worktrees 258, vues natives, résultats, reçus, archives de publication et fichiers ignorés de provenance incertaine restent conservés.

Huit images Mac restent présentes : actif 259, retour 258, lecteur 245 gardé comme témoin de qualification négative et cinq images anciennes sans provenance complète dans les reçus consultés. Ces cinq images sont 244-reference-r3, 244-transport-return-r2, 244-transport-return-r3, 246-reader-r1 et 246-quota-accord-r1. Leur suppression n'a pas été déduite du seul fait qu'elles étaient inutilisées au relevé.

## Vérifications de fonctionnement

Le watcher de mise à jour Mac possède son verrou pendant toute sa vie. La première tentative de nettoyage sous verrou a donc refusé sans aucune suppression. Seul ce watcher a ensuite été arrêté par launchctl bootout, puis relancé dans un finally par launchctl bootstrap. Ancien PID 93305, nouveau PID 34628. Aucun processus d'un autre travail n'a été terminé pour nettoyer.

Les deux services Mac conservent leurs PID 92824 et 93104 et leur empreinte 259. Les cinq services Linux conservent leurs empreintes attendues sur SYS, KS et Cartae. Les sept services chargent toujours 259-preflight-guidance-r1. Le retour 258 et ses quatre pointeurs sont vérifiés ; aucune activation n'est en cours. Aucun redémarrage de l'application T3 ou rejeu de mission n'a été effectué.

Le watcher Mac conserve un état client_check_unavailable rattaché à une génération de contrôle antérieure. Le contrôle local clients check effectué pendant ce passage rend compatible pour les quatre clients installés. Le code du watcher conserve volontairement son blocage jusqu'à une nouvelle génération de contrôle ; aucun marqueur ni droit n'a été réinitialisé à la main. Ce rapport ne déclare donc pas son ancien contrôle automatique résolu. Les exécutables actifs et leur retour sont vérifiés séparément.

Les tests ciblés 259 et la publication déjà qualifiés restent valides ; aucun code exécutable n'a changé dans ce passage. Le bilan documentaire est contrôlé par git diff --check. Aucun nouveau build ou test fournisseur n'est nécessaire pour cette seule clôture et ce nettoyage.

Preuves détaillées : /Users/moi/.cache/bridget259-guidage-preflight/cleanup-delivery-final.json, /Users/moi/.cache/bridget259-guidage-preflight/cleanup-apk-transfer.json, /Users/moi/.cache/bridget259-guidage-preflight/cleanup-updater-restart.json et /Users/moi/.cache/bridget259-guidage-preflight/cleanup-runtime-mac.json.
