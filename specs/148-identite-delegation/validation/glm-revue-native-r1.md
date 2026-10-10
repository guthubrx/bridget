# Revue indépendante — Session 148, moteur natif de délégation

Périmètre lu : `specs/148-identite-delegation/` (spec + ADR), `delegation.rs`, `delegation_mcp.rs`, `daemon/native_delegation.rs`, diffs `protocol.rs`/`registry.rs`/`lifecycle.rs`/`daemon.rs`/`cli.rs`/`mcp.rs`/`mcp_identity.rs`, tests `native_delegation_e2e.rs` + fixture Python + tests unitaires embarqués. Lecture seule. Aucune modification, aucun Git, aucun test exécuté.

## Architecture — conforme

La saga est durable et autonome. Chaque exigence du mandat est vérifiée :

- **Sans T3** : aucun appel T3 dans le moteur (grep vide). Le recette e2e tourne daemon+superviseur+wrapper réels, fournisseur fermé. Les variables de session T3 ne passent jamais aux enfants (`lifecycle.rs:461-465`, testé).
- **Saga SQLite** : insertion idempotente `UNIQUE(owner_instance, request_id)`, rejeu par canon exact, états fermés. La reprise après redémarrage du daemon est testée e2e (tâche, IDs et résultat stables).
- **Sélection exacte et définition gelée** : `for_delegation` (`registry.rs:314-363`) refuse modèle/effort hors catalogue, strippe tout modèle/effort du profil, réinjecte le choix exact. `from_resolved` revérifie le digest avant tout re-spawn : le registre courant n'est jamais consulté. La recette prouve `glm5.3`/`high` réels au lancement.
- **Mission SendIdempotent, réponse avant ACK** : `capture_reply` persiste le résultat PUIS `handle_idempotent_send` finalise Accepted (`daemon.rs:8643-8648`). Le parent ne reçoit qu'une livraison, même après coupure entre remise et checkpoint (testé, double garde `replayed`).
- **Attente enfants, annulation, cleanup, reprise managed** : `waiting_for_children` + `descendants_busy` (test petit-fils actif) ; `cancel_tree` parcourt tâches et liens flotte, stoppe feuilles d'abord, SC006 vérifié e2e ; `recover_child` exige digest identique + lien flotte `delegation_id` + parent d'origine.
- **Refus sans mutation** : testé (store vide après refus modèle, effort, cwd, posture, identité). Le rejeu précède la révocation et le changement de registre (voulu, testé).
- **Grants humains** : CLI interactive + puits `Grant` exigeant rôle Client, acteur contrôle humain, `ControlStateV1`. Jamais un outil d'agent.
- **Limites** : 4096 tâches / 128 actives / 16 par instance, mission 64 Kio, résultat 256 Kio, profondeur 8, parcours borné 4096.

## Réponses aux remarques précédentes — corrigées

1. **Failed pending** : corrigé. `failed_for_owner` (`delegation.rs:158`) + bloc `failed` du tick (`native_delegation.rs:594-642`) : notification `failure_sent` durable, retentée tant que non livrée, puis cleanup. Si le parent est absent, le cleanup passe d'abord et la notification part à la reconnexion. Cohérent.
2. **Revocation fallback** : corrigé. Le tombstone `posture='revoked'` est consulté en tête de `root_permission` (`native_delegation.rs:35-37`). Il ferme l'héritage managed (testé ligne 1470-1478) et le fallback projet. Résidu D2 ci-dessous.
3. **R1 mcp-session-id** : correctif transport du principal (`combine_session_identity` refuse un conflit de session). Hors périmètre, aucune régression visible côté moteur natif.

## Défauts prouvés

### D1 — MAJEUR (bloquant) : transfert de tâche entre conversations homonymes vivantes

- **Lignes** : `delegation.rs:96` (`can_own_task` accepte `permission(instance).is_some()`), `native_delegation.rs:99-116` (`recover_owner` mute `owner_instance` sans vérifier que l'instance d'origine est partie), `delegation.rs:86` (`by_agent_request` clé owner + request_id, sans instance).
- **Scénario** : agent X, conversation A (instance IA vivante), conversation B (instance IB, preuve auxiliaire valide, grant humain à IB). B envoie `Delegate` avec le même `request_id` et le même canon qu'une tâche vivante de A. `by_agent_request` trouve la tâche de A. `recover_owner` transfère `owner_instance` vers IB. B lit le résultat et annule la mission de A. A perd l'accès si elle n'a ni grant ni lien managed.
- **Violation** : FR018, US5 (« il ne peut pas annuler les tâches d'une autre conversation »), SC005.
- **Hypothèse testable** : cloner `native148_reincarnated_parent_requires_proof_and_reuses_the_same_task` en gardant instance-1 présente et connectée pendant que instance-2 grantée rejoue le même canon. Le transfert réussit. Attendu : `task_unavailable` tant qu'IA vit.
- **Piste de correction** : n'autoriser le transfert par grant que si l'instance d'origine n'a plus de présence ni de connexion vivante. Le test actuel supprime la présence d'instance-1 avant la reprise — la garde manquante est exactement ce cas.

### D2 — MOYEN : tombstone de révocation contournable par réincarnation

- **Lignes** : `delegation.rs:194` (le revoke tombe sur l'instance résolue à l'instant), `delegation.rs:209-211`, `native_delegation.rs:35-37`.
- **Scénario** : l'humain exécute `delegate-grant X --revoke`. Le tombstone s'applique à l'instance courante de X. X redémarre (nouvelle instance, non tombstonée). `root_permission` repart des sources générales (héritage managed ou attestation projet). Les nouvelles délégations sont acceptées alors que l'humain a révoqué « l'agent ».
- **Hypothèse testable** : révoquer l'instance courante, simuler une nouvelle instance du même nom avec lien managed ou attestation projet, appeler `Catalogue`/`Delegate`. Attendu : refus.
- **Piste** : étendre le tombstone à l'agent, ou couper l'héritage pour un agent révoqué jusqu'à un nouveau grant explicite.

### D3 — MINEUR : lecture `Status` inter-conversations

Même racine que D1 (`can_own_task` ligne 96) : une instance homonyme grantée qui connaît le `task_id` lit le résultat d'une autre conversation. La garde de D1 corrige aussi ce point.

### D4 — MINEUR (robustesse) : zombie si la persistance de capture échoue

`native_delegation.rs:531` : si `save` échoue, `capture_reply` retourne false. La réponse part en livraison directe au parent. La tâche reste `working` à vie et compte dans la limite des 128 actives. Probabilité faible (erreur disque).

## Observations (non bloquantes, non prouvées sans exécution)

- Un tour enfant terminé sans réponse corrélée laisse la tâche `working` sans échéance : le `DeliveryRejected` de reply-timeout part vers la connexion synthétique fermée, donc `rejected()` ne se déclenche pas. L'annulation manuelle reste disponible. À documenter ou couvrir par un délai de mission.

## Verdict : REQUEST_CHANGES

Le moteur est sain dans l'ensemble : saga durable, sélection exacte gelée, remises idempotentes, annulation et reprise solides, réponses aux remarques précédentes effectives. Mais D1 viole une exigence P1 explicite (FR018/US5/SC005) par un chemin réaliste — le pattern « request_id stable » est justement celui recommandé par la description de l'outil. D1 bloque. D2 doit être corrigé ou assumé par écrit. D3 suit D1. D4 et les observations peuvent attendre.
