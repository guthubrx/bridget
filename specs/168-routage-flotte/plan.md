# Plan SPEC168 — Routage de la flotte

## État courant — 15/10/2026 à 17:24 CEST

**Implemented — 56/56 tâches cochées. Pipeline : Livré, avec conservations explicites T048.** ROOT a validé le contrôle final des 35 exigences, 11 critères et 7 histoires. Aucune tâche ne reste ouverte.

La récupération documentaire T047 est prouvée : C1 `db46c7816777199da7c41bb9bc10063cd361dae7` intégré et poussé, main Mac/Cartae alignés et propres aux reçus. Source binaire B et release v0.4.0 inchangées ; aucun rebuild. Preuves : /home/moi/.local/state/bridget-delivery168/git/integration-pushC1-cartae.json, SHA-256 `e32d1529d986fff8af4441928353d8b1a5fd163e4aefaf7e3b90651d9ce90454` ; /Users/moi/.cache/bridget-delivery168/git/integration-pullC1-mac.json, SHA-256 `4cf769d0707b2b28535a6f91052d786ff5c5b1c861738e645f6a20df84111766`.

L’examen T048 est terminé avec retraits et conservations motivées. Le worktree source Cartae et sa branche sont conservés : C1 fusionné/poussé, arbre propre et zéro ignoré ; garde lsof RC0 mais 4 337 octets stderr, donc observation incomplète. Aucun match observé ne vaut preuve d’absence d’usage. Aucun contournement ni suppression inventée. Les 18 anciens candidats conservent leurs motifs. Reçu : /home/moi/.local/state/bridget-delivery168/cleanup/t048-source-final-guard.json, SHA-256 `e2410c3dc3d838b1fb49cdb065f0a9c067cda717a105cce4d0458b94ea25cfc3`. Bilan : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/nettoyage168.md. Le contrôle final T049 est validé. Le SHA du commit documentaire final est vérifié hors de son propre contenu dans les reçus Git privés.

