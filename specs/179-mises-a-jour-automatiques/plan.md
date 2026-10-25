# Plan 179
## Contexte
Rust/SQLite, services systemd/launchd, Kotlin/Compose, APK signé distribué directement. Un responsable, pas de sous-agent : délégation bloquée par la politique de ce tour. Travail dans le worktree 179 Cartae ; Mac pour compilation/installation et Pixel.
## Mise en œuvre
1. Étendre MobileRelease pour publier/lire une empreinte et vérifier un APK. Ajouter un contrôleur de mise à jour Android : préférence, tâche périodique, téléchargement borné, session PackageInstaller, état et reprise explicite. Réutiliser LocalStore pour les préférences/données persistées et Paramètres pour l'interface.
2. Ajouter un gestionnaire natif updates dans le binaire : publication, préparation atomique, configuration locale des services, exclusion d'activation, état et retour. Le gestionnaire périodique tourne comme service utilisateur et ne dépend d'aucun agent.
3. Ajouter un contrat humain fermé de réglage, état et fenêtre sûre d'activation. Barrière de travail dans le daemon : acquisition refusée si opérations ou missions actives/inconnues ; nouvelle activité bloquée pendant l'activation bornée. Les clients MCP utilisent le lien stable ; les processus chargés restent explicitement visibles.
4. Exposer le choix des services à l'application via l'authentification mobile existante et une capacité dédiée accordée à l'appareil de l'utilisateur. Le téléphone ne choisit ni URL ni commande d'installation des hôtes.
5. Distribution commune : versions strictement croissantes, manifestes/empreintes, paquets Mac/Linux et APK signé. Installer les services de mise à jour sur les deux machines avec préférence active. Garder une version de retour vérifiée, tous les binaires actifs et toutes les preuves.
## Vérification
Tests ciblés avant intégration ; vrais dépôts/HTTP/sockets/PackageInstaller quand utile, simulations clairement nommées pour panne et concurrence. UI réelle Pixel, persistance ON/OFF/ON et mise à niveau sans effacement. Tests de coupure et d'activité pour activation native, contrôle santé réel. Convergence et relecture du diff, pas de faux avis indépendant.
## Constitution et frugalité
Aucune dépense fournisseur. Aucun nouveau framework, aucune modification d'identité Git. Commandes Cargo via build.py. Nouveaux modules justifiés par responsabilités absentes, pas de doublon installateur/nettoyeur. Les archives sont bornées ; les scans de tâches/versions sont linéaires et limités au passage.
## Livraison
Commit conventionnel, intégration sur main à jour, push vérifié, activation sûre, retrait des branches/worktree propres de ce lot après conservation des preuves. Toute attente d'un composant est publiée avec motif et reprise automatique, sans maquiller l'activation.

## Clôture
Version 0.6.0 livrée le 25 octobre 2026. Paramètres automatiques actifs sur les trois plateformes. Les limites et conservations observées sont consignées dans le reçu ; aucun statut de tâche ni ancien client MCP n’est maquillé.
