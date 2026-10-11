# Audit de réemploi153 — code152 livré

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte

Passage frais du code présent dans WT153. Les ajouts153 sont absents, comme
attendu avant implémentation. Trois duplications proposées sont corrigées :
désérialiseur de présence, extraction de droits déjà faite152, copie de schéma
MCP. L'index de quota existant est étendu. Aucun arbitrage bloquant restant.

## Synthese

| Métrique | Valeur |
|---|---:|
| Items extraits et audités | 32 |
| Duplications évidentes encore ouvertes | 0 |
| Arbitrages nécessaires encore ouverts | 0 |
| Gates satisfaits | 5/5 |

## Reutilisations correctement identifiees

Chaque responsabilité proposée reçoit une décision et une preuve actuelle.
Une mention CRÉER signifie absent dans le code actuel et nécessaire au contrat ;
elle ne prétend pas que le code153 existe déjà.

| Item du plan | Décision | Preuve absolue actuelle | Application |
|---|---|---|---|
| Protocole Delegate | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-transport/src/protocol.rs:2011 | T002 ; champs optionnels omis |
| Variante Continue | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-transport/src/protocol.rs:2011 | T002 ; absente, une seule action prévue |
| Présence brief:null | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-transport/src/protocol.rs:2717 | T002/T005 ; deserialize_present_optional, JSON des deux variantes |
| Brief et rendu | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/handoff.rs:460 | T005/T006 ; validate_and_render, borne16384 |
| Sept champs de Task | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:49 | T003 ; defaults pour anciens payloads, aucun nouvel état |
| RoundRecord et rejeu | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:49 | T003/T004 ; sept entrées au plus |
| Méthodes de ronde/ID/place | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:49 | T003 ; round_no/result_message_id/holds_slot/continuable |
| Écriture begin_round | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:269 | T004 ; save_projected garde dates/ACK, transaction dédiée |
| Mission passée indexée | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:127 | T003/T028 ; aucune table équivalente, O(log n) |
| Réponse tardive | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1246 | T028 ; deux dispatch, identité pure, sans recover_child |
| Autorité et sources | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:995 | T014 ; native_launch_registry déjà commun |
| Politique actuelle complète | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:112 | T014 ; parent_fact et child_policy, aucun champ retiré |
| Voie legacy sans snapshot | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:995 | T014 ; conserver root_permission et authorize_cwd |
| Schéma de brief MCP | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/mcp.rs:2196 | T017 ; Value transmis aux deux appels adaptateurs |
| Façade tools/execute | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_mcp.rs:65 | T017 ; un outil neuf dans la façade, aucun alias |
| Catalogue et dispatch MCP | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/mcp.rs:618 | T017 ; natif6→7, global27→28 |
| Liste sûre | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/wrapper.rs:5035 | T017 ; liste commune23→24 |
| Quota128/16 | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:341 | T012 ; SQL indexable, places tenues jusqu’au nettoyage |
| Index state existant | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:127 | T003/T012 ; même nom, quatre clés, migration Immediate gardée |
| Purge4096 et arbre | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:193 | T012/T029 ; garde + unacked vides hors OR cancelled/ACK |
| Suppression atomique | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:223 | T029 ; missions passées supprimées dans même transaction |
| ACK lecture151 | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:264 | T004 ; reset acknowledged_at pour nouvelle ronde, distinct livraison |
| ACK livraison150 | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:2419 | T029 ; !delivery_acked OR anciennes rondes non accusées |
| Identifiants des résultats | RÉUTILISER/CENTRALISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:195 | T016 ; quatre consommateurs, boîte151 comprise |
| Vues et attente151 | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:615 | T018 ; ronde courante, aucune mutation dans les lectures |
| Pose/levée de garde | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1924 | T010/T011 ; tick seul pose, levée durable guard_released+idleNone |
| Annulation et nettoyage | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:2203 | T011 ; stop gardé, places jusqu’à confirmation |
| Pause autonome | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/referent_control.rs:330 | T015 ; garde existante, journal pause_overlap |
| Reprise152 | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1488 | T011 ; compteur2 vie entière, identité/CAS/refus stricts |
| Chaîne d’exécution152 | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/execution_store.rs:1024 | T001/T018 ; latest_in_chain, dates brutes conservées |
| Lineage et signal commit | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:269 | T004/T020 ; projection, wait_epoch.bump puis committed |
| Tests, recette et docs | RÉUTILISER/ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/tasks.md:18 | T005/T008/T013/T019/T021–T027 ; scénarios prévus, ADR153 nécessaire |

## Existant potentiellement pertinent

`native_child_env(task, prepared) -> Result<(), String>` existe depuis152 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1018. Aucun nouveau montage d'environnement. `sha2` existe :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/Cargo.toml:34.
`child_policy(parent, definition, cwd, source, posture)` existe :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/native_permissions.rs:206.
La posture demandée reste inchangée, None reste None, comparaison complète.

