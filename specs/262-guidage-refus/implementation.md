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

## Activation

Version 0.8.28 publiée, six actifs téléchargés sans authentification et empreintes conformes. Le paquet Android 8028 conserve sa lignée de signature ; il a été construit sur le serveur disposant de la chaîne qualifiée.

Les sept services concernés sur quatre hôtes chargent 262-guidage-portable-r1 : core, service mobile, quatre workers et raccord T3. Coupure mesurée du core : 2,17 secondes. Les formats aller et retour et les quatre pointeurs rollback ont été vérifiés contre les images réelles. La version de retour est 261-quota-refresh-r1. Les superviseurs des missions déjà actives restent conservés sur leur image 261 ; leurs processus et leurs marqueurs de naissance ont été vérifiés. Aucune mission métier n’a été rejouée.

La relance normale autorisée de T3 a rechargé ses façades MCP. Le nouveau processus serveur et les exécutables des façades sont vérifiés. L’identité native annonce la nouvelle capacité de diagnostic et bridget_status rend le build 262. La reprise automatique a été contrôlée ; les seules relances de maintenance devenues redondantes ont été retirées de la file.

Les retours aux deux demandeurs sont conservés avec leurs reçus. Le retour Menage est accepté ; celui de Parix reste en remise en vol au dernier contrôle, sans second envoi. La relecture stricte conserve le même identifiant, la date et le corps. La reproduction du refus de chemin protégé établit le mécanisme ; l’attestation historique complète n’a pas été retrouvée et son contenu n’est pas inventé.

## Ménage et conservations

Ménage exécuté dans les gardes : 58 anciennes copies exécutables retirées sur Mac, SYS et KS. Les versions actives et de retour sont conservées. Sur SYS, l’entretien natif après installation a retiré 6,40 Gio d’intermédiaires. Les exports et copies de compilation propres au passage ont aussi été retirés sur Mac et SYS, soit environ 770 Mo logiques.

Conservations précises :

- Mac : 15,77 Gio de caches, car trois processus Python rendent leur cwd illisible (ENOENT observé, PID33346, 52798 et 70824 lors du diagnostic). Aucun processus d’autrui n’est arrêté pour nettoyer.
- Cartae : 25 anciennes copies exécutables et les exports propres au passage, car l’observation complète après exclusion d’un montage FUSE est intermittente. Une observation positive isolée ne remplace pas le contrôle gardé du retrait ; les derniers passages refusés n’ont rien supprimé.
- SYS : une ancienne copie 246-reader-r1, car son dossier n’est pas privé et aucune observation globale privilégiée n’est disponible pour ce retrait.
- KS : une copie de transfert du binaire, car son dernier contrôle d’usage complet a été refusé.
- La vue de la revue indépendante reste soumise à sa conservation native. Les manifests et reçus anciens sont conservés, sans être assimilés à des exécutables inutiles.

Ces éléments sont à réexaminer lors de la prochaine livraison. Aucune garde n’a été retirée et aucun droit n’a été élargi pour faire passer le ménage.

Le refus trop vague du script d’entretien est aussi corrigé : il indique maintenant une cause fermée, le PID, le descripteur et son type, sans nom de fichier. Le cas réel de cwd illisible a été reproduit. La garde conserve la même décision de refus. Validation : 69 tests Python, dont 66 réussis et 3 cas d’observation réelle ignorés par leur garde, plus Ruff sans erreur. Ce script se charge à chaque invocation depuis le checkout aligné ; cette correction ne modifie pas l’image Rust.

Les reçus privés restent hors Git, dans le cache dédié à la livraison. La distribution locale conservée fait 34 525 944 octets. Les changements concernent des contrats et diagnostics techniques ; aucune interface graphique n’est modifiée.
