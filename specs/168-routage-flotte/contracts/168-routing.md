# Contrats SPEC168 — Routage de flotte

Statut : conception validée pour préparation ; types et exemples à transformer en code après audit et Tasks.
Toutes les nouvelles structures sont fermées et bornées. Les contrats163 restent inchangés.

## 1. Configuration privée additive

La nouvelle section optionnelle routing_policy porte version=2, revision, enabled,
connections, model_catalog et machine_policies. Elle ne remplace pas les agents/lanceurs existants.
Sans cette section, le daemon continue à servir les anciennes enveloppes163.
Une nouvelle enveloppeV2 sans politique168 active reçoit une erreur structurée, sans rétrogradationV1.
Un objet168 inconnu ou incomplet refuse sans effets.

Une connexion : connection_id, machine_id, engine, agent_type existant, enabled, display_name facultatif.
Aucun token, password ou fichier auth n’entre dans cette section ; son environnement privé demeure dans le lanceur.
Le catalogue par connexion liste les vrais IDs, efforts compatibles, défaut résoluble et provenance.
Le champ default_effort est obligatoire : membre de supported_efforts si des niveaux existent ;
sinon null avec liste vide signifie défaut natif sans flag et n’exclut pas le modèle.
Un effort imposé est alors incompatible. Aucun défaut medium ni niveau universel ne sont inventés.
Codex model/list complète la déclaration quand disponible ; son résultat n’est pas une garantie d’accès.
Un modèle absent du catalogue local de la connexion ne peut pas être introduit depuis un autre compte.

Une option admissible porte machine/moteur/connexion/modèle et la liste des efforts compatibles.
Ne pas multiplier les profils ou options par effort. Le choix final résout l’effort parmi cette liste.
Plafond global statique :128options configurées uniques avant filtres de demande, sans troncature.
Les modèles réellement disponibles doivent rester tous représentés ; une configuration trop large refuse
candidate_limit au lieu de masquer des modèles. Pas de nom Terra/Luna/Sol/Haiku/Sonnet/Opus deviné.

Chaque machine du coordinateur dispose de :
- enabled et automatic_axes : engine, model, machine, connection, effort ;
- fixed_values pour les axes non automatiques ;
- RoutingMachinePolicy.compilation_capacity: Option<u8>, entier0..10 ou null, défautNone ;
  capacité déclarée simple, jamais un benchmark ni une mesure des compilations étrangères ;
- telemetry enabled, max_age_secs défaut30/borne1..300, seuils et poids validés ;
- decision_service enabled, référence de service et accessible depuis cette machine ;
- lightweight_decision enabled, engine=claude, connection_id local, model réel, effort compatible ;
- decision_timeout_ms défaut10000/borne1000..30000 ; service_timeout_ms défaut2000 ;
  le petit décideur emploie le restant, lancement du processus inclus.

Le décideur est un rôle configuré distinct de l’exécutant. Il n’est pas lié à la machine retenue pour la tâche.
Niveau désactivé/inaccessible = zéro tentative. La configuration ne crée aucun compte et ne contourne aucune politique administrée.
Le registre ne contient aucun arbre de mots-clés vers un fournisseur.
Laya peut être déclaré sur une machine hébergeuse distincte ; c’est l’accessibilité depuis le coordinateur qui décide de son usage.

## 2. Façades native, MCP et CLI

Négocier auto_delegate_v2 dans les features avant envoi d’une demande168.
La façade CLI delegate et l’outil MCP auto-delegate existants acceptent la version168 ;
pas de commande parallèle ni de nouvel engine.
La charge native AutoDelegateV2 conserve les champs métier v1 et ajoute engine, connection_id
et expected_base_commit optionnels dans la forme, et workload fermé general|compilation, défautgeneral.
Normaliser workload dans le canonV2, le snapshot et la charge de décision. HandoffV1 et les champs
du brief hérité restent strictement inchangés ; aucun ajout workload ni classification par mots-clés.
Le discriminant natif operation=auto_delegate_v2 sélectionne ce schéma ; pas de champversion redondant.
L’opération163 reste operation=auto_delegate. Aucun champ168 n’est envoyé au typev1 fermé.
Engine et agent_type interne ne sont pas confondus : une contrainte moteur ne choisit pas un compte.

