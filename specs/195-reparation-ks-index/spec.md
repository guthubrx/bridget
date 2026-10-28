# Session195 — Déblocage opérationnel KS-STOR

Mandat : rendre les délégations Bridget sur KS utilisables et vérifier un résultat réel, sans perdre ni dupliquer les travaux.

## Fait établi
La file du worker183 sature pendant les contrôles Git. Sur le worktree bef1, une trace montre1,70s par status, dont1,63s dans refresh de1774entrées de métadonnées périmées. Le lecteur natif sans verrous facultatifs ne les actualise pas durablement.

## Réparation retenue
Actualisation ponctuelle des seules informations stat de l’index par Git update-index --refresh, avec hooks/fsmonitor/transport/maintenance désactivés. Aucun assume-unchanged, skip-worktree, sparse, changement de configuration/droits, checkout, déplacement ou retrait. Conservation de HEAD, de toutes les entrées mode/objet/stage et du chemin/device/inode. Sauvegarde privée de l’ancien index conservée ; aucune restauration manuelle automatique.

## Acceptation
La même lecture native devient plus rapide sans changer le résultat. Une délégation réellement routée sur KS lit un témoin inconnu de son brief et livre sa valeur exacte. La mission ancienne doit garder tâche/instance/clé ; toute reprise est observée, jamais recréée.
