---
name: bridget-usage
description: "Afficher immédiatement les quotas restants de tous les abonnements Bridget (Codex, Antigravity, Claude, Cursor) à 0 jeton LLM."
---

# Quotas restants Bridget (`$bridget-usage` / `/bridget-usage`)

Exécuter **immédiatement** sans lire d'autre documentation :

1. Si l’outil MCP `bridget_usage` est disponible : appeler `bridget_usage` (`provider` si un filtre est demandé : `codex`/`openai`, `antigravity`/`agy`/`google`, `claude`/`anthropic`, `cursor` ; `refresh: true` si `--refresh` est demandé ; `format: "html"` si `html_render` est disponible).
2. Sinon : exécuter `~/.local/bin/bridget usage [fournisseur] [--refresh]` (ou avec `--html` si `html_render` est disponible).
3. Si `html_render` est disponible : mesurer la hauteur avec `html_preview` (largeur `728`) puis afficher directement la page avec `html_render` sans paraphraser les chiffres. Sinon, restituer la sortie complète de `bridget usage` (tous les fournisseurs présents : Codex, Antigravity, Claude, Cursor).
4. Ne jamais inventer de pourcentage ; mentionner `bridget ui usage` si l'utilisateur souhaite ouvrir la console web des quotas.
