# Contrat daemon-wrapper - Incidents délégués v1

## Émission enfant vers daemon

Le wrapper ne choisit jamais le parent. Il soumet un fait associé à son
exécution courante. Le daemon dérive le lien et le parent depuis la connexion
enregistrée.

```json
{
  "type": "delegated_runtime_event",
  "kind": "warning",
  "execution_id": "execution-child-42",
  "code": "unsupported_provider_request",
  "reference": "sha256:ab12..."
}
```

## Remise daemon vers parent

```json
{
  "type": "delegated_runtime_event",
  "event": {
    "cursor": 7,
    "event_id": "runtime-...",
    "link_id": "link-...",
    "child_instance_id": "instance-child",
    "child_execution_id": "execution-child-42",
    "kind": "warning",
    "code": "unsupported_provider_request",
    "reference": "sha256:ab12...",
    "observed_at": 1780000000
  }
}
```

Le wrapper du parent en fait une notification système à file normale. Elle ne
porte pas de réponse attendue, n'interrompt pas le parent et ne vaut pas une
transition métier.

## Accusé parent vers daemon

```json
{
  "type": "delegated_runtime_event_acknowledged",
  "event_id": "runtime-..."
}
```

L'accusé signifie que la notification a atteint une frontière observable du
transport: `PromptDispatched` pour une session gérée, ou injection réussie
dans un pane tmux. Une simple écriture du daemon vers le wrapper ne vaut jamais
accusé. Le daemon vérifie que l'émetteur est bien le parent lié. Un accusé
inconnu, déjà accusé ou émis par un autre parent est refusé sans modifier le
fait.

## Compatibilité

- Les wrappers précédents n'émettent aucun fait et ignorent les nouvelles
  trames inconnues selon leur comportement historique.
- Les événements antérieurs à la migration ne sont pas inventés.
- Aucun contrat du guichet Maicie n'est modifié.
