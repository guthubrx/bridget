# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/analyze-report.md](../../../../../../specs/156-guidage-file/analyze-report.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Dans le modèle : agent qui utilise MCP, CLI, pont T3, wrapper. Hors modèle :
processus hostile du même compte qui forge des trames brutes (ADR 011, ADR
027). Chaque limite est documentée avec sa preuve (spec « Cadre de menace »,
`research.md` R0). Les tests de trames brutes sont de la défense en profondeur.

<a id="extrait-2"></a>

## Extrait 2

| # | Sévérité | Constat | Traitement |
|---|---|---|---|
| A1 | moyenne | Ce que reçoit l'émetteur quand un transport rejette un `steer` n'est pas relu | T002(a) ; sans avis, la doc le dit (`effect: unconfirmed`) |
| A2 | moyenne | Producteurs légitimes de `origin=Human` (attach, relais UI) hors des crates lues | T002(b) avant T015 |
| A3 | moyenne | Liste blanche T3 (Claude, Cursor) déduite de la session 129 | T002(c) |
| A4 | basse | Champs de la tâche native non relus | T002(d) |
| A5 | basse | Guidage vise 4 et non 5 | question 2 ; ADR |
| A6 | moyenne | Un `RoleHandshake{Attach}` est-il accepté sur une connexion qui a déjà une identité d'agent ? Les citations de lignes de la ronde 2 (`daemon.rs:9060-9075`, `:18125-18175`) sont approximatives | T002(e) ; l'implémenteur relit les lignes exactes |
| A7 | moyenne | Changement visible : les pairs perdent la remise en cours de tour, la CLI nue perd le contrôle | question 3 ; tests de la session 129 réécrits (T021, T029) |
| A8 | basse | Un ancien wrapper garde l'ancien `cancel_delivery` jusqu'à sa relance | noté dans la doc (T022, T027) |

<a id="extrait-3"></a>

## Extrait 3

1. Un pair du même projet peut-il guider ? Défaut : non.
2. Reporter `cancel`, `reorder`, `promote` de la file à une session dédiée ? Défaut : oui.
3. Accepter deux changements visibles : les pairs attendent la fin du tour (guidage implicite T3) ; la CLI nue ne peut plus guider ? Défaut : oui.
