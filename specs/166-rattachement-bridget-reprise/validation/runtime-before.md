# État avant correction — SPEC-166

Date : 2026-10-15, Europe/Paris.

## Versions

- Bridget source active : `3b4f101e0732`.
- T3 source installée : `3ea39f11095debbba6ca6a73f99f44fbe308cd60`.
- T3 application : `0.0.45-local.164`.
- T3 ASAR SHA-256 : `dc5623d70f45c85909703274ba59d216295b9e84b1e50ab8685b75aabe73f942`.
- Processus T3 principal observé : PID 30234, démarré le 14 octobre 2026 à 15:50 CEST.

## Défaut observé

- Fil repris : `17f50a6d-ec2f-494a-b9a3-0def5b14475b`.
- Agent Bridget : `a7621b9b-ed64-43a5-9965-c7fef76abb1d`.
- Résultat : `t3_session_unavailable` au premier appel Bridget du nouveau tour.
- Le fil était exposé par le pont avant l’appel.
- Un appel Bridget du fil courant réussissait au même moment.
- Le défaut est donc limité au rattachement privé du fil repris.

## Preuve source

`startNativeTurn()` relit la preuve courante. Si la carte de montage contient une autre preuve, le code invalide les droits puis envoie directement `turn/start`. Il ne recharge pas la configuration du fil natif.

Aucune valeur d’autorisation n’est conservée dans ce document.
