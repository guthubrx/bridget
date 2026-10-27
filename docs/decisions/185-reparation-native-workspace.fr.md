# Décision 185 — Réparation explicite du dossier d'une tâche distante
Date : 27 octobre 2026.
Statut : implémentation en cours, non activée.

## Contexte
Un déplacement inter-volumes peut conserver les fichiers tout en supprimant le dossier auquel le programme vivant reste attaché. Un lien au chemin initial ne rétablit ni ce dossier courant ni ses attestations. La continuation ordinaire doit continuer à refuser.

## Décision
Ajouter une opération native distincte de la continuation. Le parent propriétaire demande une destination et un HEAD attendu sous une clé stable. Le maître vérifie propriété et autorité ; le worker établit l'ancre Git et la politique projetée ; le wrapper met au repos puis arrête son fournisseur sous borne et reprend le même fil natif attesté.

Conserver le canon de naissance, son placement et la flotte associée. Un reçu185 porte la liaison physique effective. Les lecteurs d'exécution et les rapports utilisent cette liaison uniquement après publication. Aucune réécriture des commandes historiques, aucun appel Start ou begin_round et aucun slot métier supplémentaire.

Le rapport Git ordinaire garde ses contrôles stricts. Le chemin externe n'est accepté que dans la réparation explicite, avec entrée administrative du dépôt de confiance, identités de dossiers relevées et relectures avant chaque effet. Git répare son lien ; aucun reset, checkout, déplacement ou nettoyage des fichiers métier.

Les droits cibles proviennent des mécanismes existants. Leur comparaison conserve toutes les restrictions, à la seule traduction des chemins du même worktree près. Une équivalence non prouvée refuse.

Le worker valide d'abord le résultat repris. Le maître enregistre ensuite sa décision de publication avant de demander au worker d'ouvrir les admissions. Le wrapper garde toute son entrée fournisseur fermée jusqu'à cette publication. Un rejeu du résultat durable n'effectue aucun nouvel arrêt, même après expiration de l'autorisation initiale.

## Garanties et limites
- La file, les demandes et les tours doivent être vides avant arrêt.
- Aucun second fournisseur si l'arrêt est incertain.
- La reprise ne crée aucun tour de modèle ; la continuation ultérieure reste distincte.
- Les refus sans effet et les opérations engagées restent distingués ; les clés et les canons sont conservés.
- Première version : une migration publiée par génération ; huit tentatives au plus, dont seules les tentatives refusées sans effet peuvent être remplacées par une nouvelle décision.
- Première version Linux : le pilote natif Codex est qualifié par les tests. Les adaptations sans preuve de reprise refusent. Aucun repli silencieux vers une conversation neuve.
- Un ancien worker ou wrapper ne reçoit pas une commande qu'il ne sait pas traiter.
- Sans inventaire antérieur, l'exhaustivité de la copie faite avant la réparation reste inconnue. L'opération conserve les fichiers présents.
- La migration d'un ancien wrapper déjà chargé exige un mécanisme admis distinct. L'installation d'un binaire seul ne suffit pas.

## Conséquences
Le produit offre un seul parcours de réparation contrôlé aux agents. Il n'autorise pas un changement libre de chemin ni une édition directe de ses bases. La qualification et l'activation restent nécessaires avant toute promesse d'usage réel.
