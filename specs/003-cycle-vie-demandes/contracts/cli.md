# Contrat CLI : demandes suivies

## Créer une demande suivie

```text
bridget send --to <agent> --reply [--timeout <secondes>] <message>
```

Retour : l'identifiant de demande est affiché dans l'acquittement.

## Annuler

```text
bridget cancel <id> [--reason <texte>]
```

Résultats : `cancelled` si la demande ouverte appartient à l'agent courant; `cancelled` à nouveau si elle était déjà annulée par ce même agent; erreur actionnable sinon.

## Consulter

```text
bridget requests
bridget requests --json
```

Affiche uniquement les demandes de l'agent courant, avec identifiant, destinataire, état et délai restant ou date terminale.

## Répondre

```text
bridget reply <message>
```

La CLI associe la réponse à la dernière demande suivie reçue par l'agent. Une réponse explicite peut porter l'identifiant de la demande lorsque nécessaire.
