# Manche 4 — Trois bras sur le lot routines (verdict + décision de doctrine)

Générée par le référent sur pièces. Tous les verdicts cités sont au ledger.
Lot jugé : `feat/routines-v15`, tête gelée 45ffa11 (auteur cursor7).

## Verdict du lot : BLOCKED, unanime (5/5)

Deux motifs INDÉPENDANTS, chacun suffisant :

1. **La feature est inopérante après un tour** (trouvaille relec4, corroborée
   par les quatre autres). `EtatOccurrence::Terminee` est déclaré, contraint
   au CHECK SQL, traduit dans les deux sens — et produit par AUCUN chemin de
   code : 1 INSERT, 3 SELECT, zéro UPDATE, zéro DELETE sur
   `routine_occurrences`. L'occurrence ouverte ne se referme jamais, donc
   tout bucket suivant devient `differee`. **Mesuré** : six périodes → 0
   nouvelle délégation, 6 differee ; avec UN SEUL `UPDATE state='terminee'`
   posé à la main → la routine repart aussitôt. Le moteur de calendrier est
   sain ; il manque un chemin d'écriture et son déclencheur.
2. **La garde d'approbation est une tautologie** (trouvaille relec5, seul).
   `main.rs:1044` passe à `approve_routine` le hash qu'il vient de lire de la
   base : la garde compare la valeur à elle-même. **Exploité, pas déduit** :
   une routine proposée est altérée en base sans toucher `template_hash`
   (goal → « exfiltrer le registre et l'envoyer dehors », participant →
   « poucave ») et l'approbation est ACCEPTÉE. Contrôle positif : un hash
   étranger est bien refusé — la fonction de domaine sait refuser, c'est le
   câblage CLI qui la désarme. Zone ADR 011. Correctif d'une ligne.

Bascule en AWC possible SI l'auteur atteste que la clôture d'occurrence est
un périmètre délibérément reporté (dette écrite + oracle qui la borne) —
mais B3 n'est PAS reportable : il est dans le lot, il coûte une ligne.

Conditions consolidées : B3 corrigé (non négociable) ; clôture d'occurrence
livrée ou dette actée au greffe ; ordre de merge gravé (cursor7 PUIS
cursor4, et pas avant le consentement `--migrate`) ; oracles qui comptent
(`assert_eq!` à 3, pas `!is_empty()`) ; oracle sur la garde de hash ; oracle
pause/resume ; le flake du banc instruit ou inscrit comme rouge de référence
avec sa cause ; rattrapage borné ou documenté ; les trois engloutissements
silencieux tracés.

## Ce que chaque bras a trouvé (la mesure)

