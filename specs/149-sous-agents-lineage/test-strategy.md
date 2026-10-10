# Stratégie de tests — session 149 « sous-agents lineage »

Date : 2026-10-10. Statut : design seul, amendé des revues G-L r1, G-P r1 et G-P r3. Aucun test écrit. Aucun code produit.
Aucun test ni build exécuté : la version du worktree est encore inchangée (base 148 `6807c22b`).
Owner : GLM 5.3 Flash (identifiant exact `glm-5.3-flash`). Toute revue et tout test délégués restent GLM, pas Sol.

Ronde r2. Elle applique les 9 corrections du principal sur la ronde r1 (`session149-glm-flash-test-design-r1`, 21 scénarios). Changements : oracle Lineage corrigé (l'enfant virtuel persisté est attendu), portée `cwd` traitée comme admission logique et non comme sandbox OS, attestation réelle exigée pour tout parent hors 149, grant pty retiré du chemin positif 149, 6 scénarios ajoutés (S149-22 à S149-27), modèle réel fixé à `glm-5.3-flash`, mapping complet vers FR/SC, résumé des limites. Tous les chemins livrés ici sont absolus.

Amendement G-L r1 (revue `validation/plan-lineage-r1.md`, F1–F7) : S149-28 (recette UI web, §4.8) s'ajoute et remplace la recette desktop prévue, qu'une seconde instance T3 rendait impossible. Total : 28 scénarios, aucun exécuté.

Amendement G-P r1 (revue `validation/plan-permissions-r1.md`, G-P-01–G-P-06) : S149-29 à S149-31 ajoutés (§4.9), CLI résolu gelé à côté du launcher, registre de recette consigné (§7), `permission_source_unavailable` seul nom du cas source opaque (§9.5). Total : 31 scénarios, aucun exécuté.

Amendement G-P r3 (revue `validation/permissions-contract-deltas-r3.md`, G-P-07–G-P-08) : S149-32 et S149-33 ajoutés (§4.10). Le contrat reçoit la phrase corrigée « tant qu'aucun fait de permissions n'a jamais existé pour ce credential » et les oracles G-P-07 (union v1/v2) et G-P-08 (observer PTY). G-P-01–G-P-06 restent fermés ; la re-revue G-P est en attente. Total : 33 scénarios, aucun exécuté.

Clarification 2026-10-10 (revue source Sonnet r1, section 5) : S149-32(b) reformulé selon la section 5 de `validation/native-f2-o4-source-review-sonnet-r1.md`. Aucun scénario ajouté, aucun exécuté. Total : 33 scénarios.

Base de preuve 148 (historique, référence seule — correction 7, ce ne sont pas des tests 149 déjà verts) : 1496 PASS / 15 ignorés (workspace Rust), 45 ciblés, 2 E2E Rust, 389 PASS / 1 SKIP de régression identité T3, 6 PASS d'interop HTTP réelle R4, recette réelle GLM en 55 s. Reçu : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/148-identite-delegation/validation/final.md`.

---

## 1. Sources lues

| Source | Contenu retenu |
|---|---|
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/AGENTS.md` | Checkpoint 148. Identité par session T3. Délégation en un appel. Aucun redémarrage. |
| `/Users/moi/.speckit/constitution.md` | Articles XV (tests avant commit), XVIII (complexité), XIX (minimalisme), XXI (anti-boucle : observer avant de retenter). |
| `/Users/moi/.speckit/ref/standards-tests.md` | Nommage avec ID de spec. Tester le comportement. Un test = une assertion principale. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/spec.md` | FR001–FR019, SC001–SC007, US1–US7. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/lineage.md` | Contrat descendance native et Lineage : fils virtuels déterministes, marqueur `bridgetTaskRef`, snapshot paginé, journal, watch, cancel, refus fermés. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/permissions.md` | Contrat provenance et héritage : posture inherit, attestation v2, politiques Codex et Claude exactes, mappage inter-fournisseurs, refus nommés sans fallback. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/data-model.md` | Champs de tâche 149, séquence de projection, snapshot de permissions, journal, transitions. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/plan.md` | Phases, gates, invariants, répartition exclusive. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/checklists/requirements.md` | Checklist de validation utilisateur. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/148-identite-delegation/contracts/delegation.md` et `session.md` | Contrats 148 des 4 outils, bornes, échéance de mission, révocation. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/148-identite-delegation/validation/final.md` | Recettes réelles 148, limites admises, environnement de référence. |
| Code (worktree 149, inchangé) | `native_delegation.rs`, `delegation.rs`, `registry.rs`, `native_delegation_e2e.rs`, `native_delegation_real_glm.rs`, `support/idempotent.rs`. |
| Code T3 (worktree 149) | `BridgetReader.ts`, tests MCP `Bridget*.test.ts`, toolkit `orchestrator`. |

Le contrat `permissions.md` reste en cours de figure et non figé. La stratégie ne suppose donc pas un confinement GLM qui ne serait pas attesté.

---

## 2. Objectifs 149 traduits en exigences testables

| Objectif utilisateur | Exigence de test | Oracle principal |
|---|---|---|
| Véritables enfants Bridget en un appel | Un `bridget_delegate` crée tâche + enfant + mission durables, une seule fois. | 1 ligne `native_delegations`, 1 événement `started`, 1 événement `prompt`. |
| Lineage T3 : projection parent/nested/status/result/journal | La projection est une lecture pure des données Bridget. Elle crée exactement un fil virtuel persisté par tâche. (Correction 1 — l'oracle r1 « aucune ligne thread T3 » était faux.) | Un fil virtuel par tâche : identifiant déterministe `thread:bridget-task:<task_id>`, nœud `node:bridget-task:<task_id>`, relation `subagent`, origine `bridget_native`, marqueur `bridgetTaskRef` complet. Négatifs : zéro conversation T3 de premier niveau ; zéro provider session, turn, outbox fournisseur ou effet de lancement ; `activeTurnId` null ; le résultat n'est jamais remis au parent par T3 (remise native 148 unique). |
| Droits hérités du parent attesté, parent Codex ET GLM, sans grant humain supplémentaire | Un parent qui a lui-même le droit d'écrire confère ce droit à son enfant. Aucun grant humain dans le chemin positif 149. (Correction 4.) | Enfant écrit le fichier nonce dans `cwd`. Un parent full-access confère une mission GLM locale en bypass : l'enfant peut écrire hors `cwd` comme son parent. La portée `cwd` est une admission logique de projet, pas un sandbox OS ; le catalogue n'affirme aucun confinement que le fournisseur n'applique pas. |
| Identité et confinement selon le parent attesté, jamais le prompt | L'identité vient de la preuve native. Le texte de mission ne change ni `from`/`to` ni `cwd` ni posture. Tout parent hors 149 est attesté par une session réelle v2 ou par sa source propriétaire standalone — jamais par la signature parent-managée 148 opposée comme preuve externe. (Correction 3.) | Réponse corrélée `from=child_agent_id`, `to=parent`. Tentatives d'usurpation par prompt ou environnement refusées avant spawn. Fixtures et preuves réelles distinguées dans la section 5. |
| Hors T3 idem | Le même chemin natif réussit sans T3 et avec un endpoint T3 mort. | `t3_present=false` ; mission terminée ; refus fermé des appels MCP avec preuve invalide. |
| Pas de restart ni de config de production | Tous les tests isolent daemon, socket, DB, TMPDIR dans un dossier privé de `/tmp`. | Aucun PID production touché. Hash des settings/profil/commande source inchangés. |

Scénarios nommés par l'utilisateur, couverts en section 4 :

1. Recette réelle : GLM Flash écrit un fichier privé avec nonce, résultat corrélé unique.
2. Parent readonly : refuse avant spawn.
3. Parent development externe : pas de grant.
4. Revoke et opt-out persistants.
5. Nested, retry, restart, cancel.
6. Panne T3 et invisibilité des autres parents.

---

## 3. Tests réutilisables — inventaire

### 3.1 Rust Bridget (worktree `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`, branche `session-149-sous-agents-lineage`)

| Asset | Chemin | Ce qu'il prouve déjà | Réusage 149 |
|---|---|---|---|
| Harnais partagé | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/support/idempotent.rs` | `private_dir`, `private_write` (0o700/0o600), `isolated_command` (env fermé), `Client` de transport, nettoyage d'enfants. | Base de toute nouvelle fixture. |
| E2E natif | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/native_delegation_e2e.rs` | Refus sans grant ; grant humain par pty ; retry ×10 = 1 tâche ; réponse corrélée unique ; annulation ; reprise après redémarrage du daemon ; `t3_present=false`. | Régression 148 directe. Ses helpers (`Fixture`, `OwnedChild` avec vérification PID/exécutable et refus Firefox) sont le moule des tests 149. Le grant pty qu'il utilise reste hors du chemin positif 149. |
| Recette réelle opt-in | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/native_delegation_real_glm.rs` | Parent Codex fixture + GLM réel, modèle exact exigé, un seul enfant et une seule réponse corrélée, nettoyage MCP attesté, hash des settings/commandes inchangés. | Moule de la recette 149. Ses préfixes d'environnement sont `BRIDGET_148_*` et son exemple de modèle est `glm-5.3` (ligne 13) : la mise à jour vers `BRIDGET_149_*` et `glm-5.3-flash` appartient à l'owner natif, l'oracle 149 exige Flash exact. Ses 4 tests sec (`auth`, cleanup EOF, cleanup unwind) restent la régression. |
| Tests unitaires inline | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs` (20 tests `native148_*`) | Portée owner de status/cancel ; révocation d'agent qui survit à une nouvelle instance ; refus cwd et développement ; seul l'humain grant ; rejeu avant révocation ; découverts descendants avant parent ; échéance figée ; catalogue GLM ; fin de tour ≠ résultat. | Régression directe + modèle de nommage `native149_*`. |
| Identité T3 | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/t3_session_identity_env.rs` | Preuve partielle ou invalide : jamais de repli PID. | Régression + extension endpoint mort. |
| Fournisseur factice | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/fixtures/native_delegation_148.py` | Écrit `evidence.jsonl` (`started`, `prompt`, `blocked`), bloque sur `WAIT_CANCEL_148`. | Réutilisé tel quel pour les tests lineage fixture. |
| Relay réel | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/fixtures/native_delegation_real_glm_relay.py` | Journalise le modèle effectif annoncé par le vrai CLI. | Réutilisé ; étendu au journal des écritures effectuées et à l'attestation du modèle `glm-5.3-flash` effectif. Mise à jour de fixture locale, sans config de production. |

### 3.2 T3 (worktree `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`, même branche)

| Asset | Chemin | Ce qu'il prouve déjà |
|---|---|---|
| Attestation de session | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/BridgetSession.test.ts` | Réponse `bridget_session` version 1, sans secret. | 
| Observateur headers | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/BridgetRustInteropObserver.test.ts` | Conservation de `Mcp-Session-Id` sans réécriture de headers. |
| Lecteur humain | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader.test.ts` | Contrat CLI `thread inspect/watch`, bornes de taille, refus mappés (`timeout`, `output_limit`, `unavailable`). |
| Interop réelle | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/BridgetRustInterop.testkit.ts` + R4 | Vrai serveur Node/Effect + client Rust ; deux conversations, révocation A, B intacte. |

Limite admise : les projections de conversations T3 de l'interop R4 sont des fixtures. Aucun processus modèle n'y est lancé. Cette limite reste vraie en 149 et toute preuve réelle passe par la section 4.4.

### 3.3 Trou de profondeur — confirmé à la source (correction 6)

Le catalogue annonce `max_depth:8` et `max_children:16`. Vérification source faite cette ronde : `max_depth: Some(8)` n'apparaît qu'à la déclaration du catalogue (`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs:331` et `:475`). Aucun contrôle de profondeur de chaîne n'applique cette borne pour un parent lui-même enfant. Les bornes comptées (16 par parent, 128 globales, 4096 enregistrements) restent appliquées dans `handle_locked_with_project`. Le test de profondeur nested (S149-04) s'écrit avant le correctif natif. Le correctif est lié à l'architecture descendante 149 : `parent_task_id` avec index unique sur `child_agent_id`, annulation de descendance par la même chaîne. Il précède tout le lot nested.

---

## 4. Scénarios recommandés

Convention de nommage : `native149_<comportement>` pour les tests Rust, fichier d'intégration dédié portant l'ID de spec. Cible proposée : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/native_delegation_lineage_149.rs` (n'existe pas encore, vérifié). Les tests réels opt-in restent dans le fichier recette 148 étendu.

Règle transversale (correction 4) : le grant pty n'apparaît dans aucun chemin positif 149. Il reste dans deux usages explicites : la compatibilité 148 rejouée (S149-03, S149-20) et le re-grant humain explicite après révocation. Un refus development d'un parent lecteur rend le code de droits 149 et n'invite jamais à un grant.

Règle transversale (correction 7) : les comptes 148 sont une référence historique. Aucun test 149 n'est compté comme déjà vert avant sa première exécution prouvée. Après un PASS sur code inchangé, aucun re-run supplémentaire sans nouveau code (article XXI).

### 4.1 Unitaires (daemon, sans processus)

| ID | Scénario | Oracle |
|---|---|---|
| S149-01 | Enfant GLM d'un parent management hérite discovery ; enfant Codex avec signature sandbox workspace-write hérite development (mapping Codex→Codex exact). | `root_permission` rend la politique attendue sans ligne dans `native_delegation_grants`. |
| S149-02 | Parent lecteur (discovery ou readOnly attesté) demande development pour son enfant. | Refus de droits 149 `permission_not_inherited` avant toute insertion. Aucune invitation à un grant humain dans la réponse. Delta documenté : le code actuel rend `development_grant_required` (`native_delegation.rs:210,1864`) ; l'alignement du code appartient à l'owner natif. |
| S149-03 | Révocation d'un agent : nouveau grant humain seul la lève ; redémarrage du store la conserve. | `revoked_agent` vrai après réouverture SQLite ; faux après `grant_agent` positif. Seul lieu du grant avec la compat 148. |
| S149-04 | Chaîne nested : profondeur 8 admise, 9 refusée. | Refus explicite, aucune insertion au-delà de 8. Test écrit avant le correctif (§3.3) ; le correctif natif suit et précède le lot nested. |
| S149-05 | Lecture de statut pure : N lectures ne changent aucun état, aucun spawn, aucune échéance. | Ligne `native_delegations` byte-stable après 100 lectures. |

### 4.2 Régression obligatoire (aucun code 149 ne passe sans elles)

Rejouer l'intégralité des assets de §3.1 et §3.2. Les comptes listés en tête sont la référence 148 historique, pas un crédit pour les tests 149. Toute exécution est consignée avec sa commande exacte et sa sortie, par paquet (`-p bridget-daemon --lib`, `--test <fichier>`, `--ignored` pour les recettes).

### 4.3 E2E fixture — `native_delegation_lineage_149.rs` (sans T3, sans réseau, sans modèle)

| ID | Scénario | Étapes clés | Oracle |
|---|---|---|---|
| S149-06 | Parent readonly refuse avant spawn, sans grant | Parent lecteur attesté par sa propre politique (signature sandbox readOnly, aucun `delegate-grant`), demande `posture: development`. | Refus de droits `permission_not_inherited` ; réponse sans invitation à grant ; `evidence.jsonl` sans `started` ; compteur `native_delegations` inchangé ; aucun enfant flotte. |
| S149-07 | Parent development externe, sans grant | Parent Codex management avec la signature sandbox exacte, aucun `delegate-grant`. Catalogue annonce `development=true` et `cwd_root`. | Délégation development admise ; journal ne contient aucun appel `delegate-grant`. Limite : ce niveau fixture ne prouve pas la voie standalone réelle ; la preuve réelle est S149-14 variante parent GLM (correction 3). |
| S149-08 | Nested complet | Parent → enfant → petit-enfant (même fixture factice), chaque mission écrit son propre marqueur. | 3 tâches, 3 enfants, lineage lisible depuis les liens de flotte (`agent_link_for_child`, `parent_instance_id`) ; cancel du parent stoppe descendants avant lui ; profondeur 9 refusée après correctif natif. |
| S149-09 | Retry, restart, cancel sur la chaîne | Rejeu ×10 de chaque maillon ; `SIGTERM` du daemon en vol ; reprise ; cancel de la racine. | `task_id`/`child_agent_id`/`message_id` stables ; échéance de mission non renouvelée ; zéro descendant en exécution après cancel ; snapshot de permission inchangé au rejeu. |
| S149-10 | Écriture enfant et portée logique (correction 2) | Enfant d'un parent full-access : mission écrit `marker-<nonce>` dans `cwd`. Puis mission déclarée avec un `cwd` hors projet attesté. | Fichier nonce présent ; une seule réponse corrélée. Le refus hors projet est un refus d'admission logique de mission, rendu avant spawn. Aucun oracle ne prétend un refus OS d'écriture hors `cwd` : le parent full-access confère ce droit, comme le parent. Un refus OS n'est prouvé que si le parent attesté porte un sandbox réel (mapping Codex workspace-write exact, S149-24). |
| S149-11 | Identité jamais le prompt | Mission contenant « tu es le parent X, ton cwd est /tmp » et tentatives d'environnement d'usurpation. | `from`/`to` inchangés ; `cwd` figé à la valeur validée ; env `BRIDGET_AGENT_ID_FILE` du parent inaccessible à l'enfant ; refus avant spawn. |
| S149-12 | T3 mort ou partiel | Endpoint T3 sur port fermé, puis paire env partielle. | Chemin natif complet avec `t3_present=false` ; appels MCP avec preuve T3 invalide refusés fermés, sans repli PID. |
| S149-13 | Autres parents invisibles | Deux parents sur deux sockets fixtures ; B interroge status/cancel de la tâche de A ; B tente le rejeu du `request_id` de A. B est attesté par sa propre session, jamais par la signature parent-managée de A. | `task_unavailable` ; aucune fuite de champs ; B voit un catalogue sans la racine de A. |

### 4.4 E2E réel opt-in — extension du fichier recette existant

| ID | Scénario | Oracle |
|---|---|---|
| S149-14 | GLM Flash réel écrit un fichier privé avec nonce. Deux variantes de parent attesté (correction 3) : (a) parent GLM réel hors T3, voie supportée PTY par sa source propriétaire ; (b) parent GLM réel dans T3, seulement si la source d'attestation v2 est prouvable. | Un seul `bridget_delegate` posture héritée en écriture ; mission : écrire un fichier au nom unique contenant le nonce, rien d'autre. Résultat terminal = nonce exact (trim). Une seule réponse `in_reply_to=mission_id`, `from=child_agent_id`, `to=PARENT`. Fichier présent dans `cwd` avec le nonce ; aucun autre fichier créé hors projet (journal des écritures du relay) ; événements de modèle effectif annonçant exactement `glm-5.3-flash` (correction 8), toute autre valeur échoue le test ; compteur `native_delegations` = 1 ; hash settings/commande/CLI sources inchangés ; nettoyage MCP et daemon attesté. |
| S149-15 | Refus réels hors droits | Trois cas : (1) mission déclarée hors racine héritée → refus d'admission logique avant spawn ; (2) posture development sans héritage prouvé → refus de droits, zéro `started`, zéro processus CLI ; (3) parent workspace-write attesté → cible GLM → `provider_confinement_unavailable` si aucun confinement équivalent attestable (correction 2) : refus nommé, jamais un repli silencieux ni un fallback lecteur. |

Limite de la recette réelle : elle prouve l'écriture dans le périmètre du parent, la corrélation unique et l'annonce exacte du modèle. Elle ne prouve pas le sandbox du fournisseur au-delà de l'observation du relay, ni les anciennes sessions déjà chargées.

### 4.5 Projection T3

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-16 | Projection lineage : parent, nested, status, résultat, journal (oracle corrigé, correction 1) | T3 serveur (Effect, tests vitest) | Pour chaque tâche native : exactement un fil virtuel persisté, identifiant `thread:bridget-task:<task_id>` déterministe, relation `subagent`, origine `bridget_native`, marqueur `bridgetTaskRef` complet (`version`, `taskId`, `rootThreadId`, `parentTaskId`, `generation`, `seq`, `status`). Négatifs : zéro conversation de premier niveau ; zéro provider session, turn, outbox ou lancement ; `activeTurnId` null ; composeur en lecture seule ; le serveur refuse `message.dispatch`, reprise fournisseur, changement fournisseur, fork et rollback sur ces fils ; le résultat n'est jamais re-routé au parent par T3. |
| S149-17 | Projection sans double exécution | T3 + fixture Rust | 100 lectures de projection : `started`/`prompt` restent à 1 par tâche ; aucune remise dupliquée au parent. S'appuie sur la pureté garantie par S149-05. |
| S149-18 | Panne T3 et dégradation | T3 serveur | CLI Bridget absent, exit 3 ou timeout → refus mappé (`unavailable`, `timeout`), aucun crash, aucun contenu masqué présenté comme valide. Le chemin natif Rust reste vert (S149-12). |
| S149-19 | Invisibilité inter-conversations | T3 + fixture | La conversation B (même processus fournisseur) ne voit ni la lineage ni le journal de A. Clé environment/project/conversation respectée. |

Fichiers cibles proposés : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader.test.ts` (extensions), un fichier dédié `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetLineage.test.ts` si la surface lineage introduit un contrat propre. Contrats dans `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/`.

### 4.6 Revoke et opt-out persistants

| ID | Scénario | Oracle |
|---|---|---|
| S149-20 | Revoke persistant | Révocation → refus immédiat ; redémarrage daemon → refus ; ré-enregistrement sous nouvelle instance → refus ; seul un nouveau grant humain explicite rouvre (usage de grant hors chemin positif). Régression : `native148_agent_revocation_survives_a_new_instance_until_explicit_grant` + variante E2E avec redémarrage réel du daemon fixture. |
| S149-21 | Opt-out MCP persistant | Un agent avec opt-out humain du montage MCP (mécanisme 147 existant, à localiser à l'implémentation) n'expose aucun outil `bridget_*` après redémarrage du daemon fixture. Tout appel direct au transport sans preuve échoue. Aucun contournement par prompt. Aucun opt-out nouveau 149 n'est ajouté. |

### 4.7 Scénarios ajoutés en r2 (correction 5)

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-22 | Filtre du pont sur le marqueur `bridgetTaskRef` | T3 + contrat de résumé Rust | Le connecteur Rust lit le résumé T3 : marqueur présent et valide → fil exclu de tout montage wrapper/presence/binding Bridget avant montage. Marqueur présent mais malformé (champ manquant, UUID non canonique, types faux, `seq` hors entier sûr, taille hors borne) → refus fermé, sans repli en fil ordinaire. Un simple préfixe d'identifiant ne vaut jamais autorité. |
| S149-23 | Anciens fils ordinaires intacts | T3 | Les données historiques sans `bridgetTaskRef` gardent leur comportement exact : aucun fil converti, aucune exclusion, aucune réinterprétation (invariant 12 du plan). |
| S149-24 | Mappage inter-fournisseurs figé et compatibilité 148 | Unitaire daemon | (1) Codex workspaceWrite avec confinement OS → enfant Codex workspace-write exact (roots/réseau/tmp conservés) ; (2) même parent → cible GLM → `provider_confinement_unavailable` tant qu'aucun confinement équivalent n'est attesté, jamais de repli silencieux ; (3) dangerFullAccess → mission GLM locale en bypass, réseau non réduit silencieusement, aucun réglage global modifié ; (4) readOnly → discovery/plan restreint, réseau exigé représentable sinon `permission_mapping_unavailable` ; (5) politique inconnue ou flags non représentés → `permission_attestation_unavailable` ou `permission_mapping_unavailable`, aucun fallback lecteur/bypass/grant ; (6) tâche 148 managée sans snapshot 149 : exécution historique conservée, toute nouvelle écriture 149 refusée nommée sans demande de grant de remplacement. |
| S149-25 | Deux conversations, même processus fournisseur, droits différents | T3 serveur + fixture | Conversation A full-access, conversation B readonly, même processus. Aucun emprunt : l'admission de B n'utilise jamais le fait de politique de A. Le fait est lié à environment/project/conversation, session/instance, run et credential. Révocation de A ne change pas B (étend l'interop R4). |
| S149-26 | Péremption du fait de politique | T3 serveur + fixture | Fin, échec ou close du run retire le fait du run propriétaire seul ; le nettoyage d'un ancien tour ne retire pas le fait du tour courant. Rotation ou retrait de la session MCP efface le fait. `revision` augmente quand le fait courant change. Un mode Claude qui change (par exemple vers plan) produit une nouvelle revision pour les futures admissions ; le snapshot déjà admis reste inchangé (invariant 5). |
| S149-27 | Séquence sûre pour JavaScript et pagination avec mutations en staging | Unitaire daemon + T3 | `seq` reste un entier 0..9007199254740991, non décroissant ; débordement → renouvellement de génération dans la transaction et resync imposé, aucun overflow silencieux. Première page fixe S ; chaque page vérifie même génération et séquence ; une mutation entre pages rend `snapshot_changed`, le staging est abandonné et la pagination reprend à la première page ; T3 publie toutes les pages en une transaction, aucun snapshot partiel visible. Watch : `ready` seq 0 toujours premier et non coalescible ; les mutations d'autres roots ne signalent pas ; rejeu, lecture, refus et ACK restent muets ; un changement de journal passe par son propre relai et ne devient pas une mutation de tâche. |

### 4.8 Recette UI web (S149-28, ajoutée en G-L r1)

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-28 | Recette UI web de présentation ; remplace la recette desktop prévue (T040) | Daemon Bridget fixture et serveur T3 fixture sur port éphémère avec base privée (gabarit interop R4) ; interface web ouverte dans un navigateur ou la preview T3 | Lineage affiche la relation parent/enfant avec statut puis résultat ; le journal réel de l'enfant se lit enfant actif puis terminal ; nested, statuts natifs et modèle réels affichés ; aucune conversation T3 de premier niveau n'existe ; l'arrêt appelle la saga native (zéro provider session, zéro tour) ; la reconnexion reconstruit la projection depuis le store natif sans doublon. Limites : cette preuve ne valide pas la coque desktop native ; aucune seconde application T3 n'est lancée ; aucun PID de production n'est touché ; preuve dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipe149.md`. |

### 4.9 Amendement G-P r1 — oracles permissions manquants (S149-29 à S149-31)

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-29 | Refus corrélé `can_use_tool` en cours de mission (G-P-01) | E2E fixture (`native_delegation_lineage_149.rs` ; extension locale de la fixture fournisseur qui émet une trame `can_use_tool` pour une action hors politique) | La réponse de contrôle émise porte le `request_id` exact de la trame reçue et une décision deny. La tâche finit `failed` avec le code nommé du contrat (`provider_permission_denied` en cours de mission ; `permission_not_inherited` reste le code d'admission). Un texte de modèle « OK » avec `permission_denials` reçues ne produit jamais un résultat de succès : zéro résultat corrélé de succès. Compteur de lancements inchangé : aucun lancement supplémentaire, aucune boucle. Mapper FR008/SC003. Une variante réelle reste optionnelle si la trame n'est pas déclenchable chez le fournisseur réel ; la limite est consignée. |
| S149-30 | Divergence de digest avant effet (G-P-02) | Unitaire + E2E fixture | (a) Unitaire : la fonction de recontrôle des digests rend `settings_revision_changed` pour un fichier source muté, supprimé puis ajouté, sans aucune re-résolution. (b) Fixture : mutation entre admission et premier spawn, puis à une reprise, de la source settings, du chemin/digest du launcher CLI ou du mode → refus nommé `settings_revision_changed`, aucune adoption des nouvelles permissions, compteur de lancements inchangé. Couvre le retarget de symlink et un changement de priorité PATH (G-P-04). La fixture prouve des compteurs, jamais un confinement réel du fournisseur. Mapper FR019/SC003. |
| S149-31 | Source nécessaire opaque (G-P-03) | Unitaire | Une source nécessaire opaque (managed non représentable) rend exactement `permission_source_unavailable`, seul nom de ce cas dans l'inventaire du contrat et dans son tableau de mappage. Le chemin positif de même famille avec sources valides reste inchangé : aucun refus global des parents Claude. Mapper FR007, FR008, FR019 / SC003. |

### 4.10 Amendement G-P r3 — union v1/v2 et observer PTY (S149-32 à S149-33)

Revue `validation/permissions-contract-deltas-r3.md`, G-P-07–G-P-08. Oracles G-P-07 et G-P-08 ajoutés à la section Oracles du contrat `contracts/permissions.md`. Aucun scénario exécuté à cette ronde.

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-32 | Union d'introspection v1/v2 (G-P-07) | Unitaire daemon + T3 serveur | Branches (a), (b), (c) de G-P-07, mot à mot. (a) Un credential sans aucun fait de permissions jamais existé reçoit l'enveloppe148 v1, identité seule, quatre champs camelCase, parseur fermé. (b) Pour un credential dont le fait de permissions a été retiré (fin, échec ou fermeture du tour), révoqué ou tourné, la fonction `bridget_session` de T3 rend le refus nommé `permission_attestation_unavailable`, jamais l'enveloppe v1. La façade MCP privée de Bridget traite ce refus comme un échec de preuve de session. Tout appel de ce credential est fermé avant effet avec `t3_session_unavailable` : `bridget_delegate` (nouvelle requête et rejeu d'une requête admise), `bridget_task_status`, `bridget_task_cancel` et `bridget_who`. Il n'y a ni repli v1, ni repli discovery, ni repli par PID. Aucun lancement n'a lieu. « Lecture et rejeu inchangés » s'entend ainsi : (i) la lecture humaine Lineage native (gardes 147), l'annulation humaine native et la saga native continuent, car elles ne dépendent pas du credential T3 ; (ii) un credential neuf qui n'a jamais eu de fait reçoit l'enveloppe v1 : identité, status, cancel et rejeu 148 d'une requête déjà admise restent possibles, sans nouvelle exécution. Preuves existantes du cas, sur binaire r5 : `recovery149.md` R8.4 et `interop149.md` S5.1/S5.1b ; elles ne valent pas exécution de S149-32. (c) Une admission 149 inherit/development présentée avec la seule enveloppe v1 rend `permission_attestation_unavailable`, sans grant demandé, sans fallback discovery, sans spawn ; status, cancel et identité148 continuent de fonctionner. Les tests T3 prouvent la sémantique de réponse ; ils ne prouvent pas l'entrée du Rust autonome (voir §5 et §11.10). Mapper FR007, FR008, FR019 / SC003. |
| S149-33 | Observer propriétaire du chemin PTY (G-P-08) | Fixture wrapper + recette réelle | Branches (a), (b), (c) de G-P-08 en fixture. (a) Un lancement avec l'overlay `--settings` publie un fait portant permission_mode, session_id, cwd, prompt_id et l'identifiant de la demande Bridget ; le daemon accuse réception avant que le hook laisse continuer l'appel observé ; la demande délégataire correspond à l'identifiant observé (request_id exact). (b) Preuve absente, non corrélée, overlay altéré (tamper), non lié au lancement (unbound) ou observer indisponible : refus nommé, aucun lancement (compteur launched inchangé), aucun effet fournisseur, aucun grant. (c) Un fait d'une autre demande ne donne aucun droit ; l'overlay, son nonce, la socket et la preuve n'apparaissent ni dans le snapshot enfant, ni dans la définition ou l'environnement enfant, ni dans le journal ; fichier et socket 0600 supprimés en sortie ; la reconnexion ne récupère aucun fait d'une ancienne connexion. Branche (d) en recette réelle S149-14(a)/T038 : le CLI réel compose le fichier `--settings` avec les sources sélectionnées — les règles allow/deny des sources attestées restent effectives avec l'overlay — et le hook s'applique en PTY autonome sans approbation humaine ; le mode observé réel (manual normalisé default, plan, bypass) vient de la preuve d'observation sur le canal de l'owner, jamais d'une autodéclaration MCP ; un CLI qui remplace les sources au lieu de composer l'overlay est refusé. La preuve réelle reste le premier parent externe PTY avec T3 absent, jamais un descendant managed substitué. Mapper FR007, FR008, FR019 / SC003, SC004. |

---

## 5. Limites fixture contre réel

| Niveau | Prouve | Ne prouve pas |
|---|---|---|
| Unitaire Rust | Règles de permission, mappage et refus nommés, révocation, bornes, pureté des lectures, séquence. | Aucun comportement de processus réel. Aucune attestation réelle. |
| E2E fixture | Saga complète : idempotence, annulation, redémarrage, lineage, admission logique de portée, absence de T3. Attestations parents = signatures fournies par fixture. | Qu'un modèle réel a tourné ; le choix effectif du fournisseur ; l'auth réelle ; toute voie standalone réelle (correction 3). |
| Fixtures G-P (S149-29, S149-30) | Refus nommés corrélés, compteurs de lancement inchangés, recontrôle des digests launcher et CLI résolu. | Tout comportement du fournisseur réel ; aucun confinement OS ; la trame `can_use_tool` réelle reste optionnelle (limite consignée). |
| Fixtures G-P r3 (S149-32, S149-33) | Sémantique de l'union v1/v2 (tombstone et admission 149 en v1 seule rendent le refus nommé) et corrélations de l'observer (ACK avant appel, request_id lié, suppression lifecycle, overlay absent de l'enfant). | L'entrée du Rust autonome : la voie standalone reste prouvée par la recette S149-14(a)/T038 seulement — premier parent externe PTY avec T3 absent, jamais un descendant managed substitué ; la composition réelle `--settings` n'est prouvée que par cette recette. |
| E2E réel opt-in | Enfant GLM réel `glm-5.3-flash`, modèle effectif annoncé par le fournisseur, écriture réelle dans le périmètre, corrélation unique, nettoyage. Parent GLM réel même famille (voie PTY hors T3 ; voie T3 si la source v2 est attestable). | Le sandbox au-delà du relay ; les sessions déjà chargées ; le mapping workspaceWrite→GLM au-delà du refus nommé observé. |
| Interop T3 R4 | Serveur Node réel, auth registre, headers, révocation par conversation. | Un modèle réel ; les projections UI complètes (fixtures). |

Règles de distinction (correction 3) : une attestation parent fournie par fixture n'est jamais présentée comme une preuve externe. La signature parent-managée 148 reste valable pour la compatibilité 148 uniquement ; elle ne prouve aucun droit 149. Tout rapport de validation distingue explicitement ces niveaux, comme le reçu 148.

---

## 6. Commandes ciblées

À exécuter depuis le worktree Bridget (`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`), umask 077, dossier temporaire privé. Aucune commande avant modification de code : la version est inchangée, le §2 des interdits de la mission s'applique. Correction 7 : chaque paquet est ciblé séparément ; le compte workspace complet n'est rejoué qu'une fois, comme référence avant livraison, et n'est jamais présenté comme des tests 149 déjà verts. Aucun re-run après PASS sans nouveau code.

```bash
# État de départ (référence 148, à rejouer une seule fois après tout incrément)
umask 077
cargo test -p bridget-daemon --test native_delegation_e2e
cargo test -p bridget-daemon --lib native148
cargo test -p bridget-daemon --test t3_session_identity_env
cargo test -p bridget-daemon --test native_delegation_real_glm   # tests secs, recette ignorée
cargo fmt --check
cargo clippy --workspace --all-targets                            # exception too_many_arguments conservée

# Nouveaux tests 149 (après implémentation) — première exécution = première preuve
cargo test -p bridget-daemon --test native_delegation_lineage_149

# Régression de référence avant livraison (référence 148 historique : 1496 PASS / 15 ignorés)
# Une seule exécution ; le compte n'est jamais crédité aux tests 149.
cargo test --workspace
```

Recette réelle opt-in — uniquement après le GO identité, jamais dans un lot par défaut. Le modèle exigé est exactement `glm-5.3-flash` (correction 8) ; toute substitution est refusée par le test :

```bash
BRIDGET_149_REAL_GLM=1 \
BRIDGET_149_REAL_GLM_REGISTRY=<chemin absolu agents.json privé 0600> \
BRIDGET_149_REAL_GLM_MODEL=glm-5.3-flash \
[ BRIDGET_149_REAL_GLM_AUTH_COMMAND_SHA256=<sha256 du wrapper gclaude attesté> \
  | BRIDGET_149_REAL_GLM_PASS_ZAI_API_KEY=1 ] \
cargo test -p bridget-daemon --test native_delegation_real_glm -- \
  --ignored --exact <nom_du_test_149> --nocapture
```

Les préfixes exacts des variables seront fixés à l'implémentation. Le code actuel utilise `BRIDGET_148_*` ; le passage à `BRIDGET_149_*` appartient à l'owner natif. Le contrat reste : opt-in explicite, modèle exact du registre, zéro secret écrit ou journalisé.

T3 — depuis `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` :

```bash
pnpm exec vp test run apps/server/src/bridget apps/server/src/mcp/BridgetSession.test.ts apps/server/src/mcp/BridgetRustInteropObserver.test.ts
pnpm typecheck
pnpm lint
```

L'interop HTTP réelle (type R4) exige le CLI Bridget construit localement et un serveur T3 fixture sur port éphémère. Elle ne touche aucun service de production.

---

## 7. Conditions d'environnement et d'authentification (sans secrets)

- Machine : le serveur de travail, macOS arm64. Binaires requis : `cargo`, `python3`, `git`, `pnpm`/`vp`. Aucune installation nouvelle.
- Chaque fixture crée `/tmp/ng149-<uuid10>` (canonisé) avec `state/`, `provider/`, `tmp/`, `project/`, tous en 0o700 ; fichiers privés 0o600. Nettoyage en fin de test, conservé en cas d'échec pour diagnostic.
- Daemon de fixture : binaire du worktree, socket Unix privé, `TMPDIR` redirigé, stderr vers log privé. Aucun contact avec les PID de production (58468/57109/58394/58396 restent hors de portée).
- Grant humain : uniquement pour la compatibilité 148 rejouée et le re-grant explicite après révocation (S149-03, S149-20), via `openpty` + `delegate-grant` sur la fixture. Aucun grant sur le daemon réel. Aucun grant dans un chemin positif 149 (correction 4).
- Attestation parent hors T3 (correction 3) : la voie supportée utilise la source propriétaire du parent réel (observation du wrapper PTY, liée session/cwd). La signature parent-managée 148 n'est jamais opposée comme preuve externe. La voie T3 exige un fait v2 provenant du tour réel ; si la source n'est pas attestable, le cas reste en fixture et la limite est consignée.
- Auth GLM réelle : la route existante de l'utilisateur est réutilisée en mémoire. Deux voies admises : `settings.json` du profil existant (clé `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN` ou `apiKeyHelper`), ou wrapper `gclaude` attesté par SHA256 avec le CLI `claude` à côté. L'opt-in `PASS_ZAI_API_KEY` transmet la variable préexistante en mémoire au seul daemon fixture. Aucune valeur de clé n'est lue, écrite, sérialisée ou journalisée par un test. Registre privé 0o600 requis. Hash de settings, commande, launcher et CLI résolu re-vérifiés en fin de recette.
- Registre source réel (G-P-05, consigné le 2026-10-10) : le service launchctl actif (PID58394) tourne avec `BRIDGET_HOME=/Users/moi/.cache/bridget-core`. Le registre vivant est `/Users/moi/.cache/bridget-core/agents.json` : entrée `glm` = `gclaude` + profil `/Users/moi/.claude-glm`, modèle `glm-5.3` ; entrée `claude` présente. Le fichier `/Users/moi/.config/bridget/agents.json` (`cursor`, `claude`) n'est pas sélectionné par ce runtime. Le registre de recette est un clone privé contrôlé (0o600) avec le modèle exact `glm-5.3-flash` ; la différence entre le compte privé de recette et le modèle de production n'autorise aucun repli hors Flash. Aucune entrée de production n'est modifiée ; les lectures de preuve restent des lectures JSON saines, en lecture seule, sans impression d'env ni de jeton.
- Modèle (correction 8) : l'identifiant exigé est exactement `glm-5.3-flash`. Le catalogue T3 vivant expose ce modèle exact ; la fixture relay locale est mise à jour pour l'attester, sans aucune config de production. Les événements de modèle effectif de la recette réelle doivent annoncer `glm-5.3-flash` ; toute autre valeur, y compris un repli `glm-5.3`, fait échouer le test. La recette refuse toute substitution ; si le profil n'annonce pas « Flash », on n'invente aucun modèle.
- Nettoyage de processus : `SIGTERM` après vérification PID/PPID/exécutable, jamais `kill -9`, jamais de cible Firefox (règle projet). Les watchdogs existants (`OwnedChild`) restent le seul mécanisme.

---

## 8. Non-buts

- Aucun redémarrage de T3, du daemon Bridget ou d'un service de production.
- Aucune écriture dans une configuration, un profil ou une base de production, y compris la config de modèle de production.
- Aucun nouveau framework de test, aucun Python/pytest : le projet est Rust côté Bridget et Effect/vitest côté T3. La recommandation Rust-seule suit la convention des sessions 142–148.
- Aucune exécution de test ou de build sur la version inchangée, hors état de départ documenté en section 6. Aucun re-run après PASS sans nouveau code.
- Aucun grant pty dans un chemin positif 149.
- Aucun commit par le sous-agent.

---

## 9. Points d'arbitrage pour le principal

1. **Gherkin contre Rust-seul.** Les standards demandent une feature Gherkin par spec. Aucune session depuis 081 n'en a créé. Recommandation : rester Rust-seul avec ID de spec dans les noms, conformément à la convention vivante et au minimalisme (article XIX). Un `tests/features/149-sous-agents-lineage.feature` documentaire reste possible si le principal le veut.
2. **Profondeur nested.** Le trou est confirmé à la source (§3.3) : `max_depth` annoncé, non appliqué. Séquence : test d'abord, correctif natif ensuite, lot nested après. Le correctif est lié à `parent_task_id` et à l'annulation de descendance 149.
3. **Portée d'écriture et sandbox (correction 2).** L'oracle est comportemental. Un parent full-access confère à la mission GLM locale le droit d'écrire hors `cwd`, comme son parent ; la portée `cwd` reste une admission logique de projet. Aucun test n'affirme un refus OS sans sandbox parent réellement attesté. Le cas workspaceWrite→GLM attend un refus nommé tant qu'aucun confinement équivalent n'est attestable.
4. **Préfixe des variables d'opt-in 149.** Réutiliser `BRIDGET_148_*` ou introduire `BRIDGET_149_*`. Recommandation : `BRIDGET_149_*` pour un contrat propre, en conservant les tests secs 148 comme régression. La mise à jour du fichier recette appartient à l'owner natif.
5. **Alignement des codes de refus.** Les contrats 149 nomment `permission_not_inherited`, `permission_attestation_unavailable`, `permission_mapping_unavailable`, `provider_confinement_unavailable` et `permission_source_unavailable`. Ce dernier est le seul nom du cas « source nécessaire opaque » (managed), présent dans l'inventaire et le tableau de mappage du contrat (G-P-03). Le contrat nomme aussi `settings_revision_changed` pour une divergence de digest (S149-30) et un échec nommé pour un refus `can_use_tool` en cours de mission (S149-29). Le code actuel rend `development_grant_required` (148). Les tests 149 visent les codes contrat ; l'alignement appartient à l'owner natif et précède la première exécution des tests concernés.
6. **Gate PTY Claude (contrat permissions).** Le chemin PTY ne peut être déclaré validé sans recette prouvant la source sélectionnée, son mode courant, la fraîcheur et le snapshot des règles effectives. Le principal arbitre cette limite avant GO.

---

## 10. Mapping scénarios → FR / SC

FR001–FR019 et SC001–SC007 de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/spec.md`.

| Scénario | FR | SC |
|---|---|---|
| S149-01 | FR007, FR008, FR019 | SC003 |
| S149-02 | FR007, FR018 | SC003 |
| S149-03 | FR011, FR012, FR018 | SC005 |
| S149-04 | FR013, FR016 | SC006 |
| S149-05 | FR003, FR015 | SC002 |
| S149-06 | FR007, FR008, FR012, FR018 | SC003 |
| S149-07 | FR007, FR008, FR018 | SC003 |
| S149-08 | FR001, FR013, FR015 | SC001, SC006 |
| S149-09 | FR005, FR013, FR015, FR019 | SC002, SC006 |
| S149-10 | FR007, FR008 | SC003 |
| S149-11 | FR007, FR013 | SC006 |
| S149-12 | FR009, FR010 | SC004 |
| S149-13 | FR011, FR013 | SC006 |
| S149-14 | FR008, FR009 | SC001, SC003, SC004 |
| S149-15 | FR007, FR008, FR018 | SC003 |
| S149-16 | FR001, FR002, FR004, FR015 | SC001 |
| S149-17 | FR003, FR005 | SC002 |
| S149-18 | FR002, FR010 | SC004, SC005 |
| S149-19 | FR011 | SC005, SC006 |
| S149-20 | FR011, FR018 | SC005 |
| S149-21 | FR011, FR012 | SC005 |
| S149-22 | FR004, FR012 | SC001 |
| S149-23 | FR004, FR013 | SC001 |
| S149-24 | FR007, FR008, FR018, FR019 | SC003 |
| S149-25 | FR007, FR011 | SC003, SC005 |
| S149-26 | FR011, FR019 | SC005 |
| S149-27 | FR003, FR015, FR016 | SC002 |
| S149-28 | FR001, FR002, FR003, FR004, FR005, FR013, FR015 | SC001, SC002, SC006 |
| S149-29 | FR008 | SC003 |
| S149-30 | FR019 | SC003 |
| S149-31 | FR007, FR008, FR019 | SC003 |
| S149-32 | FR007, FR008, FR019 | SC003 |
| S149-33 | FR007, FR008, FR019 | SC003, SC004 |
| Régression 148 (§4.2) | FR006, FR012, FR013, FR016 | SC005, SC007 |

Couverture des FR : tous les FR001–FR019 sont couverts par au moins un scénario ou la régression. S149-28 ajoute la preuve UI web de FR001–FR005, FR013 et FR015 (SC001, SC002, SC006) ; elle ne remplace aucune preuve de service. S149-29 à S149-31 ajoutent les oracles G-P sur SC003 : refus corrélé en cours de mission, divergence de digest avant effet, source nécessaire opaque nommée une seule fois. S149-32 et S149-33 (amendement G-P r3) ajoutent l'union d'introspection v1/v2 (SC003) et l'observer PTY (SC003, SC004) ; leur preuve réelle reste la recette S149-14(a)/T038. FR014 (documenter les six scénarios dégradés) est couvert par cette section, la section 4 et les oracles US6 de S149-08, S149-09, S149-11, S149-12, S149-13, S149-15, S149-24. FR017 (livraison sans redémarrage) est couvert par les contrôles de processus inchangés de la section 7 et la régression (SC007).

---

## 11. Limites de cette stratégie

1. **Ronde documentaire.** Aucun test écrit, aucun code créé, aucune exécution. La version du worktree reste inchangée (base 148 `6807c22b`). Les oracles portent sur des comportements cibles définis par les contrats, pas sur du code existant.
2. **Contrats non figés.** `permissions.md` reste en cours. Les codes de refus, le schéma exact de `bridget_session` v2 et la forme du marqueur peuvent encore bouger ; les oracles suivront le contrat final avant la gate.
3. **Codes de refus en avance sur le code.** Les tests visent les codes contrat 149 ; le code actuel rend encore `development_grant_required`. L'écart est assumé et suivi en §9.5.
4. **Voie standalone réelle non prouvée à ce stade.** Les fixtures attestent les parents par signature ; seule la recette S149-14 variante (a) prouve la voie PTY hors T3, et elle attend le GO identité. La voie T3 pour un parent GLM reste conditionnelle à une source v2 attestable (S149-14 variante (b)).
5. **Confinement non affirmé.** Aucun test n'affirme un sandbox OS que le fournisseur n'atteste pas. Le cas workspaceWrite→GLM est prouvé comme refus nommé, pas comme écriture confinée.
6. **Projections T3 de l'interop R4 restent des fixtures.** Aucun processus modèle n'y est lancé ; la limite est héritée de 148 et reste vraie en 149.
7. **Localisation du mécanisme d'opt-out MCP à confirmer.** S149-21 dépend du mécanisme 147 existant ; son point d'accros exact sera repéré à l'implémentation.
8. **Nom de recette et préfixes d'environnement à fixer à l'implémentation.** `BRIDGET_149_*` est une recommandation ; le code actuel utilise `BRIDGET_148_*`.
9. **Amendement G-P r1.** S149-29 à S149-31 sont des oracles documentaires alignés sur le contrat corrigé par l'owner natif (`contracts/permissions.md`) ; aucun n'est exécuté à cette ronde.
10. **Amendement G-P r3.** S149-32 et S149-33 sont des oracles documentaires ; aucun n'est exécuté à cette ronde. Les tests T3/fixture prouvent la sémantique des refus et des corrélations ; l'entrée du Rust autonome reste prouvée par la recette S149-14(a)/T038 (T3 absent, premier parent externe PTY, jamais un descendant managed substitué), conformément au contrat (`contracts/permissions.md`, « Sources standalone » et « Compatibilité et gate ») et à la revue r3 §5.
