# Checklist — redémarrage groupé (macOS / launchd)

Préparé le 2026-08-24 par cursor8. **Le GO reste au référent.**
Aucun `kill`, aucun `launchctl`, aucun restart n’a été exécuté *par cette
préparation* pour rédiger cette page : seuls un build release et des smoke
tests sur base **jetable**.

## État figé au moment de la préparation (rectificatif)

| Élément | Valeur |
|---------|--------|
| `main` (tête code pour le binaire) | `a3707b8` — *merge: Attester les limites du referent via le hook statusline* (porte boot-fix `0e5bad4`, spawn-fantôme `a14580a`, statusline) |
| `main` (tête docs au moment du rapport) | `0688143` — docs seule après `a3707b8` ; pas de rebuild requis |
| Mandat initial | citait `16caee4` (équivalent spawn-fantôme sur autre ligne) — **rectifié** vers `a3707b8` par le référent |
| Binaire installé | `/Users/moi/Nextcloud/10.Scripts/bridget/target/release/bridget` + `maicie` (chemin plist) |
| Build release | `cargo build --release -p bridget-daemon -p maicie -p bridget-transport` — **OK**, wall **~77 s**, aucun rouge |
| CLI / daemon build-id | **`a3707b803a52`** (CLI et daemon **alignés** au moment de cette mise à jour — écart périmé résorbé) |
| `TMPDIR` | `/var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/` — **869** `bridget-*` (661 jeunes / 208 âgés >24 h) |
| Disque | WARN CLI : **~19 Gi** libres (< 20 Gi) — saturation récurrente des copies de travail |

Smoke tests (sans toucher la greffe de production) :

```bash
target/release/bridget --version          # → bridget 0.1.0
# config + sqlite sous /tmp/maicie-jetable-*  — JAMAIS ~/.local/state/maicie/
# ni ~/.config/maicie/config.json
target/release/maicie status --config /tmp/maicie-jetable-…/config.json
# → objectifs=0 … (RC 0)
printf '{}' | target/release/bridget hook claude-statusline   # exit 0, stdout vide
```

## 1. Séquence launchd (à exécuter par le référent)

Label daemon : `com.bridget.daemon`  
Plist : `~/Library/LaunchAgents/com.bridget.daemon.plist`  
Programme : `…/target/release/bridget daemon` (`KeepAlive` + `RunAtLoad`)  
Quota flotte : `BRIDGET_FLEET_QUOTA=24` (déjà dans le plist)

**Geste canonique (déjà documenté chantier + CLI stale-warning) :**

```bash
launchctl kickstart -k "gui/$(id -u)/com.bridget.daemon"
```

- `-k` = kill puis relance le job : le binaire **sur disque** est repris.
- Pas besoin d’`unload`/`load` si le plist n’a pas changé.
- Si le job n’est plus chargé : `launchctl bootstrap "gui/$(id -u)" ~/Library/LaunchAgents/com.bridget.daemon.plist`

**Avant kickstart :** vérifier `bridget status`. Si `Build-id daemon` = déjà `a3707b803a52`
et que les ~10 lots attendus sont visibles dans `who` / greffe, le GO peut être
un no-op ou un kickstart de confirmation — à trancher par le référent.

Services **associés** (même créneau, sans les confondre) :

| Label | Rôle | Après kickstart daemon |
|-------|------|-------------------------|
| `com.bridget.daemon` | Daemon Bridget | **geste ci-dessus** |
| `com.bridget.maicie.releve` | `maicie status --config …`, puis dispatcher global des rondes, périodique (120 s) | reprend au prochain tick avec les binaires neufs |
| `com.bridget.ronde` | Ronde passive | idem |
| `com.bridget.federation.cartae` | Pont distant | hors de ce GO sauf besoin explicite |

**Interdit pendant le GO :** `kill -9`, tuer Firefox, `kill $(lsof -ti:…)`,
toucher `~/.config/maicie/config.json` / greffe prod pour « tester ».

## 2. Qui se reconnecte tout seul

