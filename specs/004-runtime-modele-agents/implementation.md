# Journal d'Implémentation — Visibilité du modèle et du niveau d'effort

## Métadonnées

- **Spec** : 004-runtime-modele-agents
- **Branche** : `session-04-runtime-modele-agents`
- **Démarré** : 2026-08-17
- **Terminé** : en cours

## Écart de découpage assumé

T001, T002 et T003 forment un seul incrément livrable. Le typage Rust les rend
indissociables : ajouter un variant à `WrapperToDaemon` rend le `match` du
daemon non exhaustif, et ajouter des champs à `AgentInfo` casse ses deux
initialisations. Les livrer séparément aurait exigé un stub jetable, c'est-à-dire
du code à comportement constant interdit par l'Article XIX. Un commit unique
couvre donc les trois.

## Progression

### T001 + T002 + T003 — Socle du runtime

- **Statut** : ✅ Complété
- **Fichiers** :
  - `crates/bridget-transport/src/protocol.rs` (modifié) — variant `Runtime`,
    énumération `RuntimeSource`, champs `model`/`effort` sur `AgentInfo`
  - `crates/bridget-daemon/src/daemon.rs` (modifié) — champs sur `Presence`,
    `handle_runtime`, `validate_runtime_value`, propagation dans `agent_infos()`
  - `crates/bridget-daemon/tests/integration_test.rs` (modifié) — correction
    d'un warning préexistant `unused variable`
- **Tests** : `cargo test` vert. Nouveaux : aller-retour `Runtime`, effort
  absent décodable, source inconnue refusée, `AgentInfo` sans runtime décodable,
  remplacement atomique du couple, refus hors agent et valeurs invalides,
  survie à une reconnexion, conservation en `unreachable`.
- **Notes** : `handle_runtime` résout l'agent **par son nom** et non par la
  connexion émettrice. Défaut détecté à l'implémentation : le hook et
  `bridget runtime` passent par le client CLI, dont la connexion est éphémère et
  distincte de celle de l'agent observé ; rattacher l'observation à la connexion
  aurait mis à jour une présence inexistante. Le contrat a été corrigé en
  conséquence (`contracts/protocol.md`, champ `agent`).

### T004 — Colonnes MODÈLE et EFFORT

- **Statut** : ✅ Complété
- **Fichiers** : `crates/bridget-daemon/src/cli.rs` — `cmd_who`, `cmd_agents`,
  helper `runtime_cell`
- **Tests** : test unitaire de `runtime_cell` ; vérification réelle ci-dessous.

### T005 — `bridget runtime`

- **Statut** : ✅ Complété
- **Fichiers** : `crates/bridget-daemon/src/cli.rs` — `cmd_runtime`,
  `send_runtime_to_daemon`, dispatch et `print_usage`
- **Notes** : réutilise `current_agent_name()` de la spec `001-renommer-agent`,
  aucun nouveau mécanisme d'identification.

### T006 — Module `runtime.rs`

- **Statut** : ✅ Complété
- **Fichiers** : `crates/bridget-daemon/src/runtime.rs` (créé),
  `crates/bridget-daemon/src/lib.rs` (modifié)
- **Tests** : 10 tests. Transcript nominal ; sous-agent et ligne assistant sans
  modèle ignorés ; effort absent conservé absent ; rollout nominal ;
  `turn_context` sans modèle ignoré ; fenêtre tronquée aux deux bornes ;
  agrandissement de fenêtre ; fichier vide ou absent ; JSON invalide ;
  sélection du rollout le plus récemment écrit.
- **Notes** : les fixtures sont générées dans le test plutôt que stockées comme
  fichiers — elles restent lisibles à côté de l'assertion qu'elles servent, et
  aucune ne dépasse quelques lignes hors remplissage volumétrique.

### T007 — `bridget hook claude-runtime`

