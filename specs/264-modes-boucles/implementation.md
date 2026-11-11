# Qualification264

## Contrat et résultat vérifié

Le choix est obligatoire à chaque création. La CLI demande les trois modes en terminal ; un appel automatique sans choix s'arrête avant création. Les anciennes boucles restent ordinaires. Les remises et digests gèlent leur mode et la référence du mandat avant l'essai. Le secret reste en 0600, séparé du registre et exclu de Git.

Le mandat lie une portée de boucle, un projet attesté, au plus 64 UUID désignés et un mode de contrôle. Le client conserve son rôle autonome. Le daemon relit le mandat, la portée, le projet et la cible avant un nouvel effet ; révocation, pause, tranquillité et quotas restent contrôlés. La fermeture révoque le mandat. Une connexion peut encore relire un ancien reçu après révocation ; elle ne peut créer un nouvel effet.

## Vérifications déjà exécutées

- 164 tests Python Loop réussis, dont les scénarios de refus final, reprise explicite et délai sans exposition du corps.
- 7 tests de garde natifs264 réussis dans main et dans la base exécutable ; vérification supplémentaire du fichier de mandat privé, de ses permissions et du refus des liens.
- Recette réelle CLI → socket → daemon → acquittement wrapper réussie pour les trois modes, avec rejeu après révocation sans duplication, refus DND final et nouvelle demande après levée DND.
- Deux tests de transport existants réussis : guidage d'un tour actif et interruption sans arrêter le fournisseur.
- Clippy strict des bibliothèques réussi dans main et dans la base exécutable ; Ruff et liens documentaires réussis.
- Question et réponse saisies dans un vrai terminal, puis refus non interactif sans choix : observés. Diaporama de quatre étapes, sans clic souris inventé.

Aucune campagne globale nouvelle n'est revendiquée. Les anciennes recettes156 de délégation native ont deux limites observées : contexte projet absent et résultat qui reste working malgré turn_end. Ces défauts ne sont pas corrigés dans264 ; la recette du bus et les tests de transports couvrent séparément le raccord demandé.

## Revue indépendante et corrections

La revue du commit b7591123 a confirmé les limites du mandat. Elle a relevé un guidage de reprise incorrect pour les refus définitifs, une exception Python susceptible d’exposer l’argv et un audit manquant du cycle du mandat. Les trois défauts sont corrigés : reçu final distinct d’une issue inconnue, reprise explicite après preuve de résolution, erreurs de délai assainies et journal silencieux dans la transaction du mandat. Les tests provoquent aussi un échec d’audit et vérifient le retour arrière de la transaction.

La liaison d’un mandat révoqué reste volontairement possible pour lire un reçu ancien. Elle ne prouve aucun droit d’émission. Une cible sans route vivante est refusée sans création d’agent. Le contrôle de non-remise du banc conserve une borne de 250 ms : la recette rétablit la borne normale avant sa remise suivante. Le scénario DND possède son propre corps, identique entre refus et reprise, pour ne pas être confondu avec le premier envoi accepté par la déduplication.

## Reproductibilité et concurrence

La livraison conserve les corrections263-r2 déjà activées, dont l'audit borné des dossiers. Le script build-runtime.py reconstruit263 puis applique runtime.patch. Les douze fichiers propres au raccord reproduisent exactement le candidat. Les extractions260 non autorisées à l'activation restent hors du binaire.

Le partage initial du même target entre deux arbres de sources de même version a produit une métadonnée de dépendance périmée sous Clippy. Les qualifications de la base exécutable utilisent désormais un unique cache enregistré bridget-build-264, sans copie de target et sans cache par test. Le checkout principal conserve son cache habituel.

La compétence Loop est maintenue dans le dépôt de compétences. Son intégration utilise un worktree dédié dans ce seul dépôt : son checkout principal et son worktree main-merge portent des travaux tiers. Aucun de leurs changements n'est intégré ou retiré avec264.

Les deux demandes initiales de revue de plan n'ont créé aucune tâche : la première a rencontré un changement de HEAD pendant l'admission ; la seconde a été refusée localement à cause d'un champ workspace non accepté. Aucun moteur de remplacement n'a été lancé.

## Livraison

En cours. Ne qualifier la version de livrée qu'après publication, activation et vérification des processus, des sources et des retours.
