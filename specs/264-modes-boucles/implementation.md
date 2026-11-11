# Qualification264

## Contrat et résultat vérifié

Le choix est obligatoire à chaque création. La CLI demande les trois modes en terminal ; un appel automatique sans choix s'arrête avant création. Les anciennes boucles restent ordinaires. Les remises et digests gèlent leur mode et la référence du mandat avant l'essai. Le secret reste en 0600, séparé du registre et exclu de Git.

Le mandat lie une portée de boucle, un projet attesté, au plus 64 UUID désignés et un mode de contrôle. Le client conserve son rôle autonome. Le daemon relit le mandat, la portée, le projet et la cible avant un nouvel effet ; révocation, pause, tranquillité et quotas restent contrôlés. La fermeture révoque le mandat. Une connexion peut encore relire un ancien reçu après révocation ; elle ne peut créer un nouvel effet.

## Vérifications déjà exécutées

- 165 tests Python Loop réussis, dont les scénarios de refus final, reprise explicite et délai sans exposition du corps.
- 7 tests de garde natifs264 réussis dans main et dans la base exécutable ; vérification supplémentaire du fichier de mandat privé, de ses permissions et du refus des liens.
- Recette réelle CLI → socket → daemon → acquittement wrapper réussie pour les trois modes, avec rejeu après révocation sans duplication, refus DND final et nouvelle demande après levée DND.
- Deux tests de transport existants réussis : guidage d'un tour actif et interruption sans arrêter le fournisseur.
- Clippy strict des bibliothèques réussi dans main et dans la base exécutable ; Ruff et liens documentaires réussis.
- Question et réponse saisies dans un vrai terminal, puis refus non interactif sans choix : observés. Diaporama de six étapes, sans clic souris inventé.

Aucune campagne globale nouvelle n'est revendiquée. Les anciennes recettes156 de délégation native ont deux limites observées : contexte projet absent et résultat qui reste working malgré turn_end. Ces défauts ne sont pas corrigés dans264 ; la recette du bus et les tests de transports couvrent séparément le raccord demandé.

## Revue indépendante et corrections

La revue du commit b7591123 a confirmé les limites du mandat. Elle a relevé un guidage de reprise incorrect pour les refus définitifs, une exception Python susceptible d’exposer l’argv et un audit manquant du cycle du mandat. Les trois défauts sont corrigés : reçu final distinct d’une issue inconnue, reprise explicite après preuve de résolution, erreurs de délai assainies et journal silencieux dans la transaction du mandat. Les tests provoquent aussi un échec d’audit et vérifient le retour arrière de la transaction.

La liaison d’un mandat révoqué reste volontairement possible pour lire un reçu ancien. Elle ne prouve aucun droit d’émission. Une cible sans route vivante est refusée sans création d’agent. Le contrôle de non-remise du banc conserve une borne de 250 ms : la recette rétablit la borne normale avant sa remise suivante. Le scénario DND possède son propre corps, identique entre refus et reprise, pour ne pas être confondu avec le premier envoi accepté par la déduplication.

## Reproductibilité et concurrence

La livraison conserve les corrections263-r2 déjà activées, dont l'audit borné des dossiers. Le script build-runtime.py reconstruit263 puis applique runtime.patch. Les douze fichiers propres au raccord reproduisent exactement le candidat. Les extractions260 non autorisées à l'activation restent hors du binaire.

Le partage initial du même target entre deux arbres de sources de même version a produit une métadonnée de dépendance périmée sous Clippy. Les qualifications de la base exécutable utilisent désormais un unique cache enregistré bridget-build-264, sans copie de target et sans cache par test. Le checkout principal conserve son cache habituel.

La compétence Loop est maintenue dans le dépôt de compétences. Son intégration utilise un worktree dédié dans ce seul dépôt : son checkout principal et son worktree main-merge portent des travaux tiers. Aucun de leurs changements n'est intégré ou retiré avec264.

Les deux demandes initiales de revue de plan n'ont créé aucune tâche : la première a rencontré un changement de HEAD pendant l'admission ; la seconde a été refusée localement à cause d'un champ workspace non accepté. Aucun moteur de remplacement n'a été lancé.

## Décision après la deuxième revue

