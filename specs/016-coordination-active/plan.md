# Plan 016 — Coordination active : messagers, dépendances et réassignation

**Branche** : `session-16-coordination-active` | **Date** : 2026-08-23
**Spec** : `specs/016-coordination-active/spec.md`
**Statut** : conception amendée après revue hostile ; tâches et code non autorisés avant re-review

## Résumé

La session automatise trois gestes mécaniques du référent : annoncer un
événement attendu, ouvrir les délégations dont les prérequis sont terminés et
réassigner après un seuil de relances attestées sans livraison.

Le cœur est un réducteur déterministe Maicie. Il reçoit un événement corrélé du
guichet 015, lit un snapshot transactionnel du registre, calcule une transition
pure, puis écrit dans une transaction unique la décision, la transition et les
outboxes dues. Les envois arrivent ensuite par le contrat idempotent existant.
Bridget reste l'horloge et l'autorité transport ; Maicie ne poll pas, ne lit pas
de texte et ne lance aucun agent.

## Contexte technique

- **Langage et cible** : Rust 1.92.0, CLI/daemon macOS et Linux.
- **Stockage** : SQLite privée Maicie pour objectifs, délégations, politiques,
  événements et outboxes ; stores Bridget existants inchangés hors extension
  contractuelle attestée.
- **Dépendances** : session 015 gelée, contrat client idempotent 012, classes de
  délai de la configuration Maicie 011.
- **Tests** : tests Rust unitaires, contrats producteur-consommateur, SQLite à
  deux connexions, processus réellement tués aux frontières de crash et gate
  de bout en bout.
- **Performance** : traitement indexé et proportionnel aux arêtes ou
  destinataires concernés ; aucune analyse globale répétée du registre.

## Décisions d'architecture

### D-1601 — Un réducteur pur, aucune boucle d'autonomie cachée

Une relève du guichet fournit un `ÉvénementCoordination` versionné. Le réducteur
prend `(snapshot registre, événement, politique épinglée)` et rend une liste
fermée d'effets : `Aucun`, `Notifier`, `Ouvrir`, `Réassigner` ou
`InterventionHumaineRequise`. Il ne lit ni réseau, ni horloge, ni texte libre.

Cette forme rend FR-022 falsifiable : le même triplet d'entrée produit les
mêmes octets de décision. Elle conserve aussi le modèle compagnon de Maicie :
aucun worker permanent n'est créé.

### D-1602 — Transaction locale unique, outbox avant toute I/O

Le store applique une transition sous transaction immédiate : consommation
idempotente de l'événement, contrôle de génération, écriture de la décision,
mutation de la délégation et création des outboxes. Aucun envoi n'a lieu avant
le commit. Après commit, le dispatcher rejoue l'enveloppe immuable avec son
identifiant stable jusqu'à issue durable.

Les clés sont dérivées d'identifiants durables, pas du contenu rendu :

- notification F27 : `(event_id, recipient_id, policy_version)` ;
- ouverture F28 : `(delegation_id, generation, opened_event_id)` ;
- réassignation F29 : `(delegation_id, source_generation, threshold_event_id,
  effect_kind, recipient_id?)`, afin que l'annulation, la nouvelle demande et
  les deux notifications aient chacune une clé stable distincte.

Un crash avant commit ne laisse aucun effet ; après commit, toute reprise voit
exactement les mêmes outboxes.

### D-1603 — Graphe local par objectif, qualification épinglée par arête

Le graphe est un DAG borné entre délégations d'un même objectif. Les arêtes
sont ajoutées avant l'ouverture du dépendant et portent un mode fermé :
`hash_greffé` par défaut ou `clôture_évaluée_exigée`. Le dépendant s'ouvre
lorsque tous ses prérequis satisfont leur mode. Annulation et échec restent des
blocages explicites.

Le défaut `hash_greffé` est un choix mécanique et minimal : il prouve qu'une
livraison corrélée a été greffée, pas qu'un humain l'a évaluée. Il constitue
donc un écart explicite à la règle stricte de vérification avant relais. Le
déclarant qui sait que son arête exige cette preuve choisit
`clôture_évaluée_exigée` ; Maicie ne l'infère jamais du corps du rapport.

Une règle `any`, des conditions booléennes ou des dépendances inter-objectifs
seraient un langage de workflow nouveau sans incident fondateur. Elles sont
écartées par minimalisme.

La validation de cycle parcourt uniquement le sous-graphe de l'objectif lors
de l'ajout. L'évaluation d'une fin utilise un index `prérequis → dépendants` et
ne rescane pas tous les objectifs.

