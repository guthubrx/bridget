# Audit de code — SPEC138 priorité projet

Date : 2026-10-06. Mode demandé : fix ; correction non engagée. Cycle final : readonly. Grille pre-merge, scope diff.

Score global : A — 98.333/100. Zéro CRITICAL, zéro HIGH, cinq MEDIUM ouverts non bloquants.

Couverture : 100% des hunks modifiés, du contexte nécessaire et de la source neuve. Ce n'est pas un audit intégral du dépôt. Les pourcentages JSCPD couvrent des fichiers de contexte entiers.

Périmètre Bridget : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet

Périmètre externe Agent Loop : /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project

## Tendance

Première session pour le scope SPEC138. La session136 existante a un autre périmètre : aucune comparaison de notes ni de fingerprints pertinente. Baseline absente ; aucune suppression. Le cycle-scoring confirme les mêmes cinq MED, sans correction entre cycles.

## Résumé exécutif

| Domaine | Note | Numérique | C/H/M/L |
|---|---|---:|---|
| security | A | 100 | 0/0/0/0 |
| llm_security | A | 100 | 0/0/0/0 |
| supply_chain | A | 100 | 0/0/0/0 |
| observability | A | 100 | 0/0/0/0 |
| reliability | A | 100 | 0/0/0/0 |
| tests | A | 100 | 0/0/0/0 |
| complexity | A | 97 | 0/0/1/0 |
| quality | A- | 88 | 0/0/4/0 |
| architecture | A | 100 | 0/0/0/0 |
| ai_hygiene | A | 100 | 0/0/0/0 |
| duplication | A | 100 | 0/0/0/0 |
| minimalism | A | 100 | 0/0/0/0 |
| performance | N/A | N/A | 0/0/0/0 |
| ux | N/A | N/A | 0/0/0/0 |

Moyenne pondérée : (1770 / 18) = 98,333. Qualité : 100−4×3=88. Complexité : 100−1×3=97. Autres domaines applicables : 100 en l'absence de finding confirmé dans ce diff. Module08 inactif : performance/UX exclus. A sur le diff n'est pas A sur tout le dépôt.

## Findings MEDIUM

### QUAL-001 — Résolution du projet de communication : responsabilité Git imbriquée

Fichier : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication.rs:66

Fingerprint : bea116d53b68. Statut : open. Exploitabilité : N/A. Chemin : hot. Effort : S. Risque de régression : LOW.

resolve_communication_project couvre 61 lignes (66–126), dont le helper local git_common. La validation des faits et la commande Git restent correctes, mais leur lecture dans une même fonction augmente l'effort de revue.

```rust
    fn git_common(path: &std::path::Path) -> Option<std::path::PathBuf> {
        let output = std::process::Command::new("git")
            .args(["-C"])
            .arg(path)
            .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
```

Impact : Lisibilité et coût de maintenance du chemin de rattachement projet.

Remédiation : Sortir le helper local git_common déjà existant. Conserver exactement les validations, les variables supprimées, la canonicalisation et les résultats ; aucune nouvelle abstraction.

Vérification : Rejouer les tests spec138 Git/worktrees/symlinks/homonymes/forgeries et vérifier la forme des faits.

### QUAL-002 — update_task concentre CAS, preuves et publication sous verrou

Fichier : /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py:628

Fingerprint : 5ee8d0e1c9f9. Statut : open. Exploitabilité : N/A. Chemin : hot. Effort : S. Risque de régression : MEDIUM.

La fonction couvre 59 lignes et présente une complexité cyclomatique de 21. Elle mêle comparaison concurrente, anti-rejeu des preuves, archive, publication, commit et reprise après panne. Le verrou commun corrige la course T020 ; il doit rester commun lors de toute extraction.

```python
    with path.with_suffix(".lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        latest = load_json(path, task)
        if expected_statuses is not None and latest.get("status") not in expected_statuses:
            raise SystemExit("task changed concurrently; reload before updating")
        if expected_fields is not None and any(latest.get(key) != value for key, value in expected_fields.items()):
            raise SystemExit("task changed concurrently; reload before updating")
        proof_hash = fields.get("last_progress_hash")
```

Impact : La maintenance du commit et de son rollback devient plus difficile ; aucun défaut fonctionnel supplémentaire n'est démontré.

