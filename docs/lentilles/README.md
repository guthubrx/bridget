# Doctrine du panel de lentilles

Panel de revue Bridget / Maicie, adapté des Veilleurs (Rekall → Cartae →
MAICompany). Huit fiches dans ce dossier.

## Règle d'équilibre (utilisateur, éprouvée)

**Jamais de revue multi-lentilles sans au moins une lentille CONSTRUCTIVE.**

Un panel entièrement à charge ne discrimine plus : tout devient « à abattre »,
et l'on perd la moitié de la valeur — le *comment rendre exécutable* et le
*ce qui tient déjà*. Calibration MAICompany 2026-07-08 : les procureurs
savent dire « ça résiste » sur du solide ; les bâtisseurs (Aminata, et ici
Hiroshi) apportent autant, autrement. Un jury, pas un peloton.

Composition minimale conseillée pour un lot sensible :
1 constructif (Aminata et/ou Hiroshi) + 1 à charge + 1 neutre-exigeant selon
l'objet. Pour une décision fondatrice : les deux familles.

## Polarités

| Polarité | Fiches | Rôle |
|---|---|---|
| **CONSTRUCTIF** | Aminata (Librarian), Hiroshi (Connector) | Ce qui est bien, ce qui manque pour être complet ; liens et possibilités |
| **NEUTRE-EXIGEANT** | Aïcha (Legislator), Argos (Deal Hunter), Chandra (Pruner) | Règles, coût, minimalisme — sans parti pris d'accusation |
| **À CHARGE** | Ingrid (Sceptic), Mei-Ling (Shadow), Viktor (Entropy) | Contradictions, silences, mutants |

## Ouverture obligatoire (toutes polarités)

Toute lecture ouvre par **ce qui est déjà solide** sur pièces, puis polarise.
Une lentille à charge qui ne trouve que du négatif sur un lot sain doit écrire
« ça résiste ». Une constructive qui n'ajoute que des fleurs a raté sa lecture
autant qu'un chien qui aboie dans le vide.

## Interdit commun

Aucune complaisance. Une lentille qui approuve tout a raté sa lecture —
y compris une constructive.

## Portraits

Référence visuelle (ne pas copier dans ce dépôt) :
`/Users/moi/Nextcloud/10.Scripts/19.rekall/frontend/src/assets/agents/`
(13 portraits `_0.png` ; Argos absent de ce jeu — Veilleur Cartae n°15).

## Sources

- Axe CARACTÈRE — `specs/011-maicie-orchestration/exigences-coordination-v2.md`
- Rekall — friction constructive (`00_Vue_Ensemble_Agents.md`)
- Miroir — `00.Generic/MAICompany/40-registry/review-lenses.json`
- Calibration — `…/personas-calibration-20260708/outputs/` (dont `review-constructif-aminata-051-agent7.md`)
- Règles chantier — `docs/regles-chantier.md` ; Articles XVIII / XIX constitution SpecKit

## Protocole des deux collèges (expérience utilisateur, gravé le 2026-08-24 16h35)

Le protocole qui a donné les meilleurs résultats dans les expériences
antérieures de l'utilisateur n'est PAS deux revues parallèles indépendantes,
mais DEUX COLLÈGES DÉLIBÉRANTS :
1. Le collège À CHARGE (ex. Ingrid + Viktor) délibère EN INTERNE — les
   membres échangent leurs trouvailles, se contestent, rendent UN avis de
   collège consolidé.
2. Le collège CONSTRUCTIF (ex. Aminata + Hiroshi) fait de même de son côté.
3. RÉCONCILIATION : les deux avis de collège se confrontent — divergences
   nommées, synthèse arbitrée (référent, ou président de séance désigné).

Différence avec le panel parallèle (pilote du 24/08) : la délibération
intra-collège filtre le bruit avant la synthèse, et la réconciliation force
l'explicitation des désaccords entre polarités au lieu de les laisser au
seul juge. À dogfooder sur le prochain lot critique : 2+2 relecteurs, les
échanges intra-collège par bridget send, un avis par collège au greffe.

