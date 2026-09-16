# Spécification 100 — Observation et partage inter-agents

Branche : session-100-observation-partage
Date : 2026-09-16
Statut : Livré — fusion main et release c680c6ce36c5 installée le 2026-09-16 ; preuves dans implementation.md
Tests : 23/23 ciblés100, suites de régression et smoke daemon réel verts ; suite intégrale non exécutée (harnais SIGKILL), détails dans implementation.md.
Dépendances : 089 (communication et journal), 092–093 (rejeu), 094 (parité CLI/MCP), 097–098 (producteurs), 099 (identité et remise bornée).

## Pourquoi et périmètre

Bridget est le produit de communication inter-agents/inter-fournisseurs ; T3 est
un adaptateur, Maicie n'est pas requis. L'utilisateur veut partager un passage
utile pour une relecture, être averti d'un fait précis et repérer les écritures
concurrentes. Aucun profil, formulaire, mandat, approbation métier, verrou de
fichier ou condition préalable supplémentaire au travail des agents.

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

- FR01 : lecture et partage bornés, disponibles en CLI et MCP, sans dépendance T3/Maicie.
- FR02 : provenance agent/séquences, ordre conservé, Unicode valide, lacunes et limites explicites ; aucun résumé inventé.
- FR03 : partage par la messagerie existante, compatible avec demande de réponse (`reply`) ; aucun nouveau suivi métier.
- FR04 : catalogue fermé et expliqué : fin de tour, permission requise, écriture observée, collision observée ; filtres agent et fichier pertinents combinables.
- FR05 : création, liste, suppression, mode ponctuel et expiration ; seuls les faits reçus par le daemon après l'abonnement sont concernés, sans rejeu du journal. Un fait déjà produit mais encore en transit peut être reçu après l'abonnement ; aucune synchronisation des horloges source n'est promise.
- FR06 : propriétaire dérivé de l'identité autorisée ; aucune création ou suppression pour un tiers par simple déclaration.
- FR07 : notifications ordinaires, non bloquantes pour l'agent observé, sans boucle de notifications ni attente active d'un LLM.
- FR08 : fin de tour ne signifie ni mission terminée ni succès ; silence et déconnexion ne sont pas une preuve de fin.
- FR09 : collisions fondées uniquement sur écritures structurées attribuées, même hôte et même chemin absolu normalisé, auteurs distincts et fenêtre de 30 secondes.
- FR10 : lectures et outils non écrivains exclus ; couverture limitée aux signaux des intégrations, absence de surveillance universelle du disque ou d'interprétation de shell libre.
- FR11 : mémoire, taille des extraits, abonnements et débit bornés ; saturation visible et aucun ralentissement non borné de la messagerie.
- FR12 : documenter la portée locale des observations, les intégrations couvertes, les pertes possibles et la durée de vie des abonnements.
- FR13 : tests isolés seulement ; préserver modifications 099 et demandes/réponses existantes. L'interdiction initiale de commit/déploiement est levée par la demande explicite du 2026-09-16 : fusion locale puis adoption sauvegardée et vérifiée, sans arrêter les conversations fournisseur.

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

Les collisions se consultent par le même mécanisme d'abonnement, sans alerte
globale imposée. Le mode shell libre et les adaptateurs ne remontant pas les
chemins ne sont pas couverts ; aucune prétention de couverture T3 complète.
Les filtres portent sur les faits du catalogue, pas sur une condition libre
comme « le code est bon », pas de SQL ou de script utilisateur exécuté.
