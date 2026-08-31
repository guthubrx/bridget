# Modèle de données - SPEC-070

La SPEC ne crée ni table ni événement de protocole.

## Etat dérivé client

| Nom | Source | Durée de vie |
|---|---|---|
| `activeTurn` | dernier tour non terminal projeté depuis le journal | rendu courant du fil |
| `activity` | dernier texte, acte, permission ou raisonnement du `activeTurn` | rendu courant du fil |
| `focusMessageId` | cible agent plus message de notification | navigation courante |
| `notificationPermission` | API navigateur | lecture à chaque geste utilisateur |
| `notifiedTerminalIds` | mémoire bornée du client | session de page |

`activeTurn` et `activity` sont calculés à partir du journal. Ils ne sont pas
persistés, afin qu'une reprise reconstruise exactement le même état.

## Invariants

- Une activité est affichable seulement si son événement vient du fournisseur.
- Une activité est rattachée à un tour et à son `message_id` si celui-ci est
  disponible.
- Un terminal rend le tour inactif.
- Une notification terminale possède une cible `agent + message_id`.
- Une même cible ne produit au plus qu'une notification par session de page.

## Etats visibles

| Etat | Preuve | Affichage |
|---|---|---|
| Aucun | aucun événement fournisseur | aucune promesse de travail |
| Actif | texte, acte, permission ou raisonnement | bouille + ligne atténuée |
| Terminé | `turn_end` | réponse finale normale |
| Échec | `error` terminal | erreur liée au message, détail dépliable |

## Confidentialité

Le résumé de notification et la ligne d'activité utilisent un libellé projeté,
pas la charge brute d'un outil ni une commande complète.
