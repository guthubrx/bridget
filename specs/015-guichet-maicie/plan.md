# Plan 015 — Guichet Maicie : identité joignable et demandes corrélées

**Branche** : `session-15-guichet-maicie` | **Date** : 2026-08-23  
**Spec** : `specs/015-guichet-maicie/spec.md`  
**Statut** : plan de conception ; l'implémentation, les tâches détaillées et
les artefacts de contrat ne sont pas encore autorisés.

## Résumé

La session donne à Maicie une identité de service durable `maicie`, visible et
adressable même lorsque le compagnon n'est pas démarré. La décision est un
guichet durable détenu par Bridget et relevé par Maicie à l'ouverture bornée de
ses commandes. Ce choix résout le destinataire introuvable sans faire de
Maicie un daemon permanent ni déplacer l'état métier hors de son registre.

Le guichet n'accepte que trois demandes JSON fermées : rapport de livraison
hashée, statut de mission et question d'échéance. Les réponses sont
déterministes. Une réponse liée clôt automatiquement **la demande Bridget** au
greffe après son accusé durable ; son traitement peut déclencher une décision
et une transition atomiques vers `à_évaluer` dans Maicie, jamais la clôture
automatique de l'objectif.

## Recherche et arbitrages déjà clos

| Décision | Choix retenu | Alternatives écartées | Pourquoi |
|---|---|---|---|
| Joignabilité | boîte aux lettres de service durable Bridget + relève pull-only | enregistrement Maicie éphémère ; daemon Maicie implicite | l'éphémère échoue précisément quand l'agent écrit hors d'une commande ; le daemon serait une architecture nouvelle non justifiée par la performance |
| Traitement | relève à chaque commande, sous échéance globale | polling/ticks ; worker caché | conserve le modèle compagnon et rend l'attente explicite et bornée |
| Langage des agents | trois requêtes JSON fermées | conversation libre ; MCP sémantique | FR-022 interdit l'interprétation ; le résultat doit être testable et auditable |
| Effet métier | greffe Bridget corrélée + réception Maicie atomique sur `delivery_report` | ajout d'une copie d'état dans Bridget ; clôture d'objectif par livraison `accepted` | Bridget est autorité de demande ; la SQLite Maicie reste vérité de coordination et T015a a démontré que l'acceptation de transport n'est pas une réponse |
| Boucle temps réel | différée v2, même contrat | l'introduire à bas bruit dans `status` | SC-008 a établi que la capture éphémère est performante ; la seule justification restante est la réponse corrélée, qui mérite une session dédiée |

## Architecture cible

```text
équipier Bridget
  │ RequestEnvelope JSON, request_id idempotent
  ▼
Bridget : identité de service « maicie »
  ├── guichet durable (transport, reçu, réponse liée)
  └── aucun objectif ni décision Maicie
  │ relève bornée au début d'une commande locale
  ▼
Maicie compagnon
  ├── valide le schéma fermé et les droits de référence
  ├── SQLite privée : fait de greffe + décision + objectif, transaction unique
  └── envoie GuichetReply(in_reply_to=request_id)
  ▼
équipier Bridget, ou issue explicite si devenu indisponible
```

### Frontières de propriété

| Élément | Autorité | Interdit |
|---|---|---|
| présence, transport, message, demande corrélée, délai et boîte aux lettres | Bridget | écrire dans la SQLite Maicie ou décider un objectif |
| objectif, délégation, décision, reçu de greffe, transition `à_évaluer` | SQLite privée Maicie | lire/écrire `bridget.db`, reconstruire une observation transport, fermer automatiquement l'objectif |
| GUI/TUI/MCP/CLI | projections | posséder un état métier parallèle ou contourner le guichet |
| approbation de profil | TTY humain local Maicie | toute route Bridget, MCP, guichet ou flag non interactif |

### Séquence nominale `delivery_report`