La source binaire B `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0, est construite et activée sur Mac/Cartae. Les trois voies réelles ont résultats et rejeux vérifiés : Laya `decision_service` sur 2 candidats, Haiku `lightweight_model` sur 21 candidats, dernier repli `deterministic` sur 21 candidats. Détail : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/recette168.md.

**Release publiée et vérifiée :** https://github.com/guthubrx/bridget/releases/tag/v0.4.0 . Tag annoté `43202c9de66ce466b76ba05bd3077699ac88ca3d`, cible exacte B ; quatre actifs retéléchargés et hashes conformes. Reçu : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/release168.md, SHA-256 `15a7b44610ebe060c47164e6c511606e67c37c4d2e4e85a71c2db5be879b9f5d`. Preuve privée : /home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication168.json, SHA-256 `d815ca75ca354ec4941577cb39a29aab62cf641ac71895f2ba69e3f0c8ea9666`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-1).

Preuves existantes :

- /Users/moi/.cache/bridget-delivery168/final-health/mac-health.json — SHA-256 `be3ec722c86f8fcdd6de90353e1748e318b0de3beb086690cc66fdd276d114ba` ;
- /Users/moi/.cache/bridget-delivery168/final-health/database-health.json — SHA-256 `98feb24442412b8e2ce12d5c6a92e75605c500bd6f67b4e7fc349fc6cebab147` ;
- /home/moi/.local/state/bridget-delivery168/final-health/cartae-health.json — SHA-256 `a409e7698a2f7ff7bdede77a2a2126eeda0d835052765473f0ef60850818f32a`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-2).

Cet état daté prévaut sur les checkpoints historiques ci-dessous, dont les anciennes mentions de publication en préparation. Les DB, preuves et sauvegardes sont protégées.

---

**Statut** : Implemented ; 56/56 tâches cochées. Pipeline : Livré avec conservations motivées. T045–T049 closes ; source binaire B active, trois recettes et release vérifiées, documents intégrés et poussés.
**Gates de conception historiques** : réemploi PASS 5/5 ; contre-revue Claude Sonnet 5.5 R1 corrigée puis R2 APPROVE. Les preuves de code et de validation ultérieures sont distinguées ci-dessous.
**Date** : 2026-10-15.
**Branche** : session-168-routage-flotte.
**Base** : 1f69e5d8e651dd27907fd244aee6f90b6b734565.
**Estimation initiale ROOT publiée à09:09CEST** :300–550minutes (5h à9h10),
fin haute vers18:20CEST, hors indisponibilité fournisseur ou authentification interactive.
Une réévaluation de risque ne remplace pas silencieusement cette ETA ; recalibrer après Tasks.
**Périmètre** : totalité des sept histoires et35FR ; le petit décideur fait partie de la livraison.

## Point de contrôle avant commit A — 15 octobre 2026

État : **In Progress**, 45 tâches cochées sur 56 après T036. Le code est réalisé ; Converge P3 est CONVERGED, la contre-revue T037 et ses corrections sont closes, l’audit T038 et la FULL unique T036 ont leurs reçus. T039 est en publication ; T040 et les étapes de livraison restent à suivre. Ce checkpoint ne coche aucune tâche et ne vaut ni installation ni livraison.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-3).

Preuves et état de livraison :

- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/audit168.md
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/converge168.md
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/adversarial-review-code168.md
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md

## Contexte technique et socle

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-4).

Le socle163 porte déjà le contrôle de concurrence, le service HTTP et la reprise CAS.
Le socle167 porte les façades CLI. Les versions154/157/160/162 portent workspace, quotas et admission distante.
Les étendre dans leurs puits communs, sans refaire un second moteur.

## Vérification de constitution avant conception

| Gate | Application |
|---|---|
| Réutilisation / XIX | Mêmes transports, lifecycle, stores, maintenance et façades. Pas de moteur d’orchestration supplémentaire. |
| Complexité / XVIII | Construction indexée et classement O(C log C), C≤128 ; un probe groupé par worker et faits Git partagés par tuple dans le lot. Droits/modèle/connexion/effort restent individuels ; aucun Git répété pour des tuples équivalents. |
| Paresse vertueuse / XX | Une connexion par environnement, catalogue de modèles séparé ; pas de duplication de profil par modèle. |
| Impatience maîtrisée / XX | Campagnes ciblées par surface, un seul build/check coordonné ; pas de FULL répétée ou framework de benchmark. |
| Exigence professionnelle / XX | Les tests couvrent choix, refus et erreurs techniques ; les noms de modèles et capacités sont observés avant activation. |
| Responsabilité future / XX | Versions de wire et snapshot explicites ; compatibilité ancienne testée et modes de repli lisibles. |
| Droits et confidentialité | Décideurs sans outils, sans secrets automatiquement injectés ni autorité d’admission ; brief utilisateur borné et inerte, permissions finales inchangées. |
| Livraison | Commit, intégration, push, installation utile et nettoyage vérifié font partie de la fin de session. |

Aucun gate n’exige une nouvelle abstraction générique d’authentification.
L’absence des scripts/templates SpecKit sur Cartae est traitée par le fallback documentaire,
sans modifier les sources gérées de SpecKit. Pas de question utilisateur bloquante restante.

## Architecture retenue

### 1. Registre : connexions, catalogue et politique168

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-5).

Les options sont construites depuis les connexions et modèles, avec filtres indexés. Chaque option porte
la liste des efforts compatibles ; ne pas matérialiser un produit cartésien par effort. Le choix final
valide l’effort choisi parmi cette liste, ou garde l’effort imposé.
La limite globale statique est128options déclarées, avant filtres de demande. Registre129 : refus
explicite, jamais top128, top8 ou catalogue partiel complete:false. Pas de multiplication des profils.
Chaque modèle éligible porte un champ default_effort résoluble : membre de supported_efforts
si des niveaux existent, sinon null avec liste vide pour le défaut natif sans flag. Aucun medium inventé ;
un effort imposé sur un modèle sans réglage est incompatible, pas un motif pour masquer ce modèle.
Les labels de modèles commerciaux ne sont pas inventés depuis le nom du moteur.
Tous les modèles réellement disponibles Terra/Luna/Sol et Haiku/Sonnet/Opus des machines activées
doivent être inventoriés puis exposés, sans activation annoncée d’un lanceur absent.

Les réglages par machine du coordinateur couvrent automatisation de chaque axe, pondérations,
disponibilité du service, disponibilité du petit décideur et budgets.
Une connexion ou un niveau désactivé n’est pas essayé. Aucune nouvelle authentification dans le daemon.

### 2. Faits frais et score déterministe

Grouper la lecture/probe par machine et connexion, avec une seule collecte hôte par lot.
Rattacher le résultat à worker_identity, connection_epoch, révision de configuration et observed_at.
Un résultat d’une ancienne connexion/époque n’est jamais admis après reconnexion.

Mesures : CPUbusy depuis deux échantillons, RAMdisponible/totale, charge par CPU,
capacité compilation déclarée et occupation connue de Bridget. Données absentes = null.
Premier échantillon CPU sans delta valable = null. Mémoriser les échantillons et les rafraîchir
au heartbeat existant et lors du probe, une collecte par machine/lot. Aucun nouveau daemon,
thread de benchmark, polling Git ou attente bloquante pour fabriquer le premier delta.
Fraîcheur proposée :30s par défaut, configurable1..300s ; le maître utilise aussi son instant de réception
pour qu’une horloge distante décalée ne transforme pas une vieille mesure en fait frais.

Avant le score, une saturation franche fraîche exclut la machine si une autre machine admissible
est attestée non saturée ; aucun décideur ne peut la réintroduire. Une machine imposée saturée
refuse avec resource_saturated, sans autre compte/machine implicite.
Score entier explicable sur les possibilités restantes : bonus continuité dépôt/cache total≤20 ; pénalité saturation fraîche100.
CPU≥90%, RAMdisponible≤10% ou charge normalisée≥1 constitue une saturation proposée,
avec seuils configurables dans le registre. Les inconnus restent neutres avec mention visible.
Les coefficients bornés sont portés par telemetry.weights de la machine coordinatrice et
appliqués uniformément aux cibles ; seuils et fraîcheur restent propres à chaque machine cible.
La représentation privée fermée RoutingScoreWeights réemploie cette politique. Ses valeurs
défaut sont omises du JSON canonique pour préserver le digest des anciens snapshots168 ;
les valeurs non défaut restent figées. Aucun service ni couche de classement supplémentaire.
La capacité compilation influence la préférence seulement avec AutoDelegateV2.workload=compilation.
workload est un champ fermé general|compilation, général par défaut, hors HandoffV1 dont le schéma
est fermé. Figer ce champ dans le canon, le snapshot et la requête de décision ; aucun arbre
« présentation→Claude » ni extraction de mot-clé ou ajout au brief hérité.
Les quotas attestés bloquants sont filtrés par les gardes157/160 ; quota inconnu n’est pas autorisation.
Ajouter un bonus borné de quota restant0..5 et de reset proche0..3 sur fenêtres attestées fraîches,
à pertinence et charge proches. Aucune somme entre comptes, coût prédit ou lecture de solde.
Agrégation explicite des seules fenêtres connues, sans prétendre connaître une fenêtre absente.
À égalité, ordre stable par tuple machine/moteur/connexion/modèle, indépendant de l’ordre du registre.

Le score classe des possibilités, pas des réservations. Le puits final recontrôle ses limites.
Un changement après choix ne lance pas une seconde décision ni un second candidat.

### 3. Chaîne de décision unique

Ordre : filtrage/score → service163 si autorisé et disponible → petit décideur168 si autorisé
et disponible → meilleur candidat du score déjà figé.
Un candidat unique ou des axes tous fixés évitent les consultations.
Le résultat d’un décideur porte un ID opaque de candidat et un effort compatible ; un effort imposé
ne peut pas changer. Sans effort explicite du décideur, employer le défaut attesté du modèle.
Le format évite de multiplier les candidats pour chaque effort.
Un niveau ne reçoit jamais une liste partielle choisie pour lui faire tenir une limite.
La requête agrège les ressources par machine et les quotas par connexion ; chaque candidat
référence ses faits et expose le vrai modèle et les efforts compatibles. Les IDs opaques restent
les seules références de sélection ; ni chemin d’authentification ni identité privée de compte.

Réutiliser le service HTTP163 et ses gardes de santé/URL/borne/permis.
Pour Laya, conserver les bornes163 et adapter explicitement les validateurs168 du constructeur
et parseur (decision_service.rs autour de462/602) aux IDs v2. Employer un encodage compact sans perte,
modèles≤32octets et état agrégé, avec tous les faits FR021 de la liste entière. Si l’un de ces faits
ne tient pas, ignorer le niveau avec motif de capacité ; aucune coupe, omission de faits ou promesse Laya128.
La recette de service porte sur un ensemble entier qui tient, ≤8au départ ; douze options ou plus
sont qualifiées par le petit décideur réel et le score, avec skipLaya explicite si nécessaire.
Aucune obligation d’un serveur nommé Laya : sa sélection vient de la configuration.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-6).

Budget de chaîne proposé10s, configurable1..30s, service2s au plus par défaut,
petit décideur dans le restant (8s par défaut), sans dépassement de la deadline commune.
La correction T038 transporte le même Instant absolu depuis la frontière d’admission de la chaîne,
y compris attente mutex, préparation du décideur et scheduling ; aucun Duration ne recrée un budget.
Le cycle de vie gère annulation, drain et arrêt du groupe ; une échéance de demandeur ne libère
pas à tort un permis avant fin réelle. Aucun nouveau processus tant qu’un précédent appel occupe le permis.
Préflight Git/probe reste séparé et borné par ses limites existantes ; le budget de décision n’est pas un SLA Git.

### 4. Reçu, idempotence et reprise

Étendre le snapshot des reçus existants avec metadata_version=2 ; absence de version =163.
Snapshot168 ≤128Kio, canon complet≤512Kio, sans hausse des limites transport16Mio/2Mio.
Conserver la requête canonique une seule fois ; référencer les connexions et faits utiles sans environnement secret.
Le mode SQL reste explicit|automatic|fallback. La source détaillée est en metadata v2 :
explicit/single→explicit ; decision_service/lightweight_model→automatic ; deterministic→fallback.
Persister politique, matrice, score, premier repli admissible et réglages des décideurs avant consultation.

États existants pending/decision_started/chosen/admitted/refused et révisionCAS conservés.
Le niveau courant est une métadonnée versionnée, sans table ni ordonnanceur nouveau.
Avant de passer au petit décideur, confirmer la transition CAS depuis le niveau service en vol.
Une seule incarnation possède la chaîne ; le concurrent reçoit admission_pending.
La réponse tardive d’un niveau ne remplace ni le niveau suivant ni le repli déjà choisi.
Après crash, l’entretien existant n’appelle aucun décideur ; il revalide et admet le repli figé
ou conserve le refus/diagnostic approprié. Une tâche déjà choisie garde son canon exact.
Les erreurs techniques restent récupérables ; seuls les codes métier fermés clôturent un refus.

### Migration froide précise de la borne SQL

Le schéma actuel impose metadata BLOB CHECK(length BETWEEN1 AND16384).
Changer seulement le validateurRust ne suffit donc pas.
Au démarrage et dans la transaction existante, inspecter le schéma de auto_admission_receipts.
Une base neuve naît directement avec le DDL168, avant toute admission ; elle ne crée pas d’abord
une table163 à migrer. Si metadata_version est absente et la table correspond au schéma163 connu :
1. créer auto_admission_receipts_168_new avec les mêmes colonnes/contraintes,
   metadata_version INTEGER NOT NULL DEFAULT1 CHECK(metadata_version IN(1,2)),
   et CHECK(length(metadata) BETWEEN1 AND CASE metadata_version WHEN2 THEN131072 ELSE16384 END) ;
2. copier par INSERT…SELECT les colonnes explicitement, avec metadata_version=1 pour tous
   les anciens reçus ; conserver les BLOB canonical/metadata/delegate_canonical et tous les états octet-identiques ;
3. remplacer la table et recréer les trois index maintenance/task/refused_purge existants
   dans la même transaction, puis commit. Aucune seconde table métier ne subsiste après commit.

Si le schéma168 est déjà présent : lecture seule, migration idempotente.
Schéma inconnu, dépendanceFK/trigger inattendue ou échec de copie/index : rollback et diagnostic,
jamais réécriture writable_schema, suppression de données ou désactivation forcée des contraintes.
La copie SQL est froide et bornée par la rétention ; ne pas charger tous les BLOB en mémoire.
Après migration : quick_check, foreign_key_check et comparaison du nombre de lignes,
clés/états et SHA des BLOB legacy sur fixture persistante. Tester aussi un rollback déclenché pendant la copie,
réouverture et rejeu163 inchangé, réouverture168 avec snapshot>16Kio, refus>128Kio,
rétentionUNION et CAS après réouverture.
La sauvegarde cohérente avant activation et les gardes de rollback du chapitre livraison restent obligatoires.
La version du snapshot et la colonne doivent concorder ; un reçu163 ne passe jamais à128Kio par simple absence de version.

### 5. Compatibilité et façade

AutoDelegateV1, probev1 et Lineagev1/v2/v3 restent strictement conservés.
Ajouter AutoDelegateV2 dans la façade existante avec connection_id facultatif et axes indépendants,
workload fermé et expected_base_commit facultatif à la forme mais requis dans tout espace Git168.
Shared comme Worktree exigent le SHA source exact fourni par le coordinateur ; absence =
workspace_base_required avant tout fournisseur. Worktree.base_ref résolue doit égaler ce SHA.
Pour Shared horsGit et workload=general seulement, la base peut être absente. La CLI peut résoudre
la base source localement avant de créer l’enveloppe ; le daemon ne choisit jamais le HEAD distant.
Porter expected_base_commit au snapshot et probev2, puis au champ base_commit de WorkerPrepare
existant ; réemployer verify_base_commit et son contrôle frais avant création/lancement.
Le client doit négocier auto_delegate_v2 avant envoi ; absence = capability_unavailable précis.
Une requête163 demeure recevable sur son chemin163 ; aucune conversion silencieuse d’une demande168.
Le nouveau client d’admission automatique CLI/MCP émet toujours AutoDelegateV2, sans drapeau
supplémentaire et sans choisir la version selon ON/OFF. workload=general n’est pas un commutateur.
Sans politique168 active, une nouvelle enveloppeV2 reçoit une erreur structurée ; jamais un repliV1.
« Absence168 garde163 » signifie que le daemon sert encore les anciennes enveloppes163.
FR-034 conserve le chemin explicite bridget_delegate/NativeDelegate et AutoDelegateV1 inchangés.
Le canon d’un même request_id ne change pas de version lors d’un toggle de configuration.


WorkerCandidateProbeV2 lie jusqu’à128IDs et un snapshot hôte à l’époque négociée.
Les anciens workers reçoivent seulement leur protocole ancien.
Un worker sans capacité168 est exclu avec motif explicite pour une demande168 ; il reste utilisable en163.
Le mode de transport ne reçoit aucun secret de connexion : l’ID est résolu dans le registre de la machine.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-7).

## Répartition des futures surfaces d’écriture

| Lot | Ownership proposé et responsabilités |
|---|---|
| A — Politique et catalogue | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs ; raccord des connexions aux lanceurs, validation128/axes, modèles réels et snapshots publics sans secret. |
| B — Wire, worker et faits hôte | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs ; fixtures162. Contrats additifs et probe groupé ; aucun Prepare pour classer. |
| C — Admission, store et projections | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs ; lineage/CLI/MCP existants. Score, chaîne et CAS communs. |
| D — Transport décideur | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/managed_session.rs. Mode sans outils, deadline et arrêt ; pas de Task métier. |

Un fichier n’a qu’un écrivain à la fois. Les types du lotB sont publiés avant les consommateurs.
Les interfaces partagées sont arbitrées par ROOT ; aucune régénération massive de module.
C possède aussi /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lib.rs pour les seuls enregistrements nécessaires.
D possède aussi /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lib.rs pour les seuls exports du mode isolé.
B peut créer /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/host_resources.rs après gate anti-doublon R09 ; garder son collecteur existant pub(crate) si plus simple, C enregistre le module.
ROOT seul intègre, pousse, provisionne et active. Pas de nouveau profil par modèle.

## Ordre d’implémentation et validation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-8).

Les fixtures ne prétendent pas authentifier un fournisseur réel.
Un seul compilateur Cargo sur Cartae, CARGO_BUILD_JOBS=2 et deux threads de tests.
La campagne workspace finale est lancée une seule fois après stabilisation, jamais avant codage ;
un échec préexistant est comparé et documenté, pas masqué par un GREEN global.

## Activation et retour arrière

Inventorier les modèles réellement disponibles de chaque connexion Mac/Cartae avant d’activer168.
Préserver les environnements Codex distincts déjà provisionnés ; aucune copie de credential.
Claude est déjà installé et authentifié sur Cartae selon ROOT (2.1.251, compte max) ;
aucun nouveau login/installation n’est requis pour cette connexion. Déclarer ses capacités et vérifier
ses permissions réelles. Les flags du décideur doivent être prouvés aussi sur cette version,
pas seulement sur le Mac2.1.300. Un second environnement Codex Cartae n’est pas attesté :
sa déclaration est supportée mais son activation exige une authentification native autorisée hors Bridget.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-9).

Dès qu’une ligne v2 existe, un ancien binaire est interdit même si168 est OFF.
Retour arrière sûr : garder le binaire compatible168 et désactiver les nouveaux automatismes,
ou restaurer un ensemble cohérent seulement si aucune nouvelle écriture n’est perdue.
Récupérer et intégrer les commits avant retrait des worktrees ; contrôler SHA et absence de processus.
Conserver preuves, rollbacks, bases et travaux étrangers.

## Constitution après conception

Les gates initiaux restent satisfaits. Les ajouts de version sont nécessaires aux schémas fermés existants.
Le mode décideur ajoute une frontière de sûreté requise ; il réemploie les transports au lieu d’un engine.
Le coût cognitif est limité aux versions de contrats et au classement explicite.
Potentiel minimalisme à vérifier en audit : zéro service ou dépendance supplémentaire prévu ;
supprimer toute abstraction qui ne porte ni validation, ni isolement, ni compatibilité.
Aucune réalisation ou réussite de tests n’est déduite de ce plan.


## Précisions ROOT après revueR2 — contrat des tâches

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/168-routage-flotte/plan.md#extrait-10).


## Analyze avant implémentation — corrections A1 à A4

L’ordre final suit les dépendances et non les numéros conservés :
ciblés→contre-revue/Converge→audit et corrections→FULL unique→docs/livraison.
A/B/D démarrent leurs surfaces indépendantes après les types B publiés, sans attendre le store E.
Les consommateurs attendent une publication d’interface/code, pas le cochage d’un oracle encore différé.
Les tâches de test restent non cochées jusqu’à preuve ROOT T035 ou permis explicite plus tôt.
Cette preuve différée n’est pas un succès et Analyze ne donne aucun permis Cargo automatique.


## Précisions issues de l’audit T038 — arbitrages initiaux conservés

- Premier PrepareV2 Worktree : garder le workspace et sa base future exacte, inspect_only=true
  (false par défaut, V2 seul). Aucun passage artificiel en Shared qui comparerait au HEAD distant.
  WorkerPrepared renvoie le token réservé inspect-only-v1 sans état Prepared, worktree ou slot ;
  Start le refuse. Second Prepare normal uniquement après Task durable.
- Probe : mémoriser les seuls faits Git, erreurs comprises, par tuple distinct dans le lot ≤128.
  Les droits, définition, connexion, modèle et effort restent individuels. Prepare et Start
  revalident fraîchement l’élu ; aucun cache inter-lots ou de reconnexion.
- Seconde sonde pour effort : comparer définition brute, époque et mapping à la première
  observation fraîche ; seul le digest de droits peut varier légitimement avec l’effort.
- Échecs RED observés et justifications dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md.
  Ces arbitrages initiaux ne valaient pas validation. Les corrections, contrôles affectés,
  revue du delta et export final sont désormais consignés dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/audit168.md.
  La clôture T038 reste distincte de l’activation et de la livraison futures.