## Duplications evidentes

| Proposition initiale | Correction retenue | État |
|---|---|---|
| Désérialisation brief qui absorbe null | Réemploi deserialize_present_optional ; vraie preuve JSON T005 | clos |
| Nouveau recheck_task_authority | Réemploi native_launch_registry ; politique actuelle locale, gardes legacy préservées | clos |
| Schéma handoff recopié/helper partagé neuf | Paramètre Value aux deux adaptateurs existants | clos |
| Index de quota neuf ou scan terminal | Étendre native_delegations_state reconnu, même nom/préfixe | clos |

## Memoires et regles applicables

| Source lue | Règle | Application |
|---|---|---|
| /Users/moi/.speckit/constitution.md | XVIII/XIX/XX | Index réutilisés, aucune extraction inutile, preuves et limites |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/.specify/memory/constitution.md | Gates projet | Pont constitution contrôlé, sync RC0 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/.specify/memory/standards.md | Références applicables | Tests/code quality et couplage livrés relus |
| /Users/moi/.speckit/ref/standards-tests.md | RED avant production ; oracles réels | T005/T008/T013/T004 avant correction, baseline séparée |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/AGENTS.md | Isolation et preuves | Docs153 seulement, aucun service ni Git par l'owner |
| /Users/moi/.agents/skills/my-specify-all/SKILL.md | Auto-fix du pipeline autorisé | Corrections ciblées avant Analyze2 ; périmètre intégral conservé |

Mémoire projet MEMORY.md Bridget non trouvée au chemin attendu ; aucune règle
inventée depuis une mémoire absente.

## Specs livrees applicables

150 : quotas, purge par arbre, délais et ACK de livraison présents.
151 : attente pure, boîte de fin et accusé de lecture présents.
152 : journal de pause, reprise, compteurs et droits revalidés présents.
Les cartes et33 champs actuels sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/validation/couplage.md.
Référence compatible T001 :194/0/1, six filtres réels, aucun ajout153.

## Journal de recherche

Un passage de lecture ciblée du vrai code, complété par les couplages précis du
principal et la revue Sonnet153r1. Recherches rg : protocole/présence, champs Task,
begin_round/past_missions absents, admission_counts/index/purge, quatre
native-result, ACK/capture_reply, droits, pause/reprise et catalogues MCP.
Le prérequis officiel paths-only, SPECIFY_FEATURE=153-rondes-contexte, a donné
RC127 : .specify/scripts/bash/check-prerequisites.sh absent. Fallback manuel :
chemins réels, plan et dépendances présents, sans installation ni hook contourné.

## Arbitrages

- T026 — CRÉER JUSTIFIÉ le compte rendu d'implémentation prévu :
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/specs/153-rondes-contexte/implementation.md.
  Recherche par nom et responsabilité : fichier absent ; tests-153 et couplage
  portent les preuves partielles, sans matrice complète31FR/17SC/5US/29tâches.
  Le compte rendu agrège ces preuves et les relectures des propriétaires.
  Il reste pending jusqu'aux derniers correctifs, tests ciblés et capture finale.
  Aucun nouveau contrôle, scénario, helper ou outil public.
- T022/T023 — RÉUTILISER les trois documents de délégation existants ; CRÉER
  JUSTIFIÉ le seul ADR153 prévu. Recherche par nom (`153`, `ronde`,
  `delegation`) et responsabilité dans `docs/decisions` : les ADR150/151/152
  ne décrivent pas l'ouverture atomique de rondes et le brief dans leur mission.
  Aucun ADR153 équivalent. Le nouveau fichier prévu est
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/docs/decisions/153-rondes-et-contexte-delegation-native.md.
  D12 explicite uniquement I1/I5/I7 et T004 déjà présents dans le plan :
  transaction unique, heure fraîche, resets par ronde et signal après commit.
  Aucun nouveau comportement, outil, scénario ou dépendance.
- Les32 responsabilités recensées sont auditées ; créer uniquement les ajouts nommés
  nécessaires153 (sept champs, variante Continue, RoundRecord, transaction de
  ronde, missions passées et outil dans façade existante).
- begin_round remet uniquement les champs de ronde à zéro ; compteur de reprise2
  et checkpoint de lancement restent de vie entière.
- Lever une garde pose guard_released=true et idleNone ; refus terminal avant
  result_delivery_pending, enfant réellement attesté.