1. L'agent construit l'enveloppe canonique avec son `request_id`, les IDs de
   l'objectif et de la délégation, et le hash de livraison ; il la dépose sur
   l'identité `maicie`.
2. Bridget la déduplique et la rend durable au guichet. Il ne prétend pas que
   Maicie l'a comprise ni traitée.
3. La prochaine commande Maicie prend une relève bornée. Chaque demande est
   validée sans texte libre et reliée à un participant, une délégation et un
   objectif existants.
4. La persistance/ack du rapport clôt la demande Bridget d'origine par son
   `in_reply_to`, sans attendre que Maicie soit lancée.
5. Une transaction Maicie écrit le reçu local, la décision auditée et,
   seulement si les préconditions sont vraies, la transition vers
   `à_évaluer`. En cas de crash, le même `request_id` retrouve le même reçu et
   ne duplique jamais l'effet ni ne clôt l'objectif implicitement.

### États, crashs et vérité

Le guichet a une machine de transport distincte : `queued`, `claimed`,
`replied`, `rejected`. Le registre Maicie n'en déduit aucun état métier. Un
claim non finalisé est relivable ; une transaction Maicie déjà committée est
reconnue par la clé de reçu. La réponse perdue est reconstituée depuis le
résultat durable, sans réécrire la décision.

Le contrat doit conserver les mêmes garanties idempotentes que 012 : identité
émettrice stable, `request_id` avant I/O, octets canoniques, refus de divergence
et horizon de rétention déclaré. Au-delà de cet horizon, l'issue est
`idempotency_expired`, jamais une réémission aveugle.

## Contrat fonctionnel à produire lors de l'implémentation

La session devra produire un contrat public distinct avant le code. Il devra
préciser au minimum :

- négociation de capacité `maicie_guichet` ; la capacité absente laisse les
  comportements 007–014 inchangés ;
- `ServiceRequest` et `ServiceReply`, avec versions, canon, tailles maximales,
  idempotence, rétention, refus et table de récupération après crash ;
- enveloppes strictes `delivery_report`, `mission_status`,
  `deadline_question` ; aucun champ extensible non négocié au coeur des
  opérations ;
- `RequestLifecycleEvent` réservé à Bridget pour `answered`, `cancelled` et
  `timed_out`, déposé atomiquement avec la transition de la demande ;
- règles d'autorisation relationnelle : l'émetteur doit être le participant de
  la délégation ou la demande est refusée sans divulguer l'objectif ;
- corrélation des réponses et statut terminal d'un émetteur non joignable ;
- représentation séparée de la remise locale Maicie et du snapshot Bridget,
  avec fraîcheur et cause d'inconnu.

## Constitution check

| Gate | État | Justification |
|---|---|---|
| Minimalisme (XIX) | PASS | une boîte aux lettres durable répond au besoin prouvé ; aucune boucle permanente, GUI ou moteur sémantique |
| Responsabilités futures (XX) | PASS | transport Bridget et décisions Maicie sont explicitement séparés ; le runtime v2 est différé |
| État durable | PASS sous gate | chaque frontière de crash nécessite une table de récupération commune aux deux propriétaires, sans transaction distribuée fictive |
| Sécurité / FR-014 | PASS sous gate | aucune approbation dans la matrice ; TTY humain reste la seule route |
| Observabilité | PASS sous gate | chaque effet porte request/objective/delegation IDs, sans contenu libre |
| Complexité (XVIII) | PASS sous gate | relève bornée et indexée ; pas de scan illimité à chaque commande |
| Compatibilité | PASS sous gate | capacité négociée ; aucune mutation des rôles/messages existants sans opt-in |

## Découpage d'implémentation proposé — trois couloirs disjoints

Les tâches détaillées seront créées seulement après validation hostile de ce
plan. Les responsabilités suivantes évitent les collisions de fichiers :

### Lot A — Contrat et guichet durable Bridget (codeur A)

