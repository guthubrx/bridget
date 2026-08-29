# Recherche : Plan de contrôle Bridget et Maicie

**Date** : 2026-08-29
**Périmètre** : orchestration multi-agents, contrats fournisseurs, sécurité,
observabilité, tests et charge cognitive.

## Méthode

La recherche combine :

1. le code Bridget et Maicie au commit
   `a832221e0d326688ee3d568978eb0f618227ded5` ;
2. le code officiel Codex local au commit
   `6478a751fde8884b2fdc76486fe23175a8e795d4` ;
3. les baselines `/home/moi/.speckit/research/` ;
4. des sources primaires live consultées le 2026-08-29.

La mémoire DevKMS n'a pas pu être consultée ni enrichie : la commande `mem`
est absente du serveur. Aucun code, log privé ou contenu utilisateur n'a été
envoyé à une source externe.

## Décision 1 - Trois plans et deux vérités durables

**Décision** : Maicie porte la mission, Bridget porte le contrôle d'exécution,
les adaptateurs portent la traduction fournisseur. La base Maicie et la base
Bridget restent séparées.

**Rationale** : Maicie possède déjà les objectifs, délégations, décisions et
outboxes. Bridget possède les instances, générations, livraisons et transports.
Codex montre qu'un thread, un tour et un item forment une troisième sémantique,
distincte de la mission et du transport réseau.

**Alternatives considérées** :

- fusionner Maicie dans le daemon : rejeté, car cela crée une autorité unique
  ambiguë et contredit l'ADR 003 ;
- remplacer Maicie par le goal Codex : rejeté, car le goal reste lié à un
  thread et ne porte pas les délégations et décisions multi-fournisseurs ;
- conserver le modèle actuel : rejeté, car l'incident 063 prouve l'absence de
  vérité explicite entre acceptation et consommation.

**Impact mainteneur** : chaque état durable possède un propriétaire unique et
les projections croisées deviennent remplaçables.

## Décision 2 - Séparer soumission, livraison et exécution

**Décision** : introduire une soumission logique et une exécution durable sans
remplacer la livraison idempotente existante.

**Rationale** : la livraison répond à « les mêmes octets ont-ils été remis à la
bonne instance ? ». L'exécution répond à « ce travail est-il en file, visible,
en cours, en attente ou terminé ? ». Confondre ces questions produit soit des
acquittements trop tôt, soit des livraisons bloquées trop longtemps.

**Alternative considérée** : ajouter seulement des phases à `send_deliveries`.
Rejetée, car une même livraison peut être acquittée alors que l'exécution reste
active ou échoue ensuite.

**Impact mainteneur** : deux petites machines d'état explicites remplacent une
suite d'inférences réparties entre daemon, wrapper et adaptateur.

## Décision 3 - Intention explicite, pas d'inférence depuis busy

**Décision** : une soumission déclare si elle doit attendre, déclencher un tour,
piloter le tour courant, interrompre ou agir comme commande de contrôle.

**Rationale** : Codex sépare la mailbox du steering et distingue le message
passif d'un suivi qui déclenche un tour. Le dépôt Bridget déduit encore certains
choix de l'expéditeur et de l'état actif, ce qui ne couvre ni les agents ni les
routines de manière non ambiguë.

**Alternative considérée** : conserver une règle globale « humain prioritaire ».
Rejetée comme seul contrat, car priorité et mode d'exécution sont deux choses
différentes.

**Impact mainteneur** : les tests portent sur une intention déclarée et non sur
une combinaison implicite de champs.

## Décision 4 - Capability negotiation et contrat de version

**Décision** : combiner les capacités négociées au protocole avec un contrat de
version validé en CI et l'identité du binaire réellement exécuté.

**Rationale** : MCP définit un cycle de vie avec négociation de capacités et
état de connexion. Codex génère ses schémas TypeScript et JSON pour la version
du binaire utilisée. Le serveur possède actuellement Codex 0.150.1 dans
`/home/moi/.local/bin/codex` et 0.98.0 dans `/usr/local/bin/codex`, avec des
surfaces différentes.

**Alternatives considérées** :

- supposer la capacité à partir du type de fournisseur : rejeté ;
- générer tout le schéma à chaque démarrage : rejeté, coût runtime inutile ;
- pinner seulement le chemin : rejeté, un binaire peut être remplacé au même
  chemin.

