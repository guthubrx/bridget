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

