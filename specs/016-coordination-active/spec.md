# Spec 016 — Coordination active : messagers, dépendances et réassignation

**Branche** : `session-16-coordination-active`
**Créée le** : 2026-08-23
**Statut** : conception amendée après revue hostile ; re-review requise
**Source** : politiques 1 à 3 de
`specs/011-maicie-orchestration/exigences-coordination-v2.md`

## Contexte et problème

La coordination manuelle des sessions 007 à 015 a démontré trois pertes de
temps récurrentes. Une livraison réelle restait muette tant qu'un référent
n'annonçait pas son hash. Un équipier dépendant restait en attente après la
fin de son prérequis tant que le référent ne le réveillait pas. Après plusieurs
relances factuelles sans livraison, le référent devait enfin réassigner la
tâche à la main.

Ces gestes sont mécaniques. Ils ne demandent ni compréhension du contenu, ni
évaluation de qualité, ni choix créatif. La session 016 les transforme en
politiques déterministes de Maicie : chaque effet est une fonction pure de
faits durables du registre, d'événements Bridget attestés et d'une politique
configurée à l'avance.

La session s'appuie sur le guichet public 015 pour recevoir les événements de
demande et transmettre les notifications. Elle ne rend pas Maicie résidente :
les événements sont traités lors d'une relève bornée, et leur reprise est
assurée par les outboxes durables existantes.

## Principes non négociables

1. **FR-022 reste absolue** : aucun modèle, texte libre, résumé ou jugement ne
   déclenche une notification, un déblocage ou une réassignation.
2. **FR-014 reste absolue** : une politique ne peut ni approuver un profil, ni
   lancer un agent, ni fabriquer un participant. Une réassignation choisit
   seulement un participant déjà déclaré et autorisé pour l'objectif.
3. **Deux vérités restent séparées** : le registre Maicie porte objectifs,
   délégations, dépendances et décisions ; Bridget porte transport, demandes,
   relances et fraîcheur des observations. Aucun état n'est reconstruit depuis
   l'autre source.
4. **Bridget reste l'unique horloge active** : Maicie n'ajoute ni timer, ni
   polling, ni boucle résidente. Elle réagit à des événements datés et
   idempotents reçus au guichet.
5. **Une politique est épinglée** : sa version, ses seuils et sa chaîne de
   repli sont figés lors de la création de la délégation. Un changement de
   configuration ne réécrit pas le passé et ne modifie que les nouvelles
   délégations.
6. **La couverture F27 v1 est volontairement partielle** : elle automatise les
   messagers après clôture durable d'objectif et ouverture de délégation, mais
   ne clôt pas elle-même un objectif et ne prétend pas couvrir tout événement
   attendu. La clôture reste un acte attesté fourni à la politique.

## Scénarios utilisateurs et tests

### US1 — Réveiller les destinataires d'un événement attendu (P1)

Un coordinateur déclare, dans un objectif, quels participants attendent sa
clôture. Lorsque la clôture est durable, Maicie crée une notification pour
chacun des destinataires sans attendre un relais manuel.

**Pourquoi cette priorité** : elle supprime l'incident le plus fréquent : un
commit, un verdict ou une fin de banc existe, mais le travail suivant reste à
l'arrêt parce que personne n'a annoncé l'événement.

**Test indépendant** : clôturer un objectif possédant trois destinataires,
interrompre le processus avant envoi puis après envoi avant accusé, relancer la
relève et vérifier une notification durable par destinataire, sans doublon.

**Scénarios d'acceptation** :

1. **Étant donné** un événement attendu déclaré et trois destinataires,
   **quand** l'objectif atteint sa clôture durable, **alors** trois intentions
   de notification sont écrites dans la même transaction que la clôture.
2. **Étant donné** une notification déjà acceptée mais non acquittée localement,
   **quand** Maicie redémarre, **alors** elle rejoue les mêmes octets et le même
   identifiant, sans produire de deuxième message.
3. **Étant donné** un texte libre annonçant « terminé » sans transition durable,
   **quand** Maicie le reçoit, **alors** aucune notification ni transition n'est
   produite.

---

### US2 — Débloquer automatiquement une délégation dépendante (P1)