**Propriété** : protocole public, rôle/service Bridget, persistance et reprise
de guichet, tests socket/daemon.  
**Dépendances** : aucune modification Maicie.  
**Livrables** : capacité négociée, dépôt idempotent, relève/ack, réponse liée,
événements de demande corrélés, table de récupération, corpus de compatibilité
et trois frontières de crash.

**Gates** :

1. l'identité `maicie` reste adressable sans connexion Maicie ;
2. exact replay et divergence sont prouvés après redémarrage ;
3. `answered`, `cancelled` et `timed_out` déposent chacun un événement durable
   sans pouvoir être reconstruits par une échéance locale ;
4. aucun chemin historique n'obtient la capacité sans négociation.

### Lot B — Greffe et cas d'usage Maicie (codeur B)

**Propriété** : domaine de requête structurée, SQLite Maicie, transaction de
réception/décision/transition et tests unitaires de la matrice.  
**Dépendances** : contrat du lot A gelé, mais le store peut être préparé contre
des fixtures contractuelles.  
**Livrables** : reçu idempotent, validation relationnelle, réponses
déterministes et interdiction structurelle de toute approbation ou texte libre.

**Gates** :

1. le même `request_id` ne double aucun effet ;
2. une référence/identité incorrecte ne révèle ni ne mute de donnée ;
3. décision et transition d'objectif ne peuvent pas être séparées par un crash.

### Lot C — Adaptateur Maicie, CLI et gate réel (codeur C)

**Propriété** : seul adaptateur public Bridget de Maicie, relève bornée à
l'ouverture de commande, projection JSON et tests d'intégration.  
**Dépendances** : lots A et B intégrés sur leurs interfaces gelées.  
**Livrables** : budget global, rendu honnête des deux vérités, réponses liées,
bench et gate de bout en bout avec agent réellement absent puis joignable.

**Gates** :

1. budget épuisé : commande rend la cause, garde l'entrée récupérable et ne
   prolonge pas l'échéance ;
2. le gate prouve dépôt pendant l'absence, relève, greffe unique et réponse
   corrélée ;
3. toute approbation par ce chemin échoue dans le vrai binaire, pas seulement
   dans un test unitaire.

### Intégration et revue hostile

1. Revue croisée du contrat A avant consommation par B/C.
2. Revue croisée B sur les transactions, testée depuis C par crash réel ;
   vérifier séparément la demande Bridget `answered` et l'objectif non clos.
3. Gate final : agent réel → guichet absent → Maicie lancée → réponse liée et
   décision unique, suivi d'un redémarrage des deux processus.
4. Revue hostile finale : falsifier les frontières de crash, retirer la
   négociation, muter un champ canonique et tenter une approbation distante ;
   chaque mutation doit faire échouer un oracle dédié.

## Critères de sortie de la session

- Le destinataire `maicie` est joignable alors que le compagnon ne l'est pas,
  sans processus résident caché.
- Les trois requêtes admises et leurs refus sont publiés dans un contrat stable
  et couverts par un corpus producteur↔consommateur.
- Une réponse liée clôt sa demande Bridget exactement une fois après accusé
  durable ; son rapport est porté au registre exactement une fois malgré les
  crashs, sans clôturer l'objectif implicitement.
- La consultation d'état et d'échéance n'introduit aucune inférence métier.
- Le test réel démontre que le chemin guichet ne peut ni approuver ni lancer un
  profil.
- Le benchmark atteste que la relève bornée n'a pas dégradé la consultation
  sous son budget configuré.

## Hors périmètre de ce plan

Cette session ne construit pas le runtime Maicie résident. Son éventuelle
création requiert une nouvelle spec v2, car elle modifierait l'exploitation,
la présence, l'arrêt et le modèle de disponibilité. Elle ne construit pas non
plus les routines F27–F29, une GUI/TUI ou une lecture LLM de rapports : ils
pourront consommer le contrat fermé du guichet, jamais l'étendre implicitement.
