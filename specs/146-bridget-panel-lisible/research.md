# Recherche locale — SPEC146

Date : 2026-10-08. État : conception consolidée. Aucun test n'est revendiqué dans ce document initial.

## Décisions de conception

### Opérations récentes distinctes

Décision : ajouter `list_recent` et `history_recent`, négociées par `HumanThreadViewRecentV1`.

Pourquoi : les opérations145 ont déjà un sens et des lecteurs installés. Changer silencieusement leur ordre ou leurs curseurs pourrait rompre ces lecteurs. Deux variantes fermées rendent l'incompatibilité explicite et évitent des champs optionnels ambigus.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/research.md#extrait-1).

### Liste évolutive, historique figé

Décision : trier globalement la liste à chaque page et figer la borne de séquence de chaque historique.

Pourquoi : le dernier échange d'un fil peut évoluer entre deux lectures. Promettre un instantané de liste sans le stocker serait faux. Un historique dispose d'une borne naturelle de séquence et peut donc être paginé de manière stable sans nouvelle table.

Alternative rejetée : promettre un instantané de liste complet sans preuve de stockage ou inventer un journal supplémentaire pour cette amélioration visuelle.

### Aperçu sans réécriture

Décision : limiter visuellement les corps longs et conserver l'original pour affichage développé, copie et recherche.

Pourquoi : la lisibilité doit progresser sans perdre une preuve ni changer le sens d'un message. Le dépliage et les détails utilisent les commandes natives du panneau, sans résumé généré.

### Style adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/research.md#extrait-2).

Pourquoi : l'utilisateur souhaite une meilleure organisation, pas un nouveau langage graphique. Les règles Cartae Next.js/Axios ne correspondent pas à ce projet. Les libellés français suivent la présentation145.

## Constats reçus des lectures de l'existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/research.md#extrait-3).

Le stockage actuel145 trie les fils par UUID ASC et l'historique par séquence ASC. Sources : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/store/threads.rs:1179` et même fichier1431. Le helper de lecture partagé est vers651 ; la projection des corrections vers666 doit conserver la borne globale de l'instantané lors d'une page récente.

Le dispatch de projection existe dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/threads.rs:111`. Actions, résultats et capacité existent dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-transport/src/protocol.rs:2166`,2274 et189. Les deux gardes humaines qui évitent la maintenance, vers11782/11789, et la négociation11709 sont dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/daemon.rs`. Elles doivent intégrer le singletonRecent sans ouvrir le rôle agent.

Les flags CLI existants sont confirmés dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/cli.rs:994`,1045 et1076 : le sélecteur humain de fil est `--thread`.

Le manifeste du socle importé est `/Users/moi/.cache/bridget-workspace146.nnyH61/baseline145-source.json`. Il identifie31 sources importées. Son existence et ses hashes décrivent un héritage145, pas une implémentation146. Le principal contrôle cette séparation avant livraison.

## Références externes consultées par le principal

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/research.md#extrait-4).

Le maintien d'un focus perceptible et d'un parcours clavier cohérent suit [web.dev, Focus](https://web.dev/learn/accessibility/focus). Ces deux références éclairent les garanties ; elles ne remplacent pas les tests du composant réel et ne constituent pas un dogme de hauteur ou de couleur.

## Limites

Aucun catalogue externe ou appel de fournisseur payant n'est requis pour ces décisions locales. Aucun audit d'un autre projet n'est sollicité. Les résultats145 conservés sont des références de socle, pas des preuves146.
