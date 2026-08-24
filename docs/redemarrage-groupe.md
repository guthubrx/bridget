# Checklist — redémarrage groupé (macOS / launchd)

Préparé le 2026-08-24 par cursor8. **Le GO reste au référent.**
Aucun `kill`, aucun `launchctl`, aucun restart n’a été exécuté pour rédiger
cette page : seuls un build release et des smoke tests sur base **jetable**.

## État figé au moment de la préparation

| Élément | Valeur |
|---------|--------|
| `main` (tête build) | `285fe78` — *merge: Corriger le boot bloque par le ramasse-copies* (porte `0e5bad4` ; le mandat citait `16caee4` spawn-fantôme — `main` a avancé) |
| Binaire installé | `/Users/moi/Nextcloud/10.Scripts/bridget/target/release/bridget` + `maicie` |
| Build release | `cargo build --release -p bridget-daemon -p maicie` — **OK**, wall **143 s** (~2 min 23 s), aucun rouge |
| CLI neuf | `bridget 0.1.0` ; build-id attendu **`285fe78ac4d4`** |
| Daemon **encore** en vie | build-id **`dfe201af1482`** (périmé) — le CLI le signale déjà |
| `TMPDIR` | `/var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/` — **863** `bridget-*` (659 jeunes / 204 âgés >24 h) |
| `/tmp` | ~3227 `bridget-*` (non scanné par le daemon tant que `TMPDIR` est posé) |
| Disque | WARN CLI : **17,7 Gi** libres (< 20 Gi) — fait attesté, pas de panique auto |

Smoke tests (sans toucher la greffe de production) :

```bash
target/release/bridget --version          # → bridget 0.1.0
# config + sqlite sous /tmp/maicie-jetable-*  — JAMAIS ~/.local/state/maicie/
target/release/maicie status --config /tmp/maicie-jetable-…/config.json
# → objectifs=0 … (RC 0)
```

## 1. Séquence launchd (à exécuter par le référent)

Label daemon : `com.bridget.daemon`  
Plist : `~/Library/LaunchAgents/com.bridget.daemon.plist`  
Programme : `…/target/release/bridget daemon` (`KeepAlive` + `RunAtLoad`)

**Geste canonique (déjà documenté chantier + CLI stale-warning) :**

```bash
launchctl kickstart -k "gui/$(id -u)/com.bridget.daemon"
```

- `-k` = kill puis relance le job : le binaire **neuf** sur disque est repris.
- Pas besoin d’`unload`/`load` si le plist n’a pas changé.
- Si le job n’est plus chargé : `launchctl bootstrap "gui/$(id -u)" ~/Library/LaunchAgents/com.bridget.daemon.plist`

Services **associés** (ne pas les oublier dans le même créneau, sans les confondre) :

| Label | Rôle | Après kickstart daemon |
|-------|------|-------------------------|
| `com.bridget.daemon` | Daemon Bridget | **à redémarrer** (geste ci-dessus) |
| `com.bridget.maicie.releve` | `maicie status --config …` périodique (60 s) | pas obligatoire ; reprend tout seul au prochain tick avec le `maicie` neuf |
| `com.bridget.ronde` | Ronde passive | idem |
| `com.bridget.federation.cartae` | Pont distant | hors de ce GO sauf besoin explicite |

**Interdit pendant le GO :** `kill -9`, tuer Firefox, `kill $(lsof -ti:…)`, toucher `~/.config/maicie/config.json` / greffe prod pour « tester ».

## 2. Qui se reconnecte tout seul

| Population | Mécanisme | Attente |
|------------|-----------|---------|
| **Gérés persistants** (fleet / marqueurs) | Reprise au boot : `reserve_managed_recoveries` + supervisur ; **carte de reprise** injectée au wrapper | Repassent `recovering` puis `connected`/`busy` ; `who` doit les réafficher |
| **Wrappers / leases** | Takeover de lease, recovery commands (`acked` / `prepared` / `seen`) | Pas de respawn manuel si la flotte est saine |
| **MCP / ACP Cursor** | Reconnexion client vers la socket neuve | Sessions Cursor déjà ouvertes se rattachent ; sinon `bridget spawn` |
| **Maicie** | Pas de démon long : chaque `status` / relève ouvre la base et relève le guichet | La file guichet se vide au prochain tick `maicie.releve` |

