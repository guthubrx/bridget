# Référence de non-régression T001 — session154

Base : `89f7a4163c2fb27229587f50a46414295ba3674d`, incluant153 livrée.
Mesures et tests exécutés par le principal avant modification des sources154.

## Passage canonique

193PASS, 0FAIL, 1ignoré. Retour0. Durée87,63s. 80binaires exécutés.
Sources et binaires stables dans le reçu. Aucun nouveau test ni scénario créé.
Les six filtres ont été groupés sur les binaires exacts déjà compilés, sans Cargo
ni nouvelle compilation lors du passage canonique.

| Filtre | Cas correspondants |
|---|---:|
| `native148` | 28 |
| `native149` | 78 |
| `native150` | 51 |
| `delegation149` | 10 |
| `spec100` | 16 |
| `spec_087` | 11 |

Total : 194cas correspondants, dont193 réussis et1 ignoré.
Cas ignoré : `native148_parent_codex_fixture_delegates_once_to_real_glm`. Il reste ignoré ; aucun succès n’est déduit.

Recette : environnement vide (`env -i`), umask0077, racine privée
/private/tmp/b.diihfu6x, BRIDGET_HOME privé, configuration Git système/globale
ignorée et six variables Git de fixture présentes. `--test-threads=4`.
La liste exacte des commandes et l’environnement sont dans le JSON canonique.

Reçu canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/baseline154-results.json
Journal canonique : /Users/moi/.cache/bridget-delivery154.l50umk90/baseline154.log
SHA256 journal : `abae97aff74b5c474a874b58fc48a74d3b0d7e64ca3767ba175fe22402066641`.
Archive : /Users/moi/.cache/bridget-delivery154.l50umk90

## Premier passage R0 conservé séparément

R0 :126,03s, RC0, mêmes totaux193PASS/0FAIL/1ignoré. La recette de ce
premier lancement omettait les variables Git de fixture. R0 ne vaut donc pas
preuve de la recette canonique. Le principal a corrigé l’environnement et
rejoué les mêmes binaires, sans Cargo ni compilation supplémentaire.
R0 JSON : /Users/moi/.cache/bridget-delivery154.l50umk90/baseline154-r0-results.json
R0 journal : /Users/moi/.cache/bridget-delivery154.l50umk90/baseline154-r0.log

## Self-review et portée

Contrôles documentaires : totaux recomptés depuis les résultats, sixfiltres
nonvides, 80retours de binaires à0, sources avant/après identiques,
indicateur de stabilité des binaires, recette canonique complète, hash du
journal recalculé, copieJSON du worktree identique à l’archive.
T001 acquise. T002 acquise séparément par cinqcréations privéesRC0 et mesure
contrôlée. Seules T001 et T002 sont cochées. Cela ne vaut pas suite workspace
complète ni validation de l’implémentation154. Aucun test154 exécuté ici.
