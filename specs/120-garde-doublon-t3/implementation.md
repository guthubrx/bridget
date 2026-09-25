# Journal 120 - Se protéger d'une seconde application T3

- **Base** : main `ae4b864e` - **Date** : 2026-09-25 - **Statut** : Implemented

## Incident déclencheur (05:18Z)
Doublon `T3 Code (Alpha).app` (pid 8275, serveur 8352, port 3774) ouvert par un agent via `getApp`.
Tours de GLM3 (`dfccceef`) et GLM4 (`106521cc`) marqués en erreur à 05:18:59Z, nouveaux tours
`pending` sans `started_at` ; processus Claude 1286 et 8742 (enfants du serveur Local 22214) actifs
d'après leurs transcrits. Pont resté sur 3773 (garde 114), mais appariement impossible. Doublon
quitté par l'agent ; fichier d'état réécrit à la main.

## Correction
- La déclaration de notre serveur est lue au démarrage du pont et liée au `ServerRuntime` qu'elle
  décrit ; `restore_declaration` la réécrit si le fichier manque ou désigne un serveur mort et que
  `server_alive` (PID vivant et port qui accepte) le confirme.
- À chaque nouveau doublon : `notify_user` via `/usr/bin/osascript`, dans un fil détaché, jamais en
  test ; `app_bundle` lit le chemin par `proc_pidpath` pour nommer le paquet `.app`.
- Écarté : nouvelle catégorie de la boîte humaine (catégories fermées, migration SQL
  disproportionnée) ; fermeture automatique (accord humain requis).

## Vérifications
- Voir tasks.md T005. Notification de test envoyée depuis le terminal (code 0) ; affichage à
  confirmer par l'utilisateur, notamment depuis le contexte launchd du pont.
