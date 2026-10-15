# Interfaces SPEC168 — index de raccordement

## État final T037 — gel pour l’audit V15

Revue Sonnet R1 APPROVE_WITH_CHANGES sans bloquant : M1/L1/L2/L4 corrigés et vérifiés,
L3/R1 documentés, limites R2/R3/R4 conservées. Les quatre exacts ROOT passent chacun une fois.
Les 56 entrées source/doc sont inchangées pendant cette campagne ; les documents évoluent
uniquement ensuite pour consigner les preuves. ROOT a confirmé aucun Cargo actif.
RoutingScoreWeights est finalisé : défauts omis au canon, non défauts figés, digest historique préservé.
La garde source≤réception reste appliquée aux mesures ; les verdicts corrélés utilisent réception/TTL maître.
43 tâches sur 56 cochées ; T037 clôturée, T038 audit et T036 FULL ouvertes.
70 noms uniques à dernière issue PASS, 102 PASS bruts, 9 échecs historiques conservés.
P3 CONVERGED conservé, aucune P4. Aucune recette réelle 168, activation ou livraison déduite.
GEL final des documents pour initialisation V15. Les états et gels ci-dessous sont historiques.

## État de revue code R1 et gel documentaire

Revue Sonnet R1 : APPROVE_WITH_CHANGES, aucun bloquant établi ; corrections M1/L1/L2/L4
publiées par C, compléments de garde ressources et compatibilité canonique encore à finaliser avant tests.
E possède la représentation privée RoutingScoreWeights dans registry.rs ; C consomme ses sept
coefficients bornés. Pas de copie dans le wire ni de nouveau service. Les seuils ne sont pas des poids.
Les coefficients de la coordinatrice valent pour toutes les cibles ; seuils/TTL restent par cible.
weights égal au défaut est omis du JSON canonique, pour garder le digest des anciens snapshots168 ;
non défaut est sérialisé et figé. Ce raccord est publié sans preuve d’exécution anticipée.
M1 sépare verdict corrélé/TTL de réception maître et fraîcheur des ressources ; la garde
date source≤réception des mesures doit rester applicable. L2 conserve decision_capacity_occupied.
L3/R1 sont documentés : HEAD CLI mobile change le canon ; le choix durable ne migre jamais,
mais sa revalidation opérationnelle peut refuser son lancement.
Les quatre filtres ROOT prévus (registre telemetry, score natif, service valide, niveaux inaccessibles)
n’ont aucun résultat retenu ici. Tasks reste42/56, T037 ouverte.
GEL documentaire après publication de cet état pour export/audit. Les captures suivantes restent historiques.


État courant après P3 close : CONVERGED pour la réalisation168, aucun défaut résiduel confirmé.
42/56tâches cochées ; T036–T049 ouvertes, T037 non cochée avant contre-revue indépendante.
70noms cumulés à dernière issuePASS,98PASS bruts/9FAIL historiques inchangés ; aucune FULL/activation/recette/release déduite.
ProvenanceP3 : message final /root/verify_git_handoff et comparaison ROOT55fichiers avant/après inchangés ; lectures cumulativesP1/P2/P3, pas relecture intégraleP3.
Rapport : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/converge168.md.
Documents gelés après publication pour export Sonnet ; aucun changement sans nouveau GO ROOT. Les états antérieurs ci-dessous restent historiques.

État courant avant P3 : T055/T056 corrigées et prouvées par six replays exactsPASS, sources55entrées stables.
Tasks42/56cochées ; seules T036–T049 ouvertes, T037 non cochée.70noms uniques à dernière issuePASS, aucun nouveau nom.
C payload/snapshot et E DTO/projection ont conservé leur propriété exclusive ; aucune source de code modifiée ici.
Nouveau GEL documentaire P3 effectif après cette publication. Les états NONCONVERGED P1/P2 plus bas restent historiques.

État courant après P2 close : NON CONVERGED pour T055/T056 ;56tâches,40cochées,16ouvertes.
Les70dernières issuesPASS restent antérieures, sur leurs hashes ; aucune preuve des correctifsP2 à venir.
T055 : C constructeur/compactage crédits et E DTO/projection historique ; observations ProviderLimits conservées par connexion.
T056 : C mesures périmées→inconnues et E validation/projection Lineage compilation_slots0..=10 ; capacité déclarée conservée.
Les fichiers Lineage restent exclusivement E après transfertA→E. C ne les édite pas ; D/B conservent leurs frontières.
Le gelP2 est levé hors passe ; ROOT annoncera le prochain gel. Aucun nouveau banc/fournisseur ou changement d’exigence.
Les états et gels plus bas sont des captures historiques.

