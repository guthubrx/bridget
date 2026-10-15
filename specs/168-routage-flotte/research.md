# Recherche SPEC168 — Routage de la flotte

Date : 2026-10-15. Statut : recherche et décisions de conception ; aucune implémentation ni activation168 prouvée.
Base : 1f69e5d8e651dd27907fd244aee6f90b6b734565.

## Procédure et baselines consultées

Les skills speckit-specify et speckit-plan ont été lues dans /Users/moi/.agents/skills.
Le worktree Cartae ne contient ni templates/scripts officiels utilisables, ni extensions.yml, ni standards projet.
Le protocole de secours conserve les sections obligatoires et écrit les artefacts à la main, sans installer de framework.
Le chemin de feature est persisté dans .specify/feature.json ; AGENTS reçoit seulement la référence du plan.
Le manque de script ne justifie pas la recréation des répertoires gérés par SpecKit.

L’index de standards Mac /Users/moi/Nextcloud/10.Scripts/64.bridget/.specify/memory/standards.md
renvoie aux baselines consultées :
- /Users/moi/.speckit/research/01-ai-agents-agentic-ai.md
- /Users/moi/.speckit/research/04-architectures-patterns.md
- /Users/moi/.speckit/research/08-testing-quality.md

Ces baselines sont historiques, datées du 5 décembre2025. Elles orientent les questions ;
elles ne prouvent ni les modèles installés ni un contrat fournisseur courant.
Recherche de validation exécutée : « multi-agent architecture failure modes debugging »,
complétée par les sources primaires ci-dessous. Aucun framework multi-agent supplémentaire n’est retenu.

## Sources primaires consultées aujourd’hui

| Source | Fait utile | Conséquence |
|---|---|---|
| https://learn.chatgpt.com/docs/app-server | model/list fournit modèles et efforts dépendant du compte/client ; account/read et account/rateLimits/read sont distincts du démarrage d’un tour. | Réemployer la découverte bornée existante ; ne pas coder une liste universelle de noms ni prendre le catalogue pour une preuve d’accès. |
| https://learn.chatgpt.com/docs/auth | Les identifiants peuvent être conservés sous CODEX_HOME ou dans le trousseau. | Une connexion désigne un environnement existant ; pas de copie des fichiers d’authentification ni de login Bridget. |
| https://learn.chatgpt.com/docs/config-file/config-reference | Modèle et effort sont des réglages distincts, avec valeurs dépendant du modèle. | Valider les axes séparément et réutiliser les capacités du lanceur. |
| https://code.claude.com/docs/en/cli-reference et aides locales Mac2.1.300/Cartae2.1.251 vérifiées par ROOT | --bare restreint l’auth aux APIkey/helpers ; --safe-mode désactive les personnalisations et conserve auth/politiques administrées ; outils et sortie structurée demandent des options distinctes. | Interdire --bare pour notre OAuth ; employer safe-mode+restricted, outils absents, MCP vide, schéma et cycle de vie borné. |
| https://www.anthropic.com/engineering/building-effective-agents | Le routage peut employer une classification déterministe ou un modèle ; la complexité doit répondre à un besoin réel. | Une chaîne fixe de niveaux suffit ; aucun agent récursif, moteur ou graphe d’orchestration ajouté. |
| https://modelcontextprotocol.io/docs/2025-11-25/tutorials/security/security_best_practices | Les frontières d’autorisation ne sont pas remplacées par des indications données au modèle. | Le parseur fermé et les gardes d’admission portent l’autorité ; aucune instruction du décideur ne peut élargir les droits. |

Les sources externes établissent des contrats, pas la disponibilité de Terra/Luna/Sol ou Haiku/Sonnet/Opus
sur chaque compte réel. La recette d’activation doit produire ce catalogue pour chaque machine.
Aucune donnée privée de registre ou d’authentification n’a été envoyée à ces sources.

## Faits du socle réemployés

- SPEC154 : worktree sur commit figé ; état Git rapporté au résultat, pas de synchronisation Git du daemon.
- SPEC157/160 : faits de quotas par type et instance, crédits attestés et recontrôle du budget enfant.
  Il n’existe pas de pool canonique de comptes ; ne pas annoncer une déduplication entre connexions.
- SPEC162 : admission distante, mappings, contrôle frais au Prepare, identité et connection_epoch.
- SPEC163 : politique automatique, probe sans effets, service décision, reçus d’admission et CAS,
  clôture métier distincte des erreurs techniques, reprise sans nouvelle requête de décision.
- SPEC167 : façades CLI delegate/task autonomes ; pas de nouvelle commande168.

Lectures ciblées : protocol.rs définit AutoDelegateV1 fermé, CandidateProbe v1 borné à huit
et is_auto_candidate_id ; lineage_protocol.rs réemploie ce validateur pour LineageDecisionV3.
delegation.rs valide aussi les IDs du reçu et limite ses métadonnées.
decision_service.rs expose un client HTTP borné, cache santé, permis réseau et CallerDeadline de deux secondes,
avec requête Laya limitée à4096octets et state à1536octets. Ces bornes ne peuvent pas être
augmentées implicitement pour faire passer un catalogue168.

