# Diagnostic MCP Claude/GLM — faits assainis

Date :2026-10-09. Source : contrôles readonly du principal et du propriétaire RPC. Owner documentaire n'a pas rejoué cette commande.

Cause vérifiée en lecture seule par le principal/RPC : agent regional-wrkr-1, thread1288a673-d16e-473a-80dc-def0b2f776c8, adapter ClaudeAgent, provider claude_glm, modèle glm-5.3-flash. Home effectif /Users/moi/.claude-glm, exécutable gclaude, launchArgs sans flags. Les noms MCP home/projet sont vides. La commande `CLAUDE_CONFIG_DIR=/Users/moi/.claude-glm /Users/moi/.local/bin/claude mcp get bridget`, cwd /Users/moi/Nextcloud/10.Scripts/69.opus2D, sort RC1 et confirme l'absence de Bridget. Sortie assainie64 octets, raw non conservé. T3 injecte seulement t3-code ; ce constat ne signifie pas que GLM est interdit.

## Ce que cela prouve

Le lancement identifié ne possède pas le montage MCP Bridget effectif. Un outil interne t3-code ne remplace ni le catalogue Bridget ni une identité attestée. Le problème n'est pas une règle excluant GLM.

## Ce que cela ne prouve pas

Aucune réussite après correction, découverte d'outils dans un modèle actif ou parcours E2E modèle. Le point d'injection sûre, opt-out et permissions restent à fixer par le plan US7 avant ses tâches/code. Aucun secret, contenu de configuration sensible ou valeur ZAI enregistré.

## Absence d'effets

Lecture T3/configuration et commande mcp get seulement. Sortie assainie64 octets, raw non sauvegardé. Aucun restart, modèle, agent actif, communication envoyée ou configuration production modifiés.
