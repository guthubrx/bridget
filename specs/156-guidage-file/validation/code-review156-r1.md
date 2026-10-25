## Verdict : APPROVE, limité au périmètre lu

Je n'ai trouvé aucune objection CRITICAL, HIGH ou MEDIUM dans le code lu. Il reste trois remarques LOW, qui ne bloquent pas la livraison. J'ai lu uniquement, sans test, sans build et sans écriture. Les empreintes SHA annoncées sont confirmées : diff `c0f9a983…`, tasks `7c9bd9a8…`, daemon `a11310b7…`.

### Réponses aux questions

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/156-guidage-file/validation/code-review156-r1.md#extrait-1).

### Remarques LOW

1. **QueueList dévoile la taille de la file** (`daemon.rs:9354-9385`). La position `index+1` est calculée avant le filtrage, et la limite de 200 lignes s'applique aussi avant lui. Exemple : un pair voit son seul message à la position 57, donc il sait que 56 messages d'autres émetteurs le précèdent. Et un message propre placé au-delà du rang 200 n'apparaît pas. Correction minimale : documenter ce comportement comme accepté, ou numéroter seulement les entrées visibles.
2. **Send historique laisse des restes si `admitted` échoue** (`daemon.rs:19271-19320`). `track_reply_cycle` et `pending_attach_sends.insert` s'exécutent avant l'écriture de `admitted`. Si cette écriture échoue, le client reçoit un Nack, mais le suivi de réponse et l'entrée d'attache restent en mémoire. Cela n'arrive qu'en cas de panne de la base. Correction minimale : écrire `admitted` avant ces deux insertions, ou les retirer dans la branche d'erreur.
3. **Bruit dans le journal sur Cancel** (`daemon.rs:19779`). Chaque annulation d'un pair ajoute une ligne `refused:intent_not_permitted`, alors qu'elle ne fait qu'un retrait légitime. Ces lignes ne comptent pas dans les quotas. C'est seulement une remarque.

### Point non vérifié, signalé comme risque

`prepare_dispatch` s'exécute avant `check_guidance` sur SendIdempotent et sur Send (`daemon.rs:9773`, `19060`). Je n'ai pas vérifié si un guidage refusé consomme quand même des quotas historiques (`record`, `mark_sent` ou d'autres effets dans `prepare_dispatch`).

### Zones non lues

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/156-guidage-file/validation/code-review156-r1.md#extrait-2).

Par conséquent, je ne conclus pas sur la correspondance entre les conclusions Converge et les preuves. Je ne conclus pas non plus sur les façades CLI/MCP au-delà de la présence de `invalid_intent` dans `mcp.rs` et `cli.rs`. Rien de ce que j'ai lu ne revendique un succès global : les comptes annoncés (RC101, 11 FAIL) restent cohérents avec l'absence de GREEN.