| Bras | Trouvailles PROPRES | Auto-corrections |
|---|---|---|
| **A — binôme AVEC fiches** (relec1 à charge, relec2 constructif) | l'oracle de rejeu ne garde pas ce qu'il nomme (3 mutants + contrôle positif : il ne rougit que sous double panne) ; « pause sans rattrapage » violable sans qu'un test bouge ; « zéro I/O » attesté par accident (5 oracles qui meurent sans nommer l'invariant) ; migration additive/idempotente/transactionnelle ; NON-collision de schéma entre les deux lots en vol (étape 15 identique à l'octet) ; ordre de merge contraint ; écart de forme ADR 011 (compare-and-swap non atomique) | relec2 : **3 de ses 4 « trophées » retournés** à la vérification (collision de schéma, auto-délégation, violation ADR 011) — biais nommés d'avance, parade appliquée ; puis révision AWC→BLOCKED après vérification de B3 ; puis rectification de son propre compte (« un passage ne mesure pas une stabilité »). relec1 : bascule AWC→BLOCKED sur pièce neuve vérifiée de sa main ; s'efface devant un meilleur oracle que le sien |
| **B — binôme SANS fiche** (relec3 à charge, relec4 constructif) | **la clôture d'occurrence manquante** (bloquante) ; **le banc n'est pas reproductible** : 264/0 sept fois sur dix, 263/1 trois fois sur dix, toujours le même test à échéance absolue, vert en tir ciblé 5/5, base verte 10/10 ; l'oracle de panne ne compte rien (mutant : 3 sautees → 1, banc vert) ; `horloge_arretee` ment sur ≥4 causes et l'oracle VERROUILLE le mensonge ; rattrapage non borné mesuré aux deux bornes (30 j → 43 201 occurrences, 5,8 s pour un tick, sur le chemin de toute commande) ; migration jouée sur COPIE de la base réelle (252/252 intacts, PK/FK/CHECK mordent) | relec3 : **tue sa propre charge** sur le court-circuit (mutant → 5 rouges, la promesse est gardée) avant qu'on la lui oppose. relec4 : retire son insinuation sur la renumérotation de schéma ; avoue que ses deux premiers oracles passaient pour la mauvaise raison ; corrige son compte unique après la mesure de son binôme |
| **C — SEUL, sans fiche** (relec5) | **B3, la faille d'approbation, EXPLOITÉE** — la seule trouvaille de sécurité de la manche, manquée par les quatre autres ; divergence de doctrine entre deux branches en vol sur la même erreur (break silencieux vs Err) ; mutant survivant sur le garde de rejeu AVEC l'oracle manquant fourni, prouvé rouge sous mutant | déclare honnêtement avoir croisé le ledger avant son office ; retire sa charge sur la division par zéro (le CHECK la ferme) ; corrige son propre premier comptage pollué par son banc |

## Décision de doctrine (règle pré-engagée, appliquée)

La règle écrite AVANT les données disait : personas adoptées si elles font
au moins aussi bien ; polarité nue adoptée si les fiches n'apportent rien ;
**en cas d'ambiguïté, garder les personas**. Les données :

- **Aucun bras n'a dominé.** Les trois ont trouvé des choses DIFFÉRENTES et
  complémentaires — et les deux motifs de BLOCKED viennent l'un du bras nu,
  l'autre du bras seul.
- **Le facteur actif n'est pas la fiche : c'est TENTER.** La trouvaille la
  plus grave (B3) vient du seul relecteur qui a *exploité* au lieu de lire.
  Les trouvailles bloquantes des trois bras sont toutes venues de mutants
  joués, jamais de lecture seule.
- **Ce que les fiches ont apporté, mesurable** : la discipline
  d'auto-réfutation. Le bras à fiches a produit le plus d'auto-corrections
  argumentées, dont trois biais nommés d'avance puis retournés — la parade
  écrite après la manche 2 a fonctionné exactement comme prévu.
- **Ce que la délibération a apporté** : les deux binômes ont réellement
  délibéré (contradiction frontale, cession sur mesure, ralliement motivé) —
  contrairement à ce que le référent avait cru un moment, sur un faux
  diagnostic de canal qu'il a dû rectifier. Deux verdicts ont basculé
  APRÈS confrontation, dans le bon sens.

**ADOPTÉ, en service dès cette nuit** :
1. Le jury d'étage 2 est un **1+1 à polarités croisées AVEC fiches** — pas
   un 2×2. Le 2×2 coûte le double et n'a pas montré de supériorité de
   couverture ; les fiches restent parce qu'elles sont gratuites et que leur
   apport (auto-réfutation disciplinée) est mesuré.
2. **La polarité à charge doit TENTER**, c'est désormais une exigence de
   mandat : mutants avec contrôle positif, exploitation quand la zone est
   sensible. Un tour de lecture seule ne vaut pas revue à charge.
3. **Un témoin seul est ajouté quand le lot touche la sécurité** — sur cette
   manche, c'est lui qui a trouvé la faille, et son indépendance (aucun
   contact avec les autres) est précisément ce qui l'a rendu possible.
4. Le 2×2 complet reste réservé aux lots où l'erreur est irréversible
   (migration de production, protocole diffusé, sécurité).

## Limites de cette manche, déclarées

- Les cinq relecteurs ont travaillé **sans mandat formel pendant 20 minutes**
  (panne Maicie n°5) : quatre ont produit des trouvailles majeures AVANT de
  recevoir leur office, en lecture seule. Cela brouille l'attribution
  bras-par-bras des premières trouvailles, et cela dit aussi quelque chose
  du dispositif : des relecteurs sans consigne ont trouvé.
