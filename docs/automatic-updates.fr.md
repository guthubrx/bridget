# Mises à jour automatiques

Le réglage est actif par défaut. Dans Réglages sur Android, « Mises à jour automatiques » couvre le téléphone et les hôtes Bridget Mac/Cartae autorisés. Le choix désactivé est conservé après fermeture. Si le serveur est inaccessible, l'application affiche que son choix attend confirmation.

## Ce qui est téléchargé

Bridget lit la dernière publication stable du dépôt GitHub officiel guthubrx/bridget. Une publication contient une même version et plusieurs fichiers adaptés : archive macOS ARM64, archive Linux x86_64 et APK Android signé. Une plateforme absente est signalée ; aucun paquet d'une autre plateforme n'est utilisé.

La version doit être strictement supérieure. Les versions de développement, publications provisoires et identifiants de build ne servent pas à décider d'une mise à jour. Le code poussé sur main n'est pas installé tant qu'une publication complète n'existe pas. Les empreintes SHA-256 et tailles proviennent de l'API GitHub ; leur absence bloque l'installation. Le compte qui publie les releases reste une autorité de confiance.

## Android

La recherche périodique a lieu environ toutes les six heures. Android peut la différer pour économiser la batterie ou attendre le réseau. Une ouverture de l'application permet aussi une recherche quand elle est due.

Le téléchargement reste privé. Bridget vérifie le fichier, l'application, sa version et son signataire avant de passer par l'installateur Android. L'installation automatique attend la fermeture de l'interface et la fin des opérations. Si Android demande l'autorisation d'installer depuis Bridget ou une confirmation, le bouton affiché ouvre le parcours système. Aucune permission n'est contournée.

Le réglage désactivé arrête les nouvelles installations automatiques. Une installation déjà engagée dans Android peut toutefois finir. Une version dont l'installation a échoué n'est pas engagée à nouveau automatiquement. La recherche et l'installation manuelles restent disponibles. L'association, les brouillons et les reçus sont conservés. Android ne revient pas automatiquement à une version inférieure.

## Mac et Cartae

Le service utilisateur lance « bridget updates watch » depuis le lien stable du client. Il ne dépend d'aucun agent. Il vérifie les releases toutes les six heures et réexamine une mise à jour préparée chaque minute.

Avant de remplacer un service, il demande une fenêtre exclusive au daemon. Une mission, une admission ou une opération active bloque cette fenêtre. Un arrêt non confirmé la bloque aussi. Les nouveaux appels reçoivent update_in_progress pendant la bascule. Les hôtes s'activent l'un après l'autre.

Après bascule, le programme vérifie les services et l'exécutable réellement chargé. En cas d'échec, il restaure la version précédente puis vérifie le retour. Une coupure de réponse après libération de la fenêtre ne déclenche pas de retour arrière tardif.

L'appareil Android doit recevoir le droit de gérer les mises à jour depuis l'administration humaine. Cette capacité n'est pas ajoutée aux agents ni automatiquement aux autres appareils. Les paramètres de mise à jour ne permettent pas d'envoyer une URL, un chemin ou une commande arbitraire.

