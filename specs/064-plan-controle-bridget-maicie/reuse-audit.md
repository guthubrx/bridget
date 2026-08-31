# Audit de réutilisation de l'existant - Plan de contrôle Bridget et Maicie

## Decision

Statut: PASS
Date: 2026-08-29
Feature dir: /home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie
Base auditée: e72a79b28f51d56548ce01a30c6a103005ce169d

Conclusion courte: Le plan s'appuie sur les machines d'état, contrats, magasins et projections déjà livrés. Les concepts nouveaux représentent le travail logique et l'exécution, qui ne sont pas couverts par la seule livraison idempotente. Les existants proches ont une décision de consolidation explicite et aucune duplication évidente ne reste sans arbitrage. Le rejeu après réalignement confirme ce verdict : le seul commit entrant concerne l'identité UI d'une remise et ne duplique ni contrat, ni stockage, ni machine d'état du plan.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 19 |
| Items audites | 19 |
| Reutilisations deja prevues | 14 |
| Existants potentiellement pertinents | 5 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 15 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Origine, intention et corrélation du message | `BridgetMessage` | `crates/bridget-core/src/message.rs:43` | Étendre l'enveloppe existante avec compatibilité descendante, sans second type de message public. |
| Session fournisseur neutre | `ManagedSession` et `ManagedEvent` | `crates/bridget-transport/src/managed_session.rs:55`, `crates/bridget-transport/src/managed_session.rs:157` | Étendre le seam provider existant avec capacités et identifiants attestés. |
| Pilotage du tour Codex | Files de messages/steering et commandes du client app-server | `crates/bridget-transport/src/codex_app_server.rs:79`, `crates/bridget-transport/src/codex_app_server.rs:824`, `crates/bridget-transport/src/codex_app_server.rs:855` | Finaliser SPEC-063 dans cet adaptateur, avec preuve sur `clientId`, avant généralisation. |
| Cursor via ACP | Définition `cursor`, sélection `AcpTransport` et implémentation `ManagedSession` | `crates/bridget-daemon/src/registry.rs:945`, `crates/bridget-daemon/src/wrapper.rs:3010`, `crates/bridget-transport/src/acp.rs:630` | Étendre le transport ACP commun et ses contrats. Ne pas créer d'adaptateur Cursor séparé. |
| Livraison durable | `SendDelivery`, rejeu exact et machine `RecordState` | `crates/bridget-daemon/src/idempotency.rs:57`, `crates/bridget-daemon/src/idempotency.rs:128` | Conserver la livraison comme tentative de transport d'une soumission logique. |
| File logique et admission | Routage wrapper et remise idempotente existante | `crates/bridget-daemon/src/wrapper.rs:1`, `crates/bridget-daemon/src/idempotency.rs:128` | Ajouter l'admission au-dessus de la livraison, sans réimplémenter son stockage ni son rejeu. |
| Graphe et propriété d'agents | `SpawnOrder`, `SpawnLease` et saga de parc | `crates/bridget-daemon/src/fleet.rs:157`, `crates/bridget-daemon/src/fleet.rs:168` | Ajouter parent, mandat et travail propriétaire dans la saga existante. |
| État désiré versionné | `DesiredFleet` et `DesiredEquipier` | `crates/bridget-daemon/src/desired_state.rs:13`, `crates/bridget-daemon/src/desired_state.rs:25` | Étendre la projection atomique du parc, sans second registre de flotte. |
| Négociation du contrat Bridget | `ClientHello`, `ClientWelcome` et capacités client | `crates/bridget-transport/src/protocol.rs:777`, `crates/bridget-transport/src/protocol.rs:1289` | Versionner les nouvelles commandes dans le protocole public existant. |
| Capacités d'adaptateur et de modèle | `AdapterCapabilities`, `ModelCapabilities`, préflight de lancement | `crates/bridget-transport/src/protocol.rs:1238`, `crates/bridget-daemon/src/lifecycle.rs:110` | Étendre ce vocabulaire avec les capacités runtime, sans registre parallèle. |
| Référence et projection Maicie | Domaine, magasin et projection UI Maicie | `plugins/maicie/src/domain.rs:2164`, `plugins/maicie/src/ui_projection.rs:20` | Ajouter des références opaques et une projection dérivée, sans transférer l'autorité d'exécution. |
| Fraîcheur des faits Bridget dans Maicie | `RuntimeObservation`, `RuntimeSubscription`, `Fresh/Gap/Ended/Unavailable` | `plugins/maicie/src/runtime.rs:33`, `plugins/maicie/src/runtime.rs:112` | Réutiliser les curseurs et états de fraîcheur existants. |
| Client Bridget consommé par Maicie | `BridgetClient` | `plugins/maicie/src/bridget_client.rs:496` | Étendre le client actuel. Toute extraction en client public partagé reste conditionnée à trois consommateurs. |
| Reprise textuelle | Génération de carte de reprise et injection wrapper | `crates/bridget-daemon/src/reprise.rs:1`, `crates/bridget-daemon/src/wrapper.rs:111` | Conserver comme repli déclaré quand reprise ou bifurcation natives sont absentes. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| `WorkSubmission` | `SendDelivery` | `crates/bridget-daemon/src/idempotency.rs:128` | Créer l'entité logique mais réutiliser strictement `SendDelivery` pour chaque tentative. Les cycles de vie sont distincts. |
| `Execution` | Événements et terminaux de `ManagedSession` | `crates/bridget-transport/src/managed_session.rs:84`, `crates/bridget-transport/src/managed_session.rs:146` | Construire l'état durable Bridget à partir de ces événements, sans seconde interface fournisseur. |
| `AgentLink` | `DesiredEquipier` et `SpawnLease` | `crates/bridget-daemon/src/desired_state.rs:25`, `crates/bridget-daemon/src/fleet.rs:168` | Étendre la saga et le schéma existants. Un module séparé n'est permis que si la machine d'état le justifie. |
| `ExecutionPolicy` | Quotas de parc et permissions résolues | `crates/bridget-daemon/src/fleet.rs:27`, `crates/bridget-transport/src/protocol.rs:1274` | Consolider ces faits dans une vue de politique appliquée. Ne pas créer de moteur de règles générique. |
| Projection de mission publique | `UiMissionProjectionV1` et continuité de revue | `plugins/maicie/src/ui_projection.rs:20`, `plugins/maicie/src/review_continuity.rs:92` | Faire évoluer une sortie versionnée produite par Maicie, puis retirer la lecture directe du crate privé dans Bridget. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucune | Aucune duplication défendable après les décisions de consolidation ci-dessus | N/A | Continuer à vérifier ce gate à chaque lot. |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/home/moi/.speckit/constitution.md` | Étendre l'existant, documenter frontières, observabilité, tests et responsabilité future | Lots additifs, aucun nouveau crate par défaut, gates explicites. |
| `/home/moi/.speckit/ref/agent-orchestration.md` | Autorité, mandat, corrélation et bornes doivent être explicites | `AgentLink`, intentions et politiques bornées portent ces faits. |
| `/home/moi/.speckit/ref/standards-tests.md` | Tests unitaires, intégration, contrat et acceptation doivent viser les frontières réelles | Fixtures provider validées par contrat, crashes injectés et scénarios SPEC-064. |
| `/home/moi/.speckit/ref/standards-observability.md` | Logs structurés, métriques cardinalité bornée et corrélation | Le plan sépare journaux, métriques et traces sans corps de message par défaut. |
| `/home/moi/.speckit/ref/code-quality-details.md` | Les abstractions doivent avoir plusieurs consommateurs réels et rester supprimables | Extraction du client public et nouveau crate sont différés jusqu'à preuve d'usage. |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-003 | Cycle de vie durable des demandes | Réutiliser états terminaux et conservation des issues. |
| SPEC-004 | Modèle runtime des agents | Étendre les faits runtime, ne pas créer une seconde identité d'agent. |
| SPEC-009 | Création et cycle de vie par le daemon | Étendre la saga de parc avec propriété. |
| SPEC-011 | Maicie compagnon d'orchestration | Maintenir la frontière mission/exécution. |
| SPEC-012 | Contrat public et rejeu idempotent | Étendre négociation, clés et exactitude du rejeu. |
| SPEC-014 | Observabilité initiale | Enrichir les projections au lieu de remplacer les journaux existants. |
| SPEC-015 | Guichet Maicie et projections fraîches | Réutiliser service négocié, curseurs et fraîcheur. |
| SPEC-016 | Événements de coordination | Corréler les faits d'exécution au flux existant. |
| SPEC-023 | États occupé et indéterminé | Remplacer progressivement le booléen par une projection détaillée compatible. |
| SPEC-034 | Macrographe, refus machine et réconciliation | Gouverner les engagements critiques, pas le raisonnement ni un DAG générique. |
| SPEC-046 | Connexion distincte de l'activité | Exposer wrapper, provider, tour et progrès séparément. |
| SPEC-048 | Identité du daemon | Conserver l'instance/génération dans les écritures et projections. |
| SPEC-049 | Mission indépendante du cache Bridget | La projection runtime ne devient jamais vérité métier. |
| SPEC-052 | Délégation corrélée et rejeu | Relier délégation et exécution par références opaques. |
| SPEC-063 | Steering et interruption | Finaliser la preuve `clientId` comme lot 0 obligatoire. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n "struct BridgetMessage" crates/bridget-core/src` | Message public | Enveloppe unique existante. |
| `rg -n "RecordState|SendDelivery|PublicResult" crates/bridget-daemon/src/idempotency.rs` | Idempotence | Machine durable et rejeu exact déjà présents. |
| `rg -n "ManagedSession|ManagedEventKind|ManagedTerminal" crates/bridget-transport/src/managed_session.rs` | Adaptateurs | Seam provider neutre déjà présent. |
| `rg -n "thread/start|turn/steer|turn/interrupt|clientId" crates/bridget-transport/src/codex_app_server.rs` | Codex | Start/steer/interrupt présents, reprise/bifurcation absentes, corrélation 063 incomplète. |
| `rg -n "native_cursor_definition|AcpTransport|impl ManagedSession for AcpTransport" crates` | Cursor/ACP | Cursor est un type fournisseur natif routé vers l'adaptateur ACP commun. Deux tests de registre ciblés passent. |
| `rg -n "SpawnOrder|SpawnLease|DesiredEquipier" crates/bridget-daemon/src` | Parc | Saga, lease et état désiré versionné déjà présents. |
| `rg -n "ClientHello|ClientWelcome|AdapterCapabilities" crates/bridget-transport/src/protocol.rs` | Contrats | Négociation et capacités existantes à étendre. |
| `rg -n "RuntimeObservation|RuntimeSubscription|UiMissionProjectionV1" plugins/maicie/src` | Maicie | Fraîcheur et projection versionnée déjà présentes. |
| `rg -n "maicie" crates/bridget-daemon/Cargo.toml crates/bridget-daemon/src/ui.rs` | Frontière | Dépendance privée directe confirmée à retirer après projection publique. |
| `rg -n "struct Metrics|get_metrics|increment_sent" crates/bridget-daemon/src/daemon.rs` | Observabilité | Compteurs process locaux existants, couverture d'exécution à construire. |
| `rg -n "reprise|carte de reprise" crates/bridget-daemon/src` | Continuité | Reprise textuelle existante à conserver comme fallback. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Soumission logique contre livraison | créer nouveau en réutilisant | Une soumission peut produire plusieurs livraisons et exécutions, tandis que `SendDelivery` garantit une tentative exacte. | 2026-08-29 |
| Capacités provider | réutiliser | `AdapterCapabilities` et la négociation publique constituent le vocabulaire canonique à étendre. | 2026-08-29 |
| Graphe d'agents | réutiliser | La saga de parc possède déjà réservation, lease, génération et rollback. | 2026-08-29 |
| Client Bridget partagé | approfondir | Extraire seulement quand daemon, Maicie et un troisième consommateur réel justifient l'abstraction. | 2026-08-29 |
| Moteur de politique ou workflow | ne pas créer | Les règles sont bornées aux transitions techniques critiques selon SPEC-034. | 2026-08-29 |
| Scénarios Gherkin 064 | créer par feature | Les fichiers `tests/features/019-*.feature`, `023-*.feature` et `025-*.feature` sont des oracles propres à leur SPEC. Le nouveau fichier 064 ne crée ni runner ni abstraction partagée. | 2026-08-29 |

