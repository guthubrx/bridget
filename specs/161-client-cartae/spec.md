# SPEC-161 — Client Bridget à jour sur Cartae

**Statut :** Implemented — développement14/14 ; livraison opérationnelle achevée, clôture Git documentaire en cours
**Priorité :** P1
**Périmètre :** livraison du client distant Cartae
**Tâches :** 14/14 — développement et préparation ; livraison8/9 obligatoire distincte
**Date :** 2026-10-13
**Branche :** session-161-client-cartae

## Problème

Cartae utilise encore le client 096 alors que le maître Bridget du Mac utilise la version 160. Le tunnel fonctionne déjà. L'utilisateur veut disposer des changements livrés depuis Cartae sans interrompre ses sessions ni créer une seconde autorité.

## Solution

Mettre à jour uniquement le client Cartae vers la source approuvée de la version 160. Conserver le maître actuel, les données, les droits et la liaison existante. Vérifier la communication avant d'annoncer la livraison. Garder un retour arrière immédiat et nettoyer les ressources devenues inutiles.

## Scénarios utilisateur et tests

### US1 — Utiliser le client à jour (P1)

Comme utilisateur de Cartae, je veux que ma commande Bridget utilise la même version source que le maître, afin d'accéder aux fonctions déjà livrées.

**Test indépendant :** vérifier la provenance du client installé, puis lire le statut du maître depuis Cartae.

1. Étant donné le client 096 et le maître 160, quand la préparation réussit et les contrôles sont conformes, alors la commande habituelle utilise le nouveau client et rejoint le même maître.
2. Étant donné un cache de compilation incomplet, quand la préparation échoue, alors l'ancien client reste utilisable et aucune installation réseau n'est lancée implicitement.
3. Étant donné une copie dont les sources diffèrent de la référence approuvée, quand leur contrôle échoue, alors aucun binaire n'est compilé sous l'identité de la version approuvée.

### US2 — Préserver la continuité et revenir en arrière (P1)

Comme utilisateur ayant des conversations actives, je veux garder les services et les données en place pendant la mise à jour.

**Test indépendant :** comparer les identités des processus protégés et les références du client avant/après ; exercer le retour arrière sans arrêter le maître.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/161-client-cartae/spec.md#extrait-1).

### US3 — Disposer d'une livraison traçable et propre (P2)

Comme mainteneur, je veux distinguer les sources du client, l'outil de livraison et les preuves afin de pouvoir comprendre ou annuler la mise à jour.

**Test indépendant :** relire le reçu durable et vérifier ses empreintes, les références Git distantes et les ressources conservées ou retirées.

1. Étant donné une livraison réussie, quand je consulte son reçu, alors je retrouve les identités distinctes des sources compilées, de l'outil exécuté et des documents intégrés.
2. Étant donné la fin du lot, quand le nettoyage est terminé, alors les ressources inutiles propres au lot sont retirées ; les preuves, le client de repli et les travaux étrangers restent conservés.

### Cas limites

Préfixe déjà occupé ; outil absent ; compilation échouée ; transfert incomplet ; sources modifiées ; ancienne référence modifiée par un tiers ; cible symbolique inattendue ; panne avant ou après bascule ; tunnel indisponible ; retour arrière impossible ; nettoyage d'une ressource encore utilisée. Un résultat ambigu empêche l'annonce de livraison et n'autorise pas une mutation plus large.

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/161-client-cartae/spec.md#extrait-2).

## Entités principales

- **Livraison cliente** : référence source approuvée, cible préparée et identité du binaire.
- **Référence active** : lien utilisé par les commandes Cartae, avec cible précédente et nouvelle cible.
- **Reçu de livraison** : preuves horodatées des contrôles, de l'intégration et du nettoyage ; aucun secret.

## Critères de succès

- **SC-001** : la provenance du client activé atteste la même identité source que la version 160 attendue, et aucune divergence des sources compilées n'est constatée.
- **SC-002** : tous les contrôles de communication prévus réussissent depuis Cartae avec le maître existant, dont un échange dans chaque sens.
- **SC-003** : zéro redémarrage des processus protégés, zéro nouveau daemon Cartae et zéro édition directe ou migration de base provoqués par le déploiement ; les écritures normales de la recette (deux agents synthétiques et deux messages) restent identifiées séparément.
- **SC-004** : les échecs avant bascule laissent la référence actuelle inchangée ; un retour arrière contrôlé rétablit la référence précédente sans interruption du maître.
- **SC-005** : l'outil, les sources, le binaire et la cible Git distante ont chacun une preuve de provenance vérifiable ; chaque candidat au nettoyage a une décision documentée.
- **SC-006** : les tests ciblés du déploiement passent ; aucun succès global Rust n'est déduit de ces contrôles.

## Hypothèses et dépendances

- Maintien du maître Mac et de la fédération existante, autorisé par le plan utilisateur.
- Source binaire approuvée : e6812e3e7b2d051d630192857cc3f440180d51fa. Métadonnées antérieures : 1663cb61301e69ccf7294a567f01266be44558e7.
- Specs 095/096 : client et fédération ; spec 160 : source livrée de référence.
- Les outils Linux existent selon les lectures du 13 octobre. Leur cache complet reste à vérifier ; aucun succès de compilation n'est présumé.

## Hors périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/161-client-cartae/spec.md#extrait-3).

## État après développement

Les14tâches de développement et préparation sont vérifiées. SC006 dispose des preuves ciblées dans validation/tests161.md : harness unique30PASS/1FAIL historique, correctiondiagnostiquée puis8rejeuxaffectésPASS, sans nouveauFULL. SC001–SC005 restent ouverts aux preuves réellesG8/G9 ; aucune activation ou livraison déduite du statutImplemented. Converge sera exécuté parROOT après figement des tâches.

## Livraison opérationnelle attestée

Le13octobre2026, client161 activé surCartae depuis08:59:38CEST, mêmesourcedebinaire160 quelemaître, sansrestart. LespreuvesréellesG8 et décisionsG9 sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/161-client-cartae/validation/livraison161.md. La période décrite plushaut comme «SC001–SC005 ouverts» esthistorique ; leurs observations sont désormais consignées. Restent àcommitter/pousser les présentesmétadonnées ; aucune preuvefuture git-final161.json n'est présumée.
