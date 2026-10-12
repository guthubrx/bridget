# Plan - Session155 remontée des demandes de l'enfant

État courant : implémentation 100 %, 42/42. Pipeline In Progress ; Converge P3,
audit final, revue R2 et livraison en attente. T040 est validée par le principal.

**Spec** : `specs/155-demandes-enfant/spec.md`
**Branche** : `session-155-demandes-enfant`
**Worktree** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant` (exécution séquentielle après 150-154)
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
| commandExecution kind=command avec offre explicite et effets établis ; fileChange/permissions refusées en v1 | FileChange Params incomplets, grantRoot session incertain ; permissions est un octroi hors périmètre | Inventer des offres ou un cache de corrélation supplémentaire |
| Seules décisions `accept` (une fois) / `decline` / `cancel`, repli fermé | Pas de droit durable | Première décision autre que `accept` |
| Politique de l'enfant inchangée | Évite le relecteur automatique concurrent | `never` vers `on-request` |
| Décision et validité de remise séparées | Un accord tardif donnerait un droit | Un seul champ `decided` |
| Clé de demande liée à l'instance, la session et le tour | Les identifiants JSON-RPC se répètent | Clé `(task, id fournisseur)` |
| Lecture des items par identifiant exact | Pas de famine (200 anciens items) | Relève par préfixe |
| Table propre `native_child_requests` | Issue, trace et reprise durables | Aucun journal |
| Pas de nouvel état de tâche, pas de nouveau type de boîte | Liste fermée 149 ; `CHECK` de la boîte | `waiting_for_input`, dixième type |
| Un fichier neuf `daemon/native_child_requests.rs` | Responsabilités cohérentes de réception, arbitrage, livraison, expiration et reliaison | Tout dans `native_delegation.rs` |
| Pas de `publish_approval_wait` | Coupe l'exécution après 3 demandes | Réutiliser tel quel |

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | `Delegate.child_requests`, `NativeDelegationRequest::Respond`, cinq trames `ChildRequest` / `ChildRequestAnswered` / `ChildRequestAnswer` / `ChildRequestCancel` / `ChildRequestClosed`, types de demande, verdict, décision |
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
| `crates/bridget-daemon/src/wrapper.rs` | événement vers trame, exécution `waiting_*`, application liée à session et tour, accusé, `BRIDGET_SAFE_MCP_TOOLS` 24 vers 25, wait/ack/continue conservés |
| tests des mêmes modules, `crates/bridget-daemon/tests/native_delegation_e2e.rs`, `crates/bridget-daemon/tests/claude_native_permissions_test.rs` | tests `native155_*` et listes d'outils |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | option, outil, refus |
| `docs/decisions/155-remontee-des-demandes-de-l-enfant.md` | ADR (change la décision 149) |

## Étapes d'implémentation

0. Correctif de sécurité du repli `acceptForSession`, seul (T000), exception indépendante.
1. Référence de non-régression et formes Codex par méthode (T001, T002).
2. Types de protocole et de session, magasin (T003 à T005).
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
| VIII Forensic | OK | T002 lit les formes réelles avant le code du reste de155, hors T000 ; tests rouges d'abord |
| XV Test-before-next | Adapté | pas de commit automatique ; chaque tâche testée avant d'être cochée |
| XVI Worktree | OK | worktree155 dédié, exécution séquentielle |
| XVIII Complexité | OK | voir ci-dessous |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Complexité (Article XVIII)

- Demandes ouvertes bornées : 4 par enfant, 128 tâches actives, donc au plus 512.
  Le tick utilise `child_request_followups` pour les états ouverts, les remises
  `pending/sent` et les clôtures `none/void` non confirmées. Le prédicat OR parcourt
  l’index existant ; son coût documenté est O(n + B), n étant les entrées parcourues
  et B les données projetées. `LIMIT 512` borne les lignes retournées, pas le
  travail du scan, qui peut parcourir l’index entier.
- Lecture des items humains : un `get` par ligne non réglée, O(k), k ≤ 512, aucun
  balayage de la file.
- Table des attentes du transport : au plus 4 entrées, O(1) par clé.
- Mise en forme des chemins cités : O(c), c ≤ 64.
- `classify_request` : O(c).

### Minimalisme (Article XIX)

- Un seul module neuf, justifié par les responsabilités de réception, arbitrage, livraison, expiration et reliaison.
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
- Schéma 0.162.1 lu : les formes par méthode sont établies et les formes non sûres refusées ; T002 est final PASS, reçu validation/gate-schema155.json ; protocole/store prouvés par les tests ciblés ; recette utilisateur T040 ouverte. T016 et T034 ont leurs preuves ciblées. La recette utilisateur T040 et la validation runtime générale restent distinctes ; aucun accord après interruption n’est déduit d’une simple lecture de schéma.

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | approbation répondue en silence aujourd'hui ; élicitation déclinée ; réponse du parent absente |
| Table de verdicts | au moins 24 cas (T006) |
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
| Forme Codex différente de l’hypothèse | T002 avant tout code hors T000 ; mise à jour des artefacts |
| Collision avec 150 sur `native_delegation.rs` | exécution séquentielle ; fichier neuf pour la logique |
| Demandes nouvelles non intégrées à l’attente151 livrée | T034 réutilise bridget_task_wait et WaitEpoch ; intégration à tester |

## Livraison

Code sur disque dans le worktree, sans commit automatique ni relance de service. Diff
cumulé dans `validation/diff-155.patch`.

## Précisions de préparation après154 livrée

T000 est l’exception explicite aux gates T001/T002 : le correctif de sécurité
se livre seul avant le reste. Il modifie le point unique approval_response ;
aucune nouvelle fonction de sélection des décisions. Les gates T001/T002 sont acquises. Le parcours forward155 a 71 PASS ciblés
dans native155-postfix-r1 ; la recette utilisateur T040 reste ouverte.

La session151 est livrée. T034 réutilise bridget_task_wait et WaitEpoch ; sa
forme est connue. Les demandes sont intégrées et prouvées par les tests ciblés, sans nouvel outil. La liste
fermée du parent natif contient 24 outils (wrapper.rs:5031-5056) ; Respond porte
la liste à 25. delegation_mcp::tools contient 7 outils (65-78), futur 8, avec
wait/ack/continue conservés. Normalisation de chemins purement lexicale, citée
et non vérifiée ; aucune lecture ni résolution du système de fichiers.

## Reçu T000 transmis par root après la prélecture

T000 ciblée : 3 PASS, 0 FAIL, RC0, 3,79 s ; 207 sources stables. SHA256 du
transport : 27a2a44928ff3ecd39c260902f0e3f7580276a8f7496bc36b6e760b722162055.
Claude T000 R1 : APPROVE_WITH_CHANGES, sans blocage. Limite LOW acceptée pour
le correctif isolé : permission_summary affiche « décision absente » pour refused
(attach.rs:3739), sans accord de permission ; erreur JSON-RPC -32000 correcte.
L’offre absente doit être refusée selon T000 ; la compatibilité fournisseur réelle
reste à établir en T002. Aucun succès live ni du reste de155 n’est déduit.
Tâches figées non cochées ; le reçu ciblé ne vaut pas livraison de T000.

## Clarifications de la contre-revue plan R1 — 2026-10-12

Verdict transmis par root : APPROVE_WITH_CHANGES. Le schéma fournisseur
elicitation/approval/_meta était NON LU à la contre-revueR1 ; le passage T002 ci-dessous le remplace, avant T003. Formulaires
tiers vides, marqueurs approval et métadonnées d’effet non établi : refus fermé,
pas de routage Parent. T006 conserve sa table d’au moins 24 cas ; T014 porte
les scénarios déjà prévus. L’exception du formulaire vide Bridget reste distincte.

T005 sépare le CAS de décision (open et deadline_at > now) du claim de remise
(delivery_state = pending et FR027). Chaque CAS contrôle rowcount. Les scénarios
T023/T027 et la relecture T038 couvrent les courses prévues ; aucune nouvelle tâche.

T016/T021/T023 ferment toute attente fournisseur non confirmée par Cancel→Closed,
y compris none/pending. void invalide l’accord sans prouver la fermeture. Chaque
renvoi sent refait FR027 ; expiration et clôture passent avant redelivery. Les
replays terminaux rendent la réponse persistée sans nouvel accord invalidé.
recent_requests affiche delivery_state. Pendant la pause, une décision humaine
est acquittée sans exécution ; T027 reprend avec une nouvelle demande. T037
précise que never ne crée pas de route humaine que le fournisseur ne produit pas.

T034 conserve ready terminal et pending nonterminal. pending_requests est une
clé racine du résultat wait, uniquement pour Parent/open/nonexpirée/actionnable
avec task_id. Human reste dans status, hors liste et prédicat wait. timed_out est
faux si ready ou pending_requests est non vide, vrai seulement à l’échéance wait
avec les deux listes vides. Une question Parent déjà ouverte retourne immédiatement.
WaitEpoch notifie après commit, compteur lu avant état ; aucune mémoire de waiter
ni consommation. « Nouvellement ouverte » désigne le réveil, pas un filtre de niveau.

## Schéma fournisseur0.162.1 — arbitrage avant T003

0.161.0 est l’hypothèse historique ;0.162.1 est la version réellement générée
avec --experimental par root, initialize.experimentalApi=true. La source primaire
est /Users/moi/.cache/bridget-delivery155.um4xccv4/codex-schema155-0.162.1.
Le contrat codex-requests.md et son reçu schema155.md sont écrits par l’autre owner.
T001 est acquise : 180 PASS/0 FAIL, 207 sources stables ; T002 est final PASS et cochée par root, reçu validation/gate-schema155.json ; protocole/store prouvés par les tests ciblés ; recette utilisateur T040 ouverte. forward155 a 71 PASS ciblés dans native155-postfix-r1, sans GREEN workspace global.

fileChange est entièrement refusée en v1 :approval_method_unsupported ;
grantRoot non nul :duration_unsupported. Params ne porte ni paths ni
availableDecisions. Les notifications ItemStarted/Completed portent changes
(path,diff,kind), mais seulement des faits cités et une piste future ; aucun
cache/corrélation supplémentaire ni route Human réelle non prouvée. Les
classifiers synthétiques T006/T008 peuvent toujours tester des faits complets
sous dangerFullAccess→Human ; le transport réel ferme fileChange en amont.

permissions reçoit une erreur JSON-RPC -32000 child_request_refused, motif
approval_method_unsupported ; sa réponse réelle est permissions/scope(turn ou
session)/strictAutoReview, pas decision. commandExecution reste admissible sous
les limites si kind=command et offre explicite accept/decline/cancel.

user_input accepté :{answers:{questionId:{answers:[string]}} ; le mapping interne
answer[questionId]=[strings] est adapté, pas copié brut. Aucune forme decline/cancel
native :refus par erreur JSON-RPC child_request_refused. Elicitation classique
tout formulaire MCP tiers est refusé avec approval_method_unsupported ; user_input seul peut être Parent ; openai/form,
openaiForm, openai/userVerification, url, tiers vide et _meta non vide sont fermés.
L’exception historique vide Bridget reste distincte. resolved ne porte que
threadId/requestId, pas turnId : aucun champ de notification inventé.

Checkpoint historique du 2026-10-12 avant T040 : In Progress, 41/42. T041 documentaire est
acquise. T039 est validée
et cochée par le principal sur les preuves cumulées vérifiées. Seule T040
reste ouverte pour la validation explicite de l’utilisateur. Le refus réel
workspaceWrite est PASS ; la demande Human reste open, sans accord accepté.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/recette.md.

Checkpoint courant après T041 : In Progress, 41/42, T039 et T041 documentaire
acquises ; T040 seule ouverte. Les checkpoints historiques 40/41 restent
historiques et sont remplacés par cet état courant. T041 corrige uniquement
le paragraphe Journal du contrat, sans changement de code ou de tests.
Preuve et lectures ciblées :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/implementation.md,
section T041 (transport codex_app_server.rs:165–178,4266–4310 ;
table delegation.rs:307–327).

## Checkpoint final avant Converge P3 — 42/42

Le principal a validé et coché T040 après l’accord explicite « Oui, accepter et
valider si les quatre cas réussissent ». Le reçu conserve cet accord à
2026-10-12T03:02:02.400752+00:00. Les quatre cas sont PASS. L’implémentation
est réalisée à 100 %, 42/42 ; le pipeline reste In Progress. Converge P2 est
historique avant les correctifs C1 ; P3, audit final, revue R2 et livraison
restent à faire.

Reçu actuel :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/recette155-postfix-final.json.
SHA256 : 153b70cbbbaa94315b87f192a8fa67fc217f607eb1cc70c5e9ea5b047b6cc9f3.
H1/H2/H3 et le permis d’émission MEDIUM ont leurs correctifs et preuves ciblées
fraîches. PERF-041 change seulement le commentaire de coût. PERF-082 et DUP-001
restent les deux LOW acceptés dans le périmètre. Cela ne vaut pas audit final.
La suite complète unique reste 2259 PASS / 15 FAIL / 72 ignored avant correctifs,
avec 180/180 baseline PASS et les sept nouveaux échecs ensuite rejoués PASS.
Aucune seconde suite complète n’a été lancée.