### D-1604 — Politique et admissibilité épinglées par délégation

La configuration associe à chaque classe de délai un seuil `N > 0`. À la
création, la délégation reçoit un snapshot immuable comprenant version, seuil
et chaîne ordonnée de participants de repli. Chaque entrée épingle aussi les
faits Maicie qui l'autorisent : `objective_id`, `participant_id`, version
d'appartenance au registre et exclusion du pilote. Un changement de
configuration ou d'annuaire ne modifie que les nouvelles délégations.

Ce snapshot supprime une double source de vérité : la configuration courante
explique le futur ; le snapshot explique toute décision passée ou rejouée.
La v1 choisit uniquement à partir de ces faits de registre. Elle ne consulte
pas une présence ou disponibilité Bridget volatile pour départager ou sauter
un candidat ; l'issue de livraison ultérieure reste une vérité transport
séparée.

### D-1605 — Réassignation par génération, jamais par remplacement en place

Une délégation possède une lignée. Au seuil, la transaction vérifie que la
génération source est toujours active et sans livraison terminale, la clôt
comme `réassignée`, puis crée une génération successeur vers le premier
candidat admissible de la chaîne épinglée.

L'admissibilité est fermée : participant du même objectif, différent du pilote
et de la génération source, présent dans le snapshot épinglé, non déjà consommé
par la lignée. Si aucun candidat ne satisfait ces faits, Maicie écrit
`intervention_humaine_requise`. Elle n'appelle jamais les voies profil,
approbation ou spawn.

La même transaction locale crée quatre effets immuables : une intention
`CancelRequest` pour la demande suivie source, une nouvelle demande suivie avec
sa propre échéance pour la génération successeur, une notification au
participant sortant et une notification au successeur. Il ne s'agit pas d'une
transaction distribuée : le commit rend ces quatre outboxes indissociables,
puis leur livraison idempotente fait converger Bridget. Une génération 2 est
donc observable et relançable selon F29 comme la génération 1.

Une livraison tardive de l'ancienne génération est conservée comme fait tardif
et ne réactive pas cette génération.

Une annulation ou clôture administrative gagnante inhibe immédiatement le
compteur, prépare l'annulation de la demande suivie et notifie le participant
sortant. Elle ne crée jamais de successeur.

### D-1606 — Bridget garde l'horloge et atteste les relances

Maicie ne calcule jamais « il est temps de relancer ». Bridget émet déjà les
relances et possède leurs échéances. La session exige un événement de guichet
`reminder_sent` corrélé à la demande, au message de relance, au destinataire et
à son instant. Seuls des `event_id` distincts comptent.

Un `answered` attesté et corrélé à une relance de la génération active ne vaut
pas livraison, mais termine l'épisode de silence : le compteur consécutif est
remis à zéro. Une relance distincte postérieure le réarme à un. Cette règle
protège une réponse réelle sans transformer son contenu en preuve de livraison.

La relève normalise chaque lot avant réduction : tous les `delivery_report`
corrélés d'une génération sont appliqués avant ses `reminder_sent`, même si le
fil les présente dans l'ordre inverse. Une livraison présente dans le même lot
gagne donc la course. La normalisation ne réordonne jamais des lots déjà
committés.

Si 015 ne publie pas encore cet événement, le lot contrat l'ajoute de manière
versionnée. Une lecture directe du ledger, une déduction depuis l'âge ou un
timer Maicie sont interdites.

### D-1607 — Deux vérités, fraîcheur comme garde et non comme état métier

Le réducteur reçoit séparément :

1. le fait local Maicie (objectif, délégation, lignée, dépendances, politique) ;
2. l'observation Bridget attestée (événement, transport et fraîcheur).

Une observation `Gap`, `Unavailable` ou non rattrapée bloque l'effet et produit
une issue visible `observation_incomplète`. Elle ne transforme pas une
délégation en échec ou en disponibilité supposée. Une reprise depuis le curseur
du guichet réévalue ensuite le même événement.

### D-1608 — Notifications ciblées, pas de diffusion transport

Maicie développe la liste concrète des destinataires depuis son registre et
crée une outbox par destinataire. Bridget reste point-à-point. Chaque envoi
porte la référence de l'événement, de l'objectif et de la délégation, mais son
corps humain n'est jamais relu pour produire un effet.

Cette décision réutilise le transport existant et évite une primitive de groupe
qui dupliquerait la connaissance détenue par Maicie.

## Modèle de flux

