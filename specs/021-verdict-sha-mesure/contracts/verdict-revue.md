# Contrat 021 — Attestation Git d'un verdict de revue

## Mandat

Une délégation de revue porte :

```json
{"target_ref":"origin/fix/exemple","expected_head":"1111111111111111111111111111111111111111"}
```

Les deux champs sont présents ensemble ou absents ensemble. `target_ref` est
une référence distante `<remote>/<branche>` ; `expected_head` est un SHA-1 Git
hexadécimal minuscule de 40 caractères.

## Dépôt

Le `delivery_report` historique reste inchangé quand `review_verdict` est
absent. Pour une revue, le binaire ajoute ce bloc après avoir interrogé Git :

```json
{
  "review_verdict": {
    "verdict": "approve",
    "target_ref": "origin/fix/exemple",
    "expected_head": "1111111111111111111111111111111111111111",
    "measured_head": "1111111111111111111111111111111111111111",
    "observed_target_head": "1111111111111111111111111111111111111111"
  }
}
```

`measured_head` vient de `git rev-parse --verify HEAD^{commit}`.
`observed_target_head` vient de
`git ls-remote --exit-code <remote> refs/heads/<branche>`.

## Matrice de décision

| Mandat | Verdict | Tête distante | HEAD | Issue |
|---|---|---|---|---|
| revue | absent | — | — | `review_verdict_required` |
| ordinaire | présent | — | — | `review_verdict_unexpected` |
| revue A | mandat déclaré ≠ A | — | — | `review_mandate_mismatch` |
| revue A | A | B | A | `target_head_moved` |
| revue A | A | B | B | `target_head_moved` |
| revue A | A | B | C, avec C ≠ A et C ≠ B | `target_head_moved_and_measured_head_mismatch` |
| revue A | A | A | B | `measured_head_mismatch` |
| revue A | A | A | A | accepté |

Tous les refus précèdent toute transition. `target_head_moved` encode l'absence
de faute du juré seulement si son `HEAD` correspond encore au mandat A ou déjà
à la nouvelle cible B. Un troisième SHA C conserve les deux faits dans un motif
composé : déplacement de cible et tête mesurée étrangère à A comme à B.

## Non-garanties

L'égalité des SHA n'atteste ni la propreté du worktree, ni les fichiers ignorés,
ni le contenu du target de compilation, ni l'environnement, ni le résultat des
tests. Le nom du remote est résolu par la configuration Git locale : son URL
n'est pas gelée par le mandat. Enfin, le filaire ne prouve pas
cryptographiquement qu'un client alternatif n'a pas forgé les observations ;
la mesure forcée appartient au chemin CLI officiel. Le contrat interdit
uniquement qu'un verdict greffé déclaré se rapporte à un autre commit que celui
gelé.
