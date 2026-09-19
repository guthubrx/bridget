# ADR 003 — Livrer les messages équipiers via ACP

**Date** : 2026-08-22  
**Statut** : Accepté

## Contexte

Le transport historique livre un message Bridget en simulant une saisie dans un
terminal tmux. La réponse dépend alors d'instructions persistantes dans le
prompt de l'agent et d'une commande shell exécutée par celui-ci. Ce mécanisme
altère les messages contenant des caractères spéciaux, ne permet pas de suivre
l'état d'un tour de travail et ne fonctionne pas pour un agent headless sans
terminal.

La recherche R-001 établit que l'Agent Client Protocol (ACP) est un protocole
JSON-RPC sur stdio dont le cycle `initialize`, `session/new`,
`session/prompt`, notifications `session/update` et réponse finale avec
`stopReason` correspond directement au cycle de livraison Bridget. La recherche
R-004 compare l'implémentation officielle Rust d'ACP à l'architecture actuelle
de Bridget, synchrone et fondée sur des threads standards.

## Décision

Bridget ajoute ACP comme second transport pour les équipiers headless. Le
transport tmux existant reste le repli inchangé pour les agents interactifs.

Le client ACP est un sous-ensemble JSON-RPC bloquant, écrit dans
`bridget-transport` avec les primitives déjà employées par le protocole local :
`BufReader` ligne par ligne, `serde_json`, threads standards et écritures
sérialisées. Il couvre strictement `initialize`, `session/new`,
`session/prompt`, `session/update`, `session/request_permission` et
`session/cancel`.

Le client possède un lecteur unique de stdout, afin de démultiplexer les
réponses, notifications et requêtes de permission sans corruption du flux. Un
équipier conserve une session ACP durable ; les messages Bridget constituent
des tours FIFO successifs.

## Décisions associées

- **D-202 — Registre JSON** : les types d'agents restent déclarés dans
  `agents.json`, lu avec `serde_json` déjà présent. Le registre remplace la
  liste blanche en dur sans nouvelle dépendance.
- **D-205 — Réponse selon `reply`** : une fin de tour normale est routée et
  clôt la demande uniquement pour `reply=yes`. Pour `reply=no`, elle reste dans
  le journal de session sans message de retour.
- **D-206 — Échéances** : le daemon demeure l'autorité unique des délais et
  expirations. Le transport ne gère que le tour actif et retire les messages
  annulés ou expirés avant leur livraison.
- **D-207 — Facturation** : le wrapper refuse un équipier lorsqu'une variable
  d'API interdite est présente, sauf contournement explicite. Pour Codex, les
  deux variables interdites sont `OPENAI_API_KEY` et `CODEX_API_KEY`.
- **D-208 — Clôture fiable** : une demande est marquée répondue seulement après
  livraison effective de la réponse. Le contournement DND exige une validation
  non mutante de `in_reply_to` et de ses deux participants, afin de bloquer les
  identifiants forgés.
- **D-209 — Annulation et rejet typés** : `CancelDelivery` purge une livraison
  identifiée côté wrapper ; `DeliveryRejected` remonte au daemon un rejet
  terminal, notamment pour une file pleine ou un processus arrêté.

## Conséquences positives

- La livraison et le retour de réponse sont structurés, sans injection terminal
  ni échappement shell.
- `stopReason` et les notifications de tour rendent l'activité de l'équipier
  observable et permettent de différer les relances inutiles.
- Le transport reste local au wrapper : la fédération SSH existante ne change
  pas.
- Aucune nouvelle dépendance ni runtime async n'est ajouté au workspace.

## Conséquences négatives

- Le sous-ensemble JSON-RPC est maintenu localement et doit rester conforme aux
  adaptateurs ACP épinglés ; des fixtures couvrent donc les cas de protocole
  significatifs.
- Un agent ACP introduit une file et un état de tour à gérer explicitement.
- La compatibilité dépend des versions d'adaptateur et du coeur Codex embarqué,
  vérifiés par le spike T701 avant l'implémentation.
