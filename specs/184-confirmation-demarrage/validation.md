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

## Complément SYS-LE-2 et instruction sur les refus
Observation du 27 octobre : wrapper115301 et Codex115521 ont pour cwd le worktree58459353 supprimé, device2052/inode2490404/nlink0. Le chemin courant est /mnt/storage/opus2d/worktrees/58459353, device2065/inode127664278. Le propriétaire a été informé ; aucun arrêt, changement de mapping ou remplacement métier exécuté par ce correctif.
Le contrôleur conserve son refus. La réponse précise la preuve manquante, la machine et le dossier attendu, les vérifications possibles et la décision humaine à demander si le déplacement était voulu.
Six tests ciblés PASS via scripts/build.py : trois spec184, matrice spec172 des états/preuves (dont rapport absent pour worktree), renouvellement du tour avec contexte et absence de mutation, projection spec174 de l'ancienne incertitude.
Revue initiale du complément : tâche fdf5a11f-9a44-48fe-a47d-5d68ebdbe18c, clé bridget-spec184-guidance-review-20261027-02. Deux objections ont été traitées : ancien incident après fin de tâche, codes non couverts.
Contre-revue finale : tâche42b9dfbd-47fd-4458-bb27-53a42aed9a91, clé bridget-spec184-guidance-review-20261027-03. Verdict APPROUVÉ, aucun bloquant identifié. Aucun modèle métier n'a été lancé.
Les six tests ciblés ont été rejoués après ces ajustements : PASS. Le cas terminal failed reste visible, tandis que l'incertitude initiale reste historique. Les codes inconnus déclarent leur diagnostic manquant et un recours humain sûr.
Complément source intégré et poussé : 7899d0bdab0112d804af04ef2966484a8ed07702.
Compilation release du candidat actualisé : PASS en56,62s.
Candidat vérifié et installé : /home/moi/.local/lib/bridget-communication/184-guidance-7899d0bdab01/bin/bridget.
Build184-guidance-7899d0bdab01, version0.6.3, SHA-256621101901c8fc6c3df278cc82f94133a02132d501bd1b4a57f55a07785753483.
Nouvel appel natif updates prepare : refus update_missions_active_or_unconfirmed. Aucun lien actif modifié ni service redémarré. Le daemon relu est toujours183-admission-7478e9e7 ; core/mobile/workerCartae et liaisonSYS-LE-2 sont actifs.
L'appel installed a de nouveau différé le nettoyage pour observation des usages incomplète. Le candidat initial1caabb9e est conservé avec le candidat actualisé et les exécutables actifs/retours : aucun retrait forcé pendant la livraison ouverte. Le worktree/branche184 restent propres, intégrés mais conservés jusqu'à activation et contrôle d'usage. Aucun travail Opus2D touché.
Les refus améliorés et la projection historique sont donc livrés en source et dans le candidat, PAS dans le runtime actif.
