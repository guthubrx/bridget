# Décision 034 — Gouverner les engagements, pas la pensée

**Statut** : orientation étayée ; implémentation hors périmètre de ce lot

**Date** : 26 août 2026

**Base** : `b0a8cea00bddaa50ee243042f90eb141ba50e467`

## Décision en une phrase

Nous n'introduisons pas de moteur de graphe. Nous faisons du guichet central
la porte unique d'un macrographe de contrôle minimal : transitions critiques
typées et durables, projection d'état versionnée, refus machine aux frontières
irréversibles, réconciliation des arêtes jamais empruntées et voie latérale de
capture sans élargissement automatique du périmètre.

Le travail intellectuel d'un agent reste orienté objectif et non prescrit par
une machine d'état détaillée.

## Pourquoi cette décision existe

En douze heures, huit défauts d'orchestration ont été mesurés :

1. sept lots livrés sur dix-sept n'ont jamais été routés alors que onze
   relecteurs étaient vivants ;
2. un verdict rendu n'a jamais été inscrit et a disparu de l'inventaire
   suivant ;
3. des objectifs de revue ont été ouverts sur des versions périmées ;
4. sept délégations ont été créées sans mandat envoyé ;
5. un lot a été routé à son propre auteur ;
6. un lot a été exécuté deux fois ;
7. une ronde lancée toutes les sept minutes pendant douze heures n'a jamais
   examiné les branches ;
8. douze lots sur douze ont été envoyés en jury complet alors que le critère
   n'en désignait que quatre.

Les règles, compteurs et promesses écrites ont en outre été contournés ou
oubliés. Le seul mécanisme ayant effectivement arrêté l'action interdite était
un refus machine.

Ces étapes formaient déjà le graphe implicite suivant :

`objectif → délégation → mandat → travail → livraison → routage → revue → verdict → merge → clôture`

Le défaut n'était donc pas l'absence de graphe. Les arêtes critiques n'étaient
ni des données, ni des transitions opposables : elles dépendaient de la mémoire
et de la discipline du référent.

## Pièces du greffe et rôle de chacune

Ce dossier ne remplace pas les entrées du greffe. Il les situe dans la chaîne
de raisonnement et conserve leurs identifiants stables. Les corps exacts ont
été lus dans `docs/catalogue-du-du.md` au SHA
`b0a8cea00bddaa50ee243042f90eb141ba50e467` :

### `doctrine/murs-sur-le-progres-rattrapage-confronte-sur-la-visibilite`

Cette doctrine porte le diagnostic initial : le problème n'est pas un simple
manque d'observation. Elle distingue deux monnaies qui ne se remplacent pas :

- **progrès** : ce qu'une étape autorisée a effectivement fait avancer ;
- **visibilité** : ce qui a été observé, signalé ou rendu confrontable.

Elle oppose des murs sur le progrès aux mécanismes de rattrapage sur la
visibilité. Elle distingue aussi :

- le **contournement**, où une règle est évitée ;
- le **débordement**, où une trouvaille utile naît hors du périmètre prévu ;
- la **course**, où deux événements légitimes se croisent dans le temps.

Cette distinction empêche un refus trop large de tuer trois résultats majeurs
de la nuit : un travail livré après retrait de son objectif, une fuite trouvée
hors périmètre et une mesure rendue par un agent que l'on venait d'arrêter.

### `doctrine/cinq-refus-machine-et-etats-de-revue-chez-maicie`

Cette doctrine traduit le diagnostic en cinq gardes, avec leurs bornes :

| Refus | Propriété protégée | Danger si la borne est mauvaise |
|---|---|---|
| R1 — rendre visible sans mandat attesté | une délégation visible correspond à un mandat réellement parti | vérifier trop tôt encourage un chemin de message privé hors greffe |
| R2 — router une revue vers l'auteur | séparer conflit d'intérêt et jugement | un blocage sans voie normale pousse les routes hors Maicie |
| R3 — accepter un verdict non structuré ou sans version | rendre le verdict calculable et lié à l'objet jugé | bloquer le message au lieu de l'inscription perd la preuve |
| R4 — déclarer livré sans revue liée | empêcher une livraison de sortir du circuit | un refus immédiat mal conçu tue une course légitime ; pièce et orphelin doivent être distingués |
| R5 — fusionner ou clore sans verdict favorable sur la version | lier la décision irréversible à la version réellement jugée | une urgence humaine doit rester possible par une dérogation explicite et durable |

Les bornes exactes comptent autant que les refus : R1 exige l'injection du
mandat, pas sa consommation ; R2 compare des identifiants d'agents et prévoit
un second relecteur pour l'exception documentaire ; R3 accepte une version ou
un motif fermé `sans_code` et bloque l'état jugé, jamais la discussion ; R4
laisse l'annonce et la version visibles tandis qu'il refuse l'étiquette
`livré` ; R5 exige l'égalité de version et réserve le hotfix à un consentement
humain typé.

