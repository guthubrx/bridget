# Livraison — SPEC143

Date : 2026-10-07. Statut : installé et activé. Version143, signature, ASAR et asset servis vérifiés après redémarrage. Les sections suivantes conservent les étapes historiques avant activation ; la clôture ci-dessous les remplace pour l'état actuel.

## Autorisation distincte

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/delivery.md#extrait-1).

## Faits reçus du principal

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/delivery.md#extrait-2).

Les quatre empreintes du code gelé392 restent inchangées après commit/fusion selon le principal. Aucun build, daemon, boucle ou restart exécuté par l'agent documentaire.

Le manifeste de build futur sélectionne déjà `local-patch/bridget-headings-v0.0.45`. Le commit portable143 y est poussé ; les quatre fichiers donnent un diff vide face au code143 validé. Cette preuve capitalise le correctif dans la branche déjà utilisée, sans prétendre que le manifeste a été modifié.

## Paquet et sauvegardes constatés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/delivery.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/delivery.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/delivery.md#extrait-5).

## Candidat validé, encore non installé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/delivery.md#extrait-6).

La validation d'un candidat dans le cache ne prouve ni son installation dans Applications ni son démarrage.

## Contrôle d'activation attendu

Le principal prépare une remise en route supervisée et un reçu durable, car le redémarrage peut interrompre le tour actif. Ne pas confondre une demande de redémarrage avec une activation vérifiée. Les chemins du bundle installé, sa version, sa signature et le reçu de démarrage seront consignés uniquement après preuve.

Superviseur `deploy.sh` et plist préparés mais non activés. Revue finale APPROVE : les deux réserves sont corrigées, propagation du trap ERR avec `set -E` et attestation de sauvegarde contrôlant méthode, quick_check et taille.

Script SHA256 : `3d8a9921eed43a29032b0ef3b089b6d072fe9cbe4e501278c87164bda48c18fd`.
Plist SHA256 : `966ea16a40b555123b155a88965fa80d0248191fec72947e33793811ab1bc873`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/delivery.md#extrait-7).

Le principal a committé les docs143 sous `19296c48`, fusionné main en fast-forward et confirmé le push de main/session143 vers github. Quatre fichiers locaux142 sont byte-identiques avant/après ; stash de conservation `5df30ff87e5401a09b60d34726f226ececdf7b7a` préservé. Le présent suivi sera committé et poussé avant lancement du superviseur ; cette publication suivante n'est pas encore déclarée faite. Aucun changement AGENTS, code, git ou runtime exécuté par l'agent documentaire.

Mise à jour documentaire initiale :12:07 CEST, puis ajout des faits de push/build/sauvegarde. Aucun succès d'installation, activation ou publication documentaire n'est anticipé.

## Clôture réelle après redémarrage

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/delivery.md#extrait-8).
