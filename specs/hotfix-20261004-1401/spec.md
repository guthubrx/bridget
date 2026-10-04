# Hotfix 20261004-1401 — Continuité des identités T3

## Incident

Le fil `vehicules` recevait `identity_not_found` lors d’un envoi vers le fil
`Personnages`. Le même refus touchait aussi les autres fils T3.

## Cause confirmée

Un échec temporaire de l’inventaire des fichiers ouverts supprimait tous les
marqueurs d’identité. La boucle T3 pouvait attendre plusieurs minutes avant le
cycle de reconstruction suivant. L’inventaire incluait aussi des processus
`codex app-server` internes lancés par les outils d’un agent principal.

## Correction

- Exclure les fournisseurs internes quand un autre fournisseur T3 apparaît dans
  leur ascendance.
- Retenter une fois l’échec temporaire de l’inventaire avant toute révocation.
- Conserver la fermeture sûre après deux échecs consécutifs.

## Validation

- Les tests reproduisent les deux chemins avant la correction.
- La suite workspace passe dans un namespace privé court, en série.
- Le formatage, Clippy et la construction release passent.
- `vehicules` a envoyé un message à `Personnages` avec le statut `recu`.
