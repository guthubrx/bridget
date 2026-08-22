# Journal d'Implémentation — Transport ACP

## T701 — Spike adaptateur Codex hors Bridget

- **Statut** : terminé
- **Date** : 2026-08-22
- **Précondition** : `OPENAI_API_KEY` absent de l'environnement.
- **Script jetable** : `/tmp/bridget-spike-codex-acp.mjs`
- **Transcriptions brutes** :
  - `/tmp/bridget-spike-codex-acp-attempt1.transcript`
  - `/tmp/bridget-spike-codex-acp-attempt2.transcript`

Les réponses `initialize` complètes contiennent des métadonnées de modèles et
des instructions volumineuses, générées par le service (plus de 200 ko chacune).
Elles restent dans les transcriptions locales indiquées ci-dessus ; les lignes
JSON-RPC déterminantes sont consignées ici afin d'éviter de versionner ces
données externes non nécessaires à Bridget.

### Tentative 1 — modèle surchargé (échec)

Commande :

```text
npx @zed-industries/codex-acp@0.16.0 -c 'model="gpt-5.6-sol"' -c 'model_reasoning_effort="high"'
```

```text
> {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"bridget-spike","version":"0.1.0"}}}
< {"jsonrpc":"2.0","result":{"protocolVersion":1,...},"id":1}
> {"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/007-transport-acp","mcpServers":[]}}
< {"jsonrpc":"2.0","result":{"sessionId":"01a027b0-..."},"id":2}
> {"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{"sessionId":"01a027b0-...","prompt":[{"type":"text","text":"Réponds exactement : SPIKE_ACP_OK"}]}}
< {"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error","data":{"message":"...The 'gpt-5.6-sol' model requires a newer version of Codex..."}},"id":3}
```

### Tentative 2 — home Codex isolé (succès)

Commande :

```text
CODEX_HOME=/tmp/spike-codex-home npx @zed-industries/codex-acp@0.16.0
```

```text
> {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"bridget-spike","version":"0.1.0"}}}
< {"jsonrpc":"2.0","result":{"protocolVersion":1,...},"id":1}
> {"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/007-transport-acp","mcpServers":[]}}
< {"jsonrpc":"2.0","result":{"sessionId":"01a027b4-..."},"id":2}
> {"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{"sessionId":"01a027b4-...","prompt":[{"type":"text","text":"Réponds exactement : SPIKE_ACP_OK"}]}}
< {"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"SPIKE_ACP_OK"}}}}
< {"jsonrpc":"2.0","result":{"stopReason":"end_turn"},"id":3}
```

Le répertoire `/tmp/spike-codex-home` a été supprimé après le test. Aucun
adaptateur ne demande de clé API pendant les deux tentatives.

### Self-review Article XIX/XX

- **Nécessité** : valider la compatibilité ACP avant toute intégration évite de
  construire sur un adaptateur inutilisable.
- **Simplicité** : le spike utilise uniquement `node`, `npx` et JSON-RPC ; aucune
  dépendance ni source Bridget n'a été ajoutée.
- **Vérifications** : absence de `OPENAI_API_KEY`, deux cycles
  `initialize`/`session/new`/`session/prompt`, retour `stopReason` sur la
  tentative gagnante.
- **Non vérifié** : les adaptateurs Claude et Gemini, prévus par T707 et T708.

## T702 — Décision et dépréciations

- **Statut** : terminé
- **Fichiers** : `docs/decisions/003-transport-acp.md`,
  `docs/DEPRECATIONS.md`

### Self-review Article XIX/XX

- **Nécessité** : la nouvelle frontière de protocole est une décision
  structurante ; les chemins hérités doivent rester traçables.
- **Simplicité** : un ADR et un tableau unique, sans nouvel outil ni format.
- **Vérifications** : l'ADR reprend le cycle R-001 et l'arbitrage R-004 ; le
  registre a les trois colonnes requises.
- **Non vérifié** : aucun chemin de livraison hérité n'est encore remplacé ;
  T703 et T705 alimenteront le registre au moment de leur suppression.
