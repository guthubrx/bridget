# Configuration Audit v14.0 — SPEC136

AUDIT_SCOPE: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles
AUDIT_ROOT_PATH: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles
SCOPE_MODE: diff
DIFF_BASE: main (43afebbf), changements non committés inclus
AUDIT_MODE: fix (baseline propre cd1c3435 ; pré-audit en lecture)
AUDIT_GRID: all
MODULES_ACTIFS: 00,01,02,03,04,05,06,07,08,09,10,11,12
MAX_CYCLES: 1
MAX_FIXES_PER_CYCLE: 10 (inactif)
MAX_RETRY_PER_FINDING: 2
AUTO_COMMIT: false
SKIP_HOOKS: false
SAMPLING_THRESHOLD: 500
SAMPLING_DAYS: 90
SAMPLING_KEYWORDS: auth,login,crypto,password,secret,token,payment,admin,upload,exec
MIN_SAMPLE_SIZE: 50 (sans échantillonnage sur le diff)
SESSION_DIR: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/audits/2026-10-06/session-2026-10-06-spec-136-01
OUTPUT_FORMATS: json,md
GENERATE_DASHBOARD: false
INCLUDE_EVIDENCE_SNIPPETS: true
BASELINE_PATH: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/audits/baseline.json (absent = vide ; jamais écrit)
SEVERITY_DEFAULTS: CRITICAL SLA24h bloque, HIGH SLA168h bloque, MEDIUM SLA720h ne bloque pas, LOW SLA null ne bloque pas
DUPLICATION_THRESHOLDS: global_warn_pct5,global_crit_pct10,block_warn_lines100,block_crit_lines200
COMPLEXITY_THRESHOLDS: function_warn15,function_crit25,function_max_lines50
PERF_BUDGETS: lcp_ms2500,inp_ms200,cls0.1,bundle_initial_kb200,bundle_total_kb1000 (UI inchangée)
MODE_PARALLELISME: sequentiel
MODELE_SUBAGENTS: inherit (aucun agent lancé)
MAX_FINDINGS_PER_DOMAIN: 50
EXCLUDE_PATHS: node_modules/**,.venv/**,dist/**,build/**,__pycache__/**,.git/**,vendor/**,.next/**,target/**,*.min.js,*.bundle.js,coverage/**,.audit-sessions/**,audits/**,.worktrees/**
EXCLUDE_LANGUAGES: []

Pré-revue en lecture de la transaction136, puis baseline propre cd1c3435
committée par livraison autorisée. Cycle1fix puis scoring readonly ; aucun
candidat CRITICAL/HIGH, aucune correction. Pas de commit ou déploiement par la skill.
Découverte globale, profondeur limitée aux changements136 et leur contexte
immédiat. Ne pas présenter la note comme un audit complet du dépôt.