État courant avant P2 : correctifsP1 publiés et prouvés par33PASS ;70noms distincts cumulés à dernière issuePASS.
Tasks40/54cochées,14opérationnelles ouvertes (T036–T049). T037 non cochée ; verdictP2 non anticipé.
T050/T053/T054=C ; T052=D service+C appelant ; T051=E DTO/projection+C constructeur, transfertLineage A→E conservé.
Nouveau GEL documentaire avantP2 : aucune écriture après cette consolidation, hors appendTasks explicitement ordonné ROOT.
Les états P1/57noms/32sur54 conservés plus bas sont historiques.

État courant après clôture P1 : NON CONVERGED ;54tâches,32cochées après rattachementT025,22ouvertes. Les compteurs antérieurs ci-dessous sont historiques.
Rapport P1 : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/converge168.md.
T050/T053/T054=C /root/admission168_finish ; T052=D /root/deciders168_finish et C appelant ;
T051=E credits_delivery_prepare pour DTO/projection et C constructeur. Aucun complément exécuté présumé.
Les fichiers Lineage ancienA sont transférés exclusivement à E : lineage_protocol.rs, delegation_lineage.rs/tests,
daemon/native_lineage.rs, cli_lineage.rs, lineage_client.rs et bancs Lineage associés. A initial reste gelé.
Le gel P1 est levé pour les corrections hors passe ; ROOT annoncera le nouveau gel avantP2.

État courant avant Converge : les anciennes mentions « aucun Cargo » et « reste à raccorder » ci-dessous décrivent les publications initiales, pas le statut final des ciblés.
Reçu exécuté : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md.
57noms distincts à dernière issuePASS ;31/49tâches cochées,18ouvertes selon matrice. A initial est gelé ; E actuel = ROOT/credits_delivery_prepare.
Le code des raccords est publié ; aucune livraison/activation168 ni preuve complèteT034 n’est déduite.
GEL documentaire pour Converge : aucune écriture hors appendTasks explicitement ordonné par ROOT.

Date : 2026-10-15. Source de travail : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte
Écrivain unique de cet index : /root/document_git_handoff. Les lots publient leurs interfaces par message à ROOT et dans leur reçu propre.
Cet index décrit le code publié, sans attester son exécution. Aucun Cargo168 n’a été exécuté au point de consolidation. Les 49 tâches restent à leur état précédent : T001 seule cochée, 48 non cochées.

## Propriétaires et contacts

| Lot | Task native | Agent | Propriété exclusive |
| --- | --- | --- | --- |
| A initial gelé | 07570621-9d07-48c2-a572-5d782b71c95f | 0f3bfdc2-ffe5-4762-a2a5-4aaf476d0ba9 | Registre/catalogue et Lineage remis ; aucune édition active supposée. |
| B natif, terminé | f167837a-c567-434e-b253-a08e7ce6706c | 35020615-9ec3-4c40-a73a-3f138eb3f6b5 | Publication des types initiaux ; STOP édition confirmé. |
| Relais ROOT | exécution collaboration credits_cartae_fix | désigné par ROOT | host_resources.rs, remote_worker.rs, fixture162 et seul ajout PrepareV2 dans protocol.rs. |
| C natif fermé → relais ROOT | /root/admission168_finish | écrivain collaboration désigné par ROOT | Maître, probe côté maître, admission, CLI/MCP, catalogue ; enregistrement du module collecteur si nécessaire. Surfaces Lineage transférées à A. |
| D natif fermé → relais ROOT | /root/deciders168_finish | écrivain collaboration désigné par ROOT | Service de décision, transport Claude isolé, exports limités et Cargo.toml transport pour seule feature serde_json/raw_value. |
| E relayé ROOT | credits_delivery_prepare | E initial historique | Store delegation.rs et, depuisP1, DTO/lecteurs/projection Lineage transférés deA ; C seul constructeur des faits du snapshot. |

C et D natifs ont remis leurs résultats partiels et leurs continuabilités sont fermées. Les responsabilités C/D désignent désormais les relais ROOT ci-dessus, mêmes fichiers et bancs. A natif registre/Lineage et E natif store restent inchangés.
ROOT détient le permis Cargo, la validation et la livraison. B et E n’écrivent pas cet index central.
Les consommateurs peuvent utiliser une interface publiée avant le cochage de sa tâche de preuve. Ce cochage attend l’exécution autorisée et vérifiée.

