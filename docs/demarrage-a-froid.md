# Démarrage à froid (référent sans contexte)

Une page pour reprendre **sans mémoire**. Tout ce qui suit a été exécuté sur
la machine le 2026-08-24 ; un écart observé est noté comme découverte.

## Chemins vivants (sinon on déclare l’équipe morte à tort)

- Socket daemon : `/Users/moi/.cache/bridget/bridget.sock`
- CLI Bridget : `bridget` → `…/target/release/bridget`
- Config Maicie (obligatoire, aucun défaut) : `/Users/moi/.config/maicie/config.json`
- Greffe Maicie : `database_path` de cette config (pas `bridget.db`)
- Catalogue du dû : `docs/catalogue-du-du.md` (`catalogue_path` de la config)

## 1. Où regarder d’abord

| Ordre | Commande | Ce qu’on y lit |
|------:|----------|----------------|
| 1 | `bridget reprise` | Carte : daemon vivant ?, socket, DB, agents, worktrees, dirty. Si le dernier redémarrage a laissé des absents, le bloc `vivant.pertes_reprise` les nomme (`reason` + `detail`) — porte d'entrée : [data-model § Trace de reprise](../specs/009-daemon-spawn/data-model.md). Option `--write <chemin>` pour figer. |
| 2 | `bridget who` | Présence runtime : transport, mode, domaine, modèle, effort, **LIMITE** (si attestée), état `connected`/`busy`. |
| 3 | `maicie status --config /Users/moi/.config/maicie/config.json` | Greffe : une ligne `objectifs=N … coordination_fraîcheur=…`. `--json` pour le détail. |
| 4 | `maicie registre list --config …` | Constats **ouverts** + pied `N/M/K/P`. Ajouter `--attente` pour les pending. |
| 5 | `bridget ui --maicie-config /Users/moi/.config/maicie/config.json` | Relais **lecture seule** loopback ; imprime `http://127.0.0.1:<port>/?token=…`. Sans jeton → HTTP 403. |

**Découvertes :** (a) `bridget ui` sans `--maicie-config` absolu refuse ; (b) `maicie` sans sous-commande → `commande inconnue : delegate attendu` (pas de `--help`) ; (c) `status` peut afficher `fraîcheur=unavailable` / `snapshot_transport=unknown` avec motif `budget_capture_non_configure` — le greffe reste lisible (`coordination_fraîcheur=fresh`), seule la capture Bridget est aveugle.

## 2. Comment lire

**Deux vérités (ne se remplacent pas).** Maicie SQLite = objectifs, délégations, décisions. Bridget (`who`, socket) = présence et transport. Un agent absent de `who` n’annule pas une mission au greffe ; un `busy` n’ouvre pas un objectif.

**États observés (status --json, même instant).** Objectifs : `clos` 181, `en_coordination` 10. Délégations : `creee` 176, `a_evaluer` 15 — et ces 15 `a_evaluer` sont sur des objectifs **déjà `clos`**.  
**Découverte (corrigée côté solde ; le compteur reste documenté) :** `bridget-ronde`
compte les *objectifs* `a_evaluer` (`objectifs_a_evaluer=N` dans le résumé — pas les
délégations). Traiter les délégations `a_evaluer` via `status --json` / greffe.

**Registre.** Entrées ouvertes (sévérité déclarée) + pied déterministe. Ce n’est pas une todo list inventée : c’est le journal du dû versionné.

## 3. Gestes sûrs du premier quart d’heure

1. `bridget status` puis `bridget reprise` — prouver la vie (socket + build-id).
2. `maicie status --config …` — **relève le guichet** au passage (réconciliation au démarrage de toute commande qui ouvre la base). Pas de sous-commande `relever` ; `bridget guichet` hors dépôt = erreur « seule opération … deposer ». Observé : file guichet `replied` 17, `rejected` 1, **0** `queued`.
3. Lire les missions `en_coordination` et les délégations `a_evaluer` (`--json`) ; croiser avec `who` (limites / busy) avant de relancer quelqu’un.
4. `maicie registre list --config …` — noter les majors/blockers ouverts avant toute suite.
5. `bridget-ronde --config …` — constat passif (agents, demandes, registre) ; **aucune décision**.

### Ne pas faire sans comprendre

- **`maicie profile approve`** — frappe humaine (TTY / ADR 011) ; un agent ne l’exécute jamais.
- **`kill` / `bridget stop <nom>`** — arrête un géré ; exiger un motif et un nom exact. `bridget reaper` = **observer** seulement.
- **Éditer la config Maicie ou le registre d’agents** — décision de sécurité, pas de confort.
- **Inventer** des `objective_id` / `delegation_id` pour le guichet — les trois ids viennent du `delegate`.
- **Déduire le chantier depuis `who` seul** — présence ≠ greffe.

## 4. Doctrine complète

- Chantier multi-agents : `docs/regles-chantier.md`
- Journal du dû (v2 machine) : `docs/catalogue-du-du.md`
- Décisions : `docs/decisions/` (surtout ADR 003 Maicie, 007 idempotence, 009 GUI, 010 ponts, 011 approbation)
- Skill Maicie : deux vérités + `registre list` en tête de session
