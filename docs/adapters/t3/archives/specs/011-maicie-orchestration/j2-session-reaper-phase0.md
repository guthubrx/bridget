# Archives techniques de l’adaptateur T3

Document d’origine : [specs/011-maicie-orchestration/j2-session-reaper-phase0.md](../../../../../../specs/011-maicie-orchestration/j2-session-reaper-phase0.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La nuit du 24 a prouvé le besoin (wrappers orphelins, daemons de test, répertoires
temporaires). Les gardes à la destruction des harnais traitent la *cause* ; rien
ne nettoie les *survivants* ni les fuites d'une autre origine. T3 Code refuse de
tuer une session à tour actif **et** une session à travail de fond — leçon
reprise telle quelle. Un ramasseur qui se trompe une fois détruit la confiance :
d'où l'observateur avant l'actif, et « interdit : tuer puis demander pardon ».

<a id="extrait-2"></a>

## Extrait 2

**Constat arbitragé** : Bridget n'expose pas aujourd'hui d'équivalent de
`backgroundLiveness` (T3). En Phase 0/1, toute cible de **classe A** qui
atteindrait autrement `eligible` est forcée en **`incertain`**
(`fail_closed_background_liveness_absent`). Ce n'est pas une limite cosmétique :
c'est un **manque à instruire** (capa J2 ↔ éventuel J3). Les classes B et C ne
dépendent pas de ce signal.
