# Revue R3 — Changement post-R2 (marker JSON de mission natif)

**Verdict : APPROVE**

Le changement corrige le défaut R1 (1 « Started » mais 2 prompts). Il le corrige sans introduire de nouveau droit, sans fuite, et sans toucher aux garanties R2. Détail des vérifications, avec preuves.

## 1. Mécanisme relû et confirmé

Le daemon insère le marker seulement dans la branche `Ready` du spawn natif : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/session-148-identite-delegation/crates/bridget-daemon/src/daemon/native_delegation.rs:493-499`. Le marker contient deux UUID : `instance_id` et `mission_id`. Rien d'autre. C'est le seul enrichissement du fichier lié au bootstrap.

Le wrapper valide le marker à la naissance : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/session-148-identite-delegation/crates/bridget-daemon/src/wrapper.rs:411-419`. Il exige l'instance exacte et un UUID de mission valide. La structure est `deny_unknown_fields` (ligne 399). La mission arrive toujours du daemon : `message.id = task.mission` à `native_delegation.rs:805`, mission générée en UUID ligne 400.

## 2. Points vérifiés demandés

**Pas de nouveau droit par marker.** Le wrapper ne fabrique aucune mission. La carte se joint seulement à un message entrant dont l'id égale `mission_id` (`wrapper.rs:438`). La carte elle-même dit qu'elle ne crée ni ne relance rien (`wrapper.rs:391`). Aucune logique de permission ne lit ce marker. Test : `wrapper.rs:6243-6254` (instance étrangère, JSON vide, mission non-UUID rejetés).

**Pas de fuite credential ni d'historique parent.** La carte contient l'identité figée et des faits Git seulement : `wrapper.rs:329-392` (nom, type, protocole, digest, worktree, branche, tête, 40 fichiers max). Pas de conversation parent. Le fixture affirme l'absence d'environnement T3 (`tests/fixtures/native_delegation_148.py:13`). La recette réelle interdit les secrets dans le registre (`tests/native_delegation_real_glm.rs:310-317`).

**Idempotence et corrélation inchangées.** Les trois chemins de remise passent l'enveloppe d'origine au suivi ; seul le texte fournisseur est projeté (`wrapper.rs:4181`, `4215`, `4268`, commentaire lignes 430-432). Test d'égalité enveloppe : `wrapper.rs:6285-6289`. Rejeu ×10 : mêmes `task_id`, `child_agent_id`, `message_id` (`tests/native_delegation_e2e.rs:305-310`).

**Échec de remise ne perd pas le contexte.** Le contexte est consommé après succès seulement (`wrapper.rs:438-442`). Test échec-then-succès : `wrapper.rs:6279-6284`.

**Reconnect et restart.** Le contexte survit à la reconnexion socket (variable externe à la boucle, `wrapper.rs:4061`). À la relance fournisseur, le contexte est effacé puis la carte normale est réinjectée séparément (`wrapper.rs:4490-4510`) — pas de double jointure sur rejeu. Redémarrage daemon : résultat récupéré identique, annulé reste annulé, aucun start supplémentaire (`native_delegation_e2e.rs:419-435`).

**Injections profil indépendantes.** `apply_pending_profile_instructions` passe par `set_private_profile_instructions` (`wrapper.rs:3520`), un canal fournisseur distinct, pas un message. La carte séparée de repli ne s'applique qu'en l'absence de marker valide (`wrapper.rs:4079-4086`) — exclusion mutuelle.

**Contexte borné.** Carte ≤ 16 000 caractères avec mention de troncature (`wrapper.rs:295`, `303-314`), 40 entrées Git max (ligne 359), corps injecté refusé au-delà de 120 000 (lignes 300, `452-470`).

**Preuve E2E correcte.** Le fixture écrit lui-même ses preuves : exactement 1 « started », exactement 1 « prompt », et ce prompt unique contient carte ET mission (`native_delegation_e2e.rs:337-356`, `native_delegation_148.py:36`). Réponse corrélée unique (lignes 328-336). Annulation sans fuite (lignes 371-401).

**Docs.** `docs/delegation-native.md:47-51` décrit exactement le comportement : faits natifs joints au premier tour, ni conversation parent ni credentials T3. `specs/148-identite-delegation/contracts/delegation.md:38-41` (SendIdempotent, capture avant accusé) et lignes 15-20 (périmètres `same_project`/`root`) restent exacts. Aucun doc obsolète ne décrit l'ancien comportement à deux prompts.

## 3. Observations non bloquantes

1. **Hygiène (mineur)** — le processus fournisseur hérite de l'environnement du wrapper : `Command::new(...).envs(...)` sans `env_clear` (`crates/bridget-transport/src/acp.rs:325-331`). `BRIDGET_NATIVE_MISSION_BOOTSTRAP` est donc lisible dans l'environnement fournisseur. Contenu non sensible : deux UUID déjà connus du fournisseur (`BRIDGET_AGENT_INSTANCE_ID` est exposé volontairement à `wrapper.rs:4662-4665` ; l'id de mission arrive dans l'enveloppe). Pattern d'héritage préexistant. Durcissement possible : `std::env::remove_var` après lecture en `wrapper.rs:4076`.
2. **Couverture (mineur)** — le relais de la recette réelle n'archive que les noms de modèles (`tests/fixtures/native_delegation_real_glm_relay.py:24-43`). Il ne compte ni starts ni prompts. Si la recette réelle devient le portail final, étendre le relais à compter les trames utilisateur prouverait « 1 start 1 prompt » aussi sur fournisseur réel.

## 4. Limites de recette (pas des défauts)

- La recette réelle GLM est `#[ignore]`, opt-in explicite (`native_delegation_real_glm.rs:242-248`), et reste à exécuter. TMPDIR privé confirmé (lignes 319-325, 372).
- Le parent des deux recettes est un client synthétique enregistré comme Codex. Aucun tour de conversation Codex réel n'est exercé. Limite connue de R2, inchangée.
- La propriété « 1 start 1 prompt » est prouvée par le fixture fermé seulement. Le chemin de code wrapper est identique pour le fournisseur réel. Le risque résiduel est faible.

## 5. Clôtures R2

R2 avait approuvé avec D1–D4 fermés et une note INFO (libellé NACK générique). Le présent changement ne touche ni `capture_reply`, ni les chemins NACK, ni les injections profil. Je n'ai aucune preuve nouvelle contre ces clôtures. Je ne les rouvre pas.

**Décision finale : APPROVE.** Les deux observations sont facultatives. Elles ne bloquent ni la fusion ni la recette réelle.
