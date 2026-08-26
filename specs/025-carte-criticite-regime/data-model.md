# Modèle de données 025

## Projet de revue

Configuration, non persistée comme carte :

| Champ | Règle |
|---|---|
| `project_id` | identifiant fermé, stable, non vide |
| `repository_root` | chemin absolu, dépôt Git lisible |
| `referent_id` | identité exacte autorisée à retenir le régime |

Aucun champ de chemins critiques n’est admis.

## Zone critique

| Champ | Règle |
|---|---|
| `project_id` | partition de la carte |
| `path` | chemin complet relatif au dépôt, clé avec `project_id` |
| `first_elected_at` | première preuve attestée |
| `last_observed_at` | dernière preuve observée |
| `evidence_json` | liste canonique de preuves structurées, sans contenu source |
| `active` | toujours vrai en 025 ; aucune sortie exposée |

Une preuve contient `kind`, `source_id` et éventuellement `rule_id` ou
`severity`. Elle ne contient jamais le texte complet ayant porté la citation.

## Citation non résolue

| Champ | Règle |
|---|---|
| `project_id` | projet mesuré |
| `source_kind` | `registry` ou `contract` |
| `source_id` | constat ou chemin de contrat |
| `token_hash` | empreinte du jeton, pas le texte environnant |
| `candidates_json` | chemins complets candidats, liste bornée |
| `observed_at` | instant de la mesure |

La clé idempotente inclut projet, source et empreinte. Une ambiguïté ne crée
aucune zone.

## Soumission de lot

| Champ | Règle |
|---|---|
| `submission_id` | SHA-256 du projet, ref, base et tête canoniques |
| `issuer_scope`, `request_id` | idempotence du guichet |
| `project_id` | doit correspondre à la configuration active |
| `author_id` | issu de l’enveloppe `from` |
| `branch_ref` | référence Git complète |
| `base_head`, `target_head` | SHA complets |
| `changed_paths_json` | chemins complets, sans contenu |
| `proposed_regime` | régime calculé |
| `state` | `awaiting_decision` ou `decision_recorded` |
| `submitted_at` | horodatage de greffe |

Une même identité de soumission avec des octets divergents est un conflit.

## Décision de régime

| Champ | Règle |
|---|---|
| `decision_id` | identifiant stable |
| `submission_id` | unique : une décision par lot |
| `proposed_regime` | recopié depuis la soumission dans la transaction |
| `retained_regime` | valeur choisie par le référent |
| `direction` | `same`, `strengthened`, `lightened` |
| `decided_by` | doit égaler `referent_id` |
| `decided_at` | horodatage de greffe |

Il n’existe aucun champ libre de motif.

## Écart et clôture

Une direction autre que `same` crée un écart :

| Champ | Règle |
|---|---|
| `deviation_id` | identifiant stable lié à la décision |
| `submission_id` | lot exact |
| `direction` | `strengthened` ou `lightened` |
| `state` | `open`, `confirmed`, `not_confirmed`, `refuted` |
| `closure_fact_kind` | absent tant que l’écart est ouvert |
| `closure_fact_id` | verdict de ronde ou constat lié au lot exact |
| `closed_at` | instant attesté du fait |

Une transition de clôture est append-only. Aucun retour à `open` et aucune
seconde clôture divergente ne sont admis.

## Refus

Les refus réutilisent le reçu fédéré du guichet v19 : opération demandée,
condition fermée, requête canonique ou son empreinte, réponse exacte et
horodatage. Les états SQL restent fermés ; le vocabulaire d’opération et de
condition vient de la source Rust unique de v19.

## Projections

`review metrics` calcule depuis les lignes durables :

- soumissions et décisions ;
- régimes proposés/retenus ;
- écarts par direction et état ;
- refus par condition ;
- zones par voie et citations non résolues.

Aucun compteur séparé n’est persisté.
