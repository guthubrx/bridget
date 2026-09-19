# Contre-revue adverse — affirmations du 2026-09-19 sur la fluidité des remises

- Demandeur : bdget (Claude). Relecteur : evols-t3 (Codex, fournisseur différent), réponse en 4 minutes.
- Objet : quatre affirmations faites à l'utilisateur après les sessions 110, 111, 112.
- Verdict reçu : **BLOCKED**.

| Objection (evols-t3) | Vérifiée comment | Retenue | Suite |
|---|---|---|---|
| 1. À 64 remises la plus ancienne est écartée sans NACK (`t3code.rs:1399`) | lecture du code ; aucune `DeliveryRejected` dans le pont | oui | le pont doit signaler tout écartement au daemon |
| 2. Regrouper des `reply=true` casse la corrélation « un message = un tour » (`:2554`) | `state.pending` par `request_id`, une réponse finale par tour | oui | lots limités à `reply=false`, ou protocole de réponses structurées |
| 3. `SteerCurrent` existe dans le cœur, pas sur le pont (`:1597`) | aucune occurrence dans `t3code.rs` ; contrat T3 ne connaît que `thread.turn.start` | oui | piste séparée, dépend du support T3 |
| 4. `bridget claude` n'est pas la seule route hors T3 (`registry.rs:322`) | `claude`, `claude-son`, `gclaude` au registre, plus `spawn claude` | oui | ma formulation était fausse |
| 5. Une remise `reply=false` peut attendre des heures puis partir avec un budget complet, contexte obsolète (`:1559`) | par construction de la 112 | partiellement | accepté par design pour les messages sans échéance ; l'expéditeur peut poser `reply_timeout` ; à documenter |

Constat propre ajouté pendant la vérification : deux remises `reply=true` vers `claude-horizon`, demandes
`timed_out`, affichent encore `dispatching` côté daemon alors que le fil est inactif. Le pont purge
silencieusement ces remises (`t3code.rs`, handler `RequestList`, `queue.retain` sans trace) et ne
signale rien au daemon : la saga reste « en vol » jusqu'à expiration. 33 sagas d'août et 3 du jour en
témoignent. « En vol » côté daemon ne prouve donc pas que le pont détient encore la remise.
