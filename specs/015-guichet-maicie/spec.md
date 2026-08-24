# Spec 015 — Guichet Maicie : identité joignable et demandes corrélées

**Branche** : `session-15-guichet-maicie` | **Créée** : 2026-08-23
**Statut** : Livrée — T1501 à T1514 closes, revue hostile finale MERGEABLE
**Origine** : D18 du catalogue de coordination v2. Les deux besoins sont
prouvés : un agent a tenté d'écrire à Maicie alors qu'elle était absente, et
deux clôtures liées ont été oubliées au greffe. La conclusion T015b a aussi
établi que Subscribe seul ne porte ni `in_reply_to` de la réponse ni timeout
de demande.

## Contexte et décision de périmètre

Maicie v3 est un compagnon *pull-only* : elle n'est lancée que par une commande
locale et ne doit pas devenir discrètement un daemon résident. Cela rend sa
SQLite privée saine et lisible, mais rend aujourd'hui le destinataire `maicie`
introuvable entre deux commandes. Un simple enregistrement éphémère à chaque
commande ne corrige pas ce défaut : l'agent qui veut écrire pendant l'absence
de Maicie reçoit encore un refus de destinataire.

La présente session introduit donc le **guichet Maicie** : une identité de
service durable, tenue par Bridget, qui accepte une petite matrice de demandes
structurées et les conserve jusqu'à une relève bornée par Maicie. Bridget est
le transport durable du guichet ; la SQLite de Maicie reste l'unique vérité sur
les objectifs, délégations, décisions et clôtures. Aucun état métier Maicie
n'est recopié dans Bridget.

Le précédent C5 de l'ADR 003 s'applique directement : une identité annoncée
dans un `RoleHello` reste déclarative dans le modèle local coopératif v1. Le
guichet ne doit donc jamais traiter `from: "maicie"` comme une autorisation.
Sa surface de relève, de réponse et d'événements est protégée par une capacité
distincte négociée avec Bridget. Cette borne empêche une usurpation par simple
charge JSON ou rôle historique, sans prétendre qu'elle authentifie un processus
hostile du même compte local ; cette non-opposabilité résiduelle est assumée et
visible jusqu'à une évolution d'identité v2.

Le guichet permet une boucle de réponses déterministe sans faire de Maicie un
superviseur ni un interprète. Une boucle résidente optionnelle reste une
évolution v2 distincte : elle pourra réduire la latence de réponse, mais ne
change ni le contrat du guichet ni la source de vérité.

## Décision d'architecture

### D-1501 — Identité de service durable, relève pull-only

Bridget publie une identité réservée `maicie` comme **boîte aux lettres de
service durable**, indépendante de toute connexion Maicie. Toute commande
Maicie commence par une relève bornée des demandes non terminales, les traite
dans l'ordre durable du guichet, puis renvoie les réponses liées lorsque leur
destinataire est joignable. L'absence de processus Maicie n'est donc jamais
présentée comme une absence d'identité.

L'enregistrement éphémère seul est rejeté : il ne rendrait Maicie joignable
que durant la courte vie d'une commande et reconduirait exactement l'incident
« destinataire introuvable ». Une mini-boucle `maicie serve` résidente n'est
pas mise en oeuvre ici ; elle est explicitement différée à une session v2, avec
arrêt propre, présence visible et même contrat de guichet.

### D-1502 — Requêtes fermées et réponses déterministes

Le corps reçu par le guichet est une enveloppe JSON versionnée, de schéma
fermé, bornée et canonique. Les seules opérations admises dans ce premier
contrat sont :

| Opération | Entrées obligatoires | Réponse calculée depuis le registre Maicie |
|---|---|---|
| `delivery_report` | `objective_id`, `delegation_id`, `delivery_hash`, `in_reply_to` de la délégation suivie | reçu structuré et effet de greffe explicité |
| `mission_status` | `delegation_id` | état de coordination, remise locale, observation transport et fraîcheur |
| `deadline_question` | `delegation_id` | classe de durée et échéance contractuelle, sans qualifier un retard |