machine_id, model, engine, effort et connection_id sont des filtres indépendants.
Un axe absent suit sa politique ; un axe imposé n’est modifiable par aucun niveau.
Contradiction de filtres = aucun lancement, aucune tentative modèle pour « réparer » le choix.
Une machine imposée saturée produit resource_saturated sans changement silencieux de machine.
Les champs historiques reprennent leur sémantique exacte, notamment workspace et request_id.

Sans capability168 : capability_unavailable structuré pour une demande168.
Une ancienne demande163 garde AutoDelegateV1 et son chemin ; aucune dégradation silencieuse d’une demande168.
Le nouveau client CLI/MCP d’admission automatique émet toujours V2, sans flag supplémentaire
et sans basculer selon ON/OFF. Le défaut workload=general ne sélectionne pas une version.
FR-034 vise le comportement explicite bridget_delegate/NativeDelegate et le wire AutoDelegateV1
inchangés. Il ne promet pas de convertir la nouvelle façade automatique en163 quand168 est OFF.
Cette règle garde la version du canon stable pour un même request_id malgré un toggle.
Aucune API de politique active ni feature supplémentaire n’est introduite.

Commande exacte de lecture : bridget delegate --catalogue --json [--machine ID].
Sans --json, afficher une table lisible avec les mêmes faits. Aucun argumenttask ni lancement.
Opération native décidée : routing_catalogue_v1, sur le même canal, exclusivement en lecture.
Exiger Role::Client, version négociée et UID du pairUnix égal à celui du daemon : unix_peer_uid
(Linux/Mac), peer_uids de handle_connection et pattern de WakeAction::CliOnce.
Cette garde accepte l’humain local sans session d’agent ; elle n’utilise jamais human_authority_guard,
qui dépend de T3. Aucun --from arbitraire, endpoint réseau nouveau ou autorité de mutation.
La projection est publique et allowlistée : version=1, observed_at, options et inconnus.
Un registre dépassant128est refusé, jamais un succès partiel complete:false.
Chaque option montre engine, model réel, machine_id, connection_id, supported_efforts/default_effort,
provenance/âge et entitlement=unknown pour cette simple lecture. Ne pas exiger un appel à chaque
modèle pour qualifier le catalogue ; un refus fournisseur final est rendu explicitement sans bascule.
Un profil nommé pro n’est jamais le nom public du modèle.
La lecture ne lance pas un fournisseur pour rafraîchir son catalogue : configuration et observations déjà obtenues seulement.
Le catalogue168 est versionné et rend128options au maximum, connexions et faits sans secret.
Une ancienne lecture163 conserve son schéma ; elle n’est pas présentée comme inventaire complet168.

## 3. Probe worker168

Ajouter CandidateProbeV2 et son résultat à la négociation worker existante,
avec WorkerCapability::CandidateProbeV2 et feature worker_candidate_probe_v2.
N’ajouter aucun champ non négocié à Hello/Accepted v1 ni au probev1.
Les nouveaux messages gardent les identités, admission_id, correlation_id, révision et connection_epoch du probe163.
Le lot porte1..128IDs uniques c001..c128 ; le résultat donne exactement un verdict par ID demandé.

Variants top-level proposés : DaemonToWrapper::CandidateProbeV2 et WrapperToDaemon::CandidateProbeResultV2,
version=2, même handshake de capacités séparées que163 et aucun messagev2 à un pair sans négociation.
La sonde transporte les références de connexion, leurs modèles/efforts et un snapshot hôte commun au lot.
Les références sont résolues localement ; aucun chemin de credential ne circule.
Époque, worker, admission et ensemble d’IDs doivent correspondre exactement.
Un ancien worker sans cette capacité n’est pas sondé au format168 et n’est pas faussement déclaré admissible168.

