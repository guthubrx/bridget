# Données149 — permissions et descendance native

Statut : proposé pour gate GLM. Le magasin SQLite natif reste source unique.
T3 conserve une projection de présentation, sans exécution parallèle.

## Tâche native

La ligne native_delegations148 conserve task_id, clé owner/request_id, enveloppe
canonique, définition complète, enfant, mission, résultat et checkpoints148.
Ajouter au payload de tâche :

| Champ | Nature et règle |
|---|---|
| root_owner_agent_id | Identité stable racine dérivée de la connexion ou de la tâche parent prouvée. |
| parent_task_id | UUID de la tâche dont l'enfant est le parent actuel; null pour racine. |
| requested_posture | inherit lorsque le champ MCP est omis; sinon discovery ou development. |
| effective_posture | discovery ou development, calculée à admission. |
| permission_snapshot | Politique fournisseur effective, provenance non secrète, contraintes et définition enfant figées. |
| updated_at | Secondes Unix de la dernière mutation visible. |
| started_at | Secondes Unix de l'admission de la remise148; null avant. |
| completed_at | Secondes Unix de l'état terminal prouvé; null avant. |

Les champs148 owner, owner_instance et origin_owner_instance restent distincts.
Une reprise d'instance ne change ni root_owner_agent_id, ni parent_task_id, ni
permission_snapshot, ni enfant, ni mission, ni modèle. Une attestation fraîche
autorise l'identité de reprise; elle ne remplace pas les droits de la mission.

Le parent_task_id se résout par l'identité enfant native managed prouvée, jamais
par un argument d'agent. Un index unique sur child_agent_id permet cette résolution.
Une tâche racine n'adopte pas arbitrairement une mission en cours comme parent.
L'absence de parent prouvé reste une racine sous son owner attesté.

Indices à réutiliser ou ajouter :

- owner stable/request_id unique, conservé148;
- mission unique, conservé148;
- child_agent_id unique;
- root_owner_agent_id/created_at/task_id pour pagination;
- parent_task_id pour descendance et annulation.

Les limites148 restent16actives par parent,128globales,4096enregistrements,
profondeur8 et résultat256KiB. L'enveloppe canonique garde posture omise comme
demande inherit; elle n'est pas réécrite depuis la politique courante au retry.

## Séquence de projection

Ajouter une seule ligne native_delegation_projection_meta avec generation UUID
canonique et seq entier sûr pour JavaScript. La génération est celle du magasin,
conservée au redémarrage. La séquence commence à0 et ne décroît jamais.
Créer ou modifier un fait visible de tâche incrémente seq dans la même transaction.
Les lectures et les updates strictement internes ne l'incrémentent pas.

La transaction retourne le root et la séquence committés. Le daemon signale
ensuite uniquement les abonnements de ce root. Un échec SQL n'émet aucun signal.
Pas de table d'événements métiers nouvelle, cache de corps ou snapshot persistant
dupliqué. Si la séquence atteint sa borne, renouveler la génération dans la
transaction et imposer resync. Aucun overflow silencieux.

## Snapshot de permissions

Le snapshot contient version1, source factuelle, owner stable et instance
d'admission, session/tour fournisseur si présents, policy fournisseur observée,
frontière logique projet/cwd et exigence de confinement réellement attestée.
Pour la voie Claude de mêmes inputs, il conserve launcher/profil/cwd/sources
sélectionnées, leurs digests et absences, et les overrides sanitaires connus.
Il conserve aussi resolved_cli_path et resolved_cli_revision du binaire réellement
résolu sous l'environnement propriétaire. Le couple launcher et le couple CLI
sont vérifiés ensemble avant effet, sans cache ni adoption d'une nouvelle résolution.
Il ne prétend pas contenir les règles fusionnées que le SDK n'expose pas.
Les inputs sont recontrôlés au lancement/reprise; une divergence rend
settings_revision_changed, sans remplacer le snapshot par le nouveau fichier.
Il ne contient aucun endpoint secret, token, contenu de settings hors permissions,
credentials provider, clé d'API, variable d'environnement complète ou prompt.

Les politiques des anciens enregistrements148 sans snapshot restent lisibles.
Leur définition figée conserve leur exécution148. La migration n'élargit pas
leurs droits. Une nouvelle écriture demandée sans attestation149 rend un refus
de droits; elle ne demande pas un grant humain comme substitut.
Les révocations148 restent durables et prioritaires pour les nouveaux accès.

Le fait d'un parent standalone est éphémère, lié à sa connexion primaire et
à l'instance propriétaire. Une déconnexion invalide ce fait. Une reconnexion
exige une nouvelle observation fournisseur ou la même définition native admise.
Il n'existe pas de politique globale par nom de modèle ou PID partagé.

## Journal

Les journaux append-only natifs existants restent la source de contenu. Le
task_id pointe vers child_agent_id, puis les sessions durables de cet enfant.
Le magasin ne copie pas les journaux ni leurs fragments dans la tâche.
L'arrêt/nettoyage retire les processus et liens runtime, pas les fichiers journal.
La vue humaine contrôlée choisit les fichiers depuis la tâche; le RPC n'accepte
aucun chemin libre. Les lacunes et erreurs de lecture restent explicites.

## Projection T3

Les tables existantes de fils, nœuds et sous-agents conservent les enfants de
présentation. IDs déterministes et origine bridget_native. Le marqueur facultatif
bridgetTaskRef contient version/taskId/rootThreadId/parentTaskId/generation/seq/status.
Il est ajouté aux résumés de fil pour filtrer le réimport vers Bridget.

La transaction de projection reçoit le snapshot complet vérifié. Elle prépare
les liens depuis parent_task_id, écrit les états observés, puis conserve son reçu
idempotent. Aucun provider session, turn ou effet fournisseur n'est ajouté.
La disponibilité du connecteur est séparée de l'état natif historique.
Le résultat n'est jamais re-routé au parent par T3. La remise native148 reste unique.

## Transitions

queued → starting → mission_pending → working → waiting_for_children → result_available.
Un échec prouvé donne failed. Une annulation donne cancelling puis cancelled
après arrêt prouvé. Le résultat capturé attend les descendants. Une sortie de
processus ou une fin de tour seule ne prouve pas result_available.
Les dates et la séquence suivent ces transitions natives, pas un tour T3 fictif.
