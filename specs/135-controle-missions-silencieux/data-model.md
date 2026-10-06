# Modèle de données 135

## Run

- `status` : `open` ou `closed`.
- `mission_control_enabled_at` : début du nouveau contrat.
- Politiques : délais, limite de digest, rôle d'escalade et nombre de rappels.

## Tâche

- `assigned_agent` : responsable stable.
- `due_at` : échéance de résultat.
- `expected_result` : preuve attendue.
- `acknowledged_at` : prise en charge.
- `last_progress_at`, `last_progress_kind`, `last_progress_ref` : dernier
  progrès vérifiable.
- `last_progress_hash`, `progress_seen_hashes` : empreintes des preuves déjà
  enregistrées pendant la tentative. Une preuve ancienne ne repousse pas le
  délai, même après une autre preuve.
- `disposition` et `dispositioned_at` : décision de suite distincte du verdict.

## État heartbeat

- `issues` : table par clé stable `event + task_id`. Deux types d'anomalie sur
  la même tâche restent donc indépendants.
- Chaque entrée contient l'étape déjà notifiée et l'empreinte de la dernière
  preuve observée.
- Une entrée résolue est retirée. Un même état ne produit pas deux messages.
- `outbox` : dernier digest préparé par destinataire, avec identifiant et date
  stables avant la remise. Les reçus acceptés restent dans `receipts/`.

## Invariants

- Le contrôleur ne change jamais un verdict terminal.
- Un progrès doit avoir un type fermé et une preuve non vide.
- Une clôture exige zéro tâche active et zéro décision terminale requise.
- Les mises à jour de tâche relisent la dernière version sous verrou. Elles
  fusionnent les seuls champs modifiés. Un ACK périmé ne réactive pas une tâche
  terminée. La migration ne modifie pas son horodatage métier.
