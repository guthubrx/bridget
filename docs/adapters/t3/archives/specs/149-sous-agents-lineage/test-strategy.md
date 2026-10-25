# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/test-strategy.md](../../../../../../specs/149-sous-agents-lineage/test-strategy.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Amendement G-L r1 (revue `validation/plan-lineage-r1.md`, F1–F7) : S149-28 (recette UI web, §4.8) s'ajoute et remplace la recette desktop prévue, qu'une seconde instance T3 rendait impossible. Total : 28 scénarios, aucun exécuté.

<a id="extrait-2"></a>

## Extrait 2

Base de preuve 148 (historique, référence seule — correction 7, ce ne sont pas des tests 149 déjà verts) : 1496 PASS / 15 ignorés (workspace Rust), 45 ciblés, 2 E2E Rust, 389 PASS / 1 SKIP de régression identité T3, 6 PASS d'interop HTTP réelle R4, recette réelle GLM en 55 s. Reçu : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/148-identite-delegation/validation/final.md`.

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

| Objectif utilisateur | Exigence de test | Oracle principal |
|---|---|---|
| Véritables enfants Bridget en un appel | Un `bridget_delegate` crée tâche + enfant + mission durables, une seule fois. | 1 ligne `native_delegations`, 1 événement `started`, 1 événement `prompt`. |
| Lineage T3 : projection parent/nested/status/result/journal | La projection est une lecture pure des données Bridget. Elle crée exactement un fil virtuel persisté par tâche. (Correction 1 — l'oracle r1 « aucune ligne thread T3 » était faux.) | Un fil virtuel par tâche : identifiant déterministe `thread:bridget-task:<task_id>`, nœud `node:bridget-task:<task_id>`, relation `subagent`, origine `bridget_native`, marqueur `bridgetTaskRef` complet. Négatifs : zéro conversation T3 de premier niveau ; zéro provider session, turn, outbox fournisseur ou effet de lancement ; `activeTurnId` null ; le résultat n'est jamais remis au parent par T3 (remise native 148 unique). |
| Droits hérités du parent attesté, parent Codex ET GLM, sans grant humain supplémentaire | Un parent qui a lui-même le droit d'écrire confère ce droit à son enfant. Aucun grant humain dans le chemin positif 149. (Correction 4.) | Enfant écrit le fichier nonce dans `cwd`. Un parent full-access confère une mission GLM locale en bypass : l'enfant peut écrire hors `cwd` comme son parent. La portée `cwd` est une admission logique de projet, pas un sandbox OS ; le catalogue n'affirme aucun confinement que le fournisseur n'applique pas. |
| Identité et confinement selon le parent attesté, jamais le prompt | L'identité vient de la preuve native. Le texte de mission ne change ni `from`/`to` ni `cwd` ni posture. Tout parent hors 149 est attesté par une session réelle v2 ou par sa source propriétaire standalone — jamais par la signature parent-managée 148 opposée comme preuve externe. (Correction 3.) | Réponse corrélée `from=child_agent_id`, `to=parent`. Tentatives d'usurpation par prompt ou environnement refusées avant spawn. Fixtures et preuves réelles distinguées dans la section 5. |
| dans un terminal idem | Le même chemin natif réussit par le parcours natif Bridget et avec un endpoint T3 mort. | `t3_present=false` ; mission terminée ; refus fermé des appels MCP avec preuve invalide. |
| Pas de restart ni de config de production | Tous les tests isolent daemon, socket, DB, TMPDIR dans un dossier privé de `/tmp`. | Aucun PID production touché. Hash des settings/profil/commande source inchangés. |

<a id="extrait-5"></a>

## Extrait 5

1. Recette réelle : GLM Flash écrit un fichier privé avec nonce, résultat corrélé unique.
2. Parent readonly : refuse avant spawn.
3. Parent development externe : pas de grant.
4. Revoke et opt-out persistants.
5. Nested, retry, restart, cancel.
6. Panne T3 et invisibilité des autres parents.

<a id="extrait-6"></a>

## Extrait 6

| Asset | Chemin | Ce qu'il prouve déjà | Réusage 149 |
|---|---|---|---|
| Harnais partagé | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/support/idempotent.rs` | `private_dir`, `private_write` (0o700/0o600), `isolated_command` (env fermé), `Client` de transport, nettoyage d'enfants. | Base de toute nouvelle fixture. |
| E2E natif | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/native_delegation_e2e.rs` | Refus sans grant ; grant humain par pty ; retry ×10 = 1 tâche ; réponse corrélée unique ; annulation ; reprise après redémarrage du daemon ; `t3_present=false`. | Régression 148 directe. Ses helpers (`Fixture`, `OwnedChild` avec vérification PID/exécutable et refus Firefox) sont le moule des tests 149. Le grant pty qu'il utilise reste hors du chemin positif 149. |
| Recette réelle opt-in | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/native_delegation_real_glm.rs` | Parent Codex fixture + GLM réel, modèle exact exigé, un seul enfant et une seule réponse corrélée, nettoyage MCP attesté, hash des settings/commandes inchangés. | Moule de la recette 149. Ses préfixes d'environnement sont `BRIDGET_148_*` et son exemple de modèle est `glm-5.3` (ligne 13) : la mise à jour vers `BRIDGET_149_*` et `glm-5.3-flash` appartient à l'owner natif, l'oracle 149 exige Flash exact. Ses 4 tests sec (`auth`, cleanup EOF, cleanup unwind) restent la régression. |
| Tests unitaires inline | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs` (20 tests `native148_*`) | Portée owner de status/cancel ; révocation d'agent qui survit à une nouvelle instance ; refus cwd et développement ; seul l'humain grant ; rejeu avant révocation ; découverts descendants avant parent ; échéance figée ; catalogue GLM ; fin de tour ≠ résultat. | Régression directe + modèle de nommage `native149_*`. |
| Identité T3 | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/t3_session_identity_env.rs` | Preuve partielle ou invalide : jamais de repli PID. | Régression + extension endpoint mort. |
| Fournisseur factice | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/fixtures/native_delegation_148.py` | Écrit `evidence.jsonl` (`started`, `prompt`, `blocked`), bloque sur `WAIT_CANCEL_148`. | Réutilisé tel quel pour les tests lineage fixture. |
| Relay réel | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/fixtures/native_delegation_real_glm_relay.py` | Journalise le modèle effectif annoncé par le vrai CLI. | Réutilisé ; étendu au journal des écritures effectuées et à l'attestation du modèle `glm-5.3-flash` effectif. Mise à jour de fixture locale, sans config de production. |

<a id="extrait-7"></a>

## Extrait 7

| Asset | Chemin | Ce qu'il prouve déjà |
|---|---|---|
| Attestation de session | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/BridgetSession.test.ts` | Réponse `bridget_session` version 1, sans secret. |
| Observateur headers | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/BridgetRustInteropObserver.test.ts` | Conservation de `Mcp-Session-Id` sans réécriture de headers. |
| Lecteur humain | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader.test.ts` | Contrat CLI `thread inspect/watch`, bornes de taille, refus mappés (`timeout`, `output_limit`, `unavailable`). |
| Interop réelle | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/BridgetRustInterop.testkit.ts` + R4 | Vrai serveur Node/Effect + client Rust ; deux conversations, révocation A, B intacte. |

<a id="extrait-8"></a>

## Extrait 8

Limite admise : les projections de conversations T3 de l'interop R4 sont des fixtures. Aucun processus modèle n'y est lancé. Cette limite reste vraie en 149 et toute preuve réelle passe par la section 4.4.

<a id="extrait-9"></a>

## Extrait 9

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

<a id="extrait-10"></a>

## Extrait 10

| ID | Scénario | Oracle |
|---|---|---|
| S149-14 | GLM Flash réel écrit un fichier privé avec nonce. Deux variantes de parent attesté (correction 3) : (a) parent GLM réel dans un terminal, voie supportée PTY par sa source propriétaire ; (b) parent GLM réel dans T3, seulement si la source d'attestation v2 est prouvable. | Un seul `bridget_delegate` posture héritée en écriture ; mission : écrire un fichier au nom unique contenant le nonce, rien d'autre. Résultat terminal = nonce exact (trim). Une seule réponse `in_reply_to=mission_id`, `from=child_agent_id`, `to=PARENT`. Fichier présent dans `cwd` avec le nonce ; aucun autre fichier créé hors projet (journal des écritures du relay) ; événements de modèle effectif annonçant exactement `glm-5.3-flash` (correction 8), toute autre valeur échoue le test ; compteur `native_delegations` = 1 ; hash settings/commande/CLI sources inchangés ; nettoyage MCP et daemon attesté. |
| S149-15 | Refus réels hors droits | Trois cas : (1) mission déclarée hors racine héritée → refus d'admission logique avant spawn ; (2) posture development sans héritage prouvé → refus de droits, zéro `started`, zéro processus CLI ; (3) parent workspace-write attesté → cible GLM → `provider_confinement_unavailable` si aucun confinement équivalent attestable (correction 2) : refus nommé, jamais un repli silencieux ni un fallback lecteur. |

<a id="extrait-11"></a>

## Extrait 11

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-16 | Projection lineage : parent, nested, status, résultat, journal (oracle corrigé, correction 1) | T3 serveur (Effect, tests vitest) | Pour chaque tâche native : exactement un fil virtuel persisté, identifiant `thread:bridget-task:<task_id>` déterministe, relation `subagent`, origine `bridget_native`, marqueur `bridgetTaskRef` complet (`version`, `taskId`, `rootThreadId`, `parentTaskId`, `generation`, `seq`, `status`). Négatifs : zéro conversation de premier niveau ; zéro provider session, turn, outbox ou lancement ; `activeTurnId` null ; composeur en lecture seule ; le serveur refuse `message.dispatch`, reprise fournisseur, changement fournisseur, fork et rollback sur ces fils ; le résultat n'est jamais re-routé au parent par T3. |
| S149-17 | Projection sans double exécution | T3 + fixture Rust | 100 lectures de projection : `started`/`prompt` restent à 1 par tâche ; aucune remise dupliquée au parent. S'appuie sur la pureté garantie par S149-05. |
| S149-18 | Panne T3 et dégradation | T3 serveur | CLI Bridget absent, exit 3 ou timeout → refus mappé (`unavailable`, `timeout`), aucun crash, aucun contenu masqué présenté comme valide. Le chemin natif Rust reste vert (S149-12). |
| S149-19 | Invisibilité inter-conversations | T3 + fixture | La conversation B (même processus fournisseur) ne voit ni la lineage ni le journal de A. Clé environment/project/conversation respectée. |

<a id="extrait-12"></a>

## Extrait 12

Fichiers cibles proposés : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader.test.ts` (extensions), un fichier dédié `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetLineage.test.ts` si la surface lineage introduit un contrat propre. Contrats dans `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/`.

<a id="extrait-13"></a>

## Extrait 13

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-22 | Filtre du pont sur le marqueur `bridgetTaskRef` | T3 + contrat de résumé Rust | Le connecteur Rust lit le résumé T3 : marqueur présent et valide → fil exclu de tout montage wrapper/presence/binding Bridget avant montage. Marqueur présent mais malformé (champ manquant, UUID non canonique, types faux, `seq` hors entier sûr, taille hors borne) → refus fermé, sans repli en fil ordinaire. Un simple préfixe d'identifiant ne vaut jamais autorité. |
| S149-23 | Anciens fils ordinaires intacts | T3 | Les données historiques sans `bridgetTaskRef` gardent leur comportement exact : aucun fil converti, aucune exclusion, aucune réinterprétation (invariant 12 du plan). |
| S149-24 | Mappage inter-fournisseurs figé et compatibilité 148 | Unitaire daemon | (1) Codex workspaceWrite avec confinement OS → enfant Codex workspace-write exact (roots/réseau/tmp conservés) ; (2) même parent → cible GLM → `provider_confinement_unavailable` tant qu'aucun confinement équivalent n'est attesté, jamais de repli silencieux ; (3) dangerFullAccess → mission GLM locale en bypass, réseau non réduit silencieusement, aucun réglage global modifié ; (4) readOnly → discovery/plan restreint, réseau exigé représentable sinon `permission_mapping_unavailable` ; (5) politique inconnue ou flags non représentés → `permission_attestation_unavailable` ou `permission_mapping_unavailable`, aucun fallback lecteur/bypass/grant ; (6) tâche 148 managée sans snapshot 149 : exécution historique conservée, toute nouvelle écriture 149 refusée nommée sans demande de grant de remplacement. |
| S149-25 | Deux conversations, même processus fournisseur, droits différents | T3 serveur + fixture | Conversation A full-access, conversation B readonly, même processus. Aucun emprunt : l'admission de B n'utilise jamais le fait de politique de A. Le fait est lié à environment/project/conversation, session/instance, run et credential. Révocation de A ne change pas B (étend l'interop R4). |
| S149-26 | Péremption du fait de politique | T3 serveur + fixture | Fin, échec ou close du run retire le fait du run propriétaire seul ; le nettoyage d'un ancien tour ne retire pas le fait du tour courant. Rotation ou retrait de la session MCP efface le fait. `revision` augmente quand le fait courant change. Un mode Claude qui change (par exemple vers plan) produit une nouvelle revision pour les futures admissions ; le snapshot déjà admis reste inchangé (invariant 5). |
| S149-27 | Séquence sûre pour JavaScript et pagination avec mutations en staging | Unitaire daemon + T3 | `seq` reste un entier 0..9007199254740991, non décroissant ; débordement → renouvellement de génération dans la transaction et resync imposé, aucun overflow silencieux. Première page fixe S ; chaque page vérifie même génération et séquence ; une mutation entre pages rend `snapshot_changed`, le staging est abandonné et la pagination reprend à la première page ; T3 publie toutes les pages en une transaction, aucun snapshot partiel visible. Watch : `ready` seq 0 toujours premier et non coalescible ; les mutations d'autres roots ne signalent pas ; rejeu, lecture, refus et ACK restent muets ; un changement de journal passe par son propre relai et ne devient pas une mutation de tâche. |

<a id="extrait-14"></a>

## Extrait 14

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-28 | Recette UI web de présentation ; remplace la recette desktop prévue (T040) | Daemon Bridget fixture et serveur T3 fixture sur port éphémère avec base privée (gabarit interop R4) ; interface web ouverte dans un navigateur ou la preview T3 | Lineage affiche la relation parent/enfant avec statut puis résultat ; le journal réel de l'enfant se lit enfant actif puis terminal ; nested, statuts natifs et modèle réels affichés ; aucune conversation T3 de premier niveau n'existe ; l'arrêt appelle la saga native (zéro provider session, zéro tour) ; la reconnexion reconstruit la projection depuis le store natif sans doublon. Limites : cette preuve ne valide pas la coque desktop native ; aucune seconde application T3 n'est lancée ; aucun PID de production n'est touché ; preuve dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipe149.md`. |

<a id="extrait-15"></a>

## Extrait 15

| ID | Scénario | Niveau | Oracle |
|---|---|---|---|
| S149-32 | Union d'introspection v1/v2 (G-P-07) | Unitaire daemon + T3 serveur | Branches (a), (b), (c) de G-P-07, mot à mot. (a) Un credential sans aucun fait de permissions jamais existé reçoit l'enveloppe148 v1, identité seule, quatre champs camelCase, parseur fermé. (b) Pour un credential dont le fait de permissions a été retiré (fin, échec ou fermeture du tour), révoqué ou tourné, la fonction `bridget_session` de T3 rend le refus nommé `permission_attestation_unavailable`, jamais l'enveloppe v1. La façade MCP privée de Bridget traite ce refus comme un échec de preuve de session. Tout appel de ce credential est fermé avant effet avec `t3_session_unavailable` : `bridget_delegate` (nouvelle requête et rejeu d'une requête admise), `bridget_task_status`, `bridget_task_cancel` et `bridget_who`. Il n'y a ni repli v1, ni repli discovery, ni repli par PID. Aucun lancement n'a lieu. « Lecture et rejeu inchangés » s'entend ainsi : (i) la lecture humaine Lineage native (gardes 147), l'annulation humaine native et la saga native continuent, car elles ne dépendent pas du credential T3 ; (ii) un credential neuf qui n'a jamais eu de fait reçoit l'enveloppe v1 : identité, status, cancel et rejeu 148 d'une requête déjà admise restent possibles, sans nouvelle exécution. Preuves existantes du cas, sur binaire r5 : `recovery149.md` R8.4 et `interop149.md` S5.1/S5.1b ; elles ne valent pas exécution de S149-32. (c) Une admission 149 inherit/development présentée avec la seule enveloppe v1 rend `permission_attestation_unavailable`, sans grant demandé, sans fallback discovery, sans spawn ; status, cancel et identité148 continuent de fonctionner. Les tests T3 prouvent la sémantique de réponse ; ils ne prouvent pas l'entrée du Rust autonome (voir §5 et §11.10). Mapper FR007, FR008, FR019 / SC003. |
| S149-33 | Observer propriétaire du chemin PTY (G-P-08) | Fixture wrapper + recette réelle | Branches (a), (b), (c) de G-P-08 en fixture. (a) Un lancement avec l'overlay `--settings` publie un fait portant permission_mode, session_id, cwd, prompt_id et l'identifiant de la demande Bridget ; le daemon accuse réception avant que le hook laisse continuer l'appel observé ; la demande délégataire correspond à l'identifiant observé (request_id exact). (b) Preuve absente, non corrélée, overlay altéré (tamper), non lié au lancement (unbound) ou observer indisponible : refus nommé, aucun lancement (compteur launched inchangé), aucun effet fournisseur, aucun grant. (c) Un fait d'une autre demande ne donne aucun droit ; l'overlay, son nonce, la socket et la preuve n'apparaissent ni dans le snapshot enfant, ni dans la définition ou l'environnement enfant, ni dans le journal ; fichier et socket 0600 supprimés en sortie ; la reconnexion ne récupère aucun fait d'une ancienne connexion. Branche (d) en recette réelle S149-14(a)/T038 : le CLI réel compose le fichier `--settings` avec les sources sélectionnées — les règles allow/deny des sources attestées restent effectives avec l'overlay — et le hook s'applique en PTY autonome sans approbation humaine ; le mode observé réel (manual normalisé default, plan, bypass) vient de la preuve d'observation sur le canal de l'owner, jamais d'une autodéclaration MCP ; un CLI qui remplace les sources au lieu de composer l'overlay est refusé. La preuve réelle reste le premier parent externe PTY avec dans un environnement natif isolé, jamais un descendant managed substitué. Mapper FR007, FR008, FR019 / SC003, SC004. |

<a id="extrait-16"></a>

## Extrait 16

| Niveau | Prouve | Ne prouve pas |
|---|---|---|
| Unitaire Rust | Règles de permission, mappage et refus nommés, révocation, bornes, pureté des lectures, séquence. | Aucun comportement de processus réel. Aucune attestation réelle. |
| E2E fixture | Saga complète : idempotence, annulation, redémarrage, lineage, admission logique de portée, absence de T3. Attestations parents = signatures fournies par fixture. | Qu'un modèle réel a tourné ; le choix effectif du fournisseur ; l'auth réelle ; toute voie standalone réelle (correction 3). |
| Fixtures G-P (S149-29, S149-30) | Refus nommés corrélés, compteurs de lancement inchangés, recontrôle des digests launcher et CLI résolu. | Tout comportement du fournisseur réel ; aucun confinement OS ; la trame `can_use_tool` réelle reste optionnelle (limite consignée). |
| Fixtures G-P r3 (S149-32, S149-33) | Sémantique de l'union v1/v2 (tombstone et admission 149 en v1 seule rendent le refus nommé) et corrélations de l'observer (ACK avant appel, request_id lié, suppression lifecycle, overlay absent de l'enfant). | L'entrée du Rust autonome : la voie standalone reste prouvée par la recette S149-14(a)/T038 seulement — premier parent externe PTY avec dans un environnement natif isolé, jamais un descendant managed substitué ; la composition réelle `--settings` n'est prouvée que par cette recette. |
| E2E réel opt-in | Enfant GLM réel `glm-5.3-flash`, modèle effectif annoncé par le fournisseur, écriture réelle dans le périmètre, corrélation unique, nettoyage. Parent GLM réel même famille (voie PTY dans un terminal ; voie T3 si la source v2 est attestable). | Le sandbox au-delà du relay ; les sessions déjà chargées ; le mapping workspaceWrite→GLM au-delà du refus nommé observé. |
| Interop T3 R4 | Serveur Node réel, auth registre, headers, révocation par conversation. | Un modèle réel ; les projections UI complètes (fixtures). |

<a id="extrait-17"></a>

## Extrait 17

T3 — depuis `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` :

<a id="extrait-18"></a>

## Extrait 18

L'interop HTTP réelle (type R4) exige le CLI Bridget construit localement et un serveur T3 fixture sur port éphémère. Elle ne touche aucun service de production.

<a id="extrait-19"></a>

## Extrait 19

- Machine : le serveur de travail, macOS arm64. Binaires requis : `cargo`, `python3`, `git`, `pnpm`/`vp`. Aucune installation nouvelle.
- Chaque fixture crée `/tmp/ng149-<uuid10>` (canonisé) avec `state/`, `provider/`, `tmp/`, `project/`, tous en 0o700 ; fichiers privés 0o600. Nettoyage en fin de test, conservé en cas d'échec pour diagnostic.
- Daemon de fixture : binaire du worktree, socket Unix privé, `TMPDIR` redirigé, stderr vers log privé. Aucun contact avec les PID de production (58468/57109/58394/58396 restent hors de portée).
- Grant humain : uniquement pour la compatibilité 148 rejouée et le re-grant explicite après révocation (S149-03, S149-20), via `openpty` + `delegate-grant` sur la fixture. Aucun grant sur le daemon réel. Aucun grant dans un chemin positif 149 (correction 4).
- Attestation parent dans un terminal (correction 3) : la voie supportée utilise la source propriétaire du parent réel (observation du wrapper PTY, liée session/cwd). La signature parent-managée 148 n'est jamais opposée comme preuve externe. La voie T3 exige un fait v2 provenant du tour réel ; si la source n'est pas attestable, le cas reste en fixture et la limite est consignée.
- Auth GLM réelle : la route existante de l'utilisateur est réutilisée en mémoire. Deux voies admises : `settings.json` du profil existant (clé `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN` ou `apiKeyHelper`), ou wrapper `gclaude` attesté par SHA256 avec le CLI `claude` à côté. L'opt-in `PASS_ZAI_API_KEY` transmet la variable préexistante en mémoire au seul daemon fixture. Aucune valeur de clé n'est lue, écrite, sérialisée ou journalisée par un test. Registre privé 0o600 requis. Hash de settings, commande, launcher et CLI résolu re-vérifiés en fin de recette.
- Registre source réel (G-P-05, consigné le 2026-10-10) : le service launchctl actif (PID58394) tourne avec `BRIDGET_HOME=/Users/moi/.cache/bridget-core`. Le registre vivant est `/Users/moi/.cache/bridget-core/agents.json` : entrée `glm` = `gclaude` + profil `/Users/moi/.claude-glm`, modèle `glm-5.3` ; entrée `claude` présente. Le fichier `/Users/moi/.config/bridget/agents.json` (`cursor`, `claude`) n'est pas sélectionné par ce runtime. Le registre de recette est un clone privé contrôlé (0o600) avec le modèle exact `glm-5.3-flash` ; la différence entre le compte privé de recette et le modèle de production n'autorise aucun repli hors Flash. Aucune entrée de production n'est modifiée ; les lectures de preuve restent des lectures JSON saines, en lecture seule, sans impression d'env ni de jeton.
- Modèle (correction 8) : l'identifiant exigé est exactement `glm-5.3-flash`. Le catalogue T3 vivant expose ce modèle exact ; la fixture relay locale est mise à jour pour l'attester, sans aucune config de production. Les événements de modèle effectif de la recette réelle doivent annoncer `glm-5.3-flash` ; toute autre valeur, y compris un repli `glm-5.3`, fait échouer le test. La recette refuse toute substitution ; si le profil n'annonce pas « Flash », on n'invente aucun modèle.
- Nettoyage de processus : `SIGTERM` après vérification PID/PPID/exécutable, jamais `kill -9`, jamais de cible Firefox (règle projet). Les watchdogs existants (`OwnedChild`) restent le seul mécanisme.

<a id="extrait-20"></a>

## Extrait 20

- Aucun redémarrage de T3, du daemon Bridget ou d'un service de production.
- Aucune écriture dans une configuration, un profil ou une base de production, y compris la config de modèle de production.
- Aucun nouveau framework de test, aucun Python/pytest : le projet est Rust côté Bridget et Effect/vitest côté T3. La recommandation Rust-seule suit la convention des sessions 142–148.
- Aucune exécution de test ou de build sur la version inchangée, hors état de départ documenté en section 6. Aucun re-run après PASS sans nouveau code.
- Aucun grant pty dans un chemin positif 149.
- Aucun commit par le sous-agent.

<a id="extrait-21"></a>

## Extrait 21

1. **Ronde documentaire.** Aucun test écrit, aucun code créé, aucune exécution. La version du worktree reste inchangée (base 148 `6807c22b`). Les oracles portent sur des comportements cibles définis par les contrats, pas sur du code existant.
2. **Contrats non figés.** `permissions.md` reste en cours. Les codes de refus, le schéma exact de `bridget_session` v2 et la forme du marqueur peuvent encore bouger ; les oracles suivront le contrat final avant la gate.
3. **Codes de refus en avance sur le code.** Les tests visent les codes contrat 149 ; le code actuel rend encore `development_grant_required`. L'écart est assumé et suivi en §9.5.
4. **Voie standalone réelle non prouvée à ce stade.** Les fixtures attestent les parents par signature ; seule la recette S149-14 variante (a) prouve la voie PTY dans un terminal, et elle attend le GO identité. La voie T3 pour un parent GLM reste conditionnelle à une source v2 attestable (S149-14 variante (b)).
5. **Confinement non affirmé.** Aucun test n'affirme un sandbox OS que le fournisseur n'atteste pas. Le cas workspaceWrite→GLM est prouvé comme refus nommé, pas comme écriture confinée.
6. **Projections T3 de l'interop R4 restent des fixtures.** Aucun processus modèle n'y est lancé ; la limite est héritée de 148 et reste vraie en 149.
7. **Localisation du mécanisme d'opt-out MCP à confirmer.** S149-21 dépend du mécanisme 147 existant ; son point d'accros exact sera repéré à l'implémentation.
8. **Nom de recette et préfixes d'environnement à fixer à l'implémentation.** `BRIDGET_149_*` est une recommandation ; le code actuel utilise `BRIDGET_148_*`.
9. **Amendement G-P r1.** S149-29 à S149-31 sont des oracles documentaires alignés sur le contrat corrigé par l'owner natif (`contracts/permissions.md`) ; aucun n'est exécuté à cette ronde.
10. **Amendement G-P r3.** S149-32 et S149-33 sont des oracles documentaires ; aucun n'est exécuté à cette ronde. Les tests T3/fixture prouvent la sémantique des refus et des corrélations ; l'entrée du Rust autonome reste prouvée par la recette S149-14(a)/T038 (dans un environnement natif isolé, premier parent externe PTY, jamais un descendant managed substitué), conformément au contrat (`contracts/permissions.md`, « Sources standalone » et « Compatibilité et gate ») et à la revue r3 §5.
