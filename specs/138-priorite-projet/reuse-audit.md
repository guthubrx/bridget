# Audit de reutilisation de l'existant — 138-priorite-projet

## Decision

Statut: PASS
Date: 2026-10-06
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet

Conclusion courte: le plan étend les connexions, envois, fils et clients existants.
L'annonce projet et l'annuaire scoped sont deux contrats nouveaux nécessaires.
Aucun registre retiré n'est réactivé. Aucune table, dépendance ou service nouveau.
Une colonne de warnings dans le contrôle existant est l'exception justifiée.
Le PASS couvre la réutilisation de conception, pas les tests.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 18 |
| Items audites | 18 |
| Reutilisations deja prevues | 16 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 8 |

## Reutilisations correctement identifiees

Les lignes 1 et 3 identifient les deux nouveaux contrats. Les seize autres
étendent une responsabilité existante. Les numéros de ligne sont ceux de la base
du worktree avant implémentation ; les noms de fonction restent les repères.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/138-priorite-projet/reuse-audit.md#extrait-1).

## Existant potentiellement pertinent non mentionne

Complément vérifié avant code: les items 9/10 de façades couvrent aussi
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/handoff.rs:519
(HandoffTransport),
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/handoff.rs:546
(parse_request), et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/attach.rs:35
(JournalRequest). Le passage au transport Send est visible dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/attach.rs:3153.
Décision: étendre ces contrats existants pour porter le motif et le résultat
vers la garde commune daemon. T009/T010 incluent leurs tests. Aucun nouvel item
de service, registre ou abstraction n'est créé. Le complément contrôle porte
l'inventaire complet à 18/18.

Aucun après consolidation. ProjectReference/runtime est mentionné comme retiré,
pas comme dépendance active. derive_domain_at reste une étiquette. Les reçus102/136
et les mandats135 sont explicitement réutilisés.

## Duplications evidentes

Aucune non arbitrée. Le plan a retiré la colonne de consentement initialement
envisagée. Le motif par opération couvre le fil sans nouveau stockage.
Le contrôle de consigne requiert seulement la persistance de son warning dans
sa table existante. Ni le canon immuable ni refusal_reason ne sont détournés.
T020 est une correction de l'item13, pas une abstraction supplémentaire.
update_task, archive_previous_result et write_task_result restent les points
existants dans /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py:628,
:903 et :962. Le verrou commun protège publication/archive et chemin canonique.
L'inventaire reste 18/18 ; le PASS de conception ne remplace pas les preuves
de correctif. T020 est désormais clôturée sur 152 tests PASS exit0 confirmés
par le principal et revue corrective indépendante APPROVE, sans PASS global.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/AGENTS.md | Session, isolation et travail non fusionné | WT138 dédié ; sessions135/136 préservées ; aucune production |
| /Users/moi/.speckit/constitution.md, VII | Décision structurante tracée | ADR046 Proposé, numéro libre vérifié |
| /Users/moi/.speckit/constitution.md, XVIII | Complexité explicable | Envoi O(1), annuaire O(A), audience bornée O(M) |
| /Users/moi/.speckit/constitution.md, XIX/XX | Réutilisation et charge future | Garde commune, zéro dépendance/table/service supplémentaire |
| /Users/moi/.speckit/ref/standards-tests.md | Feature avant code, comportement observable | Gherkin138 puis tests Rust et unittest adaptés |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.specify/memory/constitution.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.specify/memory/standards.md | Pont vers source utilisateur | Fichiers absents dans WT ; ponts du principal lus, aucun contenu inventé |
| /Users/moi/.speckit/research/01-ai-agents-agentic-ai.md et /Users/moi/.speckit/research/03-cognitive-load-productivity.md et /Users/moi/.speckit/research/04-architectures-patterns.md | Baseline puis sources primaires | Recherche sourcée, décisions signalées comme déductions, aucune métrique de marché reprise |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 065 | Identité distincte du nom et chemin | Garder l'invariant sans réactiver son runtime retiré |
| 084 | Identité composée par source | Ne pas fusionner deux hôtes par homonymie |
| 094 | Façades et contrats cohérents | Une règle CLI/MCP/skill et refus structurés |
| 102 | Membres lecteurs, silence et reçus | Contrôle de tous les membres ; notify n'est pas confidentialité |
| 115 | Émetteur CLI attesté par filiation | Hériter de la connexion, jamais from déclaratif |
| 133 | Sous-agent rattaché et droits bornés | Même projet du parent ; aucun élargissement de droits |
| 135 | Missions, rôles et relances | Mandats extérieurs explicites conservés, pas de migration |
| 136 | Historique silencieux et canon legacy | Zéro rejeu historique, zéro perte, ancien canon conservé |

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/138-priorite-projet/reuse-audit.md#extrait-2).

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Registre projet | réutiliser preuves de transport/Git | Runtime retiré, aucun doublon admis | 2026-10-06 |
| Annonce de fait | créer contrat séparé après Register | Compatibilité de Register et autorité de connexion | 2026-10-06 |
| Annuaire | créer scoped, conserver global diagnostic | Intentions distinctes, AgentInfo ancien préservé | 2026-10-06 |
| Consentement de fil | aucun nouveau stockage | Motif par opération, audience immuable, canon existant | 2026-10-06 |
| Réponse directe | réutiliser demande OPEN acceptée | Participants inversés attestés, pas table consentement | 2026-10-06 |
| Projet inconnu | compatibilité avertie | Pas de faux même-projet ni interdiction absolue | 2026-10-06 |
| Agent Loop | modifier worktree dotfiles isolé | Source réelle conservée ; skill vivante inchangée | 2026-10-06 |
| Contrôle de consigne | une colonne JSON project_warnings dans execution_control_commands | Exception à zéro colonne ; restituer le warning accepté sans réinjecter ni changer canon/refusal_reason ; migration compatible par init_schema/introspection existants | 2026-10-06 |
| Scénario de contrôle réel et restart | créer uniquement tests/spec138_project_test.rs, réutiliser support/idempotent.rs | Recherche préalable rapportée dans evidence/core.md : daemon.rs couvre socketpair sans processus ; spec102 Quartet reste réservé aux fils ; Client, DaemonProcess, spawn_daemon et ExecutionStore existent déjà. Fichier de scénario distinct SteerCurrent/restart ; aucun nouveau harnais ni abstraction, item15 étendu, inventaire18/18 | 2026-10-06 |

