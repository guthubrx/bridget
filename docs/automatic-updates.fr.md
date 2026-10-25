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

Le fichier privé BRIDGET_HOME/update-host.json contient seulement host et services. Les rôles sont core/mobile/worker sur Linux, t3 sur macOS. Le rôle worker vise bridget-worker-cartae.service. Les services supervisés doivent utiliser ~/.local/bin/bridget. Python 3.9 ou supérieur est requis.

## Actions manuelles et état réel

« bridget updates status » affiche le choix et les observations datées des hôtes. « bridget updates check » demande une recherche. « bridget updates install 0.6.0 » demande cette version exacte. « bridget updates auto off » désactive l'automatisme et efface une demande manuelle encore en attente.

Un fichier téléchargé n'est pas une version active. Un client MCP déjà lancé garde son ancien code jusqu'à son rechargement par T3. Bridget ne redémarre pas T3 et ne copie aucune identité. Deux identifiants de build différents ne prouvent pas que le daemon est ancien.

Le nettoyeur conserve l'exécutable actif, un retour vérifié et toutes les copies encore utilisées. Un inventaire de processus incomplet interdit le retrait. Les autres installations, travaux, preuves et tâches en attente sont conservés. Les caches de compilation continuent de passer par scripts/build.py.

Le premier déploiement doit installer un binaire capable de gérer les mises à jour et configurer ses services. Il suit les mêmes précautions que les livraisons ordinaires. Le reçu de la session 179 distingue les versions installées, actives et encore différées.

Le droit de mise à jour est enregistré séparément des permissions historiques de l’appareil. Leur format reste lisible par la version de retour. Chaque appel vérifie toujours la révocation et l’expiration de l’association.
