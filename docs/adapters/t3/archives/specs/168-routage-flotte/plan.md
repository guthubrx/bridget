# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/plan.md](../../../../../../specs/168-routage-flotte/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Santé finale reçue : B en ligne, catalogue 27, pont 32 fils vers 97328 ; DB readonly quick_check ok/FK0, versions 7/12 ; worker B 3767260 actif, configuration inchangée. Application T3 65544 et tunnel 60206 préservés ; le serveur 65608 a été remplacé extérieurement par 97328 né à 16:51:02 CEST, sans signal T3 de notre part. La restriction temporaire de 16:56 CEST est levée.

<a id="extrait-2"></a>

## Extrait 2

Limites : FULL unique 2820 PASS/32 FAIL/71 ignorés ; 29 échecs initiaux PASS après reprises, trois annulations 159 non validables sur Linux, cause initiale T3 inconnue malgré replay PASS. Deux LOW rustls-webpki et une INFO unsafe-libyaml restent documentés ; aucun GREEN global. Aucun nouvel audit/FULL/P4/build/service/test/modèle par cette rédaction. Aucun retour direct au binaire 0.3.0 après reçu admission v2. Les commits documentaires descendants conservent la source binaire B et ne nécessitent aucune reconstruction.

<a id="extrait-3"></a>

## Extrait 3

FULL : 2 820 PASS / 32 FAIL / 71 ignorés. Après reprises ciblées, 29 des 32 échecs initiaux passent ; trois annulations 159 restent non validables sur Linux (observateur kqueue Darwin/BSD). La cause du premier échec d’identité T3 reste inconnue malgré son replay PASS. Aucun succès global vert n’est déduit.

<a id="extrait-4"></a>

## Extrait 4

Rust, transports natifs Codex/Claude, maître et workers existants, SQLite DelegationStore,
registre privé, CLI et MCP. Aucune nouvelle base, dépendance T3, file de tâches ou daemon de métriques.
La version0.3.0 a été activée sur Mac/Cartae selon ROOT ; cette note ne prouve pas168.
Lire recherche, modèle et contrat avant implémentation ; aucune décision de modèle ne vaut une permission.

<a id="extrait-5"></a>

## Extrait 5

Ajouter une section versionnée facultative, strictement validée, en conservant les données163.
Une connexion référence un lanceur agent_type existant, son moteur réel et sa machine autorisée.
Les environnements privés du lanceur isolent le compte. Le modèle n’est pas le nom de connexion.
Codex consomme les observations model/list du préflight/runtime natif existant ou de l’opérateur T043 ;
aucun processus depuis le loader de registre ou la lecturecatalogue, aucune découverte automatique nouvelle.
Claude réemploie les modèles/efforts déclarés et validés par le registre.
Le catalogue montre leurs identifiants réels et la provenance, jamais une entitlement garantie.
Lecture exacte horsT3 : bridget delegate --catalogue --json, avec filtre --machine facultatif,
et rendu humain sans --json. L’opération native routing_catalogue_v1 est décidée sur le canal existant.
Elle exige le peer UID Unix égal à l’UID du daemon, Role::Client et version négociée, sans identité
d’agent obligatoire ni lien T3. Réemployer unix_peer_uid, peer_uids et la garde de WakeAction::CliOnce ;
jamais human_authority_guard, dont le bindingT3 serait une dépendance interdite. La projection est
une liste publique allowlistée, sans autorité de commande ou de mutation.

<a id="extrait-6"></a>

## Extrait 6

Le petit décideur utilise d’abord le transport Claude existant, modèle léger réel configuré.
Il fonctionne sur la machine du coordinateur avec une connexion déjà authentifiée.
Il s’agit d’un appel interne borné, sans création de Task, AutoDelegate ou agent récursif.
Mode requis : contexte enfant isolé, env_clear puis environnement minimal allowlisté,
cwd temporaire neutre, pas de resume/session_store_root du projet, pas d’injection Bridget/T3,
--safe-mode et --restricted, --tools vide, MCP strict vide, --setting-sources vide,
--disable-slash-commands, sortie JSON validée par schéma en print/stream-json et --no-session-persistence.
--bare est interdit ici : l’aide locale des deux versions limite son authentification aux APIkey/helpers,
alors que la connexion réelle est OAuth. --disallowedTools "*" reste une défense additionnelle
seulement si son support est attesté ; il ne remplace pas les autres gardes.
stdout est borné avant parsing à64Kio et la sortie normalisée à4Kio ; stderr n’est pas conservé
et aucun nouveau tampon stderr n’est ajouté.
Ici « sans outils » interdit tout outil métier/système/réseau/MCP. Seul le formateur natif
StructuredOutput ajouté par json-schema est autorisé, sans droit d’exécution. Contrôler l’allowlist
des capacités annoncées et rejeter tout autre tool_use ; valider localement son objet fermé
puis la concordance avec structured_output/resultJSON et les ID/efforts admissibles.
La préqualification CLI Cartae a observé init.tools=[StructuredOutput], jamais une preuve init.tools=[].
Cette préqualification n’est pas une recette du transport Bridget168.
Ne pas contourner les politiques administrées du moteur ; refuser le niveau si elles empêchent ce mode.
Les écritures natives de renouvellement auth/state du fournisseur ne sont pas des écritures métier.
Codex n’est pas proposé en rôle décideur tant qu’un mode sans outils n’est pas attesté.