Un coordinateur déclare un graphe de dépendances entre les délégations d'un
même objectif. Une délégation bloquée devient ouverte uniquement lorsque tous
ses prérequis ont atteint l'état terminal qualifiant prévu par le contrat.
L'agent affecté reçoit alors une notification d'ouverture.

**Pourquoi cette priorité** : les attentes de prospective sur une interface ou
un gate déjà livré ont imposé des réveils manuels alors que Maicie possédait
déjà les relations entre délégations.

**Test indépendant** : déclarer un losange de quatre délégations, terminer les
prérequis dans plusieurs ordres et vérifier qu'aucune délégation ne s'ouvre
trop tôt, que chaque ouverture arrive une fois et que le résultat final est
indépendant de l'ordre des événements.

**Scénarios d'acceptation** :

1. **Étant donné** une délégation dépendant de deux prérequis, **quand** un seul
   est terminé, **alors** elle reste bloquée et aucune notification d'ouverture
   n'est créée.
2. **Étant donné** les deux prérequis terminés, **quand** le second terminal est
   greffé, **alors** la délégation passe une seule fois de `bloquée` à `ouverte`
   et son affectataire reçoit une notification durable.
3. **Étant donné** une dépendance formant un cycle, visant un autre objectif ou
   ajoutée après l'ouverture, **quand** elle est déclarée, **alors** elle est
   refusée avant mutation avec un motif typé.
4. **Étant donné** un prérequis annulé ou échoué, **quand** son terminal est
   greffé, **alors** le dépendant reste bloqué avec le motif attesté ; Maicie ne
   transforme pas un échec en réussite ni ne choisit une stratégie de repli.
5. **Étant donné** une arête utilisant le mode par défaut `hash_greffé`,
   **quand** le rapport de livraison et son hash sont greffés durablement,
   **alors** le prérequis qualifie sans prétendre que son contenu a été évalué.
6. **Étant donné** une arête déclarée `exiger_clôture_évaluée`, **quand** seul
   le hash de livraison est greffé, **alors** le dépendant reste bloqué jusqu'à
   la clôture évaluée durable correspondante.

---

### US3 — Réassigner après un nombre configuré de relances sans livraison (P1)

Une délégation porte une classe de délai, un seuil de relances et une chaîne
ordonnée de participants de repli. Chaque relance attestée par Bridget est
greffée au registre. Lorsque le seuil est atteint sans livraison durable,
Maicie clôt la génération active comme réassignée et crée exactement une
génération successeur pour le prochain participant admissible.

**Pourquoi cette priorité** : les creux d'agents constatés par l'utilisateur
ont exigé des relances manuelles, puis une réassignation manuelle après deux
relances sans commit. La règle était déjà stable ; seule son exécution manquait.

**Test indépendant** : pour chaque classe de délai, injecter `N-1`, puis `N`
relances attestées, une livraison concurrente et un crash à chaque frontière ;
vérifier zéro réassignation avant le seuil et une seule génération après.

**Scénarios d'acceptation** :

1. **Étant donné** un seuil `N`, **quand** `N-1` relances distinctes sont
   greffées sans livraison, **alors** la délégation reste affectée au même
   participant.
2. **Étant donné** la `N`e relance et une chaîne de repli valide, **quand**
   l'événement est traité, **alors** l'ancienne génération devient
   `réassignée`, une génération successeur est créée, l'annulation de la
   demande suivie source, la nouvelle demande suivie et les notifications au
   sortant et au successeur sont préparées dans une transaction unique.
3. **Étant donné** une livraison durable qui gagne la course avec la `N`e
   relance, **quand** les deux événements sont rejoués dans l'un ou l'autre
   ordre, **alors** une livraison déjà terminale n'est jamais réassignée.
4. **Étant donné** une chaîne épuisée, un candidat absent du snapshot de
   registre épinglé ou un événement Bridget non frais, **quand** le seuil est
   atteint, **alors**
   Maicie enregistre `intervention_humaine_requise`, notifie le référent et ne
   crée ni participant ni agent.
5. **Étant donné** une réponse `answered` corrélée à une relance de la
   génération active, **quand** elle est greffée, **alors** le compteur de
   relances consécutives revient à zéro ; seules les relances attestées
   postérieures peuvent le réarmer.