L'ordre de conception proposé était R1, R2, R3, puis R4 avec traitement des
pièces et orphelins, enfin R5. La même doctrine établit que Maicie doit porter
des états de revue et un identifiant de version opaque sans devenir un client
Git : le registre stocke le fait, un adaptateur vérifie la version.

### `etude/control-graph-gouverner-les-engagements-pas-la-pensee`

Cette étude inscrit la confrontation externe reproduite dans `recherche.md`.
Elle confirme le choix des frontières opposables, mais corrige deux éléments de
la doctrine initiale et ajoute une pièce absente :

1. la séparation auteur/relecteur protège contre le conflit d'intérêt, mais la
   séparation topologique seule ne prouve pas la qualité ;
2. un état structuré apporte déjà audit et détection avant que toutes les
   arêtes deviennent des refus ;
3. un journal ne détecte pas une arête qui n'a jamais été appelée : il faut une
   **réconciliation**.

### `constat/le-fond-de-panier-est-fait-de-victoires-non-soldees`

Cette mesure postérieure renforce la décision sans la remplacer. Sur les douze
plus anciennes entrées du registre :

- sept étaient déjà résolues dans le code mais jamais éteintes ;
- une était périmée et portait sa propre réfutation ;
- trois seulement étaient de vraies attentes ;
- une restait indécidable sans remesure.

Le blocker le plus ancien, affiché ouvert depuis quarante-quatre heures, était
résolu depuis trente-huit heures. Le registre ne mentait pas seulement par
omission : il mentait **par excès**, faute de transition de fermeture et de
réconciliation. La présence d'une entrée ne prouve donc pas qu'elle reste due.

Trois mesures indépendantes convergent désormais : l'instrument de backlog ne
peut pas dire si un lot est jugé ; le référent a failli faire relire un lot
déjà jugé ; sept constats résolus restent ouverts. Dans les trois cas, le
registre ne porte pas un état exploitable et réconcilié. Ce n'est plus une
déduction issue d'un framework externe.

## Ce que nous retenons

### 1. Une seule porte de vérité

Le guichet central reste la seule porte d'écriture. Un outil ou une interface
future peut être un client mince de cette porte, jamais un accès direct au
store ni une base locale de secours.

### 2. Un macrographe, pas un graphe cognitif

Le graphe ne porte que les engagements et décisions observables : mandat,
livraison, revue, verdict, autorisation de merge et clôture. Il ne modélise ni
les pensées, ni toutes les sous-étapes d'enquête ou d'implémentation.

Cette borne est soutenue par le retour OpenAI Symphony : le graphe opératoire
sur les tickets fonctionne, tandis que traiter les agents comme des nœuds
rigides s'est révélé trop limitant.

### 3. Journal durable et projection rapide

Le journal append-only est l'autorité historique. Une projection d'état
répond rapidement à la ronde et aux agents, mais elle porte une révision
monotone et son dernier événement. Une action fondée sur cette projection doit
fournir la révision attendue et échouer si elle est périmée.

Le journal doit rester sélectif. Un event sourcing généralisé ajouterait des
migrations, des problèmes de compatibilité et une dette de projection sans
bénéfice démontré.

### 4. Réconciliation, pas simple enregistrement

Une boucle déterministe cherche les contradictions et les omissions :

- livraison sans revue en attente ;
- mandat réservé sans émission attestée ;
- revue visant une tête différente de la tête courante ;
- verdict favorable sans transition ultérieure attendue ;
- clôture ou merge sans chaîne probante.

Chaque anomalie produit une ligne durable, puis une réparation idempotente ou
un refus typé. Sans cette boucle, les sept livraisons jamais routées restent
invisibles précisément parce que l'événement suivant n'a jamais existé.

La boucle doit aussi chercher le cas symétrique : une dette affichée ouverte
dont la propriété est déjà satisfaite par un merge, une fermeture ou une
preuve durable. Les sept victoires non soldées montrent qu'une projection peut
être fausse aussi bien par manque que par persistance excessive.

### 5. Une entrée latérale bornée

Un événement `finding_proposed` conserve la provenance, la preuve et une clé
de déduplication. Il ne change pas le périmètre et ne démarre pas de travail.
Une étape distincte décide de promouvoir, différer, dédupliquer ou refuser la
trouvaille.

Cette voie protège le débordement utile sans permettre à un agent d'élargir
seul son mandat.

### 6. Une revue vaut par son signal orthogonal

`auteur != relecteur` reste obligatoire contre le conflit d'intérêt. La
qualité de la revue doit toutefois être justifiée par au moins un signal
distinct : tests exécutables, plateforme différente, preuve externe, contexte
masqué, données spécialisées, arbitrage humain ou droit d'abstention.

Un second passage du même modèle avec le même contexte n'est pas assimilé à
une preuve indépendante.