Preuves des composants réutilisés :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/support/idempotent.rs:248
(DaemonProcess), :368 (Client), :554 (spawn_daemon), et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/spec138_project_test.rs:7
(ExecutionStore existant). La recherche avant création est rapportée par le
propriétaire dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/core.md.
Cette entrée justifie le fichier, sans déclarer son scénario exécuté PASS.

## Arbitrage de qualité après extraction Agent Loop

Décision : REUTILISER la branche existing_bridget existante de cmd_dispatch.
La recherche préparatoire avait retrouvé la transaction/update_task et l'envoi
avant code. Le contrôle nominal complémentaire sur HEAD et les deux paquets,
consigné après extraction, ne trouve aucun helper homonyme. Son ordre réel
reste visible ; ce contrôle tardif n'est pas présenté comme une preuve antérieure.

Trois responsabilités concrètes sont extraites dans
/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py :

| Responsabilité | Point exact | Existant réutilisé |
|---|---|---|
| Préparer cible et enveloppe | prepare_bridget_dispatch:1990 | Branche existante, résolution et champs durables ; pas de registre |
| Réserver avant effet | reserve_bridget_dispatch:2035 | update_task:628, même verrou/transaction et rollback T020 |
| Remettre l'enveloppe figée | dispatch_existing_bridget:2079 | send_bridget_message:1141, transport et résultat existants |

cmd_dispatch est maintenant :2120. Mesure radon rapportée et relue dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/loop.md :
complexité cyclomatique139 avant extraction,104 après,113 dans la base.
Les nouvelles fonctions mesurent23/10/6 ; chacune reste sous25. Ce sont des
chemins de décision du code, pas une mesure de latence. La complexité héritée
des autres backends n'est pas déclarée résolue. Les docstrings bornent les coûts
locaux sans promettre le réseau, l'attente du verrou ou la latence du stockage.
152 tests PASS après extraction confirmés par le principal ; py_compile et
diff --check PASS rapportés. Aucun framework, transport, service ou fichier neuf.
L'item13 est consolidé ; l'inventaire reste18/18 et les cinq gates de conception
restent cochés. T017/T018 et la validation globale restent ouverts.

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
