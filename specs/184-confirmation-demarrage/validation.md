# Validation — 27 octobre 2026

## Commande métier existante
Tâche f0b0a469-9c74-4748-bc3e-061df740141a, clé opus2d-20261027-ks-finition-lot-0432. Le parent conserve son contrôle. Aucun rejeu, remplacement ou arrêt demandé par le correctif.
Le journal de l'enfant 26dee445-1d46-49a1-a0e7-54b53f20fef2 confirme la remise métier : séquences 485 à 488, 04:42:36–04:42:40 UTC, lectures réelles des images attendues sur KS-STOR.

## Tests ciblés
Tous les appels Cargo utilisent scripts/build.py, avec le cache du checkout principal Cartae.
- spec184_late_release_reply_preserves_worker_connection : ROUGE avant le changement de production ; VERT après. La version finale fait expirer une vraie attente RPC, puis livre l'accusé tardif. Cas actif et annulé, état durable inchangé ; autre instance et ancienne époque refusées.
- spec184_slow_release_confirmation_keeps_existing_task : VERT ; réponse Release après six secondes, même tâche, aucun doublon.
- spec172_untracked_worker_refusal_keeps_known_command_channel : VERT.
- worker172_release_apres_attestation_lente_garde_revision_et_expiration : VERT.
- spec174_remote_release_uncertain_message_preserves_task_and_replay : VERT.
Dernier passage spec184 : 2 PASS, 6,13 s. Aucune suite globale ni fournisseur réel lancé par les tests.

## Revue indépendante
Bridget : requête bridget-spec184-review-20261027-01, tâche 24f07b2d-3174-4d45-936c-342f3abca2c5, source de base 9d7184b1589a4e6824d9ad46dbafdd117c0655ea, diff intégral fourni.
Aucun défaut bloquant ni affaiblissement des droits détecté.
Deux remarques traitées après lecture : test du vrai délai RPC avec état annulé conservé ; une même durée locale pour Start et Release.
Un accusé valide d'une commande annulée est ignoré sans réactivation ; le rejeter fermerait inutilement le canal des autres commandes.

## Portée
Le permis et son expiration restent inchangés. Un dépassement de trente secondes conserve une issue incertaine et la réconciliation existante.
Correction technique : aucun diaporama d'interface requis.
Nettoyage automatique appelé avant/après Cargo ; suppression différée par le garde d'observation des processus incomplet. Aucun nettoyage forcé.

## Livraison
Correctif source 1caabb9ecf7f25b68b295286e8c535ba0cdd0ae8 intégré à main et poussé.
Compilation release optimisée réussie en 1 min 08 s, via scripts/build.py.
Candidat installé et vérifié :
/home/moi/.local/lib/bridget-communication/184-1caabb9ecf7f/bin/bridget
Build : 184-confirmation-1caabb9ecf7f ; version : 0.6.3.
SHA-256 : 7344d44652f33e997ce0d44ade2cfd7c7a02ea2e1eece86f0ef602b721e9c75e.

L'appel natif updates prepare a été réellement exécuté. Refus : update_missions_active_or_unconfirmed. Aucune barrière contournée, aucun lien actif modifié, aucun service redémarré.
Le daemon reste actif sur 183-admission-7478e9e7. Core, mobile, worker Cartae et liaison KS-STOR relus actifs après ce refus.
Le correctif n'est donc PAS actif. La livraison runtime reste en attente d'une maintenance native admise.

L'appel installed a été exécuté après vérification du candidat hors cache ; il a refusé le nettoyage pour observation des processus incomplète. Les caches ne sont pas supprimés de force.
Le worktree et la branche 184 sont intégrés et propres, sans fichiers ignorés signalés. Ils sont conservés pour la livraison encore active et parce que le garde d'usage reste incomplet. Aucun retrait de worktree, branche ou ancien binaire n'est revendiqué.
Le candidat s'ajoute temporairement à la version active et au retour existant. Ces exécutables sont conservés. Les preuves locales restent dans /home/moi/.cache/bridget184, hors Git.

Prochaine action : refaire la demande native de maintenance, puis activer et vérifier ce même candidat si elle est admise. Ne pas annuler de missions ou modifier une base pour faire passer ce refus. Nettoyer les ressources de 184 après activation et vérification des usages.
