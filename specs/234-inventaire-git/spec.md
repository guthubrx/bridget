# Session234 — inventaire Git borné et diagnostic précis

Correction autorisée par le signalement opus2D du5novembre2026. Deux préparations
surSYS échouent à compact_plan/git_timeout avant création du worktree.
Le journal historique ne conserve pas la commande : ne pas l’inventer.
Le dépôt contient94worktrees, sous la borne128 ; leur liste se lit en6ms.
L’inventaire actuel mesure4739540octets et se lit avec le Git worker en160ms,
puis668ms avec la cadence du lecteur natif. La latence initiale reste inconnue.

Un inventaire accepté jusqu’à64Mio doit disposer d’un budget propre adapté,
sans augmenter les délais des contrôles Git courts ou les échéances métier.
Une expiration doit nommer une opération fermée, son délai et ses mesures,
sans chemin, texte Git ou secret. L’arrêt natif reste obligatoire et borné.
Conserver les sources, les tâches, les clés et les travaux en cours ; aucune
admission métier ne sert de test. La mission déjà démarrée surCartae reste active.
