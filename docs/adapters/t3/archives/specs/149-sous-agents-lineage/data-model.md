# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/data-model.md](../../../../../../specs/149-sous-agents-lineage/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Statut : proposé pour gate GLM. Le magasin SQLite natif reste source unique.
T3 conserve une projection de présentation, sans exécution parallèle.

<a id="extrait-2"></a>

## Extrait 2

La transaction de projection reçoit le snapshot complet vérifié. Elle prépare
les liens depuis parent_task_id, écrit les états observés, puis conserve son reçu
idempotent. Aucun provider session, turn ou effet fournisseur n'est ajouté.
La disponibilité du connecteur est séparée de l'état natif historique.
Le résultat n'est jamais re-routé au parent par T3. La remise native148 reste unique.

<a id="extrait-3"></a>

## Extrait 3

queued → starting → mission_pending → working → waiting_for_children → result_available.
Un échec prouvé donne failed. Une annulation donne cancelling puis cancelled
après arrêt prouvé. Le résultat capturé attend les descendants. Une sortie de
processus ou une fin de tour seule ne prouve pas result_available.
Les dates et la séquence suivent ces transitions natives, pas un tour T3 fictif.