Remédiation : Séparer les responsabilités CAS/preuves et archive/publication en helpers locaux concrets. Garder le même verrou couvrant résultat et tâche. Ne pas introduire de couche de transaction générale.

Vérification : Rejouer les 152 tests Python, notamment interleave CAS/publication et pannes E/S T020 ; conserver les références canoniques et la reprise bornée.

### QUAL-003 — Préparation Bridget : initialisation et reprise restent entremêlées

Fichier : /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py:1990

Fingerprint : 7aa653414dac. Statut : open. Exploitabilité : N/A. Chemin : hot. Effort : S. Risque de régression : MEDIUM.

prepare_bridget_dispatch couvre 43 lignes et présente une complexité cyclomatique de 23. Les conditions retry pilotent validation, résolution, contexte et enveloppe dans une même branche. La séparation initial/retry rendrait plus explicite l'immuabilité de l'enveloppe durable.

```python
    saved_delivery = task.get("dispatch_delivery", {})
    retry = bool(saved_delivery and task.get("dispatch_delivery_state") == "pending")
    if retry and task.get("status") != "dispatched" and not (
        task.get("status") == "blocked" and task.get("blocked_reason") == "dispatch_transport_failed"
    ):
        raise SystemExit("terminal result prevents dispatch replay")
    target = str(saved_delivery.get("target") if retry else task.get("agent_target") or "").strip()
```

Impact : Coût de revue des reprises et risque de modifier par erreur une enveloppe déjà figée lors d'une évolution.

Remédiation : Clarifier les chemins initial et retry sans recalculer cible, motif, source, corps ou identifiant du retry. Réutiliser les helpers existants ; aucune politique générale.

Vérification : Rejouer les tests de mutation de mandat après failure, reprise à enveloppe figée et vrai Loop/CLI/daemon.

### QUAL-004 — route associe choix du destinataire et transitions d'escalade

Fichier : /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py:3610

Fingerprint : 4a2cf61ce048. Statut : open. Exploitabilité : N/A. Chemin : hot. Effort : S. Risque de régression : MEDIUM.

La closure route couvre 83 lignes et présente une complexité cyclomatique de 25. Elle associe destinataire, motif, garde de projet, blocs de reprise, échec de destinataire, regroupement et escalation. La complexité 90 du parent est une dette de contexte (HEAD89) ; le constat porte sur la branche modifiée.

```python
    def route(item: dict[str, Any]) -> None:
        nonlocal scope_directory
        while True:
            item = {**item, "last_notified_at": tick_at}
            audience = item["audience"]
            if audience == "worker":
                session = session_for_worker_event(item["event"])
            else:
                role = str(orchestrator_role) if audience == "coordinator" else escalation_role
```

Impact : Lecture et maintenance des rappels worker/coordinator/ROOT plus difficiles ; aucun rappel perdu confirmé.

Remédiation : Séparer le calcul concret du destinataire/motif des transitions d'escalade. Préserver les mêmes oracles, le mandat par UUID/rôle et le motif figé par lot.

Vérification : Rejouer les tests des trois rôles mandatés, ROOT hors projet sans mandat, digest de motifs distincts et rappels durables.

### PERF-001 — Coût de parse_snapshot non documenté

Fichier : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/t3code_contract.rs:271

Fingerprint : be476edf1ba6. Statut : open. Exploitabilité : N/A. Chemin : hot. Effort : XS. Risque de régression : LOW.

La fonction publique non triviale ajoute un index des projets et une consultation par thread sans annotation du coût. L'index est une amélioration O(P+T) ; aucun bug de performance n'est démontré.

```rust
pub fn parse_snapshot(text: &str) -> Result<Snapshot, ContractError> {
    const SRC: &str = "GET /api/orchestration/snapshot";
    let value: Value = serde_json::from_str(text).map_err(|_| shape(SRC, "json"))?;
    let snapshot_sequence = value
        .get("snapshotSequence")
        .and_then(Value::as_u64)
        .ok_or_else(|| shape(SRC, "snapshotSequence"))?;
    let mut projects = Vec::new();
    let mut project_roots: HashMap<String, Option<String>> = HashMap::new();
```

Impact : Le lecteur ne dispose pas de la borne explicite du coût JSON, projets et threads.