Le probe partage la lecture de Prepare sans effet : aucun worktree, slot, tokenPrepared, fournisseur ou Task créé.
Pour tout espace Git168, Shared ou Worktree, expected_base_commit source exact est requis :
champ absent = workspace_base_required avant fournisseur ; SHA indisponible/inexact = refus
workspace_base_unavailable. La CLI peut résoudre le SHA sur la base source du coordinateur
avant l’enveloppe ; le daemon ne lit jamais le HEAD distant comme base implicite.
Worktree.base_ref résolue doit produire le même SHA ; une divergence refuse.
Shared horsGit et workload=general peuvent omettre la base, sans inventer un dépôt.
Le snapshot et probev2 portent expected_base_commit ; le maître le transmet dans WorkerPrepare.base_commit
existant et réemploie verify_base_commit, y compris au recontrôle frais avant lancement.
Le coordinateur peut préparer le SHA auparavant avec un fetch autorisé ; le daemon ne le fait pas.
Le Prepare final recontrôle droits, mapping, définition, base et capacité après choix.

### Recontrôle final de la connexion — arbitrage T024

WorkerPrepare v1 ne porte pas connection_id. Son réemploi seul ne peut donc pas
attester la même connexion après révocation si plusieurs connexions partagent agent_type.
Ajouter sur le canal existant l’opération négociée et fermée :
WorkerRequestOperation::PrepareV2 { prepare: WorkerPrepare, workload: AutoDelegateWorkload, connection_id: String,
engine: String, expected_definition_digest: String, expected_permission_digest: String, inspect_only: bool }.
inspect_only est facultatif en lecture, false par défaut, et propre à V2 ; Prepare v1 reste inchangé.
Modèle, effort et base restent dans prepare ; aucune duplication du lanceur ou de Prepare.
workload est obligatoire dans PrepareV2 et conserve la valeur du snapshot/probe.
Seul Shared hors Git et general peut garder baseNone ; Worktree ou compilation exigent Git
et la base source vérifiée. Les protections de chemins, droits et mapping sont réemployées,
sans nouvelle configuration de mapping.

Le maître C transmet à PrepareV2 le tuple et les digests attestés puis figés au probe. Le worker réemploie les gardes
CandidateProbeV2, résout sa connexion locale et relit tuple/droits/digests avant tout effet.
Connexion révoquée ou tuple/digest divergent refuse sans worktree, tokenPrepared, slot ou fournisseur.
Si l’effort choisi exige une seconde sonde, définition brute, époque et mapping de celle-ci
doivent rester identiques à la première observation fraîche validée. Le digest des droits peut
changer avec l’effort ; ce changement ne permet pas de changer de connexion ou de session.
Aucun PrepareV2 vers un pair sans négociation168 ; Prepare v1 et ses anciennes demandes restent inchangés.
Cet ajout reste dans T024 ; aucune nouvelle tâche ni nouveau workflow.
Correction d’audit T038, validation encore future : le premier PrepareV2 Worktree conserve
workspace, base_ref et SHA attendus, avec inspect_only=true. La base future est vérifiée
sans la comparer au HEAD d’un Shared de substitution. WorkerPrepared retourne alors le
token réservé inspect-only-v1, sans état Prepared persistant, worktree, slot ni fournisseur.
Start refuse explicitement ce token : cette inspection ne donne aucun droit de lancement.
Après écriture durable de la Task, le maître envoie un second PrepareV2 normal
(inspect_only=false) ; les gardes fraîches de création/Start restent applicables.


## 4. Ressources et score

Les faits hôte sont CPUbusy, RAMdisponible/totale, charge par CPU et capacité compilation déclarée.
Chaque fait porte provenance, date source et réception maître. Valeur invalide/périmée = null.
La mesure CPU sans deux échantillons valables reste null. Mémoriser le dernier échantillon et
rafraîchir au heartbeat existant et au probe, une collecte par machine/lot ; pas de nouveau daemon.
Aucun script provenant de la tâche ou du décideur.
Les faits sont agrégés une fois par machine/connexion ; éviter N+1 et répétitions par modèle.
Dans un lot de probe de 128 options au plus, seuls les faits Git sont mémorisés par tuple
distinct : worker/époque/provision, projet/révision de mapping, racines/cwd résolu/identité Git,
workload, stratégie workspace/base_ref/branche et SHA attendu. Réussites et erreurs sont
mémorisées pour ce seul lot. Connexion, modèle, effort, définition et droits restent contrôlés
pour chaque candidat. Aucun cache ne remplace la revalidation fraîche au Prepare/Start.