<a id="extrait-7"></a>

## Extrait 7

Lineagev4 expose connexion et origine de décision168.
Pour une tâche168 lue en v3 : projectionv2 inchangée, decision:null, jamais c001 dans le schémav3.
Ne pas fabriquer un profil163 ou une raison tronquée pour contourner ce validateur.
Pas de changement T3 : ses lecturesv3 restent valides ; les outils autonomes peuvent négocierv4.

<a id="extrait-8"></a>

## Extrait 8

1. Après audit réutilisation/revue, figer wire/config et tests de compatibilité ; générer les tâches.
2. Config/catalogue/connexions et score pur ; fixtures12/128/129, axes indépendants et inconnus.
3. Probev2/télémétrie et contrôle frais ; absence de capability, ancienne époque, SHA manquant/inexact
   en Shared et Worktree, base_ref divergente, général Shared horsGit, révocation avant Prepare.
4. Mode décideur natif sans outils, OAuth avec safe-mode réel, JSON-schema print/stream-json, réponse
   fermée, stdout borné, stderr non conservé, deadline absolue commune depuis admission et permis jusqu’à arrêt réel ; service entier ou refus de capacité.
5. Raccord unique chaîne/store/reprise ; concurrence, réponse tardive, crash aux frontières et erreurs SQL.
6. CLI/MCP/Lineagev4 et compatibilitév1/v3 ; scénario natifBridget.
7. Ciblés T035 → contre-revue/Converge T037 → audit et corrections T038 → FULL unique T036
   sur source stabilisée → docs T039 et livraison. Converge écrit append-only Tasks, code inchangé ;
   rapports et corrections se font hors de cette passe.
8. Construire les paquets nécessaires, intégrer/pousser, activer les connexions réelles, puis recette et nettoyage.
   Prévoir SemVer0.4.0 pour cette fonction compatible si la source reste0.3.0 ; version/changelog
   seulement en livraison, paquets/tag/release liés au même commit exact, installation Mac/Cartae et T3 préservé.

<a id="extrait-9"></a>

## Extrait 9

Sauvegarder registre, unités/plists, binaires et bases cohérentes avant migration.
Activer d’abord les capacités et vérifications, puis les niveaux configurés par machine.
Recette native : modèles et machine imposés séparément, compte isolé, saturation contrôlée,
Laya valide sur ensemble entier compatible puis indisponible, petit décideur réel sur12+options
puis indisponible, dernier score sur matrice entière, rejeu inchangé ; aucun appel à tous les modèles
pour prétendre prouver leur entitlement. Un refus fournisseur final reste explicite sans bascule.
Prouver l’indépendance dans un namespace isolé sans configuration ni pont T3 ; ne pas arrêter
le T3 de l’utilisateur. Faire trois recettes de décision réelles bornées : LayaON,
LayaOFF+petit Haiku actif, puis les deuxOFF+score. Les hôtes distants doivent rapporter cartae.

<a id="extrait-10"></a>

## Extrait 10

Le plafond128 compte toutes les combinaisons déclarées connexion×modèle, désactivées comprises,
avant filtre de demande ; les efforts ne multiplient pas ces options.
routing_catalogue_v1 conserve la version existante du clientcontract et ajoute sa feature/opération
de lecture, sameUIDUnix/Role::Client, sans human_authority_guard ni T3.
D force inherit_stderr=false. B possède aussi lineage_protocol.rs et publie tous ses types avant D/C.
Le collecteurB est réemployé par le ticklocalC existant, sans nouveau thread ; premierCPU=null.
Le préflight fermé distingue un véritable horsGit d’une erreur technique : une erreur ne vaut jamais outsideGit.
SansSHA attendu, un candidatGit Shared/Worktree est exclu avec workspace_base_required ;
si aucun candidat ne reste, ce refus est rendu globalement. Shared horsGit général accepte l’absenceSHA,
mais un SHA fourni horsGit refuse mismatch. Worktree exigeGit et la base_ref égale au SHA attendu.
Ces précisions sont reprises dans Tasks sans modifier le parcours163.
