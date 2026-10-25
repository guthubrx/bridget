# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/diagnostic-mcp.md](../../../../../../specs/147-panneau-bridget-vivant/diagnostic-mcp.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Cause vérifiée en lecture seule par le principal/RPC : agent regional-wrkr-1, thread1288a673-d16e-473a-80dc-def0b2f776c8, adapter ClaudeAgent, provider claude_glm, modèle glm-5.3-flash. Home effectif /Users/moi/.claude-glm, exécutable gclaude, launchArgs sans flags. Les noms MCP home/projet sont vides. La commande `CLAUDE_CONFIG_DIR=/Users/moi/.claude-glm /Users/moi/.local/bin/claude mcp get bridget`, cwd /Users/moi/Nextcloud/10.Scripts/69.opus2D, sort RC1 et confirme l'absence de Bridget. Sortie assainie64 octets, raw non conservé. T3 injecte seulement t3-code ; ce constat ne signifie pas que GLM est interdit.

<a id="extrait-2"></a>

## Extrait 2

Le lancement identifié ne possède pas le montage MCP Bridget effectif. Un outil interne t3-code ne remplace ni le catalogue Bridget ni une identité attestée. Le problème n'est pas une règle excluant GLM.

<a id="extrait-3"></a>

## Extrait 3

Lecture T3/configuration et commande mcp get seulement. Sortie assainie64 octets, raw non sauvegardé. Aucun restart, modèle, agent actif, communication envoyée ou configuration production modifiés.