Filtre préalable : une machine franchement saturée est retirée si une autre machine admissible
est attestée disponible. Seuils défaut : CPU≥90%, RAMdisponible≤10%, charge normalisée≥1.
Ce filtre précède les deux décideurs ; une réponse visant une option retirée est invalide.
Le choix déjà durable n’est jamais recalculé. La revalidation opérationnelle porte seulement
sur cette cible : droits, capacité et base doivent encore permettre son lancement. Une saturation
survenue après le choix peut donc refuser ce lancement, sans choisir une autre machine ni redécider.
Si la machine est imposée saturée, refuser explicitement plutôt que basculer.

Sur les candidats restants, score entier proposé :
- disponibilité CPU connue : arrondi(20×(1−CPUbusy)) ;
- RAM disponible connue : arrondi(20×RAMdisponible/RAMtotale) ;
- charge connue : arrondi(20×(1−min(charge_normalisée,1))) ;
- continuité dépôt/cache : total0..20 ; compilation déclarée compatible :0..10 ;
- quota connu restant :0..5 ; reset proche attesté :0..3 ;
- saturation fraîche non comparable à une alternative attestée : pénalité100.
Chaque terme inconnu vaut0 et garde sa provenance unknown. telemetry.weights contient les poids
cpu/ram/load/continuity/compilation/quota/reset, entiers bornés respectivement à20/20/20/20/10/5/3,
avec ces mêmes valeurs par défaut. Les poids de la machine coordinatrice sont appliqués uniformément
à toutes les cibles et figés dans politique/snapshot ; seuils et fraîcheur restent propres à chaque cible.
RoutingScoreWeights est la représentation privée fermée de ces sept coefficients dans la politique.
Les seuils préexistants ne sont pas des poids. Quand les sept valeurs sont celles du défaut,
weights est omis du JSON canonique : un ancien snapshot168 conserve ainsi son digest de politique.
Des coefficients non défaut restent sérialisés et figés dans le snapshot/digest. Aucun service,
champ de télémétrie réseau ou système de score parallèle n’est ajouté.
Les termes CPU/RAM/charge sont arrondis au plus proche ; la pénalité de saturation100 reste fixe.
Aucun dépassement entier. Le défaut de CPU=unknown ne signifie pas CPU=0%.
Le bonus quota vient des seules fenêtres connues de la connexion, sans somme entre comptes ;
un reset≤1h peut donner le bonus, son absence0. Aucune estimation financière.
Ordre secondaire stable : machine/engine/connection/model. Effort par défaut attesté, sauf axe imposé.

La compilation est le champ AutoDelegateV2.workload=compilation, pas un champ du briefHandoffV1
fermé ni une classification par mots-clés. general est le défaut explicite du nouveau contrat.
La capacité déclarée ne prétend pas observer toutes les compilations étrangères.
Le score reste inspectable avec ses termes, et non une probabilité de performance du modèle.

## 5. Requête normalisée aux décideurs

Une seule charge sémantique figée est réemployée : brief utilisateur borné, workload, urgence/livrable existants,
contraintes imposées, machines agrégées, connexions agrégées et candidats.
Machines : ressources connues/inconnues, âge, continuité et capacité déclarée.
Connexions : quotas/crédits attestés utiles, âge, sans email, solde opaque ou environnement.
Candidats : ID opaque, vrai model, engine, références opaques de machine/connexion,
supported_efforts/default_effort, capacités déclarées utiles, score et rang déterministes.
Les chemins privés, permissions détaillées, prompts de projet, tokens et définitions d’exécutables restent absents.

