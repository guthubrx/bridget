# Plan - Session155 remontée des demandes de l'enfant

**Spec** : `specs/155-demandes-enfant/spec.md`
**Branche** : `session-155-demandes-enfant`
**Worktree** : `.worktrees/150-socle-delegation` (exécution séquentielle après 150-154)
**Date** : 2026-10-11 (révisé après contre-revue Codex, verdict BLOCKED, 11 objections,
voir `adversarial-review-codex.md`)

## Résumé

Cinq changements, sans modifier le contrat d'appel 149-150 quand l'option est absente.

1. Option `child_requests` (`deny` par défaut) figée dans la tâche. La politique de
   l'enfant n'est pas modifiée. Un parent avec relecteur automatique est refusé
   (FR-001 à FR-004).
2. Transport Codex : en mode `forward`, les demandes sont enregistrées et signalées, pas
   répondues. La réponse arrive plus tard, liée à la session et au tour (FR-005 à FR-008).
3. Daemon : table des demandes, verdict `Parent` / `Human` / `Forbidden` calculé sur les
   droits figés, refus de toute écriture de fichier sous un bac à sable confiné, délais,
   issues, validité de remise séparée de la décision et fermeture sérialisée
   (FR-009 à FR-013, FR-023 à FR-030).
4. Outil `bridget_task_respond` pour les questions, ajouté à la liste fermée des outils
   du parent natif, et lecture dans `bridget_task_status` (FR-014 à FR-020).
5. Approbations vers la boîte humaine existante, lues par identifiant exact (FR-021,
   FR-022).

## Décisions architecturales

| Décision | Pourquoi | Alternative rejetée |
|---|---|---|
| Option par tâche, défaut `deny` | Garde 149 intact (SC-005) | Défaut `forward` |
| Parent : questions seulement. Humain : approbations | Un enfant hérité a la politique du parent ; un parent modèle est injectable | Parent qui approuve (question 2) |
| Limite absolue = bac à sable figé du parent, humain compris | ADR 149 : aucun gain de droits | Accord humain sans limite |
| Verdict de route décidé par le daemon sur des faits structurés | Le texte vient d'un modèle | Route déclarée par l'enfant |
| Aucune écriture de fichier approuvée sous `workspaceWrite` | La course sur les liens ne se ferme pas sans preuve de confinement | Résolution de chemins (fenêtre résiduelle) |
| État `sent` écrit avant l'envoi, fermeture confirmée par le wrapper | Un accord envoyé survit à `void` | `void` côté daemon seul |
| Correctif T000 du repli de `approval_response`, livré seul | Défaut préexistant : `acceptForSession` possible en mode `deny` | Attendre la fin de 155 |
| Seules `commandExecution` et `fileChange` transmises | Durée et effets des autres inconnus | Relayer `permissions` |
| Seules décisions `accept` (une fois) / `decline` / `cancel`, repli fermé | Pas de droit durable | Première décision autre que `accept` |
| Politique de l'enfant inchangée | Évite le relecteur automatique concurrent | `never` vers `on-request` |
| Décision et validité de remise séparées | Un accord tardif donnerait un droit | Un seul champ `decided` |
| Clé de demande liée à l'instance, la session et le tour | Les identifiants JSON-RPC se répètent | Clé `(task, id fournisseur)` |
| Lecture des items par identifiant exact | Pas de famine (200 anciens items) | Relève par préfixe |
| Table propre `native_child_requests` | Issue, trace et reprise durables | Aucun journal |
| Pas de nouvel état de tâche, pas de nouveau type de boîte | Liste fermée 149 ; `CHECK` de la boîte | `waiting_for_input`, dixième type |
| Un fichier neuf `daemon/native_child_requests.rs` | `native_delegation.rs` fait 2714 lignes | Tout dans `native_delegation.rs` |
| Pas de `publish_approval_wait` | Coupe l'exécution après 3 demandes | Réutiliser tel quel |

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | `Delegate.child_requests`, `NativeDelegationRequest::Respond`, trames `ChildRequest` / `ChildRequestAnswered` / `ChildRequestAnswer`, types de demande, verdict, décision |
| `crates/bridget-transport/src/managed_session.rs` | `ManagedEventKind::ProviderRequest`, `ManagedSession::answer_provider_request` (défaut : refus) |
| `crates/bridget-transport/src/codex_app_server.rs` | mode `forward` : table des attentes, événement, réponse différée liée à session et tour, repli fermé, nettoyage ; le chemin `deny` ne change pas |
| `crates/bridget-daemon/src/native_permissions.rs` | `classify_request` (pur) et mise en forme des chemins cités |
| `crates/bridget-daemon/src/delegation.rs` | champ `child_requests`, table `native_child_requests`, API de magasin |
| `crates/bridget-daemon/src/daemon/native_child_requests.rs` (nouveau) | réception, rapprochement d'instance, verdict, budget, avis, envoi, expiration, validité de remise, items humains |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | admission, `Respond`, vue de statut, variable d'environnement au lancement, appel du tick |
| `crates/bridget-daemon/src/daemon.rs` | routage des trames, `daemon_deposit_human_inbox` rend son résultat, fermeture au redémarrage |
| `crates/bridget-daemon/src/delegation_mcp.rs`, `mcp.rs` | outil `bridget_task_respond`, schéma `child_requests` |
| `crates/bridget-daemon/src/human_inbox.rs` | recherche d'un item par clé exacte |
| `crates/bridget-daemon/src/delegation_lineage.rs` | suppression des lignes de demandes dans la transaction de purge (150) |
| `crates/bridget-daemon/src/wrapper.rs` | événement vers trame, exécution `waiting_*`, application liée à session et tour, accusé, `BRIDGET_SAFE_MCP_TOOLS` 20 vers 21 |
| tests des mêmes modules, `crates/bridget-daemon/tests/native_delegation_e2e.rs`, `crates/bridget-daemon/tests/claude_native_permissions_test.rs` | tests `native155_*` et listes d'outils |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | option, outil, refus |
| `docs/decisions/155-remontee-des-demandes-de-l-enfant.md` | ADR (change la décision 149) |

