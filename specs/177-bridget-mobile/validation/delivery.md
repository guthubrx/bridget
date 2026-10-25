# Livraison mobile r10b — 24 octobre 2026

Statut : bilan historique r10b. Le complément courant r10c est dans r10c.md :
APK mis à jour, annuaire compact et détails des outils ; recette finale en attente
du déverrouillage du Pixel.
Le périmètre complet de la session 177 reste ouvert. Cette livraison ne déclare
ni la session terminée ni une validation globale de Bridget.

## Corrections regroupées

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/177-bridget-mobile/validation/delivery.md#extrait-1).

## Contrôles effectués

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/177-bridget-mobile/validation/delivery.md#extrait-2).

Une suite étendue antérieure r8 a donné 2231 PASS, 18 FAIL et 23 ignorés ; les
18 échecs ne sont pas classés comme hérités. Aucun succès global ni audit V15
complet n’est déduit des contrôles ciblés.

## Distribution et activité

Sources de départ communes : main `7ff749de884b6601f90f5ad6c28f2a879ec54bb1`.
Le présent commit contient les corrections et leur preuve partielle. Le manifeste
privé des sources relie les builds aux empreintes exactes de l’arbre de travail.
Les correctifs natifs intégrés par bridge1016 sont conservés.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/177-bridget-mobile/validation/delivery.md#extrait-3).

## Preuves et tâches conservées

Diaporama : `preuves-corrections/2026-10-24-mobile-conversation/preuves.html`.
Captures réelles de cette conversation et du clavier, antérieures à r10b.
Couverture partielle des clics. Les outils repliés de la version finale ne sont
pas présentés comme vérifiés sur le téléphone. Originaux et manifestes hors Git.

À terminer dès que le Pixel redevient disponible :

1. Installer l’APK signé par mise à jour sans effacer les données.
2. Vérifier les groupes d’outils, le dépliage, les noms et le suivi continu.
3. Vérifier un message normal et « Transmettre maintenant » avec le même reçu.
4. Vérifier la notification réelle en arrière-plan et l’ouverture authentifiée.
5. Qualifier Wi-Fi/4G, fermeture/reprise, fichiers et actions de tâches.
6. Qualifier le fonctionnement depuis le serveur avec Mac éteint puis rallumé.
7. Terminer la correspondance exigences/preuves et la distribution complète.

Aucune de ces tâches n’est remplacée par un test de composant.

## Nettoyage effectué

La commande native `installed` a contrôlé les installations byte-identiques.
Mac : 0,48 Gio logiques d’intermédiaires retirés ; certains fichiers protégés
restent conservés. Cartae : observation des processus incomplète, cache de build
conservé sans forcer sa suppression. Aucun nouveau cache de compilation copié.

Cinq anciennes copies installées de binaires non utilisés ont été retirées après
contrôle des processus et services (deux Mac, trois Linux). La version active
r10b, le retour r10a et les workers R8 en cours sont conservés. Les bases, preuves,
clés et exécutables encore chargés ne sont pas supprimés. Le dépôt Cartae ne
comporte plus de worktree mobile secondaire ; la branche courante reste main.