```text
Bridget, autorité temps/transport
  └── événement guichet attesté et corrélé
          │ relève bornée, curseur durable
          ▼
Maicie, transaction immédiate
  ├── déduplique event_id
  ├── charge faits locaux + politique épinglée
  ├── applique le réducteur pur
  └── écrit décision + transition + outbox(s)
          │ après commit uniquement
          ▼
dispatcher d'outbox idempotent
  └── notification ciblée / issue durable / replay exact
```

### Flux F27 — événement attendu

1. La clôture d'objectif est validée et reçoit un `event_id` durable.
2. La transaction clôt l'objectif et crée une outbox pour chaque attente
   déclarée avant cette clôture.
3. Le dispatcher envoie les notifications. Un accusé perdu rejoue la même clé.

La v1 ne produit pas la clôture de l'objectif : elle automatise uniquement le
messager dû après cette clôture attestée, ainsi que le messager d'ouverture de
délégation. Cette couverture partielle de F27 est mesurée telle quelle.

### Flux F28 — déblocage

1. Une greffe de hash ou une clôture évaluée satisfait le mode épinglé de
   l'arête du prérequis.
2. Le store charge seulement les dépendants indexés par ce prérequis.
3. Pour chacun, il vérifie tous les prérequis dans le même snapshot.
4. Le dernier prérequis ouvre la délégation et crée sa notification ; les
   événements concurrents suivants observent l'état déjà ouvert.

### Flux F29 — réassignation

1. Bridget dépose `reminder_sent`, `answered` et `delivery_report` corrélés au
   guichet.
2. Maicie priorise les rapports du lot, déduplique les événements et compte les
   relances consécutives de la génération active ; `answered` remet ce compte à
   zéro.
3. Avant le seuil : aucun effet. Au seuil : la transaction arbitre entre une
   livraison déjà durable, un successeur admissible ou l'intervention humaine.
4. Le commit prépare l'annulation source, la demande suivie successeur et les
   notifications au sortant et au successeur ; le dispatcher les fait
   converger après commit.

## Frontières de propriété

| Élément | Autorité | Interdit |
|---|---|---|
| échéance, relance envoyée, message, demande, transport, fraîcheur | Bridget | choisir le successeur ou ouvrir une délégation |
| objectif, DAG, états de délégation, lignée, politique et candidats épinglés, décision | registre Maicie | fabriquer une relance, un participant ou une observation transport |
| notifications | outbox Maicie + livraison Bridget | effet avant commit ou reconstruction depuis le corps humain |
| profils et naissance d'agents | approbation locale FR-014 | accès par F27, F28 ou F29 |

## Constitution check

| Gate | État | Justification |
|---|---|---|
| FR-022 / zéro jugement | PASS | réducteur fermé, entrées structurées, aucune lecture de texte ni modèle |
| FR-014 / approbation | PASS | réassignation limitée aux participants préexistants ; épuisement → intervention humaine |
| Deux vérités | PASS | événements/fraîcheur Bridget et états Maicie restent séparés dans les décisions |
| État durable | PASS sous tests de crash | décision, transition et outbox partagent une transaction locale ; livraison reprise par idempotence |
| Minimalisme XIX | PASS | DAG `tous prérequis satisfaits`, modes d'arête fermés, chaîne déclarée et événements attestés ; aucun moteur de workflow générique |
| Complexité XVIII | PASS sous index | traitement proportionnel aux destinataires/dépendants concernés, sans scan global ni polling |
| Responsabilité future XX | PASS | politiques versionnées et explicables ; chaque automatisme possède un oracle de mutation |
| Compatibilité | PASS sous négociation | l'événement de relance est additif ; sans politique active, aucun comportement historique ne change |

## Gate d'entrée G-1600

Le lot A reste fermé jusqu'à ce que le contrat 015 soit mergé, déclaré gelé et
que ses fixtures publiques soient pinnées. L'éventuel `reminder_sent` absent
est alors une extension 016 versionnée et relue, jamais une interprétation
locale d'un message 015. Aucun code A, B ou C ne peut consommer un DTO
provisoire avant cette gate.

## Fermeture de la revue hostile

