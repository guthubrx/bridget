# Livraison locale SPEC138

## Autorisation et périmètre

Le 2026-10-06, l'utilisateur autorise explicitement les commits, la fusion,
le push, la livraison sur cette machine et la mise à jour documentaire.
Cette autorisation complète le pipeline initial sans commit automatique.
Elle ne porte pas sur une nouvelle application T3 ni sur les fournisseurs.

## Préparation vérifiée

- Bridget : main propre à e8ed4d62, synchronisé avec github/main.
- Agent Loop : commit 8008b08ea077569478e1737557c1f2da7a0a33dd,
  fusion rapide et push origin/main confirmés.
- La copie dotfiles main-merge conserve son index étranger : SHA256
  6588fe2ba33bdec78a96fa2cb48d52a419df3c1e3f8ab31347030aeead9521d5,
  identique avant et après fusion. Aucun changement de ces travaux n'est inclus.
- La copie active /Users/moi/dotfiles reste sur 104-bridge-role-auth.
  Le push seul n'y installe pas Agent Loop. Une copie ciblée reste à faire.

Sauvegarde privée :
/Users/moi/.cache/bridget-adoptions/spec138-20261006.qJc9jg

Elle contient le binaire précédent, une sauvegarde SQLite cohérente, les quatre
plists concernés et les entrées Agent Loop précédentes. quick_check : ok.
Snapshot : 9467 entrées ledger, 434 entrées de fils, 250 demandes suivies.
Binaire précédent SHA256 :
02cfdf77ad5b6971572e328bae551dbc1028e74a993abc39778a5263611211e6.

## Validation de publication

Python frais : 152 tests PASS, 0 FAIL. Le code reste identique au gel validé.
Le premier workspace frais a échoué : 1033 PASS, 3 FAIL dans la bibliothèque,
avec permissions temporaires non privées et conflit sur HOME entre tests.
Les trois échecs sont conservés comme faits, pas renommés RED de SPEC138.
La relance utilise umask 077 et --test-threads=1. Résultat final : exit0,
1633 PASS, 0 FAIL, 55 ignored, 78 résumés externes. Les trois résumés enfants
filtrés sont exclus du total. Aucun source ni test n'est changé pour ce passage.
Capture : specs/138-priorite-projet/evidence/publication-workspace-serial.log.
Les deux validations de skills fraîches sont PASS. git diff --cached --check
est PASS avec conservation des captures brutes.

Les captures brutes gardent leurs octets, dont les séquences terminal et les
espaces. .gitattributes limite l'exception whitespace aux captures de recette
SPEC138. Le contrôle du code et des documents reste inchangé.

## Documentation

Six documents publics sont actualisés : README français et anglais,
CHANGELOG, référence de communication, installation et référence des commandes.
Ils couvrent les contrats 133 à 138. Les anciennes entrées restent conservées.
La section Non publié ne change pas la version 0.1.3 et ne prouve pas un
déploiement. Aucun résultat des 29 scénarios Gherkin non exécutés n'est inventé.

## Sécurité de bascule

L'arrêt du daemon termine ses groupes fournisseurs gérés. Il n'est donc pas
neutre pour une exécution gérée active. Le snapshot contient 28 connexions T3,
8 agents arrêtés non-T3 et zéro marqueur managed. Ce contrôle sera répété juste
avant l'arrêt. Seuls les labels com.bridget.daemon et com.bridget.t3 sont visés.
L'application T3 reste ouverte. Les plists sont conservés.

Les anciennes missions sans project_root de communication restent inconnues.
Leur domaine ou domain_context n'est pas transformé en preuve implicitement.
Leurs UUID mandatés et leurs rappels restent compatibles. Aucune mission,
preuve, décision de suite ou résultat n'est déclaré terminé par cette livraison.

Statut à ce point : préparation, pas encore de livraison production.

## Livraison vérifiée après bascule

Les sections précédentes décrivent la préparation. La livraison locale est
maintenant achevée sur autorisation explicite de l'utilisateur.

Bridget est fusionné par avance rapide et poussé sur `github/main` au commit
`7f6aba8527d2ee764e61613e3e579f54a290bf65`. Le daemon actif annonce le build
`7f6aba8527d2` et porte le PID 94364. Le pont T3 porte le PID 96700.
L'application T3 est restée ouverte. Les 28 fils et les 28 présences actives
restent disponibles, avec l'état connected ou busy. Les 28 projets de
communication sont attestés. L'annuaire local du projet Bridget contient un
agent et aucun membre d'un autre projet. La vue globale contient 36 agents,
dont huit arrêtés.