Compléments vérifiés par ROOT : CodexAppServerTransport::spawn_with_environment, ClaudeStreamJsonTransport::spawn_with_environment,
ManagedSession drain/cancel/stop, registry.pin_delegation_model, lifecycle.launch_preflight/build_environment.
Codex select_runtime réalise déjà model/list avec quatre pages et deux secondes au plus.
Les constructeurs de transport actuels n’isolent pas encore tout environnement et cwd pour le rôle décideur.
ROOT a observé Claude CLI2.1.300 et ses flags locaux ; ce constat reste séparé de leur documentation.

## Décisions et alternatives

### D1 — Connexion déclarée, catalogue de modèles séparé
Choix : réutiliser un lanceur existant par connexion et son environnement privé ; un lanceur ne se multiplie pas par modèle.
La connexion a un ID opaque, un moteur, une machine et un agent_type interne. Le catalogue étend ce lanceur
avec ses modèles/efforts attestés. Une option porte ses efforts compatibles et ne devient pas
une option par effort ; la sortie finale résout l’effort dans cette liste. Les champs publics montrent moteur et vrais modèles, pas des noms de comptes.
Alternative rejetée : un profil utilisateur par modèle ou une table de secrets/facturation.
Impact mainteneur : un raccord de configuration aux transports existants ; pas de nouveau gestionnaire d’authentification.

### D2 — 128 options explicites, capacité additive
Choix ROOT : plafond global statique128options configurées avant filtres, non tronquées ;
registre129 refusé, aucun complete:false masquant le dépassement. IDs c001 à c128 pour168.
Conserver c01 à c08, AutoDelegateV1 et probev1 pour les requêtes163.
AutoDelegateV2, probev2 négocié et Lineagev4 couvrent168 sans changer les schémas fermés anciens.
Une réponsev3 reste lisible pour une tâche168 avec placement existant et decision:null ; jamais un faux IDv3.
Alternative rejetée : remplacer le validateur globalv1 ou découper les candidats en groupes de huit avec décisions concurrentes.
Impact mainteneur : versions explicites au parseur et au snapshot, une seule admission métier.

### D3 — Score simple, puis service, puis petit modèle, puis score
Choix : un filtrage commun, un classement déterministe et un snapshot unique.
Le premier décideur est le service configuré163. Le second est un rôle interne Claude natif sans outils,
avec modèle léger réel configuré, sans Task ou AutoDelegate récursive.
Codex n’est pas annoncé comme décideur sans outils : ce contrat n’est pas prouvé.
L’aide locale des deux versions prouve --safe-mode pour garder OAuth et désactiver hooks/skills/plugins/MCP/CLAUDE.md.
--bare est exclu : il attend APIkey/helper. Ajouter restricted, tools vide, MCP strict vide,
setting-sources vide, disable-slash-commands, no-session-persistence et JSON-schema print/stream-json.
Ne pas conserver stderr ; stdout64Kio et réponse4Kio sont bornés avant parsing.
Alternative rejetée : nouvel engine, session interactive, keyAPI imposée ou règles métier par famille.
Impact mainteneur : réemploi du transport et du cycle de vie avec options d’isolement explicites.

### D4 — Liste entière ou niveau ignoré, jamais top8 caché
Chaque décideur reçoit tous les candidats admissibles dans son format borné.
Si le service Laya existant ne peut encoder cette liste entière dans ses bornes attestées,
ce niveau devient indisponible avec motif de capacité ; le petit décideur puis le score couvrent la liste entière.
Il n’existe ni tournoi, ni résumé destructeur, ni liste de huit élus intermédiaires.
Chaque niveau réellement consulté conserve tous les faits FR021 : modèles/efforts, workload,
ressources et âges, continuité et quotas. Laya emploie un encodage compact sans perte, nomsmodèles≤32octets,
IDs v2 et état agrégé. Adapter ses validateurs v2 du constructeur et parseur sans modifier163.
Recette service sur un ensemble entier qui tient (≤8conservateur) ; matrice12+ prouvée par
le petit décideur réel et le score, et skipLaya de capacité explicite si cette matrice ne tient pas.
Ni santéHTTP200 ni endpoint configuré ne prouvent le choix réel ; aucune promesse Laya128.

### D5 — Mesures fraîches et continuité non bloquante
Le même probe worker transporte un snapshot par machine, pas un appel système par candidat.
Mesures natives ou commandes fixes internes autorisées seulement ; aucun script reçu d’un modèle.
CPU en proportion depuis deux échantillons, mémoire disponible/totale et charge normalisée sont optionnels.
Premier échantillon ou mesure expirée = unknown. Une saturation franche fraîche est exclue avant les décideurs si une autre machine est prouvée
disponible. Une machine imposée saturée refuse sans bascule. Le cache dépôt/compilation donne
un bonus borné sur les candidats restants ; aucun LLM ne peut annuler ce filtre.
Alternative rejetée : Prometheus obligatoire, benchmark de compilation, nouveau service de métriques.
Impact mainteneur : petite projection à la demande et aux cadences existantes, sans ordonnanceur supplémentaire.

