# Leçons de méthode de la nuit — à porter dans docs/regles-chantier.md

Déposées ici et non dans les règles elles-mêmes : sans `git`, une
modification non commitée serait à découvert et un `git checkout` la
détruirait. **À intégrer aux règles au retour du disque.**

---

# 0. LE CRITÈRE DE TRI — à appliquer à tout ce qui suit

> **Une leçon qui ne survit pas au retrait des noms n'est pas une leçon,
> c'est un palmarès.**

**Règle de placement : dans le DOSSIER on attribue ; dans les RÈGLES on
anonymise.** L'attribution est nécessaire à la traçabilité d'un dossier en
cours — savoir qui a mesuré quoi, pour pouvoir le contester. Elle est
nuisible dans une règle, où elle transforme un enseignement en classement.

Raison de fond, donnée par ses auteurs : **un greffe nominatif meurt avec
ses agents ; un greffe anonyme est le seul qui se transmette.** Les
relecteurs de cette nuit ne seront peut-être pas là au prochain lot. Ce qui
doit survivre n'est pas qui a trouvé quoi — c'est la classe de défaut, la
parade, et le geste de mesure.

*Appliqué plus tôt, ce critère aurait épargné plusieurs tours passés à
s'attribuer et se refuser des crédits, sans rien produire d'utile au lot
suivant.*

## 0 quater. LA RÈGLE DE LA POLARITÉ À CHARGE

> **Un motif qu'on n'a pas essayé de tuer soi-même ne vaut rien devant un
> auteur qui, lui, essaiera. Mieux vaut qu'il tombe de la main du relecteur
> que de celle de l'auteur.**

Ce n'est pas une règle de vertu, c'est une règle d'efficacité — et c'est la
raison pour laquelle un relecteur a démonté **quatre de ses cinq motifs**,
dont celui qu'il avait le plus d'intérêt à garder.

Corollaire, qui explique pourquoi le motif survivant mérite d'être lu :
**rien de ce qui l'entourait n'a survécu.** *Un dossier où tout tient est un
dossier qu'on n'a pas assez attaqué.*

Effet secondaire observé, et il est majeur : à force d'attaquer, le collège
a établi que le remède qu'il visait était **structurellement fiable** — un
résultat à décharge qu'aucune revue bienveillante n'aurait produit, parce
qu'elle n'aurait pas cherché les portes par lesquelles il pouvait échouer.

## 0 ter. FACE À UNE QUESTION QU'ON NE PEUT PAS MESURER : ÉNUMÉRER LES ISSUES

> **Énumérer les issues exclusives, puis identifier la SEULE lecture qui les
> tranche.**

Transformer « il faudrait chercher quelque part » en « lis cette clause, et
selon ce qu'elle dit, l'angle est **clos**, **rouvert**, ou **déplacé** ».
Trois issues, une lecture, réponse en un échange — obtenue **sans shell, la
nuit d'une panne totale**, sur une question que personne ne pouvait mesurer.

C'est plus utile qu'un verdict : un verdict vaut pour un lot, ce geste vaut
pour toutes les questions qu'on ne peut pas exécuter. *(Et le premier geste
du repreneur devient trivial : une lecture au lieu d'une campagne.)*

## 0 bis. CE QU'UN COLLÈGE 1+1 PRODUIT RÉELLEMENT — forme anonyme

- **Six corrections mutuelles, aucune sur une divergence de jugement** —
  toutes sur des **fautes de méthode**. Un collège à deux bras ne sert donc
  pas d'abord à arbitrer entre deux lectures : il sert à **empêcher chaque
  bras de conclure trop vite sur les faits**. Un jury à un seul bras
  n'aurait pas été faux sur le jugement — il aurait été faux sur les faits,
  sans personne pour le lui dire.
- **Les fautes de chaque bras étaient les angles morts de sa propre
  lentille** : conclure d'un contexte plausible au lieu de lire la donnée
  d'un côté ; accepter trop vite une explication qui arrange, de l'autre.
  Ce ne sont pas des défauts d'attention — et c'est précisément pourquoi le
  jury est à deux bras.
- **Une revue qui compile a un coût matériel, et ce coût doit être borné
  comme le reste.** Cette nuit, les répertoires de compilation des
  relecteurs ont saturé le disque et rendu impossible le tir qui tranchait
  le dernier motif : **le dispositif a consommé la ressource dont il avait
  besoin pour conclure.**

---

