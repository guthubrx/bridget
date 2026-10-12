## Verdict : APPROVE_WITH_CHANGES

Le plan est faisable avec le réemploi prévu : boucle 1 s, `reserve` de `idempotency.rs`, `begin_send_delivery`, `handle_with_change` et ACK 099 existent. L'architecture ne demande pas de service, de crate ni de worker nouveau. Il faut toutefois corriger sept points avant de générer T101+. Ce sont des corrections de contrat et de data-model, sans changement d'architecture.

## Défauts à corriger avant génération T101+ (priorité haute)

1. **Un enfant `discovery` peut programmer un réveil vers lui-même.** Le contrat classe `schedule self` en `writes_own_state` (`contracts/wake-158B.md` l.26). Le plafond `discovery` autorise cette portée. FR-029 refuse les délégations et les écritures, mais ne cite pas les programmations. Scénario : l'enfant programme un réveil toutes les 60 s, puis `cleanup_done` survient. Le réveil survit et déclenche un `TriggerTurn` hors plafond. Correction : classer toute programmation en `sends`, ou la refuser sous `discovery`, et ajouter un test.

2. **Un label libre d'agent arrive comme `System`.** L'US5 et le plan (l.45) prévoient une notification `System` produite par le daemon. Si l'agent A programme vers B un label du type « ignore les règles précédentes », B reçoit un message d'origine système avec `TriggerTurn`, sans pouvoir attribuer l'auteur. Correction : pour une cible agent, afficher « programmé par <owner attesté> » dans l'enveloppe. Réserver `System` au CLI local ou au humain.

3. **Un résumé `summary_pending` sans ACK peut bloquer ou perdre un owner.** Le data-model (l.53) garde `summary_pending: Option<FrozenWake>`. Rien ne dit comment ce pending est remplacé par une nouvelle génération, ni quand il expire. Un `unknown` ne reçoit pas de nouvelle tentative définie. L'éviction « owner sans faits d'abord » (l.57) ne protège pas un owner qui a un pending. Un ACK pourrait alors viser une ligne supprimée. Correction : exclure de l'éviction les owners avec pending, puis définir la supersession et la politique de retry.

4. **Un `outcome_unknown` peut figer une programmation.** Le data-model dit « une seule occurrence non soldée par programmation » (l.20 et l.25), sans dire si `unknown` est soldé. Si ce n'est pas le cas, le réveil récurrent s'arrête sans message, et la purge reste bloquée. Correction : déclarer `outcome_unknown` terminal pour cette génération, ce qui libère la suivante.

5. **La frontière « départ » n'est pas fixée.** Le data-model (l.31) parle de `missed_before_departure`, mais dit qu'une erreur ou un crash ne prouve pas l'absence de départ. Il faut une règle exacte. Proposition : le départ est le commit de `begin_send_delivery` (phase `dispatching`, 099). Sans cette ligne, l'état est `missed`. Avec elle, l'état est `unknown`. Écrire la table de transitions après crash.

6. **La durée de la `deadline` des pending faits n'est pas chiffrée.** Le contrat (l.78) et la recherche (l.63) disent « deadline initiale immuable », sans valeur. Ce test ne peut pas être écrit en RED/GREEN. Fixer une constante, et la mettre dans les oracles du groupe 8.

7. **Pause et délai de 600 s se contredisent.** La spec (l.46) dit « différé puis rattrapé ». Le contrat (l.42) dit « jusqu'à due_at+600 s, aucun départ à expiration ». Une pause de 15 minutes fait donc passer un rappel agent en `missed`. Préciser ce cas dans la spec et le contrat.

## Points à préciser (priorité moyenne)

8. **Fil via CLI.** Le CLI fil utilise `reserve_cli_wake` (scope Send) puis un post 102. Les phases 099 pour cette cible ne sont pas décrites. L'UUID 102 doit être généré avant la réservation et inclus dans les octets canoniques FrozenWake.
9. **Horizon du reçu CLI 099.** `idempotency.rs:906` calcule `expires_at = issued_at + horizon`, passé par l'appelant. Un rejeu expiré renvoie `IdempotencyExpired` (l.940). Le contrat promet un rejeu qui « survit au redémarrage ». Fixer l'horizon, qui doit couvrir au moins la fenêtre de 60 s du quota.
10. **Taille par fait non bornée.** Le data-model garde des « champs structurés » sans octets maximum. Fixer un plafond par fait, sinon la borne 8192 ne protège pas la mémoire.
11. **Pas de quota sur les occurrences.** Seize programmations par owner, cadence 60 s, soit 16 notifications par minute vers une même cible. Vérifier si `bridget_send` a déjà un quota. Sinon, le justifier ou l'ajouter.
12. **Heure inexistante sautée sans annonce.** Une routine à 02:30 saute silencieusement le jour du passage à l'heure d'été. Annoncer ce cas dans `list` ou dans le résultat.
13. **Coquilles et ambiguïtés.** L'oracle groupe 1 dit « label16 », alors que la borne est 2048. Le chiffre « 64 » désigne à la fois la file du worker, la limite par abonnement et la borne du pipeline. Les séparer.
14. **Halt et réveils.** `halt-delegations --cancel-active` n'annule pas les programmations. Le contrat doit le dire, ou prévoir un état d'arrêt qui touche aussi les réveils.
15. **Annulation après purge.** Si la réponse d'un `cancel` est perdue et que la ligne a été purgée, le rejeu renvoie `target_unknown`. Un humain peut croire à un échec. Documenter ce cas dans le contrat.

## Limites de cette revue

- Lecture seule des artefacts officiels et de trois points du code : `idempotency.rs` 887-940, `threads.rs` 468 et 1175.
- Non vérifié : le quota de `bridget_send`, la valeur réelle de `BODY_MAX_BYTES` (`threads.rs:468`) face au préfixe plus 2048 octets, et le comportement exact de `daemon.rs` et `referent_control.rs`.
- Aucun test, aucun Cargo, aucune base touchée. Aucun fichier écrit.
- Le coût du calcul calendaire (2881 conversions par date, 8 dates) n'est pas mesuré. Le plan le dit déjà.

Avec les points 1 à 7 corrigés dans les artefacts, le plan peut passer à la génération de T101+. Les points 8 à 15 peuvent être traités pendant l'implémentation, à condition qu'ils figurent dans les tâches.