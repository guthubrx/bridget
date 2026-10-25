# Plan176

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/176-observation-bornee/plan.md#extrait-1).

## Invariants et limites
Aucune observation n'est une réservation exclusive ; la course après observation reste déclarée et git remove sans force conservé. Un contrôle incomplet refuse. Pas de suppression réelle de worktree applicatif pour prouver le correctif.

Bornes fixées : 30 s partagées entre commandes, 8 Mio lsof, 1 Mio Git, 4 096 entrées ; au plus 1 s de grâce TERM. Les appels filesystem Python et le rendu final restent hors délai dur ; aucune promesse de plafond RSS système. Les entrées non parcourues restent inconnues.
