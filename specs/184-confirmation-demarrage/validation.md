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
Sources validées. Compilation, installation, activation et nettoyage final à consigner après leurs résultats réels.
