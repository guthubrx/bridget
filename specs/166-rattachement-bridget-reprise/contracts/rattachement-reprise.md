# Contrat interne — Rattachement Bridget à la reprise

## Précondition

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/contracts/rattachement-reprise.md#extrait-1).

## Comportement

- Si la preuve courante est déjà montée, démarrer le tour sans reprise supplémentaire.
- Si elle n’est pas montée, reprendre le même fil natif avec la configuration courante.
- Attendre la fin de la reprise avant de publier les droits et de démarrer le tour.
- Si la reprise échoue, ne pas démarrer le tour.

## Garanties

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/contracts/rattachement-reprise.md#extrait-2).