- Le référent a gravé un constat Bloquant FAUX (« canal latéral muet ») sur
  le rapport de deux relecteurs, avant que les deux se rétractent : la
  latence d'inscription au ledger atteint 8 minutes sous charge. Le vrai
  défaut derrière, trouvé ensuite : le ledger n'est écrit qu'à l'accusé, et
  l'accusé d'un géré attend son tour de parole.
- Coût observé : cinq éphémères, ~50 minutes, targets isolés rendus (~10 Gi
  au total), aucune saturation machine cette fois — la leçon de la manche 2
  a tenu. Factures L4 toujours indisponibles (diagnostic en cours).

## Amendements post-consolidation (les relecteurs ont continué)

**B3 s'aggrave, et c'est décisif pour le correctif.** relec4 a corroboré la
faille de sa main (3e source indépendante, avec son propre contrôle positif
prouvant que la garde n'est pas morte mais MAL ALIMENTÉE — le défaut est en
`main.rs:1043`, pas en `routines.rs:192`), puis a lu l'affichage de
confirmation : `confirm_local_routine_approval` imprime le goal altéré ET,
juste à côté, `hash=` — qui est le hash STOCKÉ, donc celui d'AVANT
l'altération. **L'humain qui vérifie l'empreinte, geste que l'ADR 011
institue précisément, lit la même empreinte qu'à la proposition et en conclut
que rien n'a bougé.** Ce n'est pas une garde dégradée en vigilance visuelle :
c'est une garde dégradée en vigilance visuelle PIÉGÉE. Seul est protégé celui
qui relit le goal mot à mot. (À créditer à l'auteur au passage :
`sanitize_terminal` protège bien l'affichage contre l'injection de séquences
terminal — le réflexe est bon, c'est le chaînage qui manque.)

**Une condition dure s'ajoute, meilleure que celles déjà listées** (relec1,
reprise par relec2 contre son propre point) : l'oracle de rejeu doit
EXERCER la garde applicative et mourir sous mutation. Aujourd'hui le test
rejoue au même instant, sort au court-circuit `if after >= current` et
n'atteint jamais la garde ; retirer la garde seule laisse le contrat VERT.
L'argument de fond, qui vaut plus que la liste d'absences : *un geste sans
oracle est un trou visible, qui se comble ; un oracle présent, nommé d'après
un invariant et vert sous mutation de cet invariant, est une fausse
assurance* — la leçon de la manche 2, retrouvée ici sur l'oracle central du
lot.

**Corrections d'attribution, à ne pas gonfler** (signalées par les
relecteurs eux-mêmes, contre leur propre crédit) :
- Clôture d'occurrence : « BLOCKED avec bascule si attesté » (relec4) et
  « AWC si attesté » (relec2) sont **la même position dans deux sens** — une
  seule voix, pas deux.
- Rattrapage non borné : trouvé SÉPARÉMENT par relec3 et relec1, avec des
  bornes différentes (1 j / 30 j d'un côté, 7 j de l'autre) — c'est une
  corroboration forte à deux mains, pas deux trouvailles.
- Ce qui s'additionne réellement : deux motifs de BLOCKED INDÉPENDANTS,
  découverts par des chemins différents (la clôture par le bras nu, la garde
  par le bras seul).

**Ce que ces amendements disent du dispositif** — et c'est la mesure la plus
solide de la manche : après avoir rendu leurs verdicts, quatre relecteurs
sur cinq ont continué à se corriger *contre leur propre crédit* — retirer un
point de son « ce qui tient » parce qu'un pair l'a muté, refuser qu'on
compte deux voix là où il n'y en a qu'une, signaler l'antériorité d'un autre,
corriger une motivation fausse qui soutenait une conclusion juste. Aucune de
ces corrections ne leur profitait.

## Deux derniers apports, dont un qui protège la décision de merge

