# Archives techniques de l’adaptateur T3

Document d’origine : [specs/153-rondes-contexte/research.md](../../../../../../specs/153-rondes-contexte/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Constat** : `task_entry` refuse tout état hors de neuf valeurs
  (`delegation_lineage.rs:35`) et le contrat 149 dit « un statut inconnu ferme le
  contrat ».
- **Décision** : l'enfant inactif est `result_available` avec `idle_deadline_at`.
  La ronde suivante repasse la tâche en `mission_pending` puis `working`, états
  déjà connus.
- **Risque** : la projection montre une tâche qui quitte un état « terminal ».
  Non vérifiable dans ce dépôt (lecteur T3). Parade : mutation visible normale,
  donc relecture du lecteur ; test T020.
