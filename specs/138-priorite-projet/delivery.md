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
