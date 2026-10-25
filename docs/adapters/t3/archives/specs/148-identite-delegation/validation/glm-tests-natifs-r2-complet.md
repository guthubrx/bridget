# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-tests-natifs-r2-complet.md](../../../../../../../specs/148-identite-delegation/validation/glm-tests-natifs-r2-complet.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le correctif R1 est vérifié par les assertions mêmes du test (toutes passées) :
- **un seul `started` et un seul `prompt`** avant redémarrage ; l'unique prompt porte `resume_card=true` ET `mission=true` (carte+mission jointes au premier tour du nouvel enfant natif).
- refus sans grant (`delegation_grant_required`), puis grant humain via CLI réel.
- idempotence : 10 replays ⇒ mêmes `task_id`/`child_agent_id`/`message_id`.
- réponse corrélée unique (`replies.len()==1`, `in_reply_to` = message d'origine, corps = réponse attendue).
- SQLite : 1 ligne `native_delegations` ; `t3_present=false` ; aucun répertoire `.t3` chez le fournisseur.
- **recette cancel/restart VRAIMENT atteinte** : mission `WAIT_CANCEL_148` bloquée chez le fournisseur factice, `Cancel` ⇒ `cancelled` avec résultat nul et aucune réponse corrélée ; `StopOrder` du premier enfant ; arrêt puis relance du daemon (2 générations) ; replay de la requête d'origine ⇒ résultat durable identique (`recovered == finished`) ; `cancelled` préservé après restart ; `started == 2` au total. Le test est une fonction unique sans sortie anticipée : son PASS prouve l'exécution de toutes ces assertions.

<a id="extrait-2"></a>

## Extrait 2

`partial_or_invalid_t3_proof_never_falls_back_to_pid` : les 4 cas (endpoint seul, authorization seule, paire invalide, paire vide) passent. Le helper ignoré `isolated_identity_entrypoint` écrit un marqueur PID réellement valide, résout l'identité native par le marqueur, et `bridget_who` répond `isError=true` + `t3_session_unavailable` — refus T3 malgré marqueur PID valide, sans repli PID. 1 ignored volontaire (helper).

<a id="extrait-3"></a>

## Extrait 3

- **Aucun processus résiduel** de mes tests (worktree, daemons de test, fixtures python : rien). Les ~22 processus `bridget` présents (daemon production, `t3 serve`, MCP de sessions) préexistaient à la ronde — inchangés, non touchés.
- 4 répertoires `/tmp` issus de mes 2 exécutions en échec (umask 022) supprimés après inspection : `bg908-e2e-89339-b4e6777b`, `bg907-e2e-89339-64846808`, `bg908-e2e-4907-c9183c55`, `bg907-e2e-5470-925ab367`. Les autres répertoires `/tmp` (runs antérieurs du dev/R1) n'ont pas été touchés.
- Scratch TMPDIR privé nettoyé ; seuls les logs et empreintes sont conservés.
