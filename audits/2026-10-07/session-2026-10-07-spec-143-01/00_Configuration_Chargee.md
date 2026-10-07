# Configuration Audit v14.0 — SPEC143

Préparation autorisée avant l'implémentation. Aucun verdict avant le gel du code, les tests et la contre-revue. L'audit ne modifie pas le code. Les défauts prouvés sont transmis au principal et au worker responsable.

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

## État de départ

Le worktree T3 est propre au préflight et sa branche dédiée est session-143-bridget-discreet. HEAD correspond à DIFF_BASE. Le dépôt contient 4 040 fichiers source suivis hors références, dépendances et builds. Ce comptage global ne devient pas une affirmation d'audit global.

La sélection finale est établie après le gel : quatre fichiers,403 lignes ajoutées et24 retirées. Les empreintes sont dans source-freeze.json. Deux MEDIUM ont été transmis puis corrigés par le propriétaire. Le cycle final est readonly. L’auditeur a rejoué346 tests PASS. Le pointeur audits/latest sera mis à jour après validation143.

## Gates et limites

Module 03 non activé par cette grille. Pas de scan CVE, fournisseur, sécurité complète du dépôt, donnée active, installation ou service. Pas de contrôle global du monorepo. Aucun pourcentage de couverture instrumentée ne sera déduit d'un nombre de tests.

La phase de patch du module 09 exige un worktree propre. Si elle est nécessaire après l'implémentation non committée, l'auditeur ne l'exécute pas sur cette source : il transmet les défauts au principal, trace leur traitement par le workflow d'implémentation, puis relit le nouveau gel. Aucun commit automatique ne sera créé pour satisfaire cette précondition.
