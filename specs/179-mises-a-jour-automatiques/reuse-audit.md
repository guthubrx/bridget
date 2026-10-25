# Audit de réemploi179
Verdict : PASS pour les responsabilités ci-dessous.
| Proposition | Existant lu | Décision |
|---|---|---|
| Métadonnées/version Android | MobileRelease.kt | Étendre, pas de second client |
| Préférence/état Android | LocalStore.kt | Réutiliser les entrées chiffrées |
| Paramètres et messages | MainActivity.kt, MobileViewModel.kt, strings FR/EN | Étendre |
| Téléchargement/installation APK | Aucun installateur interne | PackageInstaller natif, contrôleur ciblé |
| Relève Android | Notifications, pas de tâche de mise à jour | JobScheduler natif dédié |
| Distribution Mac/Linux | scripts/install.sh, package-089-core.sh | Réutiliser noms d'actifs, empreintes et frontière d'autorité ; pas d'appel aveugle à l'installateur |
| Gestionnaire automatique | Aucun équivalent dans crates/ ou scripts/ | Module updates et service utilisateur dédiés |
| Configuration et droits mobiles | mobile/store.rs, mobile/http.rs, daemon/mobile_admin.rs | Étendre la portée humaine, pas de droit d'agent |
| Fenêtre sûre | Mutex daemon, tick natif, états de tâche/fleet | Barrière d'activation, observations existantes ; pas de SQL parallèle écrivain |
| Nettoyage | scripts/build.py | Réutiliser ; ne pas créer un second nettoyeur |
| Comparaison de version affichée | build_info.rs | Corriger « périmé » : un écart de build n'atteste pas l'ordre |
| Modèle de mise à jour | T3 DesktopUpdates/updateMachine | Réemploi du cycle, adaptation aux services distants et APK |
## Gate avant tasks
- [x] Plan et sources relus.
- [x] Chaque nouvelle responsabilité comparée à l'existant.
- [x] Aucun doublon évident laissé sans décision.
- [x] Aucun contrôle d'autorité assoupli.