## A — matrice déclarée et sélection pure

Source : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs

- RoutingPolicy version2 : connexions, model_catalog par connexion et machine_policies. Son absence conserve163.
- RoutingCandidate sépare machine_id, engine, connection_id, agent_type et model. Il porte enabled, supported_efforts, default_effort, provenance et observed_at.
- AgentRegistry::routing_policy() -> Option<&RoutingPolicy>.
- AgentRegistry::routing_candidates() -> Option<Vec<RoutingCandidate>> : toutes les combinaisons déclarées, désactivées comprises ; plafond128 avant filtres, sans produit cartésien des efforts.
- AgentRegistry::routing_catalogue(observed_at: i64) -> Result<RoutingCatalogueV1, String>.
- AgentRegistry::select_routing_candidates(coordinator_machine_id: &str, requested: &RoutingCandidateFilters) -> Result<Vec<RoutingCandidateSelection>, String>.
- RoutingCandidateFilters porte engine/model/machine_id/connection_id/effort optionnels ; RoutingCandidateSelection conserve le candidat et imposed_effort.
- Un défaut natif sans réglage est default_effort=null avec supported_efforts vide. Aucun medium implicite. Ces lectures ne lancent ni découverte fournisseur, ni authentification.
- La fixture et le catalogue déclaratif ne prouvent pas le droit fournisseur. Les oracles12/128/129 attendent ROOT.

## B — contrats additifs et raccord worker en cours

Sources :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lineage_protocol.rs

- AutoDelegateV2 ajoute les axes indépendants, workload fermé general|compilation et expected_base_commit. HandoffV1 reste fermé et distinct.
- routing_catalogue_v1 nomme l’opération et sa capacité dédiée. RoutingCatalogueV1 et RoutingCatalogueOptionV1 portent la projection publique.
- WorkerCapability::CandidateProbeV2 négocie le probe168. Les noms exacts sont CandidateProbeV2Candidate, CandidateProbeV2, CandidateProbeV2Verdict et CandidateProbeV2Result.
- CandidateProbeV2 porte version/correlation_id/connection_epoch/worker/admission_id/policy_revision/parent/resources/candidates. Chaque candidat porte notamment connection_id, expected_base_commit, model, effort, workload et workspace.
- CandidateProbeV2Result reprend les corrélations et contient resources et verdicts. La corrélation et l’époque doivent être contrôlées au maître ; le probe n’est pas une réservation.
- HostResourceSnapshotV1 porte source_observed_at, received_at, cpu_percent, available_memory_percent, normalized_load_percent et compilation_slots. Les quatre dernières valeurs peuvent être inconnues ; le premier CPU null reste valide.
- IDs168 : c001..c128. IDs163 : c01..c08, inchangés.
- LineageDecisionV4 porte candidate_id/connection_id/machine_id/engine/model/effort/source/reason. Sources : explicit, single, decision_service, lightweight_model, deterministic. Lineage v1/v2/v3 reste compatible ; décision168 projetée null en v3.
- Le reçu B est une publication partielle T002. B a terminé puis STOP édition ; ROOT a transféré T013/T016/T024/T032 à credits_cartae_fix. La collecte, les gardes worker et la propagation vers WorkerPrepare restent à confirmer dans une publication finale ; aucun achèvement n’est déduit ici.
- Le relais ROOT/credits_cartae_fix expose le collecteur réutilisable. C l’appelle dans l’entretien maître existant, sans nouveau thread ni daemon.

### PrepareV2 — interface de raccord T024 décidée par ROOT

WorkerRequestOperation::PrepareV2 { prepare: WorkerPrepare, workload: AutoDelegateWorkload, connection_id: String,
engine: String, expected_definition_digest: String, expected_permission_digest: String }.
L’opération est additive et négociée ; modèle/effort/base restent dans WorkerPrepare.
workload est obligatoire, figé depuis snapshot/probe : Shared hors Git général peut garder baseNone ; Worktree/compilation exigent Git/base vérifiée. Protections de chemins et mapping existants conservés, aucune configuration de mapping nouvelle.
C émet le tuple/digests figés et garde le maître. ROOT/credits_cartae_fix implémente le
worker : réemploi des gardes CandidateProbeV2, résolution locale de connexion,
relecture tuple/droits/digests avant tout effet. Le chemin Prepare v1 reste inchangé.
Ce recontrôle est nécessaire : agent_type seul ne distingue pas deux connexions et
WorkerPrepare v1 ne transporte pas connection_id. Aucun nouveau lancement implicite.
Configuration A : RoutingMachinePolicy.compilation_capacity: Option<u8>, borne0..10,
défautNone. Déclaration simple ; pas de benchmark.

