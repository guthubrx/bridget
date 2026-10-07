# Configuration Audit v14.0 — SPEC141

Audit en lecture seule du diff de présentation T3. Les artefacts sont les seules écritures autorisées. Aucun commit, correction de source, téléchargement, déploiement ou redémarrage.

- AUDIT_ROOT_PATH : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes
- AUDIT_SCOPE : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped
- SCOPE_MODE : diff
- DIFF_BASE : 9706acbde648e581d4c41302a0f4ce8e4ac20a9d
- AUDIT_MODE : readonly
- AUDIT_GRID : pre-merge, avec sélection explicite des modules de sécurité et de performance frontend
- MODULES_ACTIFS : 00, 01, 02, 03, 04, 05, 06, 07, 08, 10, 11, 12
- MAX_CYCLES : 1
- MAX_FIXES_PER_CYCLE : 10, inopérant en lecture seule
- MAX_RETRY_PER_FINDING : 2, inopérant en lecture seule
- AUTO_COMMIT : false
- SKIP_HOOKS : false
- SESSION_DATE : 2026-10-07
- SESSION_ID : session-2026-10-07-spec141-01
- SESSION_DIR : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/audits/2026-10-07/session-2026-10-07-spec141-01
- BASELINE_PATH : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/audits/baseline.json ; absent, suppressions vides
- SAMPLING_THRESHOLD : 500
- SAMPLING_DAYS : 90
- SAMPLING_KEYWORDS : auth, login, crypto, password, secret, token, payment, admin, upload, exec
- MIN_SAMPLE_SIZE : 50 pour un audit global ; dérogation de périmètre explicite : quatre fichiers du diff141 seulement
- OUTPUT_FORMATS : json, md
- GENERATE_DASHBOARD : false
- INCLUDE_EVIDENCE_SNIPPETS : true
- MODE_PARALLELISME : sequentiel dans cet agent ; contre-revue locale indépendante coordonnée par le principal
- MODELE_SUBAGENTS : inherit ; aucun agent enfant créé
- MAX_FINDINGS_PER_DOMAIN : 50
- EXCLUDE_PATHS : node_modules/**, .venv/**, dist/**, build/**, __pycache__/**, .git/**, vendor/**, .next/**, target/**, *.min.js, *.bundle.js, coverage/**, .audit-sessions/**, audits/**, .worktrees/**, .repos/**
- EXCLUDE_LANGUAGES : aucun
- SEVERITY_DEFAULTS : CRITICAL SLA24h bloque ; HIGH SLA168h bloque ; MEDIUM SLA720h ne bloque pas ; LOW SLA non fixé ne bloque pas
- DUPLICATION_THRESHOLDS : global_warn_pct5 ; global_crit_pct10 ; block_warn_lines100 ; block_crit_lines200
- COMPLEXITY_THRESHOLDS : function_warn15 ; function_crit25 ; function_max_lines50
- PERF_BUDGETS : lcp_ms2500 ; inp_ms200 ; cls0.1 ; bundle_initial_kb200 ; bundle_total_kb1000

## Limites obligatoires

La découverte distingue le monorepo du changement : 4 040 fichiers source suivis hors références, dépendances et builds. Le scan thématique porte sur les régions modifiées des quatre fichiers et leurs appelants pertinents. La note ne décrit pas la qualité globale du monorepo.

Les tests ciblés sont exécutés par le principal et l'agent d'implémentation. Aucun pourcentage de couverture instrumentée n'est déduit de leur nombre. Aucun scan CVE, Lighthouse, benchmark réseau, fournisseur ou production n'est lancé par cet audit. Les domaines non évalués restent explicitement hors périmètre.

Le premier gel a été levé après deux défauts confirmés. Son scan JSCPD est provisoire. Les hashes et le verdict finaux doivent être relevés après le gel suivant.
