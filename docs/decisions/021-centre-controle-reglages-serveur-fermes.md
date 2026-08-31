# ADR-021 - Centre de contrôle à réglages serveur fermés

## Statut

Accepté - 2026-08-31

## Contexte

L'opérateur souhaite régler chaque serveur enregistré depuis Bridget. Le relais distant est déjà accessible seulement par tunnel SSH approuvé, mais transformer cette connexion en terminal ou en éditeur de configuration libre annulerait les limites de Bridget.

## Décision

Le centre de contrôle expose un catalogue de réglages typés et fermé. Toute écriture est prévisualisée, confirmée localement, comparée à une génération attendue, appliquée atomiquement et suivie d'un reçu. Les valeurs de projet, secrets, profils, approbations, commandes et maintenance système ne rejoignent pas ce catalogue sans un contrat spécialisé.

## Conséquences

### Positives

- Une fiche serveur est utile sans devenir un accès administratif général.
- Les concurrents et les échecs sont explicites, reproductibles et auditables.
- La première clé réutilise la politique de racines existante au lieu de créer un nouveau fichier de configuration parallèle.

### Négatives

- Ajouter un réglage demande une évolution complète et testée du catalogue.
- Certaines options très attendues restent lecture seule tant que leur autorité de persistance n'est pas sûre.
- Les coûts restent des estimations et peuvent être indisponibles faute de tarif ou de modèle attesté.