- Réponse tardive : identité pure, aucun recover_child ni sauvegarde.
- T028 — CRÉER JUSTIFIÉ une seule fixture `cfg(test)`,
  `native153_reponse_tardive_fixture`, dans le module `presence_tests` de
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon.rs.
  Recherche par nom et responsabilité faite avant création : aucune fixture D
  existante ne monte une Task et une mission passée par `begin_round`.
  Le montage existant `parent_en_cours_150` est privé au module de tests N :
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:8282.
  Il reste privé ; aucun accès de test public ou tunnel ne sera ajouté.
  Composer uniquement les primitives existantes `control_socket` à
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon.rs:20556,
  `state_with_registered_agent` à
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon.rs:20747,
  `negotiate_idempotent_client` à
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon.rs:23493,
  puis le vrai `begin_round` du magasin. Quatre usages explicitement prévus
  dans ce même module : `native153_ronde_reponse_tardive_chemin_idempotent`,
  `native153_ronde_reponse_tardive_chemin_ordinaire`,
  `native153_ronde_reponse_tardive_rejeu` et
  `native153_ronde_reponse_tardive_expediteur_inconnu`.
  Ils justifient une fixture commune et évitent environ150 lignes de montage
  dupliquées (estimation du propriétaire, pas mesure). Aucun scénario neuf,
  abstraction, module ou hook de production. Gate de création accepté par le
  principal avant montage. Après insertion, la fixture est à
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon.rs:31058 ;
  les quatre tests sont à31136,31173,31204,31247 dans ce même fichier,
  avec appels réels à31138,31175,31206,31249.
  Le premier RED compilé prouve trois refus d'oracle sur la remise tardive et
  un témoin tiers authentifié PASS ; aucun GREEN T028 ni cochage n'en découle.
- Revue Sonnet : F1–F8 retenues ou déjà closes ; F9 suppression facultative de
  champs rejetée, car garde SQL/CAS et contrat des sept champs nécessaires.
- Aucun assouplissement de comparaison de politique, aucun quota neuf,
  aucun nouveau scénario ou outil de confort.

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

Les29 tâches existaient avant ce gate. Cette passe satisfait le gate avant leur
exécution ; elle ne régénère ni n'ajoute de tâche.


## Trace de réemploi T003/T004 et montage150

Recherche par nom/responsabilité effectuée par le propriétaire stockage dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs.
RoundRecord, begin_round, table native_delegation_past_missions et lecture JOIN
n'ont pas d'équivalent préexistant. L'index native_delegations_state conserve
nom/préfixe state ; les trois tests de fondations sont GREEN dans la passe R1.
result_message_id_for_round centralise la règle courante et les ACK anciens,
sans clone de Task. save_projected/visible_changed sont réutilisés ; la seule
transaction spéciale begin_round doit vider dates et ACK de ronde (RED3 au R1).

Montage T005/T008 : parent_en_cours_150 existant, neuf appels historiques.
Paramètre de requête approuvé pour les cas prévus, sans nouveau helper ;
neuf anciennes enveloppes et tous leurs oracles conservés. La commande R1 et
ses empreintes sont dans validation/tests-153.md ; aucun GREEN T004/T005/T017
complet déclaré par cette trace.


Précision de réalisation T003 après R1 : reconnaître strictement les deux SQL
connus après trim des extrémités. Supprimer tous les espaces est incorrect,
car cela modifie aussi un littéral '$.sta te'. Cette erreur d'implémentation
ne modifie pas le contrat de réemploi/migration gardée. Le cas étranger existant
est renforcé ; T003 est rouverte jusqu'au GREEN corrigé. Aucun ajout de tâche.


Clôture corrigée T003/T004 : la passe ciblée suivante prouve trois fondations
dont '$.sta te' étranger conservé, et trois transactions. trim des extrémités
seul reconnaît deux SQL connus. begin_round réutilise projection/signaux avec
un commit unique et horloge juste avant écriture. Le helper parent_en_cours_150
a maintenant son paramètre de requête approuvé ; neuf usages historiques restent
raccordés à leurs mêmes enveloppes, sans helper neuf. T005/T007 ont leurs preuves
ciblées ; deux native149_catalogue restent en diagnostic strict (pas hérités).


## Arbitrage de raccordement FR-031 / T017

RÉUTILISER/ÉTENDRE l'observateur interactif149 existant, sans nouvelle primitive.
Recherche nom/responsabilité et source relues :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/native_permission_observer.rs:45
(matcher PreToolUse),74 (liste acceptée),82 (request_id),90 (fait corrélé).
Ajouter Continue au circuit Delegate est nécessaire à la politique actuelle
FR-031. Réutiliser les dix tests existants du fichier
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/native_permission_observer149_tests.rs.
Dix PASS ciblés ; intégration finale T017 encore ouverte. Aucune baisse des
contrôles de politique, source opaque, identité, nonce ou preuve de pair.

## Clôture de réalisation T026

RÉUTILISER can_own_task pur, WaitEpoch sous verrou et tick après admission,
sans primitive ni lookup SQL supplémentaire. holds_slot est cfg(test) :
neuf assertions/quatre tests, seule annotation17 octets, inverse byte-exact.
CRÉER JUSTIFIÉ implementation.md et validation/diff-153.patch, prévus par T026,
après recherche par nom/responsabilité. T017 fermé par schéma partagé, observer149,
catalogue et intégration Claude. Sécurité complète, identité/CAS et purge conservées.