**Impact mainteneur** : toute divergence produit un refus ou repli explicite et
un banc de test reproductible.

### Cursor utilise le transport ACP existant

**Constat vérifié** : le registre Bridget déclare déjà le type `cursor` avec la
commande `cursor-agent --model auto acp`, le protocole `acp` et le chemin
d'exécution `acp`. Le wrapper sélectionne `AcpTransport` à partir du protocole,
et `AcpTransport` implémente `ManagedSession`.

**Décision** : Cursor reste un fournisseur distinct utilisant l'adaptateur ACP
commun. La SPEC-064 étend et teste `AcpTransport` ; elle ne crée pas
d'adaptateur Cursor séparé.

**Preuve locale** : les tests
`acp_generique_cursor_et_gemini_restent_admis` et
`etiquette_modele_auto_lue_depuis_les_args_cursor` ont été rejoués le
2026-08-29 sur le serveur : 2 succès, 0 échec.

**Limite observée** : `cursor-agent` n'était pas résolu dans le `PATH` de la
session SSH non interactive utilisée pour cette vérification. T002 doit donc
attester le chemin et la version du binaire sur l'hôte d'exécution réel avant
l'activation, sans remettre en cause le contrat ACP déjà présent.

## Décision 5 - Corréler Codex par clientId sur item/started

**Décision** : la preuve d'admission d'un message piloté se fonde sur le cycle
de vie officiel de l'item et le `clientId` réémis par Codex.

**Rationale** : la documentation officielle indique que
`clientUserMessageId` est réémis dans le champ `clientId` du `userMessage`.
Le cycle de vie canonique est `item/started`, deltas éventuels,
`item/completed`. Le test officiel `turn_steer` attend `item/started` et compare
le `clientId`. Le champ `item.id` reste l'identité interne de l'item.

**Alternative considérée** : attendre `item/completed.id`. Rejetée, car le
schéma distingue explicitement `id` et `clientId`.

**Impact mainteneur** : le faux fournisseur cesse de définir un comportement
plus favorable que le fournisseur réel.

## Décision 6 - Graphe de propriété, pas moteur de workflow

**Décision** : ajouter uniquement les arêtes parent-enfant, mandat, rôle,
propriété et cycle de vie nécessaires au contrôle des agents.

**Rationale** : SPEC-034 a déjà établi que le système doit gouverner les
engagements critiques et réconcilier les arêtes absentes, sans détailler la
pensée des agents. Codex fournit un exemple de graphe durable d'agents et de
quotas de profondeur.

**Alternatives considérées** : DAG générique, swarm ou framework externe.
Rejetées, car aucun besoin prouvé ne justifie cette charge et le workflow
métier existe déjà dans Maicie.

**Impact mainteneur** : le graphe répond à des questions concrètes : qui a créé
l'agent, qui peut le piloter et où doit revenir son résultat.

## Décision 7 - Reprise native avec repli textuel déclaré

**Décision** : conserver les références fournisseur, utiliser reprise ou fork
quand la capacité est attestée et garder la carte textuelle comme repli.

**Rationale** : Codex expose `thread/start`, `thread/resume` et `thread/fork`.
Une carte de reprise reconstruit le contexte mais ne préserve pas l'identité du
thread. Les deux mécanismes ont de la valeur si leur nature est visible.

**Alternative considérée** : imposer un modèle commun de reprise à tous les
fournisseurs. Rejetée, car les garanties natives diffèrent.

**Impact mainteneur** : le mode de reprise devient vérifiable et les branches
fournisseur restent dans les adaptateurs.

## Décision 8 - Projection Maicie publique et non résidente

**Décision** : Maicie publie une projection publique atomique lors de ses
mutations explicites. Bridget la lit sans importer le domaine ou ouvrir la base
privée. Les faits d'exécution reviennent à Maicie par le contrat Bridget avec
curseur et fraîcheur.

**Rationale** : l'ADR 003 exige un compagnon remplaçable et sans supervision de
processus. Le daemon dépend pourtant actuellement du crate Maicie pour certaines
vues. Une API résidente Maicie introduirait un nouveau daemon que la doctrine a
explicitement exclu.

**Alternatives considérées** :

- lecture directe SQLite : rejetée, partage de schéma privé ;
- service Maicie résident obligatoire : rejeté, nouvelle responsabilité
  d'exploitation ;