## 1. Un rouge reproductible chez soi n'est pas un rouge reproductible

*C'est peut-être sa machine qui est reproductible.*

Un relecteur a mesuré un test rouge 8 fois sur 8 — sur le lot **et** sur la
base — et en a conclu que le rouge était « permanent ». Son binôme l'a tiré
en isolation **propre** : **5 verts sur 5**. La différence n'était donc pas
dans le lot mais dans son environnement à lui — il avait pris pour une
preuve d'indépendance ce qui était la signature de son propre biais.

Il avait **raison** de refuser le mot commode « flake », et **tort** sur ce
qu'il fallait mettre à la place. Corollaire : il faut un **contrôle positif
sur son propre environnement de test**, pas seulement sur son instrument.

## 1 bis. Un rouge MAL ATTRIBUÉ est plus dangereux qu'un rouge INEXPLIQUÉ

Suite directe, et c'est la faute commise **dans l'acte même de corriger la
précédente** : le relecteur a ensuite attribué son rouge à un `TMPDIR` non
conforme, **en citant** la mesure d'un pair — ce qui donnait à son
raisonnement l'autorité d'une mesure qui n'existait pas. Réfuté par ce pair :
les deux défauts `TMPDIR` connus ont des symptômes **nommés** (`SUN_LEN` sur
des sockets unix ; « répertoire non privé » sur une ouverture SQLite), et
aucun n'est celui-ci, qui est un test **sensible au temps**.

**La cause reste inconnue** : les deux « isolés » divergent sur la même base
(5 verts chez l'un, 3 rouges chez l'autre), quatre candidates — charge,
pression disque, horloge, `TMPDIR` — et **zéro mesure discriminante**.

> *Un rouge inexpliqué appelle une mesure ; un rouge mal attribué CLÔT
> l'enquête et se réveille en fausse piste.*

**LE MÉCANISME QUI EXPLIQUE LA LEÇON 1** — et c'est lui qu'il faut retenir :

> **Une cause environnementale est invariante par SHA, donc elle imite
> parfaitement la signature d'un défaut hors lot.**

Le relecteur obtenait son rouge sur le lot **et** sur la base, et prenait
cette concordance pour une preuve d'indépendance. C'est exactement ce que
produit un environnement non contrôlé. La concordance lot/base ne prouve
donc **rien** à elle seule.

**Et dire « non contrôlés », jamais « viciés »** : « vicié » désigne une
cause, or aucune n'est établie. Des tirages non contrôlés sont **non
exploitables** — ce qui n'est pas la même chose que « telle variable en est
la cause ». Nommer une cause non démontrée, même contre soi-même, reste une
faute d'attribution.

**Ce qui survit de l'enquête**, sur les mesures à `TMPDIR` constant et
conforme : 5 verts sur 5 en isolé, 1 rouge aux trois passages en suite
complète. Donc **le test est déterministe par contexte d'exécution, pas
aléatoire — « flake » reste le mauvais mot**, même si la variable
responsable demeure inconnue.

**Transposer une cause mesurée sur un autre symptôme n'est pas une mesure**,
même en citant sa source. « C'est peut-être ma machine qui est
reproductible » reste juste ; ce qui était faux, c'était de croire savoir
**pourquoi**.

## 1 ter. UN COMPTE SANS SES CONDITIONS D'EXÉCUTION N'EST PAS UNE MESURE

> **C'est une opinion chiffrée — et deux comptes contradictoires conservés
> valent mieux qu'un compte propre choisi.**

Un relecteur avait demandé qu'on retienne les comptes de son binôme plutôt
que les siens, mieux contrôlés. Son binôme a refusé, et il avait raison :
des mesures plus contraintes **ne prouvent pas que les autres sont fausses**,
seulement qu'elles le sont moins. Surtout, **le fait brut — deux mesures
« isolées » divergentes sur la même base — est précisément la donnée qui
désigne l'environnement comme variable.** Effacer l'un des deux jeux
détruirait la seule trace exploitable.

Donc : **conserver chaque compte avec ses conditions d'exécution** (TMPDIR,
charge, cible, nombre de passages), et conserver **les jeux contradictoires
ensemble**.

## 2. Régler son répertoire temporaire avant de mesurer

`TMPDIR` doit être **court ET en 0700** (`mktemp -d /tmp/xx.XXXX`). Trop
long : « path must be shorter than SUN_LEN ». `/tmp` nu : « répertoire non
privé ». Un relecteur a payé **treize faux rouges** avant de trouver.