La charge normalisée168 est bornée64Kio ; le brief utilisateur reste une donnée inerte bornée,
sans copie intégrale implicite de la mission. Bridget n’injecte automatiquement aucun secret de
configuration/authentification ou chemin privé ; il ne promet pas de scanner parfaitement les secrets
que l’utilisateur aurait lui-même placés dans son brief.
Chaque adaptateur vérifie aussi ses bornes propres. Pas de troncature silencieuse, top8 ou tournoi.
Laya163 conserve notamment JSON4096/state1536/description32. Adapter les validateurs du constructeur
et parseur pour168 (decision_service.rs autour de462/602), sans élargir ou modifier163.
Le format compact sans perte emploie modèles≤32octets, IDs v2 et état agrégé ; tous les faits FR021
sont conservés pour chaque niveau réellement consulté. Si l’un ne tient pas,
le niveau retourne decision_capacity_unavailable et la chaîne continue, sans omission.
La recette du service choisit réellement sur un ensemble entier compatible (≤8conservateur),
un codeHTTP200 santé seul ne suffit pas. La matrice12+entière est prouvée par petit décideur réel
et score, avec skipLaya explicite si sa charge dépasse ses capacités ; aucune promesse Laya128.
La sélection d’effort Laya doit être normalisée depuis une sortie attestée ; en son absence,
garder le défaut autorisé du candidat, sans annoncer une décision d’effort du service.

Sortie normalisée fermée :
`{"candidate_id":"c012","effort":"low","reason":"capacité et disponibilité adaptées"}`.
reason est facultative, affichable1..256octets ; effort facultatif seulement si un défaut attesté existe.
ID hors ensemble, effort non compatible/imposé différent, clé inconnue, réponse multiple ou trop grande = niveau invalide.
La réponse ne peut contenir ni commande, URL, droit, nouveau modèle ou compte.
Les descriptions et le brief sont des données inertes, pas des instructions autorisées.

## 6. Petit décideur natif

Premier adaptateur : ClaudeStreamJsonTransport existant avec profil interne de décision.
Exiger le support réel des flags sur chaque binaire installé : --safe-mode, --restricted, --tools "",
--strict-mcp-config avec configuration vide, --setting-sources "", --disable-slash-commands,
--no-session-persistence et --json-schema en mode print/stream-json réel.
--safe-mode est attesté par les aides locales Mac2.1.300 et Cartae2.1.251 : il retire
CLAUDE.md/skills/plugins/hooks/MCP/personnalisations mais conserve auth et politiques administrées.
--bare est interdit pour ce rôle OAuth : l’aide des deux versions limite son auth à APIkey/helper.
--disallowedTools "*" peut renforcer les gardes si son support est attesté, sans être leur unique protection.
Pas de --dangerously-skip-permissions, pas de nouvelle cléAPI requise.
Les politiques administrées restent applicables ; si l’isolement requis ne peut être attesté, niveau indisponible.
Aucune injection MCPBridge/T3, aucun add-dir, hooks/skills/plugins/projet/autoreprise.
Environnement enfant env_clear puis liste minimale issue de la connexion autorisée,
cwd neutre privé et pas de session_store_root/resume de l’exécutant.
Les écritures natives auth/state du moteur restent possibles ; aucune écriture métier ni outil.
Un prompt « ne pas utiliser les outils » seul n’est pas une protection suffisante.
Exception de formatage strictement bornée : le seul outil natif StructuredOutput imposé par
json-schema est autorisé, sans droit métier/système/réseau/MCP ni droit d’exécution.
Le mode rejette toute autre capacité annoncée, tout autre outil ou tool_use. Un tool_use
StructuredOutput doit porter uniquement l’objet fermé prévu, validé localement parmi les ID/efforts
autorisés ; sa sortie structured_output et son resultJSON doivent concorder. Aucune instruction
ni clé supplémentaire ne gagne d’autorité par ce formateur. La qualification CLI observée
a init.tools=[StructuredOutput], pas init.tools=[] ; le transport168 reste à qualifier.

Sortie normalisée finale≤4096octets et somme de fluxstdout≤64Kio, bornées avant parsing.
stderr n’est pas conservé : aucune copie, journalisation brute ou nouveau tampon stderr.
Dépassement stdout : arrêter le niveau et continuer au score sans stockage de flux illimité.
Réemployer les bornes et parseurs de transport existants autant que possible.
Activer seulement serde_json/raw_value déjà disponible dans le Cargo.toml transport pour
borner l’objet JSON brut à4Kio avant son parsing détaillé. Aucun nouveau package ni parseur
artisanal ; l’API publique du décideur D reste inchangée.