[Détail du contrat de l’adaptateur](adapters/t3.md#note-52).

Le programme stable est `~/.local/bin/bridget`. Le surveillant intégré utilise Python3.9 ou plus et la commande `bridget updates watch`. Son fichier privé `BRIDGET_HOME/update-host.json` contient `host` et `services`. Les rôles Linux sont `core`, `mobile` et `worker`. Le service worker est `bridget-worker-cartae.service`. Les services supervisés utilisent le lien stable. La configuration ne contient aucune commande téléchargée.

## Actions manuelles et état réel

« bridget updates status » affiche le choix et les observations datées des hôtes. « bridget updates check » demande une recherche. « bridget updates install 0.6.0 » demande cette version exacte. « bridget updates auto off » désactive l'automatisme et efface une demande manuelle encore en attente.

[Détail du contrat de l’adaptateur](adapters/t3.md#note-53).

Le nettoyeur conserve l'exécutable actif, un retour vérifié et toutes les copies encore utilisées. Un inventaire de processus incomplet interdit le retrait. Les autres installations, travaux, preuves et tâches en attente sont conservés. Les caches de compilation continuent de passer par scripts/build.py.

Le premier déploiement doit installer un binaire capable de gérer les mises à jour et configurer ses services. Il suit les mêmes précautions que les livraisons ordinaires. Le reçu de la session 179 distingue les versions installées, actives et encore différées.

Le droit de mise à jour est enregistré séparément des permissions historiques de l’appareil. Leur format reste lisible par la version de retour. Chaque appel vérifie toujours la révocation et l’expiration de l’association.

Un fichier téléchargé n’est pas une version active. Un processus MCP conserve son code chargé jusqu’à un remontage pris en charge. Conserver les credentials de session. Deux identifiants de build différents ne prouvent pas à eux seuls que le daemon est ancien.

## Vérifier les clients avant la bascule (209)

Le candidat vérifie les programmes locaux avec bridget clients check, aussi disponible en JSON avec --json. Il ne teste ni compte ni accès aux modèles et ne lance aucune mission. Le registre inspecté figure dans registry_file : registre du namespace, puis registre natif worker par défaut s’il n’existe pas. Une installation personnalisée peut désigner son registre privé par BRIDGET_CLIENTS_REGISTRY=/chemin/absolu/agents.json. Ce choix ne modifie aucun fichier.

Cette release prend en charge Claude Code à partir de2.1.280 et Codex à partir de0.162.1. Le premier seuil provient du refus de modèle attesté dans208 ; le second est le schéma de référence des demandes enfant155. Ce sont des minima de support Bridget, pas une promesse sur tous les modèles futurs.

Avant tout arrêt, remplacement de lien ou activation, l’installateur et l’updater interrogent **le candidat**. Une version installée trop ancienne produit client_update_required ; un contrôle impossible produit client_check_unavailable. La machine, le programme, la version observée, le minimum et la décision attendue sont fournis dans le rapport. Le statut d’update conserve ce diagnostic dans client_compatibility. L’absence d’un client reste distincte : Bridget peut être installé seul, mais ce client reste indisponible.

L’accord aux mises à jour Bridget n’autorise pas les mises à jour de Claude ou Codex. Le message demande un accord distinct. Après cet accord, utiliser le mécanisme officiel du gestionnaire déjà installé, préserver les comptes et modèles, refaire le contrôle puis qualifier un lancement réel autorisé. Aucun programme fournisseur n’est installé ou mis à jour par ce contrôle. Une commande opaque ou une version non reconnue reste non vérifiée.

Les tâches en cours continuent avec leurs versions déjà chargées. Un refus de compatibilité suspend seulement la bascule du candidat sur cette machine. Un ancien candidat qui n’expose pas ce contrôle reste non vérifié ; aucune validation automatique fictive.

La politique de support est versionnée et porte la provenance de chaque minimum. Le validateur installé contrôle le rapport du candidat contre l’inventaire local complet avant toute réservation. Les minima futurs peuvent augmenter, jamais descendre sous le plancher connu ; chaque ligne doit reprendre exactement la politique annoncée. Une sonde ignorant TERM rend probe_stop_unconfirmed avec son PID. Aucun SIGKILL n’est émis ; l’updater suspend les nouvelles sondes jusqu’à une décision explicite et une nouvelle commande bridget updates check.

Transition : le paquet public historique0.6.3 ne possède pas ce port. Son installation manuelle conserve son comportement historique avec un avertissement explicite, sans affirmer la compatibilité des clients ; --source installe la source actuelle avec le contrôle. Ce passage ne s’applique jamais à un refus du nouveau contrôle ni à une version future dépourvue du port. L’updater209 refuse les candidats non vérifiables avant toute activation.

L’hôte système et machine_id restent distincts. Un registre worker dédié portant une unique identité provisionnée utilise cette identité pour son inventaire, même si le nom système diffère ; aucune équivalence DNS n’est devinée. Un périmètre impossible à établir reste non vérifié.