Les réponses portent le `request_id` du guichet en `in_reply_to`, sont des
faits ou transitions fermées et n'emploient aucun LLM. Aucun texte libre,
message de type inconnu, champ inconnu, référence absente, rôle non autorisé
ou enveloppe trop grande ne devient une intention implicite : le guichet
retourne un refus structuré, corrélé et sans mutation.

Le guichet reçoit aussi des **événements de cycle de vie produits par Bridget**
— `answered`, `cancelled` et `timed_out` — pour les demandes que Maicie a
émises. Ils ne sont pas des opérations ouvertes aux agents : ils portent le
`request_id` d'origine, un `event_id` durable et l'horodatage attesté. Ils
complètent la réponse liée et rendent enfin le timeout observable sans le
déduire d'ACP ou de l'horloge Maicie.

### D-1502a — Deux validations, une seule responsabilité chacune

Bridget rejette avant persistance toute trame qui n'a pas négocié la capacité
`maicie_guichet`, qui n'utilise pas le rôle de service prévu, qui vise une
identité réservée invalide, dont la version, taille, canon, identifiant ou
transition de guichet est invalide. Les primitives de relève (`claim`), de
réponse (`GuichetReply`) et de dépôt d'`ÉvénementDemande` sont exclusivement
autorisées par cette capacité : elles ne sont jamais déverrouillées par un nom
déclaré dans une charge.

Maicie effectue ensuite la validation **relationnelle et métier** : émetteur
participant, lien objectif/délégation, hash attendu, état de l'objectif et
opération admise. Elle ne refait ni la validation filaire ni l'autorisation de
capacité Bridget. Cette séparation est normative : un même refus ne peut pas
être implémenté deux fois avec des sens différents.

### D-1503 — Greffe automatique de la réponse liée, jamais de clôture métier implicite

Un `delivery_report` lié porte l'`in_reply_to` de la demande Maicie d'origine.
Bridget ne marque cette demande `answered` qu'après la persistance et
l'accusé durable de la réponse dans le guichet ; il ne reste alors ni rappel ni
timeout ouvert pour cette demande. C'est la **clôture automatique au greffe**
attendue par le besoin fondateur.

Lorsque Maicie relève ensuite le rapport, elle vérifie le participant, la
délégation, l'objectif et le hash, puis écrit dans **la même transaction** le
fait reçu, le reçu idempotent, la décision auditée et, le cas échéant, la
transition vers `à_évaluer`. Un `delivery_report` ne clôt jamais l'objectif
par simple affirmation de l'agent : `à_évaluer → clos` demeure une action
Maicie ou humaine explicite. Un retry du même `request_id` rejoue exactement
le même résultat ; une divergence d'enveloppe est refusée sans mutation.

Si la demande d'origine est déjà `timed_out` ou `cancelled`, un
`delivery_report` tardif et autrement valide est **greffé sans essayer de la
clore à nouveau**. Maicie enregistre le fait et une décision `à_évaluer` avec
l'état terminal déjà constaté ; la réponse indique `request_already_terminal`.
Ainsi une livraison réelle n'est pas perdue à cause d'une course avec le
timeout, sans rouvrir ni modifier l'état terminal Bridget.

La réponse relevée et l'événement Bridget `answered` décrivent parfois le même
fait. Ils portent alors le même couple `(in_reply_to, response_message_id)` et
convergent vers un unique reçu de corrélation Maicie. L'événement arrivé avant
le rapport est un fait de transport en attente ; le rapport arrivé avant lui
porte l'effet métier. Dans les deux ordres, il n'existe qu'une décision et une
transition Maicie.

Les autres opérations sont strictement consultatives. Une remise `accepted`,
un fait ACP ou l'absence d'activité ne sont jamais assimilés à une livraison
de travail ni à une clôture.

### D-1504 — Deux vérités, deux responsabilités

Le guichet Bridget conserve seulement la demande structurée, son identité de
transport, sa remise et sa réponse liée. Maicie conserve la décision métier,
le reçu de greffe et les références aux objectifs. La réponse `mission_status`
expose séparément :

1. la vérité locale durable Maicie (coordination et remise connue) ;
2. l'observation Bridget datée, avec `fresh`, `gap`, `ended` ou `unknown`.

