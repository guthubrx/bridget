# ADR147 — Suivi humain Bridget par invalidation sans contenu

Date : 2026-10-08 ; convergence finale le 2026-10-09. Statut : Accepté, implémenté et validé. Sources fusionnées/poussées ; Bridget installé/actif. Livraison EN COURS : T3 package prêt, activation encore en attente du reçu runtime.

## Contexte

Le panneau T3 lit déjà les fils Bridget par un chemin humain autorisé. Il doit suivre leurs changements et garder le fil choisi par conversation. Les abonnements destinés aux agents peuvent les réveiller ; ils ne conviennent pas à cette consultation silencieuse.

## Décision

Ajouter la capacité human_thread_watch_v1 et un flux fermé version/generation/seq/status, sans contenu ni UUID de fil. Le daemon filtre les mutations réellement committées selon l'accès attesté. T3 réutilise son service lecteur, ses flux RPC et son supervisor pour relire les vues146 autorisées.

Inscrire avant ready, garder ready seq0 premier non coalescible, borner la file et remplacer un intervalle perdu par resync. Le wire ne garantit pas chaque rendu DOM : le runtime conserve dernier événement, readyGeneration et subscriptionId, dans un état volatil. Un visitId client local isole chaque visite ; aucun nouveau champ RPC/IPC.

Le supervisor existant reprend le WebSocket T3. Une rupture CLI/daemon pendant que le WebSocket reste sain demande une reprise distincte du seul stream technique : Schedule existant, au plus trois reprises après l'initiale, uniquement unavailable/command_failed/timeout. Chaque tentative refait autorité et handshake ; ready neuf revalide. En attente, corps masqués et UUID gardé ; plafond atteint → message et refresh manuel. Aucun retry de invalid_output/version/projet/binding/demande invalide. Aucun polling de corps ni journal durable nouveau.

Mémoriser seulement le UUID choisi dans le store natif, par environment/project/conversation. Réconcilier la tête récente, les pages intermédiaires jusqu'à l'ancre ancienne si plus de50 nouveautés, puis le segment consulté sous un nouveau snapshot S commun avant publication atomique. Coût O(nouveautés + pages consultées), sans parcours de toute la base.

## Conséquences

Positives : aucun polling de contenu inchangé, aucune notification/réveil/modèle, autorité et transport existants conservés, choix retrouvé sans clic et mémoire durable limitée aux références.

Coûts : un petit contrat IPC/CLI/RPC nouveau, un registre volatil borné et des tests de ressources/reprise. Un changement pertinent relit la liste et le détail courant en groupe ; ce choix évite d'exposer davantage de références dans le signal.

Limites : délai de deux secondes en transport sain uniquement ; une panne conserve le choix mais n'atteste aucun accès frais. La consultation n'est ni prise en charge de mission, ni validation de verdict. L'ADR ne vaut pas preuve de production.

Convergence de conception approuvée par le principal pendant l'implémentation : coalescence ready côté atome, rupture CLI sous WebSocket sain et intervalle de plus de50 nouveautés sont des cas réellement observés. Les corrections et leurs preuves restent en cours ; cette ADR ne déclare pas leur achèvement.

Contrat et tests prévus : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/contracts/watch.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/tasks.md.

## Validation finale

Contrat et responsabilités réalisés, puis extensions US6–US9 approuvées et validées :31FR/20SC/9US et54/54 tâches après deux passes Converge et GO final du principal. UUID des actions de fils partagés normalisés ; montage MCP Claude/GLM garde opt-out et autorité ; ajout de membres par créateur sans réveil et lecture historique complète ; choix natif en haut et détails complets accessibles. Les correctifs historiques restent tracés, zéro finding actif.

Tests/interop/recette native isolée et audits frais Rust/T3 validés, validateurs RC0/0erreur/0warning. Grades A et A99.86 limités à leurs deltas, pas au dépôt global. Les preuves SDK, autorité MCP, publication d'identité et DOM à RPC synthétique restent distinctes ; aucune chaîne complète avec modèle réel revendiquée. Aucun flux humain ne réveille un agent ni ne modifie les boucles de mission. NON installé et NON activé. Aucun commit/fusion/push/installation/restart effectué.
