# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/delivery.md](../../../../../../specs/138-priorite-projet/delivery.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le 2026-10-06, l'utilisateur autorise explicitement les commits, la fusion,
le push, la livraison sur cette machine et la mise à jour documentaire.
Cette autorisation complète le pipeline initial sans commit automatique.
Elle ne porte pas sur une nouvelle application T3 ni sur les fournisseurs.

<a id="extrait-2"></a>

## Extrait 2

L'arrêt du daemon termine ses groupes fournisseurs gérés. Il n'est donc pas
neutre pour une exécution gérée active. Le snapshot contient 28 connexions T3,
8 agents arrêtés non-T3 et zéro marqueur managed. Ce contrôle sera répété juste
avant l'arrêt. Seuls les labels com.bridget.daemon et com.bridget.t3 sont visés.
L'application T3 reste ouverte. Les plists sont conservés.

<a id="extrait-3"></a>

## Extrait 3

Bridget est fusionné par avance rapide et poussé sur `github/main` au commit
`7f6aba8527d2ee764e61613e3e579f54a290bf65`. Le daemon actif annonce le build
`7f6aba8527d2` et porte le PID 94364. Le pont T3 porte le PID 96700.
L'application T3 est restée ouverte. Les 28 fils et les 28 présences actives
restent disponibles, avec l'état connected ou busy. Les 28 projets de
communication sont attestés. L'annuaire local du projet Bridget contient un
agent et aucun membre d'un autre projet. La vue globale contient 36 agents,
dont huit arrêtés.

<a id="extrait-4"></a>

## Extrait 4

Les 519 fichiers de tâches et de résultats sont identiques octet pour octet
avant et après la bascule. Aucun verdict de mission n'est modifié. Le contrôle
avant arrêt confirme 28 présences T3, huit agents arrêtés et zéro marqueur
managed. Aucun tour géré actif n'a donc été observé pendant cette bascule.
Les arrêts utilisent SIGTERM et ciblent uniquement les labels autorisés.
Les plists ne sont pas modifiés. Leurs SHA256 sont :

<a id="extrait-5"></a>

## Extrait 5

- Daemon : `66c03f5a55a8fba6d2785bf7e1d33de229961c42fabfa051e45ef8b2124cf308`.
- Pont T3 : `f847a91684aa21db188390ee05a1a50f864c24a2380947c7a3ce11a9858ed915`.