6. **Étant donné** une annulation ou clôture administrative de la délégation,
   **quand** elle gagne la transaction, **alors** le compteur est inhibé, la
   demande source reçoit une intention d'annulation et le participant sortant
   une notification durable ; aucune réassignation ultérieure n'est créée.

### Cas limites communs

- Un même événement reçu deux fois conserve le même résultat et ne recrée ni
  transition, ni notification, ni génération.
- Une observation Bridget en état `Gap`, `Unavailable` ou non rattrapée ne
  déclenche aucun déblocage ni réassignation ; la cause reste visible.
- Une clôture d'objectif concurrente avec l'ajout d'un destinataire produit
  soit la notification si l'ajout est durable avant la clôture, soit un refus
  terminal si la clôture a gagné, jamais une perte silencieuse.
- Une délégation réassignée qui livre tard conserve la livraison comme fait
  tardif lié à son ancienne génération ; elle ne reprend pas l'autorité sur la
  génération active.
- Une relance répétée avec le même identifiant ne compte qu'une fois.
- Dans un même lot relevé, un `delivery_report` corrélé à une génération est
  appliqué avant tout `reminder_sent` de cette génération, indépendamment de
  l'ordre filaire du lot. Une livraison présente dans le lot gagne donc avant
  le seuil ; deux lots déjà committés restent ordonnés par leurs transactions.
- Une réponse corrélée à une relance remet à zéro le compteur de la génération
  sans constituer une livraison. Une relance distincte ultérieure recommence
  un nouvel épisode à un.
- Une suppression ou modification de politique après création n'altère pas le
  snapshot épinglé de la délégation.

## Exigences fonctionnelles

- **FR-1601 — Événements attendus déclarés** : Maicie DOIT représenter les
  événements attendus et leurs destinataires dans le registre de l'objectif.
  La v1 couvre la clôture durable d'objectif et l'ouverture durable de
  délégation ; elle n'introduit pas de moteur d'événements générique.
- **FR-1602 — Messager transactionnel (F27)** : toute transition qualifiante
  DOIT écrire ses intentions de notification dans la même transaction que le
  fait qui les rend dues. L'envoi est idempotent et reprend les octets exacts.
- **FR-1603 — Graphe borné (F28)** : les dépendances sont déclarées uniquement
  entre délégations d'un même objectif, sans cycle, doublon ni modification
  rétroactive après ouverture. Chaque arête épingle un mode de qualification
  fermé : `hash_greffé` par défaut ou `clôture_évaluée_exigée`.
- **FR-1604 — Ouverture déterministe** : une délégation dépendante s'ouvre si
  et seulement si tous ses prérequis satisfont durablement le mode épinglé sur
  leur arête. Le mode par défaut atteste une greffe de hash, pas une évaluation
  humaine : cet écart à la règle de vérification avant relais est explicite.
  Une annulation, un échec ou une observation absente ne qualifient jamais
  implicitement.
- **FR-1605 — Notification de déblocage** : l'ouverture d'une délégation écrit
  une notification durable à son participant dans la transaction de
  transition. Son rejeu ne peut ni rouvrir la délégation ni renvoyer un autre
  message.
- **FR-1606 — Relances attestées (F29)** : seules les relances émises par
  Bridget et reçues comme événements versionnés, corrélés et idempotents
  peuvent alimenter le compteur de politique. Le temps local et le texte libre
  ne comptent jamais. Un `answered` corrélé remet le compteur de relances
  consécutives à zéro sans valoir livraison ; une annulation ou clôture
  administrative l'inhibe définitivement pour cette génération.
- **FR-1607 — Politique par classe** : chaque classe de délai définit un seuil
  entier strictement positif et une stratégie de repli déclarative. La version,
  le seuil, la chaîne ordonnée et les faits d'appartenance au registre Maicie
  sont épinglés à la délégation avant toute I/O. La v1 ne consulte pas la
  disponibilité volatile de candidats tiers dans Bridget.
- **FR-1608 — Réassignation linéarisée** : au seuil, Maicie DOIT décider dans
  une transaction unique entre livraison déjà terminale, réassignation unique
  ou intervention humaine. Une réassignation écrit aussi, dans cette même
  transaction locale, les intentions immuables d'annuler la demande suivie
  source, de créer la demande suivie du successeur et de notifier les deux
  participants. Deux processus concurrents ne peuvent créer deux successeurs.