## C — façades publiées, admission encore en raccordement

- bridget_auto_delegate construit V2 et exige auto_delegate_v2 ; V1 reste un chemin distinct.
- bridget delegate transmet les axes et résout le SHA de la source locale avant l’enveloppe.
- delegate --catalogue [--machine] [--json] lit le catalogue sans lancement ; le filtre est local à la CLI.
- Le maître impose même UID Unix, rôle Client et capacité routing_catalogue_v1. Aucune autorité T3 n’est requise pour cette lecture.
- Le plan V2 possède son snapshot typé ; ses faits métier sont conservés dans metadata v2 opaque pour E. metadata.decision: Option<LineageDecisionV4> vaut None avant choix, puis l’objet complet fermé source/combinaison au choix.
- Les constructeurs externes AutoAdmissionReceipt doivent renseigner metadata_version:u8 ; les reçus163 portent1, les nouveaux snapshots168 portent2.
- Le reçu C laisse explicitement à raccorder : probes B, score/saturation, chaîne service→Claude→score, CAS de choix, Prepare.base_commit et reprise ; A raccorde désormais la projection Lineage.
- Pour chaque candidat Git Shared/Worktree, expected_base_commit source exact doit atteindre probe et Prepare. Sans SHA, exclure ce candidat par workspace_base_required ; sans candidat restant, refuser globalement. Hors Git Shared général peut omettre le SHA ; un SHA fourni hors Git refuse mismatch. Une erreur technique ne signifie jamais hors Git. Worktree exige Git.

### Transfert des lecteurs et projections Lineage à A

C a confirmé n’avoir pas commencé ces fichiers et arrêté leur édition. A possède :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_lineage.rs
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_lineage.rs
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli_lineage.rs
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lineage_client.rs
ainsi que les tests delegation_lineage et bancs Lineage associés.

A implémente T029 et couvre les assertions de compatibilité Lineage T034, avec les surfaces
CLI/reader nécessaires à Lineage v4. T030 reste entièrement sous C pour admission, concurrence,
reprise et rollback ; aucun partage artificiel de cette tâche. C reste seul écrivain de
native_delegation.rs et des oracles natifs ; A ne prend pas ces fichiers.
C garde admission/quotas/CLI/MCP/maître et fournit le snapshot décision complet.
A lit metadata.decision figée pour la v4 ; v3 continue decision:null sur une tâche168.
Ce transfert ne signifie ni tâche terminée ni test réussi.

## D — charge, choix fermé et durée du permis

Sources :
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs
/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs

- RoutingDecisionRequest regroupe admission_id/policy_revision/brief/workload/constraints, les faits machines et connexions, puis les candidats. Chaque RoutingDecisionCandidate porte les noms réels modèle/moteur/machine/connexion, les efforts, le score et ses faits.
- RoutingDecisionChoice et ClaudeIsolatedDecisionResult portent candidate_id, effort optionnel et reason optionnelle.
- validate_routing_request(&RoutingDecisionRequest) et validate_routing_choice(candidate_id, effort, reason, &RoutingDecisionRequest) contrôlent localement les bornes et l’appartenance au snapshot.
- DecisionServiceClient::acquire_routing_permit() -> Result<RoutingDecisionPermit, DecisionServiceError>.
- DecisionServiceClient::decide_routing(service, request, deadline) -> Result<RoutingDecisionChoice, DecisionServiceError>.
- run_isolated_claude_decision(&ClaudeIsolatedDecisionOptions) -> Result<ClaudeIsolatedDecisionResult, ClaudeIsolatedDecisionError>. Options : command/model/environment/request_json/timeout.
- C conserve RoutingDecisionPermit pendant service puis Claude, jusqu’au drain réel. C valide le résultat fermé avant CAS et ignore toute réponse tardive. Il n’ouvre pas une nouvelle décision sur rejeu.
- Le relais D active la seule feature serde_json/raw_value existante dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/Cargo.toml pour borner l’objet brut à4Kio avant parsing détaillé. Aucun package nouveau, aucun parseur artisanal ; API publique D inchangée.
- Mode Claude : environnement vidé puis allowlisté, cwd privé neutre, safe-mode et drapeaux qualifiés. inherit_stderr=false ; stderr non conservé. Stdout limité à64Kio et résultat à4Kio avant parsing.
- Aucun outil métier/système/réseau/MCP. Seul StructuredOutput, formateur natif exigé par json-schema, est autorisé sans droit d’exécution. Toute autre capacité ou tool_use est refusé. Objet fermé ID/effort et concordance des sorties restent validés localement.
- La préqualification CLI réelle est distincte du transport168 et des recettes Bridget : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/qualification-claude168.md

