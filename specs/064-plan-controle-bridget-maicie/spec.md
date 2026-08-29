# Feature Specification: Plan de contrôle Bridget et Maicie

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 064-plan-controle-bridget-maicie
Titre: Plan de contrôle Bridget et Maicie
Statut: Implemented in isolated worktree - activation production non realisee
Priorité: P1
Tâches: 89/89 (100%)
Tests: 41/41 (100%)

Résumé:
- Contexte: Bridget garantit durablement le transport et le parc, tandis que Maicie garantit les objectifs et délégations, mais l'exécution réelle d'un travail reste insuffisamment représentée entre ces deux vérités.
- Objectif: rendre l'exécution pilotable, observable, reprenable et corrélée aux missions sans fusionner Bridget, Maicie et les fournisseurs.
- Dépendances: SPEC-003, SPEC-004, SPEC-009, SPEC-011, SPEC-012, SPEC-014, SPEC-015, SPEC-016, SPEC-023, SPEC-034, SPEC-046, SPEC-048, SPEC-049, SPEC-052, SPEC-063

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-064-plan-controle-bridget-maicie`
**Created**: 2026-08-29
**Status**: Implemented in isolated worktree - activation production non realisee
**Priority**: P1
**Dependencies**: SPEC-003, SPEC-004, SPEC-009, SPEC-011, SPEC-012, SPEC-014, SPEC-015, SPEC-016, SPEC-023, SPEC-034, SPEC-046, SPEC-048, SPEC-049, SPEC-052, SPEC-063

## Etat de livraison - 2026-08-29

- Les lots techniques et documentaires T001 a T089 sont prouves dans le worktree isole.
- Cursor est atteste par son binaire reel via ACP commun ; aucun adaptateur Cursor specialise nest requis.
- SPEC-063 est integree au niveau code et tests ; sa preuve de route reelle T011 attend une mise en service controlee.
- Toutes les bascules de la SPEC-064 restent desactivees. Ce statut ne vaut ni release ni activation.

## Contexte

Bridget possède déjà les identités d'agents, le parc désiré, les générations,
les livraisons durables, les reçus et les adaptateurs fournisseurs. Maicie
possède déjà les objectifs, délégations, décisions, dépendances, outboxes et
preuves métier. Ces deux sources de vérité sont nécessaires et ne doivent pas
se remplacer.

La lacune se situe entre la remise durable d'un message et l'exécution réelle
du travail demandé. Le système distingue encore mal un agent connecté, un
agent qui travaille, un agent en attente, un message seulement accepté par le
fournisseur et un message effectivement visible dans le tour du modèle. Cette
ambiguïté complique le pilotage humain, le suivi des coordinateurs, la reprise
après interruption et la coordination d'une flotte d'agents.

Cette feature définit le programme fonctionnel complet permettant à Bridget de
porter la vérité d'exécution et à Maicie de conserver la vérité de mission.
Elle prolonge la décision SPEC-034 : gouverner les engagements observables et
les frontières critiques, sans prescrire la pensée interne des agents ni
introduire un moteur de workflow générique.

## Objectifs

- Rendre chaque travail soumis à un agent explicitement identifiable et
  pilotable jusqu'à une issue bornée.
- Distinguer transport, consommation et exécution sans affaiblir les garanties
  idempotentes existantes.
- Permettre à un opérateur de connaître l'activité réelle d'un agent sans
  analyse forensique manuelle.
- Représenter durablement la propriété et la hiérarchie des agents d'une
  flotte.
- Reprendre une exécution avec le meilleur mécanisme attesté par le fournisseur
  sans présenter une reconstruction comme une continuité native.
- Garantir le même plan de contrôle sur Codex app-server, Claude stream-json et
  Cursor via l'ACP maintenu par Cursor, selon les capacités réellement exposées.
- Relier une exécution à une délégation Maicie sans transférer la décision
  métier à Bridget.
- Encadrer l'autonomie par des limites observables de temps, d'usage et de
  descendants.

## Scénarios utilisateur et acceptation

### User Story 1 - Piloter et comprendre une exécution réelle (P1)

En tant qu'humain opérateur, je veux savoir ce qu'un agent fait réellement et
lui adresser une correction immédiate afin de ne jamais confondre présence,
travail, attente et blocage.

**Pourquoi cette priorité** : l'incident de la session 063 a montré qu'un
message pouvait être accepté par le transport sans devenir une entrée
consommée par le modèle, tandis que l'interface ne permettait pas de distinguer
les deux situations.

**Test indépendant** : pendant un tour actif, envoyer successivement un message
à conserver, une correction immédiate et une demande d'interruption, puis
vérifier que chaque demande suit la stratégie annoncée, possède une issue
bornée et reste corrélée au bon tour.

**Scénarios d'acceptation** :

1. **Étant donné** un agent connecté sans tour actif, **quand** un travail doit
   commencer, **alors** il devient visible comme travail en file puis comme
   tour en cours, avec une corrélation stable.
2. **Étant donné** un tour actif pilotable, **quand** l'humain envoie une
   correction immédiate, **alors** la demande est injectée dans ce tour et
   n'est déclarée consommée qu'après une preuve corrélée.
3. **Étant donné** un tour actif non pilotable ou silencieux, **quand** la
   correction atteint sa borne, **alors** le système applique le repli annoncé
   et rend une issue observable sans attente infinie.
4. **Étant donné** un agent en attente d'une autorisation ou d'une réponse
   humaine, **quand** l'opérateur consulte son état, **alors** cette attente est
   distinguée d'un calcul en cours et d'une panne.

---

### User Story 2 - Coordonner une flotte avec propriété durable (P1)

En tant que coordinateur, je veux connaître les agents que je possède, leur
mandat et leur état afin de déléguer, relancer, interrompre ou attendre leur
travail sans créer d'agents orphelins.

**Pourquoi cette priorité** : connaître les processus vivants ne suffit pas à
connaître la structure du travail, le destinataire légitime d'un résultat ou
l'autorité autorisée à interrompre un agent enfant.

**Test indépendant** : un coordinateur crée deux enfants pour deux travaux
distincts, leur envoie des messages de nature différente, attend leurs issues,
puis disparaît ; le système conserve la propriété, les résultats et une
décision explicite pour chaque enfant.

**Scénarios d'acceptation** :

1. **Étant donné** une délégation confiée à un nouvel agent, **quand** l'agent
   est créé, **alors** son parent, son mandat, son rôle et son travail
   propriétaire sont durablement visibles.
2. **Étant donné** plusieurs enfants, **quand** le parent attend leurs
   résultats, **alors** il est réveillé par les changements pertinents sans
   interrogation périodique obligatoire.
3. **Étant donné** un parent interrompu ou disparu, **quand** un enfant termine,
   **alors** son résultat n'est ni perdu ni attribué à un autre travail et la
   règle de transfert ou de clôture est explicite.
4. **Étant donné** une limite de profondeur ou de nombre d'enfants, **quand** un
   agent demande de la dépasser, **alors** la création est refusée avec une
   raison structurée et sans état partiel.

---

### User Story 3 - Reprendre ou bifurquer un travail sans faux-semblant (P1)

En tant qu'opérateur, je veux reprendre un travail après interruption, quota ou
remplacement d'agent afin de conserver le contexte disponible et de savoir si
la continuité est native ou reconstruite.

**Pourquoi cette priorité** : une carte textuelle de reprise est utile, mais ne
constitue pas la reprise du même fil d'exécution. Les deux mécanismes doivent
être préservés et distingués.

**Test indépendant** : interrompre une exécution après plusieurs tours, puis
effectuer une reprise avec un fournisseur compatible et une reprise avec un
fournisseur incompatible ; vérifier que le mode employé et les références
conservées sont exacts dans les deux cas.

**Scénarios d'acceptation** :

1. **Étant donné** un fournisseur qui atteste une reprise native, **quand** le
   travail reprend, **alors** le même fil fournisseur est utilisé et son
   identité reste vérifiable.
2. **Étant donné** un fournisseur sans reprise native, **quand** le travail
   reprend, **alors** une reconstruction explicite est utilisée sans être
   présentée comme le même fil.
3. **Étant donné** une demande de bifurcation, **quand** le fournisseur la
   supporte, **alors** le nouveau travail conserve une ascendance durable sans
   modifier l'historique de l'original.
4. **Étant donné** une version fournisseur devenue incompatible, **quand** la
   reprise est demandée, **alors** le système refuse ou applique un repli
   déclaré au lieu de supposer la capacité.

---

### User Story 4 - Préserver les deux vérités Bridget et Maicie (P1)

En tant que responsable de mission, je veux relier une délégation Maicie à son
exécution Bridget afin d'obtenir des preuves runtime sans qu'une observation
de transport décide automatiquement du résultat métier.

**Pourquoi cette priorité** : les responsabilités sont correctement séparées
dans la doctrine, mais certains consommateurs sont déjà couplés aux détails
internes de Maicie et les références d'exécution restent insuffisantes.

**Test indépendant** : créer une délégation, lancer son exécution, interrompre
le fournisseur, rendre l'observation temporairement indisponible puis restaurer
le flux ; vérifier que la projection d'exécution évolue sans fermer, rouvrir ou
réécrire l'objectif Maicie.

**Scénarios d'acceptation** :

1. **Étant donné** une délégation Maicie, **quand** Bridget accepte son travail,
   **alors** une référence d'exécution corrélée est conservée des deux côtés
   sans partage de source de vérité.
2. **Étant donné** une exécution techniquement terminée, **quand** Maicie reçoit
   cette preuve, **alors** elle reste libre de l'évaluer avant toute transition
   métier.
3. **Étant donné** une observation périmée, incomplète ou indisponible, **quand**
   Maicie affiche le statut, **alors** la fraîcheur est explicite et aucun état
   métier n'est inventé.
4. **Étant donné** Maicie absente ou remplacée, **quand** Bridget fonctionne,
   **alors** le transport, le parc et les exécutions continuent sans dépendre
   de son état privé.

---

### User Story 5 - Exploiter plusieurs fournisseurs avec un contrat vérifiable (P2)

En tant qu'exploitant, je veux connaître la version, les capacités et les
limites de chaque fournisseur afin que Bridget choisisse une stratégie
compatible et rende tout repli visible.

**Pourquoi cette priorité** : deux versions différentes de Codex sont présentes
sur le même serveur et n'exposent pas les mêmes opérations. Cursor utilise déjà
le transport ACP commun maintenu par le fournisseur. Une configuration nominale
ou un nom de fournisseur ne prouve pas la capacité réellement disponible.

**Test indépendant** : présenter successivement Codex app-server, Claude
stream-json, Cursor via ACP, un fournisseur sans pilotage et une version
incompatible ; vérifier que les opérations autorisées, refusées et repliées
correspondent aux capacités observées.

**Scénarios d'acceptation** :

1. **Étant donné** une session fournisseur, **quand** elle démarre, **alors** sa
   version, son identité et ses capacités observées sont disponibles avant le
   premier travail dépendant de ces capacités.
2. **Étant donné** une capacité absente, **quand** un travail l'exige, **alors**
   la stratégie de repli ou le refus est explicite et vérifiable.
3. **Étant donné** une demande d'autorisation fournisseur, **quand** elle
   nécessite l'humain, **alors** l'agent apparaît en attente et la décision est
   corrélée sans acceptation globale implicite.
4. **Étant donné** des événements tardifs ou inconnus, **quand** ils arrivent,
   **alors** leurs octets et leur provenance sont conservés sans altérer un
   autre tour.

---

### User Story 6 - Gouverner l'autonomie par des faits mesurés (P2)

En tant que responsable de mission, je veux borner le temps, l'usage et les
descendants d'un travail afin qu'une continuation autonome reste explicable,
réversible et compatible avec la décision humaine.

**Pourquoi cette priorité** : l'autonomie ne peut être sûre que lorsque
l'inactivité, les coûts et les états terminaux sont fiables. Elle doit donc
être construite après le plan de contrôle.

**Test indépendant** : exécuter un objectif avec un budget de temps, un budget
d'usage et deux descendants, atteindre successivement chaque limite et vérifier
que l'exécution s'arrête ou se met en attente sans clôture métier implicite.

**Scénarios d'acceptation** :

1. **Étant donné** un budget déclaré, **quand** l'exécution et ses descendants
   consomment des ressources, **alors** le coût agrégé est visible et attribué
   au bon objectif et à la bonne délégation.
2. **Étant donné** une limite atteinte, **quand** une continuation serait
   normalement déclenchée, **alors** elle est empêchée avec un état explicite.
3. **Étant donné** un agent réellement inactif et une politique de continuation
   valide, **quand** du travail reste exécutable, **alors** la continuation peut
   être déclenchée sans nouvelle livraison concurrente.
4. **Étant donné** une mission terminée techniquement mais non évaluée, **quand**
   la politique automatique s'exécute, **alors** elle ne clôt pas l'objectif à
   la place de Maicie ou de l'humain.

## Exigences fonctionnelles

### Soumission et exécution

- **FR-001** : chaque travail logique doit posséder une identité durable
  distincte de ses tentatives de transport et de ses identifiants fournisseur.
- **FR-002** : chaque soumission doit déclarer son origine et son intention de
  traitement sans déduction fondée uniquement sur le nom de l'expéditeur ou
  l'état occupé.
- **FR-003** : le système doit distinguer au minimum l'acceptation durable, la
  réception par l'agent, l'acceptation fournisseur, la visibilité dans le tour
  et l'issue de la livraison.
- **FR-004** : le système doit distinguer au minimum les états en file, en
  démarrage, en cours, en attente humaine, en interruption, interrompu, réussi,
  échoué et injoignable.
- **FR-005** : aucune acceptation de commande fournisseur ne doit être
  présentée comme preuve de consommation sans événement corrélé défini par le
  contrat du fournisseur.
- **FR-006** : toute livraison doit atteindre une issue bornée : acquittée,
  rejouable, refusée, indéterminée ou orpheline.
- **FR-007** : toute transition doit conserver les identifiants de mission,
  soumission, livraison, agent, session, tour et message disponibles.
- **FR-008** : les messages non consommés doivent conserver leur ordre selon
  une politique de priorité déclarée et observable.
- **FR-009** : une sortie tardive ne doit jamais solder ou polluer une autre
  exécution.
- **FR-010** : le système doit publier des raisons structurées pour les refus,
  replis, attentes et terminaisons anormales.

### Flotte et propriété

- **FR-011** : chaque agent créé pour un travail doit conserver son parent, son
  rôle, son mandat et son travail propriétaire lorsque ces informations
  existent.
- **FR-012** : les liens parent-enfant doivent posséder un cycle de vie durable
  et distinct du simple état de connexion des processus.
- **FR-013** : les opérations d'envoi passif, de relance active,
  d'interruption et d'attente doivent conserver des intentions distinctes.
- **FR-014** : les limites de profondeur, de nombre et de capacité doivent être
  appliquées avant toute création partielle.
- **FR-015** : la disparition d'un parent ou d'un enfant doit conduire à une
  règle explicite de conservation, transfert, interruption ou clôture du lien.

### Reprise et capacités

- **FR-016** : chaque exécution doit pouvoir conserver une référence au fil et
  au dernier tour fournisseur lorsque celui-ci les expose.
- **FR-017** : le système doit distinguer une reprise native, une bifurcation
  native et une reconstruction textuelle.
- **FR-018** : aucune reprise native ne doit être tentée sans capacité attestée
  pour la version fournisseur réellement exécutée.
- **FR-019** : une reprise ou une bifurcation doit conserver l'ascendance, le
  mandat et les preuves antérieures sans réécriture de l'historique.
- **FR-020** : un échec de reprise doit produire une décision de repli ou un
  refus explicite, sans nouvelle identité silencieuse.

### Frontière Maicie et Bridget

- **FR-021** : Maicie doit rester l'autorité exclusive des objectifs,
  délégations, décisions, évaluations et clôtures métier.
- **FR-022** : Bridget doit rester l'autorité de la présence, du transport, du
  parc, des files et des faits d'exécution observés.
- **FR-023** : une délégation doit pouvoir référencer une exécution sans partager
  la base privée de l'autre composant.
- **FR-024** : une preuve d'exécution ne doit jamais provoquer seule une
  transition métier qui exige une évaluation ou une décision.
- **FR-025** : toute projection croisée doit être versionnée, atomique en lecture
  et explicite sur sa fraîcheur ou son indisponibilité.
- **FR-026** : Bridget doit pouvoir fonctionner sans lire les types ou l'état
  privé de Maicie.

### Fournisseurs, autorisations et sécurité

- **FR-027** : chaque session fournisseur doit publier une identité de binaire,
  une version et un ensemble de capacités observées.
- **FR-028** : chaque opération dépendante d'une capacité doit être refusée ou
  repliée explicitement lorsque cette capacité manque.
- **FR-029** : les demandes d'autorisation doivent être représentées comme des
  états corrélés et ne doivent pas être automatiquement assimilées à une
  approbation humaine.
- **FR-030** : les boucles d'autorisation répétées doivent être détectées et
  bornées avec une raison machine exploitable.
- **FR-031** : les identités, intentions et commandes de contrôle doivent être
  validées aux frontières avant effet irréversible.
- **FR-032** : les événements bruts et leur provenance doivent être préservés
  lorsqu'une projection canonique est produite.

### Observabilité et autonomie

- **FR-033** : l'opérateur doit voir séparément la connexion du wrapper, la
  vitalité fournisseur, l'état du tour, l'âge du dernier progrès et la file.
- **FR-034** : le système doit mesurer au minimum volume, latence, erreurs et
  saturation pour les livraisons et exécutions.
- **FR-035** : les logs, métriques et traces doivent partager les identifiants
  de corrélation utiles sans exposer de secret ni de contenu sensible par
  défaut.
- **FR-036** : les alertes doivent détecter les messages vieillissants, les
  tours sans progrès, les boucles d'autorisation et les files saturées avant
  une plainte humaine.
- **FR-037** : le coût de temps et d'usage doit pouvoir être agrégé sur une
  exécution et ses descendants.
- **FR-038** : une politique de budget doit distinguer pause, blocage, limite
  d'usage, limite de budget et terminaison.
- **FR-039** : une continuation automatique ne doit être possible qu'après une
  preuve d'inactivité, une politique explicite et l'absence de travail
  concurrent incompatible.

### Compatibilité et vérification

- **FR-040** : les contrats publics doivent posséder une version et une forme
  vérifiable par les producteurs, consommateurs et bancs de test.
- **FR-041** : les faux fournisseurs doivent être validés contre le même contrat
  que les fournisseurs réels et ne doivent pas inventer d'égalité entre
  identifiants distincts.
- **FR-042** : les changements doivent préserver le rejeu idempotent exact, les
  générations d'instances et les octets bruts déjà garantis.
- **FR-043** : chaque incrément doit être activable et réversible sans dépendre
  du cycle de vie d'un worktree.
- **FR-044** : les migrations doivent conserver une compatibilité explicite ou
  refuser proprement les anciennes projections, versions et événements.

## Exigences non fonctionnelles

- **NFR-001 - Durabilité** : un redémarrage à toute frontière ne doit ni perdre
  une soumission acceptée ni produire une seconde exécution silencieuse.
- **NFR-002 - Bornage** : toute attente de transport, fournisseur,
  autorisation, interruption et reprise doit avoir une borne et une issue
  observable.
- **NFR-003 - Maintenabilité** : chaque concept durable doit avoir une autorité,
  un cycle de vie et un point de vérification explicites.
- **NFR-004 - Minimalisme** : aucun moteur de workflow générique, framework
  multi-agent externe ou abstraction sans usage réel ne doit être ajouté.
- **NFR-005 - Sécurité** : les actions de contrôle doivent appliquer le moindre
  privilège compatible avec le modèle local déclaré et refuser par défaut les
  identités ou capacités incohérentes.
- **NFR-006 - Confidentialité** : la télémétrie et les projections ne doivent pas
  exposer le raisonnement, les secrets ou le corps intégral des messages sans
  finalité explicitement autorisée.
- **NFR-007 - Charge cognitive** : l'état présenté à l'opérateur doit expliquer
  la situation courante, sa preuve et la prochaine action possible sans lecture
  obligatoire des journaux bruts.

## Cas limites

- Le wrapper reste connecté tandis que le processus fournisseur est bloqué.
- Le fournisseur accepte une commande mais n'émet jamais la preuve attendue.
- Une preuve arrive après expiration, interruption ou remplacement du tour.
- Plusieurs messages humains et plusieurs rondes système arrivent pendant le
  même tour bloqué.
- Le daemon redémarre après réception par le wrapper mais avant acquittement.
- Une ancienne génération tente de terminer le travail d'une nouvelle.
- Deux versions du même fournisseur exposent des capacités différentes.
- Un fournisseur annonce une capacité mais la refuse pour un type particulier
  de tour.
- Une boucle d'autorisation survient après une décision déjà rendue.
- Un parent disparaît pendant que ses enfants continuent à travailler.
- Un enfant termine après la clôture ou le retrait de son mandat.
- Une reprise native retrouve un historique mais pas son dernier usage ou état.
- Une reconstruction textuelle est la seule reprise disponible.
- Le snapshot Bridget est périmé ou indisponible pendant une décision Maicie.
- Une limite d'usage fournisseur et une limite métier sont atteintes en même
  temps.
- Une projection inconnue ou future est lue par une ancienne version.

## Hors périmètre

- Prescrire la méthode de raisonnement interne ou les étapes cognitives d'un
  agent.
- Introduire un moteur générique de DAG ou de workflow.
- Remplacer les objectifs, délégations et décisions Maicie par un goal attaché
  à un thread fournisseur.
- Fusionner les bases privées Bridget et Maicie.
- Imposer les opérations propres à Codex aux fournisseurs qui ne les exposent
  pas.
- Importer ou réécrire l'ensemble du code Codex dans Bridget.
- Fournir une isolation hostile multi-utilisateur non prévue par le modèle de
  confiance actuel.
- Déployer, redémarrer ou migrer la production dans la phase de spécification.

## Entités métier

- **Soumission de travail** : demande logique durable, indépendante de ses
  tentatives de transport.
- **Livraison** : tentative de remise durable à une instance épinglée.
- **Exécution** : cycle runtime par lequel un agent traite une soumission.
- **Référence d'exécution** : lien stable entre mission, agent et identifiants
  fournisseur disponibles.
- **Session fournisseur** : contexte d'exécution offert par un fournisseur et
  décrit par sa version et ses capacités.
- **Tour fournisseur** : unité d'activité corrélée à une exécution.
- **Lien d'agent** : relation durable entre parent, enfant, mandat et rôle.
- **Projection d'exécution** : vue fraîche, périmée ou indisponible des faits
  runtime consommables par Maicie et l'interface.
- **Politique d'exécution** : limites, permissions et stratégies de repli
  applicables à un travail.
- **Limites d'exécution de mission** : contraintes de temps, d'usage et de
  descendants décidées dans Maicie puis soumises à l'admission technique de
  Bridget, sans effet runtime direct.

## Critères de succès

- **SC-001** : dans 100 % des scénarios d'acceptation, une soumission humaine
  atteint une issue observable dans la borne déclarée, sans état en cours
  infini.
- **SC-002** : aucun scénario ne produit d'acquittement à partir du seul accusé
  d'une commande fournisseur.
- **SC-003** : après redémarrage à chacune des frontières testées, aucune
  soumission n'est perdue et aucune exécution supplémentaire n'est créée sans
  décision visible.
- **SC-004** : un opérateur peut identifier en moins de 30 secondes le travail
  courant, l'attente éventuelle, l'âge du dernier progrès et la prochaine
  action autorisée d'un agent.
- **SC-005** : 100 % des agents créés pour une délégation possèdent une
  ascendance et un propriétaire visibles, ou un motif explicite d'absence.
- **SC-006** : 100 % des reprises indiquent si elles sont natives, bifurquées ou
  reconstruites et conservent les références disponibles.
- **SC-007** : aucun test ne ferme, ne rouvre ou ne réécrit un objectif Maicie à
  partir du seul état de transport ou de présence.
- **SC-008** : chaque version fournisseur supportée passe un contrat de
  capacités et d'identifiants avant activation des opérations dépendantes.
- **SC-009** : les tableaux de bord permettent de détecter un message
  vieillissant, un tour sans progrès, une boucle d'autorisation et une file
  saturée sans ouvrir le journal brut.
- **SC-010** : chaque exigence fonctionnelle possède au moins une tâche et une
  vérification observable avant le passage à l'implémentation.
- **SC-011** : aucune nouvelle dépendance externe ou abstraction transversale
  n'est retenue sans réutilisation auditée et justification de maintenance.

## Hypothèses

- Le modèle de confiance reste local, coopératif et mono-utilisateur tant
  qu'une feature de sécurité distincte ne le modifie pas.
- Bridget et Maicie conservent leurs bases et responsabilités propres.
- Les fournisseurs peuvent exposer des ensembles de capacités différents et
  évoluer indépendamment.
- Cursor est un fournisseur distinct utilisant le chemin d'exécution ACP
  existant ; `provider_kind=cursor` et `execution_path=acp` ne sont jamais
  fusionnés en une seule identité.
- Les mécanismes existants de livraison, idempotence, génération et journal
  restent la base à étendre.
- La session 063 est finalisée et prouvée avant d'activer les fonctions qui en
  dépendent.
- Le programme peut être livré en incréments indépendants, chacun conservant un
  état buildable, observable et réversible.

## Dépendances et continuité avec les specs existantes

- **SPEC-003** : cycle de vie durable des demandes.
- **SPEC-004** : modèle runtime des agents.
- **SPEC-009** : création et cycle de vie des agents par le daemon.
- **SPEC-011** : frontière initiale de Maicie comme compagnon d'orchestration.
- **SPEC-012** : contrat client et rejeu idempotent.
- **SPEC-014** et **SPEC-023** : observabilité et refus de tours.
- **SPEC-015** et **SPEC-016** : guichet Maicie et coordination active.
- **SPEC-034** : macrographe de contrôle et refus machine aux frontières
  critiques.
- **SPEC-046** : distinction entre connexion et activité.
- **SPEC-048** : identité de l'instance daemon.
- **SPEC-049** : état de mission indépendant du cache Bridget.
- **SPEC-052** : notification de délégation corrélée.
- **SPEC-063** : interruption et pilotage humain d'un tour actif.

## Risques

- Une abstraction trop générale pourrait recréer un moteur de workflow que la
  décision SPEC-034 a explicitement écarté.
- Une migration large et simultanée pourrait affaiblir l'idempotence existante.
- Des bancs de test non alignés sur les schémas fournisseurs pourraient donner
  de faux verdicts positifs.
- Une projection trop riche pourrait dupliquer la vérité Maicie ou Bridget.
- Une télémétrie mal bornée pourrait exposer du contenu sensible ou augmenter
  fortement la cardinalité.
- Une autonomie activée avant l'observabilité fiable pourrait amplifier les
  boucles et la consommation de ressources.
