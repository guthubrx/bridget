# Contrat149 — descendance native et Lineage

Statut : contrat proposé pour gate GLM. Version native :1. Date :2026-10-10.

## Autorité et invariants

Bridget crée, exécute, reprend et annule les tâches. T3 projette leurs faits.
Chaque tâche conserve son identifiant natif et sa corrélation148. Une lecture,
une souscription ou une projection n'exécute jamais une tâche.

La requête humaine porte `t3_thread_id` et `project_root`. Le daemon résout le
projet canonique hors verrou, puis applique les deux gardes147 sous verrou.
La connexion doit être un client humain négocié. Le binding primaire T3 doit
être vivant, unique, local et propriétaire du même projet. Un client auxiliaire
ou un AgentId fourni par l'UI ne remplace jamais cette autorité.
Le fil T3 est opaque, borné à2048octets, non vide et sans caractères de contrôle.
La garde d'identité stable reste celle148; elle ne réinterprète pas le texte comme UUID obligatoire.

Le root owner natif vient de ce binding. La tâche demandée doit avoir le même
`root_owner_agent_id`. La profondeur maximale reste8. Une identité d'enfant
connue ou un chemin de journal ne donne aucun droit de lecture.
Révoquer le binding ou perdre le projet ferme les accès courants. Les fils T3
déjà projetés restent historiques et portent un état de disponibilité séparé.

## Commandes CLI fermées

Toutes les commandes emploient le namespace courant et le binaire sélectionné147.
Les options inconnues, dupliquées ou incompatibles avec l'action sont refusées.
Les chemins ROOT sont absolus et canoniques après résolution.

```text
bridget lineage inspect --t3-thread ID --project-root ROOT --action list --limit N [--cursor CURSOR] --json
bridget lineage inspect --t3-thread ID --project-root ROOT --action show --task UUID [--offset N] [--limit N] --json
bridget lineage inspect --t3-thread ID --project-root ROOT --action journal --task UUID [--after-seq N] [--limit N] [--follow] --json
bridget lineage watch --t3-thread ID --project-root ROOT --json
bridget lineage cancel --t3-thread ID --project-root ROOT --task UUID --request-id UUID --json
```

`task` et `request-id` sont des UUID canoniques. `limit` est optionnel. Pour list
et journal, défaut50 et intervalle1..100. Pour show, il borne les octets du
résultat, défaut/max16384. `offset` est positif ou nul et tombe sur une frontière
UTF8. `after-seq` est positif ou nul. Le client humain ne peut fournir owner,
child, path, posture, modèle, provider session ou permissions.

Capacités de transport dédiées : `human_lineage_view_v1` pour inspect,
`human_lineage_watch_v1` pour watch et `human_lineage_cancel_v1` pour cancel.
Elles réutilisent la négociation et les gardes147. Pas d'outil MCP d'agent nouveau.
Les quatre outils natifs148 restent le chemin de l'agent propriétaire.

## Snapshot de liste

Réponse version1, maximum100tâches et128KiB JSON encodé par page :

```json
{
  "version": 1,
  "status": "ok",
  "generation": "UUID canonique du magasin",
  "seq": 42,
  "root_owner_agent_id": "UUID Bridget dérivé du binding",
  "tasks": [],
  "next_cursor": null
}
```

La séquence est un entier0..9007199254740991. La génération est persistée avec
le magasin. Les mutations visibles incrémentent la séquence dans la transaction
de tâche. Le suivi ne conserve pas un journal parallèle des mutations.

Chaque entrée contient exactement :

```json
{
  "task_id": "UUID",
  "parent_task_id": null,
  "parent_agent_id": "UUID",
  "child_agent_id": "UUID",
  "child_instance_id": null,
  "created_at": 0,
  "updated_at": 0,
  "started_at": null,
  "completed_at": null,
  "agent_type": "codex",
  "execution_protocol": "codex_app_server",
  "model": "valeur exacte du registre",
  "effort": null,
  "cwd": "/chemin/absolu",
  "posture": "development",
  "title": "consigne bornée",
  "status": "queued",
  "error": null,
  "result_available": false,
  "journal_available": false
}
```

