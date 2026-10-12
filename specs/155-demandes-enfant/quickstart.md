# Vérification rapide - Session155

Toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant`.

## Environnement de test (obligatoire)

```sh
umask 077
R=$(mktemp -d /tmp/b.XXXX); mkdir -m 700 $R/ns
env -i HOME=$HOME TMPDIR=$R BRIDGET_HOME=$R/ns \
  PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  perl -e 'alarm 1800; exec @ARGV' -- \
  cargo test -p bridget-daemon --features test-support <filtre> -- --test-threads=4
```

Pour le transport : même environnement, `cargo test -p bridget-transport <filtre>`.
Suite complète avant clôture : même environnement,
`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast`.

## Scénarios à observer

1. Verdicts et limite absolue (table) : filtre `native155_rights`.
0. Correctif deny (repli fermé) : filtres `approval_response`, `native155_deny`.
2. Écritures confinées refusées : filtre `native155_files`.
3. Admission (`child_requests`, relecteurs refusés) : filtres `native155_admission`, `native155_policy`.
4. Transport Codex, faux serveur, non-blocage, repli fermé : filtre `native155_transport` (crate transport).
5. Wrapper, session et tour : filtre `native155_wrapper`.
6. Réception, liaison d'instance, doublon, budget, avis : filtre `native155_request`.
7. Réponse du parent aux questions, liste d'outils : filtre `native155_respond`.
8. Boîte humaine, décision tardive, 250 décisions obsolètes : filtre `native155_human`.
9. Délai, annulation, pause, redémarrage, accord puis annulation : filtre `native155_lifecycle`.
10. Chaîne de trois niveaux : filtre `native155_chain`. Texte hostile : filtre `native155_hostile`.
11. Rétention avec la purge 150 : filtre `native155_purge`.
12. Bout en bout avec faux serveur : `native155_e2e` dans `native_delegation_e2e.rs`.
13. Non-régression : filtres `native148`, `native149`, `native150`, `delegation149`,
    `spec_087`, `elicitation`, `approval_response`, et le test `claude_native_permissions_test`.

## Recette manuelle (T040, avec l'utilisateur)

Exige la validation explicite de l’utilisateur avant de cocher T040.
Sans relancer de service. Utiliser un daemon de test isolé (`BRIDGET_HOME` privé).

1. Déléguer à un enfant Codex avec `child_requests: "forward"` et une tâche qui pose
   une question.
2. Lire `bridget_task_status` : une demande `open` avec une échéance.
3. Répondre avec `bridget_task_respond` : l'enfant reprend avec la réponse.
4. Refaire avec une tâche qui demande une commande alors que le parent a le plein accès :
   l'item arrive dans la boîte humaine avec les faits vérifiés d'abord ;
   `bridget_task_respond` rend `parent_cannot_approve`. L'humain accepte : la commande s'exécute.
5. Refaire avec une écriture alors que le parent est confiné (`workspaceWrite`), ou un
   accès réseau que le parent n'a pas : refus automatique, aucun item.
6. Ne pas répondre : après 600 s (question), l'issue est `expired` et l'enfant a reçu un refus.
6. Consigner le résultat dans `specs/155-demandes-enfant/validation/recette.md`.

## Gate fournisseur avant T003

Version courante :0.162.1 ; 0.161.0 était l’hypothèse historique. Schéma primaire
généré par root dans /Users/moi/.cache/bridget-delivery155.um4xccv4/codex-schema155-0.162.1.
T001 acquise 180 PASS/0 FAIL ; T002 final PASS attestée dans validation/gate-schema155.json ; protocole/store en validation.
La recette n’exige aucune seconde génération ni test supplémentaire.

fileChange réelle est refusée en v1 (approval_method_unsupported), grantRoot
non nul duration_unsupported. Le cas classifier synthétique DF→Human n’est pas
une preuve de route Human fileChange. permissions reçoit une erreur JSON-RPC,
jamais decision:decline. user_input est adapté en answers{qid:{answers:[string]}} ;
refus/expiration utilisent child_request_refused, faute de decline/cancel natif.
Élicitation MCP tierce : refus v1 approval_method_unsupported pour tout formulaire, même classique/typé/non vide ; URL reste url_mode_unsupported. user_input seul est routable Parent. Exception vide Bridget distincte.
La recette commande demande kind=command, offre explicite et droits compatibles ;
approval_policy=never ne crée aucune demande ou route humaine. T040 reste
conditionnée à la validation explicite utilisateur.
