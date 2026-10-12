# Configuration finale Audit157 v14 — scoring

- AUDIT_SCOPE : "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas"
- AUDIT_ROOT_PATH : "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas"
- SCOPE_MODE : "diff"
- DIFF_BASE : "8b5b313694cf05eabfe6a86a970b96f03b015f49"
- AUDIT_MODE : "fix"
- AUDIT_GRID : "all"
- MODULES_ACTIFS : ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12"]
- MAX_CYCLES : 1
- MAX_FIXES_PER_CYCLE : 10
- MAX_RETRY_PER_FINDING : 2
- AUTO_COMMIT : false
- SKIP_HOOKS : false
- SAMPLING_THRESHOLD : 500
- SAMPLING_DAYS : 90
- SAMPLING_KEYWORDS : ["auth", "login", "crypto", "password", "secret", "token", "payment", "admin", "upload", "exec"]
- MIN_SAMPLE_SIZE : 50
- SESSION_DIR : "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/audits/2026-10-12/session-2026-10-12-spec-157-01"
- OUTPUT_FORMATS : ["json", "md"]
- GENERATE_DASHBOARD : false
- INCLUDE_EVIDENCE_SNIPPETS : true
- BASELINE_PATH : "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/audits/baseline.json"
- SEVERITY_DEFAULTS : {"CRITICAL": {"sla_hours": 24, "block_release": true}, "HIGH": {"sla_hours": 168, "block_release": true}, "MEDIUM": {"sla_hours": 720, "block_release": false}, "LOW": {"sla_hours": null, "block_release": false}}
- DUPLICATION_THRESHOLDS : {"global_warn_pct": 5.0, "global_crit_pct": 10.0, "block_warn_lines": 100, "block_crit_lines": 200}
- COMPLEXITY_THRESHOLDS : {"function_warn": 15, "function_crit": 25, "function_max_lines": 50}
- PERF_BUDGETS : {"lcp_ms": 2500, "inp_ms": 200, "cls": 0.1, "bundle_initial_kb": 200, "bundle_total_kb": 1000}
- MODE_PARALLELISME : "auto"
- MODELE_SUBAGENTS : "inherit"
- MAX_FINDINGS_PER_DOMAIN : 50
- EXCLUDE_PATHS : ["node_modules/**", ".venv/**", "dist/**", "build/**", "__pycache__/**", ".git/**", "vendor/**", ".next/**", "target/**", "*.min.js", "*.bundle.js", "coverage/**", ".audit-sessions/**", "audits/**", ".worktrees/**", "cache/**", ".cache/**", "**/cache/**", "**/.cache/**", "*.log", "*.jsonl", "*.patch", "specs/157-catalogue-quotas/validation/*.json"]
- EXCLUDE_LANGUAGES : []
- THEMATIC_MODULES : ["03", "04", "05", "06", "07", "08", "12"]
- FINAL_PASS : {"directory": "cycle-scoring", "mode": "readonly", "same_seven_modules": true, "new_fix_cycle": false}
- PREPARATION_DIR : "/Users/moi/.cache/bridget-audit157-20261012-r1"
- ORCHESTRATION_OVERRIDES : {"authority": "ROOT cycle-scoring03/06 après clôture C1 ; cache seulement.", "repo_clean": "Candidat isolé non committé autorisé ; préparation, empreintes à rafraîchir après Converge.", "commit_owner_phase": "ROOT seul Git/livraison ; AUTO_COMMIT=false.", "validation_owner": "ROOT seul Cargo/tests/services/DB ; aucun test, scanner, build ou benchmark de cette préparation.", "source_capture_stage": "R4 final après correction ROOT : cycle-scoring03/06 lecture ciblée ; C1 figé historique.", "model_choice": "Héritage natif ; aucun agent/model lancé. Routage final à confirmer par ROOT.", "scope_resolution": "29fichiers/16Rust/13docs/105hunks ; validation/implementation/audits horsdénominateur.", "SLO": "SC007 existant seulement ; pas de nouveau SLO absolu proposé.", "routing": "ROOT + deux auditeurs de modules distincts ; aucun sous-agent lancé ici."}

R4 canonique ; C1 conserve les preuvesavantcorrectif. ROOT seul agrège/fingerprint/validator/grade/livre.
