# J2 — Ramasseur de sessions abandonnées — Phase 0 (observateur)

**Statut** : Phase 0 active — observation seule. Aucune action destructive.
**Arbitrage** : Maicie, 2026-08-24 — points (a)–(e) approuvés sans amendement.
**Branche / worktree** : `feat/j2-session-reaper-phase0` /
`.worktrees/feat-j2-session-reaper-phase0`

## Pourquoi

La nuit du 24 a prouvé le besoin (wrappers orphelins, daemons de test, répertoires
temporaires). Les gardes à la destruction des harnais traitent la *cause* ; rien
ne nettoie les *survivants* ni les fuites d'une autre origine. T3 Code refuse de
tuer une session à tour actif **et** une session à travail de fond — leçon
reprise telle quelle. Un ramasseur qui se trompe une fois détruit la confiance :
d'où l'observateur avant l'actif, et « interdit : tuer puis demander pardon ».

## Phase 0 — contrat

1. `bridget reaper report` balaye et **écrit uniquement un rapport**.
2. Aucun signal (`SIGTERM` / `SIGKILL`), aucun `stop`, aucun `unlink` / `rmdir`.
3. Pour chaque cible : identité, critères, gardes, verdict, **action qui aurait
   été prise** si Phase 1 était autorisée.
4. L'activation Phase 1 exige un **arbitrage explicite** après lecture multi-jours
   du journal — jamais un défaut de configuration, jamais un flag caché.

## Verdicts (trois états exclusifs)

| Verdict | Signification |
|---|---|
| `eligible` | Conjonction stricte des critères positifs **et** aucune garde. |
| `protege` | Au moins un signal de vie structuré → on ne touche pas. |
| `incertain` | Preuves incomplètes / contradictoires / absentes → on signale, on ne tue pas. |

Le doute profite **toujours** au processus.

## Classes de cibles

- **A — Wrapper managé** (`bridget managed-wrapper …`)
- **B — Daemon de test / harnais** (`bridget daemon` dont le HOME porte un
  préfixe de harnais `bg-` / `bg909-`)
- **C — Répertoire temporaire** d'origine harnais (`/tmp/bg-*`, `/tmp/bg909-*`,
  `/tmp/mg1504-*`, …) sans propriétaire vivant

## Critères positifs (conjonction)

### Classe A

- **P1** — inactivité apparente (âge processus / silence) au-delà du seuil
  (défaut 30 min) — *hypothèse*, jamais preuve seule
- **P2** — pas de tour actif (`state != busy`)
- **P3** — pas de travail de fond déclaré
- **P4** — orphelinage structurel (au moins un) : `ppid=1`, ou pas de présence
  daemon pour ce nom, ou PID déclaré mort
- **P5** — appartenance Bridget prouvée (binaire `bridget` + `managed-wrapper`)

### Classes B / C

- âge > seuil ; aucun PID vivant rattaché au home / cwd ; marqueur d'origine
  harnais identifiable

## Gardes (une seule suffit → `protege`)

| Id | Garde |
|---|---|
| G1 | Tour actif / `busy` |
| G2 | Travail de fond vivant *(signal Bridget : **absent** — voir trou)* |
| G3 | Connexion daemon `connected` \| `busy` pour ce nom |
| G4 | Mode / localisation tmux vivante (`PresenceMode::Tmux` + localisation) |
| G5 | Participant d'une demande Bridget ouverte |
| G6 | Métadonnée d'âge / last_seen illisible |
| G7 | Déjà `stopped` |
| G8 | Hors préfixe / hors inventaire connu → `incertain`, jamais kill |
| G9 | `managed-wrapper` encore rattaché au daemon (classe B) — tuer le
  parent tuerait l'enfant. Signaux : `ppid == pid` du daemon **ou**
  cmdline qui cite le HOME harnais (après réadoption par PID 1, le ppid
  ment ; le home reste). Doute sur la filiation → pas d'éligibilité. |

## Trou à instruire — signal de travail de fond

**Constat arbitragé** : Bridget n'expose pas aujourd'hui d'équivalent de
`backgroundLiveness` (T3). En Phase 0/1, toute cible de **classe A** qui
atteindrait autrement `eligible` est forcée en **`incertain`**
(`fail_closed_background_liveness_absent`). Ce n'est pas une limite cosmétique :
c'est un **manque à instruire** (capa J2 ↔ éventuel J3). Les classes B et C ne
dépendent pas de ce signal.

## Double observation

Une cible `eligible` n'est marquée `stable_for_phase1` que si le **même**
fingerprint (classe + identité stable) est `eligible` à T0 et T0+Δ
(Δ défaut 5 min), lu depuis l'état durable `~/.cache/bridget/reaper/`.
Une seule photo ne suffit jamais. En Phase 0, `stable_for_phase1` n'autorise
**aucune** action.

## Seuils (propositions, pas dogmes)

- Inactivité / âge : 30 min
- Intervalle de balayage / Δ double photo : 5 min
- Les gardes ne changent pas quand on ajuste les seuils

## Non-objectifs

- Pas de balayage global de la machine hors préfixes Bridget/harnais
- Pas de remplacement des `Drop` / gardes harnais
- Pas de jugement « agent trop lent »
- Pas de `SIGKILL` automatique
- Pas de chemin de code Phase 1 dans ce lot

## Commande

```text
bridget reaper report [--json] [--state-dir DIR] [--tmp DIR] [--min-age-secs N]
```

Sortie : rapport humain ou JSON. Append du snapshot dans
`<state-dir>/observations.jsonl` pour la double photo et la relecture humaine.

## Preuve de Phase 0

- Aucune API publique du module n'envoie de signal ni ne supprime de chemin
- Tests unitaires : classification (gardes, fail-closed A, éligibilité B/C,
  double photo)
- Validation ciblée : `cargo test -p bridget-daemon reaper` + un `report`
  manuel lu sans side-effect