Une indisponibilité du transport ne rouvre, ne clôt et ne transforme jamais un
objectif. Une erreur après relève laisse la demande durable au guichet ou
produit une issue explicite selon le contrat ; elle ne déclenche pas une
seconde demande ou une seconde décision.

## Scénarios utilisateurs et tests

### US1 — Déposer une livraison pendant l'absence de Maicie (P1)

Un équipier peut signaler la livraison hashée d'une délégation en s'adressant à
`maicie`, même lorsque le binaire Maicie n'est pas lancé. À la prochaine
commande Maicie, le guichet est relevé, la référence est vérifiée et le
registre enregistre une seule fois le fait et la décision liée. La demande
Bridget d'origine est déjà clôturée au greffe après l'accusé durable du rapport.

**Pourquoi** : cxbridget a déjà tenté d'écrire à Maicie et a rencontré un
destinataire introuvable ; deux oublis de clôture ont démontré qu'une réponse
liée doit aboutir au greffe, pas seulement apparaître dans un fil de messages.

**Test indépendant** : arrêter Maicie, déposer un `delivery_report` structuré
sur une délégation existante, lancer `maicie status`, puis constater une seule
décision corrélée et la transition prévue. Rejouer exactement la demande après
un crash de Maicie entre la relève et le commit ; constater le même résultat,
sans seconde transition ni rouvrir la demande Bridget déjà répondue.

**Scénarios d'acceptation** :

1. **Étant donné** Maicie absente et l'identité réservée active, **quand** un
   équipier dépose une enveloppe `delivery_report` valide, **alors** Bridget
   l'accepte durablement sans prétendre que Maicie l'a déjà traitée.
2. **Étant donné** une demande déposée, **quand** Maicie est invoquée avec son
   budget de relève, **alors** elle traite la demande une fois, écrit le fait,
   la décision et la transition dans une transaction unique, puis envoie une
   réponse corrélée déterministe.
3. **Étant donné** le même `request_id` et les mêmes octets, **quand** la
   commande redémarre après un accusé perdu, **alors** le résultat est rejoué
   sans nouvelle décision, sans seconde transition et sans rouvrir la demande
   Bridget déjà répondue.
4. **Étant donné** le même `request_id` et une enveloppe divergente, **quand**
   elle est soumise, **alors** le guichet la refuse sans modifier ni Bridget ni
   le registre Maicie.
5. **Étant donné** un timeout déjà publié pour la demande d'origine, **quand**
   un `delivery_report` tardif mais valide arrive, **alors** son hash est porté
   au greffe avec le motif terminal existant, sans rouvrir la demande ni perdre
   la livraison.
6. **Étant donné** le même rapport et l'événement `answered` corrélé, **quand**
   ils sont relevés dans l'un ou l'autre ordre, **alors** une seule décision
   Maicie est visible.

### US2 — Demander l'état de sa mission sans interprétation (P1)

Un équipier interroge sa délégation et reçoit les faits Maicie disponibles,
dont la remise locale durable et l'observation transport explicitement
fraîche ou inconnue. Il ne reçoit jamais un diagnostic inventé de blocage, de
retard ou de besoin de décision.

**Pourquoi** : la séparation des deux vérités de Maicie a été imposée par les
faits partiels Subscribe/ACP ; un statut commode mais deviné recréerait le
carcan que Maicie doit éviter.

**Test indépendant** : injecter successivement une observation fraîche, un
`Gap` et l'absence de source ; demander `mission_status` et vérifier les trois
sorties structurées distinctes, sans transition d'objectif.

### US3 — Poser une question d'échéance factuelle (P2)

Un équipier demande l'échéance de sa délégation. Il reçoit la classe de durée
configurée et l'échéance contractuelle immuable ; le guichet ne calcule ni
n'affirme un dépassement local.

**Pourquoi** : T019 a démontré que les classes de durée sont des contrats
passifs et que Bridget est la seule horloge des demandes suivies.

**Test indépendant** : demander l'échéance avant et après l'instant affiché,
puis constater des faits identiques tant qu'aucune issue Bridget n'a été
publiée.