| Population | Mécanisme | Attente |
|------------|-----------|---------|
| **Gérés persistants** (fleet / marqueurs) | Reprise au boot : réservations + superviseur ; **carte de reprise** injectée au wrapper | Repassent `recovering` puis `connected`/`busy` ; `who` doit les réafficher |
| **Wrappers / leases** | Takeover de lease, recovery (`acked` / `prepared` / `seen`) | Pas de respawn manuel si la flotte est saine |
| **MCP / ACP Cursor** | Reconnexion client vers la socket neuve | Sessions Cursor ouvertes se rattachent ; sinon `bridget spawn` |
| **Maicie** | Pas de démon long : chaque `status` / relève ouvre la base et relève le guichet | File guichet au prochain tick `maicie.releve` |
| **Colonne LIMITE (statusline)** | Hook sans cache : republie à chaque rafraîchissement StatusLine | Se remplit **toute seule** au 1er tour post-restart |

## 3. Qui vérifier à la main (post-GO)

1. **Présences enrichies** — `bridget who` : transport, mode, domaine, modèle, effort, plage **LIMITE**.
2. **Plages P31** — `maicie plage list --config …` : réservations cohérentes, pas de fantôme.
3. **Sondes runtime** — colonnes modèle/effort non vides pour un échantillon Claude gérés ; pas de trou post-restart (domaine/mode/transport).
4. **Carte de reprise** — `bridget reprise` : pertes éventuelles nommées (`vivant.pertes_reprise`), worktrees, build-id = `a3707b803a52`.
5. **Greffe** — `maicie status --config ~/.config/maicie/config.json` (**après** le GO seulement).
6. **Quota** — absents non amputés en silence (D20) ; `BRIDGET_FLEET_QUOTA=24`.

## 4. Durée attendue du boot (correctif ramasse-copies)

Chemin critique **après** `0e5bad4` (dans `a3707b8`) :

1. `purge_orphan_mcp_configs` (sync, léger) + `warn_if_disk_low` (`statvfs` O(1))
2. `UnixListener::bind` → log « daemon écoute » = **prêt**
3. Thread détaché `ramasse-copies` : filtre d’âge **avant** sonde, **1** `lsof` global, borne **128** entrées âgées / passage (`MAX_ENTRIES_PER_PASS`)

Chiffres machine (2026-08-24 ~19h40, `TMPDIR` ci-dessus) :

| Quantité | Valeur |
|----------|--------|
| Entrées `bridget-*` sous `$TMPDIR` | **869** (mandat historique ~854) |
| dont jeunes (<24 h) | **661** → épargnées **sans** `lsof` |
| dont âgées (>24 h) | **208** → **2** passages max (128 + 80) |
| Coût `lsof` | **variable avec la charge** (jury J2 : ~0,3 s → ~5 s) — **un seul** appel / passage |
| Ready (socket bind) | **secondes** (typ. &lt; 5 s hors contention disque) — **plus** les ~5 min–1 h de l’ancien sync |
| Ménage arrière-plan | 1er passage : 1×lsof + ≤128 âgées ; reste journalisé / passage suivant |

Le daemon scanne `std::env::temp_dir()` (= `$TMPDIR` sur macOS), **pas** `/tmp`
global. Le plist launchd n’exporte pas `TMPDIR` explicitement : après kickstart,
confirmer quelle racine est résolue si le comportement surprend.

## 5. Point de contrôle post-redémarrage (ordre)

```bash
# 1) Vie + build-id
bridget status
# → Daemon: en ligne ; Build-id daemon: a3707b803a52  (pas de « daemon périmé »)

# 2) Carte
bridget reprise

# 3) Présences
bridget who

# 4) Greffe (prod — après GO seulement)
maicie status --config ~/.config/maicie/config.json
```

Critères de succès rapides :

- [ ] `Build-id daemon` = préfixe de `a3707b8`
- [ ] Socket `~/.cache/bridget/bridget.sock` répond
- [ ] Gérés persistants de retour (ou pertes listées dans `reprise`, pas silence)
- [ ] `who` montre LIMITE / modèle / domaine / mode pour un échantillon
- [ ] `maicie status` lit la greffe (pas d’erreur sqlite / chemin)

## 6. Ce que cette préparation a déjà fait / pas fait

| Fait | Pas fait (volontaire) |
|------|------------------------|
| `cargo build --release` @ `a3707b8` (daemon + maicie + transport/CLI) | `launchctl kickstart` *par cursor8* |
| Smoke `bridget --version` + `maicie status` DB jetable + hook statusline | Tout `kill` / stop d’équipier |
| Binaires dans `target/release/` (chemin plist) | Relève ou écriture greffe **production** |
| Rédaction / mise à jour de cette checklist | Jugement métier sur les ~10 lots activés |

Quand le référent dit GO : exécuter §1 (si encore nécessaire), puis §5.
