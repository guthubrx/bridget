# Validation 179 — livraison du 25 octobre 2026

Base commune d6a2984f : sécurité 178 et mobile 177 conservés. Source publiée533f9ec60fe6f6d94868ac18974507a35c7f4c07.

## Contrôles effectués

- Gestionnaire Python : 13 PASS ; fichiers, archives et liens réels. Services/RPC simulés pour les pannes, l’exclusion concurrente et le retour arrière.
- Rust updates179 : 7 PASS, dont socket Unix réelle, garde humaine, révocation et conservation du format des permissions historiques. Rejoués après la séparation du droit de mise à jour.
- Rust build_info : 9 PASS.
- Rust mobile 177 : 69 PASS, 1 ignored préexistant.
- Android JVM : 18 PASS, aucun échec ni ignoré ; lintRelease et constructions Release/Debug/AndroidTest réussies avec le JDK178.
- Pixel Android 17 : 3 tests instrumentés PASS dans un paquet isolé. Les deux paquets de fixture sont désinstallés après recette.
- Compilation finale optimisée : Mac 1 min 48 s, Linux 1 min 38 s, succès. Toutes les commandes Cargo passent par le gestionnaire de caches.
- GitHub : six actifs publiés puis retéléchargés. Tailles et SHA-256 conformes à l’API et aux fichiers préparés. Binaires extraits conformes aux exécutables installés.
- Pixel : défaut ON, OFF local et global, fermeture/réouverture encoreOFF, retour ON. Installation finale0.6.0 depuis GitHub par PackageInstaller avec confirmation Android ; même APK signé que celui publié, association et favoris conservés.
- Mac/Cartae : services chargés en0.6.0, préférenceautomatic=true, surveillants actifs, observations des deux hôtes « à jour ». T3 et les missions existantes ne sont pas redémarrés.
- Diaporama : dix étapes réelles, rendu relu ; précédent/suivant/sélection testés, aucun message console d’erreur.

## Incidents et corrections

L’API178 ne connaissait pas la route de mise à jour. Son not_found est devenu un conseil update_hosts_unavailable ; le choix local reste enregistré. Après activation de 0.6.0, les paramètres des hôtes fonctionnent.

Le droit de mise à jour était d’abord inclus dans le format historique des permissions mobiles. Il est maintenant enregistré séparément pour garder la version de retour capable de lire les associations. Le test vérifie que les anciennes permissions restent identiques.

Pendant la recette Android, le job planifié a été déclenché explicitement pour ne pas attendre le regroupement du système. Play Protect a ouvert sa fenêtre ; la réouverture de Bridget a fermé cette fenêtre et Android a refusé la première tentative. Bridget a gardé0.5.0 sans boucle automatique. La reprise depuis le bouton de Bridget et la confirmation normale Android ont ensuite installé 0.6.0. Aucune protection système n’a été désactivée ; aucune installation finale par adb install.

## Limites vérifiées

- Délégation interdite dans ce tour : aucune contre-revue indépendante. Relecture directe des origines, signature, bornes, autorité, atomicité, course d’activation, reprise et nettoyage. Aucune FULL ni note globale de qualité.
- Installation Android entièrement silencieuse non validée sur ce Pixel. La cadence réelle de six heures n’a pas été attendue ; la planification persistante est présente.
- Les pannes d’activation/retour sont testées avec services simulés, pas en interrompant la production.
- Une tâche historique e5959986 reste en annulation non confirmée. Aucune exécution dans le cgroup des services ni commande worker active n’a été observée pendant le déploiement initial ; aucun état de tâche n’a été modifié. Le garde automatique reste conservateur et peut différer une prochaine activation.
- Les MCP déjà ouverts gardent leur ancien code. Le montage de ce tour répond encore identity_not_found ; aucune identité copiée ni relance de T3. Chemin stable configuré pour les prochains démarrages.
- Inventaire Cartae incomplet à cause de fuse.portal : caches, worktree 179 et branche conservés. Aucun retrait forcé.
