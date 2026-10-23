# Livraison ciblée de l’audit 174

Date : 23 octobre 2026.

## Livraison

Source du binaire : `b8857dfbaa56e36af0641409a5806a030d8b671a`.
Build `174-b8857dfb`, version 0.4.1, profil release optimisé.
Sources publiées sur GitHub/main, maître Mac et worker Cartae actifs.
T3 et le pont existant n’ont pas été redémarrés.
Les binaires de retour arrière sont conservés.

- SHA-256 macOS : `8daadfea43516a2517a6e166af472dba6b00fb8bd0601e354d39fa92cdb55d12`.
- SHA-256 Linux : `3bfcd6a0cbab6cbf0acc322649f8635f3e677d6cf7ad013df3ea7b2c7165fe58`.

## Neuf défauts corrigés

1. Plafond porté à 128 avec une constante partagée et une réservation contrôlée.
2. Sorties ignorées par Git conservées pendant le nettoyage.
3. Inventaire incomplet des processus traité comme une impossibilité de nettoyer.
4. Référence Git absente distinguée d’une erreur technique de Git.
5. Message et conseil de reprise transmis au client MCP.
6. Reçu de préparation libéré après un échec de lancement sans effet, avec preuve d’absence.
7. Fichiers ouverts depuis un autre répertoire pris en compte avant retrait.
8. Confirmation de démarrage distante incertaine expliquée sans suggérer une nouvelle mission.
9. Configuration d’espace de noms invalide distinguée d’une identité absente.

Le champ de tentative d’envoi ne prouve pas la réception de la mission.
Un résultat tardif reste consultable sur la même tâche. Le correctif ne change
ni les délais, ni les droits, ni les relances, ni les transitions durables.

## Validation et limites

85 tests ciblés réussis : 26 nettoyage Python, 33 préparation Python, 26 Rust.
Aucune suite complète. Les builds optimisés ont réussi sur les deux plateformes.
La revue indépendante couvre le message de démarrage et la garde de nettoyage.
Ses réserves de libellé ont été corrigées puis testées par le responsable.
La correction finale d’identité n’a pas de seconde revue indépendante.

Deux anciens tests système lsof restent non validés sur Cartae, où des avertissements
de systèmes de fichiers empêchent un inventaire complet. Le nettoyage de production
reste alors refusé. Le nouvel essai avec un fichier réellement ouvert passe ;
il ne prouve pas l’absence de toute course entre observation et suppression.

Le rapport V15 est structurellement valide, mais sa conclusion globale est
`inconclusive`, avec `release_blocked=true` pour le périmètre d’audit complet.
Les correctifs ciblés sont livrés selon l’autorisation de livraison rapide.
Neuf contrôles communs sur quatorze sont vérifiés ; 31 critères sur 42 sont notés.
Aucune note globale de sécurité n’est attribuée.

Onze candidats restent à qualifier : sept hypothèses de code et quatre paquets
signalés par les avis de dépendances. Leur présence ne prouve pas une exploitation.
Les règles statiques exécutées ne couvrent pas Rust. Le scanner de dépendances
standard n’est pas qualifié ; une interrogation ciblée de Cargo.lock a rendu les alertes.
Les lenteurs des boucles de maintenance restent à mesurer.

## Nettoyage et refus de base

Le plafond n’est pas global à la machine. Aucun réglage par projet n’a été ajouté.
L’inventaire et le plan de nettoyage sont conservés hors Git ; aucune suppression
de production n’a été effectuée. Les branches, sorties et tâches actives sont protégées.

Un dépôt Git sans premier commit reste sans base résoluble, même en demande
partagée par défaut. Ce cas ne relève ni du plafond ni de fichiers sales.
Aucun commit automatique du dossier personnel ni changement d’enveloppe n’a été fait.

Les preuves détaillées et les reçus opérationnels restent privés. Les responsables
des incidents ont reçu un avis de livraison ; leur lecture n’est pas présumée.
