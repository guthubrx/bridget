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
