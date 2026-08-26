# Contrat 025 — Soumission de lot et décision de régime

## Préconditions communes

- enveloppe `service_request` v1 canonique ;
- projet de revue configuré ;
- aucun champ inconnu ;
- aucune charge libre ;
- aucun chemin de dépôt fourni dans la requête.

## Opération `review_lot_submit`

```json
{
  "type": "service_request",
  "v": 1,
  "issuer_scope": "scope-025",
  "request_id": "request-submit-025",
  "issued_at": 1787662000,
  "from": "author-1",
  "to": "maicie",
  "operation": "review_lot_submit",
  "payload": {
    "project_id": "bridget",
    "branch_ref": "refs/remotes/origin/fix/example",
    "base": "1111111111111111111111111111111111111111",
    "head": "2222222222222222222222222222222222222222"
  }
}
```

Réponse acceptée :

```json
{
  "issue": "accepted",
  "submission_id": "<sha256 canonique>",
  "project_id": "bridget",
  "base": "1111111111111111111111111111111111111111",
  "head": "2222222222222222222222222222222222222222",
  "proposed_regime": "jury_1_plus_1",
  "state": "awaiting_decision",
  "critical_paths": ["crates/bridget-transport/src/protocol.rs"],
  "unresolved_citations": 0
}
```

`critical_paths` ne contient que les chemins modifiés ayant déclenché le
régime, jamais toute la carte ni le contenu du diff.

Une citation contractuelle n’ancre qu’un chemin complet existant hors de
`specs/*/contracts/`. Les références croisées entre contrats ne s’élisent pas
réciproquement.

## Opération `review_regime_select`

```json
{
  "type": "service_request",
  "v": 1,
  "issuer_scope": "scope-025",
  "request_id": "request-select-025",
  "issued_at": 1787662010,
  "from": "referent-1",
  "to": "maicie",
  "operation": "review_regime_select",
  "payload": {
    "submission_id": "<sha256 canonique>",
    "retained_regime": "revue_simple"
  }
}
```

Réponse acceptée :

```json
{
  "issue": "accepted",
  "submission_id": "<sha256 canonique>",
  "proposed_regime": "jury_1_plus_1",
  "retained_regime": "revue_simple",
  "direction": "lightened",
  "deviation_state": "open",
  "state": "decision_recorded"
}
```

La charge ne comporte ni motif ni justification. `proposed_regime` n’est pas
fourni par le référent : Maicie le relit depuis la soumission, puis confronte.

## Régimes fermés

```text
revue_simple < jury_1_plus_1 < jury_2x2
```

Une soumission touchant le noyau 025 reçoit directement le régime utilisateur
fixé `jury_2x2`; `review_regime_select` est alors refusée avec
`self_regime_fixed`.

## Conditions de refus métier

| Condition | Sens | Ligne durable |
|---|---|---|
| `review_project_unconfigured` | aucun projet actif dans la config | oui |
| `review_project_mismatch` | identifiant différent | oui |
| `repository_unavailable` | dépôt configuré absent ou illisible | oui |
| `branch_ref_not_full` | référence courte ou invalide | oui |
| `branch_head_mismatch` | la référence ne pointe pas sur la tête | oui |
| `commit_unavailable` | base ou tête absente | oui |
| `base_not_ancestor` | couple de commits incohérent | oui |
| `git_measurement_failed` | mesure Git non concluante | oui |
| `review_snapshot_limit_exceeded` | univers ou diff au-delà des bornes | oui |
| `catalogue_unavailable` | catalogue déclaré illisible | oui |
| `submission_missing` | décision sans soumission | oui |
| `referent_mismatch` | autre émetteur que le référent configuré | oui |
| `decision_already_recorded` | choix divergent après décision | oui |
| `self_regime_fixed` | tentative de remplacer le régime du noyau | oui |
| `reviewer_election_unavailable` | tentative d’avancer avant le lot capteur | oui |

Une erreur qui empêche l’ouverture de la SQLite ne fabrique pas une ligne
métier. Elle termine la commande par une erreur technique explicite.

Si le daemon actif refuse l’enveloppe avant livraison à Maicie — opération
inconnue ou `CanonicalBytesMismatch` — ce refus fermé est persisté et compté
par le greffe daemon qui l’observe. Maicie ne fabrique pas une seconde ligne
locale. La variante exacte doit étendre la source fédérée de la session 026,
et n’est pas redéfinie par ce contrat avant son absorption.

## Idempotence

- même `issuer_scope`, `request_id` et mêmes octets : même réponse persistée ;
- même identifiant de soumission et même tuple : même proposition ;
- même identifiant avec tuple divergent : refus ;
- une décision acceptée ne peut être remplacée ;
- une réponse n’est jamais reconstruite depuis l’état courant après son commit.

## Données exclues

Les reçus, zones et métriques ne contiennent jamais : patch, ligne source,
contenu de fichier, trame externe, texte complet de constat, secret synthétique
ou valeur de variable d’environnement.
