# Modèle de données 177

## Autorités et faits existants réutilisés

- Machine : identité et connexion worker existantes.
- Projet : CommunicationProject et mapping canonique existants.
- Agent : identité opaque existante ; affichage nom/machine/projet ne sert pas d'adresse.
- Commande et reçu : enveloppe et reçu natifs. Le mobile stocke leur référence, pas un second résultat métier.
- Tâche et événement : projections du moteur natif ; aucun état de tâche dupliqué.
- Question : HumanInboxItemFrame et décisions existantes.

## Appareil autorisé, nouveau

device_id opaque, label borné, created_at, revoked_at, allowed_projects, allowed_machines, allowed_operations, credential_hash, credential_expires_at.
Le secret brut n'est jamais stocké sur le serveur. Les listes de portée sont explicites ; une valeur absente n'implique pas tous les projets.
Le téléphone protège sa clé avec le stockage de clés Android. La révocation est relue pour chaque accès.

## Association, nouvelle

pairing_id opaque, credential_hash, expires_at, consumed_at, scope fixé par l'humain, device_binding après consommation.
Expiration maximale de cinq minutes et usage unique atomique. La perte de la première réponse ne permet pas une seconde association sous la même clé pour un autre appareil.

## État local Android, nouveau

endpoint HTTPS, device_id, credential protégé, filtre machine/projet, favoris, brouillons, file d'envoi et reçus.
Commande : identifiant opaque, date d'émission, enveloppe figée, état local, référence du reçu, erreur structurée.
Transitions : draft → queued_local → submitted → acknowledged / refused / outcome_unknown → reconciled.
L'enveloppe est durable avant la transition submitted. Aucun état pending n'est présenté comme reçu serveur.

## Notification, nouvelle projection minimale

Référence opaque d'objet, catégorie, date ; aucun contenu privé dans le payload distant.
Les préférences sont par appareil et projet. L'objet est relu avec les droits actuels lors de l'ouverture.

## Décision utilisateur — 24 octobre 2026

Première livraison : piloter les agents existants et leurs tâches. La création directe de nouveaux agents est reportée comme extension. L'envoi à un agent existant reste possible ; cet agent applique ses droits de délégation. Le téléphone ne reçoit aucun droit de lancement direct par anticipation.
