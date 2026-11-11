# Correction 262

Les corrections sont génériques : capacités négociées, codes fermés, chemins observés à l’exécution et identifiants complets. Aucun nom de machine ni chemin d’installation n’est imposé par le code ajouté. Les gardes de permission, les tâches et les admissions métier sont conservées.

Le refus de projection conserve désormais machine_path_protected, puis permission_path_protected sur le protocole négocié. Le budget du journal ne fait pas perdre la cause. Les refus distincts d’un lot restent séparés. Les anciens pairs continuent à recevoir permissions_changed.

Le contrôle Git conserve le dépôt source réellement détecté lorsqu’il ne peut pas résoudre HEAD. Un dépôt ancêtre n’est pas présenté comme hors Git. Un HEAD absent ne prouve pas une absence d’histoire. La CLI et l’admission partagent le même guidage. Le contrôle de version Git explique aussi le PATH de l’appelant, sans proposer de changement global.

Les expirations de réponse gardent l’identifiant complet et indiquent leur portée. Elles ne proposent plus de renvoyer une mission d’issue inconnue. Le corps destiné à l’agent et le contexte de la boîte humaine emploient le même formateur.

## Vérifications

Sur Mac et Linux : 8 nouvelles régressions, 8 contrôles voisins de guidage et 86 scénarios du protocole passent. Sur main Mac : analyse statique sans avertissement et 22 contrôles voisins 174 réussis. Le dossier temporaire court évite la limite des sockets Unix sur Mac ; Linux utilise le Git déjà qualifié du worker. Le test historique de preuve source supposait encore que le parent global ne pouvait pas changer de racine : la fixture accepte maintenant une autre racine absolue et conserve le refus de traversée de chemin. Aucun contrôle de production n’est affaibli.

Revue indépendante native : tâche b5bbf03a-217b-40e8-af50-3e2755ce2609, clé bridget-262-guidage-portable-review-20261111-r1. Compléments traités : notification humaine, raisons mixtes et CLI. Le diagnostic conserve toutes les causes même après saturation du journal. La conversion immédiate des candidats évitait déjà leur écrasement entre deux itérations ; l’ensemble borné rend aussi la conservation explicite et testable.

## Source de l’exécutable

La décision D1 de la session 260 réserve certaines extractions à une activation ultérieure. Le candidat 262 utilise donc la base active 75259217a72b3177cc0f50ba58c1fc3f97e5e22e et le patch exact runtime.patch. Le script build-runtime.py reconstruit cette source dans un nouveau dossier, sans branche ni worktree. La reconstitution des 14 fichiers modifiés a été comparée octet pour octet au candidat compilé. La capacité annoncée reste dans daemon.rs de cette base ; son ajout équivalent figure dans wrapper_dispatch.rs sur main.

Version : 0.8.28. Identité de compilation : 262-guidage-portable-r1.

Activation et ménage : en cours, reçus privés conservés hors Git. Ces changements concernent les contrats et les diagnostics techniques ; aucune interface graphique n’est modifiée.