## Arêtes minimales à rendre opposables

| Incident | Transition ou contrôle minimal |
|---|---|
| délégation sans mandat | ne passer à `active` qu'après `mandate_dispatched` durable et reçu |
| livraison jamais routée | créer atomiquement `review_pending`, puis réconcilier tout retard |
| verdict perdu dans la prose | seul `verdict_recorded` structuré, corrélé au lot et à la version exacte, vaut verdict |
| tête périmée | refuser si `target_version != current_version` |
| auto-revue | refuser l'identité auteur = relecteur, sauf dérogation humaine durable |
| travail dupliqué | lease actif unique, clé d'idempotence et conflit explicite |
| ronde aveugle aux branches | confronter projection du greffe et inventaire Git, sans confondre observation et vérité métier |
| merge ou clôture non prouvé | refuser sans verdict favorable sur la même version ou override humain typé |

La doctrine conserve ses cinq refus dans l'ordre R1 à R5. La recherche regroupe
quatre capacités d'infrastructure à plus fort rendement — mandat attesté,
`review_pending` atomique à la livraison, verdict structuré sur version exacte,
et refus de merge/clôture sans ce verdict — auxquelles R2 ajoute la garde
d'identité. La réconciliation accompagne ces transitions ; elle n'est pas
reportée comme un tableau de bord.

## Pourquoi nous ne choisissons pas un moteur de graphe

1. Les incidents exigent des prédicats métier précis. LangGraph ou Temporal ne
   savent pas ce qu'est un mandat réellement parti, un verdict opposable ou la
   version jugée.
2. Le guichet central possède déjà enveloppe canonique, idempotence, leases,
   expiration, corrélation et issues typées. Un nouveau moteur créerait une
   seconde porte à garder.
3. Les moteurs persistés déplacent le coût vers la compatibilité des graphes,
   la migration des checkpoints, la déduplication des effets et les workflows
   en vol.
4. Le cas actuel tient dans un macrographe court. L'implémentation la plus
   simple est d'étendre la sémantique du guichet, pas de remplacer son runtime.

Un moteur externe deviendrait rationnel si la chaîne traversait plusieurs
services ou machines et exigeait durablement reprise après crash,
compensations, longues pauses, fan-out/fan-in massif ou versionnement simultané
de nombreux workflows en vol. Ce seuil n'est pas établi aujourd'hui.

## Ce que cette décision corrige dans la doctrine initiale

- **Correction 1 — visibilité structurée** : elle a une valeur avant le refus,
  car elle rend les omissions calculables. Elle ne protège qu'avec une
  réconciliation ou une barrière.
- **Correction 2 — vérificateur séparé** : la séparation combat le conflit
  d'intérêt ; le gain de qualité vient de l'oracle ou de l'information
  orthogonale, pas de la topologie.
- **Correction 3 — événement manquant** : un journal ne voit que ce qui a été
  émis. Une ronde de réconciliation doit rechercher les arêtes absentes **et
  les états restés ouverts après résolution**.
- **Correction 4 — débordement** : le graphe a besoin d'une sortie latérale
  `capture → triage → promotion`, sans extension implicite du mandat.

## Ce que ce lot ne tranche pas

- le nom final des événements et états ;
- le schéma SQL, le format du journal et la stratégie de migration ;
- le délai de réconciliation et le propriétaire de chaque réparation ;
- la durée des leases et la politique de reprise après expiration ;
- la forme exacte de l'override humain et les actions qui l'autorisent ;
- la rétention des preuves et projections ;
- la diffusion d'une vue centrale fraîche aux agents distants, dont le
  catalogue local observé était vide ;
- le seuil quantifié qui justifierait un moteur externe ;
- les graphes de boucles autonomes à l'horizon de douze mois.

Ces points nécessitent une spécification d'implémentation séparée. Le présent
lot n'autorise aucun changement de code, de base ou de comportement.

## Conséquences

### Positives

- les huit défauts sont reliés à des contrôles vérifiables ;
- une personne absente de l'incident peut reconstituer le choix ;
- les découvertes hors périmètre restent capturables sans scope creep ;
- le système conserve une seule porte de vérité ;
- la complexité est limitée aux frontières où elle achète une propriété.

### Négatives et coûts assumés

- les événements et projections devront être versionnés et migrés ;
- la réconciliation introduit une boucle opératoire à tester ;
- les refus trop larges peuvent déplacer le travail hors circuit ;
- la voie latérale demande déduplication et triage ;
- une revue réellement orthogonale coûte davantage qu'une auto-révision.

## Critère de réexamen

La décision doit être réévaluée si une mesure montre que le macrographe du
guichet ne peut plus assurer atomicité, reprise ou lisibilité sans dupliquer un
moteur de workflow durable. Une préférence de vocabulaire ou une
visualisation plus attractive ne constitue pas ce signal.