- **Statut** : ✅ Complété
- **Fichiers** : `crates/bridget-daemon/src/cli.rs`
- **Notes** : **défaut trouvé en exécution réelle** — le hook s'est bloqué
  2 minutes face à un daemon d'une version antérieure qui ignore le message et
  ne répond rien. Un hook bloquant gèle la fin de tour de l'agent, ce qui viole
  FR-013. Correction : délai de lecture de 2 s
  (`RUNTIME_REPLY_TIMEOUT_SECS`). Vérifié : sortie en 2,02 s, code 0, sortie
  standard vide.

### T008 — `bridget install-hooks`

- **Statut** : ✅ Complété
- **Fichiers** : `crates/bridget-daemon/src/cli.rs`
- **Tests** : 4 tests sur une configuration réaliste à quatre hooks utilisateur —
  insertion additive préservant l'existant, retrait restaurant l'original octet
  pour octet, idempotence, création de la section absente.

### T009 — Sonde Codex

- **Statut** : ✅ Complété (vérification runtime en attente, voir T011)
- **Fichiers** : `crates/bridget-daemon/src/wrapper.rs` — `RuntimeProbe`,
  branchement sur la boucle d'écoute ; `runtime.rs` — `open_session_file`,
  `most_recently_written`
- **Notes** : la résolution `lsof` est amortie (une fois, puis toutes les 5 min
  ou si le fichier disparaît) ; seul un `stat` tourne toutes les 20 s. Vérifié
  à la main sur le PID Codex 2359 : le rollout ouvert est bien résolu, sur un
  volume externe que le chemin conventionnel n'aurait pas trouvé.

### T010 — Documentation

- **Statut** : ✅ Complété
- **Fichiers** : `README.md` — commandes, section « Modèle et niveau d'effort
  des agents », avertissement explicite sur la modification de
  `~/.claude/settings.json` et procédure de retour arrière.

### Corrections issues de la contre-revue d'implémentation

Trois défauts réels signalés par `agent-1` sur le code produit, tous corrigés :

1. **`runtime.rs`** — le rejet inconditionnel du dernier fragment perdait la
   dernière ligne d'un fichier JSONL complet sans saut de ligne final. Le
   fragment n'est désormais écarté que s'il suit un saut de ligne ; le parseur
   JSON strict rejette de lui-même ce qui est tronqué. Deux tests ajoutés.
2. **`wrapper.rs`** — `last_mtime` n'était pas réinitialisé lors d'un changement
   de fichier de session : un nouveau rollout dont la date coïncidait avec
   l'ancienne n'aurait jamais été lu. Corrigé, et la re-résolution du chemin
   passe de 300 s à 60 s pour suivre un `codex resume` dans le délai de FR-006.
3. **`cli.rs`** — `settings.json` était écrit par `fs::write`, donc exposé à une
   troncature en cas d'interruption. Remplacé par écriture dans un fichier
   temporaire du même répertoire puis `rename` atomique, avec conservation des
   permissions. Les `expect` sont devenus des refus explicites. Deux tests.

