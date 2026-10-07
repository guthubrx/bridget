# Contre-revue locale141

Date : 2026-10-07. Verdict post-correctif du reviewer : APPROVE. Converge passe 2 CONVERGED et audit A/100 validés. Statut : Implemented, non activé.

## Portée et fournisseur

La revue est indépendante du travail documentaire et du code examiné, mais utilise le même fournisseur Codex. L'inventaire Bridget who transmis par le principal ne fournit aucun autre fournisseur joignable. Cette contre-revue est donc locale et dégradée par rapport à une revue entre fournisseurs distincts. Aucune indépendance de fournisseur n'est revendiquée.

La revue porte sur le plan, le contrat de présentation, la copie et les noms des expéditeurs. Elle ne remplace pas la recette du composant réel ni l'audit final.

## Remarques retenues

| Remarque | Preuve source | Décision et preuve provisoire |
|---|---|---|
| La copie actuelle transforme des références de contexte en libellés si aucun contexte structuré n'est fourni. | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2269 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/packages/shared/src/composerContextReferences.ts:102 | Fournir row.message.text original au bouton existant pour les seuls lots directs, valides ou en repli sûr. Plan, contrat et T004/T005 mis à jour. Cas avec [x](t3-context://v1/skill/ctx_1) observés RED puis inclus dans le passage GREEN provisoire. Les autres familles restent inchangées. |
| Un UUID seul ne doit pas devenir un nom « Bridget » inventé dans une section. | Règle de libellé direct existante en /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:105 | Conserver l'UUID fourni visible. FR14104, modèle, plan et contrat alignés. Deux cas de libellé vide après nettoyage ont été observés RED puis GREEN avec une garde de conservation du libellé reçu. |

## Résultats et limites

Dernier passage transmis par le principal : 288 tests PASS (211 logique, 77 rendu). Les corrections documentaires ont fait l'objet d'une nouvelle analyse de cohérence en deux passes : couverture 22/22 et aucune contradiction ouverte.

Le code n'est pas gelé. La recette isolée est en cours. Les contrôles de types, format, lint, build, clavier, thèmes et défilement doivent être liés à leurs résultats effectifs avant le statut final. Aucune tâche d'implémentation n'est cochée à partir de cette revue seule.

## Relecture post-implémentation

La passe 1 de Converge révèle deux pertes de texte dans le code réel : body "A\r" réduit à "A" dans une enveloppe LF, puis copie d'une référence t3-context transformée quand l'en-tête canonique dépasse 1024 caractères et la projection est null. Ces garanties de conservation restent ouvertes. Elles sont détaillées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/converge.md et couvertes par les nouvelles tâches T009/T010.

Le verdict post-implémentation est WITH_CHANGES. Aucun APPROVE final n'est donné avant les corrections RED/GREEN, la nouvelle passe Converge et les contrôles requis. L'audit en lecture seule est en préparation ; son verdict reste suspendu.

Prochaine étape : corriger T009/T010, finir les contrôles et la recette, puis relire le diff final et consigner la nouvelle passe Converge et l'audit. Aucun redémarrage ni installation de production dans cette phase.

## Relecture après correctifs T009/T010

Verdict transmis : APPROVE. Le reviewer a exécuté le vrai code de projection avec A\r/B\r en enveloppes LF et CRLF ; les corps restent intacts. Les cas d'en-tête long et mixte conservent la copie originale. Quatre cas ont suivi RED puis GREEN avant ce verdict. Aucun résidu n'est prouvé dans la relecture ciblée.

Sources gelées à 07:50:41 CEST. Suites finales exécutées par le principal à 07:51:37 : 294 PASS, 215 de logique et 79 de rendu, 2,84 secondes. Format/lint/types/build web PASS ; 22 avertissements historiques, identiques à la baseline. La recette native post-gel est consignée dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/implementation.md.

Le fournisseur reste Codex pour la revue et l'implémentation. La limite de revue entre fournisseurs demeure. Le clic natif Copy message a été vérifié ensuite avec interception locale restaurée : 3421 caractères identiques à la fixture SQLite, presse-papiers système préservé. Une copie native de fixture t3-context et une session fournisseur de production ne sont pas revendiquées comme testées. Converge passe 2 conclut CONVERGED, couverture 22/22 et fichier des tâches identique avant/après. Le statut documentaire reste In Progress jusqu'au verdict d'audit final et à T008.

Clôture : audit A/100 reçu, aucun point critique/majeur ouvert et validateur principal sortie 0, sans erreur ni avertissement. T008 cochée ensuite hors Converge. Dix tâches validées ; session Implemented, non installée. La limite de fournisseur identique demeure explicite.