Réemployer ManagedSession drain/cancel/stop et les groupes de processus existants.
Pas de Task, request_id secondaire d’admission, sous-agent ni scheduler pour ce rôle.
La chaîne et le processus partagent le même Instant absolu, créé à la frontière d’admission
de la chaîne avant préparation du petit décideur. Attente du mutex, préflight et scheduling
ne recréent aucun budget ; entrée, préspawn et I/O vérifient cette même échéance.
L’attente du demandeur reste bornée et le permis est gardé jusqu’à l’arrêt réel.
Processus absent, quota, auth, sortie invalide ou timeout rendent le niveau indisponible ;
le score final reste disponible. Un retour tardif ne gagne jamais un CAS après passage au niveau suivant.

## 7. Reçu et compatibilité durable

Snapshot metadata_version=2 ≤128Kio, canon≤512Kio, IDc001..c128 validé selon version.
Métadonnées163 absentes/v1 ≤16Kio et IDc01..c08 inchangés.
Un reçu168 fige liste entière, faits utiles, score, repli, connexion/définition et politique avant consultation.
workload et expected_base_commit font partie du canon et du snapshot168. Le mode SQL demeure
explicit|automatic|fallback : explicit/single→explicit, decision_service/lightweight_model→automatic,
deterministic→fallback. La source détaillée est dans metadata v2, pas dans une valeur SQLnouvelle.
Même owner/request_id = même canon ou envelope_mismatch ; collision native/auto reste atomique.
La CLI delegate fige le HEAD courant de la source dans expected_base_commit. Après un nouveau
commit, la même request_id porte une autre enveloppe : envelope_mismatch est attendu, même si
la tâche précédente existe. Retrouver son état et son résultat par bridget task status, sans la redéléguer.
Une seule chaîne par clé : service au plus une fois, petit décideur au plus une fois.
Le rejeu et la reprise après panne n’appellent aucun des deux.
La transition entre niveaux et l’écriture chosen portent la révisionCAS/époque/niveau attendu.
Le snapshot C contient metadata.decision: Option<LineageDecisionV4> : None avant choix,
puis l’objet complet fermé au choix, avec source et combinaison machine/moteur/connexion/modèle/effort.
choose_auto_admission_at reçoit chosen_metadata: Vec<u8> juste avant now.
Le store valide metadata v2 et le niveau inchangé, puis persiste ces métadonnées et Chosen
dans le même CAS. Aucun instant confirmé ne sépare choix et métadonnées de projection.
Aucune nouvelle colonne ni table n’est ajoutée pour decision ; C conserve les faits métier
du snapshot, E contrôle la forme/version/niveau et l’atomicité.

Task et lien reçu sont committés ensemble ; pas d’effet en cas de rollback.
La base neuve crée directement le DDL168 avant flux admission ; une base163 utilise la migration
transactionnelle avec copieBLOB exacte et recréation des index. Toute lignev2 interdit un ancien
binaire, même avec168OFF ; garder un binaire compatible et désactiver168 ou restaurer sans perte.
Les erreurs techniques/SQL conservent le dernier état confirmé ; ne pas les terminaliser par parsing de texte.
Annulation, pause, limites actives, garde quota enfant et rétention existantes restent l’autorité.

## 8. Lineage et lecteurs anciens

Ajouter list/show v4 avec decision168 versionnée :
mode, candidate_id, connection_id, engine/model/machine/effort, source de décision et faits affichables bornés.
Pas d’environnement privé, email ou chemin de credential.
Tâche168 lue en v3 : placementv2 habituel et decision:null.
Tâche163 lue en v3 : décisionc01..c08 inchangée.
Les lecteurs CLI peuvent négocierv4 ; le repli ne masque que le refus de version structuré attendu.
Une ancienne lecture reste possible sans updateT3, sans inventer une décisionv3 pour168.
Les mutations journal/cancel et l’autorité de leur parent restent inchangées.
