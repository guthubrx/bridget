# Archives techniques de l’adaptateur T3

Document d’origine : [specs/141-messages-groupes/adversarial-review-local.md](../../../../../../specs/141-messages-groupes/adversarial-review-local.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Remarque | Preuve source | Décision et preuve provisoire |
|---|---|---|
| La copie actuelle transforme des références de contexte en libellés si aucun contexte structuré n'est fourni. | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2269 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/packages/shared/src/composerContextReferences.ts:102 | Fournir row.message.text original au bouton existant pour les seuls lots directs, valides ou en repli sûr. Plan, contrat et T004/T005 mis à jour. Cas avec [x](t3-context://v1/skill/ctx_1) observés RED puis inclus dans le passage GREEN provisoire. Les autres familles restent inchangées. |
| Un UUID seul ne doit pas devenir un nom « Bridget » inventé dans une section. | Règle de libellé direct existante en /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:105 | Conserver l'UUID fourni visible. FR14104, modèle, plan et contrat alignés. Deux cas de libellé vide après nettoyage ont été observés RED puis GREEN avec une garde de conservation du libellé reçu. |

<a id="extrait-2"></a>

## Extrait 2

Le fournisseur reste Codex pour la revue et l'implémentation. La limite de revue entre fournisseurs demeure. Le clic natif Copy message a été vérifié ensuite avec interception locale restaurée : 3421 caractères identiques à la fixture SQLite, presse-papiers système préservé. Une copie native de fixture t3-context et une session fournisseur de production ne sont pas revendiquées comme testées. Converge passe 2 conclut CONVERGED, couverture 22/22 et fichier des tâches identique avant/après. Le statut documentaire reste In Progress jusqu'au verdict d'audit final et à T008.
