# Interop réseau natif 149 - T036, O1, O5/G8, panne T3 - ronde r3 (binaire release r8)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production, aucun modèle, aucun Cargo.
Rapport r2 archivé : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149-r2.md` (binaire debug r5, 65/65). Rapport r1 : `.../interop149-r1.md`.

## Verdict : APPROVE sur le périmètre exécuté (T036 rejoué : 65/65 PASS, deux fois)

| Recette | Contrôles | Résultat brut |
|---|---|---|
| `interop149.ts` : hôte MCP de T3 réel + daemon natif | 43/43 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r3/interop149.json` |
| `server149.ts` : serveur T3 complet (`bin.ts`) + daemon natif | 15/15 PASS | `.../results-r3/server149.json` |
| `unified149.ts` : UNE instance T3 qui émet le credential ET sert Lineage | 7/7 PASS | `.../results-r3/unified149.json` |

Première exécution : 19:00 à 19:02. Rejeu final avec les outils définitifs (après ajout du second modèle et de `execs()` dans `fx.mjs`) : 19:46 à 19:48. Les deux donnent 65/65. La première est archivée dans `.../results-r3/t036-premiere-passe/`. Les scopes T3 sont inchangés ; je n'ai pas relancé de suite globale.

## Binaire et sources

- Binaire release r8 : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb`, SHA-256 `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6` recalculé avant et après chaque recette (journal `.../results-r3/shacheck.log`, toujours égal au reçu `native149-release-receipt-r8.json`). Debug r8 `833a030545c3` non utilisé.
- Sources T3 : `git diff HEAD` `59a54267ce7c1d2a` avant et après (inchangé). Les recettes importent directement les modules T3 du worktree.
- Production Rust : empreinte `3943009ca82db59d...` au début. À 19:19:03, `crates/bridget-daemon/src/wrapper.rs` a été modifié par un autre propriétaire (digest `49f3a304d9504fe6`). L'empreinte devient `4fa56fcac7b37220...`. Le binaire r8 est immuable : ces 65 contrôles valent pour r8 et ne couvrent PAS ce nouveau `wrapper.rs`.

## Couches : réel ou simulé

| Couche | État |
|---|---|
| Daemon Bridget r8, clients `bridget mcp`, CLI `bridget lineage` | RÉEL |
| Hôte MCP T3 (HTTP Node, registre, rotation, révocation, `bridget_session` v1/v2) pour `interop149.ts` | RÉEL ; projection de conversation en mémoire et fait de permissions publiés par la recette (SIMULÉ) |
| Serveur T3 complet `bin.ts` (base privée, RPC WebSocket, auth scopée) pour `server149.ts` et `unified149.ts` | RÉEL |
| Credential MCP et fait de permissions dans `unified149.ts` | RÉEL : émis par la vraie session Codex de T3 ; le pair Codex est un faux pair capturé (SIMULÉ) |
| Wrapper du fil T3 côté daemon (trames `Register` et `T3ThreadBindingFact`) | SIMULÉ (trames réelles émises par la recette) |
| Enfants | Faux serveur app-server Codex fermé (`codex149.py`, puis `codex149b.py` en r3). Aucun modèle. |

Le chemin de Codex est un chemin absolu dans `settings.json` du serveur T3 de la recette, HOME privé sans authentification. Aucun vrai `codex` n'a été lancé (incident r2 évité).

## Ports, fixtures, processus

Ports privés distincts : 14796 (hôte MCP), 15756 (serveur T3) pour T036 ; 14797 à 14800, 15757, 15758, 15778 pour les autres recettes. Fixtures `umask 077`, dossiers 0700. Aucun jeton écrit dans un rapport. Arrêts : SIGTERM sur un PID dont le parent et la commande sont vérifiés, jamais `-9`, jamais `pkill`. Zéro processus résiduel à la fin de chaque recette.

## Reprise de G-P-07(b) : O1 (R8.4 PASS)

Le contrat clarifié `permissions.md` (G-P-07, lignes 441-462) est vérifié sur le binaire r8 par `recovery149.ts` R8.4 :

| Voie après retrait du fait + rotation | Rejeu `bridget_delegate` | `bridget_task_status` | `bridget_who` |
|---|---|---|---|
| Montage MCP privé de l'ancien credential | `t3_session_unavailable` | `t3_session_unavailable` | `t3_session_unavailable` |
| Credential neuf qui n'a JAMAIS eu de fait (v1) | `ok` (même tâche, aucun lancement) | `ok` | `ok` |
| Lecture humaine Lineage native (CLI, gardes 147) | sans objet | résultat intact | sans objet |

Aucun retour à v1 pour un credential dont le fait a existé. Côté T3, `bridget_session` rend le refus nommé `permission_attestation_unavailable` (S5.1). Aucune lecture humaine native ne dépend du credential T3.

## O5/G8 : suivi `journal --follow` de la CLI réelle (7/7 PASS)

Script : `.../native-network-recipes/follow149.ts`. Commande réelle : `bridget lineage inspect --json --t3-thread T --project-root R --action journal --task X --after-seq N --limit 100 --follow`. Environnement de la fixture seulement (`BRIDGET_HOME`, `BRIDGET_SOCKET`). Enfant vivant `STREAM_149:14:700 HB_149:25000`.

| Contrôle | Observé |
|---|---|
| F1 séquence | 16 événements, `seq` 1 à 16 strictement croissants, contigus, sans doublon ; `next_seq` ne régresse jamais ; aucune lacune ; 13 frontières `caught_up` ; 16 lignes arrivées sur 8,07 s (appends en DIRECT) |
| F2 égalité | les 16 `seq` suivis sont exactement ceux de la lecture ponctuelle |
| F3 reprise | `--after-seq 9` : 7 événements, `seq` 10 à 16, aucun doublon, aucun trou |
| F4 humains silencieux | le parent ne reçoit AUCUNE remise pendant les deux suivis (~20 s) ; tâche `working` ; 1 seul lancement |
| F5 fermeture | descripteurs du daemon 30 -> 40 pendant l'ouverture, retour à 30 ; threads 8 -> 12 -> 8 ; processus 2 -> 2 -> 2 (SIGTERM individuel des deux suivis) |
| F6/F7 fin de vie | après `cancel`, un suivi ouvert reçoit 16 événements contigus et se termine (exit 2, ligne `journal_unavailable`) ; en fin : 27 descripteurs, 7 threads, 0 processus |

Observation : à l'annulation de l'enfant, le suivi se termine par `{"status":"error","code":"journal_unavailable"}`. L'interface affiche alors « Bridget indisponible (journal_unavailable). Le contenu déjà lu est conservé. » alors que Bridget est disponible (vu dans la recette UI). Texte trompeur, pas de perte de données.

## Panne de T3 après admission (5 contrôles : 4 PASS RÉEL, 1 OBS)

Script : `.../native-network-recipes/outage149.ts`, vrai serveur T3 (`bin.ts`) avec espion du pair Codex de T3. Deux délégations admises (une qui attend, une lente), puis SIGTERM vérifié du serveur T3.

| Contrôle | Observé |
|---|---|
| T3out.1 | pendant la panne : lecture servie ; annulation native de la tâche en attente (`cancelled`, PID terminé) ; la tâche lente va à son terme, résultat remis UNE fois, lu par la CLI |
| T3out.2 zéro tour T3 | l'espion voit `turn/start` 1 -> 1, `thread/start` 1 -> 1, processus 2 -> 2 avant / pendant la panne |
| T3out.3 | montage MCP fermé : `t3_session_unavailable` pour status et nouvelle admission ; aucun lancement |
| T3out.4 | T3 revenu : sa lecture Lineage montre `cancelled` et `result_available` ; prompts Bridget inchangés |
| T3out.5 (OBS) | après le redémarrage de T3 : `turn/start` reste 1, un 3e processus de pair apparaît (reprise propre de T3, indépendante de Bridget) |

Résultat brut : `.../results-r3/outage149.json`.

## Limites et non prouvé

- Modèle réel (T037), voie standalone Claude/GLM et observer PTY (T038), `can_use_tool` (G-P-01), coque desktop : hors de cette ronde.
- Le nouveau `wrapper.rs` (19:19) : non couvert.
- Pas de launchd, pas de vrai Codex.
- Les résultats r2 sont archivés (`results-r2/`, index `INDEX.md`) pour que les références du rapport r2 restent lisibles.