## Étapes d'implémentation

1. Référence de non-régression et formes Codex par méthode (T001, T002).
2. Types de protocole et de session, magasin (T003 à T005).
0. Correctif de sécurité du repli `acceptForSession`, seul (T000).
3. Limite absolue : verdicts, refus des écritures confinées, admission, lancement (T006 à T011).
4. Transport Codex : approbations, questions, wrapper (T012 à T016).
5. Daemon : réception, avis, boîte humaine, envoi, cycle de vie (T017 à T023).
6. Réponse du parent aux questions, outil, statut, pause (T024 à T027).
7. Route humaine par identifiant exact (T028, T029).
8. Boucles et texte hostile (T030 à T032).
9. Intégrations 150 et 151 (T033, T034).
10. Bout en bout, ADR, documentation, revue, suite complète, recette (T035 à T040).

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 155 (T036) change la décision 149 |
| VIII Forensic | OK | T002 lit les formes réelles avant le code ; tests rouges d'abord |
| XV Test-before-next | Adapté | pas de commit automatique ; chaque tâche testée avant d'être cochée |
| XVI Worktree | OK | worktree 150 partagé, exécution séquentielle |
| XVIII Complexité | OK | voir ci-dessous |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Complexité (Article XVIII)

- Demandes ouvertes bornées : 4 par enfant, 128 tâches actives, donc au plus 512. Le tick
  lit les ouvertes par index `(state, deadline_at)` : O(log n + k), k ≤ 512.
- Lecture des items humains : un `get` par ligne non réglée, O(k), k ≤ 512, aucun
  balayage de la file.