## E — version du reçu et transitions atomiques

Source : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs

- AutoAdmissionReceipt.metadata_version:u8 distingue v1≤16Kio/c01..c08 et v2≤128Kio/c001..c128. Canon≤512Kio. E ne duplique pas les faits métier de C.
- Base neuve : DDL168 direct. Base163 reconnue : migration transactionnelle, copie SQL byte-identique des BLOB en version1 et recréation des trois index. Forme inconnue refusée ; quick_check/FK prévus.
- AutoAdmissionDecisionLevel = DecisionService | LightweightModel.
- start_auto_decision_level(receipt, expected_epoch, expected_revision, level, now) -> Result<AutoAdmissionReceipt, String>.
- advance_auto_admission_decision_level(receipt, expected_epoch, expected_revision, expected_level, next_metadata: Vec<u8>, now) -> Result<AutoAdmissionReceipt, String>.
- choose_auto_admission_at(receipt, expected_epoch, expected_revision, level, candidate_id, delegate_canonical, mode, reason, chosen_metadata: Vec<u8>, now) -> Result<AutoAdmissionReceipt, String>.
- choose_auto_admission_at valide chosen_metadata v2 et le niveau inchangé, puis persiste metadata avec Chosen dans le même CAS. La projection ne dépend pas d’une seconde écriture ; aucune nouvelle colonne/table.
- Ces CAS vérifient époque, révision et niveau. C fournit metadata complète et fait la sélection métier ; un CAS perdu ne donne pas de droit à une redécision.
- Modes SQL conservés : explicit|automatic|fallback ; la source détaillée168 reste dans metadata. explicit/single→explicit ; decision_service/lightweight_model→automatic ; deterministic→fallback.
- Retour arrière : procédure opérateur. Avec des lignesv2, garder un binaire168 compatible, éventuellement168OFF, ou restaurer un ensemble cohérent sans perte. Aucun verrou rétroactif dans un ancien binaire n’est supposé.

## Trou contractuel observé et clôtures attendues

Le défaut de raccord Laya168 vers les validateurs163 a été signalé à ROOT. Dernier code D lu sans exécution : decide_routing appelle désormais laya_routing_request_payload puis parse_laya_routing_choice, séparés du chemin163. Le reçu D publie aussi l’alignement des flags sur la préqualification et les contrôles de concordance des sorties ; les oracles source ajoutés ne sont pas exécutés. Le correctif est donc publié au raccord, sans PASS ni clôture de validation déduite. C doit encore raccorder la chaîne complète, le refus de capacité et les transitions CAS.

Les publications finales du relais worker ROOT, de C et de A pour Lineage doivent préciser collecteur, corrélations du probe, contrôle connection_id, expected_base_commit jusqu’à Prepare, score/quota/saturation, CAS/reprise et projection Lineage. Leur absence au reçu partiel est un travail en cours, pas un succès supposé. Les oracles migration/CAS, compatibilité, chaîne et catalogue restent non exécutés.

## Reçus observés et empreintes

Préfixe absolu commun : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/

| Reçu | SHA-256 observé |
| --- | --- |
| developpement168-a.md | 765de75ab557c8381ea24f2883ada96b166e34684cb5dcf885e0bc0d4118f39a |
| developpement168-b.md | bca9033aa5984e3f30df8ae05cf83c0482c9622669fd4f50d77093f90176135c |
| developpement168-c.md | 08ab13f29792e06bd9dbd16c324b7af0bb79cc48609c1c885492b02f32406075 |
| developpement168-d.md | a45d635aa264fd4f09c245667b955fd46f793f052890f9ccb16d1e95f44fde4c |
| developpement168-e.md | 2130679d791fd20806f0a9529b8bc52354ee51f3642fdb60fd3b3d87cfe87aa7 |

Ces reçus rapportent rustfmt et contrôles d’espacement. Aucun succès Cargo, test, build ou recette168 n’est retenu par cet index. Les sources B/C peuvent évoluer pendant le raccordement ; ces empreintes identifient les reçus lus, sans prétendre figer leur code.

Consolidation documentaire terminée. Attendre le freeze de code ROOT avant toute nouvelle consolidation ; aucune validation déduite des publications.