Le binaire installé a le SHA256
`ec6667adc68e109e74a1fe028a472b3f9ec429a1b9fc965ebf645b187a500db2`.
La compilation release après commit est PASS en 54,26 s. Le contrôle clippy
frais est PASS en 12,76 s. Le formatage est PASS. Les 152 tests Python installés
sont PASS en 1,259 s. La recette isolée avec CLI, moteur canonique et daemon
réel est PASS : un test en 1,66 s. Ces répétitions ne s'ajoutent pas aux comptes
de validation déjà publiés.

Agent Loop est fusionné et poussé sur `main` au commit `8008b08e`.
La copie active dans /Users/moi/dotfiles reçoit les quatre fichiers publiés,
octet pour octet, au commit `bf0983b649bb7c9726b395a545c4cb7d3aec44e8`.
Cette copie reste sur la branche `104-bridge-role-auth`. Cette branche n'est
ni fusionnée ni poussée, car elle contient d'autres travaux. Le code SPEC138
est bien publié sur main. Les travaux étrangers de l'index main-merge restent
préservés, avec l'empreinte 6588fe2ba33bdec78a96fa2cb48d52a419df3c1e3f8ab31347030aeead9521d5.
Dans la racine Bridget, les changements déjà indexés et les changements non
indexés restent préservés. Leurs empreintes respectives sont
`1923c77ad2130ed3c9818f85fcebb58d0d974063b3353a9715534d11c001be67` et
`ddf55123868e8fdefe04c50537f1df1a40a6290f2da7872807a418905b260ee8`.

### Données et processus conservés

La sauvegarde privée indiquée plus haut passe `quick_check`. La comparaison
SQLite ne trouve aucun corps de message modifié dans le ledger. Les 434
entrées de fils sont présentes et inchangées. Aucun message non expiré ne
manque. Huit messages du 29 septembre, datés de 19:27:15 à 19:47:31 UTC,
ont été supprimés par la purge existante de sept jours au démarrage. Cette
purge fonctionne aussi chaque heure. Ces huit messages restent conservés dans
le snapshot de 9467 entrées. Le ledger complet n'est donc pas déclaré identique
octet pour octet.

Les 519 fichiers de tâches et de résultats sont identiques octet pour octet
avant et après la bascule. Aucun verdict de mission n'est modifié. Le contrôle
avant arrêt confirme 28 présences T3, huit agents arrêtés et zéro marqueur
managed. Aucun tour géré actif n'a donc été observé pendant cette bascule.
Les arrêts utilisent SIGTERM et ciblent uniquement les labels autorisés.
Les plists ne sont pas modifiés. Leurs SHA256 sont :

- Daemon : `66c03f5a55a8fba6d2785bf7e1d33de229961c42fabfa051e45ef8b2124cf308`.
- Pont T3 : `f847a91684aa21db188390ee05a1a50f864c24a2380947c7a3ce11a9858ed915`.

Les deux LaunchAgents Agent Loop sont rechargés après leur pause. Depuis la
réactivation, Politique effectue neuf passages et Psychologie dix passages.
Le dernier code de sortie de chaque job est zéro. Leurs sorties d'erreur sont
inchangées depuis le 5 octobre : elles contiennent des erreurs anciennes,
sans nouvelle erreur observée.

### Compatibilité et limites

Le wrapper Psychologie ancien reçoit une correction d'une ligne pour consulter
explicitement la vue globale et résoudre le seul UUID ROOT déjà mandaté.
Sa source reste dans Documents, hors du dépôt Bridget. Un patch portable est
conservé dans les preuves. Trois tests ont échoué avant correction et passent
après correction. Le principal les rejoue avec succès. Le lookup réel trouve
ROOT connected. Le dry-run réel constate `sent=0`, une anomalie et
`closed=false`. Aucune mission n'est clôturée. Aucun rappel réseau de production
n'est revendiqué : la recette n'impose aucun envoi.

Les nouvelles boucles utilisaient déjà la consultation adaptée. Les anciens
runs sans `project_root` restent UNKNOWN. Leur `domain_context` n'est pas
promu en preuve de projet. Les sources Rust et le moteur canonique restent
inchangés après les validations. Le nettoyage des trois worktrees propres et
fusionnés reste en attente de confirmation de l'utilisateur.
