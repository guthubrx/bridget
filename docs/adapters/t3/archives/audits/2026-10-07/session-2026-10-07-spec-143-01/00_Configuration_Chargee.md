# Archives techniques de l’adaptateur T3

Document d’origine : [audits/2026-10-07/session-2026-10-07-spec-143-01/00_Configuration_Chargee.md](../../../../../../../audits/2026-10-07/session-2026-10-07-spec-143-01/00_Configuration_Chargee.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- AUDIT_ROOT_PATH : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets
- AUDIT_SCOPE : /Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet
- SCOPE_MODE : diff
- DIFF_BASE : 5724eb7f12e4556327efa14f51f43b9caf404e25
- AUDIT_MODE : fix demandé par le pipeline, corrections éventuelles exécutées par le worker ; aucune correction directe par cet agent
- AUDIT_GRID : pre-merge avec module 08 pour l'interface
- MODULES_ACTIFS : 00, 01, 02, 04, 05, 06, 07, 08, 09, 10, 11, 12
- MAX_CYCLES : 1, puis cycle final de scoring readonly
- MAX_FIXES_PER_CYCLE : 10
- MAX_RETRY_PER_FINDING : 2
- AUTO_COMMIT : false, override explicite du pipeline my-specify-all
- SKIP_HOOKS : false
- SESSION_DATE : 2026-10-07
- SESSION_ID : session-2026-10-07-spec-143-01
- SESSION_DIR : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/audits/2026-10-07/session-2026-10-07-spec-143-01
- BASELINE_PATH : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/audits/baseline.json ; absent au démarrage
- SAMPLING_THRESHOLD : 500
- SAMPLING_DAYS : 90
- SAMPLING_KEYWORDS : auth, login, crypto, password, secret, token, payment, admin, upload, exec
- MIN_SAMPLE_SIZE : 50 pour le mode global ; périmètre explicitement restreint aux régions du diff143 et au contexte des appelants
- OUTPUT_FORMATS : json, md
- GENERATE_DASHBOARD : false
- INCLUDE_EVIDENCE_SNIPPETS : true
- MODE_PARALLELISME : sequentiel dans cet agent ; revue indépendante pilotée par le principal
- MODELE_SUBAGENTS : inherit ; aucun agent enfant prévu
- MAX_FINDINGS_PER_DOMAIN : 50
- EXCLUDE_PATHS : node_modules/**, .venv/**, dist/**, build/**, __pycache__/**, .git/**, vendor/**, .next/**, target/**, *.min.js, *.bundle.js, coverage/**, .audit-sessions/**, audits/**, .worktrees/**, .repos/**
- EXCLUDE_LANGUAGES : aucun
- SEVERITY_DEFAULTS : CRITICAL SLA 24 h bloque ; HIGH SLA 168 h bloque ; MEDIUM SLA 720 h ne bloque pas ; LOW sans SLA fixé ne bloque pas
- DUPLICATION_THRESHOLDS : global_warn_pct 5 ; global_crit_pct 10 ; block_warn_lines 100 ; block_crit_lines 200
- COMPLEXITY_THRESHOLDS : function_warn 15 ; function_crit 25 ; function_max_lines 50
- PERF_BUDGETS : lcp_ms 2500 ; inp_ms 200 ; cls 0.1 ; bundle_initial_kb 200 ; bundle_total_kb 1000

<a id="extrait-2"></a>

## Extrait 2

Le worktree T3 est propre au préflight et sa branche dédiée est session-143-bridget-discreet. HEAD correspond à DIFF_BASE. Le dépôt contient 4 040 fichiers source suivis hors références, dépendances et builds. Ce comptage global ne devient pas une affirmation d'audit global.
