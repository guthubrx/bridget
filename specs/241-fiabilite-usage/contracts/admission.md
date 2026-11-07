# Contrat 241 — autorité d'une mission admise

Une nouvelle admission est vérifiée avec la preuve active du demandeur. Une fois
la mission enregistrée, son instantané est son autorité d'exécution jusqu'à sa
fin, son échéance ou sa révocation native. La fin du tour n'est pas une révocation.
Un ancien tour ne permet aucune admission ou continuation nouvelle.

Sources natives de révocation : DelegationStore::revoked_agent pour propriétaire
et propriétaire racine, DelegationStore::revoked pour l'instance, état annulé de
la Task, contrôle d'arrêt/pause natif et autorité du placement. Ces sources
existent dans SQLite et sont relues avant Release ; elles ne viennent pas du
HTTP. Les changements de fichiers de permissions restent contrôlés par
remote_parent_proof et prove_parent_permissions. Toute panne de lecture de ces
sources reste un échec explicite, jamais une autorisation implicite.

Changer le mode des futurs tours d'une conversation n'est pas, à lui seul,
l'annulation rétroactive d'une délégation acceptée. Le retrait du droit de
mission s'effectue par les contrôles natifs précités, comme pour une mission
déjà en cours. Ce choix explicite rapproche les admissions provider_turn du
contrat native_wrapper déjà appliqué par recheck_remote_dispatch.

Le marqueur interne de contrat sera un champ optionnel de Task, absent par
défaut pour les tâches historiques. Il est produit uniquement par l'admission
attestée, jamais par un argument libre. Il ne contient aucun credential et ne
modifie ni canon, ni instantané de droits, ni politique enfant. Toute remise
exige toujours la même tâche, le même propriétaire, le même placement, la même
définition et les vérifications natives de révocation et de sources.

Les anciens lecteurs doivent être qualifiés explicitement : lecture et reprise
de tâches, refus sûr d'un contrat non pris en charge, absence de perte d'état.
Un lecteur qui ignore le marqueur mais échoue ensuite ne constitue pas à lui
seul une preuve de retour fonctionnel. Le retour livré doit traiter ce contrat.

Les contrôles de preuve active restent nécessaires avant l'acceptation et pour
les nouvelles demandes, pas pour réautoriser indéfiniment le même travail.

## Autorité des fichiers source

Les racines explicites du fait fournisseur sont utilisées sans recréer une
autorisation. Un fait Codex avec sandbox dangerFullAccess, ou le cas natif
Claude déjà qualifié sans règles privées, autorise aussi une source locale hors
du cwd initial. Le seul libellé runtime full-access ne suffit pas. Les profils
restreints conservent la liste de leurs racines. Les sources, UID/protections
worker et politiques enfant restent vérifiés ; aucun secret ou droit nouveau
n’est déduit d’un rôle de maintenance. Le contenu d’une mission reste sous le
mandat humain de son appelant. Un project_id technique explicitement incompatible
n’est ni ignoré ni remplacé. Les chemins locaux hors Git n’exigent pas de miroir.
