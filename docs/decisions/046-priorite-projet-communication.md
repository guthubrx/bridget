# ADR 046 — Priorité au projet, exception volontaire

Date: 2026-10-06
Statut: Proposé
Spécification: 138-priorite-projet

## Contexte

L'annuaire global permet de choisir un agent extérieur sans rendre le changement
de projet visible. Le domaine est mutable. L'ancien registre runtime est retiré.
Les fils silencieux restent lisibles par tous leurs membres. Les boucles ont
besoin de conserver les rappels d'un mandat extérieur déjà explicite.

## Décision

Attacher un fait de projet à la connexion attestée. Utiliser l'hôte et la racine
commune Git, ou la racine T3 attestée. Conserver l'inconnu si la preuve manque.
Garder ListAgents global et ajouter une vue de communication locale par défaut.

Un motif structuré valide exprime le choix interprojets. Une garde commune le
vérifie avant dépôt et notification. L'avertissement revient à l'émetteur hors
du corps du message. Aucune seconde confirmation humaine obligatoire.

Chaque opération de fil porte un motif borné par ses membres immuables. Aucun
consentement persistant n'est ajouté. Celui d'une réponse reste lié à sa demande
suivie OPEN. Agent Loop conserve les mandats par agents/rôles déjà
attribués. Aucun recrutement extérieur automatique, même si les locaux sont busy.
La lecture historique et les opérations déjà acceptées ne sont pas rejouées.

## Conséquences positives

Une règle observable remplace les décisions implicites. Les agents locaux sont
faciles à trouver. Les collaborations extérieures voulues restent possibles.
Les connexions auxiliaires et les reçus existants sont réutilisés. Aucun nouveau
registre, package ou service n'est requis.

## Conséquences négatives et limites

Un client ancien inconnu reste capable d'envoyer avec avertissement. Cette règle
ne garantit donc pas une isolation de sécurité. Le fait vivant doit être réannoncé
après reconnexion. Un champ structuré et une variante d'annuaire ajoutent
un coût de maintenance limité. Les canons et négociations doivent rester compatibles.

## Alternatives écartées

Comparer les domaines crée des faux locaux. Réactiver le registre retiré ajoute
une autorité hors besoin. Interdire les projets différents bloque des mandats
valides. Une confirmation humaine à chaque tick empêche les relances prévues.
