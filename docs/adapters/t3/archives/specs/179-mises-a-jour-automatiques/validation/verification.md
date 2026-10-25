# Archives techniques de l’adaptateur T3

Document d’origine : [specs/179-mises-a-jour-automatiques/validation/verification.md](../../../../../../../specs/179-mises-a-jour-automatiques/validation/verification.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

- Délégation interdite dans ce tour : aucune contre-revue indépendante. Relecture directe des origines, signature, bornes, autorité, atomicité, course d’activation, reprise et nettoyage. Aucune FULL ni note globale de qualité.
- Installation Android entièrement silencieuse non validée sur ce Pixel. La cadence réelle de six heures n’a pas été attendue ; la planification persistante est présente.
- Les pannes d’activation/retour sont testées avec services simulés, pas en interrompant la production.
- Une tâche historique e5959986 reste en annulation non confirmée. Aucune exécution dans le cgroup des services ni commande worker active n’a été observée pendant le déploiement initial ; aucun état de tâche n’a été modifié. Le garde automatique reste conservateur et peut différer une prochaine activation.
- Les MCP déjà ouverts gardent leur ancien code. Le montage de ce tour répond encore identity_not_found ; aucune identité copiée ni relance de T3. Chemin stable configuré pour les prochains démarrages.
- Inventaire Cartae incomplet à cause de fuse.portal : caches, worktree 179 et branche conservés. Aucun retrait forcé.