### D6 — Budget de compte attesté, sans comptabilité
Rattacher les observations d’instance à la connexion choisie avec identité de lanceur et époque vérifiées.
Ne pas en déduire un budget canonique de compte partagé entre machines.
Deux noms de connexion peuvent représenter le même compte ; aucune somme ni crédit transféré n’en résulte.
Un changement d’identité ou d’environnement invalide l’association ; une absence reste unknown.
Les fenêtres restantes et resets proches attestés peuvent donner un bonus borné lorsque charge
et pertinence sont proches, sans somme entre comptes ni calcul financier. Les décideurs reçoivent
ces faits, les ressources agrégées et les noms réels des modèles, pas une simple description « Codex distant ».
Le recontrôle enfant actuel reste décisif au lancement.

### D7 — Migration et reprise
Les reçus163 existants conservent leur format et leurs huit IDs.
Un snapshot168 versionné conserve la matrice complète, le score et les réglages de repli avant consultation.
Les états SQL/CAS existants sont étendus, pas recopiés dans un moteur parallèle.
Après crash, aucun nouveau modèle ou service n’est appelé pour cette clé ; l’entretien utilise le repli figé.
Un choix déjà choisi reste identique même si la charge change. Une erreur technique conserve l’état récupérable.

### D8 — Git et activation
L’audit confirme que Shared163 n’a pas de baseSHA et que le maître y portait base_commit=None.
AutoDelegateV2.expected_base_commit corrige cela sans modifier163 : SHA source obligatoire
pour tout espace Git168, Shared ou Worktree, puis snapshot/probev2/WorkerPrepare.base_commit.
Réemployer verify_base_commit ; absence = workspace_base_required avant fournisseur.
Worktree.base_ref doit résoudre le même SHA. Shared horsGit général peut omettre cette base.
Le coordinateur prépare ce SHA exact sur les machines candidates ; aucune baseHEADdistante implicite.
Aucun bonus de continuité ne dispense de ce prérequis ; préparer le dépôt ailleurs reste possible avant la demande.
La collecte des résultats et l’intégration appartiennent à un agent autorisé, pas au daemon.
ROOT possède provisionnement, authentifications natives éventuelles, installation, activation et nettoyage final.
Laya signalé sain sur127.0.0.1:8792/v1/systemone avec typed-decisions est un fait d’entrée ROOT,
pas une nouvelle recette de ce rédacteur.

## Limites de cette recherche
Aucun build, test, appel au modèle ou changement d’authentification n’a été exécuté par le rédacteur.
La disponibilité fournisseur réelle, le contrat complet d’isolement Claude/OAuth et la réponse Laya
sur un ensemble entier qui tient restent des preuves de développement puis d’activation à produire.
Le catalogue12+ n’impose pas douze appels fournisseurs : les droits d’accès restent inconnus
après une simple découverte ; un refus fournisseur final reste explicite, sans bascule silencieuse. Pas de compatibilité Jev inventée.

## Inventaire runtime communiqué par ROOT — 15 octobre2026
Sans appel modèle, initialize/model/list a retourné en une page sur Maccodex, Maccodex-pro et Cartaecodex :
gpt-6.1-sol, gpt-6-astra, gpt-6-sol, gpt-6-luna, gpt-5.6-sol, gpt-5.6-terra, gpt-5.6-luna.
Efforts annoncés low/medium/high/xhigh/max/ultra, sauf Luna sans ultra.
Ce catalogue déclaré ne prouve pas l’entitlement de chaque modèle.
Les deux environnements Mac sont distincts ; aucune seconde connexion Codex Cartae n’est attestée.
Claude Cartae est installé en2.1.251 et auth status annonce loggedIn=true, authMethod claude.ai, subscriptionType max.
Cette lecture ne révèle ni email ni token et ne justifie aucun login supplémentaire.
ROOT a vérifié les flags dans les aides2.1.251/2.1.300 ; la recette OAuth et le JSON-schema
en print/stream-json restent à prouver avant activation. Les niveaux d’effort exposés par l’aideClaude
ne prouvent pas leur acceptation pour chaque modèle. default_effort est résoluble par modèle :
niveau compatible, ou null/liste vide pour défaut natif sans flag ; aucun medium n’est inventé.

## Corrections de contrat confirmées par audit r1
HandoffV1 est fermé : workload=general|compilation, défautgeneral, est un champ AutoDelegateV2
hors brief, conservé dans le canon/snapshot et la charge de décision.
human_authority_guard dépend de bindingsT3 : il est interdit pour notre catalogue autonome.
routing_catalogue_v1 utilise unix_peer_uid/peer_uids, mêmeUID, Role::Client et version négociée,
comme le pattern de garde WakeAction::CliOnce, avec projection publique allowlistée.
Le mode SQL reste explicit|automatic|fallback ; source détaillée en metadata v2.
Une base neuve crée le DDL168 directement ; une base163 migre par transaction et copieBLOB exacte.
Toute ligne v2 interdit un ancien binaire même si168 estOFF. Ces décisions n’ajoutent aucun moteur.
