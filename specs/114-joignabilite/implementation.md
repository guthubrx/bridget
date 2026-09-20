# Journal 114 — Trois causes résiduelles de non-joignabilité

- **Base** : main `9661c127` — **Date** : 2026-09-20 — **Statut** : In Progress

## A. Une seconde application T3 ne révoque plus les identités

### Diagnostic
L'application officielle t3code 0.0.42, réinstallée le 2026-09-20 à 05:45 et lancée à 06:03, a
tourné à côté de l'application locale 0.0.40 qui héberge les agents. Les deux partagent
`~/.t3/userdata/` : la seconde s'est déclarée dans `server-runtime.json` avec son port 3774.
`refresh_inner` comparait ce fichier à son propre serveur et, en cas d'écart, révoquait tous les
marqueurs. Résultat : `identity_not_found` sur les appels MCP de tous les fils, pas seulement
celui qui s'en plaignait, et 456 avertissements identiques dans la journée.

### Correction (`t3code_identity.rs`)
Le contrôle visait une course : le serveur ne doit pas changer pendant la collecte. C'est la
**naissance** du processus qui le prouve, relevée avant et après `collect_processes`. Elle est
strictement plus forte que la comparaison de fichier — elle voit le recyclage de PID — et immune à
un tiers qui réécrit un fichier partagé. Les attestations ne dépendent que de notre serveur : les
descendants collectés sont les siens, les liens vivants viennent de notre propre snapshot.

### Signalement (`t3code.rs`)
- `report_foreign_runtime` : un avertissement par changement, nommant le PID, le port de l'autre
  application et le fichier partagé ; un retour à la normale est dit une fois.
- Les échecs d'attestation ne sont journalisés qu'au changement de motif ; le rétablissement l'est.
- `bridget t3 status` nomme les deux cas : le fichier désigne une autre application, ou il a été
  effacé alors que le pont tourne (le prochain démarrage ne trouvera rien).
- Le pont n'écrit pas dans l'état de t3code : cet invariant est conservé.

## B. Une rafale de messages n'ouvre plus un tour par message

### Diagnostic
Le 2026-09-19 à 19:16, `horizon-3D` a reçu 26 messages en 2 min 30 de la part de `opus-horizon`,
`claude-horizon` et `horizon-cursor`. Vingt-trois étaient des comptes rendus sans réponse attendue ;
chacun a ouvert son tour. `correlate` exige autant de tours assistants que de messages utilisateur
dans la fenêtre lue : un seul tour sans texte casse l'égalité pour toute la fenêtre. Deux demandes
suivies ont donc fini « au repos sans appariement certain », sans réponse pour leur expéditeur.

### Correction (`t3code.rs`, `wrapper.rs`)
- `batchable_prefix` : longueur du lot en tête de file, bornée par `BATCH_BOUND` (8) et
  `BATCH_BODY_BOUND` (32 Kio), arrêtée au premier message non groupable ou déjà écarté.
- `groupable` : ni demande suivie, ni sollicitation de fil, ni notification — chacune porte une
  consigne propre, et la réponse d'un tour ne se répartit pas entre deux demandes.
- `batch_envelope` : à un seul message, rend exactement l'enveloppe historique ; au-delà, un
  en-tête de lot et un séparateur nommant l'expéditeur et l'identifiant de chaque message.
- `dispatch_batch` remplace `dispatch_with_id` en production ; `commandId` reste l'identifiant du
  premier message, donc le rejeu après panne reconstruit la même commande.
- `deliver_batch_to_interactive` (`wrapper.rs`) : le tracker juge chaque remise séparément, seules
  les remises retenues entrent dans l'injection unique, et chacune reçoit ensuite son accusé.
  `deliver_idempotent_to_interactive` devient un appel à un lot d'un élément : aucune duplication.
- Le paramètre `command_id` de `dispatch_with_id` n'avait qu'une valeur effective ; il est retiré.

## C. Les fils Cursor sont attestés

### Diagnostic
`collect_processes` ne reconnaissait que `codex app-server` et `claude`. `cursor-agent` ne porte
pas son identifiant de session en ligne de commande — il l'a négocié par le protocole ACP — mais il
tient ouvert `~/.cursor/acp-sessions/<sessionId>/store.db`, et T3 enregistre ce même `sessionId`
dans `provider_session_runtime`. La preuve est donc de même nature que le rollout Codex.

### Correction (`t3code_identity.rs`, `runtime.rs`)
- `cursor_session` extrait l'identifiant du composant qui suit `acp-sessions`, validé.
- `collect_processes` accepte `cursor-agent` lancé en mode `acp` ; `read_sessions` lit `sessionId`.
- `open_session_files` reçoit désormais le prédicat d'acceptation de l'appelant : élargir le filtre
  pour tout le monde aurait fait classer à `open_session_file` un fichier qui n'est pas un rollout.
- `complete_session_inventory` trie les fichiers ouverts par fournisseur avant de les lire : un
  fichier étranger n'invalide plus le cycle, et la complétude par fournisseur est préservée.

## Vérifications
- Tests `spec114_*` : 8 (identité 3, pont 4, lot idempotent 1), tous verts.
- `t3code` et `t3code_identity` : 77/77. fmt OK ; clippy `-D warnings` OK.
- Recette complète (`cargo test --workspace --no-fail-fast`, umask 077, `BRIDGET_HOME` privé) :
  **1532 réussis, 0 échec, 52 ignorés** sur 80 binaires — les huit tests ajoutés, aucune régression.
