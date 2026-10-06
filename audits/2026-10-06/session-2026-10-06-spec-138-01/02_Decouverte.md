# Découverte et périmètre

Bridget : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet

Scope externe : /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project

Comparaisons : Bridget e8ed4d62, Dotfiles 5fc64e37. 35 fichiers Bridget suivis, quatre fichiers externes suivis et une source de test nouvelle. Seize nouveaux artefacts auxiliaires complètent le dossier de session : les deux artefacts finaux convergence-report.md et validation/results.json ont été ajoutés et relus. Total final56 fichiers utiles,35 sources. Les preuves générées ne sont pas du code audité.

Contexte Bridget : 177 fichiers Rust/Python trouvés, hors target/.git/.worktrees/audits/specs. Le scan approfondi porte sur les hunks modifiés, leur contexte et la source neuve entière. Couverture 100% de ce périmètre borné ; couverture dépôt non mesurée.

Daemon Rust local, CLI/MCP, SQLite et transports existants. Agent Loop Python réutilise le moteur canonique. Manifestes Cargo et verrou Cargo inchangés. Aucune interface web ni conteneur/CI détecté dans la découverte requise.

## Réemploi et risques

Pas de registre ProjectReference réactivé. Les faits projet sont vivants et attestés. Les domaines restent cosmétiques. Les fils sont visibles par tous les membres, même notify=[]. Le client ancien reste UNKNOWN ; la nouvelle capacité refuse le motif sur serveur ancien. Risques étudiés : forging, mélange d'audience, idempotence, contrôle injecté, mandats et concurrence de publication.
