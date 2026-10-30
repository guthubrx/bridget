# 207 — Durée explicite et worktrees économes

> Mise à jour221/C18 : le conseil courant réutilise git checkout-index sans --force. Il conserve un chemin déjà présent. L'ancienne commande git restore ci-dessous décrit l'état initial207 et ne doit plus servir de consigne de matérialisation ; voir les guides de délégation et la validationC18 de la session221.

Correction demandée par monuments, message mcp-72210-6ae2e112-9.

La durée reply_timeout existe : défaut 3600, limites 60–86400 et limite parent inchangées. Expliquer son réglage avant admission et la durée effective au refus.

Un dépôt peut activer administrativement bridget.worktreeProfile=compact-v1. Git worktreeConfig doit déjà être activé. Les nouvelles extractions incluent les fichiers de moins de 128 Kio, les sources et les liens Git ; les gros fichiers restent disponibles dans Git et se matérialisent privément avec git restore --ignore-skip-worktree-bits. Aucun script de projet exécuté, aucun lien partagé de données. Les worktrees existants et reçus restent inchangés.

Le mode utilise la sélection partielle persistante de Git par worktree. Le rapport refuse une configuration ou un index qui masquerait une modification. Les reçus restent au format existant.
