# Refus de précontrôle Git du 19 novembre 2026

Signalement Parix mcp-66364-6aff34b4-20. Demande
chef-parix-etude-uno-com-escada-20261119-1838, machine sysle2,
base 05cf004461cd4346c9291e3892af85a30d1382cb.

Les journaux maître et worker portent la même admission
`a-a0595be17d0c4b0f40af92b7de05fde3` :

| Heure UTC | Corrélation | Commande | Budget | Durée | Sortie retenue |
| --- | --- | --- | --- | --- | --- |
| 17:38:20 | 0599e8bc-a9cf-41f8-83e3-63eef8e9d83d | git worktree list --porcelain -z | 10000 ms | 10026 ms | 28672 octets |
| 17:39:18 | d5281952-5b85-4996-9856-e5250f643d0c | git worktree list --porcelain -z | 10000 ms | 10036 ms | 77824 octets |

Dans les deux cas : arrêt de l'enfant Git confirmé, phase de sonde `git`, aucun
contrôle fournisseur atteint. Le maître reçoit `workspace_unavailable` ; le
protocole actif ne transporte pas la sous-cause Git, donc `unobserved` reste
honnête dans ce reçu. Ce n'est pas un refus de disponibilité du modèle.

Une lecture actuelle de la même liste, à nice 19, réussit en 0,027 s :
379 worktrees, 81368 octets, code 0, aucun stderr. Ce relevé ne prouve pas la
cause du ralentissement historique. Le nombre de worktrees peut peser sur le
travail de Git, mais aucun plafond n'est attesté par ces deux refus. Le fichier
non suivi ne fait pas partie de l'étape ayant expiré. Aucun nettoyage de fichier,
worktree ou registre, aucune modification de mission, aucune admission d'essai.
La mission de remplacement e9011af2 a déjà été annoncée admise : ne pas doubler
son travail en rejouant automatiquement la demande refusée.

Correction du guidage : le refus générique indique les journaux et leurs clés
de corrélation, le diagnostic de dépassement de délai, et les prérequis avant
nettoyage ou rejeu. Il ne fabrique pas une cause absente du protocole et ne
requalifie pas un délai local en arrêt global. Le test conserve explicitement
`underlying_cause=unobserved` et vérifie le guidage de conservation.

Activation du guidage non attestée : le maître reste sur l'ancienne image ; le
blocage natif `update_missions_active_or_unconfirmed` a été constaté pendant le
lot 300. Aucun redémarrage ni nouvelle tentative de maintenance sans fait nouveau.

Contrôles : test dédié réussi. La première campagne de guidage a signalé
l'absence du recours explicite à l'utilisateur pour obtenir l'accès aux journaux ;
ce recours est ajouté sans changer la cause inconnue. Le formatage automatique
hors périmètre a été retiré après comparaison exacte avec le formatage de HEAD :
seuls les deux textes et le test de ce lot restent modifiés.

Validation finale : neuf tests de guidage et de taille de trame passent,
ainsi que Clippy strict bibliothèque/tests. Diff limité vérifié. Cache Cargo
à 5,87 Gio, sous le budget. Aucun temporaire distant, branche ou worktree créé.
Le code exécuté sur les serveurs reste inchangé ; livraison runtime incomplète.
