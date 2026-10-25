# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/154-isolation-worktree-optionnelle.md](../../../../../decisions/154-isolation-worktree-optionnelle.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Décision | Choix accepté |
|---|---|
| D1 | workspace facultatif sur Delegate ; aucun outil distinct. |
| D2 | Worktree sous cwd/.worktrees/task8 ; sous la racine parent. |
| D3 | Rejeu admis et autorisation du cwd canonique avant les lectures Git de la session 154 ; réservation sous verrou, création hors verrou, revalidation. |
| D4 | Posture development et paire d’héritage prouvée ; refus nommés avant effet. |
| D5 | Exécuteur Git unique ; formes/options en liste blanche, temps et sorties bornés. |
| D6 | Aucune suppression de branche ou de worktree par Bridget, même après échec. |
| D7 | Cleanup après tentative de rapport terminée et arrêt confirmé/fin de garde 153 ; kept ne suffit pas. |
| D8 | Faits Git structurés et suffixe écrit par Bridget ; imitation enfant neutralisée, budget exact et coupe UTF-8. |
| D9 | shared_writers informatif ; aucune nouvelle interdiction du partage. |
| D10 | Refuser filtres exécutables, inclusions, clones partiels et worktreeConfig. |
| D11 | .worktrees déjà ignoré ; plafond de 32 par union de chemins, aucune configuration écrite. |
| D12 | Rapports hors verrou ; CAS état/mission/ronde ; une tentative bornée, même en erreur Git. |
| D13 | Réservations dans Arc<Mutex> séparé ; aucune garde Drop reprenant le verrou global. |
| D14 | Git exécuté par le daemon pour parent autorisé ; modèle T3 durci, risque résiduel accepté. |

<a id="extrait-2"></a>

## Extrait 2

Comparaison déjà recherchée dans T3 : GitVcsDriverCore.ts:3418-3419 et :3789
exécutent worktree add avec les droits du serveur ; GitVcsDriver.ts:430 décrit
le retrait. Cette ADR rapporte cette recherche, sans prétendre avoir retesté T3.
Bridget ajoute gardes Git, liste blanche, bornes et aucune suppression.
