# Audit de reutilisation de l'existant — 168-routage-flotte

## Decision

Statut: PASS
Date: 2026-10-15
Feature dir: /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte

Conclusion courte: aucune duplication évidente de service, base, ordonnanceur, moteur ou commande n'est proposée. Les extensions de wire, de snapshot et de sûreté sont nécessaires aux limites réelles du socle. Les précisions de catalogue dans un terminal, workload hors brief historique, SHA Shared/Worktreev2 et mode Claude compatible OAuth sont désormais arbitrées et reportées de façon cohérente dans les documents corrigés figés. Une relecture ciblée a fermé ces conditions ; aucun doublon bloquant ni arbitrage utilisateur ne reste ouvert.

Audit de réemploi unique, en lecture des sources sur Cartae. Seul ce fichier peut être écrit par l'auditeur. Aucun code, test, modèle, service, base de production, configuration ou Git n'a été modifié. Ce verdict porte sur la conception et le réemploi ; il ne qualifie pas les drapeaux Claude installés, l'exécution, l'activation ni la revue adversariale.

Périmètre lu intégralement : plan, spec, research, data-model, contrat et quickstart168. Source du tableau ci-dessous : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/plan.md, sections Architecture 1–5, migration, ownership, validation et livraison. Les numéros R01–R40 rendent chaque responsabilité extraite traçable. Les variantes de compatibilité sont auditées individuellement ; les fichiers de lot ne sont pas comptés comme une architecture indépendante.

Empreinte du plan initial audité : 75fb09bdbb3a6a431dd159a4cb4b52eddcb75bf257f3849a5f8a690dc705fc60.
Après corrections documentaires r1, relecture ciblée du plan figé : SHA-256 3078e58e0b135abe28208bc67e07085ea70acd7dcc961af63657454bb4bec401 ; contrat SHA-256 35f6b11fb992848f1a1ca538ee6251ee20eb31e5e0c8f46e6b94c20406e4a7b8. Réponses ROOT/docowner : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/plan-review168-r1.md. Les changements de conception r1 ont été relus uniquement sur ces points, sans second passage d'exploration large.
Le script /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/.specify/scripts/bash/check-prerequisites.sh est absent. Le chemin explicite fourni par ROOT et le fallback documentaire du plan ont été utilisés, sans recréer les scripts gérés.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 40 |
| Items audites | 40 |
| Reutilisations deja prevues | 33 |
| Existants potentiellement pertinents | 7 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 10 |
| Specs existantes applicables | 8 |
| Clarifications documentaires a fermer avant Tasks | 0 |

## Reutilisations correctement identifiees

Dans chaque ligne, « étendre » signifie modifier le point commun existant, pas ajouter un second parcours indépendant. « Nouveau justifié » désigne une donnée ou frontière absente ; cela n'autorise pas un nouveau framework.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/reuse-audit.md#extrait-1).

## Existant potentiellement pertinent non mentionne

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/reuse-audit.md#extrait-2).

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun doublon 1:1 non justifié trouvé dans le plan | Tableau R01–R40 et journal ci-dessous | Conserver les extensions dans les puits communs. Les seules nouvelles surfaces sont des versions/données/frontières nécessaires ; aucune seconde base/service/commande n'est autorisée par cet audit. |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| Instructions AGENTS utilisateur du fil et /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/AGENTS.md | français, chemins complets, travail serveur et ownership | Audit sur Cartae, rapports complets, aucune écriture de code concurrente. Les checkpoints historiques ne prouvent pas168. |
| /home/moi/.speckit/constitution.md, III et XVI | cycle SpecKit et worktree | Audit avant Tasks dans le worktree168 autorisé ; pas de code anticipé. |
| /home/moi/.speckit/constitution.md, XVIII | complexité explicite et pas de N+1 | Faits groupés par hôte/connexion, O(C+P), tri unique et bornes128/TTL/sorties ; aucune collecte par modèle. |
| /home/moi/.speckit/constitution.md, XIX | réutiliser avant créer, pas d'abstraction décorative | Pas de provider/runner/base parallèle. Nouveau rôle décideur justifié par isolement et compatibilité. |
| /home/moi/.speckit/constitution.md, XX | responsabilité et confidentialité | Réponse modèle sans autorité ; aucune donnée de credential, aucune preuve de disponibilité inventée. Écritures natives auth/state distinguées des écritures métier. |
| /home/moi/.speckit/constitution.md, XXI et instructions utilisateur | routines opt-in et anti-boucle | Pas de routine d'incident, trace observer ou fichier de passation bavard ajouté par cet audit. |
| /home/moi/.speckit/ref/code-quality-details.md, CXXIV/CXXV/CXXVII/CXXVIII/CXXIX | contrat, erreur, CAS, validation et dépendances | Schémas fermés, erreurs techniques visibles, migration transactionnelle, nouvelles versions explicites ; libc/rusqlite existants suffisent. |
| /home/moi/.speckit/ref/speckit-workflow.md | statuts et preuves distinctes | Spec168 reste Draft/Planned ; audit documentaire ne coche aucun test ni implémentation. |
| /home/moi/.speckit/ref/standards-tests.md et stratégie explicite de cette session | comportement et fixtures aux frontières | Tests Rust existants réemployés ; pas de framework Python/Gherkin importé dans un workspace Rust. Campagne ciblée puis FULL unique autorisée, preuves héritées séparées selon les instructions utilisateur prioritaires. |
| /home/moi/.speckit/ref/agent-orchestration.md et mandat ROOT | exploration déléguée, un seul écrivain | Un passage de recherche par cet auditeur ; seul reuse-audit.md est écrit. Pas de build/test concurrent. |