Les horodatages sont des secondes Unix. `title` est la première ligne utile de
la consigne, au plus256caractères. `error` conserve le code natif borné à1024caractères.
La liste ne contient ni consigne complète, ni résultat, ni journal, ni credential.
Les modèles, efforts et protocoles sont ceux de la définition figée.
`posture` est effective, discovery ou development. `inherit` décrit la demande,
pas une permission effective ni une nouvelle posture globale.

États natifs : queued, starting, mission_pending, working, waiting_for_children,
cancelling, result_available, failed et cancelled. Un état inconnu ferme le contrat.
`result_available` n'est vrai que pour l'état natif éponyme. Une réponse capturée
qui attend ses descendants reste invisible comme résultat final.

Ordre stable : `(created_at,task_id)`. Le curseur opaque, maximum2048octets,
encode root/génération/seq/dernière clé. Aucun identifiant du curseur n'est une
autorisation. Le daemon recontrôle le binding et l'appartenance à chaque page.
Le nombre de lignes peut être réduit pour respecter la borne d'octets.

Une première page fixe S. Chaque page suivante vérifie la même génération et
séquence. Une mutation entre pages rend `snapshot_changed`; le client abandonne
le staging et reprend à la première page. T3 publie toutes les pages en une
transaction de projection. Aucun snapshot partiel n'est visible.
La limite totale reste4096tâches. Pas de snapshot durable dupliqué ni cache de corps.

## Détail et résultat

Show applique la même autorité et rend la même métadonnée de tâche, plus :
`result`, `result_offset`, `result_next_offset` et `result_total_bytes`.
Le résultat est null avant publication native. Après publication, il est lu par
fenêtres UTF8 bornées à16KiB. Le dernier curseur vaut null. Le résultat original
reste inchangé et limité à256KiB par148. Une page hors bornes est refusée.
Une lecture ne modifie pas result_sent, cleanup_done, mission ou owner.

## Journal durable et suivi sélectionné

Le daemon résout task→child_agent_id→journal natif, sous l'autorité du root.
Il ne reçoit et ne retourne aucun chemin de journal libre. Le magasin journal
existant reste source unique. Le nettoyage du processus ne supprime pas ce journal.
La reprise conserve les journaux déjà liés à l'identité enfant et leurs frontières
de session; elle n'assemble pas un succès fictif à partir de logs de processus.

Une lecture journal rend au plus100événements et16KiB de contenu, avec version1,
task_id, événements structurés du journal existant, next_seq et caught_up.
La frontière de sortie est celle du journal réellement durable. Une lacune
reste explicite. Les protections de rendu et de masquage existantes s'appliquent.
Un journal absent rend `journal_unavailable`, sans inventer de lignes.

Avec `--follow`, le lecteur réutilise le relai attach existant : rejeu borné,
frontière caught-up, puis événements live et lacunes. L'autorité de tâche est
recontrôlée avant ouverture et avant chaque publication. Une perte de binding
ferme le flux. T3 n'interroge pas périodiquement les corps. Le suivi se ferme
quand le fil enfant n'est plus affiché ou quand la page est masquée.

## Invalidation des tâches

Watch réutilise le contrat147 : ready, changed, resync et error. Ready seq0 est
toujours premier et non coalescible. Les autres événements portent seulement
version1, generation, seq et status. Aucun UUID de fil/tâche, corps ou texte libre.
Une fermeture au milieu du suivi exige un nouveau snapshot à la reconnexion.

Le domaine de séquence est celui du magasin de projection des tâches natives
(`native_delegation_projection_meta`). Snapshot de liste, show et watch portent
la même séquence de mutation, dans le même magasin. La génération est persistée
avec cette ligne et reste compatible du format147. Aucun flux ne croise le
domaine du watch humain147, qui porte le magasin de vue de fil. Ready seq0 reste
un marqueur d'ouverture, pas la dernière séquence du magasin.

