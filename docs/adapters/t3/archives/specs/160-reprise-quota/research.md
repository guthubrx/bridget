# Archives techniques de l’adaptateur T3

Document d’origine : [specs/160-reprise-quota/research.md](../../../../../../specs/160-reprise-quota/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11. Sources : lecture du checkout principal `1f2c661d` (avant la
session 150), des artefacts de 152 (révisés ronde 2) et de 157 dans le worktree,
et de T3 `origin/main` par `git show`. Aucun appel à un fournisseur. Les lignes
citées sont celles de cette base. Les sessions 150, 152 et 157 les déplacent.

<a id="extrait-2"></a>

## Extrait 2

- `limitRecoveryCommand` (T3) : état `failed`, classe d'erreur `usage_limit`, une
  heure de retour, pas de PR en attente ; heure de retour postérieure à la fin du
  run ; identité `thread:run:resetMs` ; première étape « armer » (persistée),
  seconde « relancer » (`message.dispatch`) avec `commandId` dérivé de
  l'identité. Dérivé de faits persistés, donc sans minuterie.
- **Repris** : armement persisté puis relance (nos trois étapes de FR-011),
  identité dérivée, garde « retour futur », absence de minuterie.
- **Mieux que T3** : opt-in par tâche (T3 : réglage global), borne de rondes et
  d'attente, aucun changement de fournisseur, qualification attestée par fenêtre,
  fonctionne par le parcours natif Bridget et entre outils.

<a id="extrait-3"></a>

## Extrait 3

- Attente d'au plus 6 h (21 600 s) : couvre la fenêtre de 5 h, exclut la
  fenêtre hebdomadaire.
- Au plus 2 rondes par tâche (longueur de `quota_waits`).
- Le retour doit être postérieur à l'instant de l'échec (garde de T3).
- Une fenêtre d'une ronde précédente ne qualifie pas la suivante : le fait doit
  avoir été observé après la fin de la dernière attente.
- Une identité ne sert qu'une fois (clé unique).
