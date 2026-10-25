# Hotfix 20261004-1401 — Continuité des identités adaptateur de bureau

## Incident

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/hotfix-20261004-1401/spec.md#extrait-1).

## Cause confirmée

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/hotfix-20261004-1401/spec.md#extrait-2).

## Correction

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/hotfix-20261004-1401/spec.md#extrait-3).

## Validation

- Les tests reproduisent les deux chemins avant la correction.
- La suite workspace passe dans un namespace privé court, en série.
- Le formatage, Clippy et la construction release passent.
- `vehicules` a envoyé un message à `Personnages` avec le statut `recu`.