- **FR-1608a — Arbitrage de relève** : dans un même lot relevé, Maicie DOIT
  appliquer les `delivery_report` corrélés avant les `reminder_sent` de la même
  génération, quel que soit leur ordre filaire. Les effets déjà committés dans
  des lots antérieurs ne sont pas réordonnés.
- **FR-1609 — Candidats préautorisés** : une chaîne de repli ne contient que
  des participants existants du même objectif au snapshot épinglé. Une
  réassignation ne peut appeler aucun chemin de profil, d'approbation ou de
  spawn, ni substituer une observation de disponibilité Bridget aux faits du
  registre.
- **FR-1610 — Deux vérités** : les décisions citent séparément le fait local
  Maicie et l'événement Bridget avec sa fraîcheur. Une observation incomplète
  bloque l'effet automatique sans être transformée en état métier.
- **FR-1611 — Réducteur pur (FR-022)** : pour un même snapshot de registre, un
  même événement et une même version de politique, le résultat calculé DOIT
  être identique, sans accès réseau, modèle ou horloge pendant la décision.
- **FR-1612 — Approbation intacte (FR-014)** : aucune surface de la session ne
  crée, approuve, refuse ou consomme une approbation. L'intervention humaine
  requise est un terminal visible, pas une approbation implicite.
- **FR-1613 — Reprise après crash** : les frontières avant transaction, après
  transaction avant envoi, après envoi avant accusé et après accusé avant
  consommation locale DOIVENT converger sans perte ni double effet.
- **FR-1614 — Observabilité** : chaque décision et notification porte
  `objective_id`, `delegation_id`, génération, `event_id`, version de politique,
  motif et identifiant d'envoi, sans corps libre requis pour l'audit.
- **FR-1615 — Compatibilité** : sans dépendance, destinataire attendu, chaîne de
  repli ou politique active, les comportements 011 et 015 restent inchangés.

## Entités et états attendus

- **ÉvénementCoordination** : fait immuable corrélé à sa source, avec type,
  identifiant, instant attesté et fraîcheur éventuelle.
- **DépendanceDélégation** : arête orientée `prérequis → dépendant` au sein d'un
  objectif, créée avant l'ouverture du dépendant.
- **AttenteNotification** : destinataire déclaré pour un type d'événement et
  clé d'idempotence déterministe.
- **NotificationOutbox** : enveloppe immuable, état `prepared`,
  `outcome_unknown`, `accepted` ou `rejected`, rejouée sans reconstruction.
- **PolitiqueRéassignation** : version, classe de délai, seuil et chaîne de
  repli épinglés à une délégation, avec les faits d'appartenance au registre
  qui autorisent chaque candidat.
- **LignéeDélégation** : générations successives reliées par un motif de
  réassignation ; une seule génération est active.
- **ÉpisodeRelance** : demande suivie, génération, compteur consécutif et
  dernier `answered` corrélé ; une réponse remet le compteur à zéro, une
  annulation administrative le rend inactif.

## Critères mesurables

- **SC-1601** : sur 100 clôtures ayant chacune trois destinataires et quatre
  frontières de crash, 300 notifications distinctes sont acceptées exactement
  une fois et 0 notification dépend du relais manuel d'un référent.
- **SC-1602** : un corpus de graphes comprenant chaîne, losange, 100 nœuds,
  300 arêtes, doublon, cycle et arête inter-objectifs produit 100 % des
  ouvertures attendues, 0 ouverture prématurée et les refus typés prévus.
- **SC-1603** : pour chaque classe de délai, `N-1` relances produisent 0
  réassignation et la `N`e produit exactement 1 successeur sur 100 répétitions,
  y compris avec deux processus concurrents et redémarrage ; chaque successeur
  possède une nouvelle demande suivie et l'ancienne demande une annulation
  durable, avec une notification unique au sortant et au successeur.
- **SC-1604** : les deux ordres de la course livraison/relance et les quatre
  frontières de crash convergent vers le même terminal, sans double génération
  ni perte de livraison tardive. Un corpus par lots inverse l'ordre filaire
  `delivery_report`/`reminder_sent` et conserve la priorité à la livraison ; un
  `answered` intercalé remet le compteur à zéro et une annulation
  administrative interdit tout successeur.
