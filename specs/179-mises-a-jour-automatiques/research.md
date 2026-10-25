# Recherche ciblée
## Existant observé
Base0774dffc, mobile r10e conservé. MobileRelease.kt lit GitHub mais n'effectue qu'une comparaison de version et une ouverture navigateur. Paramètres/LocalStore/JobScheduler Android et installateur système sont réutilisables. Pas de gestionnaire automatique existant dans Bridget. scripts/install.sh vérifie les archives et conserve les autorités existantes ; scripts/build.py gère le budget de compilation. Dernière publicationv0.4.1, application installée0.5.0 : aucune rétrogradation possible.
## T3
Lecture du fork local/main-20261009 ac26460e : apps/desktop/src/updates/DesktopUpdates.ts et updateMachine.ts. Réemploi de disponibilité/téléchargement/activation distincts, exclusion d'actions concurrentes, reprise d'un téléchargement déjà prêt et contrôle de version attendue. L'activation Electron ne couvre ni les services Unix distants ni Android : interfaces natives nécessaires.
BridgetExecutable.ts/BridgetMcp.ts et ProviderSessionManager.ts relus au correctif174 : chemin installé stable et montage privé, déchargement après inactivité avec garde des tâches de fond. Ne pas remplacer cette autorité par une copie d'environnement.
## Sources primaires
- https://developer.android.com/reference/android/content/pm/PackageInstaller.SessionParams : auto-mise à jour de soi avec USER_ACTION_NOT_REQUIRED et UPDATE_PACKAGES_WITHOUT_USER_ACTION ; STATUS_PENDING_USER_ACTION reste possible.
- https://developer.android.com/reference/android/app/job/JobInfo.Builder : tâche périodique persistée, réseau requis.
## Décisions
Pas de nouvelle bibliothèque réseau ni nouveau parseur de version généraliste. Réutiliser minreq/sha2, HTTPS officiel et les paquets de la distribution. Sur Android, JobScheduler natif et PackageInstaller, sans bibliothèque de mise à jour. Préférence et état durables ; autorisation humaine dédiée pour modifier le choix des services. Activation fermée sur état inconnu. Un composant actif conserve son exécutable jusqu'à sa fenêtre sûre.