## Protocole du premier jury éphémère (armé le 2026-08-24 16h41, GO utilisateur)

Cible : le prochain lot CRITIQUE à relire (pressenti : l'implémentation des
routines). Dispositif à CINQ relecteurs éphémères, tous non-persistants,
nés pour la revue et arrêtés après :

- COLLÈGE À CHARGE : jury-ingrid + jury-viktor. Délibération interne par
  bridget send (2 tours max), Ingrid préside et consolide UN avis de collège.
- COLLÈGE CONSTRUCTIF : jury-aminata + jury-hiroshi. Même mécanique,
  Aminata préside.
- TÉMOIN EN AVEUGLE : jury-temoin — mandat de revue CLASSIQUE, sans
  lentille, sans mention de l'existence du jury (le mot « jury » ne doit
  pas apparaître dans son mandat). Même lot, mêmes gates.

RÉCONCILIATION : le référent confronte les deux avis de collège entre eux
puis au témoin. MESURES à publier avec le verdict :
1. trouvailles par relecteur et par collège ; recouvrements ; uniques ;
2. ce que le témoin a trouvé que le jury a raté (et inversement) — c'est
   LA mesure : lentilles = outil ou liturgie ;
3. coût L4 par juré et total du dispositif vs coût du témoin seul ;
4. temps mural.
Chaque juré naît avec préambule composé : fiche de rôle relecteur + SA
fiche de lentille. Verdicts au référent + greffe (doctrine de routage).

## Jury n°1 — résultats (2026-08-24, 17h56, lot format LIMITE @ 8b23d92)

VERDICTS : témoin seul APPROVE ; collège constructif AWC (2 conditions) ;
collège à charge AWC (3 filets). Zéro contradiction de fond entre les trois.

TROUVAILLES UNIQUES (la mesure) :
- Témoin seul : 1 — le fmt rouge sur main (hors lot, corrigé sur-le-champ).
  A aussi planté un mutant de son cru. Solide, état-présent seulement.
- Collège CONSTRUCTIF : 2 — la RÉSERVE HISTORIQUE (le SHA initial perdait
  les données legacy en silence : prouvé empiriquement, aurait mérité
  BLOCKED avant le commit de compat) ; la ligne de doc consommateurs.
- Collège À CHARGE : 3 — le faux-ami d'abréviation (contains au lieu
  d'exact), l'usedPercent hors bornes qui efface la FENÊTRE au lieu du %
  (écart réel au mandat), la projection de status non nommée.
BILAN : 5 trouvailles réelles des collèges que le solo n'a pas vues ;
1 trouvaille du solo (hors lot) que les collèges n'ont pas relevée.

CE QUE LA DÉLIBÉRATION A APPORTÉ (au-delà des trouvailles) :
- le désaccord interne NOMMÉ et tranché (Hiroshi a fait retirer une
  exigence de confort du premier jet d'Aminata — le bruit filtré DANS le
  collège, la synthèse arrivée épurée) ;
- la révision d'Ingrid (BLOCKED au tour 1 sur l'ancien SHA → AWC après
  l'addendum) — le collège juge l'HISTOIRE du lot, le témoin juge l'état ;
- des conditions de retournement explicites (AWC→BLOCKED si récidive).

LIMITES HONNÊTES DE L'EXPÉRIENCE N°1 :
1. Aveugle imparfait : le témoin s'appelait « jury-temoin » et le
   protocole est public dans le dépôt. Prochain jury : nom banal.
2. Coût L4 non mesuré : les jurés étaient des cursor — seul le flux claude
   atteste l'usage aujourd'hui. Prochain jury : jurés claude natifs pour
   la facture, ou attendre l'extension codex/acp de L4.
3. n=1 : rien de statistique. Les collèges ont gagné CETTE manche.

## Manche 4 — protocole armé (décidé le 2026-08-24 à 20:11, GO utilisateur)

**Question** : où est le genou de la courbe coût/bénéfice, et le personnage
apporte-t-il quelque chose au-delà de la polarité ?

