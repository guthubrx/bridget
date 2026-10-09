# Recherche — SPEC142

Date : 2026-10-07. Recherche locale ; aucune exécution de modèle.

## Sources et décisions

| Observation | Source précise | Décision |
|---|---|---|
| Bridget existe déjà et impose une session attestée, le périmètre projet et MCP préférentiel | `/Users/moi/Nextcloud/10.Scripts/64.bridget/skills/bridget/SKILL.md:1` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/094-parite-cli-skill-mcp/plan.md` | Conserver le comportement, pas de second protocole |
| Loop moderne existe dans Codex ; Claude porte une version différente | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/agent-loop/SKILL.md` et `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/.claude/skills/agent-loop/SKILL.md` | Source moderne explicite, aucune sélection mtime |
| Handoff existe dans Codex ; sa cible live peut être un runtime externe | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/agent-handoff-ledger/SKILL.md` ; `/Users/moi/.codex/skills/agent-handoff-ledger` | Canon documentaire réutilisé, scripts/cible live préservés |
| Alias agent-bridge global moderne de 1152 octets ; sources dotfiles et Claude obsolètes utilisent encore le transport historique | `/Users/moi/.codex/skills/agent-bridge/SKILL.md` ; `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/agent-bridge/SKILL.md` ; `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/.claude/skills/agent-bridge/SKILL.md` | Alias court vers Bridget, ne jamais réimporter les instructions legacy |
| Le publisher peut promouvoir une copie plus récente et remplacer un arbre entier | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/sync_codex_skills.sh:38`, `:54`, `:78`, `:86`, `:138` | Branche dédiée avant ce chemin pour les six identités ; pas de suppression d'arbre historique |
| Des copies pre-104 sont dans les racines découvrables | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/agent-bridge.pre-104-20260713-082347/` et `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/.claude/skills/agent-bridge.pre-104-20260713-082347/` | Archiver hors découverte sans supprimer le contenu |
| Le LaunchAgent utilise le chemin ancien toutes les 60 secondes | `/Users/moi/Library/LaunchAgents/local.agent-loop.politique-20261004.plist` | Config et scripts strictement inchangés |
| Le guide local impose un prompt `$skill-name` et une courte description de 25–64 caractères | `/Users/moi/.codex/skills/.system/skill-creator/references/openai_yaml.md` | Métadonnées canoniques françaises, préserver toute policy existante |

## Inventaire des huit fichiers appelants

Racine : `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/`.

| Fichier absolu | Appels à corriger | Éléments à conserver |
|---|---|---|
| `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/codex-trace-observer/SKILL.md` | ligne46 `$agent-handoff-ledger` → `$bridget-handoff` | Règles d'observation |
| `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/session-health-check/SKILL.md` | ligne33 alias handoff → canon | Routine optionnelle |
| `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/horizon-new-episode/SKILL.md` | lignes19/27/31 référence SKILL et nom loop | CLI lignes52/63 `agent-loop/scripts/agent_loop.py` |
| `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/.claude/skills/horizon-new-episode/SKILL.md` | lignes19/27/31 référence SKILL et nom loop | CLI lignes52/63 `agent-loop/scripts/agent_loop.py` |
| `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/horizon-video-fiction/SKILL.md` | ligne130 nom loop | Politique de mission |
| `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/.claude/skills/horizon-video-fiction/SKILL.md` | ligne130 nom loop | Politique de mission |
| `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/horizon-ltx-video-producer/SKILL.md` | ligne53 nom loop | Runner existant |
| `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/.claude/skills/horizon-ltx-video-producer/SKILL.md` | ligne53 nom loop | Runner existant |

## Preuve native de catalogue

Un équipier du principal a exécuté Codex 0.160.1 via `initialize`, `initialized`, `skills/list`, sur `/tmp/bridget-spec142-catalog.jreUv2/probe.cjs`. Sortie exit0. Nouveau nom et ancien alias distincts : deux entrées enabled, affichage humanisé « Bridget Loop — compatibilité ». Même `name` dans deux dossiers : deux entrées de même nom, ancien chemin premier. Deux symlinks vers la même cible : une entrée nouvelle, identité ancienne perdue.

Décision : noms frontmatter distincts et alias courts explicites. Ne pas utiliser `user-invocable: false` : les anciens slash peuvent être refusés. La preuve porte sur la découverte locale, pas sur une exécution du modèle ni un appel payant. Claude ne reçoit pas de revendication équivalente de catalogue natif sans sonde propre.

## Préparation et alternatives rejetées

La sauvegarde de préparation du principal est `/Users/moi/.cache/bridget-skills-142.eTqmgl/`. Elle inclut sources sélectionnées, huit appelants, alias Bridget global et LaunchAgent. Le contrôle après publication reste requis.

Rejetés : moteur dupliqué dans chaque nouveau dossier ; sélection d'autorité par mtime ; alias invisible/non invocable ; second publisher ; suppression des sauvegardes ; changement de chemins CLI ; tests utilisant une mission réelle ou un modèle payant.
