# 269 : refus prouvés et conservation des espaces préparés

Statut : décision retenue, qualification et activation suivies dans la session 269.

Un lancement distant peut préparer un worktree puis être refusé par le plafond
de flotte avant toute réservation de démarrage. Le reçu Fleet conserve ce fait.
Le parcours initial le masquait derrière `remote_spawn_not_new` et laissait la
tâche en file sans échéance. Deux tâches signalées ont depuis été annulées par
leurs propriétaires. Leurs commandes ne doivent pas être rejouées ni remplacées
tant que leur conservation n'est pas qualifiée.

Le parcours initial rend désormais le refus fermé `quota_exceeded` et le plafond
enregistré dans son reçu. Ce fait termine la demande en échec. Un résultat en
attente ou un démarrage incertain conserve son contrat antérieur. Le guidage
distingue ce plafond du CPU et de la mémoire, conserve la tâche et sa clé, et
précise qu'un export dans le terminal ne configure pas le service superviseur.
La maintenance ne change aucun plafond.

Pour un worktree distant terminal, une preuve durable de refus avant démarrage
permet de terminer la tentative de rapport par le parcours existant
« rapport indisponible ». La preuve exige la même commande, le même enfant et
le même placement, un refus de quota, aucune définition passée à Start, aucune
instance, aucune date de démarrage ou de remise et aucune reprise. Aucun appel
worker n'est fabriqué. Aucun fichier distant n'est lu depuis un chemin local.
Les faits Git restent inconnus et le dossier reste conservé. La clôture native
ne donne aucun droit de supprimer ce dossier. Toute preuve absente ou incohérente
maintient le blocage conservateur. Une vue de lecture avec sa liaison originale
peut enregistrer cette tentative de rapport indisponible, mais elle exige encore
l’accusé de conservation du worker avant `cleanup_done`. Le refus initial de
notre contre-revue a reproduit ce second cas. Aucune clôture locale ne remplace
cet accusé.

Le guidage pendant un tour est refusé avant remise pour une tâche dont la
définition figée atteste le transport `claude_stream_json`. Le refus suit
l'autorisation du parent et reste durable au rejeu. Le message explique la file
ordinaire en fin de tour et l'interruption à décider par le propriétaire. Aucun
repli ni interruption implicite. Le produit ne déduit pas ce transport du seul
nom du fournisseur.

Le wrapper traite aussi le rejet d'une livraison idempotente réellement en
attente, même si elle n'est pas la mission active. Il rend le reçu existant
d'issue indéterminée, sans accusé de remise, sans changer le tour ou l'exécution
active et sans réinjecter au rejeu. Les événements non corrélés restent ignorés.
Les anciens superviseurs métier ne sont pas interrompus pour charger ce code.

Les nouveaux caches de compilation sont créés privés avant Cargo. Les caches
existants gardent leurs droits. Les gardes de retrait, les observations des
processus et les verrous restent applicables. La qualification 269 désactive
explicitement les fichiers incrémentaux pour éviter l'accumulation mesurée sur
la machine de compilation.

Les recherches de reçus et de tâches sont indexées. La vérification d'une
livraison regarde uniquement la tête de sa file : coût constant. Aucun champ
durable ou protocolaire nouveau, aucune dépendance et aucun réglage par hôte.