- **SC-1605** : 100 % des tentatives de réassignation vers un participant non
  déclaré dans le snapshot épinglé, avec événement source non frais ou après
  épuisement de chaîne aboutissent à `intervention_humaine_requise`, avec 0
  spawn et 0 approbation.
- **SC-1606** : une mutation retirant le fait durable, remplaçant l'événement
  attesté par du texte ou autorisant une observation `Gap` fait échouer un
  oracle dédié ; le corpus nominal reste déterministe sur 100 replays.
- **SC-1607** : le gate rejoue les trois incidents fondateurs — livraison
  muette, dépendant laissé en attente, deux relances sans livraison — et les
  résout sans message de réveil, déblocage ou réassignation manuels.
- **SC-1608** : sur 20 campagnes de 1 000 événements et 300 arêtes dans un
  environnement gelé, 20/20 relèves terminent en moins d'une seconde et le
  nombre d'évaluations reste proportionnel aux événements et arêtes concernés,
  sans scan d'un autre objectif.

## Hypothèses et dépendances

- Le guichet 015 et ses événements corrélés sont gelés avant le branchement de
  la session 016. Le lot A ne peut commencer qu'après merge et gel explicite
  du contrat 015. Si l'événement de relance attestée manque au contrat final,
  son ajout versionné est une extension 016 négociée, pas une déduction locale.
- Les participants de repli existent déjà dans l'objectif et ont été admis par
  les voies d'approbation antérieures. La session ne gère pas leur naissance.
- Le terminal qualifiant v1 par défaut est la greffe durable du hash de
  livraison. Une arête qui exige la règle stricte de vérification avant relais
  porte explicitement `clôture_évaluée_exigée` ; une simple acceptation
  transport ne suffit jamais dans les deux modes.
- Les politiques s'appliquent à un objectif à la fois. Les dépendances entre
  objectifs, routines récurrentes et planification calendaire restent hors
  périmètre.

## Hors périmètre explicite

- Interprétation sémantique d'un rapport, estimation de qualité ou résumé.
- Création automatique d'agent, approbation, choix de modèle ou de profil.
- Scheduler, polling, daemon Maicie permanent ou nouvelle horloge locale.
- Dépendances entre objectifs, priorisation globale de portefeuille et pause
  d'échéance.
- Réassignation vers un participant non déclaré ou choix probabiliste du
  « meilleur » agent.
- Diffusion générique de groupe au niveau Bridget ; la liste des destinataires
  reste une sémantique Maicie.

## Traçabilité des politiques et incidents

| Référence | Incident réel du 2026-08-23 | Exigences | Preuve attendue |
|---|---|---|---|
| F27 / politique 1 | commits, verdicts et fins de banc muets ; le référent devait annoncer chaque hash | FR-1601, FR-1602, FR-1613 | SC-1601, SC-1607 |
| F28 / politique 2 | prospective est restée en attente d'interfaces et de gates déjà livrés jusqu'au réveil manuel | FR-1603 à FR-1605 | SC-1602, SC-1607 |
| F29 / politique 3 | creux d'agents constatés par l'utilisateur ; deux relances factuelles sans commit avant réassignation manuelle | FR-1606 à FR-1609, FR-1608a | SC-1603 à SC-1605, SC-1607 |
| Relais manuels et corrélation perdue | des réponses non liées ont provoqué des rappels et doubles réponses ; les crashs imposaient de retrouver le terminal réel | FR-1613, FR-1614 | SC-1601, SC-1604 |
| Politiques manuelles purement factuelles | aucune des trois politiques exécutées ce jour-là ne nécessitait de lire le contenu livré | FR-1611 | SC-1606 |
| Tentatives répétées de préserver la porte humaine | une réassignation opérationnelle ne valait ni approbation ni naissance d'agent | FR-1609, FR-1612 | SC-1605 |
| Deux vérités 011/015 | les états Maicie et les observations Bridget ont des autorités distinctes | FR-1610 | SC-1604, SC-1606 |
| Non-régression après automatisations 007–015 | les relances erronées et doubles réponses sont précisément réapparues quand un nouveau chemin contournait le contrat existant | FR-1615 | corpus 011/015 intégral au vert |