### US4 — Refuser honnêtement une demande hors contrat (P1)

Un agent ne peut ni transmettre une approbation, ni demander une mutation
libre, ni faire interpréter un texte conversationnel par Maicie. Il reçoit un
refus structuré qui cite l'opération autorisée et n'écrit aucune ligne métier.

**Pourquoi** : FR-014 réserve l'activation à une frappe humaine locale ;
FR-022 interdit le LLM et l'interprétation de texte libre. Le défaut inverse
réintroduirait une porte d'approbation distante et des décisions opaques.

**Test indépendant** : soumettre un texte libre, une opération `approve`, une
enveloppe à champ inconnu et une tentative de réutilisation divergente ;
vérifier quatre refus typés, zéro création d'objectif, délégation, approbation
ou `SpawnOrder`.

## Exigences fonctionnelles

- **FR-1501 — Identité joignable** : Bridget DOIT rendre le destinataire de
  service `maicie` adressable indépendamment de l'exécution du binaire Maicie,
  sans l'afficher comme un agent vivant ni comme un processus supervisé.
- **FR-1502 — Modèle compagnon conservé** : Maicie DOIT relever le guichet
  uniquement à l'ouverture d'une commande locale et dans un budget absolu
  configuré. Cette session NE DOIT introduire ni polling, ni scheduler, ni
  daemon Maicie lancé en arrière-plan.
- **FR-1503 — Contrat public versionné** : le dépôt et la relève des demandes
  DOIVENT emprunter un protocole Bridget public, versionné, négocié et borné.
  Maicie ne lit ni `bridget.db`, ni socket interne, ni crate non public.
- **FR-1503a — Autorisation de capacité** : `maicie_guichet` DOIT être une
  capacité explicitement négociée, distincte des rôles historiques et du nom
  déclaré. Seules les connexions qui l'ont négociée peuvent relever une
  demande, émettre `GuichetReply` ou recevoir/déposer un
  `ÉvénementDemande`. Cette borne n'est pas une authentification
  cryptographique : la possibilité d'un processus local du même compte de
  négocier la capacité reste documentée comme limite coopérative v1.
- **FR-1504 — Matrice fermée** : seules `delivery_report`, `mission_status` et
  `deadline_question` sont acceptées. Chaque enveloppe porte version,
  `request_id`, émetteur déclaré, cible `maicie`, type et charge utile fermée.
  Un `delivery_report` porte en plus l'`in_reply_to` de la délégation suivie.
  Les tailles, références et caractères sont validés avant toute mutation.
- **FR-1505 — Rejeu sûr** : une demande porte une clé d'idempotence durable.
  Après crash ou accusé perdu, le même `request_id` et les mêmes octets
  rejouent l'issue réelle ; des octets différents sont refusés. Une expiration
  de tombstone est terminale et visible ; Maicie n'invente jamais de nouvel id.
- **FR-1506 — Livraison liée et greffe** : une réponse `delivery_report`
  corrélée DOIT faire passer la demande Bridget d'origine à `answered` après
  accusé durable du guichet, jamais avant. Son traitement Maicie ne produit
  une décision que si son émetteur, son objectif, sa délégation et son hash
  correspondent au registre. Fait, décision, transition vers `à_évaluer` et
  marqueur d'idempotence DOIVENT être atomiques ; aucun retry ne peut les
  doubler ni fermer directement l'objectif.
- **FR-1506a — Course terminale honnête** : un `delivery_report` dont
  l'`in_reply_to` désigne déjà une demande `timed_out` ou `cancelled` DOIT être
  greffé comme livraison tardive, avec le terminal existant, sans tentative de
  clôture, de réouverture ou de perte de hash.
- **FR-1506b — Déduplication inter-canaux** : pour une réponse liée,
  `delivery_report` et `ÉvénementDemande(answered)` DOIVENT partager une clé de
  corrélation `(in_reply_to, response_message_id)` et converger vers un seul
  reçu, une seule décision et une seule transition Maicie quel que soit leur
  ordre d'arrivée.
