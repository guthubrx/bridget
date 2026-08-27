# Spécification 039 — Valider l'identité MCP

**Statut** : Prêt pour revue

**Base gelée du lot** : `90802b0377741b509f3743c5675544315b6f0f29`

**Objectif** : `3742f4a7-0def-4636-8a1c-e36dcb6a399a`

**Dependencies**: SPEC-006-identite-agent-robuste, SPEC-010-mcp,
SPEC-026-operations-greffe-central

## Problème mesuré

Le chemin MCP résout l'identité de l'appelant avec une grammaire privée alors
que l'enregistrement et le renommage utilisent la garde canonique du routeur.
Le canal MCP peut ainsi attribuer un message durable à un nom que le routeur
refuserait comme identité d'agent.

L'écart mesuré est strictement orienté : la grammaire canonique est incluse
dans la grammaire MCP, jamais l'inverse.

| Classe | MCP actuel | Garde canonique | Écart |
|---|---|---|---|
| Vide | Refus | Refus | Aucun |
| Lettres et chiffres ASCII | Acceptation | Acceptation | Aucun |
| `-` et `_` | Acceptation | Acceptation | Aucun |
| Autre ASCII | Refus | Refus | Aucun |
| Lettres Unicode | Acceptation | Refus | MCP seul |
| Nombres Unicode | Acceptation | Refus | MCP seul |
| Contrôles, formats, symboles et marques combinatoires | Refus | Refus | Aucun |

Les deux bornes de longueur portent sur 100 octets. La forme précomposée `é`
est donc acceptée par MCP, tandis que sa forme décomposée `e` + marque
combinatoire est refusée malgré un rendu visuel équivalent. Des homoglyphes
Unicode, dont `分析` et les lettres pleine largeur, franchissent également la
grammaire MCP privée.

La mesure du parc du 27 août 2026 à 11:16:34 UTC porte sur 15 entrées, dont
14 connectées. Les 15 noms visibles satisfont la garde canonique ; le
durcissement ne retire donc aucun nom réellement observé.

Le destinataire présente une propriété différente : il ne peut être résolu
que vers un agent déjà enregistré par la garde canonique. Un nom invalide
échoue fermé comme introuvable et n'entre pas dans le durable. Le domaine MCP
est uniquement un filtre de lecture. Ces deux champs restent hors du
correctif.

## Propriété

Toute identité d'expéditeur résolue par le canal MCP satisfait exactement la
même grammaire ASCII que l'enregistrement et le renommage, avant l'exécution
d'un outil et avant toute écriture durable. Une identité invalide ne déclenche
aucune commande ; une identité légitime conserve son comportement.

## Scénarios

### US1 — Refuser une identité Unicode avant l'effet

Un fichier d'identité portant `分析` ne permet pas l'exécution d'un outil MCP.
Le retour nomme l'absence d'identité valide et aucune commande n'atteint le
daemon ni le greffe.

### US2 — Conserver les agents légitimes

Un fichier d'identité portant `rc5-test` résout cet expéditeur et l'outil MCP
est effectivement exécuté une fois. Le contrôle prouve que le refus de US1
n'est pas produit par une façade qui bloque tous les appels.

### US3 — Conserver les refus bornés existants

Un destinataire invalide continue d'échouer à la résolution sans nouvelle
grammaire locale. Un filtre de domaine opaque continue de ne piloter qu'une
lecture.

## Exigences fonctionnelles

- **FR-3901** : la résolution MCP réutilise la garde canonique ; aucune copie
  de sa grammaire n'est conservée dans le daemon.
- **FR-3902** : une identité refusée par la garde canonique est refusée avant
  l'exécution de tout outil MCP.
- **FR-3903** : le refus d'identité ne produit aucun message, aucune
  inscription auxiliaire et aucune mutation du greffe.
- **FR-3904** : toute identité acceptée par la garde canonique reste acceptée
  par MCP sous la même borne de 100 octets.
- **FR-3905** : le destinataire et le filtre de domaine restent inchangés,
  car ils ne portent pas la même propriété durable.
- **FR-3906** : aucun fichier de CLI ou du daemon central n'est modifié par ce
  lot ; la portée reste la résolution et la façade MCP.
- **FR-3907** : les noms visibles au moment du durcissement sont inventoriés
  et tous doivent passer la garde avant livraison.

## Critères de succès

- **SC-3901** : l'oracle Unicode observe zéro exécution d'outil et un refus
  avant effet.
- **SC-3902** : le contrôle ASCII observe exactement une exécution et aucun
  refus d'identité.
- **SC-3903** : réintroduire la grammaire Unicode historique fait mourir
  SC-3901 à son assertion de non-exécution, tandis que SC-3902 reste vert.
- **SC-3904** : les 15 noms de l'inventaire daté passent la garde canonique,
  avec zéro exception.
- **SC-3905** : les tests du paquet daemon n'ajoutent aucun rouge par rapport
  à la base gelée.

## Hors périmètre explicite

- Ajouter une validation locale du destinataire ou modifier sa catégorie de
  refus.
- Fermer ou énumérer les domaines de lecture.
- Modifier le protocole, la CLI, le routeur ou les transitions du daemon.
- Réparer ou migrer la base privée Maicie, actuellement en schéma 19.