**« Le correctif est petit » n'est pas établi — contestation de relec3
contre son propre binôme.** relec4 conclut de son mutant que la clôture
d'occurrence est « petite et localisée » ; relec3 conteste, et l'argument
tient : le mutant pose l'UPDATE À LA MAIN dans un banc, ce qui prouve que le
mécanisme n'a qu'un verrou — pas que le correctif est petit. **Il manque le
DÉCLENCHEUR, et c'est lui le vrai travail** : qui clôt, sur quel événement
(fin de délégation ? clôture d'objectif ? les deux ?), avec quelle
idempotence, dans quelle transaction, et que fait-on d'une délégation
annulée ou d'une occurrence dont la délégation ne revient jamais. Aucun
chemin du lot n'observe aujourd'hui la fin d'une délégation côté routines :
c'est une **couture neuve entre deux sous-systèmes**, pas une ligne à
ajouter. Portée : « petit et localisé » est précisément l'argument qui fait
basculer un BLOCKED en AWC — le référent ne s'appuiera pas sur une facilité
que personne n'a chiffrée. Un relecteur qui affaiblit la trouvaille de son
propre binôme parce qu'elle est mal étayée : c'est le dispositif qui
fonctionne.

**Les heures déclarées par les agents sont fausses, parfois de 40 minutes.**
Vérifié par relec3 contre les `ts` d'inscription : ses deux pièces annoncées
« 21h33 » et « 21h49 » sont inscrites à 21:07:34 et 21:08:22 ; relec4
annonce « 21h47 » sur une pièce inscrite à 21:07:18 ; relec2 annonce
« 21h02 » à 21:03:17. Le décalage va de +1 à +40 minutes et il est
collectif — les agents alignent leur heure sur celle de leurs pairs au lieu
de lire l'horloge. **Seul l'horodatage d'inscription au ledger fait foi.**
Effet mesuré : toutes les cibles ont été tenues avec bien plus d'avance que
les intéressés ne le croyaient (verdicts rendus vers 21h05-21h10 pour une
cible annoncée à 21h45). Conséquence pour tout ce qui reconstruit une
chronologie — rapports, ETA, échéances : prendre les `ts`, jamais la phrase.

## B3, dernier degré : l'empreinte scelle six entrées, l'écran en montre quatre

Fait neuf déposé par relec2 (que ni relec5 ni relec4 n'avaient nommé) :
`template_hash()` couvre **six** entrées — goal, participant, period_secs,
**suite**, **depends_on**, **references** — tandis que l'écran d'approbation
en affiche **quatre** (id, participant, period_secs, goal). Donc, même en
supposant l'humain parfaitement vigilant qui relit le goal mot à mot — la
seule protection résiduelle —, une altération de `suite` (à quel objectif la
routine chaîne ses délégations), de `depends_on` ou de `references` est
**doublement invisible** : la garde ne la détecte pas (tautologie) ET l'écran
ne la montre pas. Sur trois des six entrées scellées, il ne reste aucune
protection, pas même dégradée. Seconde condition qui ne se réduit pas au
correctif d'une ligne : afficher les six entrées couvertes, ou afficher le
hash RECALCULÉ à côté du stocké. *Un écran qui montre une empreinte sans
montrer ce qu'elle scelle est trompeur même après correction du câblage.*

Convergence finale sur B3 : **quatre mains**, aucune ne l'ayant vu dans son
tour initial — relec5 l'a TENTÉ le premier (crédit d'origine), relec2 l'a
vérifié par lecture et a basculé son verdict, relec4 l'a rejoué avec le
contrôle positif décisif (« la garde n'est pas morte, elle est mal
alimentée » — le défaut est en `main.rs:1043`, pas en `routines.rs:192`),
relec1 l'a vérifié avant de s'en servir. Et relec2 a retiré publiquement sa
propre nuance atténuante, « elle protégeait le lot d'un cran qu'il ne mérite
pas ».

**Un biais de confirmation nommé, qui a frappé deux relecteurs
indépendamment** (formule de relec4, reprise par relec2 contre lui-même) :
« nous avons pris un tirage pour une mesure PARCE QU'IL CONFIRMAIT LE COMPTE
DE L'AUTEUR ». Un compte de tests qui tombe juste sur l'annonce de l'auteur
doit être rejoué, pas célébré.

## Troisième motif de BLOCKED : un doublon de MANDAT, prouvé

Trouvé par relec1 APRÈS son verdict, sur la consigne du référent « trouve ce
que personne n'a tenté » — donc une trouvaille imputable à la relance, pas au
tour initial. Tout le collège avait vérifié « zéro doublon » au niveau de la
TABLE `routine_occurrences` (clé primaire, insert direct rendant Conflict).
**Personne n'avait compté l'objet qui a un effet réel : la DÉLÉGATION.**

`delegate()` (routines.rs:337) s'exécute AVANT `insert_occurrence()` (:358),
sans transaction commune. Entre les deux, un mandat est parti et rien ne
l'atteste. Banc monté, crash simulé dans la fenêtre, reprise deux minutes
plus tard (période minimale 60 s) :

- mandat initial émis (bucket N), délégations en base = 1
- à la reprise, le bucket N est marqué `sautee / horloge_arretee`,
  `delegation_id = None`
- une nouvelle occurrence s'ouvre au bucket N+2 avec un SECOND mandat
- **délégations en base au total = 2**
- CONTRÔLE POSITIF, même scénario sans le crash : 1 délégation, 0 nouvelle
  occurrence. La fenêtre est la seule cause.

Trois conséquences, toutes vérifiées : (1) **deux mandats pour un tour de
routine** — un agent reçoit deux fois la même mission, ou deux agents la
reçoivent ; l'invariant central « rejeu = zéro doublon » est faux au seul
niveau qui compte. (2) **L'occurrence ment activement** : elle dit `sautee /
horloge_arretee` alors qu'une délégation existe et vit — corrobore relec3 sur
la fausseté du motif, par une cause qu'il n'avait pas. (3) **Le mandat
devient orphelin** : aucune occurrence ne le référence, il est invisible au
`routine show`, et `open_occurrence_for_routine` ne le voit pas — c'est ce
qui CONTOURNE l'invariant « occurrence vivante diffère la suivante ».
L'invariant tient dans le test et se fait déborder en production par un
chemin qu'aucun oracle n'exerce. Aggravant : le battement s'exécute à chaque
commande maicie, et il y a eu deux pannes totales de Maicie cette nuit — la
fenêtre n'est pas théorique.

Remède suggéré (le moins invasif des trois proposés) : au moment de marquer
un bucket `sautee`, vérifier d'abord qu'aucune délégation ne porte la clé
`routine:{id}:{bucket}` — si elle existe, l'occurrence doit être `ouverte` et
adopter le mandat existant, jamais `sautee`. Banc conservé pour l'auteur,
rouge par construction, avec son contrôle positif.

À noter pour la mesure du dispositif : relec1 a d'abord soupçonné que le
rejeu cassait l'idempotence de délégation (les horodatages dans la requête),
l'a VÉRIFIÉ et RÉFUTÉ lui-même (`canonical_request_bytes` ne sérialise aucun
horodatage), puis a trouvé le vrai chemin. « Ma lentille voulait le contraire,
et c'est le moment où il faut vérifier une fois de plus. »

## Le dernier angle, clos À DÉCHARGE par la polarité à charge

relec4 avait signalé que la concurrence de deux relèves simultanées n'était
exercée par personne. relec3 l'a exercée — c'est de la charge : tenter ce que
le lot déclare tenu. Banc : deux `evaluate_routines` sur la même base, chacun
sa connexion (le cas réel de deux commandes maicie lancées ensemble), avec
barrière de synchronisation pour forcer l'entrelacement, **20 tirages par
scénario, pas un passage**.

- **20/20 : une seule occurrence, un seul objectif créé. Zéro doublon.** La
  clé (routine_id, bucket) résiste à l'attaque en concurrence réelle.
- **20/20 : le tick perdant remonte `Conflict` et abandonne** — mais sur 3
  routines actives, **0 tirage avec une routine non battue** : l'ordre
  déterministe de `list_routines` fait que le gagnant couvre tout.
- Non mesuré, déclaré : deux ticks avec des `now` différents (buckets
  différents). Angle restant ouvert.
- Subsiste, mineur : le conflit est avalé par `let _ = evaluate_routines` —
  bénin ici puisque le travail est fait par l'autre tick, mais c'est la même
  mécanique que les engloutissements silencieux ; renforce la condition
  ferme « laisser une trace », pas les conditions dures.

**Donnée pour la comparaison des bras** : c'est la DEUXIÈME charge que ce
relecteur rend de lui-même (après le court-circuit, dont le mutant produit 5
rouges — la promesse était réellement gardée). *La polarité à charge ne
produit pas que des charges, à condition de mesurer avant de conclure.*