- dépendance de compilation conservée : rejetée à terme, car elle invalide la
  remplaçabilité annoncée.

**Impact mainteneur** : les consommateurs lisent un contrat versionné et savent
si la projection est fraîche, périmée ou indisponible.

## Décision 9 - Observabilité corrélée et contenu minimisé

**Décision** : concevoir ensemble journaux, métriques et traces, en propageant
les identifiants de contrôle mais pas le contenu ou le raisonnement par défaut.

**Rationale** : Google SRE recommande de surveiller latence, trafic, erreurs et
saturation. OpenTelemetry sépare logs, métriques et traces mais permet leur
corrélation par contexte. L'incident 063 n'était compréhensible qu'en recoupant
SQL, reçus wrapper, journal fournisseur et UI.

**Alternative considérée** : conserver seulement le JSONL. Rejetée, car il est
excellent pour la preuve a posteriori mais insuffisant pour détecter rapidement
une file vieillissante ou répondre à une question d'état.

**Impact mainteneur** : le dashboard répond aux questions courantes et le
journal reste disponible pour l'enquête détaillée.

## Décision 10 - Tests de contrat avant mocks de comportement

**Décision** : valider les fixtures fournisseurs contre les schémas de version,
séparer volontairement les identifiants et couvrir les erreurs avant les tests
de chemin nominal.

**Rationale** : le faux Codex actuel copie `clientUserMessageId` dans
`item.id`, masquant une incompatibilité. La documentation Google distingue les
types de doubles de test, et le dépôt Codex possède des suites app-server qui
traitent le protocole comme contrat public.

**Alternative considérée** : multiplier les tests unitaires du mock existant.
Rejetée, car cela renforcerait la confiance dans une hypothèse fausse.

**Impact mainteneur** : les tests échouent quand le fournisseur change de
contrat au lieu de valider une simulation locale périmée.

## Décision 11 - Sécurité outcome-based et moindre privilège

**Décision** : protéger les effets irréversibles par identité, génération,
capacité et politique de permission, avec refus par défaut sur incohérence.

**Rationale** : NIST SSDF organise le développement sécurisé autour de
résultats vérifiables et de la protection des composants. Les guides OWASP
recommandent validation aux frontières, moindre privilège et journalisation des
succès et échecs de sécurité. Le modèle local coopératif reste déclaré, sans
revendiquer une isolation hostile inexistante.

**Alternative considérée** : conserver une chaîne globale allow ou deny pour
toutes les autorisations fournisseur. Rejetée, car elle masque l'attente et ne
porte pas l'autorité par action.

**Impact mainteneur** : les permissions deviennent des événements et décisions
inspectables, pas un réglage implicite de l'adaptateur.

## Sources primaires live

- OpenAI, `codex app-server`, protocole, schémas, threads, tours, items,
  steering, reprise, fork et backpressure :
  https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md
- OpenAI, type `ThreadItem` et séparation `id` / `clientId` :
  https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/v2/item.rs
- Model Context Protocol, lifecycle et négociation de capacités :
  https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle
- NIST, Secure Software Development Framework :
  https://csrc.nist.gov/projects/ssdf
- Google SRE, monitoring des systèmes distribués :
  https://sre.google/sre-book/monitoring-distributed-systems/
- OpenTelemetry, signaux et propagation de contexte :
  https://opentelemetry.io/docs/concepts/signals/
  https://opentelemetry.io/docs/concepts/context-propagation/
- Google Testing Blog, catégories de doubles de test :
  https://testing.googleblog.com/2013/07/testing-on-toilet-know-your-test-doubles.html
- OWASP, Secure Coding Practices Quick Reference Guide :
  https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/

## Red flags retenus

- Promesse de « full autonomy » sans budget ni état fiable.
- Adoption d'un framework multi-agent qui duplique Maicie et Bridget.
- Projection qui devient une troisième source de vérité.
- Capability matrix déclarative sans preuve sur le binaire réel.
- Faux fournisseur non validé contre un schéma réel.
- Logs seuls sans métriques de vieillissement et saturation.
- Identifiants à forte cardinalité comme labels de métriques.
- Permission globale assimilée à une décision humaine.
- Reprise textuelle présentée comme reprise native.
- Extraction prématurée de nouveaux crates ou wrappers sans trois usages.
