# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/interop149.md](../../../../../../../specs/149-sous-agents-lineage/validation/interop149.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Recette | Contrôles | Résultat brut |
|---|---|---|
| `interop149.ts` : hôte MCP de T3 réel + daemon natif | 43/43 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r3/interop149.json` |
| `server149.ts` : serveur T3 complet (`bin.ts`) + daemon natif | 15/15 PASS | `.../results-r3/server149.json` |
| `unified149.ts` : UNE instance T3 qui émet le credential ET sert Lineage | 7/7 PASS | `.../results-r3/unified149.json` |

<a id="extrait-2"></a>

## Extrait 2

Première exécution : 19:00 à 19:02. Rejeu final avec les outils définitifs (après ajout du second modèle et de `execs()` dans `fx.mjs`) : 19:46 à 19:48. Les deux donnent 65/65. La première est archivée dans `.../results-r3/t036-premiere-passe/`. Les scopes T3 sont inchangés ; je n'ai pas relancé de suite globale.

<a id="extrait-3"></a>

## Extrait 3

- Binaire release r8 : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb`, SHA-256 `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6` recalculé avant et après chaque recette (journal `.../results-r3/shacheck.log`, toujours égal au reçu `native149-release-receipt-r8.json`). Debug r8 `833a030545c3` non utilisé.
- Sources T3 : `git diff HEAD` `59a54267ce7c1d2a` avant et après (inchangé). Les recettes importent directement les modules T3 du worktree.
- Production Rust : empreinte `3943009ca82db59d...` au début. À 19:19:03, `crates/bridget-daemon/src/wrapper.rs` a été modifié par un autre propriétaire (digest `49f3a304d9504fe6`). L'empreinte devient `4fa56fcac7b37220...`. Le binaire r8 est immuable : ces 65 contrôles valent pour r8 et ne couvrent PAS ce nouveau `wrapper.rs`.

<a id="extrait-4"></a>

## Extrait 4

| Couche | État |
|---|---|
| Daemon Bridget r8, clients `bridget mcp`, CLI `bridget lineage` | RÉEL |
| Hôte MCP T3 (HTTP Node, registre, rotation, révocation, `bridget_session` v1/v2) pour `interop149.ts` | RÉEL ; projection de conversation en mémoire et fait de permissions publiés par la recette (SIMULÉ) |
| Serveur T3 complet `bin.ts` (base privée, RPC WebSocket, auth scopée) pour `server149.ts` et `unified149.ts` | RÉEL |
| Credential MCP et fait de permissions dans `unified149.ts` | RÉEL : émis par la vraie session Codex de T3 ; le pair Codex est un faux pair capturé (SIMULÉ) |
| Wrapper du fil T3 côté daemon (trames `Register` et `T3ThreadBindingFact`) | SIMULÉ (trames réelles émises par la recette) |
| Enfants | Faux serveur app-server Codex fermé (`codex149.py`, puis `codex149b.py` en r3). Aucun modèle. |

<a id="extrait-5"></a>

## Extrait 5

Le chemin de Codex est un chemin absolu dans `settings.json` du serveur T3 de la recette, HOME privé sans authentification. Aucun vrai `codex` n'a été lancé (incident r2 évité).

<a id="extrait-6"></a>

## Extrait 6

Ports privés distincts : 14796 (hôte MCP), 15756 (serveur T3) pour T036 ; 14797 à 14800, 15757, 15758, 15778 pour les autres recettes. Fixtures `umask 077`, dossiers 0700. Aucun jeton écrit dans un rapport. Arrêts : SIGTERM sur un PID dont le parent et la commande sont vérifiés, jamais `-9`, jamais `pkill`. Zéro processus résiduel à la fin de chaque recette.

<a id="extrait-7"></a>

## Extrait 7

Aucun retour à v1 pour un credential dont le fait a existé. Côté T3, `bridget_session` rend le refus nommé `permission_attestation_unavailable` (S5.1). Aucune lecture humaine native ne dépend du credential T3.

<a id="extrait-8"></a>

## Extrait 8

Script : `.../native-network-recipes/follow149.ts`. Commande réelle : `bridget lineage inspect --json --t3-thread T --project-root R --action journal --task X --after-seq N --limit 100 --follow`. Environnement de la fixture seulement (`BRIDGET_HOME`, `BRIDGET_SOCKET`). Enfant vivant `STREAM_149:14:700 HB_149:25000`.

<a id="extrait-9"></a>

## Extrait 9

Script : `.../native-network-recipes/outage149.ts`, vrai serveur T3 (`bin.ts`) avec espion du pair Codex de T3. Deux délégations admises (une qui attend, une lente), puis SIGTERM vérifié du serveur T3.

<a id="extrait-10"></a>

## Extrait 10

| Contrôle | Observé |
|---|---|
| T3out.1 | pendant la panne : lecture servie ; annulation native de la tâche en attente (`cancelled`, PID terminé) ; la tâche lente va à son terme, résultat remis UNE fois, lu par la CLI |
| T3out.2 zéro tour T3 | l'espion voit `turn/start` 1 -> 1, `thread/start` 1 -> 1, processus 2 -> 2 avant / pendant la panne |
| T3out.3 | montage MCP fermé : `t3_session_unavailable` pour status et nouvelle admission ; aucun lancement |
| T3out.4 | T3 revenu : sa lecture Lineage montre `cancelled` et `result_available` ; prompts Bridget inchangés |
| T3out.5 (OBS) | après le redémarrage de T3 : `turn/start` reste 1, un 3e processus de pair apparaît (reprise propre de T3, indépendante de Bridget) |
