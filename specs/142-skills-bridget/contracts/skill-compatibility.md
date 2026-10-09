# Contrat de compatibilité — SPEC142

## Identités publiques

| Appel conseillé | Appel ancien accepté | Comportement conservé |
|---|---|---|
| `$bridget` | `$agent-bridge` | Communication Bridget attestée, périmètre et transport existants |
| `$bridget-loop` | `$agent-loop` | Boucle bornée et politiques existantes |
| `$bridget-handoff` | `$agent-handoff-ledger` | Transmission et journal de contexte existants |

Les alias portent leur ancien `name` et une description explicite de compatibilité. Ils restent invocables. Aucun `user-invocable: false`. Le corps pointe vers le canon avec un chemin absolu stable. L'alias `agent-bridge` interdit `bridge.sh` et l'ancien protocole AgentBridge, même si leurs fichiers sont conservés.

## Chemins préservés

`/Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py` reste inchangé. Le LaunchAgent `/Users/moi/Library/LaunchAgents/local.agent-loop.politique-20261004.plist` reste inchangé. Les nouveaux liens `scripts` sont relatifs au dossier frère historique. Les alias ne remplacent pas leur dossier entier ; un lien live externe n'autorise pas à écraser son moteur.

## Publication

Une source explicite par compétence ; copies identiques d'instructions pour Codex et Claude. Les six noms suivent une branche ciblée du publisher existant. Aucun `PROMOTE` fondé sur mtime ne s'applique à eux. Les autres compétences gardent leur comportement de synchronisation existant.

Préconditions : sources présentes, chemins et cibles exacts vérifiés, sauvegarde hors découverte, configuration active relevée. Postconditions : contenu canon/alias conforme, liens résolus, scripts/config inchangés, sauvegardes pre-104 hors découverte, seconde publication stable. Toute violation bloque l'annonce de publication réussie.

## Références et preuves

Les huit appelants recensés dans `research.md` utilisent les noms canoniques. Les chemins de commandes `agent-loop/scripts/agent_loop.py` restent anciens par contrat. L'inspection ne confond pas un appel documentaire à remplacer et un chemin runtime à conserver.

Une sonde `skills/list` prouve la découverte des fichiers dans son environnement. Elle ne prouve ni l'exécution du modèle, ni la sélection implicite, ni le catalogue Claude. L'existence de trois canons et trois alias peut produire six entrées. Aucun quota de trois entrées UI n'est promis.