**Lot désigné** : le lot v16 de cursor4 (délégations soldées à la clôture) —
réel, déjà livré, famille critique (migration de schéma), revue due de toute
façon : l'expérience est payée une seule fois. JAMAIS de rejeu sur un lot déjà
jugé : la vérité est écrite dans le dépôt et le ledger, on mesurerait
« retrouver », pas « trouver ».

**Trois bras en parallèle, même mandat, aveugle entre bras** :
1. **1+1 avec personas** — une fiche à charge + une constructive, qui se
   lisent et se répondent (un tour chacun + une confrontation) ;
2. **1+1 polarité nue** — deux agents frais SANS fiche mais AVEC la même
   consigne de polarité (« cherche ce qui casse » / « cherche ce qui tient »,
   puis confrontation) — sinon on compare « polarité+personnage » à « rien »
   et on ne sait pas quel ingrédient travaille ;
3. **1 seul** — témoin de contrôle.

**Lectures** : bras 2 vs 3 = valeur de la structure ; bras 1 vs 2 = valeur du
personnage seul ; bras 1 vs le 2×2 de la manche 2 = position du genou.

**Conditions matérielles** (héritées des leçons de la manche 2, non
négociables) : un CARGO_TARGET_DIR isolé par relecteur ; comptes de tests et
rouges de référence, jamais un rc seul ; binaire d'essai à l'abri ; mandat
neutre ; capture de budget Maicie CONFIGURÉE AVANT le départ (les factures de
la manche 2 sont restées « inconnu » — le motif budget_capture_non_configure
est la dette L4 à solder d'abord).

**Séquence** : jury n°3 (routines, 2×2+témoin, l'étalon) → merge routines →
manche 4 sur v16.

## Règle de décision de la manche 4 — FIXÉE D'AVANCE (2026-08-24 20:15, opposable au référent)

Le résultat de la manche 4 S'INSTALLE la nuit même comme doctrine de revue —
ce n'est pas une expérience de plus, c'est la dernière avant adoption.

- 1+1-personas trouve autant que le 2×2 de la manche 2 (trouvailles
  VÉRIFIÉES) → **adopter 1+1 avec personas** comme revue standard.
- Les personas n'apportent rien vs la polarité nue → **adopter 1+1 nu**.
- Le **2×2 complet reste réservé au tout-critique** (migrations, idempotence,
  chemins de boot/arrêt) quelle que soit l'issue.
- Résultat ambigu → **défaut : garder les personas** (faisceau manche 1-2
  favorable, surcoût d'une fiche nul).

Installation = la config gagnante entre dans docs/regles-chantier.md
(doctrine de revue), les fiches de lentilles deviennent les profils de spawn
standard des relecteurs, et toutes les revues suivantes de la nuit tournent
avec. Décision et motifs consignés ici même, chiffres à l'appui.

## Amendement du 2026-08-24 20:17 — jury n°3 ANNULÉ, manche 4 avancée (arbitrage utilisateur)

Le « 2×2 de calibration » sur les routines est SUPPRIMÉ : le 2×2 est déjà
mesuré deux fois (manches 1 et 2), une troisième mesure serait de la liturgie.
**La manche 4 se joue directement sur le lot des routines de cursor7** (le
prochain à livrer — v16 attend son merge, tout avance d'autant). Trois bras
réunis = cinq relecteurs sur le lot : pour la DÉCISION DE MERGE on prend
l'UNION des trouvailles des trois bras (couverture supérieure à un 2×2) ;
pour l'EXPÉRIENCE on compare bras par bras. Le lot v16 sera ensuite revu avec
la configuration ADOPTÉE — premier test en service réel de la doctrine.
Précision de doctrine issue du même arbitrage : la fiche de personnage est
GRATUITE (une consigne de spawn) — ce que les paliers décident, c'est le
NOMBRE de relecteurs, jamais le port de la fiche. Si les personas gagnent,
même le relecteur seul du tout-venant en porte une.