Une objection a été rejetée avec raison écrite (usurpation d'identité) : voir
`adversarial-review-agent-1.md`.

### T011 — Vérification bout en bout

- **Statut** : ✅ Complété pour tout ce qui est vérifiable sans relancer un agent
- **Environnement** : daemon redémarré sur la version compilée (accord
  utilisateur du 2026-08-17), six agents reconnectés automatiquement.

## Vérifications réelles effectuées

| Critère | Résultat | Preuve |
|---|---|---|
| SC-006 — aucune régression | ✅ | nouveau client contre daemon **non redémarré** : `bridget who` affiche les colonnes, valeurs `—`, aucune erreur |
| FR-003 — sortie machine | ✅ | `bridget agents --json` expose `"model": null, "effort": null` |
| FR-004 — inconnu explicite | ✅ | tirets cadratins alignés sur cinq agents réels |
| Compatibilité ascendante | ✅ | `#[serde(default)]` validé contre un daemon d'une version antérieure, en conditions réelles |
| Parsing transcript réel | ✅ | hook exécuté sur un transcript de 2 Mo : parsing réussi, passage à l'envoi |
| Résolution rollout Codex | ✅ | `lsof` sur le PID 2359 → rollout sur `/Volumes/DATA/…` |
| FR-013 — hook non bloquant | ✅ | 2,02 s dans le pire cas, code 0, sortie vide |
| Suite de tests | ✅ | 79 tests verts (28 core + 26 daemon + 6 intégration + 19 transport) |
| Warnings | ✅ | `cargo build` : 0. `cargo clippy` : 32, **tous préexistants**, 0 sur le code neuf |

### Mesures sur le daemon en version nouvelle

| Critère | Résultat | Mesure |
|---|---|---|
| US3 — déclaration explicite | ✅ | `bridget runtime --model claude-opus-5 --effort high` → visible dans `who` |
| FR-009 — refus hors agent | ✅ | code 1 et message explicite hors contexte Bridget |
| SC-002 — délai de mise à jour | ✅ | **0,05 s** entre l'appel du hook et la valeur dans l'annuaire, contre 60 s exigées |
| Invariant atomique | ✅ | valeur bidon `faux-modele/minimal` écrasée par `claude-opus-5/high` lu dans le transcript réel |
| Effacement de l'effort | ✅ | `runtime --model modele-sans-effort` sans `--effort` → colonne EFFORT repassée à `—` |
| Parsing Haiku réel | ✅ | transcript d'une session `--model haiku` → `claude-haiku-4-5-20251001`, effort `—` |
| SC-004 — lisibilité | ✅ | six agents affichés, valeurs présentes et absentes mêlées, colonnes alignées |
| FR-012 — réversibilité | ✅ | sur le vrai `~/.claude/settings.json` à quatre hooks : installation additive, `--remove`, puis comparaison JSON → **restauration exacte** |
| Idempotence | ✅ | seconde installation : « Aucun changement » |
| FR-013 — hook non bloquant | ✅ | 0,05 s en fonctionnement normal, 2,02 s dans le pire cas (daemon muet) |

## Limite constatée et non résolue

Le hook inscrit dans `~/.claude/settings.json` **ne se déclenche pas en mode
`claude -p`** (headless). Reproduit deux fois. Le même hook passé par
`--settings` en ligne de commande se déclenche normalement, avec un payload
correct et un environnement Bridget complet — vérifié par capture. Il s'agit
donc d'un comportement du mode headless de Claude Code, et non d'un défaut de
la feature : la commande `bridget hook claude-runtime` exécutée avec le payload
réel met bien l'annuaire à jour.

Le cas d'usage visé est la session interactive lancée par `bridget claude`,
mode dans lequel les hooks globaux fonctionnent — ceux de l'utilisateur
(`agent-attention.sh` sur `Stop` et `UserPromptSubmit`) en dépendent déjà au
quotidien. **À confirmer néanmoins au prochain lancement d'un agent Claude
interactif** : `bridget who` doit afficher son modèle après son premier tour.

De même, la sonde Codex ne peut être observée que dans un wrapper lancé avec le
nouveau binaire ; les agents Codex actuellement connectés tournent sur l'image
précédente. Ses briques sont vérifiées séparément : résolution `lsof` sur le PID
Codex réel 2359 (rollout trouvé sur un volume externe), parseur couvert par huit
tests dont les cas de troncature.

## Findings hors périmètre, non traités

1. `crates/bridget-daemon/src/managers.rs` — 197 lignes de code mort.
2. `Store::purge_if_too_large` (`store.rs:255`) — corps vide ignorant son paramètre.
3. 32 avertissements `clippy` préexistants sur le workspace.
4. `send_rename_to_daemon` (`cli.rs`) souffre du même défaut d'absence de délai
   de lecture que celui corrigé dans `send_runtime_to_daemon` — même symptôme
   possible sur `bridget rename` face à un daemon muet. Hors périmètre de cette
   session, mais désormais identifié et reproductible.
