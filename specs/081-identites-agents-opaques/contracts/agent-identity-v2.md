# Contrat Agent Identity v2

## Enregistrement wrapper vers daemon

    {
      "type": "Register",
      "agent_id": "550e8400-e29b-41d4-a716-446655440000",
      "agent_type": "codex",
      "instance_id": "..."
    }

Le champ name est refusé. agent_id est un UUID v4 connu, sauf création contrôlée par le daemon.

## Annuaire

    {
      "agent_id": "550e8400-e29b-41d4-a716-446655440000",
      "display_name": "Bibliothécaire",
      "state": "alive",
      "type": "codex"
    }

Les clients techniques ciblent agent_id. Les surfaces utilisateur affichent seulement display_name.

## Message

    {
      "id": "msg-opaque",
      "from": "550e8400-e29b-41d4-a716-446655440000",
      "to": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      "body": "Peux-tu vérifier ce point ?",
      "reply": true
    }

from et to portent des principaux documentés. Leur rendu humain est fait par le daemon, jamais par le client.

## Profil

    GET /v2/agent-profiles/{agent_id}
    PATCH /v2/agent-profiles/{agent_id}

La réponse contient agent_id, display_name, labels, avatar, instructions et révision. profile_ref n'existe pas dans v2.

## Maicie

    {
      "profile_id": "reviewer",
      "agent_id": "550e8400-e29b-41d4-a716-446655440000",
      "display_name": "Relecteur"
    }

Une référence non résolue devient :

    {
      "state": "requires_retarget",
      "agent_id": null,
      "retarget_reason": "legacy_identity_removed"
    }
