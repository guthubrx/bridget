# Modèle de données — SPEC-166

## État existant utilisé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/data-model.md#extrait-1).

## Invariant

Avant `turn/start`, si une preuve privée courante existe, elle doit être identique à la preuve mémorisée comme montée pour ce `ThreadId`.

## Transitions

1. Le fil natif est chargé avec A.
2. La session privée tourne vers B.
3. Le début du tour observe A différent de B.
4. `thread/resume` recharge le même fil avec B.
5. La carte mémorise B.
6. Les droits du run sont publiés pour B.
7. `turn/start` démarre.

Aucune table, migration ou donnée persistante nouvelle n’est requise.