## Rejeu après réalignement

- Rejoué le 2026-08-29 contre `e72a79b28f51d56548ce01a30c6a103005ce169d`.
- Le diff entrant `a832221..e72a79b` est limité à
  `crates/bridget-daemon/src/ui.rs` et
  `crates/bridget-daemon/assets/ui/app.js` : il unifie l'identité de la bulle
  optimiste avec celle du message durable.
- Les recherches par nom et responsabilité sur `BridgetMessage`,
  `SendDelivery`, `ManagedSession`, le pilotage Codex, `SpawnLease`,
  `AdapterCapabilities`, les projections Maicie, `Metrics` et `reprise`
  confirment les seams recensés ci-dessus.
- Aucun équivalent 1:1 de `WorkSubmission` ou `Execution` n'est apparu. Leur
  création reste bornée aux deux machines d'état réutilisées par core, daemon,
  transport, UI, Maicie et tests.
- Une correction technique SPEC-063 est confirmée : l'adaptateur compare encore
  `item/completed.userMessage.id`, alors que le contrat réel à prouver porte
  `item/started.userMessage.clientId`. Elle est couverte par T006-T008 et ne
  constitue pas une contradiction architecturale.
- Ce constat SPEC-063 est désormais clos : `item/started.userMessage.clientId` est implémenté et attesté dans `specs/063-interruption-pilotage-tour-humain/evidence/client-id-consumption.md`.
- Le fichier Gherkin 064 a été créé après recherche des scénarios existants, sans nouveau framework de test.

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
