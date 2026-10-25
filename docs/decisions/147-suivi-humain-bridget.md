# ADR147 — Suivi humain Bridget par invalidation sans contenu

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/147-suivi-humain-bridget.md#extrait-1).

## Contexte

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/147-suivi-humain-bridget.md#extrait-2).

## Décision

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/147-suivi-humain-bridget.md#extrait-3).

Inscrire avant ready, garder ready seq0 premier non coalescible, borner la file et remplacer un intervalle perdu par resync. Le wire ne garantit pas chaque rendu DOM : le runtime conserve dernier événement, readyGeneration et subscriptionId, dans un état volatil. Un visitId client local isole chaque visite ; aucun nouveau champ RPC/IPC.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/147-suivi-humain-bridget.md#extrait-4).

Mémoriser seulement le UUID choisi dans le store natif, par environment/project/conversation. Réconcilier la tête récente, les pages intermédiaires jusqu'à l'ancre ancienne si plus de50 nouveautés, puis le segment consulté sous un nouveau snapshot S commun avant publication atomique. Coût O(nouveautés + pages consultées), sans parcours de toute la base.

## Conséquences

Positives : aucun polling de contenu inchangé, aucune notification/réveil/modèle, autorité et transport existants conservés, choix retrouvé sans clic et mémoire durable limitée aux références.

Coûts : un petit contrat IPC/CLI/RPC nouveau, un registre volatil borné et des tests de ressources/reprise. Un changement pertinent relit la liste et le détail courant en groupe ; ce choix évite d'exposer davantage de références dans le signal.

Limites : délai de deux secondes en transport sain uniquement ; une panne conserve le choix mais n'atteste aucun accès frais. La consultation n'est ni prise en charge de mission, ni validation de verdict. L'ADR ne vaut pas preuve de production.

Convergence de conception approuvée par le principal pendant l'implémentation : coalescence ready côté atome, rupture CLI sous WebSocket sain et intervalle de plus de50 nouveautés sont des cas réellement observés. Les corrections et leurs preuves restent en cours ; cette ADR ne déclare pas leur achèvement.

Contrat et tests prévus : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/contracts/watch.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/tasks.md.

## Validation finale

Contrat et responsabilités réalisés, puis extensions US6–US9 approuvées et validées :31FR/20SC/9US et54/54 tâches après deux passes Converge et GO final du principal. UUID des actions de fils partagés normalisés ; montage MCP Claude/GLM garde opt-out et autorité ; ajout de membres par créateur sans réveil et lecture historique complète ; choix natif en haut et détails complets accessibles. Les correctifs historiques restent tracés, zéro finding actif.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/147-suivi-humain-bridget.md#extrait-5).
