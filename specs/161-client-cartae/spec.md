# SPEC-161 — Client Bridget à jour sur Cartae

**Statut :** Implemented — développement14/14 ; pipeline In Progress
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

1. Étant donné une préparation encore incomplète, quand une erreur survient, alors la commande courante reste inchangée.
2. Étant donné une préparation validée, quand la commande courante est remplacée, alors chaque nouvelle invocation trouve soit l'ancien client complet, soit le nouveau client complet.
3. Étant donné une communication non conforme après bascule, quand le retour arrière s'applique, alors la référence précédente est rétablie et le résultat est signalé comme non livré.
4. Étant donné les sessions actuelles, quand la livraison se termine, alors aucun redémarrage de T3, du maître, du pont ou de la liaison n'a eu lieu et aucun nouveau daemon Cartae n'existe.

### US3 — Disposer d'une livraison traçable et propre (P2)

Comme mainteneur, je veux distinguer les sources du client, l'outil de livraison et les preuves afin de pouvoir comprendre ou annuler la mise à jour.

**Test indépendant :** relire le reçu durable et vérifier ses empreintes, les références Git distantes et les ressources conservées ou retirées.

1. Étant donné une livraison réussie, quand je consulte son reçu, alors je retrouve les identités distinctes des sources compilées, de l'outil exécuté et des documents intégrés.
2. Étant donné la fin du lot, quand le nettoyage est terminé, alors les ressources inutiles propres au lot sont retirées ; les preuves, le client de repli et les travaux étrangers restent conservés.

### Cas limites

Préfixe déjà occupé ; outil absent ; compilation échouée ; transfert incomplet ; sources modifiées ; ancienne référence modifiée par un tiers ; cible symbolique inattendue ; panne avant ou après bascule ; tunnel indisponible ; retour arrière impossible ; nettoyage d'une ressource encore utilisée. Un résultat ambigu empêche l'annonce de livraison et n'autorise pas une mutation plus large.

## Exigences fonctionnelles

- **FR-001** : installer un client issu exactement de la source 160 approuvée ; distinguer cette identité de celle de l'outil de livraison.
- **FR-002** : vérifier les prérequis et l'intégrité des sources avant la compilation ; refuser toute différence.
- **FR-003** : ne télécharger ni outil ni dépendance implicitement ; la copie explicitement tracée de dépendances déjà présentes sur le Mac et vérifiées contre le verrou approuvé est permise, sans changer ce verrou. Une préparation impossible laisse le client courant intact.
- **FR-004** : borner la compilation à deux travaux parallèles et à une priorité réduite ; une seule compilation du lot active à la fois.
- **FR-005** : préparer la nouvelle livraison dans un emplacement neuf et privé sans écraser une ressource existante.
- **FR-006** : vérifier l'identité, l'architecture et les droits du nouveau client avant son activation ; vérifier sa communication avec le maître existant.
- **FR-007** : remplacer la référence courante de façon atomique après validation et seulement si elle est toujours celle relevée au départ.
- **FR-008** : conserver et vérifier le retour arrière vers le client précédent ; restaurer cette référence si la recette après bascule échoue et si la référence courante nous appartient encore.
- **FR-009** : préserver sans redémarrage T3, le maître, son pont et la fédération ; ne pas démarrer de daemon Cartae ; aucune édition directe, migration ou suppression de base. Les inscriptions et messages synthétiques envoyés par la recette de communication sont autorisés par le périmètre validé et doivent être identifiés dans le reçu.
- **FR-010** : vérifier status et l'annuaire via le binaire candidat, puis un échange synthétique bidirectionnel via la recette transport existante, sans mission ni appel à un fournisseur de modèle. La recette transport ne prouve pas l'exécution du binaire.
- **FR-011** : livrer les changements du lot dans la branche cible et vérifier la référence distante après envoi.
- **FR-012** : produire un reçu durable distinguant Git, fichier installé et communication réellement observée ; ne pas présenter cette livraison comme une exécution distante de mission.
- **FR-013** : nettoyer uniquement les ressources inutiles appartenant au lot, après transfert et vérification des preuves ; justifier toute conservation.
- **FR-014** : conserver les refus de sécurité déjà présents dans l'outil de livraison et leurs vérifications, notamment les chemins privés et les hôtes connus.

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

Exécuteur distant, sélection de machine, choix de moteur, serveur de décision, changement d'interface T3, migration de maître, réplication de données, création de tunnel et redémarrage des services. Ces besoins restent dans les lots 162/163. Aucun test global Rust répété pour un changement limité au script de livraison et à ses tests.

## État après développement

Les14tâches de développement et préparation sont vérifiées. SC006 dispose des preuves ciblées dans validation/tests161.md : harness unique30PASS/1FAIL historique, correctiondiagnostiquée puis8rejeuxaffectésPASS, sans nouveauFULL. SC001–SC005 restent ouverts aux preuves réellesG8/G9 ; aucune activation ou livraison déduite du statutImplemented. Converge sera exécuté parROOT après figement des tâches.
