# SPEC138 — Annonces propriétaires wrapper et T3

Périmètre : wrapper.rs, t3code.rs et t3code_contract.rs uniquement.
Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet.

## Réutilisation

- `connect_and_register_with_domain_at` reste le chemin propriétaire du wrapper.
  Il annonce le cwd constaté avec `source=git` à chaque connexion, y compris
  lorsque le daemon attribue la nouvelle identité. Le domaine reste distinct.
- `connect_and_register_at` n'annonce jamais le cwd du wrapper. T3 réutilise ce
  chemin pour ne pas hériter du projet du processus d'adaptation.
- `parse_snapshot` rattache la racine T3 par `projectId` à un seul projet publié.
  Référence absente ou ambiguë : racine inconnue. Le worktree reste indépendant.
- `LinkWorker::new` et les ticks existants publient les faits T3 via
  `send_wrapper_message`. Chaque reconnexion repart d'une annonce. Un tick
  identique ne republie rien. Un changement est transmis sans décider de sa
  cohérence : seul le daemon détermine si les racines sont contradictoires.
- Les lecteurs traitent `ProjectContextResult` sans injecter son résultat dans
  la conversation, ni consommer une livraison ou un Disconnect en attente.

## Tests préparés avant implémentation

- `spec138_snapshot_uses_project_id_not_worktree_or_title`.
- `spec138_snapshot_missing_or_ambiguous_project_stays_unknown`.
- `spec138_owner_announces_git_cwd_on_each_registration_without_consuming_frames` :
  deux échanges avec un véritable pair Unix, fait Git puis Disconnect conservé.
- `spec138_t3_owner_announces_unknown_project_instead_of_using_adapter_cwd` :
  un socketpair réel, annonce T3 inconnue plutôt que le cwd de l'adaptateur.
- L'assertion du test existant `snapshot_reel_0_0_40_est_lu_et_les_champs_manquants_sont_nommes`
  vérifie également la racine publiée.

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
