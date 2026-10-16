# Session 170 — Délais de réponses distantes

## Problème

Une réponse native distante acceptée avant l’échéance peut être persistée et
marquée `result_sent` alors que `tracked_requests` reste `open` tant que l’ACK
de remise n’est pas arrivé. Le thread de rappels transforme alors une réponse
valide en fausse expiration.

## Cause démontrée

`capture_remote_result` appelle `commit_remote_result`, puis la publication
utilise `send_as_with_result_authority`. La transition `tracked_requests →
answered` était uniquement dans `acknowledge_send_delivery`. Les faits runtime
170 montrent `result_available=true`, `result_sent=true`, `delivery_acked=false`
et `tracked_requests.state=timed_out` malgré une fin avant la deadline.

## Contrat

- Une disposition distante `Accepted` clôt la demande corrélée par la primitive
  existante `mark_answered`, sans modifier `delivery_acked`.
- Une erreur SQL laisse le résultat et son reçu durables, puis l’entretien
  réessaie la même clôture.
- Les refus, résultats étrangers, conflits, résultats expirés et absences
  réelles ne ferment aucune demande.
- La clé de rejeu et le payload restent inchangés.

## Périmètre

`crates/bridget-daemon/src/daemon/native_delegation.rs` et les oracles natifs
existants. Aucun endpoint, table, worker, dépendance ou service nouveau.
