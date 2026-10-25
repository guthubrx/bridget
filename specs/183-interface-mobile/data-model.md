# Modèle183

Réutilisation des agents, favoris, préférences par association, historique local, commandes et demandes humaines. Pas de table serveur créée.

Repère de lecture local : identité endpoint/device/agent, dernier événement effectivement affiché. Une ouverture du volet ou un simple rafraîchissement ne le déplace pas. Réinitialisation sur nouvelle association ; taille bornée par la limite d’agents locale existante.

Attention dérivée : question ouverte prioritaire ; activité en cours attestée ; nouveauté/terminal observé après le repère ; inconnu/ancien si observation insuffisante. Ne pas persister un booléen dérivé à côté des faits sans nécessité.

Préparation fichier : identité immuable du destinataire/association, référence locale et métadonnées bornées, sans réseau ; retrait avant premier essai. Après premier essai, enveloppe et clé restent immuables et passent par les reçus existants.

Compactage : agent/session/tour/item, statut running/completed, séquence native ; clos par fin corrélée ou terminal/perte de suivi. Pas d’état global partagé entre agents.

Aperçu : contenu borné256Kio, mode statique/interactif/code et instance de vue éphémère. Aucune session, préférence, permission ou référence native exposée à la page.
