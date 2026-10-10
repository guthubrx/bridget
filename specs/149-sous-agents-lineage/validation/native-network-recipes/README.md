> **Ronde r3 (binaire release r8) : voir la section « Ronde r3 » en fin de fichier.** Le tableau ci-dessous décrit la ronde r2 (binaire debug r5) ; les outils sont les mêmes, `fx.mjs` pointe maintenant vers le release r8.

# Recettes natives réseau 149 - T036 (interop) et T039 (scénarios dégradés) - ronde r2

Propriétaire : testeur Claude Sonnet 5.5 (claudeAgent), ronde natif/réseau r1.
Ces outils ne modifient aucun code de production, aucun autre dossier de recette et aucun service.
Ils n'utilisent aucun modèle.

## Ce que chaque couche est (réel ou simulé)

| Couche | État | Détail |
|---|---|---|
| Daemon Bridget 149 | RÉEL | binaire debug IMMUABLE `bridget-ec6b18b5d468` (sha256 `ec6b18b5…8b0f`), reçu natif r5 (`native149-debug-receipt-r5.json`) |
| Client MCP `bridget mcp`, CLI `bridget lineage` | RÉEL | même binaire, processus possédés par la recette |
| Hôte MCP T3 (HTTP Node, registre d'auth, toolkit, `bridget_session` v1/v2) | RÉEL | modules T3 du worktree, port 14776 |
| Serveur T3 complet `apps/server/src/bin.ts` (base SQLite privée, RPC WebSocket, auth scopée, `BridgetReader`) | RÉEL | port 15736, `T3CODE_BRIDGET_EXECUTABLE` = binaire ci-dessus |
| Fait de permissions du fournisseur (`publishMcpProviderPermissions`) | SIMULÉ (API publique réelle) | publié par la recette à la place de l'adaptateur Codex/Claude, car aucun fournisseur ni modèle ne tourne |
| Projection de conversation T3 pour l'hôte MCP (run actif) | SIMULÉ | en mémoire (même gabarit que `BridgetRustInterop.test.ts`) |
| Wrapper du fil T3 côté daemon (Register + `T3ThreadBindingFact`) | SIMULÉ | trames réelles, émises par la recette ; le credential vient du vrai `Registered` |
| Enfant (Codex) | FERMÉ, sans modèle | `codex149.py` : app-server Codex minimal ; chaque lancement réel consigne PID, argv, noms d'env |

Aucune preuve de modèle, aucune preuve de confinement OS, aucune preuve de la voie standalone GLM/Claude ici.
Ces points restent aux autres propriétaires (T037, T038).

## Fichiers

| Fichier | Rôle |
|---|---|
| `fx.mjs` | fixture : cache privé 0700, projet git sous `/tmp`, daemon, parents, clients MCP, SQL lecture seule, nettoyage SIGTERM vérifié |
| `t3host.ts` | hôte MCP T3 réel (registre, rotation, révocation, publication du fait) |
| `t3server.ts` | serveur T3 complet réel + client RPC WebSocket (copie adaptée de `ui-recipes/rpc149.ts`) |
| `common149.ts` | appels HTTP MCP bruts, faits de politique, attentes |
| `provider149.py`, `codex149.py` | fournisseurs fermés (Claude stream-json / Codex app-server) |
| `interop149.ts` | T036 partie 1 : 43 contrôles (sessions, v1/v2, rotation, révocation, identité, projet, Lineage) |
| `server149.ts` | T036 partie 2 : 15 contrôles (serveur T3 complet <-> daemon natif) |
| `unified149.ts` | T036 partie 3 : 7 contrôles, UNE instance T3 `bin.ts` qui émet le credential MCP (vraie session Codex, faux pair) ET sert Lineage |
| `codexspy.mjs` | espion du faux pair app-server Codex de T3 (consigne les trames reçues, aucun modèle) |
| `recovery149.ts` | T039 : 37 contrôles (six scénarios US6 + panne T3, nested, retry droits, reprise, R9.2 à R9.6 avec vrais SIGTERM du daemon) |
| `fingerprints.py` | empreintes des sources exercées et comparaison au reçu natif |
| `results/*.json` | résultats bruts (sans secret) |
| `results-r1/` | archive des résultats de la ronde r1 (binaire r4 `823e8a5f`) |

## Exécution

```zsh
umask 077
export PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:$PATH
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes
node interop149.ts      # ports 14776 ; ~1,5 min
node server149.ts       # ports 14776 + 15736 ; ~2 min
node recovery149.ts     # port 14776 ; ~3 min
node unified149.ts      # port 15736 ; ~40 s ; aucun hôte MCP simulé
python3 -I fingerprints.py
```

Prérequis : ports 14776 et 15736 libres, `node_modules` (lien vers `apps/server/node_modules` du worktree T3), `sqlite3`.
Un binaire plus récent se rejoue avec `NATIVE149_BIN=/chemin/bridget` (son empreinte est calculée et consignée). `fingerprints.py` compare au reçu r5 (binaire immuable) et situe séparément la dérive FUTURE des manifestes.
Ne lancer qu'une recette à la fois (même port).

## Garde-fous appliqués

- Cache neuf `/Users/moi/.cache/bridget149-native-interop.*` (0700), `umask 077`, projet git privé sous `/tmp/b149n.*` (hors du dépôt `/Users/moi/.git`).
- Le daemon ne reçoit jamais `T3CODE_HOME` (garde BillingGuard) : le runtime T3 est publié dans `HOME/.t3/userdata`.
- Arrêt : SIGTERM sur un PID dont le parent et la commande sont vérifiés, jamais `-9`, jamais `pkill`/`killpg`.
- Aucun secret n'est écrit : les jetons restent en mémoire et dans l'environnement des processus possédés.
- Aucune base, config, socket, registre ou processus de production, ni d'un autre propriétaire, n'est lu ou touché.
- `unified149.ts` impose le faux pair par `settings.json` en CHEMIN ABSOLU et désactive les autres fournisseurs. Un faux `codex` dans le PATH ne suffit PAS : T3 résout le vrai `codex` par le shell de connexion (incident documenté dans `interop149.md`).
- Harnais : `delivery_generation` (u64, 19 chiffres) est lu et renvoyé en chiffres exacts, jamais via `Number`.


## Ronde r3 - binaire release r8

Binaire : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb` (SHA-256 `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6`). Résultats : `results-r3/` ; r2 archivés dans `results-r2/` (index `results-r2/INDEX.md`).
`fx.mjs` : `NATIVE149_BIN` / `NATIVE149_BIN_SHA256` rejouent un autre binaire ; `NATIVE149_RESULTS` change le dossier ; `NATIVE149_CODEX_PROVIDER` choisit le fournisseur fermé (`codex149b.py` en r3). `fx-r2.mjs.txt` et `recovery149-r2.ts` sont les versions r2.

| Fichier | Rôle |
|---|---|
| `run-r3.sh <recette.ts>` | lance une recette entre deux contrôles SHA (journal `results-r3/shacheck.log`) |
| `check-r3.sh` | SHA-256 du binaire (fatal si différent) et empreinte de production (informative) |
| `codex149b.py` | fournisseur fermé r3 : lecture de stdin non bloquante, `HB_149` (battement + `turn/interrupt`), `IGNORE_TERM_149`, `LATE_COMPLETE_149`, `STREAM_149`, journal `sigterm`/`interrupt`/`stdin_eof` |
| `recovery149.ts` | T039 r3 : 43 contrôles, R9.4 sans mutation de base |
| `o4-149.ts` | O4 : durée de vie du fournisseur après SIGTERM du daemon (6 cas, 9 contrôles) |
| `e2e3-149.ts`, `craft-queued.py` | E2 (état durable rejoué, SIMULÉ) et E3 (API + SIGTERM réels) |
| `exec-lag149.ts` | E3d : décalage résultat capturé / exécution close, coupure dans la fenêtre |
| `follow149.ts` | O5/G8 : `bridget lineage inspect --action journal --follow` réel |
| `outage149.ts` | panne de T3 après admission, zéro tour T3 |
| `ui-native149.ts` | tient la fixture de la recette UI web native (serveur T3 + Vite + daemon réel) ; marqueurs `report`, `restart-t3`, `restart-daemon`, `finish` dans `<fixture>/ctl/` |
| `fingerprints-r3.py` | empreintes (binaire, production, dérive, T3, outils) -> `results-r3/fingerprints.json` |

Ports r3 : 14796 (T036), 14797 (O4), 14798 (E2/E3), 14799 (E3d), 14800 (follow), 15756 (T036 serveur), 15757 (panne T3), 15758 et 15778 (UI native).
Exécution : `./run-r3.sh interop149.ts` (idem `server149.ts`, `unified149.ts`, `recovery149.ts`, `o4-149.ts`, `e2e3-149.ts`, `exec-lag149.ts`, `follow149.ts`, `outage149.ts`).
