# Journal 120 - Se protéger d'une seconde application adaptateur de bureau

- **Base** : main `ae4b864e` - **Date** : 2026-09-25 - **Statut** : Implemented

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/120-garde-doublon-t3/implementation.md#extrait-1).

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