Remédiation : Ajouter une documentation Big-O : coût du parsing JSON et des données lues/copées, puis index O(P+T) attendu avec HashMap. Préciser P=projets et T=threads.

Vérification : Relire la documentation et conserver les tests de snapshot, références absentes et identifiants ambigus.

## Tests et résultats

WorkspaceV5 : 1633 PASS/0 FAIL/55 ignored ; 78 résumés externes à filtered=0. Python : 152 PASS (124+28). 63 tests138 uniques (35Rust+28Python), sous-ensemble des suites. Recette Loop réelle : 1 PASS après dernière source ; pas ajoutée au total unique. fmt/clippy/release : exit0. Skills validées. BDD : 29 scénarios écrits, non exécutés comme Gherkin. RED historiques et limites : voir /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/audits/2026-10-06/session-2026-10-06-spec-138-01/Controles_Realises.md.

## Minimalisme et frugalité

Potentiel minimalisme : ~0 lignes suppressibles prouvées à comportement constant.

Checklist12 appliquée au diff : aucune abstraction/framework/dépendance supplémentaire non justifiée prouvée ; manifests Cargo inchangés. Le wrapper Claude délègue au canon. Aucun ratio attribué à une origine IA. Les quatre extractions suggérées réorganisent du code concret ; elles ne justifient pas un framework.

## Vertus LLM et responsabilité future

Le réemploi des gardes, reçus et transports limite la charge future. Les branches mêlées signalées par QUAL-001 à004 augmentent le coût de lecture. Le volume est lié aux quatre US et aux tests ; aucune ligne superflue démontrée. Les abstractions réutilisées servent le besoin. Les preuves et limites permettent à l'humain responsable d'expliquer le comportement. Aucun pattern décoratif/stub/framework confirmé dans le diff.

## Duplication et complexité : bornes

JSCPD : Bridget4,029571% (31sources/76832lignes), Loop0,449135% (2sources/4453lignes), pas taux du diff. Top5blocs40/29/28/25/25 ; aucun100+. isNew=false sans baseline ne prouve pas absence de nouveau clone. Dispatch104 contre HEAD113/préextraction139. Parentnotify90 contre89 : dette héritée, hors verdict de hunks. Annotation demandée par PERF-001 relève du module04 ; ce n'est pas un bug de performance.

## Points positifs

- 03 — Le projet source est issu du fait de connexion ; from et domain ne font pas autorité (daemon.rs / communication.rs).
- 04 — parse_snapshot indexe les projets et consulte chaque thread par HashMap ; index attendu O(P+T) (t3code_contract.rs:279).
- 05 — La copie Claude délègue au moteur canonique au lieu de recopier le moteur (scope externe claude/.claude/skills/agent-loop/scripts/agent_loop.py).
- 06 — Le contrôle conserve les warnings dans le résultat durable ; un replay accepté ne réexécute pas l'effet (execution_store.rs / spec138_project_test.rs).
- 06 — CAS, archive et publication partagent le verrou de tâche ; les tests couvrent course et pannes E/S ordinaires (agent_loop.py:638).
- 07 — Aucun bloc de 100 lignes dans les deux rapports JSCPD ; les taux sont des mesures de contexte, pas du diff.
- 12 — Cargo.toml/Cargo.lock et les manifests des crates restent inchangés ; aucun nouveau service, registre ou framework.

## Findings supprimés par baseline

Aucun. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/audits/baseline.json absent, non créé et non modifié.

## Roadmap

Aucun C/H à corriger immédiatement. Court terme : annotation PERF-001 (XS). Moyen terme : QUAL-001 à004 (S) avec les mêmes oracles et verrous. Aucun correctif appliqué par l'audit : modefix demandé mais Phase9 non engagée, WIP et baseline historique rouge. AUTO_COMMIT=false.

## Limites

Même fournisseur pour les revues indépendantes, pas de revue inter-fournisseur. Pas d'analyse intégrale dépôt ni de mesures UX/performance front. Pas de scan CVE déclaré exécuté ni de conformité sécurité globale revendiquée. Garde projet de contexte, pas anti-malveillance. UNKNOWN legacy averti reste possible. T020 borne les pannes E/S ordinaires ; crashmachine/rollbackpanne et anciens backends/writers non convertis hors garantie. release_blocked=false signifie absence de C/H audit ; ce n'est pas une autorisation de déploiement.
