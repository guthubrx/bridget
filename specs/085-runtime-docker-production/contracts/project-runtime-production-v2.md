# Contrat - Runtime projet de production v2

## Opérations

- `status`
- `prepare`
- `activate_docker`
- `stop`
- `remove`
- `recreate`
- `switch_to_host`

Chaque requête contient version, `command_id`, timestamps bornés, `project_id`, génération attendue et éventuellement une référence de politique fermée. Aucun argument Docker, chemin, image ou environnement libre n'est accepté.

## Activation Docker

```json
{
  "contract_version": 2,
  "command_id": "opaque",
  "issued_at": 0,
  "deadline_at": 0,
  "operation": "activate_docker",
  "project_id": "opaque",
  "expected_binding_generation": 3,
  "policy_id": "dev-standard",
  "policy_version": 1
}
```

Le daemon résout le digest de politique, prépare l'environnement et ne publie la nouvelle liaison qu'après attestation. Un échec retourne la liaison Host inchangée.

## Refus fermés

- `invalid_contract`
- `project_not_found`
- `generation_mismatch`
- `policy_unavailable`
- `docker_unavailable`
- `image_mismatch`
- `environment_busy`
- `prepare_failed`
- `compensation_failed`
- `store_unavailable`

## Projection UI

La réponse peut exposer backend, génération, état, policy id/version/digest, image version/digest, epoch, capacité et raison fermée. Elle n'expose ni container id complet, ni commande Docker, ni secret, ni contenu de mount.

