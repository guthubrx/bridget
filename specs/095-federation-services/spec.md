# 095 — Fédération SSH permanente autonome

Branche : `session-095-federation-services` — 2026-09-07 — Statut : validée pour réalisation.
Demande : restaurer au plus simple la fédération historique et permettre au nouveau Bridget de créer ses propres services macOS/Linux sans ancienne installation.

## Scénarios utilisateur et tests

### US1 — Installer et gérer une liaison permanente (P1)
L'humain installe une liaison depuis la distribution actuelle sur macOS ou Linux, ferme son terminal et conserve la communication.
Acceptation : installation, statut et retrait fonctionnent avec le gestionnaire natif ; le retrait ne supprime ni historique ni clé ; aucun chemin d'exécution ne dépend du dépôt historique ou du worktree.

### US2 — Retrouver un seul annuaire Mac/Cartae (P1)
Le Mac reste maître et Cartae devient client du même annuaire par le lien SSH.
Acceptation : les UUID vus des deux côtés sont identiques ; deux clients attestés échangent dans chaque sens ; l'ancienne autorité Cartae est arrêtée après sauvegarde et autorisation humaine déjà reçue.

### US3 — Reconnexion sûre (P1)
Une interruption du tunnel est visible puis la supervision rétablit la liaison sans créer un second daemon.
Acceptation : refus de remplacer une socket vivante/étrangère ; récupération automatique d'une socket périmée uniquement attestée et privée ; reconnexion sans perte d'identité du client ni réinjection des messages.

## Exigences

- FR001 : fournir `install`, `status`, `remove` depuis le nouveau paquet, avec `launchd` ou `systemd --user` suivant l'OS.
- FR002 : réutiliser `run` et les gardes SSH actuelles ; conserver la vérification de clé d'hôte et l'absence de transfert d'identités SSH.
- FR003 : installer une copie autonome stable du lanceur, configuration privée explicite, aucune exécution de configuration comme code non validé.
- FR004 : service utilisateur uniquement ; pas de sudo, aucun daemon métier nouveau, aucun besoin GUI/Maicie/tmux.
- FR005 : collisions, types/permissions de fichiers inattendus et échecs d'activation restent des refus explicites ; pas de remplacement silencieux.
- FR006 : les retries du service sont bornés en cadence ; aucune boucle shell agressive.
- FR007 : préserver toutes les données historiques lors de la migration. Les services Maicie/GUI devenus incompatibles ne doivent pas redémarrer l'ancienne autorité.
- FR008 : indiquer explicitement le prérequis session utilisateur/linger pour la persistance Linux après déconnexion.

## Critères de succès

- SC001 : deux annuaires identiques et deux livraisons réelles reçues, une par sens.
- SC002 : liaison toujours disponible après fermeture du processus d'installation ; récupération après interruption testée.
- SC003 : les deux plateformes ont une preuve du cycle de vie de service ; distinguer mocks, supervision réelle et réseau réel.
- SC004 : aucun service installé ne référence l'ancien dépôt ni le worktree ; aucune donnée de conversation supprimée.

## Hypothèses et limites

Le maître Mac doit rester allumé et accessible ; ce n'est pas une réplication multi-maître. Les gérés lancés par le maître s'exécutent sur le maître : la fédération de communication n'est pas un ordonnanceur distant. Les clients Cartae s'exécutent sur Cartae et se connectent à la socket transférée. L'arrêt des anciens agents Cartae est autorisé, celui des sessions Mac ne l'est pas implicitement. La version 094 du binaire est déjà construite sur les deux OS ; cette session n'altère pas Rust.