## 3. Vérifier l'objet qu'on vous donne à juger

Le référent a nommé **deux fois** dans un mandat une tête qui portait le
défaut actif du lot. Les relecteurs ont vérifié la chaîne avant de juger et
ont rendu un verdict par SHA. Sans cela, un verdict favorable aurait porté
sur un objet embarquant le défaut, et le merge l'aurait introduit.

## 4. Avant d'exiger un changement, chercher si le dépôt le fait déjà

Un relecteur a trouvé **deux précédents documentés** du pattern qu'il
réclamait, dont un dans le fichier même que l'auteur modifiait. Cela
transforme une exigence de relecteur en **incohérence interne**, et chiffre
le coût du correctif par le codebase lui-même.

## 5. Quand un auteur corrige vite sous revue, le danger n'est pas l'oubli

C'est la **régression née du remède**. Sur un lot corrigé en neuf SHA,
l'inventaire complet n'a révélé **aucun abandon** — mais deux défauts
*ajoutés* par les correctifs : un écran devenu menteur, et un point de
coupure de test devenu code de production.

## 6. Un critère de blocage automatique ne porte que sur un ORACLE ASSERTIF

Le référent avait écrit « si ce banc est rouge, le lot reste bloqué ». Or le
banc finissait par un `panic!` **inconditionnel** : banc d'observation,
rouge même quand le lot est sain. **Avec ce critère, on bloquait un lot
sain.** Le critère juste portait sur un **compte** (« le nombre de
délégations vaut 2 »), pas sur une couleur.

## 7. Ne pas borner le cumul des répertoires de compilation

La règle « un répertoire par relecteur » a évité les corruptions croisées,
mais **personne ne borne le total**. Un relecteur qui compile en *release*
en plus du *debug* double sa facture — et c'est ce qui a saturé le disque et
tué les shells de toute la flotte. Prévoir un plafond, ou une vérification
d'espace avant de lancer une campagne.

## 8. Graver la FORMULE, jamais un chiffre nu

Un relecteur a accusé l'auteur d'un compte faux (43135 contre 43136), avec
insistance, et a fait relayer la correction par le référent. **Les deux
chiffres étaient justes** : la formule `truncated_end - after - 1` est
exacte, et le résultat dépend de l'état initial — selon qu'une relève a déjà
eu lieu ou non. Il avait transformé une **différence de scénario** en erreur
de l'auteur.

Tout exemple chiffré doit **préciser son état initial**, sinon il induit en
erreur. Et un relecteur qui met en cause un chiffre doit d'abord se demander
si les deux valeurs ne décrivent pas deux cas.

## 9. Un banc vert peut l'être hors du bon régime

Variante inédite, trouvée par le relecteur qui l'avait lui-même produite :
ses bancs, intégrés de bonne foi par l'auteur, reprennent à **+2 périodes**
alors que le défaut restant vit **au-delà de 64**. Ils n'exercent pas le
régime que la borne gouverne. *« Deux correctifs justes se sont croisés et
ont laissé un trou entre eux. »* Ce n'est pas un banc rendu vert
artificiellement — c'est un banc vert **au mauvais endroit**.

## 8 bis. LIRE LA DONNÉE, PAS LE CONTEXTE PLAUSIBLE — une racine, trois symptômes

Un relecteur a affirmé qu'une colonne appartenait à une table donnée. Elle
appartient à une **troisième** table. Cause qu'il nomme lui-même : *il avait
lu un `INSERT` par sa liste de colonnes **sans lire la ligne qui nomme la
table***.