Sources projet absentes sur ce worktree : CLAUDE.md, .specify/memory/constitution.md, .specify/memory/standards.md et .specify/memory/user-layer.json. Le fallback est explicite dans la recherche168 et le plan. Aucune source gérée SpecKit n'a été recréée. Aucun accès aux secrets ni écriture DevKMS hors mandat n'a été effectué.

Minimalisme & Frugalité : nouveau service0, base métier0, ordonnanceur0, dépendance0, commande parallèle0. Les variantesv2/v4 ne sont pas des moteurs concurrents ; elles portent les contrats fermés et la compatibilité réelle. Potentiel minimalisme : ~0 ligne de code chiffrable à ce stade, aucun code168 n'existant ; éviter surtout de recopier JSON-RPC, stdout, santé/CAS et quotas. La liste R01–R40 est une couverture de responsabilités, pas une demande de40helpers.

Vertus LLM & Responsabilité Future : la charge cognitive augmente avec les versions nécessaires, mais reste contenue par une seule admission et les transports existants. Chaque nouvelle frontière a une raison explicite. Ni un prompt ni une probabilité du modèle ne vaut permission. Le volume réel et la simplicité de l'implémentation devront être relus sur le diff ; ils ne sont pas prouvés par le présent plan.

## Specs livrees applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/reuse-audit.md#extrait-3).

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/reuse-audit.md#extrait-4).

## Arbitrage de raccord T024 — complément du 2026-10-15

Décision ROOT après lecture : WorkerPrepare v1 ne transporte pas connection_id.
Le contrôle final ne peut pas déduire une connexion depuis agent_type partagé.
R10 réemploie l’inspection Prepare et R35 la base ; ces lignes ne couvrent pas à elles seules
la preuve de connexion/digests entre probe et Prepare. L’extension additive fermée
WorkerRequestOperation::PrepareV2 { prepare: WorkerPrepare, workload: AutoDelegateWorkload, connection_id: String,
engine: String, expected_definition_digest: String, expected_permission_digest: String }
est donc justifiée. Réemployer le même canal négocié, WorkerPrepare et ses contrôles,
avec relecture locale tuple/droits/digests avant effets. Aucun nouveau runner/store ou workflow.
Modèle/effort/base restent dans WorkerPrepare ; le chemin v1 reste inchangé.
Ce complément est un arbitrage de réemploi, sans nouveau gate PASS ni preuve d’exécution.
Ownership après STOP B : ROOT/credits_cartae_fix possède worker/collecteur/fixture et
le seul ajout PrepareV2 de protocol.rs ; C garde maître/admission/façades.

## Complément de raccord — workload et borne JSON avant parsing

PrepareV2 porte workload: AutoDelegateWorkload obligatoire avant connection_id.
Le worker réemploie l’inspection/protection des chemins et le mapping existants :
Shared hors Git général peut omettre la base, Worktree ou compilation exigent Git/base vérifiée.
Aucune nouvelle configuration de mapping n’est justifiée.
Pour la limite4Kio avant parsing détaillé, activer la seule feature serde_json/raw_value
existante dans le Cargo.toml transport. R24/R39 imposent le réemploi du parseur :
aucune dépendance nouvelle ni analyse JSON artisanale. API publique D inchangée.
Relais d’écriture : /root/admission168_finish remplace C natif fermé ;
/root/deciders168_finish remplace D natif fermé et possède ce changement Cargo limité.
ROOT/credits_cartae_fix conserve worker/collecteur/PrepareV2 ; A et E natifs inchangés.
Ce complément ne crée aucune tâche, aucun gate PASS ni preuve d’exécution.

