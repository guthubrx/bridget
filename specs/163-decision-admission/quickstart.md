# Validation SPEC-163 — procédure initiale et résultats qualifiés

**Statut courant au 14 octobre 2026 :** Implemented, 58/58 (100 %) ; **Pipeline : Livré** avec conservations justifiées. G8/G9 et documents de livraison sont acceptés ROOT ; T045 clôturée sur le commit documentaire c57cfd35efa1338cda5256a02c57d71258d8b8d4 poussé et vérifié. Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 inchangée ; les marqueurs de clôture 58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5, github/main égal HEAD propre vérifié par ROOT. La source active reste 3efa, distincte de ce commit documentaire.

La procédure ci-dessous est le plan initial historique. Ses résultats réels, limites et G8/G9 sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/163-decision-admission/validation/livraison163.md.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/163-decision-admission/quickstart.md#extrait-1).

Paramètres de production non fixés à ce stade : service_id/endpoint/machine/protocole/model obligatoire, checkpoint si requis, profil fixe/repli, préférences et transport HTTPS ou tunnel loopback. Les automatismes restent désactivés tant que leur gate réelle manque.

## Contrôle de contrat avant recette Laya

Déclarer model="typed-decisions" pour laya_systemone_v1 et vérifier sa conservation dans le snapshot. La requête porte questions objet, candidate:{type:"choice", instructions, criteria:{c01:description,…,c08:description}}, state borné, max_len=1024 et head_max_len=512. Contrôler routing.model égal au modèle attendu ; model racine laya-rl-agent est seulement informatif. Ne pas réutiliser routing.reason comme raison du candidat : normalisation reason=None/null si absente. Le texte d’une raison réellement présente conserve sa borne 1–256 octets UTF-8.

Le constructeur préparatoire historique avec questions liste reste une preuve historique invalide pour l’API Laya. Le correctif rapporte un PASS ciblé de normalisation, pas un groupe final ni une recette HTTP. Les bornes128/1536/4096/32 restent identiques. Aucune nouvelle tâche, CLI, fichier de test ou recette ajoutée ; G8/G9 non prouvés.