- **FR-1507 — Statut déterministe** : `mission_status` et
  `deadline_question` sont calculés exclusivement depuis le registre Maicie et
  les observations Bridget attestées. Ils distinguent toujours absence,
  indisponibilité, flux incomplet et fait frais, sans déduction métier.
- **FR-1508 — Greffe liée traçable** : toute réponse qui clôt une demande
  Bridget porte `objective_id`, `delegation_id`, `request_id`, `in_reply_to`,
  hash et décision de coordination dans le journal d'audit. Une livraison non
  liée ne clôture ni demande suivie ni objectif.
- **FR-1509 — Approbation hors guichet** : aucune opération de guichet, réponse
  liée, CLI non TTY, outil MCP ou texte libre ne peut créer, approuver,
  consommer, refuser ou contourner une approbation d'activation. Seule la
  frappe locale déjà définie par FR-014 conserve cette autorité.
- **FR-1510 — Zéro interprétation** : un message libre ou JSON non conforme est
  rejeté honnêtement avec un code stable. Le guichet n'appelle aucun modèle,
  n'infère pas une intention, ne résume pas et ne sélectionne pas d'agent.
- **FR-1511 — Réponses liées** : une réponse au guichet DOIT utiliser
  `in_reply_to=request_id`. Si l'émetteur est momentanément indisponible, le
  contrat conserve une issue de réponse explicite ; le registre Maicie ne perd
  ni ne répète la décision de greffe.
- **FR-1512 — Observabilité minimale** : chaque dépôt, relève, refus,
  traitement, réponse et transition est journalisé par `request_id`,
  `objective_id` et `delegation_id` quand ils existent, sans corps libre ni
  secret. Les compteurs de demandes en attente et de refus sont mesurables.
- **FR-1513 — Compatibilité** : les messages Bridget historiques, les demandes
  suivies existantes, l'attachement ACP et les commandes Maicie v3 restent
  inchangés hors activation explicite de la capacité de guichet.
- **FR-1514 — Événements de demande corrélés** : Bridget DOIT déposer au
  guichet, exactement une fois et avec un `event_id` durable, les états
  terminaux `answered`, `cancelled` et `timed_out` des demandes émises par
  Maicie. La transition terminale et le dépôt de l'événement DOIVENT être
  atomiques ; Maicie ne reconstitue jamais un timeout depuis une échéance ou
  une absence de message.
- **FR-1515 — Limite C5 explicite** : le statut de sortie et le journal de
  guichet DOIVENT qualifier l'identité v1 comme déclarative et coopérative. Ils
  ne DOIVENT jamais présenter la capacité `maicie_guichet` comme une preuve
  d'identité opposable entre processus du même compte.

## Entités et états attendus

| Entité | Propriétaire | Rôle | États ou invariant |
|---|---|---|---|
| `GuichetRequest` | Bridget | enveloppe entrante durable adressée à `maicie` | `queued → claimed → replied` ou `rejected` ; le claim n'est jamais une décision métier |
| `GuichetReply` | Bridget | réponse corrélée à `request_id` | exactement une issue durable ou une indisponibilité explicite |
| `ÉvénementDemande` | Bridget | fait terminal d'une demande émise par Maicie | `event_id` unique, écrit avec l'état `answered` / `cancelled` / `timed_out` |
| `ReceptionGreffe` | Maicie | preuve locale d'une requête traitée | clé unique `(request_id, operation)` ; ne conserve pas de corps libre |
| `ReçuCorrélation` | Maicie | jointure d'un rapport et de son événement `answered` | clé unique `(in_reply_to, response_message_id)` ; une seule décision quelle que soit la source première |
| `DemandeCorrélée` | Bridget | demande suivie émise par Maicie | devient `answered` après accusé durable de la réponse liée, jamais par inférence Maicie |
| `DecisionCoordination` | Maicie | décision auditée causée par un rapport valide | écrite avec l'effet objectif dans la même transaction ; ne clôt pas implicitement |
| `TransportSnapshot` | Bridget/lecture Maicie | observation externe datée | jamais autorité d'une clôture d'objectif ou d'une approbation |

## Critères mesurables

