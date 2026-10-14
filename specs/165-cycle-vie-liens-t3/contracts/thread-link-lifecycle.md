# Contrat interne — Cycle de vie d’un lien T3

## Entrée

Un résumé de fil validé à la frontière HTTP T3.

## Sortie

Une décision pure : le fil exige ou non un agent Bridget connecté.

## Préconditions

- Le résumé respecte le schéma V1 ou V2 déjà accepté.
- La filiation, si elle existe, vient du contrat T3. Elle n’est jamais déduite du titre.
- Une relation V2 présente accepte seulement `subagent`, `fork` ou `null`. Toute autre forme est invalide.

## Postconditions

- Un fil non vivant n’exige jamais de lien.
- Une racine V2, un fork ou un fil V1 vivant exige un lien.
- Un sous-agent vivant exige un lien pendant un état non terminal, une activité courante ou une obligation persistée.
- Une filiation absente conserve le comportement historique.

## Effets de bord

La décision pure n’a aucun effet de bord. La boucle appelante ouvre un lien manquant. Un lien existant reçoit toujours son dernier résumé avant sa fermeture.

## Drain

Le worker publie la clé du résumé seulement après un rafraîchissement réussi. Sa file, son journal, ses corrélations et ses réponses acquittables doivent être vides. Une remise efface cette clé dès sa lecture réseau, puis à sa mise en file. Une nouvelle entrée `pending` interdit la prochaine publication. La boucle ferme le lien si la clé encore publiée égale la clé terminale.

Le `Link` démarre `drain_since` à la première observation inéligible. Un changement de clé terminale ne repousse pas cette horloge. Seule une réactivation l'annule. `turn_wait` ferme le lien si une confirmation reste absente. L'état durable reste alors conservé. Cette tentative est consommée pour le processus courant afin d'éviter une boucle de réouverture.

Le premier snapshot réussi recherche les obligations par `load_thread_state(id)` sur les sous-agents inéligibles. Une obligation est un `pending` non vide ou un dernier tour connu mais absent de `ended_turns`. Un lien mort de façon imprévue déclenche la même lecture ciblée si sa tentative n'a pas été consommée.

La réconciliation rouvre au plus 32 liens de drain simultanément pour des obligations persistées. Elle conserve les obligations excédentaires pour les cycles suivants. Les liens déjà actifs qui deviennent terminaux ne comptent pas comme des réouvertures.

## Fermeture

La fermeture doit :

1. désinscrire l’agent ;
2. laisser le worker arrêter le relais et le journal, puis fermer son flux ;
3. joindre le worker ;
4. fermer le clone possédé par `Link` comme garantie après une panique ;
5. joindre le lecteur après cette fermeture ;
6. libérer le verrou de reçus ;
7. conserver les données sur disque.

## Compatibilité

Le contrat V1 ne fournit pas la filiation. Dans ce cas, un fil vivant reste exposé comme avant.

## Décision testable

Une fonction pure reçoit les faits du snapshot, du lien et des obligations. Elle retourne `ouvrir`, `rafraîchir`, `drainer`, `fermer` ou `ignorer`. Le test de fermeture utilise un `Link` construit à partir de handles et de sockets de test. Il ne dépend pas d'un daemon réel.