## 3. Qui vérifier à la main (post-GO)

1. **Présences enrichies** — `bridget who` : transport, mode, domaine, modèle, effort, plage **LIMITE** (P31 / multi-fenêtres si le lot format LIMITE est dans le binaire).
2. **Plages / quotas** — absents trop longs, `BRIDGET_FLEET_QUOTA` (plist = 24), pas de fantômes spawn.
3. **Sondes runtime** — colonnes modèle/effort non vides pour les Claude gérés (équipement MCP) ; pas de trou post-restart sur la sonde.
4. **Carte de reprise** — `bridget reprise` : pertes éventuelles nommées, worktrees, build-id daemon = `285fe78ac4d4`.
5. **Greffe** — `maicie status --config ~/.config/maicie/config.json` (prod, **après** le GO seulement) : fraîcheur / guichet.

## 4. Durée attendue du boot (correctif ramasse-copies)

Chemin critique **après** `0e5bad4` / `285fe78` :

1. `purge_orphan_mcp_configs` (sync, léger) + `warn_if_disk_low` (`statvfs` O(1))
2. `UnixListener::bind` → log « daemon écoute » = **prêt**
3. Thread détaché `ramasse-copies` : filtre d’âge **avant** sonde, **1** `lsof` global, borne **128** entrées âgées / passage

Chiffres machine (2026-08-24 ~19h30, `TMPDIR` ci-dessus) :

| Quantité | Valeur |
|----------|--------|
| Entrées `bridget-*` sous `$TMPDIR` | **863** (mandat : ~854) |
| dont jeunes (<24 h) | **659** → épargnées **sans** `lsof` |
| dont âgées (>24 h) | **204** → **2** passages max (128 + 76) |
| Coût `lsof` unitaire | **variable avec la charge** (jury J2 : ~0,3 s → ~5 s) — **un seul** appel / passage |
| Ready (socket bind) | **secondes** (typ. &lt; 5 s hors contention disque) — **plus** les ~5 min–1 h de l’ancien sync |
| Ménage arrière-plan | 1er passage : 1×lsof + traitement ≤128 âgées ; reste journalisé |

`/tmp` (~3k entrées) n’est balayé **que** si le daemon tourne **sans** `TMPDIR` (launchd actuel n’exporte pas `TMPDIR` dans le plist → selon l’environnement launchd, vérifier quelle racine `std::env::temp_dir()` résout **après** kickstart).

## 5. Point de contrôle post-redémarrage (ordre)

```bash
# 1) Vie + build-id neuf
bridget status
# → Daemon: en ligne ; Build-id daemon: 285fe78ac4d4  (plus de « daemon périmé »)

# 2) Carte
bridget reprise

# 3) Présences
bridget who

# 4) Greffe (prod — après GO seulement)
maicie status --config ~/.config/maicie/config.json
```

Critères de succès rapides :

- [ ] `Build-id daemon` = préfixe de `285fe78`
- [ ] Socket `~/.cache/bridget/bridget.sock` répond
- [ ] Gérés persistants de retour (ou pertes listées dans `reprise`, pas silence)
- [ ] `who` montre les colonnes attendues (LIMITE / modèle) pour un échantillon
- [ ] `maicie status` lit la greffe (pas d’erreur sqlite / chemin)

## 6. Ce que cette préparation a déjà fait / pas fait

| Fait | Pas fait (volontaire) |
|------|------------------------|
| `cargo build --release -p bridget-daemon -p maicie` @ `285fe78` | `launchctl kickstart` |
| Smoke `bridget --version` + `maicie status` sur DB jetable | Tout `kill` / stop d’équipier |
| Copie atomique des binaires vers `target/release/` (chemin plist) | Relève ou écriture greffe **production** |
| Rédaction de cette checklist | Jugement métier sur les ~10 lots activés |

Quand le référent dit GO : exécuter §1, puis §5.
