# Contre-revue du code160 — Claude R1/R2

Clôture documentaire le 2026-10-12T21:57:42.811733+00:00.
R1 : **APPROVE_WITH_CHANGES**. R2 : **APPROVE_WITH_CHANGES**, aucune objection bloquante retenue après décisions ROOT.
Deux rondes closes par ROOT ; aucun R3 ni test supplémentaire demandé. Ce texte ne signifie ni37/37, ni FULL vert, ni livraison/runtime160.
R1 est archivée dans /Users/moi/.cache/bridget-review160-preparation-20261012/ROOT-review-r1.json. Le verdict R2 et ses limites sont reçus du message ROOT ; sa réponse brute n’était pas présente dans le cache assigné au rédacteur. Ne pas prétendre une lecture du reçu brut R2.

## Quatre objections et disposition ROOT

| Point | Disposition et preuves réelles transmises par ROOT |
|---|---|
| Quarantaine dispatching/ACK tardif | La quarantaine sur échec ou annulation réalise FR011b/FR017, distincte du solde ACK normal. Task.failed ne réémet pas ; indeterminate est absorbant. L’UPDATE ACK ne vise que dispatching : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/idempotency/send_delivery.rs:1185–1196 et1411. Un ancien snapshot/génération est StaleIgnored : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs:2918–2948. Les effets déjà engagés peuvent continuer après l’échec du parent ; aucune unicité des effets externes n’est garantie. |
| defer Err(String) | Classe Storage conservatrice. begin_send_delivery grave dispatching AVANT defer ; le retry lit dispatching sans réémettre, puis échéance/reconnexion clôture. Pas de cohorte Prepared ambiguë : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon.rs:6475–6637. Ordinary conserve le comportement antérieur : appel6437, Err journalisée puis fin ; même mutation et absence de nouvelle remise que la base6490. |
| resumeIDs incompatibles | Corruption étrangère refusée pour éviter de clôturer une autre feuille. Le checkpoint peut rester bloqué indéfiniment pour cette Task ; aucune attente ni blocage du tick global : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs:3060–3143. Limite explicitée dans l’ADR ; pas nouveau moteur de réparation. |
| Réponse completed vide/non vide | Les prédicats sont mutuellement exclusifs ; la capture des métadonnées précède le terminal et son retrait de binding : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/wrapper.rs:6687–6745. Aucun double succès/refus inventé. |

## Delta REL001 et preuve de corrélation partielle

Le delta FR006b ferme la corrélation partielle dans le code final. L’oracle prescrit T033/T035 existe déjà sous le nom `native160_rouge_refus_ancien_et_autres_causes` : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs:6253, branches6260/6302/6304 et assertions6330 (Task byte-identique, suivi conservé et zéro avis pour refus ancien).
L’index de la revue ne l’avait pas retrouvé ; cette absence d’index n’est pas une absence de test. Le delta borné et le replay de l’oracle existant sont distincts des tests antérieurs : aucun nouveau nom ni scénario de confort.
Reçu ROOT : /Users/moi/.cache/bridget-tests160-20261012/T035-partial-correlation-prescribed-table-exact-replay.json, **1PASS/0FAIL/RC0,15.09s** ; log SHA `d019e1c93417fbc53e7d73cccc0c54a192afdf6a74762db8a605e20789988797`. Pas d’exécution par le rédacteur.
Capture finale : /Users/moi/.cache/bridget-review160-preparation-20261012/scope-files-final.json, SHA `5c27880cd8d859481ff74af3c949a7c1163c2ef37f8289f9c9f92689465d73fc` ;19sources, N SHA `c7b7583eb6e40ef99e392b15eacc9bc3b6942e7b890fe75a4cdb7344cec6b174`. Les SHA de cette capture proviennent du manifeste ROOT, sans audit autonome de tous les19fichiers par ce rédacteur.

## Limites et preuves distinctes

- L’ACK tardif après indeterminate est déjà couvert par `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/idempotency/tests.rs:2080` (`accuse_sans_enveloppe_respecte_les_trois_verdicts_metier`), assertions2148–2166 : ACK donne InvalidDelivery et phase inchangée. Cette preuve source est reçue de ROOT ; son exécution appartient au FULL à venir, pas à cette revue. Aucun test160 spécifique lateACK après failed revendiqué.
- Le settlement vérifie l’intent à /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs:3260 ; capture_reply absorbe Task.failed à2836–2868. Ce sont des lectures ROOT, pas une reproduction d’effets externes arrêtés.
- R2 n’a pas lu Spec/Plan/Tasks/execution_store/provider_limits. Ne pas présenter ces deux revues comme un audit complet du dépôt ou des19sources.
- Quota réel non vu. T030 FULL et T031 diff complet restent des gates séparées ; aucun GLOBALGREEN ni livraison déduit des replays ciblés.
- FR003 est une corrélation temporelle non causale. Les effets externes peuvent se répéter ou continuer après parent.failed. L’opt-in, les bornes, l’identité et l’ACK ne prouvent pas exactly-once métier.
- Aucun changement de production, nouveau test, outil, gate ou service par le rédacteur ; seul ce candidat documentaire CACHE est préparé. ROOT applique et archive les preuves.
