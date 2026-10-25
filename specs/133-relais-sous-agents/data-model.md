# Modèle des données 133

## Preuve de délégation locale

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/data-model.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/data-model.md#extrait-2).

Cycle : observé → publié → relu par MCP → retiré à la disparition, au changement
de naissance, à l'ambiguïté ou à l'arrêt du pont. Aucun état historique n'est
conservé.

## Identité résolue

Une identité MCP résolue contient le nom et l'instance du parent. Elle contient
en option un contexte délégué `{ provider, child_ref }`.

- sans contexte : droits actuels du principal ;
- avec contexte : seulement `bridget_who` et `bridget_send` ;
- par la CLI, la présence d'un contexte produit `delegated_mcp_only`.

## Provenance du message

`BridgetMessage.delegated_origin` est optionnel et reprend `{ provider,
child_ref }`. Il est posé par le serveur MCP après résolution et apparaît dans
le rendu destinataire. Il ne participe pas à l'empreinte de rejeu. `from`, `to`,
`in_reply_to`, le suivi et l'idempotence continuent à utiliser le parent. Un
rejeu par le parent relit donc l'issue existante et ne remplace pas la provenance
du premier dépôt.

Un ancien message sans ce champ se désérialise avec `None` et se réencode sans
champ supplémentaire.
