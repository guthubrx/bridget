# Vérification de la session 043

## Décision

Le délai n'est pas renommé ni rendu légitime hors demande suivie : le type,
le daemon et le MCP lui donnent déjà un sens unique. La CLI refuse les deux
incohérences avant de joindre le daemon. Un `--reply` humain est également un
refus, et non un avertissement, car le succès ne pourrait pas honorer cette
intention.

## Progression

- [x] Mesure sur le vrai binaire et le JSON sérialisé.
- [x] Contrat MCP, type et consommateur daemon rapprochés.
- [x] Gardes CLI et oracles permanents.
- [x] Mutants, formatage et banc ciblé.

## Rebase et preuves finales

- Base rebasée sur `75dd315bc2e6389f9c206d179443701f03f94100`; le conflit du
  harnais avec le refus des options inconnues a conservé les deux familles de
  témoins.
- Le vrai binaire, relié à une socket Unix jetable, donne **16 passés / 0
  échec / 0 ignoré** pour `cli_arguments_integration_test`.
- Retirer la garde de `send` donne **0 passé / 2 échecs** : les deux témoins
  voient la trame quitter le programme.
- Retirer séparément la garde de `reply` donne **0 passé / 2 échecs** : avec
  un dernier expéditeur réel, les deux trames interdites quittent aussi le
  programme.
- La fixture distingue désormais `connection_accepted` de la trame : elle
  accepte toute connexion en attente avant le signal d'arrêt. Insérer une
  connexion nue juste avant la garde de `send`, puis de `reply`, donne chaque
  fois **0 passé / 2 échecs** avec `connection_accepted: true`.
- La restauration redonne le banc nominal; `rustfmt` ciblé et `git diff
  --check` restent verts.
