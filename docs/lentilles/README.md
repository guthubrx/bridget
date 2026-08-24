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