| Constat | Décision vérifiable |
|---|---|
| C1, C5b | La transaction F29 crée l'annulation source, la demande suivie successeur et les notifications au sortant et au successeur. |
| C2 | Chaque arête épingle `hash_greffé` par défaut ou `clôture_évaluée_exigée` ; l'écart du mode par défaut avec la vérification humaine est écrit. |
| C3 | La v1 choisit uniquement dans le snapshot des faits du registre Maicie épinglé, jamais depuis une disponibilité Bridget volatile. |
| C4 | Un `answered` corrélé remet le compteur consécutif à zéro ; une relance ultérieure le réarme. |
| C5a | Une annulation ou clôture administrative inhibe le compteur et ne peut produire de successeur. |
| C6 | Un `delivery_report` du même lot est réduit avant les relances de sa génération, indépendamment de l'ordre filaire. |
| C7 | G-1600 interdit le lot A avant merge et gel du contrat 015. |
| C8 | F27 v1 est explicitement partielle : elle transporte la clôture, elle ne la décide pas. |
| C9 | Les clés F29 distinguent chaque effet et les bornes anti-tempête restent fermées par génération et destinataire. |
| C10 | Le lot B possède dans `store.rs` la couture de toutes les clôtures vers la transaction FR-1602 et son oracle de mutation. |

## Structure projet envisagée

```text
specs/016-coordination-active/
├── spec.md
├── plan.md
├── data-model.md                 # après validation du plan
├── contracts/
│   └── evenements-coordination.md
├── quickstart.md
└── tasks.md

crates/bridget-transport/src/protocol.rs
crates/bridget-daemon/src/daemon.rs
crates/bridget-daemon/src/store.rs
plugins/maicie/src/domain.rs
plugins/maicie/src/store.rs
plugins/maicie/src/app.rs
plugins/maicie/src/config.rs
plugins/maicie/src/bridget_client.rs
plugins/maicie/src/reconcile.rs
plugins/maicie/src/main.rs
```

Le plan ne crée aucun nouveau crate ni dépendance. Les fichiers communs
`Cargo.toml` et `lib.rs` restent hors couloir ; une tâche d'intégration unique
les possède seulement si le branchement final l'exige.

## Découpage d'implémentation — trois couloirs disjoints

### Lot A — Contrat d'événements Bridget (codeur A)

**Propriété exclusive** : `crates/bridget-transport/src/protocol.rs`, portions
guichet de `crates/bridget-daemon/src/daemon.rs` et
`crates/bridget-daemon/src/store.rs`, fixtures de contrat Bridget.

**Livrables** :

- événement versionné `reminder_sent` si absent du contrat 015 final ;
- corrélation demande/message/destinataire, canon, déduplication et reprise ;
- curseur/fraîcheur exposés sans lecture directe de base par Maicie ;
- corpus de compatibilité prouvant que les clients sans capacité 016 restent
  inchangés.

**Gate A** : producteur et consommateur partagent les mêmes fixtures ; crash
avant/après persistance ne perd ni ne double un événement ; aucun texte libre
ne peut se faire passer pour une relance.

### Lot B — Domaine, DAG et transactions Maicie (codeur B)

**Propriété exclusive** : `plugins/maicie/src/domain.rs`,
`plugins/maicie/src/store.rs` et tests de contrat du registre.

**Livrables** :

- migrations idempotentes pour événements, arêtes, politiques, lignées et
  notifications ;
- réducteur pur et matrice fermée des effets ;
- transaction décision/transition/outbox ;
- couture de **tous** les chemins de clôture d'objectif existants vers une API
  commune qui écrit la clôture et les outboxes FR-1602 dans la même transaction
  de `store.rs` ; aucun appelant ne peut clôturer puis notifier séparément ;
- validation DAG, index inversé et arbitrage atomique livraison/réassignation ;
- modes de qualification par arête, épisodes de relance, annulation
  administrative et cycle complet des demandes inter-générations ;
- crashs réels et tests SQLite concurrents.

**Gate B** : une mutation séparant transition et outbox, contournant la couture
FR-1602 depuis un chemin de clôture existant, retirant le contrôle de
génération ou autorisant un cycle fait échouer un test nommé.

### Lot C — Application, configuration et gate de bout en bout (codeur C)

**Propriété exclusive** : `plugins/maicie/src/config.rs`,
`plugins/maicie/src/app.rs`, `plugins/maicie/src/bridget_client.rs`,
`plugins/maicie/src/reconcile.rs`, `plugins/maicie/src/main.rs` et tests
d'intégration.

**Livrables** :

- validation des seuils par classe et des chaînes de repli ;
- relève bornée et passage des faits structurés au réducteur B ;
- dispatcher des outboxes et projections CLI/JSON des issues ;
- gate rejouant les trois incidents fondateurs, y compris arrêt/reprise des
  processus aux quatre frontières ;
- mesures SC-1601 à SC-1608 et documentation d'exploitation.

**Gate C** : aucun polling ni timer Maicie ; le gate échoue si une notification
manuelle est nécessaire, si une observation incomplète déclenche un effet ou
si une voie de politique atteint profil/approve/spawn.