Le signal est émis après commit d'une mutation visible dans le root surveillé.
Les changements d'autres roots ne produisent pas de signal pour cette vue.
Rejeu, lecture, refus, ACK et simple changement de cleanup interne ne signalent
pas une nouvelle mutation de présentation. Un changement de journal utilise son
propre relai sélectionné. Il ne transforme pas chaque fragment en mutation de tâche.

Le suivi est borné comme147, avec ressources par connexion, idleTtlMs0 et fermeture
à la disparition du contexte. Les signaux peuvent coalescer après ready. Une
lacune exige resync. Le compteur ne déclenche jamais modèle, mission ou réveil agent.

## Annulation humaine

Cancel conserve une clé idempotente liée au root owner stable et request_id.
L'enveloppe contient task_id. Un rejeu identique retourne le reçu natif stable;
une enveloppe différente donne envelope_mismatch. Le reçu contient version1,
task_id et status observé. Il n'annonce jamais cancelled avant la fin prouvée.

Après garde, le daemon appelle l'annulation native existante de tâche et de
descendance. Une tâche d'un autre root rend task_unavailable. Une tâche déjà
terminale rend son état courant. Aucun nouveau tour T3 ni contrôle fournisseur
T3 n'est créé. Arrêter le parent utilise ce même moteur pour ses descendants.

## Projection T3 et boucle de pont

T3 utilise les fils/relations/sous-agents existants. Identifiants déterministes :
`thread:bridget-task:<task_id>` et `node:bridget-task:<task_id>`.
Origine : `bridget_native`. Relation : `subagent`. Parent d'une tâche racine :
fil T3 attesté qui l'a demandée. Parent d'une tâche imbriquée : fil projeté parent_task_id.
Une racine T3 déjà forkée garde ses relations existantes; le binding désigne son
fil courant, pas une racine de lineage reconstruite comme autorité.

Le fil projeté porte exactement le marqueur T3 suivant, en camelCase :

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

Ce marqueur est le champ facultatif `bridgetTaskRef`. Les données historiques
sans ce champ gardent leur comportement. Le connecteur Rust valide le marqueur
puis exclut le fil de tout montage wrapper/presence/binding Bridget. Un marqueur
présent mais invalide est fermé, sans repli en fil ordinaire. Un simple préfixe
d'identifiant n'est pas suffisant comme autorité.

T3 ne crée aucun provider session, turn, lancement, outbox fournisseur ni retour
de résultat orchestré. La commande de projection écrit seulement événements,
projection et reçu existants, atomiquement. Une mise à jour stale est ignorée
par generation/seq. Une nouvelle génération exige un snapshot complet vérifié.
Le fil affiche le statut natif; activeTurnId reste null.

Lineage ouvre le fil enfant et son journal. Les descendants sont masqués dans
la colonne de gauche par la relation subagent existante. Le composeur est en
lecture seule. Le serveur refuse lancement, message.dispatch, reprise fournisseur,
changement fournisseur, fork et rollback sur ces fils. L'arrêt appelle Bridget.
Un indisponible garde l'historique vérifié sans le présenter comme état courant.

## Refus fermés

unsupported_version, invalid_request, binding_unavailable, project_mismatch,
task_unavailable, snapshot_changed, journal_unavailable, result_offset_invalid,
envelope_mismatch, resource_limit et store_unavailable sont des codes fermés.
Un refus d'autorité ne révèle pas l'existence d'une tâche étrangère.
La disponibilité technique peut donner un retry borné. Un refus métier ne
déclenche ni repli d'identité, ni permission humaine supplémentaire, ni retry infini.

La pagination indexée vise O(log N + P), P<=100. La réconciliation T3 vise O(N)
avec Maps, N<=4096. Aucun parcours ou requête par enfant dans un hot path.
