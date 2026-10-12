# Configuration Audit v14 — préparation158A

Préparation seule. Aucun audit thématique, finding, scoring ou grade produit. ROOT seul propriétaire des mutations.

- AUDIT_SCOPE : "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle"
- SCOPE_MODE : "diff"
- DIFF_BASE : "bd37e927eee0b28f058df9d6a6992e38665a6ad0"
- AUDIT_MODE : "fix"
- AUDIT_GRID : "all"
- MODULES_ACTIFS : ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12"]
- THEMATIC_MODULES : ["03", "04", "05", "06", "07", "08", "12"]
- MAX_CYCLES : 1
- MAX_FIXES_PER_CYCLE : 10
- MAX_RETRY_PER_FINDING : 2
- AUTO_COMMIT : false
- SKIP_HOOKS : false
- SAMPLING_THRESHOLD : 500
- SAMPLING_DAYS : 90
- SAMPLING_KEYWORDS : ["auth", "login", "crypto", "password", "secret", "token", "payment", "admin", "upload", "exec"]
- MIN_SAMPLE_SIZE : 50
- SESSION_DIR : "/Users/moi/.cache/bridget-audit158-20261012-r1"
- OUTPUT_FORMATS : ["json", "md"]
- GENERATE_DASHBOARD : false
- INCLUDE_EVIDENCE_SNIPPETS : true
- BASELINE_PATH : "/Users/moi/Nextcloud/10.Scripts/64.bridget/audits/baseline.json"
- SEVERITY_DEFAULTS : {"CRITICAL": {"sla_hours": 24, "block_release": true}, "HIGH": {"sla_hours": 168, "block_release": true}, "MEDIUM": {"sla_hours": 720, "block_release": false}, "LOW": {"sla_hours": null, "block_release": false}}
- DUPLICATION_THRESHOLDS : {"global_warn_pct": 5.0, "global_crit_pct": 10.0, "block_warn_lines": 100, "block_crit_lines": 200}
- COMPLEXITY_THRESHOLDS : {"function_warn": 15, "function_crit": 25, "function_max_lines": 50}
- PERF_BUDGETS : {"lcp_ms": 2500, "inp_ms": 200, "cls": 0.1, "bundle_initial_kb": 200, "bundle_total_kb": 1000}
- MODE_PARALLELISME : "agents"
- MODELE_SUBAGENTS : "inherit"
- MAX_FINDINGS_PER_DOMAIN : 50
- EXCLUDE_PATHS : ["node_modules/**", "target/**", ".worktrees/**", "cache/**", ".cache/**", "audits/**", ".audit-sessions/**", ".git/**", ".venv/**", "dist/**", "build/**", "__pycache__/**", "vendor/**", ".next/**", "coverage/**", "*.min.js", "*.bundle.js"]
- EXCLUDE_LANGUAGES : []

Les budgets frontend sont des valeurs de configuration inapplicables au périmètre sans UI. Aucun SLO nouveau proposé. Le delta autorisé en cours explique le repo non propre ; aucune mutation Git par la préparation.
