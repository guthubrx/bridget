# État avant correction — SPEC-166

Date : 2026-10-15, Europe/Paris.

## Versions

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/validation/runtime-before.md#extrait-1).

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
