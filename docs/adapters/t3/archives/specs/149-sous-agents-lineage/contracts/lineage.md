# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/contracts/lineage.md](../../../../../../../specs/149-sous-agents-lineage/contracts/lineage.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Bridget crée, exécute, reprend et annule les tâches. T3 projette leurs faits.
Chaque tâche conserve son identifiant natif et sa corrélation148. Une lecture,
une souscription ou une projection n'exécute jamais une tâche.

<a id="extrait-2"></a>

## Extrait 2

La requête humaine porte `t3_thread_id` et `project_root`. Le daemon résout le
projet canonique hors verrou, puis applique les deux gardes147 sous verrou.
La connexion doit être un client humain négocié. Le binding primaire T3 doit
être vivant, unique, local et propriétaire du même projet. Un client auxiliaire
ou un AgentId fourni par l'UI ne remplace jamais cette autorité.
Le fil T3 est opaque, borné à2048octets, non vide et sans caractères de contrôle.
La garde d'identité stable reste celle148; elle ne réinterprète pas le texte comme UUID obligatoire.

<a id="extrait-3"></a>

## Extrait 3

Le root owner natif vient de ce binding. La tâche demandée doit avoir le même
`root_owner_agent_id`. La profondeur maximale reste8. Une identité d'enfant
connue ou un chemin de journal ne donne aucun droit de lecture.
Révoquer le binding ou perdre le projet ferme les accès courants. Les fils T3
déjà projetés restent historiques et portent un état de disponibilité séparé.

<a id="extrait-4"></a>

## Extrait 4

```text
bridget lineage inspect --t3-thread ID --project-root ROOT --action list --limit N [--cursor CURSOR] --json
bridget lineage inspect --t3-thread ID --project-root ROOT --action show --task UUID [--offset N] [--limit N] --json
bridget lineage inspect --t3-thread ID --project-root ROOT --action journal --task UUID [--after-seq N] [--limit N] [--follow] --json
bridget lineage watch --t3-thread ID --project-root ROOT --json
bridget lineage cancel --t3-thread ID --project-root ROOT --task UUID --request-id UUID --json
```

<a id="extrait-5"></a>

## Extrait 5

Une première page fixe S. Chaque page suivante vérifie la même génération et
séquence. Une mutation entre pages rend `snapshot_changed`; le client abandonne
le staging et reprend à la première page. T3 publie toutes les pages en une
transaction de projection. Aucun snapshot partiel n'est visible.
La limite totale reste4096tâches. Pas de snapshot durable dupliqué ni cache de corps.

<a id="extrait-6"></a>

## Extrait 6

Avec `--follow`, le lecteur réutilise le relai attach existant : rejeu borné,
frontière caught-up, puis événements live et lacunes. L'autorité de tâche est
recontrôlée avant ouverture et avant chaque publication. Une perte de binding
ferme le flux. T3 n'interroge pas périodiquement les corps. Le suivi se ferme
quand le fil enfant n'est plus affiché ou quand la page est masquée.

<a id="extrait-7"></a>

## Extrait 7

Après garde, le daemon appelle l'annulation native existante de tâche et de
descendance. Une tâche d'un autre root rend task_unavailable. Une tâche déjà
terminale rend son état courant. Aucun nouveau tour T3 ni contrôle fournisseur
T3 n'est créé. Arrêter le parent utilise ce même moteur pour ses descendants.

<a id="extrait-8"></a>

## Extrait 8

T3 utilise les fils/relations/sous-agents existants. Identifiants déterministes :
`thread:bridget-task:<task_id>` et `node:bridget-task:<task_id>`.
Origine : `bridget_native`. Relation : `subagent`. Parent d'une tâche racine :
fil T3 attesté qui l'a demandée. Parent d'une tâche imbriquée : fil projeté parent_task_id.
Une racine T3 déjà forkée garde ses relations existantes; le binding désigne son
fil courant, pas une racine de lineage reconstruite comme autorité.

<a id="extrait-9"></a>

## Extrait 9

Le fil projeté porte exactement le marqueur T3 suivant, en camelCase :

<a id="extrait-10"></a>

## Extrait 10

```json
{
  "version": 1,
  "taskId": "UUID",
  "rootThreadId": "fil T3 attesté",
  "parentTaskId": null,
  "generation": "UUID",
  "seq": 42,
  "status": "working"
}
```

<a id="extrait-11"></a>

## Extrait 11

T3 ne crée aucun provider session, turn, lancement, outbox fournisseur ni retour
de résultat orchestré. La commande de projection écrit seulement événements,
projection et reçu existants, atomiquement. Une mise à jour stale est ignorée
par generation/seq. Une nouvelle génération exige un snapshot complet vérifié.
Le fil affiche le statut natif; activeTurnId reste null.

<a id="extrait-12"></a>

## Extrait 12

La pagination indexée vise O(log N + P), P<=100. La réconciliation T3 vise O(N)
avec Maps, N<=4096. Aucun parcours ou requête par enfant dans un hot path.
