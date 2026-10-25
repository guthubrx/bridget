# SPEC138 — Annonces propriétaires wrapper et adaptateur de bureau

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/138-priorite-projet/evidence/transport-facades.md#extrait-1).

## Réutilisation

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/138-priorite-projet/evidence/transport-facades.md#extrait-2).

## Tests préparés avant implémentation

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/138-priorite-projet/evidence/transport-facades.md#extrait-3).

Ces tests ont été écrits avant le comportement nouveau. Leur exécution RED
propre n'a pas été capturée : la compilation simultanée du contrat du noyau
bloquait l'intégration. Le principal a autorisé l'implémentation sans inventer
une preuve RED. La preuve RED des autres contrats de la session reste distincte.

Test complémentaire :
`spec138_t3_reannounces_only_changed_project_facts_and_keeps_verdict_out_of_conversation`
vérifie un fait initial, deux ticks identiques, une contradiction conservée,
une disparition de racine et un résultat sans effet sur la conversation.

## Validation

- rustfmt, seulement les trois fichiers possédés : PASS.
- git diff --check, seulement les trois fichiers possédés : PASS.
- Les cinq nouveaux tests transport `spec138_` : PASS, vérifiés dans
  /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/facades-green-138.log.
  Cette vague commune comporte 20 PASS et 1 FAIL, sur la fixture auxiliaire du
  noyau `spec138_project_announcement_recovers_absence_but_not_conflict_and_aux_inherits`.
  Elle ne constitue donc pas une validation globale. Le propriétaire du noyau
  annonce avoir corrigé cette fixture ensuite ; ce rapport ne revendique pas
  de nouveau passage global à partir de ce seul journal.
  L'absence de preuve RED exécutée pour ces tests reste inchangée.
- Aucun commit, installation, message Bridget ou changement de production.
