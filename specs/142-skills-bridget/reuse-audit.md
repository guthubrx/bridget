# Audit de reutilisation de l'existant — SPEC142

## Decision

Statut: PASS
Date: 2026-10-07
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/specs/142-skills-bridget

Conclusion courte: Le plan étend les compétences et le publisher existants. Les nouveaux dossiers portent des identités de catalogue, pas des moteurs. Aucun doublon opérationnel ni arbitrage bloquant.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 12 |
| Items audites | 12 |
| Reutilisations deja prevues | 12 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 2 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| 1 Canon Bridget | SKILL source du dépôt | `/Users/moi/Nextcloud/10.Scripts/64.bridget/skills/bridget/SKILL.md:1` | Comportement conservé |
| 2 Canon loop | Instructions Codex modernes | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/agent-loop/SKILL.md:1` | Pas de second moteur |
| 3 Canon handoff | Instructions existantes | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/agent-handoff-ledger/SKILL.md:1` | Pas de nouvelle obligation |
| 4 Alias | Alias Bridget live moderne | `/Users/moi/.codex/skills/agent-bridge/SKILL.md:1` | Étendre le pattern court, pas le transport legacy |
| 5 Métadonnées | Format OpenAI existant | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/codex/.codex/skills/agent-loop/agents/openai.yaml:1` | Noms et prompts ciblés |
| 6 Scripts/config | Arbres historiques et LaunchAgent | `/Users/moi/Library/LaunchAgents/local.agent-loop.politique-20261004.plist` | Chemins et empreintes protégés |
| 7 Publisher | Synchroniseur existant | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/sync_codex_skills.sh:78` | Branche ciblée, pas de second outil |
| 8 Sauvegardes | Mécanisme existant et backup principal | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/sync_codex_skills.sh:64` ; `/Users/moi/.cache/bridget-skills-142.eTqmgl/` | Archive hors découverte |
| 9 Appelants | Huit skills existantes | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/specs/142-skills-bridget/research.md` | Inventaire exact ; CLI préservées |
| 10 Tests ciblés | Paramètres de racines du publisher | `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/sync_codex_skills.sh:4` | Fixture nouvelle justifiée, moteurs non sollicités |
| 11 Découverte | Protocole natif local Codex | `/tmp/bridget-spec142-catalog.jreUv2/probe.cjs` | Exit0, pas d'appel de modèle |
| 12 Documentation | Artefacts SpecKit et workflow utilisateur | `/Users/moi/.speckit/ref/speckit-workflow.md` | Pas de nouveau framework |

## Existant potentiellement pertinent non mentionne

Aucun. Les cibles live externes et copies Claude legacy sont déjà traitées par le plan.

## Duplications evidentes

Aucune. Les nouveaux dossiers correspondent aux nouvelles identités demandées et réutilisent les scripts anciens.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/AGENTS.md` | Session validée et chemins absolus | Worktrees et publication ciblés |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/.specify/memory/constitution.md` | Gates projet | Tests et contrôle avant état final |
| `/Users/moi/.speckit/constitution.md` | Réutilisation, isolation, preuves | Aucun moteur doublé ni contrôle global |
| `/Users/moi/.speckit/ref/standards-tests.md` | Tests proportionnés et résultats réels | Fixtures de synchronisation ciblées |
| `/Users/moi/.codex/skills/.system/skill-creator/references/openai_yaml.md` | Métadonnées et prompt canonique | Format validé, policy préservée |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/094-parite-cli-skill-mcp/plan.md` | Canon Bridget unique | Pas de second transport |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/138-priorite-projet/plan.md` | Périmètre projet existant | Règles modernes conservées |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg --files --hidden -g '!**/.git/**'` | Worktree dotfiles | Sources et sauvegardes pre-104 repérées |
| `rg 'agent-loop|agent-handoff-ledger'` | Appelants ciblés | Huit fichiers, chemins CLI distingués |
| Lecture `sync_codex_skills.sh` | Publisher | Promotion mtime et remplacement d'arbre à contourner |
| Lecture plist et liens live | Racines actives | Runtime actif à préserver |
| Sonde `skills/list` équipier du principal | Fixture Codex 0.160.1 | Identités distinctes nécessaires |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Source loop/handoff | Réutiliser Codex moderne explicite | Claude legacy ne doit pas gagner par date | 2026-10-07 |
| Nouveaux dossiers | Créer identités de présentation | Demande utilisateur, scripts partagés | 2026-10-07 |
| Alias | Garder distincts et invocables | Sonde native et compatibilité slash | 2026-10-07 |
| Tests publisher | Ajouter fixture ciblée | Tests moteurs non équivalents | 2026-10-07 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