La revue f23e9b57 du commit8bb730ed ne trouve aucun défaut bloquant prouvé. Ses trois points Python sont vérifiés dans le script complet et dans les tests : un rappel refusé reste silencieux à preuve inchangée, le reçu reste à son chemin de notification, et heartbeat/retry-refused prennent exactement le même verrou. Le guidage distingue aussi les refus de protocole d'un reçu rejected. Une expiration ne prouve pas une absence d'effet ; une divergence d'enveloppe exige de retrouver l'original, sans nouvelle clé automatique. Ce texte possède un test couvrant les trois codes.

Le journal conserve volontairement un événement par opération de liaison ou de retrait reçue. L'acteur human atteste le type d'autorité ; aucun nom humain n'est déduit de la connexion. L'attribution plus fine et les cas supplémentaires de contexte projet restent des réserves non bloquantes de couverture, sans changement de droits.

## Livraison activée

Version0.8.30, build264-loop-modes-r1, tag source8bb730ed53bd931bd2f55a5cafa18b6c7898f26d. Release publique v0.8.30 : les six actifs ont été téléchargés sans authentification et leurs empreintes et tailles comparées au manifeste. Paquet Android8030 signé dans la lignée existante. Le code de la compétence Loop est intégré au main distant de son dépôt (4529089d après les intégrations précédentes).

Sept services vérifiés sur quatre hôtes : core/mobile/workerCartae, workersSYS/KS/Mac et raccord Bridget de T3. Core : coupure2,72s. T3 lui-même n'a pas été arrêté. Le lien natif est revenu ; deux boucles temporaires de contrôle ont été créées puis fermées sur le runtime actif. Le journal confirme authorize/revoke et revoked=1 pour chacune. Aucun message métier ni mission de remplacement créé.

Mac : image56b73111d51514e5620483c719e5d1abaa7610aec7d0c8a4e5bfd49e9d85c070. Linux : imagec8badaa739fe9562785f293235b57a2a79f5dcdc89e211cc9de65825c2f207c5, compilée sur Debian12, exige glibc2.34 (maximum autorisé2.36). Huit tests natifs, recette du bus et Clippy strict réussis aussi sur Linux. Les douze fichiers du raccord reproduisent exactement la source compilée. Patch264 : 9bc7f88bfa4293fbdd5052cad2935210002bb8aef3929c7bf2bd4b17bdbdb2b2.

Les quatre pointeurs privés de retour désignent263-r2, avec images et formats qualifiés dans les deux sens. Les superviseurs de missions en cours ont conservé PID et marqueur. Leurs anciennes images restent compatibles avec les intents existants ; le mandat264 est contrôlé dans le nouveau daemon et le nouveau CLI. Aucun résultat inconnu n'est rejoué pour les moderniser. Les propriétaires affectés reçoivent un avis ciblé, sans réveil de maintenance.

## Ménage et conservations

Le nettoyeur natif a retiré6,22Gio d'intermédiaires sur SYS ; les exports propres terminés de SYS représentent758640696octets supplémentaires retirés. Cinq anciennes copies exécutables ont été retirées sur SYS/KS/Cartae, total225076288octets. Actif264 et retour263-r2 sont conservés.

La garde Mac refuse le ménage des caches et anciennes images, car le PID33346 a un dossier courant non observable. Le processus appartient au serveur documentaire d'un autre travail ; il n'est pas arrêté. Cartae conserve une ancienne image262 encore utilisée et ses exports, dont l'observation de processus reste incomplète. SYS conserve aussi une image246 hors du périmètre privé qualifié.

Le ménage SpecKit a été appliqué aux deux dépôts. Aucun worktree ni branche Bridget à retirer. Le worktree temporaire d'intégration de la compétence est propre et ses quatre commits sont intégrés au main distant ; il reste classé non fusionné par le garde local, car le main local de dotfiles est ancien et occupé par un autre travail. Ses milliers de modifications et les autres worktrees sont préservés. Aucun retrait forcé ni déplacement du main tiers.

Les sources265 concurrentes sont conservées ; elles ne changent ni le tag ni le binaire de cette livraison. Leur constructeur réutilise explicitement la base264. Les reçus, rapports, manifestes, journaux et paquets restent dans le cache privé de livraison. Le diaporama versionné est autonome ; aperçu, commandes de navigation, lecture/pause et dernière étape vérifiés.
