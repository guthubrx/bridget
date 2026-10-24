# Aide à la reprise des refus de base source

Le commit d’une mission était présent sur Cartae mais absent du dépôt source du
parent Mac. Sa récupération explicite a permis l’admission avec la même clé.
Ce cas contextuel ne démontre aucun défaut de résolution multihôte. Le défaut
produit est le manque de consignes pour retrouver une base exacte sans doublon.

Le delta enrichit les refus existants dans detail.message/detail.recovery et la
description MCP de workspace.base_ref. Les contrôles Git, droits, codes, reçus
et transitions ne changent pas. La documentation EN/FR distingue commit absent,
base différente, rejeu identique et demande modifiée. Aucun fetch automatique,
reset, nouvelle mission ou modification d’un dépôt applicatif.

Référence T3 lue : GitVcsDriverCore.ts, resolveCommit (résolution locale),
fetchPullRequestHeadCommit (fetch explicite avant résolution), fetchRemoteBranch.
Mécanisme réutilisé : séparer disponibilité locale et récupération explicite.
Différence nécessaire : Bridget vérifie la source du parent avant une exécution
sur une autre machine et conserve une identité de demande durable. La création
ou remise à jour forcée d’une branche locale de T3 n’est pas reprise.

Revue indépendante : Task 4a962220-a485-4709-a7de-224ab5537d68 lue et acquittée.
Deux ambiguïtés corrigées : issue après refus durable et absence de réouverture
au rejeu. La proposition « refus répété = durable et sans tâche » n’est pas
retenue : cette déduction ne serait pas sûre. Le contrôle source V2 précède la
réservation du reçu (admit_auto_routing), contrairement au test V1 cité par la
revue. Cela explique le fetch puis rejeu réussi de l’incident sans changer de clé.

Premier lot ciblé : 3 tests base, 1 projection MCP et 1 schéma MCP réussis.
Contrôle documentaire : 27 pages, 422 liens locaux, 11 paires de langues.
Rejeu après clarification : 3 tests base et 1 test de projection MCP réussis ;
le test du schéma (1 PASS) était inchangé par cette clarification. Aucun test
ignoré. Les tests socket utilisent une réponse simulée ; ils vérifient le
rendu de detail, pas une nouvelle admission métier. Aucune suite complète.

Livraison : source 41ade47a7d6061d75d4d0df67b4827e8e33e7d5f poussée sur main.
Binaire macOS optimisé et signé ; SHA-256
28b4db9d1b0699d9d0e5c0da5006699b238723ccd4b2e773c6344d7ef1f92a95.
Daemon actif confirmé par le MCP officiel : build 41ade47a7d60.
Une reconnexion MCP transitoire pendant activation, puis accès rétabli.
Les processus T3 et worker distant sont conservés. Les missions distantes
encore actives gardent leur instance et leur état working sans erreur.
Une mission est devenue result_available sans erreur entre les observations.
Aucune mission métier créée ou rejouée pour cette validation.
Conseils transmis au parent demandeur par Bridget ; lecture non confirmée.
Entretien du cache effectué par le wrapper, sauvegarde précédente conservée.