C'est **la même racine** que ses deux autres erreurs de la nuit :
- le chiffre juste dans un scénario, donné pour une vérité générale (sans
  lire l'état initial dont il dépendait) ;
- les heures annoncées au ressenti (sans lire l'horloge).

**Trois symptômes, une seule cause : conclure d'un contexte plausible au
lieu de lire la donnée.** C'est le défaut le plus difficile à voir chez soi,
parce que le contexte plausible *est* presque toujours le bon — et qu'il ne
se signale jamais quand il ne l'est pas.

*Effet secondaire notable : cette erreur-ci était **à décharge** pour
l'auteur du lot une fois corrigée. Un relecteur qui lit mal ne charge pas
toujours — il peut aussi laisser un angle ouvert qui devrait être clos.*

## 9 bis. LES CONDITIONS DU JURY DOIVENT ÊTRE RELUES COMME DU CODE

**Deux occurrences dans la même nuit, sur le même lot** — nos exigences ont
elles-mêmes produit des défauts :

1. Nous avons fait retirer un `let _ = evaluate_routines(...)` dont le
   commentaire disait *« un échec du tick ne doit jamais faire échouer
   status / delegate »*. Résultat : une collision entre deux ticks
   concurrents fait désormais **échouer la commande utilisateur** — et deux
   commandes simultanées sont le régime normal d'une flotte.
2. Un complément répondant à une de nos demandes a rendu **fausse** une
   ligne d'écran (« recalculé depuis les champs affichés », alors que trois
   champs scellés ne l'étaient pas).

> *Une condition de revue est une modification du code, écrite par
> quelqu'un qui ne l'exécutera pas. Elle mérite le même examen que le code
> qu'elle corrige — et le même bénéfice du doute pour l'auteur qui
> l'applique.*

Corollaire pratique : quand une condition demande de **retirer** une
protection existante (un `unwrap_or`, un `let _`, un repli), exiger que le
mandat dise ce qui remplace la protection retirée — sinon on déplace le
défaut au lieu de le corriger.

## 10. Consolider par écrit, pas par messages croisés

La latence du ledger a atteint **huit minutes**. Six agents ont passé deux
heures à répondre à des états vieux de plusieurs échanges, chacun corrigeant
un motif déjà clos chez l'autre. Sur un sujet à plusieurs mains, tenir **une
note** que tout le monde relit vaut mieux que dix messages qui se croisent.

## 11. Un nom faux sur une analyse juste se paie deux fois

Le motif de l'adoption hors borne a été baptisé **« doublon de mandat »**.
Le compte disait `2`, et personne n'a demandé *quel bucket* portait chaque
mandat. La mesure a montré autre chose : offsets `[0]`, puis `[0,65]` et
`[0,100]`, le bucket de la coupure **absent** du `produced`. Ce n'est donc
pas un doublon — c'est un **mandat neuf sur le bucket courant pendant que
celui de la coupure reste orphelin, définitivement hors fenêtre**. Le fait
était juste, le nom était faux, et le nom est ce que l'auteur corrige.

**Le plus instructif n'est pas l'erreur, c'est sa récidive** : la même faute
avait déjà été commise sur le motif précédent, signalée par un relecteur à
l'autre, et nommée. Elle a été refaite ensuite, à deux, sur le motif suivant.

> *Corriger l'occurrence ne corrige pas le réflexe. Un motif ne se nomme
> qu'après avoir demandé ce que le compte compte.*

Règle opérationnelle : **un compte nu ne qualifie jamais un défaut.** Avant
d'écrire un nom au registre, exiger la dimension qui le désambiguïse — quel
bucket, quelle clé, quel destinataire, quel instant.

## 12. Un shell substitué tue toute exécution en silence

La flotte entière — vingt agents — a perdu ses mains pendant **quatre heures
et demie**. Deux diagnostics successifs ont été rendus, tous deux faux : le
disque plein (réel, mais non causal), puis la table des processus (fausse).
La cause était que `/opt/homebrew/bin/zsh`, désigné par `$SHELL`, avait été
remplacé par un script de 32 octets **sans bit d'exécution**. Or un agent
exécute ses commandes par `$SHELL -c "…"` : un shell non exécutable rend un
code d'erreur **sans stdout ni stderr**.

**Les deux signatures qui auraient fait gagner quatre heures** :
- une commande qui échoue **sans une seule ligne de sortie**, `echo` compris,
  n'est pas une commande qui échoue — c'est **le shell qui ne démarre pas** ;
- sur macOS, `df /` mesure la partition système en lecture seule et affiche
  un taux rassurant. **Le seul chiffre qui compte est
  `df -h /System/Volumes/Data`.** Cette erreur a produit le second faux
  diagnostic, alors que la note qui la contenait mettait explicitement en
  garde, deux paragraphes plus bas, contre le fait de mesurer le mauvais
  répertoire.

> *Écrire l'avertissement ne protège pas de l'erreur ; seule la commande
> juste protège.*

Corollaire de méthode, vérifié : la restauration du lien a rendu les mains à
tous les processus vivants **sans aucune relance de session**. Mais un
processus **neuf** avait des mains alors que le shim était encore en place —
ce qui a permis de mesurer pendant la panne. Quand la flotte est muette,
**spawner un agent neuf sous un nom libre** est le test le moins cher, et il
n'abîme rien.
