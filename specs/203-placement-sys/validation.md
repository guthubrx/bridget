# Validation203 — Configuration de placement SYS

## Cause réelle et préconditions
Incidents d6536518-9ac3-4636-af7d-7313082f04d3 et e2dd7089-e42f-452f-ac39-55c645fad11c : remote_start_failed:remote_workspace_prepare:DiskFull, aucune mission émise. /dev/sda4 monté sur / a7,2Gio libres ; /mnt/storage a6,7Tio. Le SHA49d905b81f4bd3128b6103abc307244650add44c représente20915entrées suivies/9921285268octets,9,24Gio ; inodes seulement8% utilisés. Le mapping révision1 dirigeait les extractions sous /home/moi/.local/share/bridget-projects/opus2d/repository. Le worker183 confond ENOSPC/EDQUOT : le sous-code historique exact reste inconnu, mais la capacité actuelle ne suffit pas au checkout complet.

Avant publication, lecture maître en SQLite query_only : aucune garde SYS ouverte. Inventaire worker : toutes commandes terminal/refused. Les deux tâches signalées sont cancelled/cleanup_done=true, aucune remise ; reçus et clés conservés. Cette situation diffère de la demande antérieure de placement167, alors limitée par des tâches encore actives. Aucun remplacement de mapping sous mission vivante n'est revendiqué. Cartae04b6633d est restée indépendante et a terminé normalement/result_available/errornull/cleanup_done ; aucune action dessus.

## Modification exacte
Projet natif opus2d, source_host monordinateur, source_root /Users/moi/Nextcloud/10.Scripts/69.opus2D conservés. Pour SYS seulement :
- remote_root /mnt/storage/bridget-projects/opus2d/repository
- expected_git_identity /mnt/storage/bridget-projects/opus2d/mirror.git
- mapping_revision2

Pas d'alias, pas de nouveau défaut implicite, aucune modification des autres mappings, modèles, permissions ou empreintes de credentials. Additional_root reste inchangé. Le dépôt prêt existe, UID1001, mode0755, aucun composant symlink ; son origin correspond à l'identitéGit annoncée. Il n'est ni déplacé ni recréé.

Trois fichiers d'autorité privés mis à jour : /home/moi/.cache/bridget-core/agents.json surCartae, /home/moi/.config/bridget/agents.json surSYS et /Users/moi/.cache/bridget-core/origin-agents177.json surMac. Chaque modification utilise le verrou existant project-preparation.lock, backup privé0600, comparaison des octets précédents, fichier temporaire0600 puis remplacement atomique/fsync. La comparaison après réinjection de l'ancien seul mapping prouve que le reste du registre est identique. Les identités/clefs privées ne sont pas copiées. Sauvegardes et reçus sous /home/moi/.cache/bridget203 et /Users/moi/.cache/bridget203, horsGit. Les registres complets ne sont pas publiés.

## Contrôles exécutés
Le helper NATIF scripts/prepare-project.py a réussi depuis l'origineMac configurée pourSYS/opus2d, sur99a7ed783687138f57cf14d353bb94a5b48df058 et49d905b81f4bd3128b6103abc307244650add44c. Le second SHA était absent du dépôt prêt ; le transport natif a vérifié/importé le bundle. Les deux objets commit sont maintenant présents. Le helper retourne dans les deux cas la destination /mnt/storage/bridget-projects/opus2d/repository et revision2. Aucun Task, admission ou fournisseur créé pour cette vérification. HEAD du dépôt conservé à8691b2189513541247cd40a4c6ec534a75a45ceb ; node_modules présent, aucun runtime réinstallé. Les fichiers de travail, dépendances et anciens worktrees ne sont pas modifiés par cette opération Git d'import.

Le préparateur et le worker lisent le mapping courant lors des préparations ; le maître relit son registre dans prepare_routing_projects et observe_routing_candidates. Aucun redémarrage, nouvelle compilation ou binaire203 requis. Maître202, workerSYS183 inchangés. Les gardes strictes de mapping restent en place : une ancienne admission figée sur revision1 ne gagne pas revision2. Les tâches annulées ne sont pas réanimées. La validation couvre la préparation Git et les configurations réelles ; aucune revue métier R52/triangulation n'est annoncée exécutée. Correction de configuration pure : pas de nouvelle revue de code ni testCargo sans changement de code.

## Parcours et notifications
Pour les futures demandes autorisées : bridget_auto_delegate avec machine_id=sysle2, cwd sourceMac habituel, projet opus2d (facultatif si résolution implicite unique), expected_base_commit égal à workspace.base_ref pour un worktree. Le mapping natif place désormais ses futurs worktrees sous /mnt/storage/bridget-projects/opus2d/repository/.worktrees. Le SHA choisi doit toujours être présent à l'origine ; le helper natif peut l'importer. Aucune nouvelle demande n'est créée par le mainteneur. Les clés annulées e2dd/d653 restent closes.

Politique sol : mcp-67950-6ae28ef9-2/issued1793232633/delivery368d21f9-e3bb-43b6-a8c6-ab2262b345c1. Trajetsbus : mcp-67950-6ae28efa-3/issued1793232634/delivery901844af-3f92-4432-a888-931ffd27006b. Acceptés in_flight ; aucune lecture déduite.

## Conservation
Aucun nouveau worktree source pour le rapport : réutilisation du checkout existant de202 sur branche203. Ancien dépôt/miroir/worktrees SYS conservés intégralement ; dépendances/runtimes et travaux actifs conservés. Aucun fichier métier supprimé, aucun serviceOpus modifié. La configuration est persistante pour cette machine/projet ; elle ne constitue pas un nouveau défaut de placement universel pour toutes installations vierges. Le binaire commun n'est pas changé.
