# Configuration Audit v14.0

Horodatage de production : 2026-10-06T18:56:40Z. Les scans et revues ont eu lieu dans le pipeline préalable.

- AUDIT_SCOPE : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet
- EXTERNAL_SCOPE : /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project
- SCOPE_MODE : diff
- DIFF_BASE : e8ed4d62
- EXTERNAL_DIFF_BASE : 5fc64e37
- AUDIT_MODE : fix
- AUDIT_GRID : pre-merge
- MODULES_ACTIFS : ["00","01","02","03","04","05","06","07","09","10","11","12"]
- MAX_CYCLES : 1
- MAX_FIXES_PER_CYCLE : 10
- MAX_RETRY_PER_FINDING : 2
- AUTO_COMMIT : false
- SKIP_HOOKS : false
- SAMPLING_THRESHOLD : 500
- SAMPLING_DAYS : 90
- SAMPLING_KEYWORDS : ["auth","login","crypto","password","secret","token","payment","admin","upload","exec"]
- MIN_SAMPLE_SIZE : 50
- OUTPUT_FORMATS : ["json","md"]
- GENERATE_DASHBOARD : false
- INCLUDE_EVIDENCE_SNIPPETS : true
- BASELINE_PATH : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/audits/baseline.json
- SEVERITY_DEFAULTS : {"CRITICAL":{"sla_hours":24,"block_release":true},"HIGH":{"sla_hours":168,"block_release":true},"MEDIUM":{"sla_hours":720,"block_release":false},"LOW":{"sla_hours":null,"block_release":false}}
- DUPLICATION_THRESHOLDS : {"global_warn_pct":5,"global_crit_pct":10,"block_warn_lines":100,"block_crit_lines":200}
- COMPLEXITY_THRESHOLDS : {"function_warn":15,"function_crit":25,"function_max_lines":50}
- PERF_BUDGETS : {"lcp_ms":2500,"inp_ms":200,"cls":0.1,"bundle_initial_kb":200,"bundle_total_kb":1000}
- MODE_PARALLELISME : sequentiel
- MODELE_SUBAGENTS : inherit
- MAX_FINDINGS_PER_DOMAIN : 50
- EXCLUDE_PATHS : ["node_modules/**",".venv/**","dist/**","build/**","__pycache__/**",".git/**","vendor/**",".next/**","target/**","*.min.js","*.bundle.js","coverage/**",".audit-sessions/**","audits/**",".worktrees/**"]
- EXCLUDE_LANGUAGES : []
- SESSION_DIR : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/audits/2026-10-06/session-2026-10-06-spec-138-01

## Exceptions explicites

Contexte source de découverte : 177 fichiers Rust/Python, hors target/.git/.worktrees/audits/specs. Cette mesure n'est pas la couverture du dépôt. Le périmètre final approfondi comprend 56 fichiers de diff et artefacts utiles, dont35 sources, après ajout et lecture de convergence-report.md et validation/results.json.

Mode demandé `fix`, AUTO_COMMIT=false. Phase 9 corrections non engagée : worktrees WIP isolés, baseline historique rouge au démarrage et aucun candidat CRITICAL/HIGH. Les cinq MED restent ouverts. Cycle-scoring final en lecture seule. Aucune prétention de dépôt propre ni de correction appliquée par l'audit. Le principal a exécuté Converge manuellement, du 18:52:52 au 18:56:19 UTC.
