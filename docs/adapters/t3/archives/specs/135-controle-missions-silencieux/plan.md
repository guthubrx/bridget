# Archives techniques de l’adaptateur T3

Document d’origine : [specs/135-controle-missions-silencieux/plan.md](../../../../../../specs/135-controle-missions-silencieux/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Conserver `events.jsonl`, les tâches JSON et `heartbeat-state.json`.
- Conserver `ack`, `resolve-task`, `relaunch` et les sessions Bridget.
- Étendre les reçus de continuation au lieu de créer un second journal.
- Conserver le LaunchAgent. Remplacer les deux surcharges Politique par l'appel
  direct au script officiel, qui porte désormais leurs garanties utiles.
- Ne pas modifier Bridget, T3, tmux ou les fichiers métier Politique.
