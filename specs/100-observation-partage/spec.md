# Spécification 100 — Observation et partage inter-agents

Branche : session-100-observation-partage
Date : 2026-09-16
Statut : Livré — fusion main et release c680c6ce36c5 installée le 2026-09-16 ; preuves dans implementation.md
Tests : 23/23 ciblés100, suites de régression et smoke daemon réel verts ; suite intégrale non exécutée (harnais SIGKILL), détails dans implementation.md.
Dépendances : 089 (communication et journal), 092–093 (rejeu), 094 (parité CLI/MCP), 097–098 (producteurs), 099 (identité et remise bornée).

## Pourquoi et périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/100-observation-partage/spec.md#extrait-1).

## Scénarios utilisateur et tests

### US1 — Partager un extrait du journal (P1)

Lire les dernières entrées du journal d'un agent, ou reprendre depuis une
séquence, puis les envoyer directement à un autre agent pour contrôle. Le
destinataire reçoit le contenu, sa source et ses bornes ; une lacune ou une
coupe est explicite. Le partage ne copie pas arbitrairement des fichiers locaux.
Acceptation : journal fragmenté avec Unicode, fenêtre bornée, partage réellement
reçu, journal absent, délai dépassé et contenu incomplet signalés.

### US2 — S'abonner à un événement précis (P1)

« Préviens-moi quand Alpha rend la main », « quand il attend une permission »,
ou « quand un fichier correspondant à ce motif est écrit ». Choisir un agent,
un événement et éventuellement un fichier ; une seule occurrence ou plusieurs.
Lister et supprimer ses abonnements. Le demandeur reprend immédiatement son
travail ; la notification arrive par la messagerie ordinaire.
Acceptation : aucun événement ancien rejoué, bon filtre, une seule notification
en mode ponctuel, suppression/expiration effectives, identité de propriétaire
vérifiée, absence de notification pour un événement non observable.

### US3 — Signaler des écritures concurrentes (P2)

S'abonner aux collisions observées : deux agents distincts écrivent le même
fichier sur le même hôte dans une fenêtre de 30 secondes. Le message indique
les deux agents et le chemin. L'alerte décrit un risque, jamais un conflit Git
prouvé, et n'empêche ni écriture ni communication.
Acceptation : deux auteurs/même chemin déclenchent ; même auteur, deux chemins,
deux hôtes ou fenêtre dépassée ne déclenchent pas ; répétitions bornées.

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/100-observation-partage/spec.md#extrait-2).

## Entités

Extrait sourcé ; abonnement appartenant à un agent ; événement observable ;
écriture attribuée ; notification sans obligation de réponse.

## Critères de succès

- SC01 : un extrait de 50 entrées peut être lu et partagé en deux actions au plus ; fenêtre plus grande que la limite annoncée explicitement incomplète.
- SC02 : un abonnement créé est confirmé sans attendre son déclenchement ; essais locaux sous une seconde hors indisponibilité du pair.
- SC03 : zéro notification pour mauvais agent, lecture simple ou répétition d'une même écriture ; zéro confusion fin de tour/succès.
- SC04 : suppression et expiration empêchent tout nouveau déclenchement ; un abonnement ponctuel ne déclenche qu'une fois.
- SC05 : collision positive et les quatre témoins négatifs d'US3 sont automatisés ; les communications témoins restent possibles.
- SC06 : CLI, MCP, transport et événements sont testés, avec contrôle des erreurs et limites, puis formatage et analyse statique.

## Hypothèses explicites et limites

Abonnements attachés à l'agent, conservés lors de la fermeture du client CLI/MCP,
en mémoire pour la durée de vie du daemon ; expiration par défaut une heure,
maximum sept jours. Un redémarrage impose leur recréation et cette limite doit
figurer dans le reçu et la documentation. Pas de garantie de livraison durable
d'une notification à un agent absent. Ce choix n'affecte pas les réponses suivies.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/100-observation-partage/spec.md#extrait-3).