- **SC-1501** : dans 100 dépôts de `delivery_report` pendant l'absence de
  Maicie, 100 sont soit durablement en attente soit explicitement refusés ;
  aucun n'échoue par « destinataire Maicie introuvable ».
- **SC-1502** : une relève après crash aux trois frontières (avant lecture,
  après claim/avant commit Maicie, après commit/avant réponse) produit une
  unique décision, une unique transition vers `à_évaluer` au plus et une seule
  demande Bridget terminale `answered` pour le même `request_id`.
- **SC-1503** : le corpus de contrat couvre les trois opérations admises et au
  moins six refus (texte libre, type inconnu, champ inconnu, référence absente,
  approbation interdite, enveloppe divergente) ; chaque refus laisse le
  registre métier inchangé.
- **SC-1504** : sur 100 délégations et une relève à froid, p95 de la commande
  Maicie qui effectue sa relève reste sous le budget configuré ; à son
  épuisement, elle rend un statut explicite sans bloquer ni prolonger la
  commande.
- **SC-1505** : une réponse `mission_status` distingue dans 100 % du corpus la
  remise locale, l'observation transport et sa fraîcheur ; aucun oracle de
  test n'accepte les mots « bloqué », « terminé » ou « en retard » sans fait
  source correspondant.
- **SC-1506** : 100 % des tentatives d'approbation via le guichet sont refusées
  et ne créent ni approbation, ni `SpawnOrder`, ni processus.
- **SC-1507** : sur les trois états terminaux d'une demande Maicie, un
  redémarrage Bridget puis Maicie livre exactement un événement corrélé au
  guichet ; retirer ce dépôt atomique du harnais fait échouer l'oracle.
- **SC-1508** : le corpus exécute `answered → delivery_report` et
  `delivery_report → answered`, ainsi qu'un rapport arrivé après `timed_out` ;
  il obtient respectivement une seule décision et un greffe tardif visible,
  jamais un refus de livraison ni une demande rouverte.
- **SC-1509** : les primitives de relève, réponse et événement sont refusées
  dans 100 % des cas sans capacité `maicie_guichet`, même si la trame déclare
  le nom `maicie` ; le journal qualifie explicitement la limite coopérative v1.

## Hors périmètre explicite

- boucle résidente `maicie serve`, polling autonome, scheduler et cron ;
- LLM, analyse de texte libre, résumé génératif, sélection sémantique ou
  autorisation implicite ;
- approbation, refus ou activation de profil via Bridget, MCP ou guichet ;
- GUI/TUI, panneau de boîte de réception et automatisations F27–F29 ;
- accès à une base Bridget interne, partage de SQLite, supervision de processus
  ou modification du cycle de vie ACP ;
- multi-utilisateur, réseau distant et authentification inter-tenant.

## Traçabilité des besoins

| Référence | Besoin ou incident fondateur | Exigences concernées |
|---|---|---|
| D18 / Conclusion T015b | Maicie n'est pas joignable ; Subscribe ne corrèle pas réponse ni timeout | FR-1501 à FR-1505, FR-1511 |
| Incident cxbridget « destinataire introuvable » | un agent a tenté d'écrire à Maicie absente | FR-1501 à FR-1503, SC-1501 |
| Deux oublis de clôture au greffe | une livraison liée n'a pas produit de décision durable | FR-1506, FR-1508, SC-1502 |
| FR-014 de 011 | l'approbation est une frappe humaine locale seulement | FR-1509, SC-1506 |
| FR-022 et messages libres 011 | aucune interprétation de texte ni LLM | FR-1504, FR-1510, SC-1503 |
| T019 | échéance affichée passivement, Bridget seule horloge | FR-1507, SC-1505 |
| T018 / deux vérités | snapshot observé n'est pas une décision de coordination | D-1504, FR-1507, SC-1505 |
| Conclusion T015b | réponse liée et timeout ne sont pas observables par Subscribe seul | D-1502, FR-1514, SC-1507 |
| C5 / ADR 003 | un rôle déclaré ne rend pas l'approbation opposable côté Bridget | Contexte, D-1502a, FR-1503a, FR-1515, SC-1509 |
