# Protocole du spike-gate de branchement (D-403 / FR-012 — session 010)

**But** : prouver, pour chaque harness pinné, l'injection **strictement
éphémère** du serveur MCP `fake-mcp-server.py` (outil `probe`), sans daemon
Bridget et **sans aucune écriture de configuration utilisateur**. Un échec
révise la spec (gate), il ne se contourne pas.

## Préparation commune

1. `SPIKE=/Users/moi/Nextcloud/10.Scripts/bridget/specs/010-mcp/spike/fake-mcp-server.py`
2. Instantané avant : `ls -laR ~/.claude ~/.codex ~/.gemini 2>/dev/null | shasum`
   (re-calculer après chaque voie : **diff vide exigé**).
3. Test à vide du serveur :
   `printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}\n' | python3 $SPIKE`

## Les quatre voies (versions pinnées consignées à l'exécution)

| Voie | Injection éphémère à tester |
|---|---|
| Claude Code | lancement avec config MCP passée au démarrage (`--mcp-config` inline/fichier temporaire) — jamais `claude mcp add` (persistant) |
| Codex CLI | surcharge à la volée `-c 'mcp_servers.probe={command="python3",args=["…/fake-mcp-server.py"]}'` (motif validé au spike 007 pour `model`) — jamais d'édition de `config.toml` |
| Gemini CLI | statut **attendu : non testable voie individuelle** (constat T708) — consigner le refus tel quel, la gate le documente |
| Équipier ACP | `mcpServers` de `session/new` (constaté au spike 007-T701) via un lancement d'équipier de test |

## Pour chaque voie, consigner

- commande exacte + version du harness ;
- preuve que `tools/list` a été rappelé (log stderr du serveur) et que
  `probe` a été exécuté (`PROBE_OK` dans la réponse du harness) ;
- stdout/stderr du serveur (pureté stdout vérifiée) ;
- diff des configurations utilisateur = **vide** ;
- nettoyage (aucun processus ni fichier résiduel).

## Issue de la gate

- 4/4 (ou 3/4 avec Gemini documenté indisponible — décision déjà actée en
  007) → SC-004 de la 010 s'applique selon sa forme révisée ;
- toute autre voie en échec → révision de spec AVANT tasks (jamais
  d'`unsupported` silencieux).
