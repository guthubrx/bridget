# Plan 264

## Réemploi et périmètre

Bridget Loop existe dans la compétence utilisateur, hors du dépôt Bridget. Son contrôleur autonome utilise déjà le client SendIdempotent et conserve son enveloppe. Le daemon possède une garde commune pour steer/interrupt, des quotas, une pause et des transports natifs. Réemployer ces points ; ne créer ni planificateur ni transport.

Ajouter une autorisation de contrôle spécifique à une boucle. Elle lie une portée de remise, un projet, une liste bornée de cibles et un mode. Son secret reste dans un fichier privé distinct du registre public. Une connexion cliente peut présenter cette autorisation, sans devenir Attach et sans s'attribuer une filiation. La garde relit le mandat à chaque nouvel effet. La révocation invalide aussi les connexions déjà ouvertes. La pause et la protection des missions natives actives restent en place.

Le mode ordinaire ne nécessite pas de mandat. Le CLI de création demande les trois choix ; hors terminal, un choix explicite est obligatoire. Les modes de contrôle exigent les cibles désignées. Les nouvelles remises figent leur mode et la référence privée avant la première tentative. Les anciennes remises sans mode gardent queue.

## Vérification et livraison

Tests Python de création, prompt, transport et enveloppes ; tests Rust de protocole, mandat, portée, révocation et garde ; recette avec daemon et cibles isolés. Construire par scripts/build.py. La base exécutable 263 est reconstruite séparément du refactoring 260 réservé à une activation ultérieure ; ne pas activer ce refactoring par ce raccord. Contrôler les processus chargés et le rollback sur chaque hôte. Préserver les travaux concurrents.

Complexité : contrôle du mandat O(log g + t), g mandats et t cibles bornées ; enveloppes et messages O(B). Aucun balayage de toutes les boucles par envoi.