### Discipline d'intégration

1. G-1600 exige le contrat 015 mergé et gelé ; A fige ensuite l'extension 016 et
   ses fixtures avant consommation par C.
2. B publie les DTO et signatures du store avant le branchement de C ; C ne
   modifie jamais `store.rs` ou `domain.rs`.
3. Les couloirs valident leurs cibles pendant le travail ; le workspace complet
   est la gate d'intégration finale.
4. Chaque lot est relu par un agent différent de son auteur.
5. La revue hostile finale mute séparément : événement attesté, cycle du DAG,
   seuil, génération active, fraîcheur, transaction outbox et FR-014.

## Traçabilité exigences → décisions → couloirs

| Exigences | Décisions | Propriétaire | Gate principal |
|---|---|---|---|
| FR-1601, FR-1602 | D-1602, D-1608 | B puis C | couture de toutes les clôtures + outboxes atomiques, crashs et zéro doublon |
| FR-1603 à FR-1605 | D-1602, D-1603 | B | corpus DAG, concurrence du dernier prérequis, notification unique |
| FR-1606 | D-1601, D-1606 | A puis C | relance/answered attestés, reset du compteur, texte libre refusé |
| FR-1607 à FR-1609, FR-1608a | D-1604 à D-1606 | B puis C | candidats registre épinglés, priorité du lot, cycle complet des demandes, aucun accès profil/spawn |
| FR-1610 | D-1607 | A et C | Gap/Unavailable/non frais bloquent tout effet automatique |
| FR-1611 | D-1601 | B | replay déterministe et mutations d'oracle sans réseau/horloge |
| FR-1612 | D-1605 | B et C | routes d'approbation structurellement inaccessibles |
| FR-1613, FR-1614 | D-1602 | A, B et C | quatre frontières de crash et journal corrélé complet |
| FR-1615 | D-1606, D-1608 | A puis gate final | corpus 011/015 inchangé sans capacité/politique 016 |

## Stratégie de vérification

### Tests de contrat

- round-trip et refus des événements 016 ;
- compatibilité des messages 015 sans champ 016 ;
- seuils/configurations invalides refusés avant mutation ;
- snapshot de politique stable après modification de configuration.

### Tests transactionnels

- deux connexions concurrentes terminent deux prérequis du même dépendant ;
- livraison et `N`e relance courent sur la même génération ;
- `delivery_report` et `reminder_sent` apparaissent dans le même lot dans les
  deux ordres filaires et la livraison gagne dans les deux cas ;
- `answered` remet le compteur à zéro, une relance postérieure le réarme et une
  annulation administrative interdit toute réassignation ;
- une réassignation crée atomiquement annulation source, demande successeur et
  notifications aux deux participants ;
- deux relèves consomment le même événement ;
- faute injectée après décision mais avant transition/outbox entraîne rollback
  total, puis reprise unique.

### Crashs réels

Un processus enfant est arrêté à des barrières déterministes : avant commit,
après commit avant envoi, après écriture socket avant accusé et après accusé
avant consommation. La réouverture doit montrer le même terminal, les mêmes
octets et le même nombre d'effets.

### Gate mécanique des incidents

Le gate versionné rejoue :

1. objectif clôturé sans référent actif → destinataires réveillés ;
2. interface livrée pendant l'attente de prospective → dépendant ouvert ;
3. participant silencieux après deux relances attestées → successeur déclaré
   ouvert, ou intervention humaine si la chaîne est épuisée.

Le gate n'autorise aucun message manuel entre l'injection de l'événement et
l'observable final.

## Critères de sortie

- F27, F28 et F29 satisfont leurs critères mesurables et leurs mutations
  d'oracle.
- Aucune transition ne dépend d'un texte, d'un modèle, d'un timer Maicie ou
  d'une observation non fraîche.
- Aucune politique ne peut appeler les voies d'approbation ou de spawn.
- Les quatre frontières de crash convergent sans perte ni double effet.
- Le graphe et les index respectent la borne linéaire annoncée.
- Le corpus 015 reste vert et le gate 016 reproduit les trois incidents sans
  intervention manuelle.

## Hors périmètre du plan

La session ne construit ni moteur BPMN, ni règles booléennes libres, ni
scheduler, ni routines récurrentes, ni dépendances inter-objectifs, ni
priorisation globale, ni GUI. Elle n'étend pas l'approbation et ne choisit pas
un modèle ou un profil. Ces sujets exigeraient chacun un incident fondateur et
une session séparée.