## Arbitrages

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/reuse-audit.md#extrait-5).

| OAuth et mode Claude | remplacer --bare par --safe-mode et les restrictions explicites ; ne pas conserver stderr | ROOT a vérifié les aides locales : --bare exige une clé API, incompatible avec la connexion OAuth visée. Les bornes stdout et l'arrêt détenu restent obligatoires ; aucune transcription stderr privée à stocker. | 2026-10-15 |
| SQL neuf et legacy | DDL168 pour DB neuve, migration163 connue pour DB existante, mode SQL garde explicit/automatic/fallback | La source détaillée168 appartient aux métadonnées versionnées, pas à une nouvelle valeur rejetée par le CHECK SQL historique. | 2026-10-15 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

Conditions fermées avant Tasks : R06 catalogue natif same-UID/Client par le parcours natif Bridget ; R15 workload fermé v2 hors brief153 ; R35 expected_base_commit obligatoire pour Shared/Worktree Git, corrélé au snapshot/probe/Prepare et recontrôlé localement ; mode Claude safe-mode/OAuth et stdout borné sans conservation stderr ; DDL168 neuf et trois modes SQL historiques conservés. Aucun autre doublon à arbitrer, aucune nouvelle question humaine exigée.

PASS signifie passage possible à la génération des tâches après la revue de plan gérée par ROOT. Il ne transforme pas les capacités Claude projetées, les mesures CPU/RAM, la migration SQL ou les recettes de décision168 en preuves d'exécution.


## Complément revue code R1 — coefficients et canon

Arbitrage ROOT, 2026-10-15 ; aucune nouvelle tâche ni nouvelle exécution.
RoutingScoreWeights dans le registre représente seulement les sept coefficients privés fermés
déjà demandés par R13 et le contrat : cpu/ram/load/continuity/compilation/quota/reset.
Aucun type équivalent de coefficients n’a été trouvé dans le registre ; les seuils de saturation
préexistants sont des conditions d’admissibilité, pas des poids. Réemployer la politique/snapshot
et le calcul pur commun. Aucun service, couche, dépendance ou stockage supplémentaire.
Les poids de la coordinatrice s’appliquent à toutes les cibles ; seuils et fraîcheur restent par cible.
Bornes/défauts20/20/20/20/10/5/3, arrondi CPU/RAM/charge au plus proche ; pénalité100 fixe.
E finalise skip_serializing_if pour weights==Default : omission du JSON canonique afin de
préserver le digest d’un ancien snapshot168 ; non défauts sérialisés et figés.
C a publié calcul/horloge/note/aide selon ROOT. La garde source≤réception des mesures reste
requise avant validation ; aucune lecture théorique ou publication de code ne vaut résultat de test.
Ce complément précise R13 sans nouveau grade, gate PASS ou audit global.


## Complément audit T038 — correction des raccords, validation future

Le probe groupé ne suffisait pas : la boucle worker répétait origin/HEAD/préflight Git par modèle.
Le RED instrumenté ROOT a compté 521 appels Git réels pour 128 options et 2 tuples.
Réemploi R14 : cache éphémère des seuls faits Git par tuple complet dans le lot, succès et erreurs
compris. Le tuple distingue worker/époque/provision, projet/révision mapping, racines/cwd/identité Git,
workload, workspace/base_ref/branche et SHA. Aucun cache de droits par simple chemin.
La connexion, la définition, le modèle, l’effort et les droits restent individuels ; les gardes
fraîches Prepare/Start déjà présentes restent l’autorité. Pas de watcher, service ou cache global.

Le premier contrôle distant du puits Worktree ne doit pas changer sa stratégie en Shared :
PrepareV2 ajoute inspect_only, false par défaut et absent de V1. En true, vérifier le Worktree
futur/base_ref/SHA et renvoyer WorkerPrepared avec token réservé inspect-only-v1 sans état Prepared,
worktree ni slot. Start refuse ce token ; second Prepare normal après Task durable.
Ce champ fermé étend le canal et les gardes existants ; aucun runner, store ou saga supplémentaire.

Réemploi R25 : passer le même Instant absolu admission→préparation→scheduling→I/O, sans renouveler
un budget relatif. Réemploi R11/R24 : si l’effort exige une seconde sonde, comparer définition
brute, époque et mapping avec la première observation fraîche validée ; droits peuvent différer
avec l’effort, jamais la session ou la définition par dérive silencieuse.

Ces arbitrages répondent aux défauts d’audit. Les corrections et leur exécution finale ne sont
pas encore attestées ici : aucun finding fixed, nouveau gate PASS ou audit global revendiqué.
T038 demeure ouverte ; ses recettes/livraison ultérieures ne sont pas déduites.
