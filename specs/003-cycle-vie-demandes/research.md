# Recherche : cycle de vie et annulation

## D1 — Une demande suivie est distincte d'une notification

**Décision** : seuls les messages qui attendent une réponse créent une demande persistée. Les notifications restent sans état.

**Justification** : A2A distingue les messages immédiats des tâches à cycle de vie; une tâche atteint un état terminal tel que `completed`, `canceled`, `rejected` ou `failed`. Cette distinction évite d'imposer un suivi à chaque message.

**Sources** : https://a2a-protocol.org/v0.3.0/topics/life-of-a-task/ et https://a2a-protocol.org/dev/specification/

## D2 — L'annulation est idempotente et terminale

**Décision** : annuler deux fois retourne le même état `cancelled`; une demande annulée ne peut pas être rouverte par une réponse tardive.

**Justification** : A2A précise que les opérations d'annulation sont idempotentes et qu'une tâche terminale ne redémarre pas. Ce comportement rend sûre une annulation répétée après reconnexion ou retransmission.

**Sources** : https://a2a-protocol.org/dev/specification/ et https://a2a-protocol.org/latest/whats-new-v1/

## D3 — L'annulation Bridget est coopérative

**Décision** : l'annulation met fin à l'obligation de répondre, aux rappels et aux timeouts Bridget. Elle informe le destinataire, sans promettre d'interrompre son processus IA ou ses outils en cours.

**Justification** : les runtimes qui interrompent réellement une tâche doivent contrôler l'exécuteur et son mécanisme d'annulation. Bridget route des CLIs hétérogènes et ne possède pas ce contrôle.

**Source** : https://a2a-protocol.org/latest/sdk/python/api/a2a.server.agent_execution.agent_executor.html

## Alternatives rejetées

- **Simple commande d'annulation en mémoire** : rejetée, car un redémarrage ferait perdre l'état et pourrait réactiver des relances.
- **Conserver la détection de réponse par couple d'agents** : rejetée, car deux demandes parallèles entre les mêmes agents deviennent ambiguës.
- **Modéliser tous les messages comme tâches** : rejetée, car cela ajoute un coût cognitif et un état sans valeur pour les notifications ordinaires.