- Table des attentes du transport : au plus 4 entrées, O(1) par clé.
- Mise en forme des chemins cités : O(c), c ≤ 64.
- `classify_request` : O(c).

### Minimalisme (Article XIX)

- Un seul fichier neuf, justifié par la taille de `native_delegation.rs`.
- Aucun nouvel état de tâche, aucun nouveau type de boîte, aucune dépendance.
- Supprimés après la contre-revue : route d'approbation du parent, table de couverture
  pour le parent, changement de `approval_policy`, relève par préfixe, route humaine des
  élicitations en mode URL. Ces retraits diminuent le code et la surface d'attaque.
- Réutilisés : `send_as`, `daemon_deposit_human_inbox`, `deposit_and_notify`, `get`,
  `resolve` / `ack` / `close_self`, `admit_autonomous_effect`, `recover_owner`,
  `publish_execution_transition`, le patron `ControlExecutionDispatch`.

Potentiel minimalisme : aucune ligne supprimable connue ; `approval_response` reste
utile au mode `deny`.

### Responsabilité future (Article XX)

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- Non vérifié ici : champs d'effet et durée des octrois de chaque méthode Codex (T002),
  transition `waiting_approval` vers `running` (T016), forme de l'attente de la
  session 151 (T034), comportement de Codex quand un accord arrive après l'interruption
  d'un tour.

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | approbation répondue en silence aujourd'hui ; élicitation déclinée ; réponse du parent absente |
| Table de verdicts | au moins 20 cas (T006) |
| Écritures confinées | refus quel que soit le chemin (T008) |
| Correctif deny | offres sans `decline` ou sans `accept` (T000) |
| Transport | faux serveur Codex : attente, non-blocage, nettoyage, méthode `permissions`, repli fermé, mode `deny` inchangé |
| Daemon | liaison instance, session, tour ; doublon ; budget ; avis ; envoi ; accord puis annulation puis reconnexion |
| Boîte | dépôt idempotent, décision tardive, 250 décisions obsolètes, fermeture |
| Chaîne | trois niveaux avec de vrais parents natifs, liste d'outils |
| Texte hostile | usurpation, faux identifiants, commande cachée, caractères de direction |
| Non-régression | filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087`, `elicitation`, `approval_response` |
| Suite complète | recette `--workspace --no-fail-fast` avant clôture |

## Risques

| Risque | Parade |
|---|---|
| Un accord dépasse la limite du parent | verdict calculé sur faits structurés, humain compris ; refus par défaut ; recalcul avant envoi |
| Course sur un lien symbolique avant l'écriture | aucune écriture de fichier approuvée sous `workspaceWrite` ; chemins non décisifs |
| Méthode ou champ d'effet inconnu | liste blanche de T002 ; hors liste : `Forbidden` |
| Relecteur automatique concurrent | refus à l'admission si `approvals_reviewer` n'est pas `user` |
| Accord envoyé puis annulation | `sent` avant envoi, verrou du wrapper, fermeture confirmée, `unknown` sans accusé |
| Accord après l'échéance | échéance contrôlée à la décision et à chaque envoi |
| `acceptForSession` choisi par le repli actuel | T000, repli fermé dans les deux modes |
| Identifiant fournisseur réutilisé | clé liée à instance, session, tour ; empreinte du contenu |
| Famine de la boîte humaine | lecture par identifiant exact, abandon durable |
| Injection par le texte | pas d'approbation par un modèle ; faits d'abord ; texte cité et nettoyé |
| Parent natif sans l'outil | liste fermée mise à jour et testée |
| Forme Codex différente de l'hypothèse | T002 avant tout code ; mise à jour des artefacts |
| Collision avec 150 sur `native_delegation.rs` | exécution séquentielle ; fichier neuf pour la logique |
| Attente de la session 151 absente | T034 conditionnelle, signalée dans analyze-report.md |

## Livraison

Code sur disque dans le worktree, sans commit automatique ni relance de service. Diff
cumulé dans `validation/diff-155.patch`.
